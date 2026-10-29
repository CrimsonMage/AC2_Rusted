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
        let controls = Controls::load_with(store, Some(&files)).ok().flatten()?;
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
                actions::ZOOM_IN | actions::ZOOM_OUT | actions::ZOOM_SCALE_IN | actions::ZOOM_SCALE_OUT
            ) {
                walk.camera_action(action, start);
            }
        }
    }
}
