//! The HUD's input in the viewer, as the client takes it (`bin/ac2`'s window events and world
//! frame): the window's keys, buttons, wheel and pointer go through the retail input stack
//! (`ac2_ui::Controls`: the input manager, the player's key maps and the action interpreter), so
//! the toolbar's keys, the chat's Enter, typing in text fields and the wheel over a panel reach
//! the HUD. Of the camera actions the controls start, the viewer's camera takes the zooms (the
//! wheel over the world); its own keys still walk, turn and fly.

use std::time::Instant;

use ac2_client::keys;
use ac2_client::view::ClientView;
use ac2_ui::{wheel_event_delta, Controls, InputEvent, WheelAccumulator};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase};
use winit::keyboard::PhysicalKey;

use crate::walk::{actions, Walk};

pub struct HudInput {
    controls: Controls,
    wheel: WheelAccumulator,
    /// Super held: the macOS Cmd clipboard shortcuts.
    super_held: bool,
    started: Instant,
}

impl HudInput {
    /// The controls over the player's keymap files, else the DATs' maps (`gmClient::InitKeymap`);
    /// `None` when the DATs hold no action map.
    pub fn load(store: &ac2_db::ResourceStore) -> Option<Self> {
        let files = ac2_ui::keymap_file::KeymapFiles::user();
        let mut controls = Controls::load_with(store, Some(&files)).ok().flatten()?;
        // The HUD's tree takes the UI actions (the toolbar's panel keys, clicks over panels, the
        // wheel over one), as `bin/ac2` attaches it; unattached, the camera UI would take them.
        controls.ui_attached = true;
        Some(Self {
            controls,
            wheel: WheelAccumulator::default(),
            super_held: false,
            started: Instant::now(),
        })
    }

    /// `Timer::cur_time` for the input manager: milliseconds and seconds since the start.
    fn now(&self) -> (u32, f64) {
        let elapsed = self.started.elapsed();
        (elapsed.as_millis() as u32, elapsed.as_secs_f64())
    }

    fn input(&mut self, event: InputEvent) {
        let (stamp, now) = self.now();
        self.controls.event(event, stamp, now);
    }

    /// Whether the HUD's active text element takes the typing (the viewer's keys then stay off).
    pub fn text_mode(&self) -> bool {
        self.controls.input.text_mode()
    }

    pub fn focus(&mut self, focused: bool) {
        let (_, now) = self.now();
        self.controls.focus(focused, now);
    }

    pub fn modifiers(&mut self, alt: bool, super_key: bool) {
        self.controls.set_alt(alt);
        self.super_held = super_key;
    }

    pub fn cursor(&mut self, x: f64, y: f64) {
        self.input(InputEvent::Cursor {
            x: x as i32,
            y: y as i32,
        });
    }

    /// A key: the clipboard shortcut it is (macOS), else its `DIK` key and, pressed, its text.
    /// Outside text mode a held key is one press.
    pub fn key(&mut self, event: &KeyEvent) {
        if event.repeat && !self.text_mode() {
            return;
        }
        let pressed = event.state == ElementState::Pressed;
        let shortcut = keys::clipboard_shortcut(
            keys::ShortcutPlatform::current(),
            self.super_held,
            &event.logical_key,
        );
        if let Some(code) = shortcut {
            if pressed {
                self.input(InputEvent::Text { ch: code });
            }
            return;
        }
        if let PhysicalKey::Code(code) = event.physical_key {
            if let Some(scan) = keys::dik(code) {
                self.input(InputEvent::Key { scan, pressed });
            }
        }
        if pressed {
            if let Some(text) = &event.text {
                for event in ac2_ui::typed_text(text) {
                    self.input(event);
                }
            }
        }
    }

