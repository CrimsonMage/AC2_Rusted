//! An FX's streak nodes on their emitter: made once per running FX, started and stopped by the
//! node flags, destroyed when the emitter leaves. The scene test needs the DATs (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_assets::{
    Ac2FxObject, Ac2ParticleAnchor, Ac2ParticleAttachment, Ac2StreakFx, Ac2VisualFxParticles,
};
use ac2_core::DataId;
use ac2_math::{Frame, Quat, Vec3};
use ac2_render::scene::{ObjectPlacement, WorldScene};
use ac2_render::Camera;
use ac2_world::Ac2WorldSource;

use super::ObjectStreakCall;
use crate::fx_objects::FxObjects;

const FX: u32 = 0x4000_0100;
const EMITTER: u64 = 3;
const MATERIAL: u32 = 0x4000_0032;

fn streak() -> Ac2StreakFx {
    Ac2StreakFx {
        color: [1.0, 0.5, 0.25, 1.0],
        start: Vec3::new(0.0, 0.0, 0.5),
        end: Vec3::new(0.0, 0.0, -0.5),
        material: MATERIAL,
        lifespan: 0.0,
        trail_duration: 1.0,
        min_segment_time: 0.0,
        min_segment_distance: 0.05,
        texture_scale: 1.0,
    }
}

/// A node of `fx` that carries the given streak facts and no particle system.
fn node(streak: Option<Ac2StreakFx>, start: bool, stop: bool) -> Ac2ParticleAttachment {
    Ac2ParticleAttachment {
        particle_descs: Arc::from([]),
        placement: ac2_anim::Ac2Transform::IDENTITY,
        anchor: Ac2ParticleAnchor::Object { offset: Vec3::ZERO },
        remove_rotation: false,
        water_plane: false,
        fire_and_forget: false,
        light: None,
        looping: Arc::from([]),
        object: Ac2FxObject {
            streak,
            start_streak: start,
            stop_streak: stop,
            ..Ac2FxObject::default()
        },
    }
}

fn table(nodes: Vec<Ac2ParticleAttachment>) -> Ac2VisualFxParticles {
    Ac2VisualFxParticles::from_entries([(FX, nodes)])
}

/// `DoStreakFX` makes the node's streak on the emitter, and its start flag activates it (in node
/// order: the streak first, then the start and stop flags).
#[test]
fn an_fx_with_a_streak_node_makes_a_streak_and_starts_it() {
    let table = table(vec![node(Some(streak()), true, false)]);
    let mut fx = FxObjects::default();
    fx.start_streaks(EMITTER, FX, &table);
    let calls = std::mem::take(&mut fx.streak_calls);
    assert_eq!(calls.len(), 2, "one streak and one start");
    let ObjectStreakCall::Create { emitter, spec } = calls[0] else {
        panic!("the first call makes the streak: {calls:?}");
    };
    assert_eq!(emitter, EMITTER);
    assert_eq!(spec.material, DataId::new(MATERIAL));
    assert_eq!(spec.start, Vec3::new(0.0, 0.0, 0.5));
    assert_eq!(spec.trail_duration, 1.0);
    assert_eq!(
        calls[1],
        ObjectStreakCall::Activate {
            emitter: EMITTER,
            on: true
        }
    );
}

/// `DoTargettedFX` starts nothing for an FX that is already running; a stop of the FX lets it
/// start again. `StreakEffects` off makes no streak but a stop still applies.
#[test]
fn a_running_fx_starts_nothing_and_streaks_off_makes_none() {
    let table = table(vec![node(Some(streak()), false, true)]);
    let mut fx = FxObjects::default();
    fx.start_streaks(EMITTER, FX, &table);
    assert_eq!(fx.streak_calls.len(), 2, "make, then stop");
    fx.streak_calls.clear();
    fx.start_streaks(EMITTER, FX, &table);
    assert!(
        fx.streak_calls.is_empty(),
        "the running FX does not restart"
    );
    fx.stop_streaks(EMITTER, FX);
    fx.start_streaks(EMITTER, FX, &table);
    assert_eq!(fx.streak_calls.len(), 2, "a stopped FX starts again");

    let mut off = FxObjects::default();
    off.streaks_off = true;
    off.start_streaks(EMITTER, FX, &table);
    assert_eq!(
        off.streak_calls,
        vec![ObjectStreakCall::Activate {
            emitter: EMITTER,
            on: false
        }],
        "no streak is made while StreakEffects is off; the stop still applies"
    );
}

