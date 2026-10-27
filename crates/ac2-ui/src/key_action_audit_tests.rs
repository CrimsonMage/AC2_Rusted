//! The key actions the DAT maps bind, each traced to its Legions 7160 handler and to ours.
//!
//! The maps are `KEYMAP_MAIN`, `KEYMAP_SHIFT` and `KEYMAP_CONTROL` (`gmClient::InitKeymap`) and
//! the preproc map (`CInputManager_WIN32::Init`, enum `0x40000001`). Status: `handled` (ours
//! already did it), `wired` (this audit added it), `gap` (retail does it, ours does not yet) and
//! `skip` (the Legions client has no handler for the key: an admin-only feature, so nothing is
//! done, as retail does nothing).
//!
//! Retail handlers are cited by class and function. Global key presses reach the listeners as
//! `0xffff0003` messages carrying the action (`GLOBAL_KEY_PRESS`), and releases as `0xffff0004`.

use ac2_db::ResourceStore;

use super::maps::{load_action_map, load_key_map, KEYMAP_CONTROL, KEYMAP_MAIN, KEYMAP_SHIFT};

const KEYMAP_PREPROC: u32 = 0x4000_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Handled,
    Wired,
    Gap,
    Skip,
}

/// `(action, binding name, retail handler, our handler, status)`.
type Row = (u32, &'static str, &'static str, &'static str, Status);

mod camera;
mod gameplay;
mod movement;
mod preproc;

/// Every audit row, in the order of the sections (preproc, movement, camera, gameplay HUD).
fn audit() -> Vec<Row> {
    [preproc::ROWS, movement::ROWS, camera::ROWS, gameplay::ROWS].concat()
}

/// The three keys the input manager loads, with the preproc map.
fn bound_actions(store: &ResourceStore) -> Vec<u32> {
    let mut actions = Vec::new();
    for enum_value in [KEYMAP_PREPROC, KEYMAP_MAIN, KEYMAP_SHIFT, KEYMAP_CONTROL] {
        let map = load_key_map(store, enum_value)
            .unwrap()
            .unwrap_or_else(|| panic!("key map {enum_value:#x}"));
        actions.extend(map.mappings.iter().map(|m| m.action));
    }
    actions.sort_unstable();
    actions.dedup();
    actions
}

#[test]
fn every_bound_action_has_an_audit_row() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    assert!(load_action_map(&store).unwrap().is_some());
    let bound = bound_actions(&store);
    for action in &bound {
        assert!(
            audit().iter().any(|row| row.0 == *action),
            "no audit row for {action:#010x}"
        );
    }
    for row in audit() {
        assert!(
            bound.contains(&row.0),
            "audit row {} ({:#010x}) is not bound by any map",
            row.1,
            row.0
        );
    }
}

#[test]
fn every_gap_and_skip_row_says_why() {
    for (action, name, retail, ours, status) in audit() {
        assert!(
            !retail.is_empty() && !ours.is_empty(),
            "{name} {action:#010x}"
        );
        if matches!(status, Status::Gap | Status::Skip) {
            assert!(!ours.is_empty(), "{name}");
        }
    }
    let skips = audit().iter().filter(|r| r.4 == Status::Skip).count();
    let gaps = audit().iter().filter(|r| r.4 == Status::Gap).count();
    assert_eq!(
        gaps, 0,
        "every key action is handled or skipped (KeyStone is handled)"
    );
    assert_eq!(
        skips, 8,
        "admin-only keys: sound, benchmark, debug HUD, groove, exam tabs"
    );
}
