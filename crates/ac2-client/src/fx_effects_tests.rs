//! A server's `DoFXScript` on a human body, on the real DATs (`AC2_DAT_DIR`, skipped without):
//! the script's timeline starts its effect; the effect's footstep sound plays through the
//! object's `FXManager` sound part and its particle systems are queued for the renderer.

use std::sync::Arc;
use std::time::Duration;

use ac2_anim::{Ac2AnimEvent, Ac2AnimTrigger, Ac2FxScript};
use ac2_audio::{RecordingSink, SinkCommand};
use ac2_core::Random;
use ac2_db::ResourceStore;
use ac2_math::Vec3;
use ac2_net::protocol::ObjectAction;
use ac2_physics::{Frame, PhysicsBody, Position, Quaternion, Vector};
use ac2_render::scene::{ObjectPlacement, WorldScene};
use ac2_render::Camera;
use ac2_world::Ac2WorldSource;

use super::{FxCall, FxSoundData, FxSounds};
use crate::motion::apply;
use crate::world_data::{DatWorld, WorldData};
use ac2_render::fx_particles::particle_starts;

/// The human visual (`0x1F000023`).
const HUMAN: u32 = 0x1F00_0023;
/// The footstep FX: its nodes pick among the step sounds, and it has particle systems.
const STEP_FX: u32 = 0x4000_0002;
/// The FX script id the server sends; the timeline is attached under it.
const SCRIPT: u32 = 0x5700_0001;
/// The emitter (object) the event is for.
const EMITTER: u64 = 7;

