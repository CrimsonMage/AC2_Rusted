//! The client's preferences on the real DATs (`AC2_DAT_DIR`): the section, key and label names
//! come from the DAT's enum tables, the sound preferences register with their defaults, and a
//! save and a load through those names round-trip. The names are read at run time and never
//! stored; the files are written to a temporary directory, never next to the DATs.

use std::path::PathBuf;
use std::sync::Arc;

use ac2_audio::prefs::{
    AMBIENT_VOLUME, DISABLE_MUSIC, DISABLE_SOUND, FX_VOLUME, MUSIC_VOLUME, NUM_PLAYING_SOUNDS,
    PLAYER_MUSIC, SECTION, SOUND_FEATURES, SOUND_FEATURES_MAP,
};
use ac2_client::{ClientPrefs, DatNames};
use ac2_prefs::{NameTable, Value};

const LABELS: [u32; 3] = [0x4000_000F, 0x4000_0010, 0x4000_0011];

fn store() -> Option<Arc<ac2_db::ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ac2_db::ResourceStore::new(dataset)))
}

/// A file in its own directory per test (tests run in parallel and each removes its directory).
fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join(format!("ac2-client-prefs-{}-{name}", std::process::id()))
        .join(name)
}

#[test]
fn the_sound_keys_and_their_labels_have_names_in_the_dat() {
    let Some(store) = store() else {
        return;
    };
    let names = DatNames::new(&store);
    assert!(
        names.section_name(SECTION).is_some(),
        "the sound section is named"
    );
    let keys = [
        FX_VOLUME,
        MUSIC_VOLUME,
        PLAYER_MUSIC,
        AMBIENT_VOLUME,
        SOUND_FEATURES,
        NUM_PLAYING_SOUNDS,
        DISABLE_SOUND,
        DISABLE_MUSIC,
    ];
    let mut seen = std::collections::BTreeSet::new();
    for key in keys {
        let name = names
            .key_name(key)
            .unwrap_or_else(|| panic!("{key:08X} is named"));
        assert!(seen.insert(name), "key names are distinct");
    }
    for label in LABELS {
        let name = names
            .enum_name(SOUND_FEATURES_MAP, label)
            .unwrap_or_else(|| panic!("label {label:08X} is named"));
        assert_eq!(names.enum_label(SOUND_FEATURES_MAP, &name), Some(label));
        assert_eq!(
            names.enum_label(SOUND_FEATURES_MAP, &name.to_uppercase()),
            Some(label)
        );
    }
}

#[test]
fn the_sound_preferences_register_with_their_defaults_from_the_dat_names() {
    let Some(store) = store() else {
        return;
    };
    let prefs = ClientPrefs::load(temp_file("unused.ini"), store).expect("no file is read");
    let bank = prefs.bank();
    assert_eq!(bank.find(FX_VOLUME).unwrap().value, Value::Float(1.0));
    assert_eq!(
        bank.find(NUM_PLAYING_SOUNDS).unwrap().value,
        Value::Float(32.0)
    );
    assert_eq!(bank.find(DISABLE_SOUND).unwrap().value, Value::Bool(false));
    let features = bank.find(SOUND_FEATURES).unwrap();
    assert_eq!(features.value, Value::Int(1));
    let text = features.default_text(prefs.names());
    assert!(
        !text.is_empty() && text.parse::<i32>().is_err(),
        "a label name: {text}"
    );
}

#[test]
fn a_save_writes_the_named_keys_and_a_load_reads_the_same_values_back() {
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("Preferences.ini");
    let _ = std::fs::remove_file(&path);
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    prefs
        .bank_mut()
        .find_mut(FX_VOLUME)
        .unwrap()
        .set_float(0.25);
    prefs
        .bank_mut()
        .find_mut(DISABLE_MUSIC)
        .unwrap()
        .set_bool(true);
    prefs
        .bank_mut()
        .find_mut(SOUND_FEATURES)
        .unwrap()
        .set_choice(2);
    prefs
        .save()
        .join()
        .expect("the writer thread")
        .expect("the file is written");
    let text = String::from_utf8(std::fs::read(&path).expect("written")).expect("ASCII names");
    let section = prefs.names().section_name(SECTION).expect("section name");
    assert!(
        text.contains(&format!("[{section}]\r\n")),
        "the section header: {text}"
    );
    let fx = prefs.names().key_name(FX_VOLUME).expect("key name");
    assert!(text.contains(&format!("{fx}=0.250000\r\n")), "{text}");
    let back = ClientPrefs::load(path.clone(), store).expect("reload");
    assert_eq!(
        back.bank().find(FX_VOLUME).unwrap().value,
        Value::Float(0.25)
    );
    assert_eq!(
        back.bank().find(DISABLE_MUSIC).unwrap().value,
        Value::Bool(true)
    );
    assert_eq!(
        back.bank().find(SOUND_FEATURES).unwrap().value,
        Value::Int(2)
    );
    let _ = std::fs::remove_dir_all(path.parent().expect("a directory"));
}

#[test]
fn the_graphics_quality_is_written_at_once_and_reads_back_through_the_dat_names() {
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("Quality.ini");
    let _ = std::fs::remove_file(&path);
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    assert_eq!(
        prefs.overall_graphics_quality(),
        None,
        "no quality is saved yet"
    );
    prefs
        .save_overall_graphics_quality(3)
        .join()
        .expect("the writer thread")
        .expect("the file is written");
    let text = String::from_utf8(std::fs::read(&path).expect("written")).expect("ASCII");
    assert!(
        text.contains("[Graphics]\r\nOverallGraphicsQuality=3\r\n"),
        "retail's literal section and key: {text}"
    );
    assert_eq!(
        prefs.overall_graphics_quality(),
        Some(3),
        "the session value"
    );
    let back = ClientPrefs::load(path.clone(), store).expect("reload");
    assert_eq!(
        back.overall_graphics_quality(),
        Some(3),
        "the file value, by the DAT's names"
    );
    let _ = std::fs::remove_dir_all(path.parent().expect("a directory"));
}

