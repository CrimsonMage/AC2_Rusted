//! The pregame flow and arms without data, and character selection on the real layouts
//! (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::{CharacterIdentity, CharacterSet};

use super::*;
use crate::token::{tokenize, BUTTON_GHOSTED};
use crate::{DIALOG_CONFIRMATION, DIALOG_WAIT, UI_POINTER_X, UI_POINTER_Y};

fn headings(arms: &CharacterArms) -> Vec<f32> {
    arms.arms()
        .iter()
        .map(|a| (a.frame().heading_degrees() * 100.0).round() / 100.0)
        .collect()
}

#[test]
fn the_arms_share_the_circle_and_turn_the_selection_to_the_front() {
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
fn login_errors_pick_their_mode_and_string() {
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

fn character(id: u64, name: &str, greyed: u32) -> CharacterIdentity {
    CharacterIdentity {
        id,
        name: name.into(),
        seconds_greyed_out: greyed,
        visual: Default::default(),
    }
}

fn set() -> CharacterSet {
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

fn click_named(ui: &mut PregameUi, name: &str) {
    let element = find(ui, name);
    click(ui, element);
}

fn click(ui: &mut PregameUi, element: usize) {
    let b = ui.tree().screen_box(element);
    let (x, y) = ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2);
    ui.input(&[
        (UI_POINTER_X, x as f32, true),
        (UI_POINTER_Y, y as f32, true),
        (0x4000_0004, 1.0, true),
        (0x4000_0004, 1.0, false),
    ]);
}

fn try_find(ui: &PregameUi, name: &str) -> Option<usize> {
    let id = tokenize(name);
    ui.tree().roots().iter().find_map(|&r| {
        if ui.tree().get(r).unwrap().id() == id {
            Some(r)
        } else {
            ui.tree().child_recursive(r, id)
        }
    })
}

fn find(ui: &PregameUi, name: &str) -> usize {
    try_find(ui, name).unwrap_or_else(|| panic!("{name}"))
}

fn ghosted(ui: &PregameUi, name: &str) -> bool {
    ui.tree()
        .get(find(ui, name))
        .unwrap()
        .attribute(BUTTON_GHOSTED)
        == Some(&crate::layout::AttributeValue::Bool(true))
}

fn real_ui() -> Option<PregameUi> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let assets = load_pregame(&store).unwrap();
    for layout in PREGAME_LAYOUTS {
        assert!(assets.layout(layout).is_some(), "{layout:08x}");
    }
    assert!(!assets.images.is_empty());
    Some(PregameUi::new(assets, 1024, 768))
}

#[test]
fn character_selection_lists_the_set_and_enters_with_the_selection() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    assert_eq!(ui.mode(), 0, "connecting");
    ui.character_set(set());
    ui.use_time(0.0);
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
        ui.take_requests(),
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
fn deleting_asks_first_and_create_opens_avatar_selection() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.character_set(set());
    ui.use_time(0.0);
    let delete = find(&ui, "DeleteButton");
    click(&mut ui, delete);
    let (context, kind) = ui.dialogs().shown().expect("confirmation");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert!(context > 0 && ghosted(&ui, "CreateButton"));
    // No: nothing sent, the buttons back.
    click_named(&mut ui, "NoButton");
    assert!(ui.take_requests().is_empty());
    assert!(ui.dialogs().shown().is_none() && !ghosted(&ui, "CreateButton"));
    // Yes: the delete request.
    click_named(&mut ui, "DeleteButton");
    click_named(&mut ui, "YesButton");
    assert_eq!(
        ui.take_requests(),
        vec![PregameRequest::DeleteCharacter(0x5000_0001)]
    );
    // Create: please wait, then avatar selection.
    ui.character_set(set());
    click_named(&mut ui, "CreateButton");
    assert_eq!(ui.flow().next_mode(), Some(MODE_AVATAR_SELECTION));
}

/// The composed screen, surfaces over black in compose order.
fn snapshot(ui: &mut PregameUi, name: &str) {
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
fn character_selection_draws() {
    let Some(mut ui) = real_ui() else {
        return;
    };
    ui.character_set(set());
    ui.use_time(0.0);
    snapshot(&mut ui, "pregame_character_selection.png");
    click_named(&mut ui, "DeleteButton");
    snapshot(&mut ui, "pregame_delete_confirmation.png");
}