    pub fn button(&mut self, button: MouseButton, pressed: bool) {
        let button = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            _ => return,
        };
        self.input(InputEvent::Button { button, pressed });
    }

    /// The wheel's notches, one wheel action each (`ProcessDeviceData`'s Z axis).
    pub fn wheel(&mut self, delta: MouseScrollDelta, phase: TouchPhase, scale_factor: f64) {
        if phase == TouchPhase::Started {
            self.wheel.start_gesture();
        }
        let notches = match delta {
            MouseScrollDelta::LineDelta(_, y) => self.wheel.line_delta(y),
            MouseScrollDelta::PixelDelta(p) => self.wheel.pixel_delta(p.y / scale_factor),
        };
        let step = notches.signum();
        for _ in 0..notches.unsigned_abs() {
            self.input(InputEvent::Wheel {
                delta: wheel_event_delta(step),
            });
        }
    }

    /// The frame's input into the HUD (its UI actions, then the typing), the HUD's state back to
    /// the input manager, and the zooms the wheel started to the viewer's camera.
    pub fn frame(&mut self, view: &mut ClientView, walk: &mut Walk) {
        let modifiers = self.controls.modifiers();
        let ui_input = self.controls.take_ui_input();
        let mut messages = view.ui_input(&ui_input, modifiers);
        let (text_actions, characters, _) = self.controls.take_text_input();
        messages.extend(view.text_input(&text_actions, &characters, modifiers));
        let (stamp, now) = self.now();
        self.controls.set_text_mode(view.text_mode(), stamp, now);
        self.controls.set_copy_text(view.text_copy_focus());
        self.controls
            .set_camera_keys_blocked(view.camera_keys_blocked());
        self.controls.ui_messages(&messages, now);
        for (action, _, start) in self.controls.frame().camera_actions {
            if matches!(
                action,
                actions::ZOOM_IN
                    | actions::ZOOM_OUT
                    | actions::ZOOM_SCALE_IN
                    | actions::ZOOM_SCALE_OUT
            ) {
                walk.camera_action(action, start);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use ac2_client::view::ClientView;
    use ac2_client::ClientFrame;
    use ac2_db::ResourceStore;
    use ac2_ui::{InputEvent, ToolbarChild};
    use ac2_world::Ac2WorldSource;

    use super::HudInput;
    use crate::session::Session;
    use crate::walk::{Landing, Walk};

    /// `DIK_I`, the inventory's key in the DATs' main key map.
    const DIK_I: u32 = 0x17;

    /// On the real DATs (`AC2_DAT_DIR`): the inventory key, through the viewer's HUD input,
    /// opens the inventory panel of the session's HUD.
    #[test]
    fn the_inventory_key_opens_the_inventory() {
        let Some(dataset) = ac2_dat::test_support::env_dataset() else {
            return;
        };
        let Ok(gpu) = ac2_gpu::Gpu::headless() else {
            return;
        };
        let store = Arc::new(ResourceStore::new(dataset));
        let world = Ac2WorldSource::open(Arc::clone(&store)).unwrap().unwrap();
        let landing = Landing {
            block: (0x7F, 0x7F),
            x: 80.0,
            y: 80.0,
        };
        let mut session = Session::start(&store, &world, landing).expect("the session starts");
        let mut walk = Walk::new(&world, Arc::clone(&store), 0.0);
        let mut view = ClientView::new(world, &gpu, ac2_gpu::COLOR_FORMAT, 1, (1280, 720));
        let mut hud = HudInput::load(&store).expect("the DATs' key maps");
        let mut step = |view: &mut ClientView, hud: &mut HudInput| {
            let mut frame = ClientFrame {
                in_world: true,
                cell_id: Some(landing.cell()),
                ..ClientFrame::default()
            };
            assert!(!session.frame(view, &mut frame, 1.0 / 64.0));
            view.update(&gpu, &frame, Duration::ZERO);
            hud.frame(view, &mut walk);
        };
        let started = std::time::Instant::now();
        while view.shortcut_slots() == 0 {
            assert!(
                started.elapsed() < Duration::from_secs(60),
                "the HUD is built"
            );
            step(&mut view, &mut hud);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!view.panel_shown(ToolbarChild::Inventory));
        for pressed in [true, false] {
            hud.input(InputEvent::Key {
                scan: DIK_I,
                pressed,
            });
            for _ in 0..2 {
                step(&mut view, &mut hud);
            }
        }
        assert!(view.panel_shown(ToolbarChild::Inventory));
    }
}
