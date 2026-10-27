//! The creation screens' answers (the data patch screen, the naming dialog) and the avatar
//! modification tooltips, on the real layouts (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_db::ResourceStore;

use super::tests::*;
use super::*;
use crate::token::tokenize;

/// `gmDataPatchUI::RecvNotice_CharGenVerification`: without a character set the screen ignores
/// the answer; with the set queued (not yet shown), a created character is entered at once and
/// any other answer queues character selection with no text.
#[test]
fn the_data_patch_screen_answers_a_created_character() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.use_time(0.0);
    ui.input(&[(KEY, 1.0, true), (KEY, 1.0, false)]);
    ui.use_time(0.0);
    assert_eq!(ui.mode(), MODE_DATA_PATCH);
    let created = character(0x5000_0009, "Gamma", 0);
    ui.char_gen_verification(1, Some(&created), 0);
    ui.use_time(0.1);
    assert_eq!(ui.mode(), MODE_DATA_PATCH, "no set yet: ignored");
    assert!(!actions(&mut ui)
        .iter()
        .any(|r| matches!(r, PregameRequest::EnterWorld(_))));

    ui.character_set(set());
    ui.char_gen_verification(1, Some(&created), 0);
    assert_eq!(ui.data().characters().unwrap().characters.len(), 3);
    assert!(actions(&mut ui).contains(&PregameRequest::EnterWorld(created.id)));
    ui.use_time(0.2);
    assert_eq!(ui.mode(), MODE_GAMEPLAY);

    let mut refused = real_ui().unwrap();
    refused.use_time(0.0);
    refused.input(&[(KEY, 1.0, true), (KEY, 1.0, false)]);
    refused.use_time(0.0);
    refused.character_set(set());
    refused.char_gen_verification(3, None, 0);
    refused.use_time(0.3);
    assert_eq!(refused.mode(), MODE_CHARACTER_SELECTION);
    assert!(!actions(&mut refused).iter().any(|r| matches!(
        r,
        PregameRequest::EnterWorld(_) | PregameRequest::CreateCharacter(_)
    )));
}

/// The creation's answers on the naming dialog: each code shows its retail string, the
/// weenie's error for response 5 (`EnumMapper` 0x40000035 has no string in the stub, so the
/// variable is empty), and the buttons come back.
#[test]
fn the_creation_answers_show_their_retail_strings() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.set_char_gen_system(Box::new(Stub));
    to_selection(&mut ui, set());
    click_named(&mut ui, "CreateButton");
    ui.use_time(0.1);
    let viewport = find(&ui, "AvatarSelectionViewport");
    click(&mut ui, viewport);
    ui.finish_pick(Some(0));
    click(&mut ui, viewport);
    ui.finish_animation();
    ui.use_time(0.2);
    click_named(&mut ui, "NextButton");
    click_named(&mut ui, "NameEntry");
    let typed: Vec<(u16, u32)> = "Abc".encode_utf16().map(|c| (c, 0)).collect();
    ui.text_input(&[], &typed, crate::chat::Modifiers::default());
    click_named(&mut ui, "EnterButton");
    assert!(matches!(
        actions(&mut ui).as_slice(),
        [PregameRequest::CreateCharacter(_)]
    ));
    let dataset = ac2_dat::test_support::env_dataset().unwrap();
    let strings = load_pregame(&Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .strings;
    let cases: [(u32, u32, &str); 13] = [
        (2, 0, "ID_Character_Err_Pending"),
        (3, 0, "ID_Character_Err_NameInUse"),
        (4, 0, "ID_Character_Err_NameBanned"),
        (5, 0, "ID_Character_Err_Corrupt"),
        (5, 0xC749, "ID_Character_Err_Gun_Specific_Error"),
        (5, 0xC74A, "ID_Character_Err_NameInUse"),
        (5, 0xC74B, "ID_Character_Err_NameBanned"),
        (5, 200, "ID_Character_Err_NameBanned"),
        (5, 0x1234, "ID_Character_Err_Weenie_Specific_Error"),
        (6, 0, "ID_Character_Err_DatabaseDown"),
        (7, 0, "ID_Character_Err_AdminPriv"),
        (8, 0, "ID_Character_Err_NoMoreSlots"),
        (9, 0, "ID_Character_Err_Unknown"),
    ];
    for (response, weenie, id) in cases {
        ui.char_gen_verification(response, None, weenie);
        let vars = vec![
            crate::pregame::strings::int_var("response", response as i32),
            crate::pregame::strings::text_var("weenieError", ""),
        ];
        let expected = strings.render(tokenize(id), vars);
        assert_eq!(
            text_of(&ui, "GenerationConfirmationErrorMsg"),
            expected,
            "response {response} weenie {weenie:#x}"
        );
        assert!(
            !ghosted(&ui, "EnterButton"),
            "{response}: the buttons return"
        );
    }
}

