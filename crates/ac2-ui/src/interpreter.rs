//! `ActionInterpreter`: the action handler that keeps the actions currently held
//! (`m_current_actions`) by their toggle type and starts and stops what they drive.
//!
//! `ActionHandler`: an action without a toggle type is ignored. A release (extent 0) ends a hold
//! action (`0x40000001`) held by that key; a press notes double taps (`HandleDoubleTap`), ends a
//! toggle action (`0x40000002`) the key already holds, turns run lock off for forward, backward
//! and auto-run, then records the action (extent truncated to an integer) and starts it.
//! `StartAction` / `StopAction` dispatch on the action map class: movement (`0x40000001`,
//! interrupting a move-to), behaviours (`0x40000002`), camera actions (`0x40000004`) and UI
//! actions (`0x40000006`); modifiers (`0x40000005`) only feed `GetModifierBitfield`. One-shot
//! actions (`0x40000003`) are dropped every `UseTime`. The movement module reads the held actions
//! through `GetActionValues` (modifiers, then without the `8` bit, then without modifiers).

use ac2_weenie::ActiveAction;

use crate::maps::ActionMap;

pub const TOGGLE_HOLD: u32 = 0x4000_0001;
pub const TOGGLE_TOGGLE: u32 = 0x4000_0002;
pub const TOGGLE_ONE_SHOT: u32 = 0x4000_0003;

pub const CLASS_MOVEMENT: u32 = 0x4000_0001;
pub const CLASS_BEHAVIOR: u32 = 0x4000_0002;
pub const CLASS_CAMERA: u32 = 0x4000_0004;
pub const CLASS_MODIFIER: u32 = 0x4000_0005;
pub const CLASS_UI: u32 = 0x4000_0006;

/// Actions that end run lock (`TurnOffRunLock`): forward, backward and auto-run.
const ACTION_FORWARD: u32 = 0x4000_0020;
const ACTION_BACKWARD: u32 = 0x4000_0021;
const ACTION_AUTO_RUN: u32 = 0x4000_00A3;
const ACTION_RUN_LOCK: u32 = 0x4000_0027;
/// Turn keys stop an auto-move attack (`StopAttackingIfAutoMoving`).
const ACTION_TURN_LEFT: u32 = 0x4000_0024;
const ACTION_TURN_RIGHT: u32 = 0x4000_0025;
/// `HandleDoubleTap` ignores the pointer actions and taps further apart than this.
const DOUBLE_TAP_SECONDS: f64 = 0.2;

/// `ActionInfo`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionInfo {
    pub action: u32,
    pub extent: f32,
    pub device: u32,
    pub key: u32,
    pub toggle: u32,
}

/// What starting or stopping an action asks of the rest of the client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dispatch {
    /// `PlayerMovementModule::InterruptMoveTo` (any movement action starting).
    InterruptMoveTo,
    /// `StopAttackingIfAutoMoving` (a turn key starting).
    StopAttackingIfAutoMoving,
    /// `PlayerMovementModule::DoBehavior` / `StopBehavior` of the action's enum value.
    Behavior { behavior: u32, start: bool },
    /// `CameraManager::StartAction(enum value, map extent)` / `StopAction(action)`.
    Camera {
        action: u32,
        extent: f32,
        start: bool,
    },
    /// `IUIManager` start (`0x4c`, with the pressed extent) and stop (`0x50`).
    Ui {
        action: u32,
        value: f32,
        start: bool,
    },
}

/// The interpreter's state.
#[derive(Debug, Clone)]
pub struct ActionInterpreter {
    map: ActionMap,
    current: Vec<ActionInfo>,
    double_tap: bool,
    last_device: u32,
    last_key: u32,
    last_press: f64,
}

impl ActionInterpreter {
    /// `ActionInterpreter::Startup` with its action map.
    pub fn new(map: ActionMap) -> Self {
        Self {
            map,
            current: Vec::new(),
            double_tap: false,
            last_device: 0,
            last_key: 0,
            last_press: 0.0,
        }
    }

    pub fn map(&self) -> &ActionMap {
        &self.map
    }

    /// `m_current_actions`.
    pub fn current(&self) -> &[ActionInfo] {
        &self.current
    }

    /// `m_is_double_tap`.
    pub fn is_double_tap(&self) -> bool {
        self.double_tap
    }

    /// `GetModifierBitfield`: the enum values of the held modifier-class actions.
    pub fn modifier_bitfield(&self) -> u32 {
        self.current
            .iter()
            .filter_map(|info| self.map.values(info.action, 0))
            .filter(|value| value.class == CLASS_MODIFIER)
            .fold(0, |bits, value| bits | value.enum_value)
    }

    /// `ActionInterpreter::GetActionValues`: the action under the held modifiers, then without
    /// bit 8, then without modifiers.
    pub fn action_values(&self, action: u32) -> Option<crate::maps::ActionValue> {
        let modifiers = self.modifier_bitfield();
        self.map
            .values(action, modifiers)
            .or_else(|| {
                (modifiers & 8 != 0)
                    .then(|| self.map.values(action, modifiers & !8))
                    .flatten()
            })
            .or_else(|| self.map.values(action, 0))
    }