#[test]
fn a_do_fx_script_event_queues_the_effects_sound_and_particle_ids() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let world = DatWorld::new(Arc::clone(&store)).expect("world data opens");
    let data = world.body(Some(HUMAN)).expect("the human body loads");
    let sounds = data.fx_sounds.clone().expect("the human has FX sounds");
    let particles = data
        .fx_particles
        .clone()
        .expect("the human has FX particles");
    // The DAT's FX scripts all read (the library the client attaches at start).
    assert_eq!(ac2_anim::load_fx_scripts(&store).len(), 22);

    // A timeline that starts the footstep FX at its first period.
    let script = Arc::new(Ac2FxScript {
        effects: vec![Ac2AnimTrigger {
            fx: STEP_FX,
            time: 0.0,
            stop: false,
            probability: 1.0,
            intensity: 1.0,
            intensity_var: 0.0,
            play: 1,
        }],
        impulses: Vec::new(),
        period: 1.0,
    });
    let place = Position::new(
        0x0299_0109,
        Frame::new(
            Vector::new(59.0, 240.0, -44.5),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        ),
    );
    let mut body = PhysicsBody::new(place, data.body.shape.clone(), data.body.motion.clone());
    body.set_radius(data.body.radius);
    body.set_animation(data.body.animation.clone().expect("the human animates"));

    // The server's `DoFXScript`, the script's record, then one animation step.
    apply(&mut body, &ObjectAction::DoFxScript(SCRIPT));
    body.resolve_fx_scripts(|id| (id == SCRIPT).then(|| Arc::clone(&script)));
    let mut random = Random::default();
    body.update_animation(0.5, 1.0, &mut random);
    let started: Vec<u32> = body
        .take_animation_events()
        .into_iter()
        .filter_map(|event| match event {
            Ac2AnimEvent::StartFx { fx, intensity } => {
                assert!(intensity > 0.0, "the effect starts at full intensity");
                Some(fx)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        started,
        vec![STEP_FX],
        "the timeline starts the footstep FX"
    );

    // The sound: the FX's sound part plays one sound per node that plays, each from its list.
    let sink = RecordingSink::new();
    let mut manager = crate::sound_manager(Arc::clone(&store), Box::new(sink.clone()));
    let sound_data = FxSoundData {
        table: sounds.table.clone(),
        looping: sounds.looping.clone(),
    };
    let call = FxCall {
        data: &sound_data,
        emitter: EMITTER,
        is_player: false,
        now: 1.0,
    };
    let mut fx_sounds = FxSounds::default();
    let played = fx_sounds.do_fx(STEP_FX, 1.0, &call, &mut manager, &mut random);
    let allowed: Vec<u32> = sounds
        .table
        .get(STEP_FX)
        .expect("the footstep FX has sound nodes")
        .iter()
        .flat_map(|node| node.sounds.iter().copied())
        .collect();
    assert!(!played.is_empty(), "a footstep sound plays");
    assert!(
        played.iter().all(|did| allowed.contains(did)),
        "every sound is one the FX lists: {played:?}"
    );
    assert!(manager.wait_for_loads(Duration::from_secs(30)));
    manager.use_time(0.1);
    let voices = sink
        .commands()
        .iter()
        .filter(|c| matches!(c, SinkCommand::Start { .. }))
        .count();
    assert_eq!(voices, played.len(), "each sound queued starts a voice");

    // The particles: one system per node of the FX's entry, each desc from that node's list.
    let nodes = particles
        .get(STEP_FX)
        .expect("the footstep FX has particle nodes");
    let queued = particle_starts(EMITTER, STEP_FX, &particles, &mut random, None);
    assert!(!queued.is_empty(), "a particle system starts");
    assert_eq!(
        queued.len(),
        nodes
            .iter()
            .filter(|n| !n.particle_descs.is_empty())
            .count()
    );
    for start in &queued {
        assert_eq!((start.emitter, start.fx), (EMITTER, STEP_FX));
        assert!(
            nodes
                .iter()
                .any(|node| node.particle_descs.contains(&start.desc)),
            "{:08X} is one of the FX's particle descs",
            start.desc.raw()
        );
    }
}

/// A `DoFXScript` starts the footstep FX's particle systems; the scene draws them on the object
/// where it stands (the batches of the frame), and a system ends once it is stopped and has run
/// out its particles, or once its emitters have finished and it has no particle left.
#[test]
fn a_do_fx_script_particle_system_is_drawn_on_its_object_until_it_ends() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("the world opens")
        .expect("the portal DAT has a world");
    let data = DatWorld::new(Arc::clone(&store))
        .expect("world data opens")
        .body(Some(HUMAN))
        .expect("the human body loads");
    let particles = data
        .fx_particles
        .clone()
        .expect("the human has FX particles");
    let script = Arc::new(Ac2FxScript {
        effects: vec![Ac2AnimTrigger {
            fx: STEP_FX,
            time: 0.0,
            stop: false,
            probability: 1.0,
            intensity: 1.0,
            intensity_var: 0.0,
            play: 1,
        }],
        impulses: Vec::new(),
        period: 1.0,
    });
    let at = Vec3::new(10.0, 10.0, 1.0);
    let mut body = PhysicsBody::new(
        Position::new(
            0x0299_0109,
            Frame::new(
                Vector::new(at.x, at.y, at.z),
                Quaternion::new(1.0, 0.0, 0.0, 0.0),
            ),
        ),
        data.body.shape.clone(),
        data.body.motion.clone(),
    );
    body.set_radius(data.body.radius);
    body.set_animation(data.body.animation.clone().expect("the human animates"));
    let mut random = Random::default();
    apply(&mut body, &ObjectAction::DoFxScript(SCRIPT));
    body.resolve_fx_scripts(|id| (id == SCRIPT).then(|| Arc::clone(&script)));
    body.update_animation(0.5, 1.0, &mut random);
    let started: Vec<_> = body
        .take_animation_events()
        .into_iter()
        .filter_map(|event| match event {
            Ac2AnimEvent::StartFx { fx, .. } => Some(fx),
            _ => None,
        })
        .flat_map(|fx| particle_starts(EMITTER, fx, &particles, &mut random, None))
        .enumerate()
        .map(|(index, mut start)| {
            start.handle = index as u64 + 1;
            start
        })
        .collect();
    assert!(
        !started.is_empty(),
        "the footstep FX starts a particle system"
    );

    // The client's frame: the systems start on the scene, and the object is placed where it stands.
    let mut scene = WorldScene::new(world, 1);
    scene.set_clock(0.0);
    for start in &started {
        scene.start_object_particle(start.handle, start.emitter, start.desc, start.anchor, true);
    }
    scene.wait();
    assert_eq!(scene.object_particle_systems(), started.len());
    // The human's posed bones: the footstep system attaches to a bone of its Setup.
    let posed = ac2_render::object::load_object_model(
        &store,
        ac2_core::DataId::new(HUMAN),
        &ac2_render::object::RenderSettings::default(),
    )
    .expect("the human model loads")
    .and_then(|model| model.pose(None))
    .expect("the human is posed");
    let placements = [ObjectPlacement {
        emitter: EMITTER,
        cell: 0,
        frame: ac2_math::Frame::new(at, ac2_math::Quat::IDENTITY),
        scale: Vec3::ONE,
        bones: posed.bones,
        named_bones: posed.named_bones,
    }];
    let camera = Camera {
        eye: at + Vec3::new(0.0, -4.0, 1.0),
        target: at + Vec3::new(0.0, 0.0, 1.0),
        fov_y: 1.0,
        near: 0.1,
        far: 500.0,
    };
    let mut time = 0.0;
    let mut drawn = false;
    for _ in 0..400 {
        time += 0.05;
        scene.set_clock(time);
        scene.place_objects(&placements);
        if !scene.frame(&camera, 1.0).particles.is_empty() {
            drawn = true;
            break;
        }
    }
    assert!(
        drawn,
        "the started system reaches the frame's particle batches"
    );
    assert!(
        scene.particles_alive() > 0,
        "the drawn system has particles"
    );

    // Stopped: the system keeps its last place, runs out its particles, and is gone.
    for start in &started {
        scene.stop_object_particle(start.handle);
    }
    let mut gone = false;
    for _ in 0..12_000 {
        time += 1.0;
        scene.set_clock(time);
        scene.place_objects(&placements);
        let frame = scene.frame(&camera, 1.0);
        if scene.object_particle_systems() == 0 {
            assert!(
                frame.particles.is_empty(),
                "a system that ended draws nothing"
            );
            gone = true;
            break;
        }
    }
    assert!(
        gone,
        "a stopped system ends once its particles have run out"
    );
}

