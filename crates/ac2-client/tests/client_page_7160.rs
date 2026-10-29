//! The client options page (`gmClientUI`) on the real layout (`AC2_DAT_DIR`): its rows are the
//! bank's preferences under their labels; Apply applies them (the scene's draw distance, the sound
//! manager's volume, the camera's sensitivity and Y inversion); Default and Recall ask first. A
//! display change (the Rendering section, with FullScreen) is verified before it is saved or it
//! reaches the window: a timed question that runs out answers no, and only a yes applies it. The
//! window never goes fullscreen unless the player says yes. The tests start no window.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use ac2_audio::prefs::FX_VOLUME;
use ac2_audio::{Channel, NullSink};
use ac2_client::ClientPrefs;
use ac2_db::ResourceStore;
use ac2_ui::{
    build_gameplay_hud, load_layout_by_enum, tokenize, ClientOptionsUi, ClientRequest,
    DialogFactory, DialogOutcome, GameplayHud, OptionsPrefs, PageAction, UiMessage, UiTree,
    DIALOG_CONFIRMATION, DIALOG_TIMED, MSG_BUTTON_RELEASE_ON, SECTION_ORDER,
};
use common::*;
use legions_server_double::Spawn;

/// `ID_Rendering_FullScreen` (the display section's key).
const FULL_SCREEN: u32 = 0x4000_0003;
/// `ID_Graphics_LandscapeDrawDistance`.
const DRAW_DISTANCE: u32 = 0x4000_0043;
/// The toolbar and the gameplay layouts (`LAYOUT_TOOLBAR`, `LAYOUT_GAMEPLAY` in the HUD), and the
/// tab's `opened` message (`MSG_PANEL_PAGE_OPENED`).
const LAYOUT_TOOLBAR: u32 = 0x4100_0003;
const LAYOUT_GAMEPLAY: u32 = 0x4100_0009;
const MSG_OPENED: u32 = tokenize("opened");

/// A fresh directory for one test's `Preferences.ini`, under the system temporary directory.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ac2-client-page-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temporary directory");
    dir
}

fn dataset_store() -> Option<Arc<ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ResourceStore::new(dataset)))
}

fn load_prefs(store: &Arc<ResourceStore>, name: &str) -> (ClientPrefs, PathBuf) {
    let dir = temp_dir(name);
    let path = dir.join("Preferences.ini");
    let prefs = ClientPrefs::load(path.clone(), Arc::clone(store)).expect("load");
    (prefs, path)
}

/// The HUD as the client builds it, and the client page made over it from the bank.
fn page_over_hud(
    store: &Arc<ResourceStore>,
    prefs: &ClientPrefs,
) -> (GameplayHud, ClientOptionsUi) {
    let mut hud = build_gameplay_hud(store, 1024, 768).expect("the HUD");
    let toolbar = load_layout_by_enum(store, LAYOUT_TOOLBAR)
        .expect("the toolbar layout")
        .expect("present");
    let text = hud.tree.text_data().expect("the text data");
    let tables = (
        hud.ui_table.expect("the UI string table"),
        hud.pref_table.expect("the preference label table"),
    );
    let page = ClientOptionsUi::create(
        &mut hud.tree,
        &toolbar,
        &*text,
        tables,
        prefs.bank(),
        prefs.names(),
        ac2_ui::pref_label,
    )
    .expect("the client page");
    (hud, page)
}

/// A dialog factory over the gameplay layout, as the HUD has it.
fn dialogs_over(store: &Arc<ResourceStore>) -> DialogFactory {
    DialogFactory::new(load_layout_by_enum(store, LAYOUT_GAMEPLAY).expect("layout"))
}

/// A UI message the tree sends for a widget (`element` is the widget's element, if any).
fn message(element_id: &str, element: Option<usize>, message: u32) -> UiMessage {
    UiMessage {
        element_id: tokenize(element_id),
        element,
        message,
        data: 0,
        item: None,
    }
}

/// The player flips a checkbox of the page: its value changes to the other one.
fn open_tab(
    page: &mut ClientOptionsUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    prefs: &mut ClientPrefs,
) {
    page.message(
        tree,
        dialogs,
        prefs.bank_mut(),
        &message("ClientTab", None, MSG_OPENED),
    );
}

