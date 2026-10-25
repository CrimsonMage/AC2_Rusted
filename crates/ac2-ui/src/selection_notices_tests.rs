//! `gmSelectionUI`'s notice handlers and the non-cycling selection keys on synthetic selections.

use crate::selection_notices::{
    being_deleted, direct_key, last_attacker_notice, selectability_notice, set_targeting_pointer,
    SelectionAction, SelectionSnapshot, AUTO_TARGET_KEY, CLEAR_KEY, LAST_ATTACKER_KEY,
    LAST_SELECTION_KEY, RESELECT_KEY,
};

const PLAYER: u64 = 100;
const FOE: u64 = 7;
const OLD: u64 = 3;
const ATTACKER: u64 = 9;

fn snapshot(selection: Option<u64>) -> SelectionSnapshot {
    SelectionSnapshot {
        player: PLAYER,
        selection,
        last_selection: Some(OLD),
        last_attacker: Some(ATTACKER),
    }
}

#[test]
fn last_attacker_selects_only_when_auto_target_is_on_and_nothing_is_selected() {
    // `RecvNotice_E_LastAttacker`: `GetAutoTarget` and not `HasSomethingSelected`.
    assert_eq!(
        last_attacker_notice(ATTACKER, true, false),
        vec![SelectionAction::Select(Some(ATTACKER))]
    );
    assert!(last_attacker_notice(ATTACKER, true, true).is_empty());
    assert!(last_attacker_notice(ATTACKER, false, false).is_empty());
}

#[test]
fn selectability_leaves_a_selectable_or_unselected_object_alone() {
    // `RecvNotice_UpdateSelectability`: only a selected, non-player object going unselectable.
    assert!(selectability_notice(FOE, true, &snapshot(Some(FOE)), true).is_empty());
    assert!(selectability_notice(FOE, false, &snapshot(Some(OLD)), true).is_empty());
    assert!(selectability_notice(FOE, false, &snapshot(None), true).is_empty());
    assert!(selectability_notice(PLAYER, false, &snapshot(Some(PLAYER)), true).is_empty());
}

#[test]
fn selectability_clears_stops_attacking_and_picks_the_next_foe() {
    // Retail order: SetSelection(INVALID), StopAttacking(obj), then Select(0x41000024) when
    // the auto-target option is on.
    assert_eq!(
        selectability_notice(FOE, false, &snapshot(Some(FOE)), true),
        vec![
            SelectionAction::Select(None),
            SelectionAction::StopAttacking(FOE),
            SelectionAction::Key(AUTO_TARGET_KEY),
        ]
    );
    assert_eq!(
        selectability_notice(FOE, false, &snapshot(Some(FOE)), false),
        vec![
            SelectionAction::Select(None),
            SelectionAction::StopAttacking(FOE),
        ]
    );
}

#[test]
fn being_deleted_ignores_objects_that_are_not_the_selection() {
    assert!(being_deleted(FOE, &snapshot(Some(OLD)), true).is_empty());
    assert!(being_deleted(FOE, &snapshot(None), true).is_empty());
}

#[test]
fn being_deleted_picks_the_next_foe_then_clears_when_still_selected() {
    // `RecvNotice_BeingDeleted`: the auto-target runs before the second check, which clears the
    // selection only if it is still the deleted object.
    assert_eq!(
        being_deleted(FOE, &snapshot(Some(FOE)), true),
        vec![
            SelectionAction::StopAttacking(FOE),
            SelectionAction::Key(AUTO_TARGET_KEY),
            SelectionAction::ClearIfSelected(FOE),
        ]
    );
    assert_eq!(
        being_deleted(FOE, &snapshot(Some(FOE)), false),
        vec![SelectionAction::ClearIfSelected(FOE)]
    );
}

#[test]
fn reselect_picks_the_player_unless_already_selected_then_the_last_selection() {
    // `Select(0x41000018)`: SetSelection(player) when not selected; else the last selection.
    assert_eq!(
        direct_key(RESELECT_KEY, &snapshot(Some(FOE)), false),
        Some(vec![SelectionAction::Select(Some(PLAYER))])
    );
    assert_eq!(
        direct_key(RESELECT_KEY, &snapshot(Some(PLAYER)), false),
        Some(vec![SelectionAction::Select(Some(OLD))])
    );
}

#[test]
fn last_selection_and_last_attacker_keys_select_the_stored_objects() {
    // `Select(0x41000028)` and `Select(0x4100002A)` (`_lastAttackerID`).
    assert_eq!(
        direct_key(LAST_SELECTION_KEY, &snapshot(None), false),
        Some(vec![SelectionAction::Select(Some(OLD))])
    );
    assert_eq!(
        direct_key(LAST_ATTACKER_KEY, &snapshot(None), false),
        Some(vec![SelectionAction::Select(Some(ATTACKER))])
    );
}

#[test]
fn an_empty_last_selection_clears_the_selection() {
    let mut empty = snapshot(Some(FOE));
    empty.last_selection = None;
    assert_eq!(
        direct_key(LAST_SELECTION_KEY, &empty, false),
        Some(vec![SelectionAction::Select(None)])
    );
}

#[test]
fn clear_key_clears_unless_the_input_takes_typing() {
    // `Select(0x4000001E)`: `ICIDM`'s text mode returns before the clear.
    assert_eq!(
        direct_key(CLEAR_KEY, &snapshot(Some(FOE)), false),
        Some(vec![SelectionAction::Select(None)])
    );
    assert_eq!(
        direct_key(CLEAR_KEY, &snapshot(Some(FOE)), true),
        Some(Vec::new())
    );
}

#[test]
fn cycling_keys_are_not_direct_keys() {
    // `0x41000019` (previous) and `0x41000024` (auto-target) cycle through `select_target`.
    assert_eq!(direct_key(0x4100_0019, &snapshot(None), false), None);
    assert_eq!(direct_key(AUTO_TARGET_KEY, &snapshot(None), false), None);
}

#[test]
fn the_targeting_pointer_swaps_the_field_cursor_and_restores_it() {
    // `RecvNotice_SetTargetingPointer` on the built HUD's `ItemSelectionField`: the usage cursor
    // replaces the cursor (the old one saved), and off restores the saved one.
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut tree = crate::build_gameplay_hud(&store, 1024, 768).unwrap().tree;
    let field = crate::token::tokenize("ItemSelectionField");
    let index = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.el(r).id() == field)
        .expect("ItemSelectionField root");
    let before = tree.el(index).cursor;
    let usage = tree
        .el(index)
        .attribute(crate::token::tokenize("usage_cursor"))
        .cloned();
    assert!(usage.is_some(), "the field has a usage cursor");
    set_targeting_pointer(&mut tree, true);
    let after = tree.el(index).cursor;
    assert_eq!(tree.el(index).saved_cursor, before);
    assert_ne!(after, before);
    set_targeting_pointer(&mut tree, false);
    assert_eq!(tree.el(index).cursor, before);
}
