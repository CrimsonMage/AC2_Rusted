//! The streak ribbon math (`StreakEffect::Think`, `CreateGeometry`) on synthetic placements.

use ac2_core::DataId;
use ac2_math::{Frame, Vec3};

use super::{StreakEffect, StreakEffects, StreakSpec};
use crate::particles::ParticleGeometry;

fn spec(distance: f32, trail: f32, color: [f32; 4]) -> StreakSpec {
    StreakSpec {
        start: Vec3::new(0.0, 1.0, 0.0),
        end: Vec3::new(0.0, -1.0, 0.0),
        lifespan: 0.0,
        trail_duration: trail,
        min_segment_time: 0.0,
        min_segment_distance: distance,
        color,
        texture_scale: 1.0,
        material: DataId::new(0x4000_0032),
    }
}

/// An emitter standing at `x` on the x axis, in cell `cell`.
fn at(x: f32) -> Frame {
    Frame {
        origin: Vec3::new(x, 0.0, 0.0),
        ..Frame::IDENTITY
    }
}

/// A streak made and activated, as `CPartArray::ActivateStreakEffects` leaves it.
fn live(spec: StreakSpec, now: f64) -> StreakEffect {
    let mut streak = StreakEffect::new(spec, now);
    streak.activate();
    streak
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

/// Segments cut as the emitter moves: close samples merge into the newest segment, and one
/// farther than the minimum distance is fixed and a new one starts (`StreakEffect::Think`).
#[test]
fn a_streak_cuts_a_segment_when_the_emitter_has_moved_far_enough() {
    let mut streak = live(spec(0.5, 1.0, [1.0; 4]), 0.0);
    streak.set_position(1, 1, at(0.0), 1.0);
    streak.think(0.0);
    assert_eq!(
        streak.segments().len(),
        2,
        "a fixed segment and the working one"
    );
    streak.set_position(2, 1, at(0.25), 1.0);
    streak.think(0.1);
    streak.set_position(3, 1, at(0.5), 1.0);
    streak.think(0.2);
    assert_eq!(
        streak.segments().len(),
        2,
        "0.5 m is the minimum distance, so it still merges"
    );
    streak.set_position(4, 1, at(0.75), 1.0);
    streak.think(0.3);
    assert_eq!(streak.segments().len(), 3, "0.75 m cuts a new segment");
    let fixed = streak.segments()[1];
    assert!(
        close(fixed.distance, 0.75),
        "the ribbon's length: {fixed:?}"
    );
}

/// Segments fade to `(1 - age / trail)^2`, and go once they are older than trail plus 0.1 s.
#[test]
fn trailing_segments_fade_and_go_past_the_trail() {
    let mut streak = live(spec(0.5, 1.0, [1.0; 4]), 0.0);
    streak.set_position(1, 1, at(0.0), 1.0);
    streak.think(0.0);
    streak.think(0.5);
    let fading = streak.segments()[0];
    assert!(close(fading.opacity, 0.25), "(1 - 0.5)^2: {fading:?}");
    streak.think(1.05);
    assert_eq!(streak.segments()[0].opacity, 0.0, "past its trail, clear");
    streak.think(1.2);
    assert_eq!(
        streak.segments().len(),
        1,
        "the first segment is cut past trail + 0.1 s"
    );
}

/// A streak with a lifespan ends once its age passes it.
#[test]
fn a_streak_with_a_lifespan_ends_when_it_is_over() {
    let mut streak = live(
        StreakSpec {
            lifespan: 1.0,
            ..spec(0.5, 1.0, [1.0; 4])
        },
        10.0,
    );
    assert!(streak.is_active(), "a lifespan starts it");
    streak.set_position(1, 1, at(0.0), 1.0);
    streak.think(10.5);
    assert!(streak.is_active());
    streak.think(11.5);
    assert!(!streak.is_active(), "over at 1 s");
}

/// A streak keeps its base cell: moves between cells do not restart it, but entering a cell from
/// none does (`StreakEffect::SetPosition`).
#[test]
fn a_streak_restarts_only_when_it_enters_a_cell_from_none() {
    let mut streak = live(spec(0.5, 1.0, [1.0; 4]), 0.0);
    streak.set_position(1, 1, at(0.0), 1.0);
    streak.think(0.0);
    streak.think(0.1);
    let before = streak.segments().len();
    streak.set_position(2, 2, at(5.0), 1.0);
    assert_eq!(
        streak.segments().len(),
        before,
        "a base cell keeps the trail"
    );

    let mut from_none = live(spec(0.5, 1.0, [1.0; 4]), 0.0);
    from_none.set_position(1, 0, at(0.0), 1.0);
    from_none.think(0.0);
    assert_eq!(from_none.segments().len(), 2);
    from_none.set_position(2, 3, at(0.0), 1.0);
    assert!(
        from_none.segments().is_empty(),
        "entering a cell from none restarts it"
    );
}

/// The ribbon is a strip of two vertices per segment, the line1 point then line0, with the
/// texture's v 0 then 1 and u the distance times the texture scale; the list keeps the strip's
/// triangles facing one way (`CreateGeometry`).
#[test]
fn the_ribbon_is_a_strip_with_its_texture_along_its_length() {
    let mut streak = live(
        StreakSpec {
            texture_scale: 2.0,
            ..spec(0.5, 1.0, [1.0; 4])
        },
        0.0,
    );
    streak.set_position(7, 1, at(0.0), 1.0);
    streak.think(0.0);
    streak.set_position(8, 1, at(0.25), 1.0);
    streak.think(0.1);
    streak.set_position(9, 1, at(0.75), 1.0);
    streak.think(0.2);
    assert_eq!(streak.segments().len(), 3);
    let mut geometry = ParticleGeometry::default();
    assert!(streak.fill(9, &mut geometry), "placed this frame");
    assert_eq!(geometry.vertices.len(), 6, "two per segment");
    assert_eq!(geometry.indices.len(), 12, "four triangles of the strip");
    assert_eq!(&geometry.indices[..6], &[0, 1, 2, 2, 1, 3]);
    assert_eq!(&geometry.indices[6..], &[2, 3, 4, 4, 3, 5]);
    let first = geometry.vertices[0];
    assert_eq!(
        first.position,
        [0.0, -1.0, 0.0],
        "line1 of the first segment"
    );
    assert_eq!(first.uvs[0], [0.0, 0.0]);
    assert_eq!(geometry.vertices[1].uvs[0], [0.0, 1.0], "line0 at v = 1");
    let last = geometry.vertices[4];
    assert!(close(last.uvs[0][0], 1.5), "u = 0.75 m x 2: {:?}", last.uvs);
    assert_eq!(last.normal, [0.0, 0.0, 1.0]);
}

/// The vertex colour is the streak colour with alpha `opacity * a`, each channel truncated to a
/// byte (the D3D packing), and a streak not placed this frame draws nothing.
#[test]
fn colour_is_truncated_to_bytes_and_only_this_frames_streaks_draw() {
    let mut streak = live(spec(0.5, 1.0, [1.0, 0.5, 0.0, 0.5]), 0.0);
    streak.set_position(4, 1, at(0.0), 1.0);
    streak.think(0.0);
    streak.set_position(5, 1, at(0.1), 1.0);
    streak.think(0.1);
    let mut geometry = ParticleGeometry::default();
    assert!(!streak.fill(4, &mut geometry), "stamp 4 is last frame");
    assert!(streak.fill(5, &mut geometry));
    // Vertex 2 is the working segment's line1: opaque, so alpha is the streak's 0.5 alone.
    let colour = geometry.vertices[2].color;
    assert_eq!(colour[0], 1.0);
    assert_eq!(colour[1], 127.0 / 255.0, "0.5 * 255 truncates to 127");
    assert_eq!(colour[2], 0.0);
    assert_eq!(colour[3], 127.0 / 255.0);
}

/// The manager thinks every streak, and draws each material's ribbons as one batch.
#[test]
fn the_manager_batches_ribbons_by_material_and_drops_destroyed_streaks() {
    let mut effects = StreakEffects::default();
    let a = effects.create(spec(0.5, 1.0, [1.0; 4]), 0.0);
    let b = effects.create(
        StreakSpec {
            material: DataId::new(0x4000_0099),
            ..spec(0.5, 1.0, [1.0; 4])
        },
        0.0,
    );
    let c = effects.create(spec(0.5, 1.0, [1.0; 4]), 0.0);
    for id in [a, b, c] {
        let streak = effects.get_mut(id).expect("made");
        streak.activate();
        streak.set_position(3, 1, at(0.0), 1.0);
    }
    effects.think(0.0);
    let batches = effects.geometry(3);
    assert_eq!(batches.len(), 2, "two materials: {batches:?}");
    assert_eq!(batches[0].0, DataId::new(0x4000_0032));
    effects.destroy(a);
    effects.destroy(c);
    assert_eq!(effects.len(), 1);
    assert_eq!(effects.geometry(3).len(), 1);
}
