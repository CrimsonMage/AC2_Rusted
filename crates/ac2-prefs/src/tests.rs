//! The preferences without game data: the C number rules, each type's text and constraints, the
//! bank's load and save over the INI bytes, and the session cache. The names are fixed here; the
//! client's are the DAT's enum tables.

use std::collections::HashMap;

use crate::cnum;
use crate::names::NameTable;
use crate::pref::{Choice, Pref, Value};
use crate::{PreferenceBank, Preferences};

const SOUND: u32 = 0x4000_0007;
const FX: u32 = 0x4000_0040;
const DISABLE: u32 = 0x4000_0033;
const FEATURES: u32 = 0x4000_0057;
const MAP: u32 = 0x4000_004A;
const LABEL_SOFTWARE: u32 = 0x4000_000F;
const LABEL_HARDWARE: u32 = 0x4000_0010;
const LABEL_EAX: u32 = 0x4000_0011;

/// The test's tables: sections, keys, and one enum mapper.
#[derive(Default)]
struct Names {
    sections: HashMap<u32, &'static str>,
    keys: HashMap<u32, &'static str>,
    labels: HashMap<u32, &'static str>,
}

impl Names {
    fn sound() -> Self {
        let mut names = Self::default();
        names.sections.insert(SOUND, "Sound");
        names.keys.insert(FX, "FXVolume");
        names.keys.insert(DISABLE, "DisableSound");
        names.keys.insert(FEATURES, "SoundFeatures");
        names.labels.insert(LABEL_SOFTWARE, "Software");
        names.labels.insert(LABEL_HARDWARE, "Hardware");
        names.labels.insert(LABEL_EAX, "EAX");
        names
    }
}

impl NameTable for Names {
    fn section_name(&self, id: u32) -> Option<String> {
        self.sections.get(&id).map(|s| (*s).to_owned())
    }
    fn key_name(&self, id: u32) -> Option<String> {
        self.keys.get(&id).map(|s| (*s).to_owned())
    }
    fn enum_name(&self, map: u32, label: u32) -> Option<String> {
        (map == MAP)
            .then(|| self.labels.get(&label).map(|s| (*s).to_owned()))
            .flatten()
    }
    fn enum_label(&self, map: u32, name: &str) -> Option<u32> {
        if map != MAP {
            return None;
        }
        self.labels
            .iter()
            .find(|(_, n)| n.eq_ignore_ascii_case(name))
            .map(|(id, _)| *id)
    }
}

fn features(default: i32) -> Pref {
    let choices = vec![
        Choice {
            value: 0,
            label: LABEL_SOFTWARE,
        },
        Choice {
            value: 1,
            label: LABEL_HARDWARE,
        },
        Choice {
            value: 2,
            label: LABEL_EAX,
        },
    ];
    Pref::enumeration(SOUND, FEATURES, Some(MAP), choices, default)
        .expect("the default is a choice")
}

// --- C number parsing ----------------------------------------------------------------------

#[test]
fn c_integers_take_base_zero_and_a_whole_string() {
    assert!(cnum::is_int32("42"));
    assert!(cnum::is_int32("-7"));
    assert!(cnum::is_int32("0x1F"));
    assert!(cnum::is_int32("010"));
    assert_eq!(cnum::strtol(b"010").value, 8, "octal with a leading zero");
    assert!(!cnum::is_int32("12 "), "trailing text is not a number");
    assert!(cnum::is_int32("  12"), "leading white space is skipped");
    assert!(!cnum::is_int32("08"), "8 is not an octal digit");
    assert!(!cnum::is_int32("2147483648"), "out of a 32-bit long");
    assert!(
        cnum::is_int32(""),
        "an empty string is a number (its end is the NUL)"
    );
    assert!(!cnum::is_int32("abc"));
    assert!(
        !cnum::is_int32("0x"),
        "0x with no digit stops after the zero"
    );
}

#[test]
fn c_unsigned_integers_negate_and_refuse_overflow() {
    assert_eq!(cnum::strtoul(b"0x10").value, 16);
    assert_eq!(cnum::strtoul(b"-1").value, u32::MAX);
    assert!(cnum::is_uint32("4294967295"));
    assert!(!cnum::is_uint32("4294967296"));
}

#[test]
fn c_doubles_take_decimal_exponent_inf_nan_and_hex() {
    assert!(cnum::is_double("0.5"));
    assert!(cnum::is_double(".5"));
    assert!(cnum::is_double("5."));
    assert!(cnum::is_double("1e3"));
    assert!(cnum::is_double("-2.5E-1"));
    assert!(cnum::is_double("inf"));
    assert!(cnum::is_double("nan"));
    assert!(cnum::is_double("0x1p3"));
    assert_eq!(cnum::strtod(b"0x1p3").value, 8.0);
    assert!(!cnum::is_double("1e400"), "an overflow is a range error");
    assert!(!cnum::is_double("1.5x"));
    assert!(
        !cnum::is_double("."),
        "a point with no digit is not a number"
    );
    assert!(cnum::is_double(""));
}

