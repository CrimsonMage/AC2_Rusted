//! A client with the gameplay HUD over the Legions server double, driven the way the window
//! drives it: the pointer and the mouse buttons as UI actions (`UI_POINTER_X` / `UI_POINTER_Y`,
//! the left button), then `Client::use_time` and the HUD's frameworks every frame
//! (`bin/ac2/main.rs`'s `world_frame`). Needs `AC2_DAT_DIR` and a GPU adapter.

#![allow(dead_code)]
#![cfg(feature = "window")]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use ac2_client::view::ClientView;
use ac2_client::{Client, Command, DatWorld, FrameInput};
use ac2_gpu::{Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_ui::{Modifiers, ToolbarChild, UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent};
use legions_server_double::{addr, double, Spawn, State, SERVER};

pub const STEP: f64 = 1.0 / 64.0;
pub const SIZE: (u32, u32) = (1024, 768);
pub const LEFT: u32 = 0x4000_0004;

/// One HUD rig at a time per test binary: each holds a headless GPU device with the gameplay
/// HUD's textures, and a binary's tests run in parallel, which runs a shared GPU out of memory.
static GPU_SLOT: Mutex<()> = Mutex::new(());

pub struct Rig {
    pub client: Client,
    pub server: Rc<RefCell<State>>,
    pub view: ClientView,
    pub gpu: Gpu,
    /// The commands the HUD's frameworks asked for, sent with the next frame.
    pending: Vec<Command>,
    /// The commands every frame asked for, in order.
    pub asked: Vec<Command>,
    pub shift: bool,
    /// The Alt key held (the window exception's Alt + drag).
    pub alt: bool,
    /// The world the view draws, kept to rebuild the HUD.
    pub world: Arc<ac2_world::Ac2WorldSource>,
    /// Held for the rig's life ([`GPU_SLOT`]); dropped last.
    _gpu_slot: MutexGuard<'static, ()>,
}

impl Rig {
    /// The player in the world with the script attached and the HUD built (its panels hidden).
    /// `None` without `AC2_DAT_DIR` or a GPU adapter.
    pub fn new() -> Option<Self> {
        Self::new_in(None)
    }

    /// As [`Rig::new`], with the player's window position files in `positions` (a fresh login
    /// reads the files there; `None` is the player's AC2 directory).
    pub fn new_in(positions: Option<PathBuf>) -> Option<Self> {
        let dataset = ac2_dat::test_support::env_dataset()?;
        let tables_dataset = Arc::clone(&dataset);
        // A test that panicked while holding the slot leaves it poisoned; the slot guards no data.
        let gpu_slot = GPU_SLOT
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Ok(gpu) = Gpu::headless() else {
            eprintln!("no GPU adapter; skipping");
            return None;
        };
        let store = Arc::new(ac2_db::ResourceStore::new(dataset));
        let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
        let world = Arc::clone(dat_world.world());
        let block = ac2_core::CellId::new(0xCB4D_0001);
        let top = world
            .load_landblock(block)
            .unwrap()
            .unwrap()
            .height_grid()
            .into_iter()
            .fold(0.0, f32::max);
        let (transport, server) = double(Spawn {
            cell: 0xCB4D_0025,
            origin: [96.0, 96.0, top + 2.0],
            rotation: [1.0, 0.0, 0.0, 0.0],
        });
        let config = OnlineConfig {
            server: addr(SERVER),
            account: "local".into(),
            credential: Credential::Account,
            version: ac2_net::LEGIONS_VERSION.into(),
            character: CharacterChoice::First,
            server_key: None,
            language: 0,
            dat_iterations: Vec::new(),
        };
        let mut session = OnlineSession::with_transport(config, Box::new(transport)).unwrap();
        let script_data = ac2_script::data::DatScriptData::new(tables_dataset, (*store).clone());
        session.set_string_tables(ac2_client::string_table_ids(&script_data));
        let dat_world = dat_world.with_script_data(script_data.clone());
        let mut client = Client::new(session, Arc::new(dat_world));
        client.set_window_positions_dir(positions);
        let wlib =
            ac2_script::load_wlib(&store, ac2_core::DataId::new(ac2_script::LEGIONS_WLIB_DID))
                .unwrap()
                .expect("the script library");
        let program = Arc::new(ac2_script::vm::Program::from_stream(&wlib.stream));
        script_data.preload_tables();
        script_data.preload_states();
        script_data.preload_skill_states(&program);
        script_data.set_cache_only(true);
        client.attach_scripts(
            ac2_script::host::ScriptHost::new(program, Box::new(script_data)),
            ac2_client::SCRIPT_VM_FUNCTIONS,
        );
        client.set_load_budget(Duration::from_secs(30));
        let view = ClientView::new(Arc::clone(&world), &gpu, COLOR_FORMAT, 1, SIZE);
        let mut rig = Self {
            client,
            server,
            view,
            gpu,
            pending: Vec::new(),
            asked: Vec::new(),
            shift: false,
            alt: false,
            world: Arc::clone(&world),
            _gpu_slot: gpu_slot,
        };
        let mut frames = 0;
        while !rig.client.frame().in_world {
            rig.client.settle_loads(Duration::from_secs(30)).unwrap();
            rig.client.use_time(&FrameInput::default()).unwrap();
            rig.server.borrow_mut().elapsed += STEP;
            frames += 1;
            assert!(frames < 600, "no world entry");
        }
        Some(rig)
    }

    /// One frame: the UI `actions` into the HUD, the client's frame with the commands the
    /// frameworks asked for last frame, then the frameworks and the view.
    pub fn frame(&mut self, actions: &[(u32, f32, bool)]) -> Vec<SessionEvent> {
        let modifiers = Modifiers {
            shift: self.shift,
            alt: self.alt,
            ..Modifiers::default()
        };
        self.view.ui_input(actions, modifiers);
        let input = FrameInput {
            commands: std::mem::take(&mut self.pending),
            ..FrameInput::default()
        };
        self.client.settle_loads(Duration::from_secs(5)).unwrap();
        let events = self.client.use_time(&input).unwrap();
        self.server.borrow_mut().elapsed += STEP;
        self.server.borrow_mut().combat_tick();
        let frame = self.client.frame();
        let notices = self.client.take_script_notices();
        let asked = self
            .view
            .frameworks(&mut self.client, &frame, &events, &notices);
        self.asked.extend(asked.iter().cloned());
        self.pending.extend(asked);
        self.view.update(&self.gpu, &frame, Duration::ZERO);
        events
    }

    /// The starter items given, the player's login state sent and the inventory panel shown.
    pub fn open_inventory(&mut self) {
        {
            let mut server = self.server.borrow_mut();
            server.give_starter_items();
            server.send_init_player();
        }
        self.idle(16);
        self.wait("the bar", |r| r.view.shortcut_slots() > 0);
        self.panel(ToolbarChild::Inventory, true);
    }

    /// A click at `at`: the button down and up there.
    pub fn click(&mut self, at: (i32, i32)) {
        self.point(at);
        self.button(true);
        self.button(false);
        self.idle(4);
    }

    /// A gameplay command for the next frame (as the window's UI would ask).
    pub fn command(&mut self, command: Command) {
        self.pending.push(command);
    }

    /// One frame with `events` as the session's notices for the HUD (no pointer input).
    pub fn notify(&mut self, events: &[SessionEvent]) {
        let frame = self.client.frame();
        let asked = self.view.frameworks(&mut self.client, &frame, events, &[]);
        self.asked.extend(asked.iter().cloned());
        self.pending.extend(asked);
        self.view.update(&self.gpu, &frame, Duration::ZERO);
    }

    /// Frames with nothing pressed.
    pub fn idle(&mut self, frames: usize) {
        for _ in 0..frames {
            self.frame(&[]);
        }
    }

    /// Idle frames until `ready` holds (the HUD's loads finish on worker threads). The budget is
    /// wall-clock time, not a frame count, so a loaded machine's slow loaders still finish.
    pub fn wait(&mut self, what: &str, ready: impl Fn(&Self) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(60);
        while Instant::now() < deadline {
            if ready(self) {
                return;
            }
            self.frame(&[]);
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("{what}");
    }

    /// A new view of the same world: the HUD is built again, as after a relog, and its windows
    /// take the player's saved places at the first frame in the world.
    pub fn rebuild_hud(&mut self) {
        self.view = ClientView::new(Arc::clone(&self.world), &self.gpu, COLOR_FORMAT, 1, SIZE);
        self.wait("the rebuilt HUD", |r| r.view.shortcut_slots() > 0);
    }

    /// Shows or hides a toolbar panel.
    pub fn panel(&mut self, child: ToolbarChild, shown: bool) {
        self.view.show_panel(child, shown);
        self.idle(2);
    }

    /// The pointer to `at` (window pixels), in one frame.
    pub fn point(&mut self, at: (i32, i32)) {
        self.frame(&[
            (UI_POINTER_X, at.0 as f32, true),
            (UI_POINTER_Y, at.1 as f32, true),
        ]);
    }

    /// The left button down or up.
    pub fn button(&mut self, down: bool) {
        self.frame(&[(LEFT, 1.0, down)]);
    }

    /// Presses at `from`, moves in steps to `to` and releases there.
    pub fn drag(&mut self, from: (i32, i32), to: (i32, i32)) {
        self.press_and_move(from, to);
        self.release();
    }

    /// Presses at `from` and moves to `to`, the button held.
    pub fn press_and_move(&mut self, from: (i32, i32), to: (i32, i32)) {
        self.point(from);
        self.button(true);
        for step in 1..=4 {
            let t = step as f32 / 4.0;
            let x = from.0 as f32 + (to.0 - from.0) as f32 * t;
            let y = from.1 as f32 + (to.1 - from.1) as f32 * t;
            self.point((x.round() as i32, y.round() as i32));
        }
    }

    /// The left button up, then frames for the answers.
    pub fn release(&mut self) {
        self.button(false);
        self.idle(8);
    }
}
