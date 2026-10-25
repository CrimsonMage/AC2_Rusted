//! `gmCameraUI`: the camera's part of the game UI. Its global action messages
//! (`ListenToMessage` with `0xffff0003` start / `0xffff0004` stop) re-send the arrow keys as turn
//! and move actions, hold or toggle mouse look and toggle first person, and `UpdateCamera`
//! applies `CameraManager::SetMouseLookState(want && !blocked)` and
//! `SetFirstPersonState(allowed && want)`; its world element's wheel messages zoom
//! (`0x400000b8` in, `0x400000b9` out).

/// UI actions the camera UI handles.
const UI_TURN_LEFT: u32 = 0x4000_0013;
const UI_TURN_RIGHT: u32 = 0x4000_0014;
const UI_FORWARD: u32 = 0x4000_0015;
const UI_BACKWARD: u32 = 0x4000_0016;
const UI_MOUSE_LOOK_HOLD: u32 = 0x4000_002C;
const UI_FIRST_PERSON: u32 = 0x4000_00A2;
const UI_MOUSE_LOOK_TOGGLE: u32 = 0x4000_00BA;
pub const UI_WHEEL_IN: u32 = 0x4000_00B8;
pub const UI_WHEEL_OUT: u32 = 0x4000_00B9;
/// The movement actions the arrow keys stand for.
const TURN_LEFT: u32 = 0x4000_0024;
const TURN_RIGHT: u32 = 0x4000_0025;
const FORWARD: u32 = 0x4000_0020;
const BACKWARD: u32 = 0x4000_0021;

/// What one UI action asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraUiEffect {
    /// Re-send an action to the action interpreter (`(action, extent, time, key = action,
    /// device 1)`).
    Action { action: u32, extent: f32 },
    /// `UpdateCamera`'s states.
    States {
        mouse_look: bool,
        first_person: bool,
    },
    /// `CameraManager::ZoomIn` / `ZoomOut`.
    Zoom { zoom_in: bool },
}

/// `gmCameraUI`'s state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraUi {
    /// `m_canMouseLook`: set, it holds mouse look off.
    pub mouse_look_blocked: bool,
    pub can_first_person: bool,
    pub want_mouse_look: bool,
    pub want_first_person: bool,
}

impl Default for CameraUi {
    /// The constructor's values.
    fn default() -> Self {
        Self {
            mouse_look_blocked: false,
            can_first_person: true,
            want_mouse_look: false,
            want_first_person: false,
        }
    }
}

impl CameraUi {
    /// `UpdateCamera`.
    pub fn update_camera(&self) -> CameraUiEffect {
        CameraUiEffect::States {
            mouse_look: !self.mouse_look_blocked && self.want_mouse_look,
            first_person: self.can_first_person && self.want_first_person,
        }
    }

    /// `ListenToMessage` for a UI action starting or stopping.
    pub fn action(&mut self, action: u32, start: bool) -> Option<CameraUiEffect> {
        let extent = if start { 1.0 } else { 0.0 };
        let resend = |action| Some(CameraUiEffect::Action { action, extent });
        match action {
            UI_TURN_LEFT => resend(TURN_LEFT),
            UI_TURN_RIGHT => resend(TURN_RIGHT),
            UI_FORWARD => resend(FORWARD),
            UI_BACKWARD => resend(BACKWARD),
            UI_MOUSE_LOOK_HOLD => {
                // Start turns it on; the stop toggles it (back off).
                if start && self.want_mouse_look {
                    return None;
                }
                self.want_mouse_look = if start { true } else { !self.want_mouse_look };
                Some(self.update_camera())
            }
            UI_FIRST_PERSON if start => {
                self.want_first_person = !self.want_first_person;
                Some(self.update_camera())
            }
            UI_MOUSE_LOOK_TOGGLE if start => {
                self.want_mouse_look = !self.want_mouse_look;
                Some(self.update_camera())
            }
            // The world element's wheel messages.
            UI_WHEEL_IN if start => Some(CameraUiEffect::Zoom { zoom_in: true }),
            UI_WHEEL_OUT if start => Some(CameraUiEffect::Zoom { zoom_in: false }),
            _ => None,
        }
    }
}