// --- bool ----------------------------------------------------------------------------------

#[test]
fn a_bool_is_zero_or_one_in_any_case_and_other_text_keeps_the_value() {
    let names = Names::sound();
    let mut pref = Pref::boolean(SOUND, DISABLE, false);
    assert!(pref.set_text("1", &names));
    assert_eq!(pref.value, Value::Bool(true));
    assert!(pref.set_text("0", &names));
    assert!(!pref.set_text("true", &names), "only 0 and 1 name a bool");
    assert_eq!(pref.value, Value::Bool(false));
    assert_eq!(pref.value_text(&names), "0");
}

#[test]
fn a_bool_default_text_that_names_nothing_still_resets_the_value() {
    let names = Names::sound();
    let mut pref = Pref::boolean(SOUND, DISABLE, false);
    pref.set_bool(true);
    assert!(!pref.set_default_text("maybe", &names));
    assert_eq!(pref.default, Value::Bool(false), "the default is unchanged");
    assert_eq!(
        pref.value,
        Value::Bool(false),
        "the value follows the default"
    );
}

// --- float ---------------------------------------------------------------------------------

#[test]
fn a_float_is_written_with_six_decimals_and_parsed_with_strtod() {
    let names = Names::sound();
    let mut pref = Pref::float(SOUND, FX, 1.0, 0.0, 1.0).unwrap();
    assert_eq!(pref.default_text(&names), "1.000000");
    assert!(pref.set_text("0.25", &names));
    assert_eq!(pref.value, Value::Float(0.25));
    assert_eq!(pref.value_text(&names), "0.250000");
    assert!(
        !pref.set_text("loud", &names),
        "not a number: the value is kept"
    );
    assert_eq!(pref.value, Value::Float(0.25));
}

#[test]
fn a_float_range_is_stored_and_an_inverted_one_is_refused() {
    let mut pref = Pref::float(SOUND, FX, 0.5, 0.0, 1.0).unwrap();
    assert!(!pref.set_range(2.0, 1.0), "SetRange refuses max below min");
    assert_eq!((pref.min, pref.max), (0.0, 1.0));
    assert!(pref.set_range(8.0, 64.0));
    assert_eq!((pref.min, pref.max), (8.0, 64.0));
    assert!(Pref::float(SOUND, FX, 0.5, 1.0, 0.0).is_none());
}

#[test]
fn a_float_is_not_clamped_by_its_setters_the_slider_clamps_it() {
    // Retail's `SetValue(float)` stores what it is given; the range is what the slider shows.
    let mut pref = Pref::float(SOUND, FX, 0.5, 0.0, 1.0).unwrap();
    pref.set_float(5.0);
    assert_eq!(pref.value, Value::Float(5.0));
    let names = Names::sound();
    assert!(pref.set_text("-3", &names));
    assert_eq!(pref.value, Value::Float(-3.0));
}

// --- enum ----------------------------------------------------------------------------------

#[test]
fn an_enum_takes_a_name_or_an_integer_that_is_a_choice() {
    let names = Names::sound();
    let mut pref = features(1);
    assert_eq!(
        pref.default_text(&names),
        "Hardware",
        "the default is named by its label"
    );
    assert!(pref.set_text("EAX", &names), "a name is case-insensitive");
    assert_eq!(pref.value, Value::Int(2));
    assert!(pref.set_text("0", &names), "an integer is a choice value");
    assert_eq!(pref.value, Value::Int(0));
    assert!(pref.set_text("0x1", &names), "base 0 integers");
    assert_eq!(pref.value, Value::Int(1));
    assert!(!pref.set_text("9", &names), "9 is not a choice");
    assert!(!pref.set_text("Surround", &names), "no such name");
    assert_eq!(pref.value, Value::Int(1), "a refused text keeps the value");
}

#[test]
fn an_enum_without_a_name_is_written_as_its_integer() {
    let names = Names::default();
    let pref = Pref::enumeration(
        SOUND,
        FEATURES,
        None,
        vec![Choice { value: 3, label: 0 }],
        3,
    )
    .unwrap();
    assert_eq!(pref.value_text(&names), "3");
}

#[test]
fn an_enum_default_must_be_a_choice() {
    assert!(Pref::enumeration(SOUND, FEATURES, Some(MAP), Vec::new(), 1).is_none());
    let mut pref = features(1);
    assert!(!pref.set_default(Value::Int(7)));
    assert_eq!(pref.default, Value::Int(1));
    assert!(pref.set_default(Value::Int(0)));
    assert_eq!(
        pref.value,
        Value::Int(0),
        "a default sets the value with it"
    );
}

