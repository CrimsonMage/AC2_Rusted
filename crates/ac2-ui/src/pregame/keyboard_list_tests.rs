//! `gmKeyboardUI`'s binding list on synthetic action and key maps.

use std::collections::{BTreeMap, HashMap};

use crate::maps::{ActionMap, ActionMapping, KeyMap};
use crate::pregame::{KeyBinding, KeyboardList, SetOutcome, SlotRef, META_CONTROL, META_NONE};

const MOVE: u32 = 0x4000_0100;
const JUMP: u32 = 0x4000_0101;
const HIDDEN: u32 = 0x4000_0102;
const W: u32 = 0x8100_0457;
const SPACE: u32 = 0x8100_0439;
const K: u32 = 0x8100_0425;
const MOUSE_LEFT: u32 = 0x8200_0300;

fn actions() -> ActionMap {
    // MOVE and JUMP have descriptions; HIDDEN has none, so it has no row.
    let mut descriptions = HashMap::new();
    descriptions.insert(JUMP, (11, 12));
    descriptions.insert(MOVE, (9, 10));
    ActionMap {
        string_table: 1,
        descriptions,
        actions: BTreeMap::new(),
        toggles: HashMap::new(),
    }
}

fn map(name: &str, mappings: &[(u32, u32)]) -> KeyMap {
    KeyMap {
        name: name.to_string(),
        guid: [0; 16],
        mappings: mappings
            .iter()
            .map(|&(action, semantic)| ActionMapping {
                action,
                semantic,
                name: String::new(),
            })
            .collect(),
    }
}

fn key(semantic: u32) -> KeyBinding {
    KeyBinding {
        semantic,
        meta: META_NONE,
    }
}

/// The main map binds W and SPACE to MOVE and JUMP, the control map binds K to MOVE, and HIDDEN
/// (no row) binds K too.
fn list() -> KeyboardList {
    let main = map("main", &[(MOVE, W), (JUMP, SPACE), (HIDDEN, K)]);
    let control = map("control", &[(MOVE, K), (MOVE, MOUSE_LEFT), (MOVE, SPACE)]);
    KeyboardList::from_maps(&actions(), &[(&main, META_NONE), (&control, META_CONTROL)])
}

#[test]
fn rows_follow_the_actions_with_descriptions_and_two_slots() {
    let list = list();
    let rows = list.rows();
    // Action order: MOVE (0x40000100), then JUMP. HIDDEN has no description and no row.
    assert_eq!(
        rows.iter().map(|r| r.action).collect::<Vec<_>>(),
        [MOVE, JUMP]
    );
    assert_eq!((rows[0].name, rows[0].description), (9, 10));
    // MOVE's first two bindings in map order; the third is not listed.
    assert_eq!(
        rows[0].slots,
        [
            Some(KeyBinding {
                semantic: W,
                meta: META_NONE
            }),
            Some(KeyBinding {
                semantic: K,
                meta: META_CONTROL
            }),
        ]
    );
    assert_eq!(rows[1].slots[0], Some(key(SPACE)));
    assert_eq!(rows[1].slots[1], None);
    assert!(!list.modified());
    assert_eq!(list.selected(), None);
}

#[test]
fn a_header_click_selects_its_first_slot_and_a_slot_click_selects_that_slot() {
    let mut list = list();
    // Item 0 is MOVE's header: the first slot is selected instead.
    assert_eq!(list.click(0), Some(SlotRef { row: 0, slot: 0 }));
    assert_eq!(list.click(2), Some(SlotRef { row: 0, slot: 1 }));
    // Item 3 is JUMP's header, item 4 its first slot.
    assert_eq!(list.click(4), Some(SlotRef { row: 1, slot: 0 }));
    // Past the last row there is nothing to select.
    assert_eq!(list.click(9), None);
}

#[test]
fn a_free_key_is_set_on_the_selected_slot() {
    let mut list = list();
    list.click(4);
    assert_eq!(list.start_set(key(MOUSE_LEFT)), Some(SetOutcome::Set));
    assert!(list.modified());
    assert_eq!(list.rows()[1].slots[0], Some(key(MOUSE_LEFT)));
    // The selection clears once the key is written.
    assert_eq!(list.selected(), None);
}

