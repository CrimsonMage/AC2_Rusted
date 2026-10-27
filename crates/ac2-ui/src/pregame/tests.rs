//! The pregame flow and arms without data, and character selection on the real layouts
//! (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::{CharacterIdentity, CharacterSet};

use super::*;
use crate::token::{tokenize, BUTTON_GHOSTED};
use crate::{DIALOG_CONFIRMATION, DIALOG_WAIT, UI_POINTER_X, UI_POINTER_Y};

pub(super) fn headings(arms: &CharacterArms) -> Vec<f32> {
    arms.arms()
        .iter()
        .map(|a| (a.frame().heading_degrees() * 100.0).round() / 100.0)
        .collect()
}

#[test]
pub(super) fn the_arms_share_the_circle_and_turn_the_selection_to_the_front() {
    let mut arms = CharacterArms::default();
    arms.init(5);
    // 45 back for the second, then (360 - 90) / 3 = 90 more for each.
    assert_eq!(headings(&arms), vec![0.0, 315.0, 225.0, 135.0, 45.0]);
    // One slot on: the shorter way is left; the two arms nearest turn by 45, the rest by 90.
    arms.set_target(1);
    arms.global_loop(0.0);
    assert_eq!(arms.selected(), (1, 1));
    assert!(arms.arms()[0].slerping());
    arms.global_loop(0.75);
    assert!(arms.arms()[1].slerping(), "half way");
    arms.global_loop(1.6);
    assert_eq!(headings(&arms), vec![45.0, 0.0, 315.0, 225.0, 135.0]);
    // A direct pick turns all the way at once.
    arms.set_target(3);
    arms.set_move_immediate(true);
    arms.global_loop(2.0);
    assert_eq!(arms.selected(), (3, 3));
    assert_eq!(headings(&arms)[3], 0.0);
    assert!(!arms.arms()[0].slerping());
}

#[test]
pub(super) fn login_errors_pick_their_mode_and_string() {
    assert_eq!(
        character_error_mode(char_error::ACCOUNT_IN_USE),
        Some((MODE_DISCONNECTED, "ID_CHAR_ERROR_ACCOUNT_IN_USE"))
    );
    assert_eq!(
        character_error_mode(char_error::ENTER_GAME_CHARACTER_LOCKED),
        Some((MODE_CHARACTER_SELECTION, "ID_CHAR_ERROR_ENTER_GAME_LOCKED"))
    );
    assert_eq!(character_error_mode(1), None);
    let mut flow = UiFlow::new();
    flow.character_error(&UiStrings::default(), char_error::DELETE, Some("gone"));
    assert_eq!(
        flow.use_new_mode(),
        Some((MODE_CHARACTER_SELECTION, "gone".into()))
    );
    assert_eq!(flow.mode(), MODE_CHARACTER_SELECTION);
    flow.queue_mode(MODE_DISCONNECTED, "x");
    flow.server_died(&UiStrings::default());
    assert_eq!(flow.use_new_mode(), Some((MODE_DISCONNECTED, "x".into())));
}

pub(super) fn character(id: u64, name: &str, greyed: u32) -> CharacterIdentity {
    CharacterIdentity {
        id,
        name: name.into(),
        seconds_greyed_out: greyed,
        visual: Default::default(),
    }
}

pub(super) fn set() -> CharacterSet {
    CharacterSet {
        characters: vec![
            character(0x5000_0001, "Alpha", 0),
            character(0x5000_0002, "Beta", 3600),
        ],
        deleted: Vec::new(),
        status: 0,
        allowed: 5,
        account: "local".into(),
        turbine_chat: true,
        legions: Some([1, 1]),
    }
}

pub(super) fn click_named(ui: &mut PregameUi, name: &str) {
    let element = find(ui, name);
    click(ui, element);
}

pub(super) fn click(ui: &mut PregameUi, element: usize) {
    let b = ui.tree().screen_box(element);
    let (x, y) = ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2);
    ui.input(&[
        (UI_POINTER_X, x as f32, true),
        (UI_POINTER_Y, y as f32, true),
        (0x4000_0004, 1.0, true),
        (0x4000_0004, 1.0, false),
    ]);
}

