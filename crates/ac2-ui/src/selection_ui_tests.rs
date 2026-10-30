//! `gmSelectionUI::Finish3DSelection` on synthetic potential lists: the first entry's
//! selectability, the player's selectable bits, and the clicks that wait for a selection.

use crate::mouse::UiMessage;
use crate::selection_ui::{PotentialTarget, SelectionUi};
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};

const ROCK: u64 = 5;
const FOE: u64 = 7;
/// The player's type with every selectable bit (`0x3f`) set.
const ALL_BITS: u32 = 0x3f;

/// A press on the item selection field's top layer (`mouse_press_top`) with `action`.
fn press(action: u32) -> UiMessage {
    UiMessage {
        element_id: tokenize("ItemSelectionField"),
        element: None,
        message: MSG_MOUSE_PRESS_TOP,
        data: action as i32,
        item: None,
    }
}

const PLAYER: u64 = 0x8000_0000_0000_0001;

fn target(id: u64, selectable: Option<bool>) -> PotentialTarget {
    PotentialTarget {
        id,
        selectable,
        wielder: 0,
    }
}

#[test]
fn a_pick_selects_its_first_entry_nearest_first() {
    let mut ui = SelectionUi::default();
    let outcome = ui
        .finish(
            &[target(FOE, Some(true)), target(ROCK, Some(true))],
            0,
            PLAYER,
        )
        .unwrap();
    assert_eq!(outcome.select, Some(FOE));
    assert!(!outcome.use_selection && !outcome.open_examination);
}

#[test]
fn an_empty_potential_list_clears_the_selection() {
    let mut ui = SelectionUi::default();
    let outcome = ui.finish(&[], 0, PLAYER).unwrap();
    assert_eq!(outcome.select, None);
}

#[test]
fn an_unselectable_first_target_is_refused_unless_the_player_has_all_bits() {
    let mut ui = SelectionUi::default();
    assert_eq!(ui.finish(&[target(ROCK, Some(false))], 0, PLAYER), None);
    let outcome = ui
        .finish(&[target(ROCK, Some(false))], ALL_BITS, PLAYER)
        .unwrap();
    assert_eq!(outcome.select, Some(ROCK));
}

#[test]
fn a_target_without_a_record_skips_the_selectability_check() {
    let mut ui = SelectionUi::default();
    let outcome = ui.finish(&[target(ROCK, None)], 0, PLAYER).unwrap();
    assert_eq!(outcome.select, Some(ROCK));
}

#[test]
fn a_refused_pick_keeps_its_clicks_for_the_next_one_as_retail_does() {
    let mut ui = SelectionUi::default();
    // A double click on an unselectable target: nothing changes, the double click waits.
    ui.message(&press(0x4000_0007));
    assert_eq!(ui.finish(&[target(ROCK, Some(false))], 0, PLAYER), None);
    let outcome = ui.finish(&[target(FOE, Some(true))], 0, PLAYER).unwrap();
    assert!(outcome.use_selection, "the pending double click is used");
}

#[test]
fn a_right_click_opens_the_examination_only_with_a_target() {
    let mut ui = SelectionUi::default();
    ui.message(&press(0x4000_0005));
    assert!(!ui.finish(&[], 0, PLAYER).unwrap().open_examination);
    ui.message(&press(0x4000_0005));
    assert!(
        ui.finish(&[target(FOE, Some(true))], 0, PLAYER)
            .unwrap()
            .open_examination
    );
}

#[test]
fn a_double_click_does_not_use_an_item_the_player_wields() {
    let mut ui = SelectionUi::default();
    let worn = PotentialTarget {
        id: ROCK,
        selectable: Some(true),
        wielder: PLAYER,
    };
    ui.message(&press(0x4000_0007));
    let outcome = ui.finish(&[worn], 0, PLAYER).unwrap();
    assert_eq!(outcome.select, Some(ROCK));
    assert!(!outcome.use_selection);
    // Someone else's wielded item is used.
    ui.message(&press(0x4000_0007));
    let other = PotentialTarget {
        wielder: PLAYER + 1,
        ..worn
    };
    assert!(ui.finish(&[other], 0, PLAYER).unwrap().use_selection);
}
