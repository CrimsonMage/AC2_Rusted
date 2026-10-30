//! The HUD's session in the viewer: the client logged on to the Legions server double in process
//! (no socket), so the gameplay HUD has a player to ask. The double gives the starter items, a
//! worn shield and sword, two quests and the player's login state (skills, options, money), and the HUD's
//! frameworks run against the client every frame as in `bin/ac2`'s world frame: the inventory, the
//! paper doll, the skills, the shortcut bar and the toolbar's panels and menus all work. The
//! viewer still walks and draws its own player; the HUD's parts of the frame (inventory, vitals,
//! progress, selection, calendar) and what the player holds come from the session.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{Client, ClientFrame, Command, DatWorld, FrameInput};
use ac2_db::ResourceStore;
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent};
use ac2_world::Ac2WorldSource;
use legions_server_double::{addr, double, Spawn, State, SERVER};

use crate::walk::Landing;

/// Frames the login may take before the session gives up.
const LOGIN_FRAMES: usize = 600;
/// The double's clock step while logging on.
const LOGIN_STEP: f64 = 1.0 / 64.0;

pub struct Session {
    client: Client,
    server: Rc<RefCell<State>>,
    /// The commands the HUD asked for, sent with the next frame.
    commands: Vec<Command>,
}

impl Session {
    /// Logs on at `landing` (over the block's highest ground, as the HUD tests do) and gives the
    /// starter inventory; an error when the script library or the login fails (the viewer then
    /// runs the HUD without a session).
    pub fn start(
        store: &Arc<ResourceStore>,
        world: &Arc<Ac2WorldSource>,
        landing: Landing,
    ) -> Result<Self, String> {
        let cell = landing.cell();
        let top = world
            .load_landblock(ac2_core::CellId::new(cell))
            .ok()
            .flatten()
            .map_or(0.0, |block| {
                block.height_grid().into_iter().fold(0.0, f32::max)
            });
        let (transport, server) = double(Spawn {
            cell,
            origin: [landing.x, landing.y, top + 2.0],
            rotation: [1.0, 0.0, 0.0, 0.0],
        });
        let config = OnlineConfig {
            server: addr(SERVER),
            account: "viewer".into(),
            credential: Credential::Account,
            version: ac2_net::LEGIONS_VERSION.into(),
            character: CharacterChoice::First,
            server_key: None,
            language: 0,
            dat_iterations: Vec::new(),
        };
        let mut session = OnlineSession::with_transport(config, Box::new(transport))
            .map_err(|e| e.to_string())?;
        let script_data =
            ac2_script::data::DatScriptData::new(Arc::clone(store.dataset()), (**store).clone());
        session.set_string_tables(ac2_client::string_table_ids(&script_data));
        let dat_world = DatWorld::with_world(Arc::clone(store), Arc::clone(world))?
            .with_script_data(script_data.clone());
        let mut client = Client::new(session, Arc::new(dat_world));
        let wlib =
            ac2_script::load_wlib(store, ac2_core::DataId::new(ac2_script::LEGIONS_WLIB_DID))
                .map_err(|e| e.to_string())?
                .ok_or("no script library in the DATs")?;
        let program = Arc::new(ac2_script::vm::Program::from_stream(&wlib.stream));
        script_data.preload_tables();
        script_data.preload_states();
        script_data.preload_skill_states(&program);
        script_data.set_cache_only(true);
        client.attach_scripts(
            ac2_script::host::ScriptHost::new(program, Box::new(script_data)),
            ac2_client::SCRIPT_VM_FUNCTIONS,
        );
        let mut frames = 0;
        while !client.frame().in_world {
            client.settle_loads(Duration::from_secs(30))?;
            client.use_time(&FrameInput::default())?;
            server.borrow_mut().elapsed += LOGIN_STEP;
            frames += 1;
            if frames > LOGIN_FRAMES {
                return Err("the double's login did not enter the world".into());
            }
        }
        {
            let mut server = server.borrow_mut();
            server.give_starter_items();
            server.give_starter_gear();
            server.give_starter_quests();
            server.send_init_player();
        }
        Ok(Self {
            client,
            server,
            commands: Vec::new(),
        })
    }