pub(super) fn try_find(ui: &PregameUi, name: &str) -> Option<usize> {
    let id = tokenize(name);
    ui.tree().roots().iter().find_map(|&r| {
        if ui.tree().get(r).unwrap().id() == id {
            Some(r)
        } else {
            ui.tree().child_recursive(r, id)
        }
    })
}

pub(super) fn find(ui: &PregameUi, name: &str) -> usize {
    try_find(ui, name).unwrap_or_else(|| panic!("{name}"))
}

pub(super) fn ghosted(ui: &PregameUi, name: &str) -> bool {
    ui.tree()
        .get(find(ui, name))
        .unwrap()
        .attribute(BUTTON_GHOSTED)
        == Some(&crate::layout::AttributeValue::Bool(true))
}

pub(super) fn real_ui() -> Option<PregameUi> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let assets = load_pregame(&store).unwrap();
    for layout in PREGAME_LAYOUTS {
        assert!(assets.layout(layout).is_some(), "{layout:08x}");
    }
    assert!(!assets.images.is_empty());
    Some(PregameUi::new(assets, 1024, 768))
}

/// The intro passed with a key press, then the data patch screen takes `set` into character
/// selection.
pub(super) fn to_selection(ui: &mut PregameUi, set: CharacterSet) {
    ui.use_time(0.0);
    assert_eq!(ui.mode(), MODE_INTRO);
    ui.input(&[(KEY, 1.0, true), (KEY, 1.0, false)]);
    ui.use_time(0.0);
    assert_eq!(ui.mode(), MODE_DATA_PATCH);
    ui.character_set(set);
    ui.use_time(0.0);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
}

/// A keyboard action (any action that is not the pointer's).
pub(super) const KEY: u32 = 0x4000_0100;

/// The requests the screens make of the client, without the screens' music (see
/// [`the_screens_ask_for_their_music_as_they_open`]).
pub(super) fn actions(ui: &mut PregameUi) -> Vec<PregameRequest> {
    ui.take_requests()
        .into_iter()
        .filter(|request| !matches!(request, PregameRequest::Music(_)))
        .collect()
}

/// The intro's and the character selection's music (`gmIntroUI` and
/// `gmCharacterSelectionUI` constructors) are requested as their screens open, in that order.
#[test]
pub(super) fn the_screens_ask_for_their_music_as_they_open() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.use_time(0.0);
    assert_eq!(ui.mode(), MODE_INTRO);
    assert_eq!(ui.take_requests(), vec![PregameRequest::Music(0x2C00_039B)]);
    ui.input(&[(KEY, 1.0, true), (KEY, 1.0, false)]);
    ui.use_time(0.0);
    ui.character_set(set());
    ui.use_time(0.0);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
    assert_eq!(ui.take_requests(), vec![PregameRequest::Music(0x2C00_039A)]);
}