#[test]
fn a_first_run_takes_the_hardware_quality_and_a_saved_quality_wins() {
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("StartupQuality.ini");
    let _ = std::fs::remove_file(&path);
    let mut fresh = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    assert_eq!(
        fresh.scene_options().degrade.model_detail,
        1,
        "registered default"
    );
    assert_eq!(
        fresh.apply_startup_quality(5),
        5,
        "nothing saved: the hardware level"
    );
    let degrade = fresh.scene_options().degrade;
    assert_eq!(
        (degrade.model_detail, degrade.scenery_draw_distance),
        (2, 2),
        "level 5's high model detail and scenery distance"
    );
    assert_eq!(
        fresh.overall_graphics_quality(),
        None,
        "the first-run level is not saved"
    );
    fresh
        .save_overall_graphics_quality(2)
        .join()
        .expect("the writer thread")
        .expect("the file is written");
    let mut saved = ClientPrefs::load(path.clone(), store).expect("reload");
    assert_eq!(saved.apply_startup_quality(5), 2, "the saved level wins");
    assert_eq!(
        saved.scene_options().degrade.model_detail,
        0,
        "level 2's low model detail"
    );
    let _ = std::fs::remove_dir_all(path.parent().expect("a directory"));
}

/// `UIElementManager::Init`: the tooltip delay and switch are named in the DAT and register with
/// the manager's defaults.
#[test]
fn the_tooltip_preferences_register_from_the_dat_names() {
    let Some(store) = store() else {
        return;
    };
    let prefs = ClientPrefs::load(temp_file("tooltip.ini"), store).expect("no file is read");
    let bank = prefs.bank();
    // The section also holds the client's own HUD style key, after the manager's two.
    assert_eq!(
        bank.list(ac2_ui::tooltip_prefs::TOOLTIP_SECTION),
        vec![
            ac2_ui::tooltip_prefs::TOOLTIP_DELAY,
            ac2_ui::tooltip_prefs::TOOLTIP_ENABLE,
            ac2_ui::hud_style::CLASSIC_HUD,
        ]
    );
    assert_eq!(ac2_ui::tooltip_prefs::values(bank), (0.25, true));
}

/// The HUD's style (`ClassicHud`, a client-only key the DAT does not name): registered classic by
/// default, written under its own name and read back.
#[test]
fn the_hud_style_saves_under_its_client_name_and_reads_back() {
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("hud_style.ini");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    assert_eq!(
        ac2_ui::hud_style::hud_style(prefs.bank()),
        ac2_ui::HudStyle::Classic
    );
    prefs
        .bank_mut()
        .find_mut(ac2_ui::hud_style::CLASSIC_HUD)
        .expect("the key is registered")
        .value = Value::Bool(false);
    prefs
        .save()
        .join()
        .expect("the save thread")
        .expect("saved");
    let text = std::fs::read_to_string(&path).expect("the file");
    assert!(text.contains("ClassicHud"), "{text}");
    let back = ClientPrefs::load(path.clone(), store).expect("reload");
    assert_eq!(
        ac2_ui::hud_style::hud_style(back.bank()),
        ac2_ui::HudStyle::Modern
    );
    let _ = std::fs::remove_dir_all(path.parent().expect("a directory"));
}

/// `ChannelPrefs::LoadChannelPrefs` on the real DATs: the gore amount, the gore hue and the foot
/// step particles register in the FX section with the language's `StringState` top levels, their
/// names are in the DAT, and a saved value reaches the table the FX nodes' channel keys compare
/// with (`ChannelPrefs::HandleApply`).
#[test]
fn the_fx_channel_preferences_register_and_reach_the_channel_table() {
    use ac2_smartbox::fx_prefs::{
        apply, channel_prefs, FOOT_STEP_PARTICLES, FX_SECTION, GORE_AMOUNT, GORE_HUE,
    };
    let Some(store) = store() else {
        return;
    };
    let names = DatNames::new(&store);
    assert!(
        names.section_name(FX_SECTION).is_some(),
        "the FX section is named"
    );
    for key in [GORE_AMOUNT, GORE_HUE, FOOT_STEP_PARTICLES] {
        assert!(names.key_name(key).is_some(), "{key:08X} is named");
    }
    let path = temp_file("fx.ini");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let section = names.section_name(FX_SECTION).unwrap();
    let particles = names.key_name(FOOT_STEP_PARTICLES).unwrap();
    std::fs::write(&path, format!("[{section}]\r\n{particles}=0\r\n")).unwrap();
    let prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    let bank = prefs.bank();
    for key in [GORE_AMOUNT, GORE_HUE, FOOT_STEP_PARTICLES] {
        assert!(bank.find(key).is_some(), "{key:08X} is registered");
    }
    assert_eq!(
        bank.find(FOOT_STEP_PARTICLES).unwrap().value,
        Value::Bool(false)
    );
    let table = channel_prefs(bank);
    assert_eq!(
        table.values[3],
        Some(0),
        "the saved switch is the table's value"
    );
    assert!(table.values[0].is_some() && table.values[1].is_some());
    apply(bank);
    assert_eq!(ac2_assets::fx_channel_prefs(), table);
    ac2_assets::set_fx_channel_prefs(Default::default());
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
