//! An FX's repulsor nodes on their emitter: made once per running FX, gated like the node's other
//! parts, destroyed when the emitter leaves.

use std::sync::Arc;

use ac2_assets::{
    Ac2FxObject, Ac2ParticleAnchor, Ac2ParticleAttachment, Ac2RepulsorFx, Ac2VisualFxParticles,
};
use ac2_math::Vec3;

use super::{FxRepulsors, ObjectRepulsorCall};
use crate::fx_streaks::StreakContext;
use crate::particles::RepulsorSpec;

const FX: u32 = 0x4000_0100;
const EMITTER: u64 = 3;

fn repulsor() -> Ac2RepulsorFx {
    Ac2RepulsorFx {
        acceleration: 12.0,
        range: 4.0,
        lifetime: 2.5,
        minimum_velocity: 1.0,
        attractor: true,
    }
}

fn node(repulsor: Option<Ac2RepulsorFx>) -> Ac2ParticleAttachment {
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
            repulsor,
            ..Ac2FxObject::default()
        },
    }
}

fn day() -> StreakContext<'static> {
    StreakContext {
        player: None,
        night: false,
        holds: &[],
        terrain: 0,
    }
}

#[test]
fn a_node_with_a_repulsor_makes_it_once_per_running_fx() {
    let table =
        Ac2VisualFxParticles::from_entries([(FX, vec![node(None), node(Some(repulsor()))])]);
    let mut fx = FxRepulsors::default();
    fx.start(EMITTER, FX, &table, &day());
    fx.start(EMITTER, FX, &table, &day());
    let expected = ObjectRepulsorCall::Create {
        emitter: EMITTER,
        spec: RepulsorSpec {
            acceleration: 12.0,
            range: 4.0,
            lifetime: 2.5,
            minimum_velocity: 1.0,
            attractor: true,
        },
    };
    assert_eq!(fx.take_calls(), vec![expected]);
    // Stopped, the FX can run again and make another.
    fx.stop(EMITTER, FX);
    fx.start(EMITTER, FX, &table, &day());
    assert_eq!(fx.take_calls(), vec![expected]);
}

#[test]
fn a_night_only_node_makes_none_by_day() {
    let mut night_only = node(Some(repulsor()));
    night_only.object.gate.day_time = false;
    let table = Ac2VisualFxParticles::from_entries([(FX, vec![night_only])]);
    let mut fx = FxRepulsors::default();
    fx.start(EMITTER, FX, &table, &day());
    assert!(fx.take_calls().is_empty());
}

#[test]
fn the_repulsors_go_once_with_their_emitter() {
    let table = Ac2VisualFxParticles::from_entries([(FX, vec![node(Some(repulsor()))])]);
    let mut fx = FxRepulsors::default();
    fx.start(EMITTER, FX, &table, &day());
    fx.take_calls();
    assert_eq!(fx.repulsed_emitters(), vec![EMITTER]);
    fx.remove_repulsors(EMITTER);
    fx.remove_repulsors(EMITTER);
    assert_eq!(
        fx.take_calls(),
        vec![ObjectRepulsorCall::Gone { emitter: EMITTER }]
    );
    assert!(fx.repulsed_emitters().is_empty());
}

/// `DoStartupFX` makes every repulsor node's repulsor with no gate (a night-only node by day too),
/// and the emitter's `FXManager::m_terrain` selects the node list a started FX makes it from.
#[test]
fn startup_nodes_make_their_repulsors_without_a_gate_and_a_terrain_picks_the_list() {
    let mut night_only = node(Some(repulsor()));
    night_only.object.gate.day_time = false;
    let mut fx = FxRepulsors::default();
    fx.start_startup(EMITTER, &[node(None), night_only.clone()]);
    assert_eq!(
        fx.calls.len(),
        1,
        "the night-only node makes its repulsor by day"
    );
    assert_eq!(fx.repulsed_emitters(), vec![EMITTER]);

    // A terrain's list replaces the default's: here the default has none and the terrain has one.
    let table = Ac2VisualFxParticles::from_entries([(FX, vec![node(None)])]).with_terrain(
        FX,
        0x8100_0004,
        vec![node(Some(repulsor()))],
    );
    let mut on_grass = FxRepulsors::default();
    let ctx = StreakContext {
        terrain: 0x8100_0004,
        ..day()
    };
    on_grass.start(EMITTER, FX, &table, &ctx);
    assert_eq!(on_grass.take_calls().len(), 1);
    let mut elsewhere = FxRepulsors::default();
    elsewhere.start(EMITTER, FX, &table, &day());
    assert!(elsewhere.take_calls().is_empty());
}

/// The scene keeps a repulsor on its emitter, places it where the emitter draws with the body's
/// speed (`PHYSICSOBJREP::Draw`), destroys it past its lifetime, and with its emitter. Needs the
/// DATs (the world's scene).
#[test]
fn the_scene_places_a_repulsor_with_its_emitter_and_ends_it_by_lifetime() {
    use crate::scene::{ObjectPlacement, WorldScene};
    use ac2_math::{Frame, Quat};
    use ac2_world::Ac2WorldSource;

    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let world = Ac2WorldSource::open(store)
        .expect("the world opens")
        .expect("the portal DAT has a world");
    let mut scene = WorldScene::new(world, 1);
    let spec = RepulsorSpec {
        acceleration: 5.0,
        range: 3.0,
        lifetime: 1.0,
        minimum_velocity: 2.0,
        attractor: false,
    };
    scene.start_object_repulsor(EMITTER, spec);
    assert_eq!(scene.object_repulsor_count(EMITTER), 1);
    let placed = |speed: f32| ObjectPlacement {
        emitter: EMITTER,
        cell: 1,
        frame: Frame::new(Vec3::new(4.0, 5.0, 6.0), Quat::IDENTITY),
        scale: Vec3::ONE,
        particle_scale: 1.0,
        bones: Vec::new(),
        named_bones: Vec::new(),
        parents: Arc::from(Vec::new()),
        speed,
    };
    scene.set_clock(0.0);
    scene.place_objects(&[placed(1.0)]);
    let near = |scene: &WorldScene, now: f64| scene.repulsors().nearby(now);
    assert!(
        near(&scene, 0.5).is_empty(),
        "too slow for its minimum velocity"
    );
    scene.place_objects(&[placed(3.0)]);
    let nearby = near(&scene, 0.5);
    assert_eq!(nearby.len(), 1);
    assert_eq!(nearby[0].position, Vec3::new(4.0, 5.0, 6.0));
    // Past its lifetime it is no longer active, and the next draw destroys it.
    assert!(near(&scene, 1.5).is_empty());
    scene.set_clock(1.5);
    scene.place_objects(&[placed(3.0)]);
    assert_eq!(scene.object_repulsor_count(EMITTER), 0);
    assert!(scene.repulsors().is_empty());

    scene.start_object_repulsor(EMITTER, spec);
    scene.remove_object_repulsors(EMITTER);
    assert_eq!(scene.object_repulsor_count(EMITTER), 0);
    assert!(scene.repulsors().is_empty());
}