#[test]
pub(super) fn character_selection_lists_the_set_and_enters_with_the_selection() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    assert_eq!(ui.mode(), 0, "connecting");
    to_selection(&mut ui, set());
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
    let selection = ui.character_selection().expect("character selection");
    // Five slots (the allowed count), both characters standing on theirs.
    assert_eq!(selection.arms().arms().len(), 5);
    assert_eq!(selection.occupants().len(), 2);
    let list = find(&ui, "CharacterListBox");
    let items = ui.tree().list_items(list).to_vec();
    assert_eq!(items.len(), 2);
    let texts: Vec<_> = items
        .iter()
        .map(|&i| ui.tree().get(i).unwrap().text().unwrap())
        .collect();
    assert_eq!(texts, ["Alpha", "Beta"]);
    assert_eq!(
        ui.tree().get(items[1]).unwrap().id(),
        tokenize("deleted_list_box_item")
    );
    // The first is selected: enter and delete open (the field in its delete state), create
    // open (free slots).
    assert_eq!(ui.tree().list_selected_index(list), Some(0));
    assert!(!ghosted(&ui, "EnterButton") && !ghosted(&ui, "DeleteButton"));
    assert!(try_find(&ui, "RestoreButton").is_none() && !ghosted(&ui, "CreateButton"));
    // Selecting the one pending deletion shows restore instead and closes enter.
    click(&mut ui, items[1]);
    assert_eq!(ui.tree().list_selected_index(list), Some(1));
    assert!(ghosted(&ui, "EnterButton") && !ghosted(&ui, "RestoreButton"));
    assert!(try_find(&ui, "DeleteButton").is_none());
    // Back to the first; enter: the request, "please wait", gameplay next.
    click(&mut ui, items[0]);
    let enter = find(&ui, "EnterButton");
    click(&mut ui, enter);
    assert_eq!(
        actions(&mut ui),
        vec![PregameRequest::EnterWorld(0x5000_0001)]
    );
    assert_eq!(
        ui.dialogs().shown().map(|(_, kind)| kind),
        Some(DIALOG_WAIT)
    );
    assert_eq!(ui.flow().next_mode(), Some(MODE_GAMEPLAY));
    ui.use_time(0.1);
    assert_eq!(ui.mode(), MODE_GAMEPLAY);
    assert!(
        ui.dialogs().shown().is_none(),
        "the framework closed its dialog"
    );
    assert!(ui.tree().roots().is_empty());
    // The server refused: back to selection with the error shown.
    ui.character_error(char_error::ENTER_GAME_CHARACTER_IN_WORLD, None);
    ui.use_time(0.2);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
    let error = ui
        .tree()
        .get(find(&ui, "ErrorMsg"))
        .unwrap()
        .text()
        .unwrap();
    eprintln!("error text: {error:?}");
    assert!(!error.is_empty());
}

#[test]
pub(super) fn deleting_asks_first_and_create_opens_avatar_selection() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    to_selection(&mut ui, set());
    let delete = find(&ui, "DeleteButton");
    click(&mut ui, delete);
    let (context, kind) = ui.dialogs().shown().expect("confirmation");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert!(context > 0 && ghosted(&ui, "CreateButton"));
    // No: nothing sent, the buttons back.
    click_named(&mut ui, "NoButton");
    assert!(actions(&mut ui).is_empty());
    assert!(ui.dialogs().shown().is_none() && !ghosted(&ui, "CreateButton"));
    // Yes: the delete request.
    click_named(&mut ui, "DeleteButton");
    click_named(&mut ui, "YesButton");
    assert_eq!(
        actions(&mut ui),
        vec![PregameRequest::DeleteCharacter(0x5000_0001)]
    );
    // Create: please wait, then avatar selection.
    ui.character_set(set());
    click_named(&mut ui, "CreateButton");
    assert_eq!(ui.flow().next_mode(), Some(MODE_AVATAR_SELECTION));
}

/// The composed screen, surfaces over black in compose order.
pub(super) fn snapshot(ui: &mut PregameUi, name: &str) {
    let images = ui.images().clone();
    let (w, h) = (1024_usize, 768_usize);
    let mut rgba = vec![0_u8; w * h * 4];
    for px in rgba.chunks_mut(4) {
        px[3] = 255;
    }
    for surface in ui.tree_mut().compose(&images) {
        for sy in 0..surface.height as usize {
            for sx in 0..surface.width as usize {
                let (x, y) = (surface.x + sx as i32, surface.y + sy as i32);
                if x < 0 || y < 0 || x as usize >= w || y as usize >= h {
                    continue;
                }
                let s = &surface.rgba[(sy * surface.width as usize + sx) * 4..][..4];
                let d = &mut rgba[(y as usize * w + x as usize) * 4..][..4];
                let a = u32::from(s[3]);
                for c in 0..3 {
                    d[c] = ((u32::from(s[c]) * a + u32::from(d[c]) * (255 - a)) / 255) as u8;
                }
            }
        }
    }
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).unwrap();
    let file = std::fs::File::create(dir.join(name)).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&rgba)
        .unwrap();
}

#[test]
pub(super) fn character_selection_draws() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    to_selection(&mut ui, set());
    snapshot(&mut ui, "pregame_character_selection.png");
    click_named(&mut ui, "DeleteButton");
    snapshot(&mut ui, "pregame_delete_confirmation.png");
}

/// A `CharacterGenSystem` with one Empyrean (needs the expansion) male and one slider.
pub(super) struct Stub;

