//! A decal node's spawn (`FXManager::DoDecalFX`): where the decal lands, how it turns, and the
//! cells retail makes none in.

use ac2_assets::Ac2DecalFx;
use ac2_core::Random;
use ac2_math::{Vec3, Waveform};

use super::{decal_spec, FxDecals};

const BLOCK: f32 = 192.0;

fn decal(rotation: f32, radius: f32) -> Ac2DecalFx {
    Ac2DecalFx {
        color: [1.0, 1.0, 1.0, 1.0],
        material: 0x3100_0001,
        origin: Vec3::new(0.5, 0.0, 0.0),
        lifespan: 30.0,
        size: Waveform::constant(2.0),
        rotation: Waveform::constant(rotation),
        position_radius: Waveform::constant(radius),
    }
}

#[test]
fn a_landblock_decal_lands_at_the_emitter_plus_the_node_origin() {
    let mut random = Random::default();
    // Block (1, 2) is the cell 0x0102_0001: its origin is (192, 384) in metres.
    let spec = decal_spec(
        &decal(0.0, 0.0),
        0x0102_0001,
        Vec3::new(10.0, 20.0, 3.0),
        Vec3::Y,
        BLOCK,
        0.0,
        &mut random,
    )
    .expect("a landblock decal");
    assert_eq!(spec.origin, Vec3::new(192.0 + 10.5, 384.0 + 20.0, 3.0));
    assert_eq!(spec.size, 2.0);
    assert_eq!(spec.lifespan, 30.0);
    assert!(!spec.draw_once);
}

#[test]
fn a_dungeon_or_cellless_decal_is_not_made() {
    let mut random = Random::default();
    let origin = Vec3::new(1.0, 1.0, 1.0);
    assert!(decal_spec(
        &decal(0.0, 0.0),
        0,
        origin,
        Vec3::Y,
        BLOCK,
        0.0,
        &mut random
    )
    .is_none());
    // An environment cell (id 0x0102_0100 and up).
    assert!(decal_spec(
        &decal(0.0, 0.0),
        0x0102_0100,
        origin,
        Vec3::Y,
        BLOCK,
        0.0,
        &mut random
    )
    .is_none());
}

#[test]
fn rotation_turns_the_decal_about_the_vertical_axis() {
    let mut random = Random::default();
    let spec = decal_spec(
        &decal(90.0, 0.0),
        0x0102_0001,
        Vec3::ZERO,
        Vec3::Y,
        BLOCK,
        0.0,
        &mut random,
    )
    .unwrap();
    // The emitter's forward (+y) turned a quarter about z points along -x.
    assert!((spec.direction.x + 1.0).abs() < 1e-6 && spec.direction.y.abs() < 1e-6);
    assert_eq!(spec.direction.z, 0.0);
}

/// `DoDecalFX` turns the emitter's forward vector (`fl2gv[3..6]`), not a fixed axis: a decal
/// with no rotation points where the emitter faces, and the result is a unit vector.
#[test]
fn the_decal_starts_from_the_emitters_heading() {
    let mut random = Random::default();
    let forward = Vec3::new(3.0, 0.0, 4.0) / 5.0;
    let spec = decal_spec(
        &decal(0.0, 0.0),
        0x0102_0001,
        Vec3::ZERO,
        forward,
        BLOCK,
        0.0,
        &mut random,
    )
    .unwrap();
    assert!((spec.direction - forward).length() < 1e-6);
    let turned = decal_spec(
        &decal(180.0, 0.0),
        0x0102_0001,
        Vec3::ZERO,
        forward,
        BLOCK,
        0.0,
        &mut random,
    )
    .unwrap();
    assert!((turned.direction - Vec3::new(-0.6, 0.0, 0.8)).length() < 1e-6);
}

#[test]
fn a_radius_scatters_the_origin_within_it() {
    let mut random = Random::default();
    for _ in 0..64 {
        let spec = decal_spec(
            &decal(0.0, 4.0),
            0x0102_0001,
            Vec3::ZERO,
            Vec3::Y,
            BLOCK,
            0.0,
            &mut random,
        )
        .unwrap();
        let dx = spec.origin.x - (192.0 + 0.5);
        let dy = spec.origin.y - 384.0;
        assert!(dx.hypot(dy) <= 4.0 + 1e-4, "scattered {dx} {dy}");
        assert_eq!(spec.origin.z, 0.0);
    }
}

#[test]
fn decal_effects_off_makes_no_decal() {
    let mut fx = FxDecals::default();
    assert!(fx.enabled());
    fx.set_enabled(false);
    assert!(!fx.enabled());
    assert!(fx.take_calls().is_empty());
}