/// The emitter leaving the world destroys its streaks once.
#[test]
fn leaving_the_world_destroys_the_emitters_streaks_once() {
    let table = table(vec![node(Some(streak()), true, false)]);
    let mut fx = FxObjects::default();
    fx.start_streaks(EMITTER, FX, &table);
    fx.streak_calls.clear();
    assert_eq!(fx.streaked_emitters(), vec![EMITTER]);
    fx.remove_streaks(EMITTER);
    fx.remove_streaks(EMITTER);
    assert_eq!(
        fx.streak_calls,
        vec![ObjectStreakCall::Gone { emitter: EMITTER }]
    );
    assert!(fx.streaked_emitters().is_empty());
}

/// The streak's ribbon follows its emitter from the FX's start: each frame the emitter is placed
/// it moves the ribbon's newest segment with it, and the older ones trail behind; once destroyed
/// it draws nothing. Needs the DATs (the world's scene).
#[test]
fn an_object_streak_is_created_moves_with_its_emitter_and_is_removed() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let world = Ac2WorldSource::open(store)
        .expect("the world opens")
        .expect("the portal DAT has a world");
    let table = table(vec![node(Some(streak()), true, false)]);
    let mut fx = FxObjects::default();
    fx.start_streaks(EMITTER, FX, &table);
    let calls = std::mem::take(&mut fx.streak_calls);

    let mut scene = WorldScene::new(world, 1);
    let camera = Camera {
        eye: Vec3::new(0.0, -20.0, 0.0),
        target: Vec3::ZERO,
        fov_y: 1.0,
        near: 0.1,
        far: 500.0,
    };
    for call in &calls {
        match *call {
            ObjectStreakCall::Create { emitter, spec } => scene.start_object_streak(emitter, spec),
            ObjectStreakCall::Activate { emitter, on } => {
                scene.set_object_streaks_active(emitter, on)
            }
            ObjectStreakCall::Gone { emitter } => scene.remove_object_streaks(emitter),
        }
    }
    assert_eq!(scene.object_streak_count(EMITTER), 1);

    let placed = |x: f32| ObjectPlacement {
        emitter: EMITTER,
        cell: 1,
        frame: Frame::new(Vec3::new(x, 0.0, 0.0), Quat::IDENTITY),
        scale: Vec3::ONE,
        bones: Vec::new(),
        named_bones: Vec::new(),
    };
    let mut time = 0.0;
    let mut ribbon = Vec::new();
    for step in 0..20 {
        time += 0.05;
        let x = step as f32 * 0.5;
        scene.set_clock(time);
        scene.place_objects(&[placed(x)]);
        let frame = scene.frame(&camera, 1.0);
        ribbon = frame
            .particles
            .iter()
            .flat_map(|batch| batch.geometry.vertices.iter().map(|v| v.position))
            .collect();
    }
    assert!(!ribbon.is_empty(), "the moving streak draws its ribbon");
    let newest = ribbon.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
    let oldest = ribbon.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
    assert!(
        (newest - 9.5).abs() < 1e-3,
        "the newest segment is at the emitter: {newest}"
    );
    assert!(oldest < newest, "the older segments trail behind it");

    scene.set_object_streaks_active(EMITTER, false);
    scene.remove_object_streaks(EMITTER);
    time += 0.05;
    scene.set_clock(time);
    scene.place_objects(&[placed(10.0)]);
    assert!(
        scene.frame(&camera, 1.0).particles.is_empty(),
        "a destroyed emitter's streak draws nothing"
    );
    assert_eq!(scene.object_streak_count(EMITTER), 0);
}