// --- string and the apply, recall and reset of a value -------------------------------------

#[test]
fn a_string_is_stored_as_it_is_and_recall_and_reset_return_it() {
    let names = Names::sound();
    let mut pref = Pref::text(SOUND, FX, "dflt");
    assert!(pref.set_text("  spaced  ", &names));
    assert_eq!(pref.value_text(&names), "  spaced  ");
    pref.apply();
    pref.set_text("other", &names);
    assert!(pref.changed());
    pref.recall();
    assert_eq!(pref.value, Value::Text("  spaced  ".to_owned()));
    pref.reset();
    assert_eq!(pref.value, Value::Text("dflt".to_owned()));
}

// --- the bank: registration, load, save ----------------------------------------------------

#[test]
fn a_registered_preference_takes_the_file_value_and_defaults_without_one() {
    let names = Names::sound();
    let mut bank = PreferenceBank::new(b"[Sound]\r\nFXVolume=0.250000\r\nSoundFeatures=EAX\r\n");
    bank.register(Pref::float(SOUND, FX, 1.0, 0.0, 1.0).unwrap(), &names);
    bank.register(Pref::boolean(SOUND, DISABLE, true), &names);
    bank.register(features(1), &names);
    assert_eq!(bank.find(FX).unwrap().value, Value::Float(0.25));
    assert_eq!(
        bank.find(DISABLE).unwrap().value,
        Value::Bool(true),
        "missing: the default"
    );
    assert_eq!(bank.find(FEATURES).unwrap().value, Value::Int(2));
    assert!(
        !bank.find(FX).unwrap().changed(),
        "a loaded value is the applied one"
    );
}

#[test]
fn an_invalid_file_value_keeps_the_default() {
    let names = Names::sound();
    let mut bank = PreferenceBank::new(b"[Sound]\r\nDisableSound=yes\r\n");
    bank.register(Pref::boolean(SOUND, DISABLE, false), &names);
    assert_eq!(bank.find(DISABLE).unwrap().value, Value::Bool(false));
}

#[test]
fn a_loaded_text_is_cut_to_ninety_nine_bytes() {
    let names = Names::sound();
    let long = "x".repeat(150);
    let file = format!("[Sound]\r\nFXVolume={long}\r\n");
    let mut bank = PreferenceBank::new(file.as_bytes());
    bank.register(Pref::text(SOUND, FX, ""), &names);
    assert_eq!(
        bank.find(FX).unwrap().value,
        Value::Text("x".repeat(99)),
        "Preference::Load reads into a 100-byte buffer"
    );
}

#[test]
fn a_preference_with_no_name_is_not_registered() {
    let mut bank = PreferenceBank::new(b"");
    assert!(bank
        .register(Pref::boolean(SOUND, DISABLE, true), &Names::default())
        .is_none());
    assert!(bank.all().is_empty());
}

#[test]
fn saving_writes_every_preference_in_list_order_and_reloads_to_the_same_values() {
    let names = Names::sound();
    let mut bank = PreferenceBank::new(b"; kept\r\n[Camera]\r\nSpeed=2\r\n");
    bank.register(Pref::float(SOUND, FX, 1.0, 0.0, 1.0).unwrap(), &names);
    bank.register(Pref::boolean(SOUND, DISABLE, false), &names);
    bank.register(features(1), &names);
    bank.find_mut(DISABLE).unwrap().set_bool(true);
    bank.find_mut(FEATURES).unwrap().set_choice(2);
    bank.save(&names);
    let saved = bank.file_bytes();
    assert_eq!(
        saved,
        b"; kept\r\n[Camera]\r\nSpeed=2\r\n[Sound]\r\nFXVolume=1.000000\r\nDisableSound=1\r\nSoundFeatures=EAX\r\n"
            .to_vec(),
        "new keys append to their section; the rest of the file is as it was"
    );
    let mut again = PreferenceBank::new(&saved);
    again.register(Pref::float(SOUND, FX, 0.0, 0.0, 1.0).unwrap(), &names);
    again.register(Pref::boolean(SOUND, DISABLE, false), &names);
    again.register(features(1), &names);
    assert_eq!(again.find(FX).unwrap().value, Value::Float(1.0));
    assert_eq!(again.find(DISABLE).unwrap().value, Value::Bool(true));
    assert_eq!(again.find(FEATURES).unwrap().value, Value::Int(2));
}

#[test]
fn an_unchanged_file_saves_back_byte_for_byte() {
    let names = Names::sound();
    let bytes = b"[Sound]\r\nFXVolume=1.000000\r\nDisableSound=0\r\n";
    let mut bank = PreferenceBank::new(bytes);
    bank.register(Pref::float(SOUND, FX, 1.0, 0.0, 1.0).unwrap(), &names);
    bank.register(Pref::boolean(SOUND, DISABLE, false), &names);
    bank.save(&names);
    assert_eq!(bank.file_bytes(), bytes.to_vec());
}