/// `gmAvatarModificationUI::InitElements`: each slider and its arrows carry the tooltip string
/// ids (`ID_AvModify_TT_Slider`, `_RButton`, `_LButton`) with the slider's label as `label`.
#[test]
fn the_slider_tooltips_come_from_their_string_ids() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.set_char_gen_system(Box::new(Stub));
    to_selection(&mut ui, set());
    click_named(&mut ui, "CreateButton");
    ui.use_time(0.1);
    let viewport = find(&ui, "AvatarSelectionViewport");
    click(&mut ui, viewport);
    ui.finish_pick(Some(0));
    click(&mut ui, viewport);
    ui.finish_animation();
    ui.use_time(0.2);
    assert_eq!(ui.mode(), MODE_AVATAR_MODIFICATION);
    let dataset = ac2_dat::test_support::env_dataset().unwrap();
    let strings = load_pregame(&Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .strings;
    let label = strings.render(
        tokenize("ID_AvModify_TT_RButton"),
        vec![crate::pregame::strings::text_var("label", "Tall")],
    );
    for (element, id) in [
        ("HeightSlider", "ID_AvModify_TT_Slider"),
        ("HeightRightButton", "ID_AvModify_TT_RButton"),
        ("HeightLeftButton", "ID_AvModify_TT_LButton"),
    ] {
        let tip = ui.tree().tooltip_text(find(&ui, element));
        let expected = strings.render(
            tokenize(id),
            vec![crate::pregame::strings::text_var("label", "Tall")],
        );
        assert_eq!(tip.as_deref(), Some(expected.as_str()), "{element}");
    }
    assert!(!label.is_empty());
}

/// `gmAvatarSelectionUI::MouseOnObject` / `MouseOffObject`: the viewport's tooltip is
/// `ID_AvSelect_TT_3DAvatar` while an avatar is under the pointer, and the bottom field's
/// `primSong` is requested when the screen opens.
#[test]
fn the_avatar_under_the_pointer_sets_the_viewport_tooltip() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.set_char_gen_system(Box::new(Stub));
    to_selection(&mut ui, set());
    let _ = ui.take_requests();
    click_named(&mut ui, "CreateButton");
    ui.use_time(0.1);
    let music = ui
        .take_requests()
        .into_iter()
        .filter(|r| matches!(r, PregameRequest::Music(_)))
        .count();
    assert_eq!(music, 1, "primSong plays once");
    let viewport = find(&ui, "AvatarSelectionViewport");
    let dataset = ac2_dat::test_support::env_dataset().unwrap();
    let strings = load_pregame(&Arc::new(ResourceStore::new(dataset)))
        .unwrap()
        .strings;
    assert_eq!(ui.tree().tooltip_text(viewport), None);
    ui.finish_pick(Some(0));
    assert_eq!(
        ui.tree().tooltip_text(viewport),
        Some(strings.get("ID_AvSelect_TT_3DAvatar"))
    );
    ui.finish_pick(None);
    assert_eq!(ui.tree().tooltip_text(viewport), None);
}

/// `StringInfo::InqStringInternal` fetches the table a `StringInfo` names: the character
/// generation script's slider labels (table 0x250007C9) are text, not a raw id.
#[test]
fn a_label_in_a_script_table_renders_as_text() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let strings = crate::pregame::UiStrings::load(&ResourceStore::new(dataset)).unwrap();
    let text = strings.render_in(0x2500_07C9, 0x4E0D_DF65);
    assert!(!text.starts_with("0x"), "{text}");
    assert!(!text.is_empty());
}