impl CharGenSystem for Stub {
    fn race_sex_data(&mut self) -> Option<Vec<(u32, Vec<u32>)>> {
        Some(vec![(0x10, vec![0x1000])])
    }
    fn race_sex_info(&mut self) -> Vec<(u32, ac2_net::protocol::GmRaceSexInfo)> {
        Vec::new()
    }
    fn set_defaults(&mut self, cgr: &mut ac2_net::protocol::CharGenResult) -> bool {
        set_phys_type_value(cgr, phys_type::HEIGHT, 1.0);
        true
    }
    fn randomize(&mut self, _: u32, cgr: &mut ac2_net::protocol::CharGenResult) -> bool {
        set_phys_type_value(cgr, phys_type::HEIGHT, 1.1);
        true
    }
    fn phys_obj(&mut self, _: u32, _: u32) -> u32 {
        0x0200_0001
    }
    fn valid_phys_type_values(&mut self, _: u32, _: u32, t: u32) -> Option<Vec<f32>> {
        (t == phys_type::HEIGHT).then(|| vec![0.9, 1.0, 1.1])
    }
    fn phys_type_label(&mut self, _: u32, _: u32, phys_type: u32) -> Option<ac2_db::RenderInfo> {
        let text: Vec<u16> = "Tall".encode_utf16().collect();
        (phys_type == phys_type::HEIGHT).then(|| ac2_db::RenderInfo {
            string_id: 0,
            table_id: 0,
            literal: Some(text),
            variables: Vec::new(),
        })
    }
    fn avatar_entity_did(&mut self, _: bool) -> u32 {
        0x4700_0001
    }
    fn applied_appearance(
        &mut self,
        _: &ac2_net::protocol::CharGenResult,
    ) -> Option<ac2_weenie::Appearance> {
        None
    }

    fn enum_string(&mut self, mapper: u32, value: u32) -> Option<String> {
        match (mapper, value) {
            (RACE_NAMES, 0x10) => Some("Empyrean".into()),
            (SEX_NAMES, 0x1000) => Some("Male".into()),
            _ => None,
        }
    }
    fn enum_value(&mut self, _: u32, _: &str) -> Option<u32> {
        None
    }
}

#[test]
pub(super) fn naming_checks_the_expansion_and_sends_the_result() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.set_char_gen_system(Box::new(Stub));
    let mut without = set();
    without.legions = Some([0, 1]);
    to_selection(&mut ui, without);
    click_named(&mut ui, "CreateButton");
    ui.use_time(0.1);
    assert_eq!(ui.mode(), MODE_AVATAR_SELECTION);
    let avatar = &ui.avatar_selection().unwrap().avatars()[0];
    assert_eq!(avatar.state, tokenize("empyreanmale"));
    // The pick, a press, the animation's end: modification with the one slider.
    let viewport = find(&ui, "AvatarSelectionViewport");
    click(&mut ui, viewport);
    ui.finish_pick(Some(0));
    click(&mut ui, viewport);
    ui.finish_animation();
    ui.use_time(0.2);
    assert_eq!(ui.mode(), MODE_AVATAR_MODIFICATION);
    assert_eq!(
        ui.avatar_modification().unwrap().offered(),
        vec![phys_type::HEIGHT]
    );
    assert!(
        try_find(&ui, "BuildField").is_none(),
        "a type not offered loses its field"
    );
    // The height slider at the default's stop (1.0 of 0.9 / 1.0 / 1.1); its right button
    // steps to 1.1.
    let slider = find(&ui, "HeightSlider");
    assert_eq!(ui.tree().scrollbar_position(slider), 0.5);
    click_named(&mut ui, "HeightRightButton");
    let height = |ui: &PregameUi| phys_type_value(ui.data().current().unwrap(), phys_type::HEIGHT);
    assert_eq!(height(&ui), 1.1);
    // Naming: the expansion is missing, so nothing is sent.
    click_named(&mut ui, "NextButton");
    click_named(&mut ui, "NameEntry");
    let typed: Vec<(u16, u32)> = "Ab1c".encode_utf16().map(|c| (c, 0)).collect();
    ui.text_input(&[], &typed, false);
    click_named(&mut ui, "EnterButton");
    assert!(actions(&mut ui).is_empty());
    // With the expansion it goes out, named.
    let mut with = set();
    with.legions = Some([1, 1]);
    ui.character_set(with);
    click_named(&mut ui, "EnterButton");
    match actions(&mut ui).as_slice() {
        [PregameRequest::CreateCharacter(cgr)] => {
            assert_eq!(cgr.name, "Abc");
            assert_eq!(
                (cgr.race, cgr.sex, cgr.entity_did),
                (0x10, 0x1000, 0x4700_0001)
            );
            assert_eq!(phys_type_value(cgr, phys_type::HEIGHT), 1.1);
        }
        other => panic!("{other:?}"),
    }
    // Refused name: the error shows and the buttons come back.
    ui.char_gen_verification(3, None, 0);
    assert!(!ghosted(&ui, "EnterButton"));
}

