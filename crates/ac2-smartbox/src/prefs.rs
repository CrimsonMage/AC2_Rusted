//! The camera preferences (`CameraManager::SetupPrefs`, admin 0x005751C0; Legions 0x00545090).
//! The function registers fourteen keys in three sections: section `0x40000008` (eight: six
//! floats and two bools), `0x4000000E` (three: a float and two bools) and `0x4000000D` (three
//! bools). Each is linked to the camera manager (`PreferenceBank::Link`), which reads it when its
//! value changes.
//!
//! The defaults and ranges are the ones the function sets (`SetRange` and `SetDefault`); the bank
//! does not clamp a value to its range (retail's setters do not either).

use ac2_prefs::{NameTable, Pref, PreferenceBank};

/// The camera's sections.
pub const CAMERA_SECTION: u32 = 0x4000_0008;
pub const CAMERA_SECTION_E: u32 = 0x4000_000E;
pub const CAMERA_SECTION_D: u32 = 0x4000_000D;

/// The owner the bank records for the camera's keys.
pub const OWNER: &str = "CameraManager";

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
