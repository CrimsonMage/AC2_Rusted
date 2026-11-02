//! The HUD rig driven the way the window program drives the client: winit-equivalent events
//! (cursor moves, buttons, keys, relative motion) go into `ac2_ui::Controls` as
//! `bin/ac2/main.rs`'s `window_event` / `device_event` feed them, and each frame runs the
//! window's world frame (`ac2_client::world_input::begin_frame`, `Client::use_time`, the mouse-look
//! sync, the HUD's frameworks, the selection's pick) over the server double.

use std::time::Duration;

use ac2_client::world_input::{begin_frame, finish_selection};
use ac2_client::{Command, FrameInput};
use ac2_db::ResourceStore;
use ac2_ui::{Controls, InputEvent, SelectionUi};
use ac2_weenie::SessionEvent;
use winit::keyboard::KeyCode;

use super::{Rig, STEP};

/// The window loop's input state.
pub struct WindowState {
    controls: Controls,
    selection: SelectionUi,
    commands: Vec<Command>,
    /// Client time, in seconds, advanced a step per window frame.
    clock: f64,
    /// The window, in pixels.
    pub window: (u32, u32),
    /// The mouse-look state the window sees (the pointer grabbed and hidden).
    pub grabbed: bool,
    /// The camera inputs the frames sent, newest last.
    pub frames: Vec<FrameInput>,
    /// Every session event the window frames saw.
    pub seen: Vec<SessionEvent>,
}

impl WindowState {
    pub fn new(store: &ResourceStore) -> Self {
        let mut controls = Controls::load(store)
            .expect("the key maps")
            .expect("the DATs hold the action map");
        controls.ui_attached = true;
        Self {
            controls,
            selection: SelectionUi::default(),
            commands: Vec::new(),
            clock: 0.0,
            window: super::SIZE,
            grabbed: false,
            frames: Vec::new(),
            seen: Vec::new(),
        }
    }

    fn stamp(&self) -> (u32, f64) {
        ((self.clock * 1000.0) as u32, self.clock)
    }
}

impl Rig {
    /// The window resized (`WindowEvent::Resized`): the HUD lays out on the new display.
    pub fn win_resize(&mut self, size: (u32, u32)) {
        self.win.window = size;
        self.view.resize(size.0, size.1);
        self.win_idle(2);
    }

    fn win_input(&mut self, event: InputEvent) {
        let (stamp, now) = self.win.stamp();
        self.win.controls.event(event, stamp, now);
    }

    /// `WindowEvent::CursorMoved`.
    pub fn win_cursor(&mut self, at: (i32, i32)) {
        self.win_input(InputEvent::Cursor { x: at.0, y: at.1 });
    }

    /// `WindowEvent::MouseInput`: 0 left, 1 right, 2 middle.
    pub fn win_button(&mut self, button: u32, pressed: bool) {
        self.win_input(InputEvent::Button { button, pressed });
    }

    /// `DeviceEvent::MouseMotion`.
    pub fn win_motion(&mut self, dx: i32, dy: i32) {
        self.win_input(InputEvent::Motion { dx, dy });
    }

    /// `WindowEvent::KeyboardInput` for a physical key.
    pub fn win_key(&mut self, code: KeyCode, pressed: bool) {
        let scan = ac2_client::keys::dik(code).expect("a DirectInput key");
        self.win_input(InputEvent::Key { scan, pressed });
    }

    /// `WindowEvent::ModifiersChanged`'s Alt.
    pub fn win_alt(&mut self, held: bool) {
        self.win.controls.set_alt(held);
        self.alt = held;
    }

    /// One window frame (`world_frame`); returns the session's events.
    pub fn win_frame(&mut self) -> Vec<SessionEvent> {
        self.win.clock += STEP;
        let clock = self.win.stamp();
        let pending = std::mem::take(&mut self.pending);
        self.win.commands.extend(pending);
        let begun = begin_frame(
            &mut self.win.controls,
            &mut self.win.selection,
            &mut self.win.commands,
            &mut self.view,
            &mut self.client,
            clock,
        );
        self.client.settle_loads(Duration::from_secs(5)).unwrap();
        let events = self.client.use_time(&begun.input).unwrap();
        self.win.frames.push(begun.input);
        self.win.seen.extend(events.iter().cloned());
        self.server.borrow_mut().elapsed += STEP;
        self.server.borrow_mut().combat_tick();
        // `State::sync_mouse_look`.
        let mouse_look = self.client.camera().mouse_look;
        self.win.controls.input.set_mouse_look(mouse_look);
        self.win.grabbed = mouse_look;
        let frame = self.client.frame();
        let notices = self.client.take_script_notices();
        let asked = self
            .view
            .frameworks(&mut self.client, &frame, &events, &notices);
        self.asked.extend(asked.iter().cloned());
        self.win.commands.extend(asked);
        let screen = self.win.window;
        if finish_selection(
            &mut self.win.selection,
            &mut self.win.controls,
            &mut self.win.commands,
            &self.view,
            &self.client,
            &frame,
            screen,
        ) {
            self.view.open_examination();
        }
        self.view.update(&self.gpu, &frame, Duration::ZERO);
        self.gpu.flush_writes();
        events
    }

    /// Window frames with nothing new.
    pub fn win_idle(&mut self, frames: usize) {
        for _ in 0..frames {
            self.win_frame();
        }
    }

    /// The pointer moved to `at`, then a frame (the window delivers a move, the frame sees it).
    pub fn win_point(&mut self, at: (i32, i32)) {
        self.win_cursor(at);
        self.win_frame();
    }

    /// A button press or release, then a frame.
    pub fn win_press(&mut self, button: u32, pressed: bool) {
        self.win_button(button, pressed);
        self.win_frame();
    }

    /// A left click at `at`.
    pub fn win_click(&mut self, at: (i32, i32)) {
        self.win_point(at);
        self.win_press(0, true);
        self.win_press(0, false);
        self.win_idle(4);
    }

    /// A key press and release, one frame each.
    pub fn win_tap(&mut self, code: KeyCode) {
        self.win_key(code, true);
        self.win_frame();
        self.win_key(code, false);
        self.win_frame();
    }

    /// A left drag from `from` to `to` in steps.
    pub fn win_drag(&mut self, from: (i32, i32), to: (i32, i32)) {
        self.win_point(from);
        self.win_press(0, true);
        for step in 1..=4 {
            let t = step as f32 / 4.0;
            self.win_point((
                from.0 + ((to.0 - from.0) as f32 * t).round() as i32,
                from.1 + ((to.1 - from.1) as f32 * t).round() as i32,
            ));
        }
        self.win_press(0, false);
        self.win_idle(4);
    }
}
