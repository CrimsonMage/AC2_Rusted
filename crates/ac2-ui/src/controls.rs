//! The client's controls as `gmClient` and `CInputManager_WIN32::Init` set them up: the preproc
//! key map (`0x40000001`, pointer axes, buttons, modifiers, arrows, wheel), then
//! `gmClient::InitKeymap`'s main, shift (left shift) and control (left control) maps from the
//! DATs (the `.keymap` files a player saved are not read), the `ActionInterpreter` over the
//! action map, and per frame what the movement module and the camera consume.

use ac2_db::{ResourceResult, ResourceStore};
use ac2_weenie::ActiveAction;

use crate::camera_ui::{CameraUi, CameraUiEffect, UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::input::{InputEvent, InputManager, RawAction, DEVICE_MOUSE};
use crate::interpreter::{ActionInterpreter, Dispatch};
use crate::maps::{load_action_map, load_key_map, KEYMAP_CONTROL, KEYMAP_MAIN, KEYMAP_SHIFT};
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS, GLOBAL_KEY_RELEASE};
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};

/// `CInputManager_WIN32::Init`'s map.
const KEYMAP_PREPROC: u32 = 0x4000_0001;
const LEFT_SHIFT: u32 = 0x8100_042A;
const LEFT_CONTROL: u32 = 0x8100_041D;

/// `CameraManager::StartAction` ids for the camera UI's zoom (`ZoomIn` / `ZoomOut`).
const CAMERA_ZOOM_IN: u32 = 0x4000_002D;
const CAMERA_ZOOM_OUT: u32 = 0x4000_00B2;

/// One frame of controls for the client core.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ControlFrame {
    /// The held actions (`m_current_actions` through `GetActionValues`).
    pub actions: Vec<ActiveAction>,
    /// Camera actions started and stopped (`CameraManager::StartAction` / `StopAction`).
    pub camera_actions: Vec<(u32, f32, bool)>,
    /// Behaviours started and stopped on the player.
    pub behaviors: Vec<(u32, bool)>,
    /// A movement action started (`InterruptMoveTo`).
    pub interrupt_move_to: bool,
    /// The mouse-look motion of the frame (`CallMouseLookHandler`), in mouse-look mode.
    pub mouse_look: Option<(i32, i32)>,
    /// `gmCameraUI::UpdateCamera`, when it ran.
    pub mouse_look_state: Option<bool>,
    pub first_person_state: Option<bool>,
    /// UI actions nothing here handles (inventory, chat, menus...).
    pub ui: Vec<(u32, f32, bool)>,
}

pub struct Controls {
    pub input: InputManager,
    pub interpreter: ActionInterpreter,
    pub camera_ui: CameraUi,
    /// A UI element tree takes the UI actions (`IUIManager` start / stop): they wait in
    /// [`Controls::take_ui_input`] and come back as [`Controls::ui_messages`]. Without one the
    /// camera UI handles them directly.
    pub ui_attached: bool,
    ui_input: Vec<(u32, f32, bool)>,
    pending: ControlFrame,
}

impl Controls {
    /// The default controls from the DATs; `None` without the action map.
    pub fn load(store: &ResourceStore) -> ResourceResult<Option<Self>> {
        let Some(action_map) = load_action_map(store)? else {
            return Ok(None);
        };
        let mut input = InputManager::new();
        for (map, meta) in [
            (KEYMAP_PREPROC, 0),
            (KEYMAP_MAIN, 0),
            (KEYMAP_SHIFT, LEFT_SHIFT),
            (KEYMAP_CONTROL, LEFT_CONTROL),
        ] {
            if let Some(map) = load_key_map(store, map)? {
                input.push_key_map(&map, meta);
            }
        }
        Ok(Some(Self::new(
            input,
            ActionInterpreter::new((*action_map).clone()),
        )))
    }

    pub fn new(input: InputManager, interpreter: ActionInterpreter) -> Self {
        Self {
            input,
            interpreter,
            camera_ui: CameraUi::default(),
            ui_attached: false,
            ui_input: Vec::new(),
            pending: ControlFrame::default(),
        }
    }

    /// One input event at `timestamp` milliseconds and client time `now`.
    pub fn event(&mut self, event: InputEvent, timestamp: u32, now: f64) {
        let mut actions: Vec<RawAction> = Vec::new();
        self.input.process(event, timestamp, &mut actions);
        for action in actions {
            self.action(action.action, action.extent, action.key, action.device, now);
        }
    }

