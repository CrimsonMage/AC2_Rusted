//! The camera preferences: fourteen keys in their three sections, with the function's defaults.

use ac2_prefs::{NameTable, PreferenceBank, Value};

use super::{register, CAMERA_SECTION, CAMERA_SECTION_D, CAMERA_SECTION_E, OWNER};

/// Names for the test: every section and key is named (as the DAT names them).
struct Names;

impl NameTable for Names {
    fn section_name(&self, id: u32) -> Option<String> {
        Some(format!("Section{id:08X}"))
    }

    fn key_name(&self, id: u32) -> Option<String> {
        Some(format!("Key{id:08X}"))
    }

    fn enum_name(&self, _map: u32, _label: u32) -> Option<String> {
        None
    }

    fn enum_label(&self, _map: u32, _name: &str) -> Option<u32> {
        None
    }
}

#[test]
fn fourteen_keys_register_in_their_sections() {
    let mut bank = PreferenceBank::new(b"");
    assert_eq!(register(&mut bank, &Names), 14);
    assert_eq!(
        bank.list(CAMERA_SECTION),
        vec![
            0x4000_0027,
            0x4000_002A,
            0x4000_0054,
            0x4000_002F,
            0x4000_002D,
            0x4000_002C,
            0x4000_0051,
            0x4000_005A
        ]
    );
    assert_eq!(
        bank.list(CAMERA_SECTION_E),
        vec![0x4000_0025, 0x4000_0026, 0x4000_006B]
    );
    assert_eq!(
        bank.list(CAMERA_SECTION_D),
        vec![0x4000_0068, 0x4000_0069, 0x4000_006A]
    );
    assert_eq!(bank.owners(0x4000_0027), [OWNER]);
}

#[test]
fn the_defaults_and_ranges_are_the_ones_the_function_sets() {
    let mut bank = PreferenceBank::new(b"");
    register(&mut bank, &Names);
    let pref = |key| bank.find(key).expect("registered");
    assert_eq!(pref(0x4000_002A).value, Value::Float(4.5));
    assert_eq!((pref(0x4000_002A).min, pref(0x4000_002A).max), (-0.2, 10.0));
    assert_eq!(pref(0x4000_0054).value, Value::Float(0.25));
    assert_eq!(pref(0x4000_0025).value, Value::Float(1.0));
    assert_eq!(pref(0x4000_0026).value, Value::Bool(false));
    assert_eq!(pref(0x4000_0051).value, Value::Bool(true));
}
