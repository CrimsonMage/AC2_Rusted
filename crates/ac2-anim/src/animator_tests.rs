use std::sync::Arc;

use ac2_math::{Quat, Vec3};

use super::*;

fn clip(period: f32, x: f32) -> Arc<Ac2Animation> {
    Arc::new(Ac2Animation {
        period,
        bones: Arc::from([Ac2BoneTrack {
            scale: vec![(0.0, Vec3::ONE)],
            offset: vec![
                (0.0, Vec3::new(x, 0.0, 0.0)),
                (period, Vec3::new(x + 1.0, 0.0, 0.0)),
            ],
            rotation: vec![(0.0, Quat::IDENTITY)],
        }]),
    })
}

fn slider(id: u32, ticks: &[f32]) -> Slider {
    Slider {
        id,
        ticks: ticks.to_vec(),
    }
}

#[test]
fn lattice_weights_are_multilinear_with_the_first_slider_fastest() {
    let axes = [slider(1, &[-1.0, 0.0, 1.0]), slider(2, &[0.0, 2.0])];
    let weights = lattice_weights(&axes, &[(1, 0.5), (2, 0.5)]);
    let weight = |index| {
        weights
            .iter()
            .find(|(i, _)| *i == index)
            .map_or(0.0, |(_, w)| *w)
    };
    // Slider 1 between ticks 1 and 2 (half), slider 2 a quarter of the way to tick 1.
    assert!((weight(1) - 0.375).abs() < 1e-6);
    assert!((weight(2) - 0.375).abs() < 1e-6);
    assert!((weight(4) - 0.125).abs() < 1e-6);
    assert!((weight(5) - 0.125).abs() < 1e-6);
    assert_eq!(weights.len(), 4);
}

#[test]
fn slider_values_clamp_to_the_tick_range_and_missing_sliders_read_zero() {
    let axes = [slider(1, &[-7.0, 0.0, 3.0])];
    assert_eq!(lattice_weights(&axes, &[(1, 10.0)]), vec![(2, 1.0)]);
    assert_eq!(lattice_weights(&axes, &[]), vec![(1, 1.0)]);
    assert_eq!(lattice_weights(&axes, &[(1, -9.0)]), vec![(0, 1.0)]);
}

#[test]
fn blends_share_one_phase_and_average_their_period_and_pose() {
    let (slow, fast) = (clip(2.0, 0.0), clip(1.0, 10.0));
    let animator = Ac2MotionAnimator {
        root: Node::Cube {
            sliders: Arc::from([slider(7, &[0.0, 1.0])]),
            children: Arc::from([Node::Clip(slow), Node::Clip(fast)]),
        },
    };
    let blend = animator.blend(&[(7, 0.25)]);
    assert!((blend.period() - 1.75).abs() < 1e-6);
    // Half way through the phase both clips are half way through their offset ramps.
    let pose = blend.sample(0.5, &[Ac2Transform::IDENTITY]);
    assert!((pose[0].offset.x - (0.75 * 0.5 + 0.25 * 10.5)).abs() < 1e-5);
    let still = blend.still_frame(1.5);
    assert_eq!(still.period, 0.0);
    assert!(
        (still.sample(123.0, &[Ac2Transform::IDENTITY])[0].offset.x - pose[0].offset.x).abs()
            < 1e-5
    );
}

#[test]
fn opposite_hemisphere_rotations_blend_without_cancelling() {
    let turn = Quat::from_rotation_z(0.4);
    let track = |rotation| Ac2BoneTrack {
        scale: Vec::new(),
        offset: Vec::new(),
        rotation: vec![(0.0, rotation)],
    };
    let mut accumulator = Accumulator::default();
    accumulator.add(&track(turn), 0.0, 0.5);
    accumulator.add(&track(-turn), 0.0, 0.5);
    let rotation = accumulator.finish().rotation[0].1;
    assert!(rotation.dot(turn).abs() > 0.999);
}
