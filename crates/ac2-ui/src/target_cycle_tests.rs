//! `gmSelectionUI::Select`'s target cycling on synthetic candidates.

use crate::target_cycle::{
    cycle_rule, farther, is_select_key, select_target, type_vs_key, Candidate, Step,
};

const PLAYER: u64 = 100;
/// A creature: the type bits `0x4F` (keys 0x19-0x1B, 0x22-0x24, 0x5C-0x5E).
const CREATURE: u32 = 0x4F;

fn at(id: u64, x: f32) -> Candidate {
    Candidate {
        id,
        kind: CREATURE,
        selectable: true,
        owner: None,
        fellow: false,
        owner_fellow: false,
        can_attack: true,
        offset: [x, 0.0, 0.0],
    }
}

fn world() -> Vec<Candidate> {
    // Planar distances 5, 10, 20 for ids 1, 2, 3, and the player.
    vec![at(1, 5.0), at(2, 10.0), at(3, 20.0)]
}

const RANGE_SQ: f32 = 400.0 * 400.0;

#[test]
fn cycling_keys_step_next_previous_and_closest_with_their_groups() {
    assert_eq!(cycle_rule(0x4100_001A).unwrap().step, Step::Next);
    assert_eq!(cycle_rule(0x4100_0019).unwrap().step, Step::Previous);
    assert_eq!(cycle_rule(0x4100_001B).unwrap().step, Step::Closest);
    // The foe group, the fellow group and the pets.
    let foe = cycle_rule(0x4100_0024).unwrap();
    assert!(foe.foes && !foe.fellows && !foe.pets);
    let fellow = cycle_rule(0x4100_0025).unwrap();
    assert!(fellow.fellows && !fellow.foes);
    assert!(cycle_rule(0x4100_005C).unwrap().pets);
    // Keys `Select` handles otherwise have no rule.
    assert_eq!(cycle_rule(0x4100_0028), None);
    assert_eq!(cycle_rule(0x4100_002A), None);
}

#[test]
fn only_the_target_keys_reach_select() {
    for key in [
        0x4100_0018,
        0x4100_0028,
        0x4100_002A,
        0x4000_001E,
        0x4100_005C,
        0x4100_005E,
    ] {
        assert!(is_select_key(key), "{key:#x}");
    }
    for key in [
        0x4100_0017,
        0x4100_0029,
        0x4100_002B,
        0x4100_005B,
        0x4100_005F,
    ] {
        assert!(!is_select_key(key), "{key:#x}");
    }
}

#[test]
fn a_key_takes_the_types_it_names() {
    assert!(type_vs_key(CREATURE, 0x4100_001A));
    assert!(!type_vs_key(CREATURE, 0x4100_001F));
    // The 0x81 pattern takes the 0x1C-0x1E keys only.
    assert!(type_vs_key(0x81, 0x4100_001D));
    assert!(!type_vs_key(0x81, 0x4100_001A));
    assert!(!type_vs_key(0, 0x4100_001A));
}

#[test]
fn farther_orders_by_distance_then_by_id() {
    assert!(farther((5.0, 1), (4.0, 9)));
    assert!(!farther((4.0, 9), (5.0, 1)));
    // Equal distances: the lower id is the nearer.
    assert!(farther((5.0, 7), (5.0, 3)));
    assert!(!farther((5.0, 3), (5.0, 7)));
}

#[test]
fn next_and_previous_step_through_the_objects_by_distance_and_wrap_round() {
    let objects = world();
    let next = 0x4100_001A;
    assert_eq!(
        select_target(next, PLAYER, RANGE_SQ, Some(1), &objects),
        Some(2)
    );
    assert_eq!(
        select_target(next, PLAYER, RANGE_SQ, Some(2), &objects),
        Some(3)
    );
    // Past the farthest: the nearest again.
    assert_eq!(
        select_target(next, PLAYER, RANGE_SQ, Some(3), &objects),
        Some(1)
    );
    let prev = 0x4100_0019;
    assert_eq!(
        select_target(prev, PLAYER, RANGE_SQ, Some(3), &objects),
        Some(2)
    );
    assert_eq!(
        select_target(prev, PLAYER, RANGE_SQ, Some(2), &objects),
        Some(1)
    );
    // Before the nearest: the farthest again.
    assert_eq!(
        select_target(prev, PLAYER, RANGE_SQ, Some(1), &objects),
        Some(3)
    );
}

#[test]
fn closest_takes_the_nearest_whatever_is_selected() {
    let objects = world();
    let closest = 0x4100_001B;
    assert_eq!(
        select_target(closest, PLAYER, RANGE_SQ, None, &objects),
        Some(1)
    );
    assert_eq!(
        select_target(closest, PLAYER, RANGE_SQ, Some(3), &objects),
        Some(1)
    );
}

#[test]
fn the_height_counts_at_one_and_a_fifth_times() {
    // Object 1 is 5 m away planar but 10 m high: 5 + 12 = 17, farther than object 2 at 10.
    let mut objects = world();
    objects[0].offset = [5.0, 0.0, 10.0];
    assert_eq!(
        select_target(0x4100_001B, PLAYER, RANGE_SQ, None, &objects),
        Some(2)
    );
}

#[test]
fn the_player_unselectable_untyped_and_out_of_range_are_never_picked() {
    let mut objects = world();
    objects.push(Candidate {
        id: PLAYER,
        ..at(PLAYER, 1.0)
    });
    objects[0].selectable = false;
    objects[1].kind = 0;
    objects.push(at(4, 500.0));
    // Only object 3 is left, the farthest in range.
    assert_eq!(
        select_target(0x4100_001B, PLAYER, RANGE_SQ, None, &objects),
        Some(3)
    );
    assert_eq!(
        select_target(0x4100_001B, PLAYER, 1.0, None, &objects),
        None
    );
}

#[test]
fn foes_fellows_and_pets_keep_to_their_groups() {
    let mut objects = world();
    objects[0].can_attack = false;
    // A fellow takes the fellow keys by its type (`0x1F` bits).
    objects[1].fellow = true;
    objects[1].kind = 0x1F;
    objects[2].owner = Some(55);
    // The closest foe: 1 cannot be attacked, 2 is a fellow, 3 is the foe.
    assert_eq!(
        select_target(0x4100_0024, PLAYER, RANGE_SQ, None, &objects),
        Some(3)
    );
    // The closest fellow.
    assert_eq!(
        select_target(0x4100_0027, PLAYER, RANGE_SQ, None, &objects),
        Some(2)
    );
    // Pets of someone else: object 3 belongs to 55.
    assert_eq!(
        select_target(0x4100_005E, PLAYER, RANGE_SQ, None, &objects),
        Some(3)
    );
    // Its owner is the player: not a pet of someone else.
    objects[2].owner = Some(PLAYER);
    assert_eq!(
        select_target(0x4100_005E, PLAYER, RANGE_SQ, None, &objects),
        None
    );
}