#[test]
fn a_preference_with_no_name_is_skipped_by_save() {
    let names = Names::sound();
    let mut bank = PreferenceBank::new(b"");
    bank.register(Pref::float(SOUND, FX, 1.0, 0.0, 1.0).unwrap(), &names);
    bank.save(&Names::default());
    assert!(bank.file_bytes().is_empty());
}

// --- sections: submit, apply, recall, reload, and owners -----------------------------------

#[test]
fn submit_lists_the_changes_and_apply_commits_them() {
    let names = Names::sound();
    let mut bank = PreferenceBank::new(b"");
    bank.register(Pref::float(SOUND, FX, 1.0, 0.0, 1.0).unwrap(), &names);
    bank.register(Pref::boolean(SOUND, DISABLE, false), &names);
    bank.find_mut(DISABLE).unwrap().set_bool(true);
    assert_eq!(bank.submit(SOUND), vec![DISABLE], "only the changed one");
    assert_eq!(bank.apply(SOUND), vec![DISABLE]);
    assert!(bank.submit(SOUND).is_empty());
}

#[test]
fn recall_returns_every_preference_of_a_section_to_its_applied_value() {
    let names = Names::sound();
    let mut bank = PreferenceBank::new(b"");
    bank.register(Pref::float(SOUND, FX, 1.0, 0.0, 1.0).unwrap(), &names);
    bank.find_mut(FX).unwrap().set_float(0.1);
    assert_eq!(bank.recall(SOUND), vec![FX]);
    assert_eq!(bank.find(FX).unwrap().value, Value::Float(1.0));
    assert_eq!(bank.reload(SOUND), vec![FX]);
    assert!(bank.apply_key(FX));
    assert!(!bank.apply_key(0x4000_0099));
}

#[test]
fn owners_are_the_subsystems_that_link_a_key_once_each() {
    let mut bank = PreferenceBank::new(b"");
    bank.link(FX, "SoundManager");
    bank.link(FX, "SoundManager");
    bank.link(FX, "CameraManager");
    assert_eq!(bank.owners(FX), &["SoundManager", "CameraManager"]);
    assert!(bank.owners(DISABLE).is_empty());
}

// --- the session cache ---------------------------------------------------------------------

#[test]
fn the_cache_reads_its_session_values_first_then_the_file() {
    let names = Names::sound();
    let mut cache = Preferences::new(b"[Sound]\r\nDisableSound=0x10\r\nFXVolume=words\r\n");
    assert_eq!(
        cache.inq_bool(&names, SOUND, DISABLE),
        Some(16),
        "base 0 unsigned"
    );
    assert_eq!(
        cache.inq_bool(&names, SOUND, FX),
        None,
        "not an unsigned integer"
    );
    cache.set_bool(SOUND, DISABLE, false);
    assert_eq!(
        cache.inq_bool(&names, SOUND, DISABLE),
        Some(0),
        "the session wins"
    );
    cache.set_uint32(SOUND, FX, 7);
    assert_eq!(cache.inq_string(&names, SOUND, FX).as_deref(), Some("7"));
    assert_eq!(cache.inq_string(&names, SOUND, 0x4000_0099), None);
}

#[test]
fn a_session_value_is_not_written_to_the_file() {
    let names = Names::sound();
    let mut bank = PreferenceBank::new(b"");
    let mut cache = Preferences::new(&bank.file_bytes());
    cache.set_bool(SOUND, DISABLE, true);
    bank.save(&names);
    assert!(
        bank.file_bytes().is_empty(),
        "retail's SetBool never writes the file"
    );
}

// --- paths ---------------------------------------------------------------------------------

#[test]
fn the_default_file_is_under_the_user_config_directory_not_the_game() {
    let path = crate::paths::default_file();
    assert!(path.ends_with(std::path::Path::new("AC2").join("Preferences.ini")));
    let relative = crate::paths::resolve("Preferences.ini");
    assert!(relative.is_absolute());
    assert_eq!(
        crate::paths::resolve("/tmp/x.ini"),
        std::path::PathBuf::from("/tmp/x.ini")
    );
}

#[test]
fn a_missing_file_reads_as_empty_and_a_written_file_reads_back() {
    let dir = std::env::temp_dir().join(format!("ac2-prefs-test-{}", std::process::id()));
    let path = dir.join("Preferences.ini");
    assert!(crate::paths::read_or_empty(&path).unwrap().is_empty());
    crate::paths::write_atomic(&path, b"[Sound]\r\nX=1\r\n").unwrap();
    assert_eq!(
        crate::paths::read_or_empty(&path).unwrap(),
        b"[Sound]\r\nX=1\r\n".to_vec()
    );
    let _ = std::fs::remove_dir_all(&dir);
}
