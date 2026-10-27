//! The options page on the real DATs (`AC2_DAT_DIR`): the page's OK writes `Preferences.ini` in
//! a temporary directory and changes the live sound setting, Cancel changes nothing, a render
//! change asks for verification and does not save, and a quality level reaches the landscape's
//! draw distance. No window is opened and no audio is played: the sound manager writes to a null
//! sink.

use std::path::PathBuf;
use std::sync::Arc;

use ac2_audio::prefs::{DISABLE_SOUND, FX_VOLUME};

/// `ID_Rendering_FullScreen`, the display section's key.
const FULL_SCREEN: u32 = 0x4000_0003;
use ac2_audio::{Channel, NullSink, SoundManager};
use ac2_client::{ClientPrefs, Close, OptionsWindow};
use ac2_ui::{ClientPage, ClientPageText, DialogFactory, PageAction, UiTree};

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
    // The display section (Rendering) is the one whose change asks for the verification.
    // The player flips FullScreen (its default is not assumed).
    let on = prefs
        .bank()
        .find(FULL_SCREEN)
        .expect("registered")
        .value
        .clone();
    prefs
        .bank_mut()
        .find_mut(FULL_SCREEN)
        .expect("FullScreen is registered")
        .set_bool(on != ac2_prefs::Value::Bool(true));
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

#[test]
fn a_render_verification_answered_no_recalls_the_display_and_saves_once() {
    let Some(store) = store() else {
        return;
    };
    let dir = temp_dir("verify-no");
    let path = dir.join("Preferences.ini");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    prefs
        .bank_mut()
        .find_mut(0x4000_0043)
        .expect("the landscape draw distance is registered")
        .set_choice(4);
    // The player flips FullScreen (its default is not assumed).
    let on = prefs
        .bank()
        .find(FULL_SCREEN)
        .expect("registered")
        .value
        .clone();
    prefs
        .bank_mut()
        .find_mut(FULL_SCREEN)
        .expect("FullScreen is registered")
        .set_bool(on != ac2_prefs::Value::Bool(true));
    let outcome = prefs.options_apply();
    assert!(outcome.verify_render);
    // The graphics section applied at once; the display waits for the answer.
    assert!(outcome.keys.contains(&0x4000_0043));
    assert!(!outcome.keys.contains(&FULL_SCREEN));
    // The page asks the question (a timed dialog) and the player answers no.
    let mut tree = UiTree::new(1024, 768);
    let mut dialogs = DialogFactory::new(None);
    let mut page = ClientPage::new(ClientPageText::default());
    assert!(page.after_apply(&mut tree, &mut dialogs, outcome.verify_render));
    let render = page.open_contexts()[0];
    let actions = page.recv_close_confirmation(&mut tree, &mut dialogs, render, false);
    assert_eq!(actions, vec![PageAction::RecallDisplay]);

    let answer = prefs.options_page_answer(&actions);
    assert_eq!(answer.saves.len(), 1, "one save for the answer");
    answer.wait_saved().expect("the file is written");
    assert!(path.exists());
    assert_eq!(
        ac2_render::prefs::landscape_draw_distance(prefs.bank()),
        Some(4),
        "the graphics change stays applied: only the display section is recalled"
    );
}

#[test]
fn a_default_answered_yes_resets_every_section_and_saves() {
    let Some(store) = store() else {
        return;
    };
    let dir = temp_dir("default-yes");
    let path = dir.join("Preferences.ini");
    let mut prefs = ClientPrefs::load(path.clone(), Arc::clone(&store)).expect("load");
    prefs
        .bank_mut()
        .find_mut(FX_VOLUME)
        .expect("FX volume is registered")
        .set_float(0.1);
    prefs.options_apply().wait_saved().expect("saved");

    let mut tree = UiTree::new(1024, 768);
    let mut dialogs = DialogFactory::new(None);
    let mut page = ClientPage::new(ClientPageText::default());
    page.confirm_default(&mut tree, &mut dialogs);
    let default = page.open_contexts()[2];
    let actions = page.recv_close_confirmation(&mut tree, &mut dialogs, default, true);
    assert_eq!(actions, vec![PageAction::Default]);

    let answer = prefs.options_page_answer(&actions);
    assert!(!answer.verify_render);
    answer.wait_saved().expect("saved");
    let volume = prefs.bank().find(FX_VOLUME).expect("registered");
    assert!(!volume.changed());
    assert_ne!(
        volume.value,
        ac2_prefs::Value::Float(0.1),
        "the default came back"
    );
}

#[test]
fn the_scene_takes_the_draw_and_frill_distances_the_page_sets() {
    let Some(store) = store() else {
        return;
    };
    let dir = temp_dir("scene");
    let mut prefs = ClientPrefs::load(dir.join("Preferences.ini"), store).expect("load");
    assert_eq!(
        prefs.scene_options(),
        ac2_render::prefs::SceneOptions::default(),
        "the scene's defaults until the page changes them"
    );
    prefs
        .bank_mut()
        .find_mut(0x4000_0043)
        .expect("LandscapeDrawDistance")
        .set_choice(0);
    prefs
        .bank_mut()
        .find_mut(0x4000_0044)
        .expect("LandscapeFrillDistance")
        .set_choice(1);
    // The graphics section applies at once: only the Rendering section asks for verification.
    let outcome = prefs.options_apply();
    assert!(
        !outcome.verify_render,
        "a graphics change needs no verification"
    );
    assert!(outcome.keys.contains(&0x4000_0044));
    outcome.wait_saved().expect("saved");
    let options = prefs.scene_options();
    assert_eq!(options.far_radius, 0);
    assert!(options.frill.short_distance);
}