#[test]
fn a_key_on_another_slot_is_already_mapped_and_nothing_is_written() {
    let mut list = list();
    list.click(4);
    // W is MOVE's first slot: JUMP's selection cannot take it.
    let outcome = list.start_set(key(W));
    assert_eq!(
        outcome,
        Some(SetOutcome::AlreadyMapped {
            occupied: SlotRef { row: 0, slot: 0 }
        })
    );
    assert!(!list.modified());
    assert_eq!(list.rows()[1].slots[0], Some(key(SPACE)));
    assert_eq!(list.selected(), Some(SlotRef { row: 1, slot: 0 }));
}

#[test]
fn a_free_key_after_a_conflict_clears_the_conflicting_slot() {
    // `FinishSet` clears `m_occupied`: the slot the conflict named loses its key.
    let mut list = list();
    list.click(4);
    list.start_set(key(W));
    assert_eq!(list.start_set(key(MOUSE_LEFT)), Some(SetOutcome::Set));
    assert_eq!(list.rows()[0].slots[0], None);
    assert_eq!(list.rows()[1].slots[0], Some(key(MOUSE_LEFT)));
    assert!(list.modified());
}

#[test]
fn setting_a_slot_to_its_own_key_is_not_a_conflict() {
    let mut list = list();
    list.click(1);
    assert_eq!(list.start_set(key(W)), Some(SetOutcome::Set));
    assert_eq!(list.rows()[0].slots[0], Some(key(W)));
}

#[test]
fn a_set_without_a_selection_does_nothing() {
    let mut list = list();
    assert_eq!(list.start_set(key(MOUSE_LEFT)), None);
    assert!(!list.modified());
}

#[test]
fn keys_are_read_only_while_a_slot_is_selected_and_no_dialog_is_open() {
    let mut list = list();
    // Nothing selected: the key does nothing.
    assert_eq!(list.key_hit(MOUSE_LEFT, META_NONE, false, false), None);
    list.click(4);
    // A dialog, a reserved key and the mouse semantics `KeyHitHandler` ignores.
    assert_eq!(list.key_hit(MOUSE_LEFT, META_NONE, true, false), None);
    assert_eq!(list.key_hit(MOUSE_LEFT, META_NONE, false, true), None);
    assert_eq!(list.key_hit(0x8200_0304, META_NONE, false, false), None);
    assert!(!list.modified());
    assert_eq!(
        list.key_hit(SPACE, META_NONE, false, false),
        Some(SetOutcome::Set)
    );
    assert_eq!(list.rows()[1].slots[0], Some(key(SPACE)));
}

#[test]
fn the_cancel_key_leaves_the_set() {
    let mut list = list();
    list.click(4);
    assert_eq!(list.key_hit(0x8200_040C, META_NONE, false, false), None);
    assert_eq!(list.selected(), None);
    // With the selection gone, the next key does nothing.
    assert_eq!(list.key_hit(SPACE, META_NONE, false, false), None);
    assert!(!list.modified());
}

#[test]
fn the_clear_button_empties_the_selected_slot() {
    let mut list = list();
    list.click(1);
    list.clear_selected();
    assert_eq!(list.rows()[0].slots[0], None);
    assert!(list.modified());
    assert_eq!(list.selected(), None);
    // Without a selection the clear button changes nothing more.
    list.clear_selected();
    assert_eq!(
        list.rows()[0].slots[1],
        Some(key_with_meta(K, META_CONTROL))
    );
}

#[test]
fn clear_all_empties_every_slot_and_marks_the_list_modified() {
    let mut list = list();
    list.click(1);
    list.clear_all();
    assert!(list
        .rows()
        .iter()
        .all(|r| r.slots.iter().all(|s| s.is_none())));
    assert!(list.modified());
    assert_eq!(list.selected(), None);
}

#[test]
fn bindings_list_the_filled_slots_in_list_order() {
    let mut list = list();
    assert_eq!(
        list.bindings(),
        vec![
            (MOVE, key(W)),
            (MOVE, key_with_meta(K, META_CONTROL)),
            (JUMP, key(SPACE)),
        ]
    );
    list.click(4);
    list.clear_selected();
    assert_eq!(
        list.bindings(),
        vec![(MOVE, key(W)), (MOVE, key_with_meta(K, META_CONTROL)),]
    );
}

fn key_with_meta(semantic: u32, meta: u32) -> KeyBinding {
    KeyBinding { semantic, meta }
}
