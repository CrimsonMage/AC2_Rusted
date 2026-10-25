//! The options page on the real DATs (`AC2_DAT_DIR`): the page's OK writes `Preferences.ini` in
//! a temporary directory and changes the live sound setting, Cancel changes nothing, a render
//! change asks for verification and does not save, and a quality level reaches the landscape's
//! draw distance. No window is opened and no audio is played: the sound manager writes to a null
//! sink.

use std::path::PathBuf;
use std::sync::Arc;

use ac2_audio::prefs::{DISABLE_SOUND, FX_VOLUME};
use ac2_audio::{Channel, NullSink, SoundManager};
use ac2_client::{ClientPrefs, Close, OptionsWindow};

/// No sounds: the page never loads one.
struct NoSounds;

impl ac2_audio::SoundSource for NoSounds {
    fn load(&self, _did: u32) -> Option<ac2_audio::LoadedSound> {
        None
    }
}

fn store() -> Option<Arc<ac2_db::ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ac2_db::ResourceStore::new(dataset)))
}

/// A fresh directory for one test's `Preferences.ini`, under the system temporary directory.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ac2-options-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temporary directory");
    dir
}

fn live_manager() -> SoundManager {
    SoundManager::new(Box::new(NullSink), Arc::new(NoSounds))
}

#[test]
fn ok_writes_the_file_and_changes_the_live_sound_setting() {
    let Some(store) = store() else {
        return;
    };
    let dir = temp_dir("ok");
    let path = dir.join("Preferences.ini");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    let mut live = live_manager();
    assert_eq!(
        live.channel_volume(Channel::Main),
        1.0,
        "retail starts full"
    );

    prefs
        .bank_mut()
        .find_mut(FX_VOLUME)
        .expect("FX volume is registered")
        .set_float(0.25);
    let outcome = prefs.options_apply();
    assert!(outcome.applied_sound(), "the sound section applies");
    assert!(!outcome.verify_render);
    outcome.wait_saved().expect("the file is written");
    ac2_audio::prefs::apply(&mut live, prefs.bank());

    let text = std::fs::read_to_string(&path).expect("the file exists");
    assert!(
        text.contains("[Sound]\r\nSoundVolume=0.250000\r\n"),
        "the sound section holds the new volume: {text:?}"
    );
    assert_eq!(live.channel_volume(Channel::Main), 0.25);
}

#[test]
fn cancel_changes_nothing() {
    let Some(store) = store() else {
        return;
    };
    let dir = temp_dir("cancel");
    let path = dir.join("Preferences.ini");
    let original = b"[Sound]\r\nSoundVolume=0.750000\r\n";
    std::fs::write(&path, original).expect("seed the file");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    let live = live_manager();
    assert_eq!(
        prefs.bank().find(FX_VOLUME).expect("registered").value,
        ac2_prefs::Value::Float(0.75),
        "the saved volume is loaded"
    );

    prefs
        .bank_mut()
        .find_mut(FX_VOLUME)
        .expect("registered")
        .set_float(0.1);
    let mut window = OptionsWindow::default();
    assert_eq!(window.request_close(&prefs), Close::Confirm);
    assert!(window.confirming());
    assert!(
        window.answer(&mut prefs, false).is_none(),
        "no applies nothing"
    );
    assert!(!window.confirming());

    assert!(
        !prefs.options_changed(),
        "the page is back at its applied values"
    );
    assert_eq!(
        prefs.bank().find(FX_VOLUME).expect("registered").value,
        ac2_prefs::Value::Float(0.75)
    );
    assert_eq!(
        std::fs::read(&path).expect("the file is still there"),
        original,
        "the file is byte for byte as it was"
    );
    assert_eq!(
        live.channel_volume(Channel::Main),
        1.0,
        "the live setting is untouched"
    );
    assert!(!prefs
        .bank()
        .find(DISABLE_SOUND)
        .expect("registered")
        .changed());
}

#[test]
fn a_render_change_asks_for_verification_and_does_not_save() {
    let Some(store) = store() else {
        return;
    };
    let dir = temp_dir("render");
    let path = dir.join("Preferences.ini");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    prefs
        .bank_mut()
        .find_mut(0x4000_0043)
        .expect("the landscape draw distance is registered")
        .set_choice(4);
    let outcome = prefs.options_apply();
    assert!(outcome.verify_render);
    assert!(
        outcome.save.is_none(),
        "nothing is written while verification is asked"
    );
    assert!(!path.exists());
}

#[test]
fn a_quality_level_reaches_the_landscape_draw_distance() {
    let Some(store) = store() else {
        return;
    };
    let dir = temp_dir("quality");
    let path = dir.join("Preferences.ini");
    let mut prefs = ClientPrefs::load(path, Arc::clone(&store)).expect("load");
    assert!(prefs.set_overall_graphics_quality(5));
    assert_eq!(
        ac2_render::prefs::landscape_draw_distance(prefs.bank()),
        Some(4)
    );
    assert_eq!(
        ac2_render::terrain::far_radius_for_draw_distance(4),
        8,
        "the draw distance 4 is the widest far radius"
    );
    assert!(!prefs.set_overall_graphics_quality(0));
}
