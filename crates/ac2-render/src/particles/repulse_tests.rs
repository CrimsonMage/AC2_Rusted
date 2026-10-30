//! `ParticleEmitter::DoRepulse` on a one-emitter system built here: the particles sit still until a
//! repulsor is in range, are pushed away by a repulsor and pulled in by an attractor.

use std::sync::Arc;

use ac2_assets::{Ac2ParticleEmitterDesc, Ac2ParticleKeyframe, Ac2ParticleSystemDesc};
use ac2_math::{Frame, Vec3, Waveform};

use super::{NearbyRepulsor, ParticleSystem};

/// A repulse emitter that emits a motionless particle every tenth of a second, which lives ten
/// seconds, and holds each where it is born unless pushed.
fn system(constrain_z: bool) -> ParticleSystem {
    let key = Ac2ParticleKeyframe {
        time: 0.0,
        key_flags: 0x80,
        scale_x: 1.0,
        scale_y: 1.0,
        color: [1.0; 4],
        mass: 1.0,
        controller: 3,
        force_flags: 0,
        position: Default::default(),
        points: Arc::from([]),
    };
    let emitter = Ac2ParticleEmitterDesc {
        max_particles: 8,
        particle_shape: 4,
        scale: Waveform::constant(1.0),
        particle_scale: Waveform::constant(1.0),
        active: true,
        birth_rate: 0.1,
        lifespan: Waveform::constant(10.0),
        velocity: Waveform::constant(0.0),
        blast_count: 1,
        constrain_axes: [false, false, constrain_z],
        particle_snap: Waveform::constant(0.0),
        keyframes: Arc::from([key]),
        ..Default::default()
    };
    let desc = Ac2ParticleSystemDesc {
        max_particles: 8,
        scale_type: 1,
        physics_time_step: 1.0 / 30.0,
        stop_fade_distance: 100.0,
        world_space: true,
        emitters: Arc::from([emitter]),
        ..Default::default()
    };
    ParticleSystem::new(Arc::new(desc), Frame::IDENTITY, Vec3::ONE, 1.0, 7, 100.0)
}

fn run(system: &mut ParticleSystem, repulsors: &[NearbyRepulsor], seconds: f64) {
    let mut now = 100.0;
    while now < 100.0 + seconds {
        now += 1.0 / 30.0;
        system.think_with(now, repulsors);
    }
}

fn origins(system: &ParticleSystem) -> Vec<Vec3> {
    system.emitters[0]
        .particles()
        .iter()
        .map(|p| p.origin)
        .collect()
}

fn at(x: f32, attractor: bool) -> NearbyRepulsor {
    NearbyRepulsor {
        position: Vec3::new(x, 0.0, 0.5),
        range: 4.0,
        acceleration: 20.0,
        attractor,
    }
}

#[test]
fn a_repulse_emitter_responds_to_repulsors() {
    assert!(system(false).needs_repulsors());
}

#[test]
fn without_a_repulsor_the_particles_stay_put() {
    let mut system = system(false);
    run(&mut system, &[], 1.0);
    let origins = origins(&system);
    assert!(!origins.is_empty());
    assert!(origins.iter().all(|o| o.length() < 1e-5), "{origins:?}");
}

#[test]
fn a_repulsor_pushes_particles_away_and_an_attractor_pulls_them_in() {
    let mut pushed = system(false);
    run(&mut pushed, &[at(-1.0, false)], 1.0);
    let pushed = origins(&pushed);
    // The repulsor is at x = -1, z = 0.5: the push is along +x and -z.
    assert!(
        pushed.iter().all(|o| o.x > 0.01 && o.z < -0.001),
        "{pushed:?}"
    );

    let mut pulled = system(false);
    run(&mut pulled, &[at(-1.0, true)], 1.0);
    let pulled = origins(&pulled);
    assert!(
        pulled.iter().all(|o| o.x < -0.01 && o.z > 0.001),
        "{pulled:?}"
    );
}

#[test]
fn a_repulsor_out_of_range_does_nothing_and_a_constrained_axis_holds() {
    let mut far = system(false);
    run(&mut far, &[at(-5.0, false)], 1.0);
    assert!(origins(&far).iter().all(|o| o.length() < 1e-5));

    let mut flat = system(true);
    run(&mut flat, &[at(-1.0, false)], 1.0);
    let flat = origins(&flat);
    assert!(flat.iter().all(|o| o.x > 0.01 && o.z == 0.0), "{flat:?}");
}