    /// One frame of the session `seconds` after the last: the client's frame with the HUD's
    /// commands, then the HUD's frameworks over `frame` (the viewer's) with the session's parts
    /// filled in. Returns whether the player confirmed the quit question.
    pub fn frame(&mut self, view: &mut ClientView, frame: &mut ClientFrame, seconds: f64) -> bool {
        for request in view.take_chat_requests() {
            if let ac2_ui::ChatRequest::Line(line) = request {
                self.commands.push(Command::ChatLine(line));
            }
        }
        let input = FrameInput {
            commands: std::mem::take(&mut self.commands),
            ..FrameInput::default()
        };
        let events = match self.client.use_time(&input) {
            Ok(events) => events,
            Err(error) => {
                eprintln!("world-viewer: session: {error}");
                Vec::new()
            }
        };
        self.server.borrow_mut().elapsed += seconds;
        for event in &events {
            if let SessionEvent::Notice(GameplayNotice::DisplayStringInfo { kind, text }) = event {
                view.chat_display_info(*kind, text);
            }
        }
        let session = self.client.frame();
        // The HUD's player is the session's: the viewer's walker takes its id, and with no walker
        // (the fly camera) the session's own player stands in, not drawn.
        match (frame.player.as_mut(), session.player.as_ref()) {
            (Some(player), Some(own)) => player.id = own.id,
            (None, Some(own)) => {
                frame.player = Some(ac2_client::PlayerFrame {
                    hidden: true,
                    ..own.clone()
                });
            }
            _ => {}
        }
        // What the player holds (the starter sword and shield) hangs from the walker's hands:
        // the walker carries the session player's id, the holder these name.
        let player_id = session.player.as_ref().map(|p| p.id);
        frame.objects.extend(
            session
                .objects
                .iter()
                .filter(|o| o.holder.as_ref().map(|h| h.id) == player_id && player_id.is_some())
                .cloned(),
        );
        frame.selection = session.selection;
        frame.player_vitals = session.player_vitals;
        frame.inventory = session.inventory;
        frame.progress = session.progress;
        frame.date_time = session.date_time;
        let notices = self.client.take_script_notices();
        let asked = view.frameworks(&mut self.client, frame, &events, &notices);
        self.commands.extend(asked);
        view.take_quit_request()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use legions_server_double::{
        POTIONS, PRIMARY_HAND, SECONDARY_HAND, SHIELD, STARTER_QUEST, SWORD,
    };

    /// The session on the real DATs (`AC2_DAT_DIR`) under a headless view: the HUD's inventory
    /// lists the starter items and the paper doll shows the worn sword and shield.
    #[test]
    fn the_session_gives_the_hud_its_inventory_and_worn_gear() {
        let Some(dataset) = ac2_dat::test_support::env_dataset() else {
            return;
        };
        let Ok(gpu) = ac2_gpu::Gpu::headless() else {
            return;
        };
        let store = Arc::new(ResourceStore::new(dataset));
        let world = Ac2WorldSource::open(Arc::clone(&store)).unwrap().unwrap();
        let landing = Landing {
            block: (0xCB, 0x4D),
            x: 96.0,
            y: 96.0,
        };
        let mut session = Session::start(&store, &world, landing).expect("the session starts");
        let mut view = ClientView::new(world, &gpu, ac2_gpu::COLOR_FORMAT, 1, (1024, 768));
        let step = |session: &mut Session, view: &mut ClientView| {
            let mut frame = ClientFrame {
                in_world: true,
                cell_id: Some(landing.cell()),
                ..ClientFrame::default()
            };
            assert!(!session.frame(view, &mut frame, 1.0 / 64.0));
            view.update(&gpu, &frame, Duration::ZERO);
        };
        // The HUD builds on its workers; then the inventory panel shows, as its button does.
        let started = std::time::Instant::now();
        while view.shortcut_slots() == 0 {
            assert!(
                started.elapsed() < Duration::from_secs(60),
                "the HUD is built"
            );
            step(&mut session, &mut view);
            std::thread::sleep(Duration::from_millis(5));
        }
        view.show_panel(ac2_ui::ToolbarChild::Inventory, true);
        for _ in 0..32 {
            step(&mut session, &mut view);
        }
        let items: Vec<u64> = view
            .backpack_items()
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        assert!(
            items.contains(&POTIONS),
            "the backpack lists the potions: {items:?}"
        );
        assert_eq!(view.paper_doll_item(PRIMARY_HAND), Some(SWORD));
        assert_eq!(view.paper_doll_item(SECONDARY_HAND), Some(SHIELD));
        // The sword and the shield hang from the player's hands in the merged frame.
        let mut frame = ClientFrame {
            in_world: true,
            cell_id: Some(landing.cell()),
            ..ClientFrame::default()
        };
        session.frame(&mut view, &mut frame, 1.0 / 64.0);
        let player = frame.player.as_ref().expect("the player").id;
        let held: Vec<(u64, u32)> = frame
            .objects
            .iter()
            .filter_map(|o| Some((o.id, o.holder.as_ref().filter(|h| h.id == player)?.location)))
            .collect();
        assert!(
            held.contains(&(SWORD, legions_server_double::RIGHT_HAND_HOLD)),
            "{held:?}"
        );
        assert!(
            held.contains(&(SHIELD, legions_server_double::LEFT_HAND_HOLD)),
            "{held:?}"
        );
        let (underway, finished) = view.quest_ids().expect("the quest panel");
        assert_eq!(
            (underway, finished),
            (vec![STARTER_QUEST], vec![STARTER_QUEST + 1])
        );
    }
}
