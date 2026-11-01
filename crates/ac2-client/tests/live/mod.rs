//! The client against a running AC2e server, driven as the window drives it: a fresh account and
//! character made through the real character creation screens, then the gameplay HUD's
//! frameworks every frame (`bin/ac2/main.rs`'s `world_frame`), with the server's own clock and
//! rules. Runs only with `AC2E_LIVE` set to the server's address and `AC2_DAT_DIR` naming the
//! 7230 DATs; needs a GPU adapter. `scripts/ac2e-live-test.sh` starts a server of its own for
//! the suite (its own data, ports and developer account, `AC2E_ACCOUNT`), so a player's server
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
use ac2_ui::{tokenize, Modifiers, ToolbarChild, UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent};

pub const SIZE: (u32, u32) = (1024, 768);
pub const LEFT: u32 = 0x4000_0004;
/// The frame's step, as the window's: the server keeps its own time.
const FRAME: Duration = Duration::from_millis(16);

/// The server's address, when live tests are asked for.
pub fn server() -> Option<SocketAddr> {
    let text = std::env::var("AC2E_LIVE").ok()?;
    Some(
        text.parse()
            .unwrap_or_else(|e| panic!("AC2E_LIVE={text}: {e}")),
    )
}

/// The server's developer account (`ac2e serve --account`), `local` by default.
fn account() -> String {
    std::env::var("AC2E_ACCOUNT").unwrap_or_else(|_| "local".into())
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
    /// Every session event the frames saw, in order.
    pub seen: Vec<SessionEvent>,
    pub world: Arc<ac2_world::Ac2WorldSource>,
    started: Instant,
}

impl LiveRig {
    /// A new character named `name` (letters only) on a fresh account, made through avatar
    /// selection (the first avatar) and avatar modification, entered into the world, the HUD
    /// built. `None` without `AC2E_LIVE`, `AC2_DAT_DIR` or a GPU adapter.
    pub fn new(name: &str) -> Option<Self> {
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
            version: ac2_net::default_version().into(),
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

        // Character creation on the real screens, against the server.
        let chargen =
            ac2_script::host::ScriptHost::new(Arc::clone(&program), Box::new(script_data.clone()));
        let mut ui = PregameUi::new(load_pregame(&store).unwrap(), 1024, 768);
        ui.skip_intro();
        ui.set_char_gen_system(Box::new(ScriptCharGen::new(chargen)));
        let mut t = 0.0;
        let pregame = |client: &mut Client, ui: &mut PregameUi, t: &mut f64| {
            client.settle_loads(Duration::from_secs(30)).unwrap();
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
        client.attach_scripts(
            ac2_script::host::ScriptHost::new(program, Box::new(script_data)),
            ac2_client::SCRIPT_VM_FUNCTIONS,
        );
        let view = ClientView::new(Arc::clone(&world), &gpu, COLOR_FORMAT, 1, SIZE);
        let mut rig = Self {
            client,
            view,
            gpu,
            account,
            name: name.to_owned(),
            pending: Vec::new(),
            seen: Vec::new(),
            world,
            started: Instant::now(),
        };
        rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
        Some(rig)
    }

    /// One frame: the UI `actions` into the HUD, the client's frame with the commands the
    /// frameworks asked for last frame, then the frameworks and the view; the server answers in
    /// its own time.
    pub fn frame(&mut self, actions: &[(u32, f32, bool)]) -> Vec<SessionEvent> {
        self.view.ui_input(actions, Modifiers::default());
        let input = FrameInput {
            commands: std::mem::take(&mut self.pending),
            ..FrameInput::default()
        };
        self.client.settle_loads(Duration::from_secs(5)).unwrap();
        let events = self.client.use_time(&input).unwrap();
        let frame = self.client.frame();
        let notices = self.client.take_script_notices();
        let asked = self
            .view
            .frameworks(&mut self.client, &frame, &events, &notices);
        self.pending.extend(asked);
        self.view.update(&self.gpu, &frame, Duration::ZERO);
        self.gpu.flush_writes();
        self.seen.extend(events.iter().cloned());
        std::thread::sleep(FRAME);
        events
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

impl Drop for LiveRig {
    /// The character leaves the world (`gmGamePlayUI::EndSession`), so no body is left standing.
    fn drop(&mut self) {
        let character = self.client.session().player();
        self.command(Command::LogOffCharacter);
        let _ = self.until(10.0, |r| !r.client.frame().in_world);
        if let Some(character) = character {
            self.client.session_mut().delete_character(character);
            let _ = self.until(5.0, |r| {
                r.seen
                    .iter()
                    .any(|e| *e == SessionEvent::CharacterDeleted(character))
            });
        }
    }
}
