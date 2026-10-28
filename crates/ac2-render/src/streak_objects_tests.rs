//! An emitter's streaks through the scene's store: created by an FX, placed as the emitter
//! moves, activated and deactivated by its FX's start and stop flags, and destroyed with it.

use ac2_core::DataId;
use ac2_math::{Frame, Vec3};

use super::ObjectStreaks;
use crate::streak::StreakSpec;

const EMITTER: u64 = 9;
const MATERIAL: u32 = 0x4000_0032;

fn spec() -> StreakSpec {
    StreakSpec {
        start: Vec3::new(0.0, 1.0, 0.0),
        end: Vec3::new(0.0, -1.0, 0.0),
        lifespan: 0.0,
        trail_duration: 1.0,
        min_segment_time: 0.0,
        min_segment_distance: 0.1,
        color: [1.0, 1.0, 1.0, 1.0],
        texture_scale: 1.0,
        material: DataId::new(MATERIAL),
    }
}

fn at(x: f32) -> Frame {
    Frame {
        origin: Vec3::new(x, 0.0, 0.0),
        ..Frame::IDENTITY
    }
}

/// One emitter's placement for a frame: its cell, frame and scale.
fn place(store: &mut ObjectStreaks, x: f32) {
    store.place([(EMITTER, 5, at(x), 1.0)]);
}

fn drawn(store: &ObjectStreaks) -> usize {
    store
        .geometry()
        .iter()
        .map(|(_, geometry)| geometry.indices.len())
        .sum()
}

#[test]
fn a_streak_is_created_placed_moved_and_removed_with_its_emitter() {
    let mut store = ObjectStreaks::default();
    store.start(EMITTER, spec(), 0.0);
    assert_eq!(store.count(EMITTER), 1);

    // Activated where the emitter stands, then it moves along x: each frame cuts segments.
    store.set_active(EMITTER, true);
    let mut time = 0.0;
    for step in 0..6 {
        time += 0.05;
        place(&mut store, step as f32);
        store.think(time);
    }
    assert!(
        drawn(&store) > 0,
        "a moving, placed streak draws its ribbon"
    );

    // Deactivated: it stops cutting, and it stays until its emitter goes.
    store.set_active(EMITTER, false);
    store.remove_emitter(EMITTER);
    assert_eq!(store.count(EMITTER), 0);
    place(&mut store, 10.0);
    store.think(time + 0.05);
    assert!(
        store.geometry().is_empty(),
        "a removed emitter's streak draws nothing"
    );
}

#[test]
fn a_streak_draws_only_in_the_stamp_its_emitter_was_placed_in() {
    let mut store = ObjectStreaks::default();
    store.start(EMITTER, spec(), 0.0);
    store.set_active(EMITTER, true);
    let mut time = 0.0;
    for step in 0..4 {
        time += 0.05;
        place(&mut store, step as f32);
        store.think(time);
    }
    assert!(drawn(&store) > 0);
    // Next frame the emitter is not placed: its ribbon is not drawn.
    store.place(std::iter::empty());
    store.think(time + 0.05);
    assert!(store.geometry().is_empty());
}