/// An FX-created object (`DoCreateObjectFX`'s `CreateTemporaryObject`) of a real FX table's
/// `EntityDesc` loads a body with a shape to draw, through the loader's body path.
#[test]
fn an_fx_created_entity_object_loads_a_drawable_body() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = DatWorld::new(Arc::clone(&store)).expect("world data opens");
    let mut entity = None;
    for did in dataset.ids_in_range(ac2_dat::DatFileKind::Portal, 0x1E00_0000, 0x1EFF_FFFF) {
        let Some(table) = store
            .load_cached_packed::<ac2_assets::Ac2FxTable>(ac2_db::TypedResourceRef::new(
                ac2_dat::DatFileKind::Portal,
                ac2_core::DataId::new(did),
                0,
            ))
            .expect("an FX table decodes")
        else {
            continue;
        };
        entity = table
            .entries
            .iter()
            .flat_map(|(_, nodes)| nodes.iter())
            .map(|node| node.object.vdesc)
            .find(|vdesc| vdesc >> 24 == 0x47);
        if entity.is_some() {
            break;
        }
    }
    let entity = entity.expect("an FX table names an EntityDesc object");
    let data = world
        .body(Some(entity))
        .unwrap_or_else(|e| panic!("EntityDesc {entity:08X} body: {e}"));
    assert!(data.body.shape.height > 0.0 || !data.body.shape.spheres.is_empty());
}
