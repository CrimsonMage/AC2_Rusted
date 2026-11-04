//! `ParticleEmitter::ParticleThink` and `ApplyKeyframe` turning a particle (Legions 0x005974D0,
//! 0x00596F40): the rotation is the start rotation plus the turn rate times the whole time
//! alive, and each controller key adds the rate times that key's own time to the start
//! rotation, so after a second key the turn counts twice, as shipped.

use std::sync::Arc;

use ac2_assets::{
    Ac2ParticleEmitterDesc, Ac2ParticleKeyframe, Ac2ParticleSystemDesc, Ac2WaveformVec3,
};
use ac2_math::{Frame, Vec3, Waveform};

use super::ParticleSystem;

fn key(time: f32) -> Ac2ParticleKeyframe {
    Ac2ParticleKeyframe {
        time,
        key_flags: 0x80,
        scale_x: 1.0,
        scale_y: 1.0,
        color: [1.0; 4],
        mass: 1.0,
        controller: 1,
        force_flags: 0,
        position: Default::default(),
        points: Arc::from([]),
    }
}

fn zero() -> Ac2WaveformVec3 {
    Ac2WaveformVec3 {
        x: Waveform::constant(0.0),
        y: Waveform::constant(0.0),
        z: Waveform::constant(0.0),
    }
}

fn system() -> ParticleSystem {
    let emitter = Ac2ParticleEmitterDesc {
        max_particles: 1,
        particle_shape: 4,
        scale: Waveform::constant(1.0),
        particle_scale: Waveform::constant(1.0),
        active: true,
        birth_rate: 0.1,
        lifespan: Waveform::constant(10.0),
        velocity: Waveform::constant(0.0),
        blast_count: 1,
        particle_snap: Waveform::constant(0.0),
        rotate_velocity: Ac2WaveformVec3 {
            x: Waveform::constant(1.0),
            y: Waveform::constant(0.0),
            z: Waveform::constant(0.0),
        },
        rotation: zero(),
        world_rotation: zero(),
        keyframes: Arc::from([key(0.0), key(0.5), key(1.0)]),
        ..Default::default()
    };
    let desc = Ac2ParticleSystemDesc {
        max_particles: 1,
        scale_type: 1,
        physics_time_step: 1.0 / 30.0,
        stop_fade_distance: 100.0,
        world_space: true,
        emitters: Arc::from([emitter]),
        ..Default::default()
    };
    ParticleSystem::new(Arc::new(desc), Frame::IDENTITY, Vec3::ONE, 1.0, 3, 100.0)
}

/// Thinks to `until` and returns the first particle's rotation and time alive.
fn rotation_at(system: &mut ParticleSystem, now: &mut f64, until: f64) -> (f32, f32) {
    while *now < until {
        *now += 1.0 / 30.0;
        system.think(*now);
    }
    let p = &system.emitters[0].particles()[0];
    (p.rotation.x, (*now - p.time_created) as f32)
}

/// Before the second key the turn is the time alive; past the keys at 0.5 and 1 s it is the
/// time alive plus 0.5 plus 1 (each key's own time, where the controller spans were 0.5 each).
#[test]
fn a_particle_turns_over_its_life_and_a_key_carries_its_time() {
    let mut system = system();
    let mut now = 100.0;
    // One think step of slack: the rotation is the last think's.
    let step = 1.0 / 30.0 + 1e-3;
    let (early, alive) = rotation_at(&mut system, &mut now, 100.4);
    assert!(
        alive < 0.5 && (early - alive).abs() < step,
        "{early} at {alive}"
    );
    let (late, alive) = rotation_at(&mut system, &mut now, 101.6);
    assert!(
        alive > 1.0 && (late - (alive + 1.5)).abs() < step,
        "{late} at {alive}"
    );
}
