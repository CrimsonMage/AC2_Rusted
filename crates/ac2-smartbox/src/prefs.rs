//! The camera preferences (`CameraManager::SetupPrefs`, admin 0x005751C0; Legions 0x00545090).
//! The function registers fourteen keys in three sections: section `0x40000008` (eight: six
//! floats and two bools), `0x4000000E` (three: a float and two bools) and `0x4000000D` (three
//! bools). Each is linked to the camera manager (`PreferenceBank::Link`), which reads it when its
//! value changes.
//!
//! The defaults and ranges are the ones the function sets (`SetRange` and `SetDefault`); the bank
//! does not clamp a value to its range (retail's setters do not either).

use ac2_prefs::{NameTable, Pref, PreferenceBank, Value};

/// The camera's sections.
pub const CAMERA_SECTION: u32 = 0x4000_0008;
pub const CAMERA_SECTION_E: u32 = 0x4000_000E;
pub const CAMERA_SECTION_D: u32 = 0x4000_000D;

/// The owner the bank records for the camera's keys.
pub const OWNER: &str = "CameraManager";

/// The keys [`values`] reads: the default distance (`Camera_Distance`), the tilt (`Camera_Tilt`),
/// the rotate speed (`Camera_Rotate_Speed`), the mouse-look sensitivity and Y inversion
/// (`Input_Mouse_Look_Sensitivity`, `Input_Invert_Y`), the third-person steering and the
/// align-to-slope flag.
const DISTANCE: u32 = 0x4000_002A;
const TILT: u32 = 0x4000_0054;
const ROTATE_SPEED: u32 = 0x4000_002D;
const MOUSE_SENSITIVITY: u32 = 0x4000_0025;
const INVERT_Y: u32 = 0x4000_0026;
const STEER_PLAYER: u32 = 0x4000_006B;
const ALIGN_TO_SLOPE: u32 = 0x4000_005A;
/// `Camera_Zoom_Speed` (`CameraManager::SetupPrefs`'s `m_zoomScale`).
const ZOOM_SPEED: u32 = 0x4000_002C;
/// The look-at-selection switch: `CameraManager::HandleChange` passes it to the player's
/// `CPhysicsObj::SetLookAtSelection` (ICPhysicsObj slot 0x4DC, read from the admin exe at
/// 0x00577003).
const LOOK_AT_SELECTION: u32 = 0x4000_006A;

/// The camera's preference values, as the bank holds them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraValues {
    pub distance: f32,
    pub tilt: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
    pub steer_player: bool,
    pub align_to_slope: bool,
    /// The player looks at what it selects (`SetLookAtSelection`).
    pub look_at_selection: bool,
}

/// The camera's values in `bank` (a missing key keeps the camera's constructor default).
pub fn values(bank: &PreferenceBank) -> CameraValues {
    let float = |key: u32, default: f32| match bank.find(key).map(|p| &p.value) {
        Some(Value::Float(v)) => *v,
        _ => default,
    };
    let flag = |key: u32, default: bool| match bank.find(key).map(|p| &p.value) {
        Some(Value::Bool(v)) => *v,
        _ => default,
    };
    CameraValues {
        distance: float(DISTANCE, 4.5),
        tilt: float(TILT, 0.25),
        rotate_speed: float(ROTATE_SPEED, 1.0),
        zoom_speed: float(ZOOM_SPEED, 1.0),
        mouse_sensitivity: float(MOUSE_SENSITIVITY, 1.0),
        invert_y: flag(INVERT_Y, false),
        steer_player: flag(STEER_PLAYER, true),
        align_to_slope: flag(ALIGN_TO_SLOPE, true),
        look_at_selection: flag(LOOK_AT_SELECTION, true),
    }
}

/// Registers the camera preferences in `bank`, in the order `SetupPrefs` creates them, and links
/// them to the camera manager. Returns how many were registered.
pub fn register(bank: &mut PreferenceBank, names: &dyn NameTable) -> usize {
    let defs = [
        Pref::float(CAMERA_SECTION, 0x4000_0027, 0.0, -1.0, 1.0),
        Pref::float(CAMERA_SECTION, 0x4000_002A, 4.5, -0.2, 10.0),
        Pref::float(CAMERA_SECTION, 0x4000_0054, 0.25, -0.25, 1.0),
        Pref::float(CAMERA_SECTION, 0x4000_002F, 1.0, 0.1, 2.0),
        Pref::float(CAMERA_SECTION, 0x4000_002D, 1.0, 0.1, 5.0),
        Pref::float(CAMERA_SECTION, 0x4000_002C, 1.0, 0.1, 5.0),
        Some(Pref::boolean(CAMERA_SECTION, 0x4000_0051, true)),
        Some(Pref::boolean(CAMERA_SECTION, 0x4000_005A, true)),
        Pref::float(CAMERA_SECTION_E, 0x4000_0025, 1.0, 0.5, 2.0),
        Some(Pref::boolean(CAMERA_SECTION_E, 0x4000_0026, false)),
        Some(Pref::boolean(CAMERA_SECTION_E, 0x4000_006B, true)),
        Some(Pref::boolean(CAMERA_SECTION_D, 0x4000_0068, true)),
        Some(Pref::boolean(CAMERA_SECTION_D, 0x4000_0069, true)),
        Some(Pref::boolean(CAMERA_SECTION_D, 0x4000_006A, true)),
    ];
    let mut registered = 0;
    for def in defs.into_iter().flatten() {
        let key = def.key;
        if bank.register(def, names).is_some() {
            bank.link(key, OWNER);
            registered += 1;
        }
    }
    registered
}

#[cfg(test)]
#[path = "prefs_tests.rs"]
mod prefs_tests;
