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
            ac2_ui::ui_scale_pref::UI_SCALE,
            ac2_ui::damage_text_pref::DAMAGE_TEXT_SIZE,
            ac2_ui::hit_chance_pref::SHOW_HIT_CHANCE,
            ac2_ui::hit_chance_pref::EXAMINE_NEARBY,
        ]
    );
    assert_eq!(ac2_ui::tooltip_prefs::values(bank), (0.25, true));
}

/// The HUD's style (`ClassicHud`, a client-only key the DAT does not name): registered modern by
/// default, the classic switch written under its own name and read back.
#[test]
fn the_hud_style_saves_under_its_client_name_and_reads_back() {
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("hud_style.ini");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    assert_eq!(
        ac2_ui::hud_style::hud_style(prefs.bank()),
        ac2_ui::HudStyle::Modern
    );
    prefs
        .bank_mut()
        .find_mut(ac2_ui::hud_style::CLASSIC_HUD)
        .expect("the key is registered")
        .value = Value::Bool(true);
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
        ac2_ui::HudStyle::Classic
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

/// `maxgore` / `greenblood` (`StringStateInfo::SetTopGoreLevelOverride`,
/// `SetTopBloodColorOverride`): the overrides are the gore amount's and the gore hue's defaults
/// in place of the language's `StringState` levels.
#[test]
fn the_command_line_gore_and_blood_overrides_become_the_fx_defaults() {
    use ac2_smartbox::fx_prefs::{GORE_AMOUNT, GORE_HUE};
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("fx_overrides.ini");
    let mut overrides = ac2_db::TopLevelOverrides::default();
    overrides.set_max_gore("low").expect("low");
    overrides.set_green_blood();
    let prefs = ClientPrefs::load_with(path.clone(), Arc::clone(&store), overrides).expect("load");
    let default = |key| prefs.bank().find(key).map(|p| p.default.clone());
    assert_eq!(default(GORE_AMOUNT), Some(Value::Int(3)));
    assert_eq!(default(GORE_HUE), Some(Value::Int(1)));
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// The UI scale (`UiScale`, a client-only key): registered at 1x, a choice written under its own
/// name as a number (Fill is 0) and read back.
#[test]
fn the_ui_scale_saves_under_its_client_name_and_reads_back() {
    use ac2_ui::UiScaleSetting;
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("ui_scale.ini");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    assert_eq!(
        ac2_ui::ui_scale_pref::ui_scale(prefs.bank()),
        UiScaleSetting::Retail
    );
    for (value, expected) in [
        (250, UiScaleSetting::Percent(250)),
        (0, UiScaleSetting::Fill),
    ] {
        prefs
            .bank_mut()
            .find_mut(ac2_ui::ui_scale_pref::UI_SCALE)
            .expect("the key is registered")
            .value = Value::Int(value);
        prefs
            .save()
            .join()
            .expect("the save thread")
            .expect("saved");
        let text = std::fs::read_to_string(&path).expect("the file");
        assert!(text.contains(&format!("UiScale={value}")), "{text}");
        let back = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("reload");
        assert_eq!(ac2_ui::ui_scale_pref::ui_scale(back.bank()), expected);
    }
    let _ = std::fs::remove_dir_all(path.parent().expect("a directory"));
}

/// The damage text size (`DamageTextSize`, a client-only key): registered at 1.5x, a choice
/// written under its own name in hundredths and read back as a factor of the blob's scale.
#[test]
fn the_damage_text_size_saves_under_its_client_name_and_reads_back() {
    use ac2_ui::damage_text_pref::{damage_text_size, DAMAGE_TEXT_SIZE};
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("damage_text.ini");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    assert_eq!(damage_text_size(prefs.bank()), 1.5);
    for (value, expected) in [(100, 1.0), (300, 3.0)] {
        prefs
            .bank_mut()
            .find_mut(DAMAGE_TEXT_SIZE)
            .expect("the key is registered")
            .value = Value::Int(value);
        prefs
            .save()
            .join()
            .expect("the save thread")
            .expect("saved");
        let text = std::fs::read_to_string(&path).expect("the file");
        assert!(text.contains(&format!("DamageTextSize={value}")), "{text}");
        let back = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("reload");
        assert_eq!(damage_text_size(back.bank()), expected);
    }
    let _ = std::fs::remove_dir_all(path.parent().expect("a directory"));
}

/// `Turbine::Client::GetLanguage`'s `EnumMapper::GetEnum(0x4000002A, name)`: the language names
/// (the local DATs' suffixes) are the ids `gmCharNameCharacterValidator`'s `LanguageLetters` rows
/// and `gmClient::Init`'s Korean check use.
#[test]
fn the_language_names_map_to_the_name_validators_ids() {
    use ac2_prefs::NameTable;
    let Some(store) = store() else {
        return;
    };
    let names = DatNames::new(&store);
    let id = |name| names.enum_label(0x4000_002A, name);
    assert_eq!(id("English"), Some(ac2_db::DEFAULT_NAME_LANGUAGE));
    assert_eq!(id("Francais"), Some(0x8100_0001));
    assert_eq!(id("Deutsch"), Some(0x8100_0003));
    assert_eq!(id("Korean"), Some(0x8100_0005));
    assert_eq!(id("Japanese"), Some(0x8100_0004));
    assert_eq!(id("Chinese"), Some(0x4100_0001));
}

/// `CBaseMesh::DrawMesh`'s first-person check names its mesh types in mapper `0x4000000F`: the
/// DAT has all four (head, beard, crest, totem), each a different type.
#[test]
fn the_first_person_player_hides_four_named_mesh_types() {
    let Some(store) = store() else {
        return;
    };
    let path = temp_file("mesh_types.ini");
    let prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    let mut types = prefs.head_mesh_types().to_vec();
    assert_eq!(types.len(), 4, "{types:08X?}");
    types.sort_unstable();
    types.dedup();
    assert_eq!(types.len(), 4, "four different types");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