pub(super) fn text_of(ui: &PregameUi, name: &str) -> String {
    ui.tree()
        .get(find(ui, name))
        .unwrap()
        .text()
        .unwrap_or_default()
}

/// `gmIntroUI`: three pictures of four seconds, then `intro_stop` closes it into the data patch
/// screen, which shows the connection and takes the set into character selection.
#[test]
pub(super) fn the_intro_runs_its_pictures_then_the_data_patch_screen_connects() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.use_time(0.0);
    assert_eq!(ui.mode(), MODE_INTRO);
    assert!(try_find(&ui, "IntroField").is_some());
    let mut t = 0.0;
    while ui.mode() == MODE_INTRO {
        t += 0.05;
        ui.use_time(t);
        assert!(t < 20.0, "the intro never stopped");
    }
    eprintln!("the intro ran {t:.2} s");
    assert!(t > 11.9, "every picture showed: {t}");
    assert_eq!(ui.mode(), MODE_DATA_PATCH);
    let connecting = text_of(&ui, "ErrorText");
    eprintln!("data patch: {connecting:?}");
    assert!(!connecting.is_empty());
    ui.connected();
    let connected = text_of(&ui, "ErrorText");
    assert!(!connected.is_empty() && connected != connecting);
    ui.character_set(set());
    ui.use_time(t + 0.05);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
}

/// `gmCreditsUI`: the credit strings scroll up the screen while the pictures change; a key
/// press, or the end of the scroll, returns to character selection.
#[test]
pub(super) fn the_credits_scroll_then_return_to_character_selection() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    to_selection(&mut ui, set());
    click_named(&mut ui, "CreditsButton");
    ui.use_time(0.1);
    assert_eq!(ui.mode(), MODE_CREDITS);
    let area = find(&ui, "TextArea");
    let text = ui.tree().get(area).unwrap().text().unwrap();
    assert!(text.starts_with("ASHERON'S CALL 2"), "{:?}", &text[..40]);
    assert!(text.lines().count() > 400);
    // The first loop shows a picture, fading, and the text starts below the field and rises.
    ui.use_time(0.2);
    let picture = find(&ui, "PictureField");
    assert_eq!(ui.tree().get(picture).unwrap().state(), tokenize("fading"));
    assert!(ui.tree().get(picture).unwrap().image().is_some());
    let start = ui.tree().screen_box(area).y0;
    assert!(start > 700, "{start}");
    ui.use_time(10.2);
    let later = ui.tree().screen_box(area).y0;
    assert!(later < start, "{later} < {start}");
    // The credits' sounds: a loop asks for the next sound unless the last one still plays.
    let sounds = |ui: &mut PregameUi| -> Vec<u32> {
        ui.take_requests()
            .into_iter()
            .filter_map(|r| match r {
                PregameRequest::CreditSound(did) => Some(did),
                _ => None,
            })
            .collect()
    };
    let mut heard = Vec::new();
    for step in 0..4 {
        ui.use_time(10.3 + f64::from(step) * 0.1);
        heard.extend(sounds(&mut ui));
    }
    assert!(!heard.is_empty(), "the credits name a sound");
    assert!(heard.iter().all(|&did| did != 0));
    ui.set_credit_sound_busy(true);
    ui.use_time(10.8);
    assert!(sounds(&mut ui).is_empty(), "no sound while one plays");
    ui.set_credit_sound_busy(false);
    // A key press closes it.
    ui.input(&[(KEY, 1.0, true), (KEY, 1.0, false)]);
    assert_eq!(ui.flow().next_mode(), Some(MODE_CHARACTER_SELECTION));
    ui.use_time(10.3);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
    // Again, to the end of the scroll.
    click_named(&mut ui, "CreditsButton");
    ui.use_time(20.0);
    assert_eq!(ui.mode(), MODE_CREDITS);
    let mut t = 20.0;
    while ui.mode() == MODE_CREDITS {
        t += 1.0;
        ui.use_time(t);
        assert!(t < 1000.0, "the credits never ended");
    }
    eprintln!("the credits ran {:.0} s", t - 20.0);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
}

