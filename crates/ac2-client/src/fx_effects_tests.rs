//! A server's `DoFXScript` on a human body, on the real DATs (`AC2_DAT_DIR`, skipped without):
//! the script's timeline starts its effect; the effect's footstep sound plays through the
//! object's `FXManager` sound part and its particle systems are queued for the renderer.

use std::sync::Arc;
use std::time::Duration;

use ac2_anim::{Ac2AnimEvent, Ac2AnimTrigger, Ac2FxScript};
use ac2_audio::{RecordingSink, SinkCommand};
use ac2_core::Random;
use ac2_db::ResourceStore;
use ac2_net::protocol::ObjectAction;
use ac2_physics::{Frame, PhysicsBody, Position, Quaternion, Vector};

use super::{FxCall, FxSoundData, FxSounds};
use crate::fx_particles::particle_starts;
use crate::motion::apply;
use crate::world_data::{DatWorld, WorldData};

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
    assert_eq!(crate::load_fx_scripts(&store).len(), 22);

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
    let mut body = PhysicsBody::new(place, data.shape.clone(), data.motion.clone());
    body.set_radius(data.radius);
    body.set_animation(data.animation.clone().expect("the human animates"));

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
    let queued = particle_starts(EMITTER, STEP_FX, &particles, &mut random);
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
