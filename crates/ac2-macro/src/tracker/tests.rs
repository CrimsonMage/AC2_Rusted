use std::collections::BTreeMap;
use std::time::Duration;

use super::*;

fn object(id: u64, name: &str) -> ObservedObject<'_> {
    ObservedObject {
        object: id,
        name: Some(name),
        containment_known: true,
        container: None,
        cell: Some(0x7F7F_0025),
    }
}

fn patterns(lines: &[&str]) -> PatternRead {
    let mut text = format!("{}\r\n", lines.len());
    for l in lines {
        text.push_str(l);
        text.push_str("\r\n");
    }
    PatternRead::Bytes(text.into_bytes().into())
}

#[test]
fn a_load_announces_present_matches_and_later_only_newcomers() {
    let mut t = Tracker::default();
    let start = [object(1, "Hoary Mattekar")];
    let update = t.load_initial(patterns(&["mattekar"]), PatternRead::Missing, 1, start);
    // A load scans what is around (Virindi Sense reports present matches on load).
    assert!(matches!(
        update.effects.as_slice(),
        [Effect::Detected { object: 1, .. }]
    ));
    let update = t.reconcile(
        2,
        [object(1, "Hoary Mattekar"), object(2, "Hoary Mattekar")],
    );
    assert_eq!(update.effects.len(), 1);
    let Effect::Detected {
        object: id,
        name,
        at,
    } = &update.effects[0]
    else {
        panic!("{update:?}");
    };
    assert_eq!((*id, name.as_ref()), (2, "Hoary Mattekar"));
    // Block 0x7F7F's cell 0x25 is land cell (1020, 1020): the map's middle.
    assert_eq!(at.as_deref(), Some("0.0N, 0.0E"));
    // Nothing new: nothing announced.
    assert!(t
        .reconcile(
            3,
            [object(1, "Hoary Mattekar"), object(2, "Hoary Mattekar")]
        )
        .effects
        .is_empty());
}

#[test]
fn contained_objects_and_gone_objects_leave_the_list() {
    let mut t = Tracker::default();
    t.load_initial(patterns(&["^Gem$"]), PatternRead::Missing, 1, []);
    let mut packed = object(5, "Gem");
    packed.container = Some(9);
    assert!(t.reconcile(2, [packed]).effects.is_empty());
    t.reconcile(3, [object(6, "Gem")]);
    assert_eq!(t.view().matches.len(), 1);
    t.reconcile(4, []);
    assert!(t.view().matches.is_empty());
}

#[test]
fn legacy_lists_import_unchanged_and_exact_patterns_anchor() {
    let mut t = Tracker::default();
    let update = t.load_initial(PatternRead::Missing, patterns(&["drudge"]), 1, []);
    assert!(update
        .effects
        .iter()
        .any(|e| matches!(e, Effect::SavePatterns(_))));
    let update = t.add_pattern(
        "Gold Ore",
        true,
        [object(7, "Gold Ore"), object(8, "Gold Ore Vein")],
    );
    let detected: Vec<u64> = update
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Detected { object, .. } => Some(*object),
            _ => None,
        })
        .collect();
    assert_eq!(detected, [7]);
    assert_eq!(t.view().patterns[1].expression.as_ref(), "^Gold Ore$");
    assert!(t.view().patterns[1].exact);
}

#[test]
fn picking_a_match_points_the_arrow_and_another_selection_releases_it() {
    let mut t = Tracker::default();
    t.load_initial(
        patterns(&["rabbit"]),
        PatternRead::Missing,
        1,
        [object(3, "White Rabbit")],
    );
    assert_eq!(t.select_match(3), Some(3));
    assert_eq!(t.pointer_target(), Some(3));
    assert!(!t.observe_selection(Some(3)));
    assert!(t.observe_selection(Some(4)));
    assert_eq!(t.pointer_target(), None);
    assert_eq!(t.select_match(99), None);
}

struct Positions(BTreeMap<u64, TrackerPosition>);

impl PositionQuery for Positions {
    fn position(&self, object: u64) -> Option<TrackerPosition> {
        self.0.get(&object).copied()
    }
}

#[test]
fn distances_are_sampled_and_only_changes_present() {
    let mut t = Tracker::default();
    t.load_initial(
        patterns(&["rabbit"]),
        PatternRead::Missing,
        1,
        [object(3, "White Rabbit")],
    );
    let player = TrackerPosition {
        cell: 0x7F7F_0025,
        origin: [10.0, 10.0, 0.0],
    };
    // One block east (160 m) and 30 m north.
    let positions = Positions(BTreeMap::from([(
        3,
        TrackerPosition {
            cell: 0x807F_0025,
            origin: [10.0, 40.0, 0.0],
        },
    )]));
    let first = t
        .project(Duration::ZERO, Some(player), &positions)
        .expect("the first presentation");
    let row = &first.matches[0];
    assert_eq!(
        row.distance_decimetres,
        (160.0f64.hypot(30.0) * 10.0).round() as u32
    );
    assert!(row.coordinates.is_some());
    // Within the sample interval and nothing changed: nothing new.
    assert!(t
        .project(Duration::from_millis(100), Some(player), &positions)
        .is_none());
    assert!(t
        .project(Duration::from_millis(300), Some(player), &positions)
        .is_none());
}