/// `gmKeyboardUI`: the controls screen opens with nothing to save and quits back.
#[test]
pub(super) fn the_controls_screen_opens_and_quits_back() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    to_selection(&mut ui, set());
    click_named(&mut ui, "OptionsButton");
    ui.use_time(0.1);
    assert_eq!(ui.mode(), MODE_KEYBOARD);
    assert!(try_find(&ui, "list_box").is_some());
    assert!(ghosted(&ui, "save_button") && ghosted(&ui, "undo_button"));
    assert!(ghosted(&ui, "clear_button"));
    click_named(&mut ui, "quit_button");
    assert_eq!(ui.flow().next_mode(), Some(MODE_CHARACTER_SELECTION));
    ui.use_time(0.2);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
}

/// The controls screen's list items' texts (each row's `textid` child), in list order.
pub(super) fn list_texts(ui: &PregameUi) -> Vec<String> {
    let list = find(ui, "list_box");
    let textid = tokenize("textid");
    ui.tree()
        .list_items(list)
        .iter()
        .map(|&item| {
            let child = match ui.tree().get(item).unwrap().attribute(textid) {
                Some(crate::layout::AttributeValue::Id(id)) => ui.tree().child_recursive(item, *id),
                _ => None,
            };
            child
                .and_then(|c| ui.tree().get(c).unwrap().text())
                .unwrap_or_default()
        })
        .collect()
}

/// The controls screen, opened from character selection's options button.
pub(super) fn controls(ui: &mut PregameUi) {
    to_selection(ui, set());
    click_named(ui, "OptionsButton");
    ui.use_time(0.1);
    assert_eq!(ui.mode(), MODE_KEYBOARD);
}

/// Selects list item `index` (a header selects its first key).
pub(super) fn select_item(ui: &mut PregameUi, index: usize) {
    let list = find(ui, "list_box");
    ui.tree_mut()
        .list_set_selected_index(list, Some(index), true);
    ui.input(&[]);
}

/// A key the defaults leave unbound (`DIK_APPS`).
const APPS: u32 = 0x8100_04DD;

pub(super) fn press(ui: &mut PregameUi, semantic: u32) {
    ui.key_presses(&[crate::input::KeyPress {
        semantic,
        meta_mode: 0,
    }]);
}

/// The dialog on screen answers `yes`.
pub(super) fn answer(ui: &mut PregameUi, yes: bool) {
    let context = ui.shown_dialog().expect("a dialog on screen");
    ui.answer_dialog(context, yes);
}

/// `gmKeyboardUI::Create`: a row per action (header, then two keys) with the names as text.
#[test]
pub(super) fn the_controls_screen_lists_each_action_with_its_two_keys() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    controls(&mut ui);
    let texts = list_texts(&ui);
    assert!(
        !texts.is_empty() && texts.len().is_multiple_of(3),
        "{texts:?}"
    );
    assert!(texts.iter().step_by(3).all(|t| !t.is_empty()), "{texts:?}");
    assert!(
        texts
            .iter()
            .enumerate()
            .any(|(i, t)| i % 3 != 0 && !t.is_empty()),
        "some key is bound: {texts:?}"
    );
    assert!(ghosted(&ui, "clear_button"));
    assert!(ghosted(&ui, "save_button") && ghosted(&ui, "undo_button"));
}