    /// The held actions as the movement module reads them.
    pub fn active_actions(&self) -> Vec<ActiveAction> {
        self.current
            .iter()
            .filter_map(|info| {
                let value = self.action_values(info.action)?;
                Some(ActiveAction {
                    class: value.class,
                    enum_value: value.enum_value,
                    map_extent: value.extent,
                    extent: info.extent,
                })
            })
            .collect()
    }

    fn find(&self, device: u32, key: u32) -> Option<usize> {
        self.current
            .iter()
            .position(|info| info.device == device && info.key == key)
    }

    /// `HandleDoubleTap` at client time `now` (seconds).
    fn handle_double_tap(&mut self, action: u32, device: u32, key: u32, now: f64) {
        if action == crate::input::ACTION_POINTER_X || action == crate::input::ACTION_POINTER_Y {
            return;
        }
        if now <= self.last_press + DOUBLE_TAP_SECONDS
            && self.last_device == device
            && self.last_key == key
        {
            self.double_tap = true;
            return;
        }
        self.double_tap = false;
        self.last_device = device;
        self.last_key = key;
        self.last_press = now;
    }

    /// `TurnOffRunLock`.
    fn turn_off_run_lock(&mut self) {
        self.current.retain(|info| info.action != ACTION_RUN_LOCK);
    }

    /// `ActionHandler` for one action (`extent` 0 is a release) at client time `now`.
    pub fn handle(
        &mut self,
        action: u32,
        extent: f32,
        key: u32,
        device: u32,
        now: f64,
        out: &mut Vec<Dispatch>,
    ) {
        let toggle = self.map.toggle_type(action);
        if toggle == 0 {
            return;
        }
        if extent == 0.0 {
            if toggle == TOGGLE_HOLD {
                if let Some(index) = self.find(device, key) {
                    self.current.remove(index);
                    self.stop_action(action, out);
                }
            }
            return;
        }
        self.handle_double_tap(action, device, key, now);
        if toggle == TOGGLE_TOGGLE {
            if let Some(index) = self.find(device, key) {
                self.current.remove(index);
                self.stop_action(action, out);
                return;
            }
        } else if matches!(action, ACTION_FORWARD | ACTION_AUTO_RUN | ACTION_BACKWARD) {
            self.turn_off_run_lock();
        }
        // `_ftol2`: the extent is held as an integer.
        let held = extent.trunc() as i16;
        let info = ActionInfo {
            action,
            extent: f32::from(held),
            device,
            key,
            toggle,
        };
        match self.find(device, key) {
            Some(index) => self.current[index] = info,
            None => self.current.push(info),
        }
        self.start_action(action, held, out);
    }

    /// `StartAction`.
    fn start_action(&self, action: u32, held: i16, out: &mut Vec<Dispatch>) {
        let Some(value) = self.action_values(action) else {
            return;
        };
        match value.class {
            CLASS_MOVEMENT => {
                out.push(Dispatch::InterruptMoveTo);
                if matches!(action, ACTION_TURN_LEFT | ACTION_TURN_RIGHT) {
                    out.push(Dispatch::StopAttackingIfAutoMoving);
                }
            }
            CLASS_BEHAVIOR => out.push(Dispatch::Behavior {
                behavior: value.enum_value,
                start: true,
            }),
            CLASS_CAMERA => out.push(Dispatch::Camera {
                action: value.enum_value,
                extent: value.extent,
                start: true,
            }),
            CLASS_UI => out.push(Dispatch::Ui {
                action: value.enum_value,
                value: f32::from(held),
                start: true,
            }),
            _ => {}
        }
    }

    /// `StopAction`: the values without modifiers (`GetActionValues(action, 0)`).
    fn stop_action(&self, action: u32, out: &mut Vec<Dispatch>) {
        let Some(value) = self.map.values(action, 0) else {
            return;
        };
        match value.class {
            CLASS_BEHAVIOR => out.push(Dispatch::Behavior {
                behavior: value.enum_value,
                start: false,
            }),
            CLASS_CAMERA => out.push(Dispatch::Camera {
                action,
                extent: 0.0,
                start: false,
            }),
            CLASS_UI => out.push(Dispatch::Ui {
                action,
                value: 0.0,
                start: false,
            }),
            _ => {}
        }
    }

    /// `UseTime` / `PurgeSetActions`: one-shot actions last one frame.
    pub fn use_time(&mut self) {
        self.current.retain(|info| info.toggle != TOGGLE_ONE_SHOT);
    }

    /// `FocusSwitchHandler`: losing focus (or entering text mode with focus) releases every held
    /// action, except modifier and UI actions while the window keeps focus.
    pub fn focus_switch(
        &mut self,
        have_focus: bool,
        text_mode: bool,
        now: f64,
        out: &mut Vec<Dispatch>,
    ) {
        if text_mode && have_focus {
            return;
        }
        let held = self.current.clone();
        for info in held {
            let Some(value) = self.action_values(info.action) else {
                continue;
            };
            if !have_focus || (value.class != CLASS_UI && value.class != CLASS_MODIFIER) {
                self.handle(info.action, 0.0, info.key, info.device, now, out);
            }
        }
    }
}
