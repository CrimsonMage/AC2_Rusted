//! `CInputManager_WIN32`: device data in, actions out. Every key and mouse event becomes an input
//! semantic (`GetInput`: keys `0x81000400 | DIK scan code`, mouse axes `0x82000300/304/308`,
//! buttons `0x8200040c..0e`); `ProcessDeviceData` tracks the shift and control meta mode
//! (`SetKey` / `ReleaseKey`, remembering the mode a key went down in so its release resolves the
//! same binding), looks the (semantic, meta mode) pair up through the key-map stack
//! (`GetInputActionFromKey`, the most recently pushed map first) and hands the action and its
//! extent to the action handlers (`CInputManager::CallActionHandler`). In mouse-look mode the
//! pointer axes are held back and their relative motion goes to the mouse-look handler instead.

use std::collections::HashMap;

use crate::maps::KeyMap;

/// Pointer actions `GetInput` raises from the cursor (and `CallActionHandler` drops in mouse look).
pub const ACTION_POINTER_X: u32 = 0x4000_0001;
pub const ACTION_POINTER_Y: u32 = 0x4000_0002;
/// The mouse-button actions whose double clicks raise `action + 3`.
const ACTION_SELECT_FIRST: u32 = 0x4000_0004;
const ACTION_SELECT_LAST: u32 = 0x4000_0006;

pub const SEMANTIC_KEY_BASE: u32 = 0x8100_0400;
pub const SEMANTIC_POINTER_X: u32 = 0x8200_0300;
pub const SEMANTIC_POINTER_Y: u32 = 0x8200_0304;
pub const SEMANTIC_BUTTON_LEFT: u32 = 0x8200_040C;
pub const SEMANTIC_BUTTON_RIGHT: u32 = 0x8200_040D;
pub const SEMANTIC_BUTTON_MIDDLE: u32 = 0x8200_040E;
/// `ProcessDeviceData`'s wheel keys: up (positive) and down.
const SEMANTIC_WHEEL_UP: u32 = 0xFF00_4704;
const SEMANTIC_WHEEL_DOWN: u32 = 0xFF00_4705;
/// `WHEEL_DELTA`.
const WHEEL_DELTA: f32 = 120.0;

const LEFT_SHIFT: u32 = 0x8100_042A;
const RIGHT_SHIFT: u32 = 0x8100_0436;
const LEFT_CONTROL: u32 = 0x8100_041D;
const RIGHT_CONTROL: u32 = 0x8100_049D;

/// Device indices (`m_aDevices`): the keyboard, then the mouse.
pub const DEVICE_KEYBOARD: u32 = 0;
pub const DEVICE_MOUSE: u32 = 1;

/// `CInputManager_WIN32::MetaModeFromKey`.
pub fn meta_mode_from_key(key: u32) -> u32 {
    match key {
        LEFT_CONTROL => 4,
        LEFT_SHIFT => 1,
        RIGHT_SHIFT => 2,
        RIGHT_CONTROL => 8,
        _ => 0,
    }
}

/// `IsMetaKey`.
fn is_meta_key(key: u32) -> bool {
    matches!(key, LEFT_CONTROL | LEFT_SHIFT | RIGHT_SHIFT | RIGHT_CONTROL)
}

/// `SetKey` / `ReleaseKey` fold the right-hand keys onto the left ones.
fn meta_bit(key: u32) -> u32 {
    meta_mode_from_key(match key {
        LEFT_SHIFT | RIGHT_SHIFT => LEFT_SHIFT,
        LEFT_CONTROL | RIGHT_CONTROL => LEFT_CONTROL,
        _ => 0,
    })
}

/// One raw input event, as the window delivers it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    /// A keyboard key by DirectInput scan code (`DIK_*`).
    Key { scan: u32, pressed: bool },
    /// A mouse button: 0 left, 1 right, 2 middle.
    Button { button: u32, pressed: bool },
    /// Wheel motion in `WHEEL_DELTA` units of 120 per notch, positive away from the user.
    Wheel { delta: i32 },
    /// Relative mouse motion (the exclusive mouse's axes).
    Motion { dx: i32, dy: i32 },
    /// The cursor's client-area position (the non-exclusive mouse).
    Cursor { x: i32, y: i32 },
}