/// `KeyHitHandler` and `FinishSet` bind the key to the selected slot; `SaveKeyboardList` hands the
/// maps to the input manager and the list is unmodified.
#[test]
pub(super) fn a_key_is_bound_to_the_selected_slot_and_saved_to_the_input_manager() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    controls(&mut ui);
    select_item(&mut ui, 1);
    assert!(!ghosted(&ui, "clear_button"));
    press(&mut ui, APPS);
    assert_eq!(list_texts(&ui)[1], "Apps");
    assert!(!ghosted(&ui, "save_button") && !ghosted(&ui, "undo_button"));
    click_named(&mut ui, "save_button");
    let requests = actions(&mut ui);
    let saved = requests
        .iter()
        .find_map(|r| match r {
            PregameRequest::KeyMaps(set) => Some(set),
            _ => None,
        })
        .expect("the saved maps");
    assert!(saved.main.mappings.iter().any(|m| m.semantic == APPS));
    assert!(ghosted(&ui, "save_button") && ghosted(&ui, "undo_button"));
}

/// `StartSet`'s already-mapped question: yes moves the key and clears the other slot; no keeps
/// both as they were.
#[test]
pub(super) fn a_key_already_bound_to_another_slot_asks_before_it_moves() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    controls(&mut ui);
    select_item(&mut ui, 1);
    press(&mut ui, APPS);
    let other = list_texts(&ui)[4].clone();
    select_item(&mut ui, 4);
    press(&mut ui, APPS);
    assert!(ui.shown_dialog().is_some(), "the question is up");
    answer(&mut ui, false);
    let texts = list_texts(&ui);
    assert_eq!(
        (texts[1].as_str(), texts[4].as_str()),
        ("Apps", other.as_str())
    );
    select_item(&mut ui, 4);
    press(&mut ui, APPS);
    answer(&mut ui, true);
    let texts = list_texts(&ui);
    assert_eq!((texts[1].as_str(), texts[4].as_str()), ("", "Apps"));
}

/// `ListenToMessage`'s clear all, undo, restore and quit questions: each asks, and each answer
/// does what retail does.
#[test]
pub(super) fn clear_all_undo_restore_and_quit_ask_first() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    controls(&mut ui);
    let original = list_texts(&ui);
    click_named(&mut ui, "clear_all_button");
    answer(&mut ui, true);
    let cleared = list_texts(&ui);
    assert!(cleared
        .iter()
        .enumerate()
        .all(|(i, t)| i % 3 == 0 || t.is_empty()));
    // Undo asks; no keeps the clearing.
    click_named(&mut ui, "undo_button");
    answer(&mut ui, false);
    assert_eq!(list_texts(&ui), cleared);
    // Yes rebuilds the list from the input manager's maps.
    click_named(&mut ui, "undo_button");
    answer(&mut ui, true);
    assert_eq!(list_texts(&ui), original);
    // A restore preset asks, then its maps are the input manager's and the list is modified.
    click_named(&mut ui, "restore_eq_button");
    answer(&mut ui, true);
    assert!(!ghosted(&ui, "save_button"));
    assert!(actions(&mut ui)
        .iter()
        .any(|r| matches!(r, PregameRequest::KeyMaps(_))));
    // Quit with changes asks first: no stays, yes leaves for character selection.
    click_named(&mut ui, "quit_button");
    assert!(ui.shown_dialog().is_some());
    answer(&mut ui, false);
    assert_eq!(ui.mode(), MODE_KEYBOARD);
    click_named(&mut ui, "quit_button");
    answer(&mut ui, true);
    ui.use_time(0.2);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
}

/// The UI reopened after a session ended goes straight to character selection with the list the
/// session kept (no intro, no data patch).
#[test]
pub(super) fn a_resumed_ui_lists_the_kept_set_at_character_selection() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.resume_at_character_selection(set());
    ui.use_time(0.0);
    assert_eq!(ui.mode(), MODE_CHARACTER_SELECTION);
    let list = find(&ui, "CharacterListBox");
    let texts: Vec<_> = ui
        .tree()
        .list_items(list)
        .to_vec()
        .iter()
        .map(|&i| ui.tree().get(i).unwrap().text().unwrap())
        .collect();
    assert_eq!(texts, ["Alpha", "Beta"]);
}
