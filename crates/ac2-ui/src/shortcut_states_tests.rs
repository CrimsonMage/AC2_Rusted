//! `gmShortcutUI`'s special attack states and skill reset timers on the real toolbar layout
//! (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::layout::AttributeValue;
use crate::shortcut::{Shortcut, ShortcutOptions, ShortcutSource, SHORTCUT_SKILL};
use crate::token::tokenize;
use crate::{build_gameplay_hud, GameplayHud, UiTree};

const MANEUVER: u32 = 0x77;
const OTHER_SKILL: u32 = 0x03;

/// Slot 0 is the maneuver, slot 1 another skill; both trained.
struct Bar(Vec<Shortcut>);

impl ShortcutSource for Bar {
    fn shortcuts(&self) -> Vec<Shortcut> {
        self.0.clone()
    }
    fn icon(&self, _: usize, _: &Shortcut) -> u32 {
        0
    }
    fn options(&self) -> ShortcutOptions {
        ShortcutOptions {
            which_set: 1,
            show_numbers: false,
            insta_scroll: false,
            rows: 1,
            locked: false,
        }
    }
    fn skill_trained(&self, _: u32) -> bool {
        true
    }
    fn has_item(&self, _: u64) -> bool {
        true
    }
}

fn skill(skill: u32) -> Shortcut {
    Shortcut {
        kind: SHORTCUT_SKILL,
        skill,
        ..Default::default()
    }
}

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

/// The bar's slot element at `index`.
fn slot(tree: &UiTree, list: usize, index: usize) -> usize {
    tree.list_items(list)[index]
}

fn float(tree: &UiTree, index: usize, name: &str) -> Option<f32> {
    match tree.get(index)?.attribute(tokenize(name)) {
        Some(AttributeValue::Float(v)) => Some(*v),
        _ => None,
    }
}