    /// `ActionInterpreter::FocusSwitchHandler`: losing focus releases every held action (with
    /// focus kept, modifier and UI actions stay).
    pub fn focus(&mut self, have_focus: bool, now: f64) {
        let held: Vec<_> = self.interpreter.current().to_vec();
        for info in held {
            let Some(value) = self.interpreter.action_values(info.action) else {
                continue;
            };
            let keep = value.class == crate::CLASS_UI || value.class == crate::CLASS_MODIFIER;
            if !have_focus || !keep {
                self.action(info.action, 0.0, info.key, info.device, now);
            }
        }
    }

    /// `ActionInterpreter::ActionHandler`, then what its dispatches do.
    fn action(&mut self, action: u32, extent: f32, key: u32, device: u32, now: f64) {
        let mut dispatch = Vec::new();
        self.interpreter
            .handle(action, extent, key, device, now, &mut dispatch);
        for d in dispatch {
            match d {
                Dispatch::InterruptMoveTo => self.pending.interrupt_move_to = true,
                Dispatch::StopAttackingIfAutoMoving => {}
                Dispatch::Behavior { behavior, start } => {
                    self.pending.behaviors.push((behavior, start))
                }
                Dispatch::Camera {
                    action,
                    extent,
                    start,
                } => self.pending.camera_actions.push((action, extent, start)),
                Dispatch::Ui {
                    action: ui,
                    value,
                    start,
                } if self.ui_attached => self.ui_input.push((ui, value, start)),
                Dispatch::Ui {
                    action: ui,
                    value,
                    start,
                } => self.camera_ui_action(ui, value, start, now),
            }
        }
    }

    /// The UI actions for the UI tree (`UIElementManager::StartInput` / `StopInput`), in order.
    pub fn take_ui_input(&mut self) -> Vec<(u32, f32, bool)> {
        std::mem::take(&mut self.ui_input)
    }

    /// The UI tree's messages for the frameworks listening here: `gmCameraUI::ListenToMessage`
    /// takes the global key press / release messages and `ItemSelectionField`'s
    /// `mouse_press_top` wheel turns (the pointer over the world) as zoom.
    pub fn ui_messages(&mut self, messages: &[UiMessage], now: f64) {
        let world = tokenize("ItemSelectionField");
        for message in messages {
            match message.message {
                GLOBAL_KEY_PRESS | GLOBAL_KEY_RELEASE if message.element.is_none() => {
                    let start = message.message == GLOBAL_KEY_PRESS;
                    self.camera_ui_action(message.data as u32, 0.0, start, now);
                }
                MSG_MOUSE_PRESS_TOP if message.element_id == world => {
                    let action = message.data as u32;
                    if matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT) {
                        self.camera_ui_action(action, 0.0, true, now);
                    }
                }
                _ => {}
            }
        }
    }

    /// `gmCameraUI`'s handling of one UI action; actions it ignores go to [`ControlFrame::ui`].
    fn camera_ui_action(&mut self, ui: u32, value: f32, start: bool, now: f64) {
        match self.camera_ui.action(ui, start) {
            Some(CameraUiEffect::Action { action, extent }) => {
                // `gmCameraUI` re-sends with the action as its key, from device 1.
                self.action(action, extent, action, DEVICE_MOUSE, now);
            }
            Some(CameraUiEffect::States {
                mouse_look,
                first_person,
            }) => {
                self.pending.mouse_look_state = Some(mouse_look);
                self.pending.first_person_state = Some(first_person);
            }
            Some(CameraUiEffect::Zoom { zoom_in }) => {
                let zoom = if zoom_in {
                    CAMERA_ZOOM_IN
                } else {
                    CAMERA_ZOOM_OUT
                };
                self.pending.camera_actions.push((zoom, 1.0, true));
            }
            None => self.pending.ui.push((ui, value, start)),
        }
    }

    /// The frame's controls; then `ActionInterpreter::UseTime` drops the one-shot actions.
    pub fn frame(&mut self) -> ControlFrame {
        let mut frame = std::mem::take(&mut self.pending);
        frame.actions = self.interpreter.active_actions();
        frame.mouse_look = self.input.take_mouse_look();
        self.interpreter.use_time();
        frame
    }
}
