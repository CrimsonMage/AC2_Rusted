//! The client against a running Pyrealium server, driven as the window drives it: a fresh account and
//! character made through the real character creation screens, then the gameplay HUD's
//! frameworks every frame (`bin/ac2/main.rs`'s `world_frame`), with the server's own clock and
//! rules. Runs only with `PYREALIUM_LIVE` set to the server's address and `AC2_DAT_DIR` naming the
//! 7230 DATs; needs a GPU adapter. `scripts/pyrealium-live-test.sh` starts a server of its own for
//! the suite (its own data, ports and developer account, `PYREALIUM_ACCOUNT`), so a player's server
//! and characters are never touched.
//!
//! Each rig makes a character with a name no other rig uses, logs it off when dropped (so no
//! body is left in the world) and deletes it (an account holds five).

#![allow(dead_code)]
#![cfg(feature = "window")]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ac2_client::view::ClientView;
use ac2_client::{
    pregame_event, pregame_requests, Client, Command, DatWorld, FrameInput, ScriptCharGen,
};
use ac2_gpu::{Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_ui::pregame::{
    load_pregame, PregameUi, MODE_AVATAR_MODIFICATION, MODE_AVATAR_SELECTION,
    MODE_CHARACTER_SELECTION, MODE_GAMEPLAY,
};
use ac2_ui::{tokenize, Modifiers, SelectionUi, ToolbarChild, UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent};

pub mod chain;
pub mod craft;
pub mod trade;
mod walk;

pub const SIZE: (u32, u32) = (1024, 768);
pub const LEFT: u32 = 0x4000_0004;
pub const RIGHT: u32 = 0x4000_0005;
/// The frame's step, as the window's: the server keeps its own time.
const FRAME: Duration = Duration::from_millis(16);

/// The server's address, when live tests are asked for.
pub fn server() -> Option<SocketAddr> {
    let text = std::env::var("PYREALIUM_LIVE").ok()?;
    Some(
        text.parse()
            .unwrap_or_else(|e| panic!("PYREALIUM_LIVE={text}: {e}")),
    )
}

/// The server's developer account (`pyrealium serve --account`), `local` by default.
fn account() -> String {
    std::env::var("PYREALIUM_ACCOUNT").unwrap_or_else(|_| "local".into())
}

/// `name` with a letter suffix no other rig of this run uses (names take letters only).
fn unique_name(name: &str) -> String {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let mut n =
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + std::process::id() % 10_000 * 64;
    let mut suffix = String::new();
    for _ in 0..4 {
        suffix.push((b'a' + (n % 26) as u8) as char);
        n /= 26;
    }
    format!("{name}{suffix}")
}

fn find(ui: &PregameUi, name: &str) -> Option<usize> {
    let id = tokenize(name);
    let tree = ui.tree();
    tree.roots().iter().find_map(|&r| {
        (tree.get(r).unwrap().id() == id)
            .then_some(r)
            .or_else(|| tree.child_recursive(r, id))
    })
}

fn pregame_click(ui: &mut PregameUi, name: &str) {
    let element = find(ui, name).unwrap_or_else(|| panic!("no {name}"));
    let b = ui.tree().screen_box(element);
    ui.input(&[
        (UI_POINTER_X, ((b.x0 + b.x1) / 2) as f32, true),
        (UI_POINTER_Y, ((b.y0 + b.y1) / 2) as f32, true),
    ]);
    ui.input(&[(LEFT, 1.0, true), (LEFT, 1.0, false)]);
}

pub struct LiveRig {
    pub client: Client,
    pub view: ClientView,
    pub gpu: Gpu,
    pub account: String,
    pub name: String,
    pending: Vec<Command>,
    /// Input actions held down every frame (movement keys), as the window's key state sends.
    pub held: Vec<ac2_weenie::ActiveAction>,
    /// The commands the HUD's frameworks asked for, in order.
    pub asked: Vec<Command>,
    /// `gmSelectionUI`, fed the way `bin/ac2/main.rs` feeds it (a click in the world selects).
    selection: SelectionUi,
    /// Every session event the frames saw, in order.
    pub seen: Vec<SessionEvent>,
    pub world: Arc<ac2_world::Ac2WorldSource>,
    store: Arc<ac2_db::ResourceStore>,
    started: Instant,
    /// Whether each frame first waits for the loader threads (see [`Self::new_unsettled`]).
    settle: bool,
}

impl LiveRig {
    /// A new character named `name` (letters only) on a fresh account, made through avatar
    /// selection (the first avatar) and avatar modification, entered into the world, the HUD
    /// built. `None` without `PYREALIUM_LIVE`, `AC2_DAT_DIR` or a GPU adapter.
    pub fn new(name: &str) -> Option<Self> {
        Self::with_loads(name, true, None)
    }

    /// [`Self::new`] with a silent sound manager attached before the login, as the window
    /// attaches its own: the objects' FX then start their particle systems (`FXManager`'s
    /// particle part runs with the sound device), which each frame hands the view.
    pub fn new_with_sound(name: &str) -> Option<Self> {
        Self::with_loads(name, true, Some(Box::new(ac2_audio::NullSink)))
    }

    /// [`Self::new`] with a sound manager on `sink` attached before the login, as
    /// `bin/ac2/main.rs` attaches the speakers' (with the portal and rain sounds).
    pub fn new_with_audio(name: &str, sink: Box<dyn ac2_audio::SoundSink>) -> Option<Self> {
        Self::with_loads(name, true, Some(sink))
    }

    /// [`Self::new`] with the frames not waiting for the loader threads (`settle` false): the
    /// window's frame never does (`Client::settle_loads` is for tests), so a DAT read finishes
    /// some frames after what asked for it.
    pub fn new_unsettled(name: &str) -> Option<Self> {
        Self::with_loads(name, false, None)
    }

    fn with_loads(
        name: &str,
        settle: bool,
        audio: Option<Box<dyn ac2_audio::SoundSink>>,
    ) -> Option<Self> {
        let server = server()?;
        let name = &unique_name(name);
        let dataset = ac2_dat::test_support::env_dataset()?;
        let Ok(gpu) = Gpu::headless() else {
            eprintln!("no GPU adapter; skipping");
            return None;
        };
        let store = Arc::new(ac2_db::ResourceStore::new(Arc::clone(&dataset)));
        let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
        let world = Arc::clone(dat_world.world());
        let account = account();
        let config = OnlineConfig {
            server,
            account: account.clone(),
            credential: Credential::Account,
            // The logon's data version (Pyrealium takes 7148 and 7230).
            version: std::env::var("PYREALIUM_LIVE_VERSION")
                .unwrap_or_else(|_| ac2_net::default_version().into()),
            character: CharacterChoice::Wait,
            server_key: None,
            language: 0,
            dat_iterations: ac2_weenie::dat_iterations(&dataset).unwrap(),
        };
        let mut session = OnlineSession::connect(config).unwrap();
        let script_data = ac2_script::data::DatScriptData::new(dataset, (*store).clone());
        session.set_string_tables(ac2_client::string_table_ids(&script_data));
        session.set_chat_commands(ac2_client::chat_commands(&script_data));
        let dat_world = dat_world.with_script_data(script_data.clone());
        let mut client = Client::new(session, Arc::new(dat_world));
        client.set_window_positions_dir(Some(std::env::temp_dir().join(&account)));
        let wlib =
            ac2_script::load_wlib(&store, ac2_core::DataId::new(ac2_script::LEGIONS_WLIB_DID))
                .unwrap()
                .expect("the script library");
        let program = Arc::new(ac2_script::vm::Program::from_stream(&wlib.stream));
        script_data.preload_tables();
        script_data.preload_states();
        script_data.preload_skill_states(&program);
        script_data.set_cache_only(true);
        client.set_load_budget(Duration::from_secs(30));
        if let Some(sink) = audio {
            client.attach_audio(ac2_client::sound_manager(Arc::clone(&store), sink));
            client.set_portal_sound_ids(ac2_client::portal_sound_ids(&store));
            client.set_rain_sound_ids(ac2_client::rain_sound_ids(&store));
        }
        client.set_fx_scripts(ac2_anim::load_fx_scripts(&store));
        // As `bin/ac2/main.rs` does, before the connection is used: the weenies' scripts must
        // hear the login (`WM_Player` 0x27), or the vitals and the HUD never get their notices.
        client.attach_scripts(
            ac2_script::host::ScriptHost::new(Arc::clone(&program), Box::new(script_data.clone())),
            ac2_client::SCRIPT_VM_FUNCTIONS,
        );

        // Character creation on the real screens, against the server.
        let chargen =
            ac2_script::host::ScriptHost::new(Arc::clone(&program), Box::new(script_data.clone()));
        let mut ui = PregameUi::new(load_pregame(&store).unwrap(), 1024, 768);
        ui.skip_intro();
        ui.set_char_gen_system(Box::new(ScriptCharGen::new(chargen)));
        let mut t = 0.0;
        let pregame = |client: &mut Client, ui: &mut PregameUi, t: &mut f64| {
            if settle {
                client.settle_loads(Duration::from_secs(30)).unwrap();
            }
            for event in client.use_time(&FrameInput::default()).unwrap() {
                pregame_event(ui, &event);
            }
            ui.use_time(*t);
            pregame_requests(ui, client.session_mut());
            *t += FRAME.as_secs_f64();
            std::thread::sleep(FRAME);
        };
        let deadline = Instant::now() + Duration::from_secs(60);
        while ui.mode() != MODE_CHARACTER_SELECTION {
            pregame(&mut client, &mut ui, &mut t);
            assert!(Instant::now() < deadline, "no character selection");
        }
        pregame_click(&mut ui, "CreateButton");
        pregame(&mut client, &mut ui, &mut t);
        assert_eq!(ui.mode(), MODE_AVATAR_SELECTION);
        let viewport = find(&ui, "AvatarSelectionViewport").unwrap();
        let b = ui.tree().screen_box(viewport);
        ui.input(&[
            (UI_POINTER_X, ((b.x0 + b.x1) / 2) as f32, true),
            (UI_POINTER_Y, ((b.y0 + b.y1) / 2) as f32, true),
        ]);
        ui.finish_pick(Some(0));
        ui.input(&[(LEFT, 1.0, true), (LEFT, 1.0, false)]);
        ui.finish_animation();
        pregame(&mut client, &mut ui, &mut t);
        assert_eq!(ui.mode(), MODE_AVATAR_MODIFICATION);
        pregame_click(&mut ui, "NextButton");
        pregame_click(&mut ui, "NameEntry");
        let typed: Vec<(u16, u32)> = name.encode_utf16().map(|c| (c, 0)).collect();
        ui.text_input(&[], &typed, Modifiers::default());
        pregame_click(&mut ui, "EnterButton");
        let deadline = Instant::now() + Duration::from_secs(60);
        while !client.frame().in_world || ui.mode() != MODE_GAMEPLAY {
            pregame(&mut client, &mut ui, &mut t);
            assert!(
                Instant::now() < deadline,
                "no world entry (mode {:#x})",
                ui.mode()
            );
        }
        let view = ClientView::new(Arc::clone(&world), &gpu, COLOR_FORMAT, 1, SIZE);
        let mut rig = Self {
            client,
            view,
            gpu,
            account,
            name: name.to_owned(),
            pending: Vec::new(),
            held: Vec::new(),
            asked: Vec::new(),
            selection: SelectionUi::default(),
            seen: Vec::new(),
            world,
            store,
            started: Instant::now(),
            settle,
        };
        rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
        Some(rig)
    }

    /// One frame: the UI `actions` into the HUD, the client's frame with the commands the
    /// frameworks asked for last frame, then the frameworks and the view; the server answers in
    /// its own time.
    pub fn frame(&mut self, actions: &[(u32, f32, bool)]) -> Vec<SessionEvent> {
        for message in self.view.ui_input(actions, Modifiers::default()) {
            self.selection.message(&message);
        }
        let input = FrameInput {
            actions: self.held.clone(),
            commands: std::mem::take(&mut self.pending),
            ..FrameInput::default()
        };
        if self.settle {
            self.client.settle_loads(Duration::from_secs(5)).unwrap();
        }
        let events = self.client.use_time(&input).unwrap();
        // As the window does: the particle systems the objects' FX started and stopped.
        let starts = self.client.take_object_particle_starts();
        let stops = self.client.take_object_particle_stops();
        self.view.set_object_particles(&starts, &stops);
        // The lights, streaks, repulsors and decals too (`bin/ac2/main.rs`).
        let lights = self.client.take_object_light_starts();
        let gone = self.client.take_object_lights_gone();
        self.view.set_object_lights(&lights, &gone);
        let streaks = self.client.take_object_streak_calls();
        self.view
            .set_object_streaks(&streaks, self.client.streaks_enabled());
        let repulsors = self.client.take_object_repulsor_calls();
        self.view.set_object_repulsors(&repulsors);
        let decals = self.client.take_object_decal_calls();
        self.view.set_object_decals(&decals);
        let frame = self.client.frame();
        let notices = self.client.take_script_notices();
        let asked = self
            .view
            .frameworks(&mut self.client, &frame, &events, &notices);
        self.asked.extend(asked.iter().cloned());
        self.pending.extend(asked);
        // The selection render: the object under the pointer, selected by
        // `gmSelectionUI::Finish3DSelection` (the window's `world_frame` does the same).
        if self.selection.wants_pick() {
            let potential = self.view.pick_list(&frame, SIZE, self.view.pointer());
            let (targets, player_type, player_id) =
                ac2_client::pick_targets(&self.client, &frame, &potential);
            if let Some(outcome) = self.selection.finish(&targets, player_type, player_id) {
                self.pending.push(Command::Select(outcome.select));
                self.pending.push(Command::LookAtSelection(outcome.select));
                if outcome.use_selection {
                    self.pending.push(Command::UseSelection);
                }
                if outcome.open_examination {
                    self.view.open_examination();
                }
            }
        }
        self.view.update(&self.gpu, &frame, Duration::ZERO);
        self.gpu.flush_writes();
        self.seen.extend(events.iter().cloned());
        std::thread::sleep(FRAME);
        events
    }

    /// Whether the UI image `did` reads and decodes (what the HUD's image loader does for an icon).
    pub fn ui_image_decodes(&self, did: u32) -> bool {
        ac2_assets::load_ac2_ui_surface(&self.store, ac2_core::DataId::new(did))
            .ok()
            .flatten()
            .is_some()
    }

    /// `VisualDesc::GetTotalIconDesc` of the DAT visual `did`: its icon layers' images.
    pub fn total_icon_images(&self, did: u32) -> Vec<u32> {
        ac2_assets::ac2_total_icon_desc(&self.store, ac2_core::DataId::new(did), None)
            .ok()
            .flatten()
            .map_or_else(Vec::new, |d| d.layers.iter().map(|l| l.image).collect())
    }

    /// Loads the view's meshes now (the frames give it no load budget), so a pick can find the
    /// objects drawn.
    pub fn settle_view(&mut self) {
        let frame = self.client.frame();
        let aspect = SIZE.0 as f32 / SIZE.1 as f32;
        self.view.settle(&self.gpu, &frame, aspect);
        self.view.update(&self.gpu, &frame, Duration::from_secs(10));
        self.view.settle(&self.gpu, &frame, aspect);
    }

    /// A gameplay command for the next frame.
    pub fn command(&mut self, command: Command) {
        self.pending.push(command);
    }

    /// Frames for `seconds` of the server's time.
    pub fn run(&mut self, seconds: f64) {
        let until = Instant::now() + Duration::from_secs_f64(seconds);
        while Instant::now() < until {
            self.frame(&[]);
        }
    }

    /// Frames until `ready` holds, at most `seconds`; false when it never did.
    pub fn until(&mut self, seconds: f64, ready: impl Fn(&Self) -> bool) -> bool {
        let until = Instant::now() + Duration::from_secs_f64(seconds);
        while Instant::now() < until {
            if ready(self) {
                return true;
            }
            self.frame(&[]);
        }
        ready(self)
    }

    /// [`Self::until`] that fails the test with `what`.
    pub fn wait(&mut self, what: &str, ready: impl Fn(&Self) -> bool) {
        assert!(self.until(120.0, ready), "{what}");
    }

    /// Shows or hides a toolbar panel.
    pub fn panel(&mut self, child: ToolbarChild, shown: bool) {
        self.view.show_panel(child, shown);
        self.run(0.2);
    }

    /// A click at `at` (window pixels).
    pub fn click(&mut self, at: (i32, i32)) {
        self.frame(&[
            (UI_POINTER_X, at.0 as f32, true),
            (UI_POINTER_Y, at.1 as f32, true),
        ]);
        self.frame(&[(LEFT, 1.0, true)]);
        self.frame(&[(LEFT, 1.0, false)]);
        self.run(0.1);
    }
}

impl LiveRig {
    /// Logs off to character selection and enters the world again with the same character, as
    /// the player does (`gmGamePlayUI::EndSession`, then `gmCharacterSelectionUI`'s Enter on the
    /// character the session kept in its list); the HUD is built anew, as after a relog.
    pub fn relog(&mut self) {
        self.command(Command::LogOffCharacter);
        assert!(
            self.until(20.0, |r| !r.client.frame().in_world),
            "the log off never left the world"
        );
        let set = self
            .client
            .session()
            .characters()
            .cloned()
            .expect("the character set");
        let mut ui = PregameUi::new(load_pregame(&self.store).unwrap(), 1024, 768);
        ui.skip_intro();
        ui.resume_at_character_selection(set);
        let mut t = 0.0;
        let mut step = |rig: &mut Self, ui: &mut PregameUi| {
            rig.client.settle_loads(Duration::from_secs(30)).unwrap();
            for event in rig.client.use_time(&FrameInput::default()).unwrap() {
                pregame_event(ui, &event);
                rig.seen.push(event);
            }
            ui.use_time(t);
            pregame_requests(ui, rig.client.session_mut());
            t += FRAME.as_secs_f64();
            std::thread::sleep(FRAME);
        };
        let deadline = Instant::now() + Duration::from_secs(60);
        while ui.mode() != MODE_CHARACTER_SELECTION {
            step(self, &mut ui);
            assert!(Instant::now() < deadline, "no character selection");
        }
        for _ in 0..10 {
            step(self, &mut ui);
        }
        pregame_click(&mut ui, "EnterButton");
        let deadline = Instant::now() + Duration::from_secs(60);
        while !self.client.frame().in_world || ui.mode() != MODE_GAMEPLAY {
            step(self, &mut ui);
            assert!(Instant::now() < deadline, "no world entry after the relog");
        }
        self.view = ClientView::new(Arc::clone(&self.world), &self.gpu, COLOR_FORMAT, 1, SIZE);
        self.selection = SelectionUi::default();
        self.wait("the HUD after the relog", |r| r.view.shortcut_slots() > 0);
    }

    /// A pointer move to `at` (window pixels), in one frame.
    pub fn point(&mut self, at: (i32, i32)) {
        self.frame(&[
            (UI_POINTER_X, at.0 as f32, true),
            (UI_POINTER_Y, at.1 as f32, true),
        ]);
    }

    /// A right click at `at` (window pixels): the selection and its examination, as the player's.
    pub fn right_click(&mut self, at: (i32, i32)) {
        self.point(at);
        self.frame(&[(RIGHT, 1.0, true)]);
        self.frame(&[(RIGHT, 1.0, false)]);
        self.run(0.1);
    }

    /// The left button down or up.
    pub fn button(&mut self, down: bool) {
        self.frame(&[(LEFT, 1.0, down)]);
    }

    /// Presses at `from`, moves in steps to `to` and releases there, then runs frames for the
    /// server's answer (a drag and drop through the HUD, as the player does it).
    pub fn drag(&mut self, from: (i32, i32), to: (i32, i32)) {
        self.point(from);
        self.button(true);
        for step in 1..=4 {
            let t = step as f32 / 4.0;
            let x = from.0 as f32 + (to.0 - from.0) as f32 * t;
            let y = from.1 as f32 + (to.1 - from.1) as f32 * t;
            self.point((x.round() as i32, y.round() as i32));
        }
        self.button(false);
        self.run(0.5);
    }
}

impl Drop for LiveRig {
    /// The character leaves the world (`gmGamePlayUI::EndSession`), so no body is left standing.
    fn drop(&mut self) {
        let character = self.client.session().player();
        self.command(Command::LogOffCharacter);
        let _ = self.until(10.0, |r| !r.client.frame().in_world);
        if let Some(character) = character {
            self.client.session_mut().delete_character(character);
            let _ = self.until(5.0, |r| {
                r.seen.contains(&SessionEvent::CharacterDeleted(character))
            });
        }
    }
}