fn press(
    page: &mut ClientOptionsUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    prefs: &mut ClientPrefs,
    button: &str,
) {
    page.message(
        tree,
        dialogs,
        prefs.bank_mut(),
        &message(button, None, MSG_BUTTON_RELEASE_ON),
    );
}

#[test]
fn the_page_lists_each_preference_under_its_label_in_retail_order() {
    let Some(store) = dataset_store() else {
        return;
    };
    let (prefs, _) = load_prefs(&store, "rows");
    let (hud, page) = page_over_hud(&store, &prefs);
    assert_eq!(page.sections(), SECTION_ORDER.to_vec());
    assert!(!page.changed(prefs.bank()), "nothing changed yet");
    for name in [
        "ID_Sound_FXVolume",
        "ID_Graphics_LandscapeDrawDistance",
        "ID_Rendering_FullScreen",
        "ID_Camera_Distance",
        "ID_Input_Invert_Y",
    ] {
        let label = page
            .row_label(&hud.tree, name)
            .unwrap_or_else(|| panic!("{name} has a row"));
        assert!(!label.is_empty(), "{name} is labelled");
    }
    assert_eq!(page.value(prefs.bank(), "ID_Sound_FXVolume"), Some(1.0));
}

#[test]
fn apply_reaches_the_scene_the_sound_and_the_camera() {
    let Some(store) = dataset_store() else {
        return;
    };
    let (prefs, _) = load_prefs(&store, "apply");
    let (mut hud, mut page) = page_over_hud(&store, &prefs);
    let (mut client, _server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    client.attach_prefs(prefs);
    client.attach_audio(ac2_client::sound_manager(
        Arc::clone(&store),
        Box::new(NullSink),
    ));
    let mut dialogs = dialogs_over(&store);
    open_tab(
        &mut page,
        &mut hud.tree,
        &mut dialogs,
        client.prefs_mut().expect("prefs"),
    );

    // The player's changes on the page.
    let bank = client.prefs_mut().expect("prefs").bank_mut();
    assert!(page.set_value(&mut hud.tree, bank, "ID_Sound_FXVolume", 0.25));
    assert!(page.set_value(&mut hud.tree, bank, "ID_Input_Mouse_Look_Sensitivity", 1.5));
    assert!(page.set_value(&mut hud.tree, bank, "ID_Input_Invert_Y", 1.0));
    assert!(page.changed(client.prefs().expect("prefs").bank()));
    // The draw distance is a choice of the graphics section; the page's drop-down sets it by bank value.
    client
        .prefs_mut()
        .expect("prefs")
        .bank_mut()
        .find_mut(DRAW_DISTANCE)
        .expect("registered")
        .set_choice(4);

    // Apply: the request the press makes, then the client's answer to it.
    press(
        &mut page,
        &mut hud.tree,
        &mut dialogs,
        client.prefs_mut().expect("prefs"),
        "ApplyButton",
    );
    assert_eq!(page.take_requests(), vec![ClientRequest::Apply]);
    let applied = client.prefs_mut().expect("prefs").options_apply();
    assert!(
        !applied.verify_render,
        "graphics and sound need no verification"
    );
    assert!(applied.keys.contains(&DRAW_DISTANCE));
    applied.wait_saved().expect("the file is written");
    let scene = client.apply_live_preferences().expect("the scene options");
    assert_eq!(
        scene.far_radius,
        ac2_render::terrain::far_radius_for_draw_distance(4),
        "the landscape draw distance reaches the scene"
    );
    assert_eq!(
        client
            .audio_mut()
            .expect("the sound manager")
            .channel_volume(Channel::Main),
        0.25,
        "the FX volume reaches the sound manager"
    );
    assert_eq!(client.camera().mouse_sensitivity, 1.5);
    assert!(client.camera().invert_y);
}

#[test]
fn a_display_change_waits_for_its_verification_and_only_a_yes_applies_it() {
    let Some(store) = dataset_store() else {
        return;
    };
    let (prefs, path) = load_prefs(&store, "verify");
    let (mut hud, mut page) = page_over_hud(&store, &prefs);
    let (mut client, _server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    client.attach_prefs(prefs);
    let mut dialogs = dialogs_over(&store);
    open_tab(
        &mut page,
        &mut hud.tree,
        &mut dialogs,
        client.prefs_mut().expect("prefs"),
    );

    // Full screen is asked for; the window does not change.
    let bank = client.prefs_mut().expect("prefs").bank_mut();
    // The player flips the checkbox, whatever its default is.
    let before = page
        .value(bank, "ID_Rendering_FullScreen")
        .expect("the row");
    assert!(page.set_value(&mut hud.tree, bank, "ID_Rendering_FullScreen", 1.0 - before));
    press(
        &mut page,
        &mut hud.tree,
        &mut dialogs,
        client.prefs_mut().expect("prefs"),
        "ApplyButton",
    );
    assert_eq!(page.take_requests(), vec![ClientRequest::Apply]);
    let applied = client.prefs_mut().expect("prefs").options_apply();
    assert!(
        applied.verify_render,
        "a display change asks for its verification"
    );
    assert!(
        !applied.keys.contains(&FULL_SCREEN),
        "the display waits for the answer"
    );
    assert!(applied.save.is_none(), "nothing is saved while it is asked");
    assert!(!path.exists());

    // The question is a timed dialog: it counts down and, run out, answers no.
    let roots = hud.tree.roots().len();
    assert!(page.after_apply(
        &mut hud.tree,
        &mut dialogs,
        client.prefs().expect("prefs").bank(),
        true
    ));
    assert_eq!(
        hud.tree.roots().len(),
        roots + 1,
        "the timed question is drawn"
    );
    assert_eq!(dialogs.shown().map(|(_, kind)| kind), Some(DIALOG_TIMED));
    dialogs.tick(&mut hud.tree, 0.0);
    dialogs.tick(&mut hud.tree, 5.0);
    assert!(dialogs.take_outcomes().is_empty(), "still counting");
    dialogs.tick(&mut hud.tree, 16.0);
    let outcomes = dialogs.take_outcomes();
    assert!(matches!(
        outcomes.as_slice(),
        [DialogOutcome::Confirmation { yes: false, .. }]
    ));
    page.dialog_outcomes(&mut hud.tree, &mut dialogs, &outcomes);
    let requests = page.take_requests();
    assert_eq!(
        requests,
        vec![ClientRequest::Answer(vec![PageAction::RecallDisplay])]
    );
    let ClientRequest::Answer(actions) = &requests[0] else {
        unreachable!()
    };
    let answer = client
        .prefs_mut()
        .expect("prefs")
        .options_page_answer(actions);
    assert!(!answer.verify_render);
    answer.wait_saved().expect("saved");
    assert!(
        !client
            .prefs()
            .expect("prefs")
            .bank()
            .find(FULL_SCREEN)
            .expect("registered")
            .changed(),
        "the display is back at its applied value"
    );

    // Asked again and answered yes: the display section applies (and the window follows).
    let bank = client.prefs_mut().expect("prefs").bank_mut();
    // The player flips the checkbox, whatever its default is.
    let before = page
        .value(bank, "ID_Rendering_FullScreen")
        .expect("the row");
    assert!(page.set_value(&mut hud.tree, bank, "ID_Rendering_FullScreen", 1.0 - before));
    press(
        &mut page,
        &mut hud.tree,
        &mut dialogs,
        client.prefs_mut().expect("prefs"),
        "ApplyButton",
    );
    assert_eq!(page.take_requests(), vec![ClientRequest::Apply]);
    let asked = client.prefs_mut().expect("prefs").options_apply();
    assert!(asked.verify_render);
    asked.wait_saved().expect("saved");
    assert!(page.after_apply(
        &mut hud.tree,
        &mut dialogs,
        client.prefs().expect("prefs").bank(),
        true
    ));
    let context = page.page().open_contexts()[0];
    page.dialog_outcomes(
        &mut hud.tree,
        &mut dialogs,
        &[DialogOutcome::Confirmation { context, yes: true }],
    );
    let requests = page.take_requests();
    let ClientRequest::Answer(actions) = &requests[0] else {
        panic!("the yes answers the question: {requests:?}");
    };
    assert_eq!(actions, &vec![PageAction::SynchronizeDisplay]);
    let answer = client
        .prefs_mut()
        .expect("prefs")
        .options_page_answer(actions);
    assert!(
        answer.keys.contains(&FULL_SCREEN),
        "the verified display change is applied"
    );
    answer.wait_saved().expect("saved");
}

#[test]
fn default_asks_first_and_a_yes_resets_the_page() {
    let Some(store) = dataset_store() else {
        return;
    };
    let (prefs, _) = load_prefs(&store, "default");
    let (mut hud, mut page) = page_over_hud(&store, &prefs);
    let (mut client, _server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    client.attach_prefs(prefs);
    let mut dialogs = dialogs_over(&store);
    open_tab(
        &mut page,
        &mut hud.tree,
        &mut dialogs,
        client.prefs_mut().expect("prefs"),
    );
    let bank = client.prefs_mut().expect("prefs").bank_mut();
    assert!(page.set_value(&mut hud.tree, bank, "ID_Sound_FXVolume", 0.1));
    press(
        &mut page,
        &mut hud.tree,
        &mut dialogs,
        client.prefs_mut().expect("prefs"),
        "ApplyButton",
    );
    assert_eq!(page.take_requests(), vec![ClientRequest::Apply]);
    let applied = client.prefs_mut().expect("prefs").options_apply();
    applied.wait_saved().expect("saved");

    // Default asks; nothing changes until the player says yes.
    press(
        &mut page,
        &mut hud.tree,
        &mut dialogs,
        client.prefs_mut().expect("prefs"),
        "DefaultButton",
    );
    assert!(page.take_requests().is_empty());
    assert_eq!(
        dialogs.shown().map(|(_, kind)| kind),
        Some(DIALOG_CONFIRMATION)
    );
    assert_eq!(
        client
            .prefs()
            .expect("prefs")
            .bank()
            .find(FX_VOLUME)
            .map(|p| p.value.clone()),
        Some(ac2_prefs::Value::Float(0.1)),
        "the value holds until the answer"
    );
    let context = page.page().open_contexts()[2];
    page.dialog_outcomes(
        &mut hud.tree,
        &mut dialogs,
        &[DialogOutcome::Confirmation { context, yes: true }],
    );
    let requests = page.take_requests();
    let ClientRequest::Answer(actions) = &requests[0] else {
        panic!("the yes answers the question: {requests:?}");
    };
    assert_eq!(actions, &vec![PageAction::Default]);
    let answer = client
        .prefs_mut()
        .expect("prefs")
        .options_page_answer(actions);
    answer.wait_saved().expect("saved");
    assert_eq!(
        client
            .prefs()
            .expect("prefs")
            .bank()
            .find(FX_VOLUME)
            .map(|p| p.value.clone()),
        Some(ac2_prefs::Value::Float(1.0)),
        "the default came back"
    );
}

/// `gmClientUI::VerifyRenderOptions` asks `ID_Option_RenderText` (the `Tokenize` string of the
/// admin tree's initializer for `DAT_00a4f870`); the DVD's UI table does not carry it, while the
/// other two questions are there.
#[test]
fn the_render_verification_question_is_the_one_the_ui_table_lacks() {
    let Some(store) = dataset_store() else {
        return;
    };
    let hud = build_gameplay_hud(&store, 1024, 768).expect("the HUD");
    let text = hud.tree.text_data().expect("the text data");
    let table = hud.ui_table.expect("the UI string table");
    assert!(text
        .string(table, tokenize("ID_Option_RecallText"))
        .is_some());
    assert!(text
        .string(table, tokenize("ID_Option_RenderText"))
        .is_none());
}

/// `ID_Item_TT_WaitingForServer` (`FIXMESetupItemElement`, `ItemDisplay::Update`) is not in the
/// DVD's UI table either, so `StringInfo::IsReady` fails and a waiting item's tooltip is the
/// element's own `tooltip_entry` (`UIElementManager::StartTooltip`).
#[test]
fn the_waiting_for_server_tooltip_is_not_in_the_ui_table() {
    let Some(store) = dataset_store() else {
        return;
    };
    let hud = build_gameplay_hud(&store, 1024, 768).expect("the HUD");
    let text = hud.tree.text_data().expect("the text data");
    let table = hud.ui_table.expect("the UI string table");
    assert!(text
        .string(table, tokenize("ID_Item_TT_WaitingForServer"))
        .is_none());
}