#[test]
fn a_special_attack_marks_its_maneuvers_slots_pending_active_and_back() {
    let Some(mut hud) = hud() else {
        return;
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    let source = Bar(vec![skill(MANEUVER), skill(OTHER_SKILL)]);
    bar.reload(&mut hud.tree, &source);
    let list = bar.elements().1.expect("the list");
    let (first, other) = (slot(&hud.tree, list, 0), slot(&hud.tree, list, 1));
    let state = |tree: &UiTree, s: usize| tree.get(s).unwrap().state();

    bar.request_special_attack(&mut hud.tree, 5, MANEUVER);
    assert_eq!(state(&hud.tree, first), tokenize("pending"));
    assert!(!hud
        .tree
        .get(first)
        .unwrap()
        .attribute(tokenize("dragable"))
        .is_some_and(|v| matches!(v, AttributeValue::Bool(true))));
    // Asked again: no change.
    bar.request_special_attack(&mut hud.tree, 5, MANEUVER);
    assert_eq!(bar.specials.len(), 1);

    bar.start_special_attack(&mut hud.tree, 5);
    assert_eq!(state(&hud.tree, first), tokenize("active"));

    bar.end_special_attack(&mut hud.tree, 5);
    assert_eq!(state(&hud.tree, first), tokenize("stateless"));
    assert!(bar.specials.is_empty());
    // The other skill's slot never moved.
    assert_ne!(state(&hud.tree, other), tokenize("pending"));
}

#[test]
fn a_toggled_maneuver_stays_toggled_through_its_special_attack() {
    let Some(mut hud) = hud() else {
        return;
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    bar.reload(&mut hud.tree, &Bar(vec![skill(MANEUVER)]));
    let list = bar.elements().1.expect("the list");
    let first = slot(&hud.tree, list, 0);
    bar.toggle_special_attack(&mut hud.tree, MANEUVER, true);
    assert_eq!(hud.tree.get(first).unwrap().state(), tokenize("toggled"));
    bar.request_special_attack(&mut hud.tree, 9, MANEUVER);
    bar.end_special_attack(&mut hud.tree, 9);
    assert_eq!(hud.tree.get(first).unwrap().state(), tokenize("toggled"));
    bar.toggle_special_attack(&mut hud.tree, MANEUVER, false);
    assert_eq!(hud.tree.get(first).unwrap().state(), tokenize("stateless"));
}

#[test]
fn a_reset_timer_fills_its_slot_meter_until_the_skill_is_ready() {
    let Some(mut hud) = hud() else {
        return;
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    let source = Bar(vec![skill(MANEUVER), skill(OTHER_SKILL)]);
    bar.reload(&mut hud.tree, &source);
    let list = bar.elements().1.expect("the list");
    let meter = hud
        .tree
        .child_recursive(slot(&hud.tree, list, 1), tokenize("TimeLeftMeter"))
        .expect("the slot's meter");

    // 4 of 10 seconds left at the start: the meter is 0.6 full.
    bar.update_time_left(&mut hud.tree, 0.0);
    bar.reset_timer(&mut hud.tree, 1, OTHER_SKILL, 10.0, 4.0);
    bar.update_time_left(&mut hud.tree, 0.0);
    assert!((float(&hud.tree, meter, "position").unwrap() - 0.6).abs() < 1e-6);
    bar.update_time_left(&mut hud.tree, 2.0);
    assert!((float(&hud.tree, meter, "position").unwrap() - 0.8).abs() < 1e-6);
    // The time is up: the entry is gone.
    bar.update_time_left(&mut hud.tree, 4.0);
    assert!(bar.timers.is_empty());
}

#[test]
fn a_timer_goes_when_its_slot_no_longer_holds_its_skill() {
    let Some(mut hud) = hud() else {
        return;
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    bar.reload(
        &mut hud.tree,
        &Bar(vec![skill(MANEUVER), skill(OTHER_SKILL)]),
    );
    bar.update_time_left(&mut hud.tree, 0.0);
    bar.reset_timer(&mut hud.tree, 1, OTHER_SKILL, 10.0, 4.0);
    assert_eq!(bar.timers.len(), 1);
    // Slot 1 now holds another skill: the entry goes on the next update.
    bar.reload(&mut hud.tree, &Bar(vec![skill(MANEUVER), skill(MANEUVER)]));
    bar.update_time_left(&mut hud.tree, 1.0);
    assert!(bar.timers.is_empty());
}

#[test]
fn the_bar_shows_and_hides_its_field() {
    let Some(mut hud) = hud() else {
        return;
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    let field = bar.elements().0.expect("the field");
    bar.show(&mut hud.tree, false);
    assert!(!hud.tree.get(field).unwrap().visible());
    bar.show(&mut hud.tree, true);
    assert!(hud.tree.get(field).unwrap().visible());
}

/// `gmShortcutUI::ListenToMessage`'s global loop after the grip resized the field: 104 pixels
/// tall is `(104 - 12) / 32 + 0.5` = 3 rows, so the field snaps to 108 with its bottom edge
/// kept, the list takes 3 rows and 120 slots, and the rows are sent on once.
#[test]
fn a_resized_bar_snaps_to_whole_rows_and_sends_its_height() {
    let Some(mut hud) = hud() else {
        return;
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    let source = Bar(vec![skill(MANEUVER)]);
    bar.reload(&mut hud.tree, &source);
    let (field, list) = bar.elements();
    let (field, list) = (field.unwrap(), list.unwrap());
    bar.resize_loop(&mut hud.tree, &source);
    let _ = bar.take_requests();
    let start = hud.tree.screen_box(field);
    assert_eq!(start.height(), 44, "one row");

    // The grip's drag: 60 taller, the bottom edge kept.
    let tree = &mut hud.tree;
    let (x, y) = tree.get(field).unwrap().object.unwrap();
    tree.resize_to(field, start.width(), 104);
    tree.move_to(field, x, y - 60);
    bar.resize_loop(tree, &source);
    let snapped = tree.screen_box(field);
    assert_eq!((snapped.height(), snapped.y1), (108, start.y1));
    assert_eq!(
        tree.get(list).unwrap().attribute(tokenize("max_rows")),
        Some(&AttributeValue::Int(3))
    );
    assert_eq!(tree.list_items(list).len(), 120);
    assert_eq!(bar.take_requests(), vec![crate::ShortcutRequest::Height(3)]);
    bar.resize_loop(tree, &source);
    assert!(bar.take_requests().is_empty(), "nothing moved since");
}