/// One action for the handlers (`CallActionHandler`'s arguments).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawAction {
    pub action: u32,
    pub extent: f32,
    /// Milliseconds (`dwTimeStamp` / `GetTickCount`).
    pub timestamp: u32,
    pub key: u32,
    pub device: u32,
}

/// The input manager's state.
#[derive(Debug, Clone, Default)]
pub struct InputManager {
    /// `m_keyToActionMapStack`, front (index 0) the most recently pushed map.
    stack: Vec<HashMap<(u32, u32), u32>>,
    meta_mode: u32,
    /// `m_keyHitTable`: each held non-meta key and the meta mode it went down in.
    key_hits: HashMap<u32, u32>,
    mouse_look: bool,
    swap_buttons: bool,
    mouse_rel: (i32, i32),
    last_cursor: Option<(i32, i32)>,
    cursor: (i32, i32),
    /// `m_doubleClickSet`, and the last click's time, action and cursor (`CheckDoubleClick`).
    double_clicks: Vec<u32>,
    last_click: Option<(u32, u32)>,
    last_click_at: (i32, i32),
    /// `GetSystemMetrics(SM_CXDOUBLECLK / SM_CYDOUBLECLK)` and `GetDoubleClickTime`.
    pub double_click_size: (i32, i32),
    pub double_click_time: u32,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            double_click_size: (4, 4),
            double_click_time: 500,
            ..Self::default()
        }
    }

    /// `PushKeyMap_Internal`: a new map on top of the stack, its bindings keyed by the meta mode
    /// of `meta_key` (0 for none). A semantic bound twice in one map keeps its first action.
    pub fn push_key_map(&mut self, map: &KeyMap, meta_key: u32) {
        let meta = meta_mode_from_key(meta_key);
        let mut table = HashMap::new();
        for mapping in &map.mappings {
            table
                .entry((mapping.semantic, meta))
                .or_insert(mapping.action);
        }
        self.stack.insert(0, table);
    }

    /// `PopKeyMap`.
    pub fn pop_key_map(&mut self) {
        if !self.stack.is_empty() {
            self.stack.remove(0);
        }
    }

    /// `CInputManager::SetMouseLookMode`.
    pub fn set_mouse_look(&mut self, on: bool) {
        self.mouse_look = on;
    }

    pub fn mouse_look(&self) -> bool {
        self.mouse_look
    }

    /// `m_metaKeyMode`.
    pub fn meta_mode(&self) -> u32 {
        self.meta_mode
    }

    /// `GetInputActionFromKey`.
    pub fn action_from_key(&self, key: u32, meta: u32) -> Option<u32> {
        self.stack
            .iter()
            .find_map(|map| map.get(&(key, meta)).copied())
    }

    /// `CheckDoubleClick`: a press inside the double-click rectangle of the last click, of the
    /// same action and within the double-click time, is a double click (its release too).
    fn double_click(&mut self, pressed: bool, timestamp: u32, action: u32) -> bool {
        let mut double = false;
        if !pressed {
            if let Some(index) = self.double_clicks.iter().position(|a| *a == action) {
                self.double_clicks.remove(index);
                double = true;
            } else {
                self.last_click = Some((timestamp, action));
            }
        } else {
            let (w, h) = self.double_click_size;
            let (x, y) = self.cursor;
            let (lx, ly) = self.last_click_at;
            let near =
                (lx - w / 2..=lx + w / 2).contains(&x) && (ly - h / 2..=ly + h / 2).contains(&y);
            if let Some((time, last)) = self.last_click {
                if near && last == action && timestamp.wrapping_sub(time) <= self.double_click_time
                {
                    self.double_clicks.push(action);
                    double = true;
                }
            }
        }
        self.last_click_at = self.cursor;
        double
    }

    /// `ProcessDeviceData` for one key or button transition.
    fn key(
        &mut self,
        key: u32,
        device: u32,
        pressed: bool,
        timestamp: u32,
        out: &mut Vec<RawAction>,
    ) {
        let before = self.meta_mode;
        let meta = if pressed {
            // `SetKey`.
            if is_meta_key(key) {
                self.meta_mode |= meta_bit(key);
            } else {
                self.key_hits.entry(key).or_insert(before);
            }
            before
        } else {
            // `ReleaseKey`.
            if is_meta_key(key) {
                self.meta_mode &= !meta_bit(key);
                0
            } else {
                match self.key_hits.remove(&key) {
                    Some(meta) => meta,
                    None => return,
                }
            }
        };
        let Some(mut action) = self.action_from_key(key, meta) else {
            return;
        };
        if (ACTION_SELECT_FIRST..=ACTION_SELECT_LAST).contains(&action)
            && self.double_click(pressed, timestamp, action)
        {
            action += 3;
        }
        self.handle(
            RawAction {
                action,
                extent: if pressed { 1.0 } else { 0.0 },
                timestamp,
                key,
                device,
            },
            out,
        );
    }

    /// `CInputManager::CallActionHandler`: the pointer actions do not reach the handlers in
    /// mouse-look mode.
    fn handle(&self, action: RawAction, out: &mut Vec<RawAction>) {
        if self.mouse_look && matches!(action.action, ACTION_POINTER_X | ACTION_POINTER_Y) {
            return;
        }
        out.push(action);
    }

    /// One event (`GetInput`'s device loop); the actions it raises are appended to `out`.
    pub fn process(&mut self, event: InputEvent, timestamp: u32, out: &mut Vec<RawAction>) {
        match event {
            InputEvent::Key { scan, pressed } => self.key(
                SEMANTIC_KEY_BASE | scan,
                DEVICE_KEYBOARD,
                pressed,
                timestamp,
                out,
            ),
            InputEvent::Button { button, pressed } => {
                let semantic = match (button, self.swap_buttons) {
                    (0, false) | (1, true) => SEMANTIC_BUTTON_LEFT,
                    (0, true) | (1, false) => SEMANTIC_BUTTON_RIGHT,
                    _ => SEMANTIC_BUTTON_MIDDLE,
                };
                self.key(semantic, DEVICE_MOUSE, pressed, timestamp, out);
            }
            InputEvent::Wheel { delta } => {
                // The wheel is not a held key: it is looked up with the current meta mode.
                let extent = (delta / WHEEL_DELTA as i32) as f32;
                let key = if extent > 0.0 {
                    SEMANTIC_WHEEL_UP
                } else {
                    SEMANTIC_WHEEL_DOWN
                };
                if let Some(action) = self.action_from_key(key, self.meta_mode) {
                    self.handle(
                        RawAction {
                            action,
                            extent,
                            timestamp,
                            key,
                            device: DEVICE_MOUSE,
                        },
                        out,
                    );
                }
            }
            InputEvent::Motion { dx, dy } => {
                // The exclusive mouse's bound axes set (not add to) the frame's relative motion:
                // `m_nMouseRelX = dwData`, so the last buffered delta of a frame is the one the
                // mouse-look handler sees.
                if dx != 0
                    && self
                        .action_from_key(SEMANTIC_POINTER_X, self.meta_mode)
                        .is_some()
                {
                    self.mouse_rel.0 = dx;
                }
                if dy != 0
                    && self
                        .action_from_key(SEMANTIC_POINTER_Y, self.meta_mode)
                        .is_some()
                {
                    self.mouse_rel.1 = dy;
                }
            }
            InputEvent::Cursor { x, y } => {
                self.cursor = (x, y);
                if self.mouse_look {
                    return;
                }
                let (lx, ly) = self.last_cursor.unwrap_or((i32::MIN, i32::MIN));
                if x != lx {
                    self.handle(
                        RawAction {
                            action: ACTION_POINTER_X,
                            extent: x as f32,
                            timestamp,
                            key: SEMANTIC_POINTER_X,
                            device: DEVICE_MOUSE,
                        },
                        out,
                    );
                }
                if y != ly {
                    self.handle(
                        RawAction {
                            action: ACTION_POINTER_Y,
                            extent: y as f32,
                            timestamp,
                            key: SEMANTIC_POINTER_Y,
                            device: DEVICE_MOUSE,
                        },
                        out,
                    );
                }
                self.last_cursor = Some((x, y));
            }
        }
    }

    /// The end of `GetInput`: in mouse-look mode the frame's relative motion
    /// (`CallMouseLookHandler`), then it resets.
    pub fn take_mouse_look(&mut self) -> Option<(i32, i32)> {
        let rel = std::mem::take(&mut self.mouse_rel);
        self.mouse_look.then_some(rel)
    }
}
