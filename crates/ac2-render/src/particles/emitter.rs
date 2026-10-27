//! `ParticleEmitter`: one emitter of a system. It discharges particles on its birth rate
//! (`EmitTimedParticles`, `DischargeParticle`), thinks them (`Think`, `ParticleThink`) and drops
//! the ones past their lifespan.

use ac2_assets::{Ac2ParticleEmitterDesc, Ac2ParticleSystemDesc, Ac2Waveform, Ac2WaveformVec3};
use ac2_math::{Quat, Vec3};

use super::keyframes::{
    flag, setup_keyframes, KeyEntry, KeyInfo, Particle, CONTROLLER_PHYSICS, CONTROLLER_POINT_FILE,
    CONTROLLER_POINT_FILE_LERP, CONTROLLER_REPULSE, CONTROLLER_WAVEFORM,
};
use super::{SystemContext, EPSILON, GRAVITY};

/// `Waveform::kind::NONE`.
const WAVEFORM_NONE: u32 = ac2_math::waveform::kind::NONE;

fn is_none(wave: &Ac2Waveform, base: f32) -> bool {
    wave.kind == WAVEFORM_NONE && wave.base == base
}

fn vec_is_zero(vector: &Ac2WaveformVec3) -> bool {
    is_none(&vector.x, 0.0) && is_none(&vector.y, 0.0) && is_none(&vector.z, 0.0)
}

pub(crate) struct Emitter {
    /// `m_nEmitterDesc`: this emitter's place in the description.
    pub index: usize,
    pub flags: u32,
    respond_to_repulsors: bool,
    pub keys: Vec<KeyEntry>,
    particles: Vec<Particle>,
    max_particles: usize,
    pub last_think: f64,
    next_birth: f64,
    discharged: u32,
    pub gravity: Vec3,
    pub wind: Vec3,
}

impl Emitter {
    /// `ParticleEmitter::ParticleEmitter`, `Initialize` and `Reset`.
    pub(crate) fn new(system: &Ac2ParticleSystemDesc, index: usize) -> Self {
        let desc = &system.emitters[index];
        let keys = setup_keyframes(desc);
        let (flags, respond_to_repulsors) = data_flags(system, desc);
        let mut max = desc.max_particles as usize;
        if desc.active && desc.emission_limit != 0 && desc.emission_limit <= desc.max_particles {
            max = desc.emission_limit as usize;
        }
        Self {
            index,
            flags,
            respond_to_repulsors,
            keys,
            particles: Vec::new(),
            max_particles: if desc.active { max } else { 0 },
            last_think: 0.0,
            next_birth: 0.0,
            discharged: 0,
            // `ParticleSystem::GetExternalForces`: the world's gravity, no wind.
            gravity: GRAVITY,
            wind: Vec3::ZERO,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.particles.len()
    }

    /// `ParticleEmitter::GetReadyToDie`: the emitter has stopped discharging for good. It is
    /// inactive, its time limit has passed (the window runs from the system's start and the
    /// emitter's start time), or its emission limit is spent.
    pub(crate) fn ready_to_die(&self, desc: &Ac2ParticleSystemDesc, system_start: f64, now: f64) -> bool {
        let emitter = &desc.emitters[self.index];
        if !emitter.active {
            return true;
        }
        let ended = emitter.time_limit.abs() >= EPSILON
            && system_start + f64::from(emitter.start_time) + f64::from(emitter.time_limit) <= now;
        let spent = emitter.emission_limit != 0 && self.discharged >= emitter.emission_limit;
        ended || spent
    }

    pub(crate) fn responds_to_repulsors(&self) -> bool {
        self.respond_to_repulsors
    }

    /// `ParticleEmitter::SetLastDischargeTime`.
    pub(crate) fn set_last_discharge_time(&mut self, time: f64) {
        self.last_think = time;
        self.next_birth = time;
    }

    pub(crate) fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// `ParticleEmitter::EmitParticles`: timed emission while the emitter is within its window.
    pub(crate) fn emit_particles(&mut self, context: &mut SystemContext<'_>, now: f64) {
        let system = context.desc;
        let desc = &system.emitters[self.index];
        if !desc.active {
            return;
        }
        let since_start = now - context.start_time;
        let begins = f64::from(desc.start_time);
        let within =
            desc.time_limit.abs() < EPSILON || since_start < f64::from(desc.time_limit) + begins;
        if begins <= since_start && within {
            self.emit_timed_particles(context, now);
        }
    }

    /// `ParticleEmitter::EmitTimedParticles`.
    fn emit_timed_particles(&mut self, context: &mut SystemContext<'_>, now: f64) {
        if self.next_birth > now {
            return;
        }
        let system = context.desc;
        let desc = &system.emitters[self.index];
        if desc.birth_rate == 0.0 {
            return;
        }
        let late = now - self.next_birth;
        let count = (late / f64::from(desc.birth_rate)) as i64;
        if count * i64::from(desc.blast_count) == 0 {
            return;
        }
        let mut room = self.max_particles.saturating_sub(self.particles.len());
        let lifespan = desc.lifespan.value(self.next_birth);
        if lifespan < late as f32 {
            // Emission fell so far behind that the oldest particles would already be dead:
            // skip to the births that can still be alive.
            let skipped = ((late - f64::from(lifespan)) / f64::from(desc.birth_rate)) as i64;
            self.next_birth += skipped as f64 * f64::from(desc.birth_rate);
            self.discharged = self
                .discharged
                .wrapping_add((i64::from(desc.blast_count) * skipped) as u32);
        }
        let limit = desc.emission_limit;
        if (limit != 0 && self.discharged > limit) || self.next_birth >= now {
            return;
        }
        loop {
            if room == 0 {
                self.next_birth = now;
                return;
            }
            let mut blast = desc.blast_count;
            if limit != 0 && limit < self.discharged + blast {
                blast = limit.saturating_sub(self.discharged);
            }
            if blast < 1 {
                self.next_birth = now;
                return;
            }
            let blast = (blast as usize).min(room);
            self.discharged += blast as u32;
            let sweep = if desc.shape == 4 {
                let direction = vec_value(&desc.direction, now);
                direction.normalize_or_zero() * desc.scale.value(now)
            } else {
                Vec3::ZERO
            };
            for _ in 0..blast {
                self.discharge_particle(context, now, self.next_birth, Vec3::ZERO, sweep);
            }
            room -= blast;
            self.next_birth += f64::from(desc.birth_rate);
            if now <= self.next_birth {
                return;
            }
        }
    }

    /// `ParticleEmitter::DischargeParticle` from outside the emitter's timing: born now, at
    /// `offset`, with no sweep.
    pub(crate) fn discharge(&mut self, context: &mut SystemContext<'_>, now: f64, offset: Vec3) {
        self.discharge_particle(context, now, now, offset, Vec3::ZERO);
    }

    /// `ParticleEmitter::CalculateInitialOrigin`: where in the emitter's shape a particle starts.
    fn initial_origin(&self, context: &mut SystemContext<'_>, wave_time: f64, sweep: Vec3) -> Vec3 {
        let system = context.desc;
        let desc = &system.emitters[self.index];
        let scale = desc.scale.value(wave_time);
        let rng = &mut *context.rng;
        match desc.shape {
            1 => {
                let (low, high) = (scale * -0.5, scale * 0.5);
                let y = rng.roll_dice_f32(low, high);
                let x = rng.roll_dice_f32(low, high);
                let mut out = Vec3::new(x, y, 0.0);
                if !desc.inclusive_shape {
                    match rng.roll_dice(0, 3) {
                        0 => out.x = low,
                        1 => out.y = low,
                        2 => out.x = high,
                        3 => out.y = high,
                        _ => {}
                    }
                }
                out
            }
            2 => {
                let y = rng.roll_dice_f32(-1.0, 1.0);
                let x = rng.roll_dice_f32(-1.0, 1.0);
                let mut out = Vec3::new(x, y, 0.0).normalize_or_zero();
                if desc.inclusive_shape {
                    out *= rng.roll_dice_f32(0.0, 1.0);
                }
                out * (scale * 0.5)
            }
            3 => {
                let z = rng.roll_dice_f32(-1.0, 1.0);
                let y = rng.roll_dice_f32(-1.0, 1.0);
                let x = rng.roll_dice_f32(-1.0, 1.0);
                let mut out = Vec3::new(x, y, z).normalize_or_zero();
                if desc.inclusive_shape {
                    out *= rng.roll_dice_f32(0.0, 1.0);
                }
                out * (scale * 0.5)
            }
            4 => {
                if sweep.length_squared() < EPSILON {
                    return Vec3::ZERO;
                }
                sweep.normalize() * rng.roll_dice_f32(0.0, scale)
            }
            _ => Vec3::ZERO,
        }
    }

    /// `ParticleEmitter::DischargeParticle`: a particle born at `birth`, aged to the emitter's
    /// last think. `offset` is where the system had moved since (zero for timed emission) and
    /// `sweep` the swept line's extent.
    fn discharge_particle(
        &mut self,
        context: &mut SystemContext<'_>,
        now: f64,
        birth: f64,
        offset: Vec3,
        sweep: Vec3,
    ) {
        let system = context.desc;
        let desc = &system.emitters[self.index];
        let wave_time = birth - (f64::from(desc.start_time) + context.start_time);
        let lifespan = desc.lifespan.value(wave_time);
        let alive_until = f64::from(lifespan) + birth;
        if alive_until <= now || self.particles.len() >= self.max_particles {
            return;
        }
        let local_direction = vec_value(&desc.direction, wave_time);
        let mut direction = context.frame.local_to_global_vector(local_direction);
        let origin_in_shape = self.initial_origin(context, wave_time, sweep);
        if desc.exploding_dir {
            direction = origin_in_shape;
            if direction.length_squared() < EPSILON * EPSILON {
                direction = Vec3::new(
                    context.rng.roll_dice_f32(-1.0, 1.0),
                    context.rng.roll_dice_f32(-1.0, 1.0),
                    context.rng.roll_dice_f32(-1.0, 1.0),
                );
            }
        }
        direction = if direction.length() < EPSILON {
            Vec3::Y
        } else {
            direction.normalize()
        };
        let local_origin = origin_in_shape + vec_value(&desc.origin, wave_time);
        let min_spread = desc.min_spread.value(wave_time);
        let max_spread = desc.max_spread.value(wave_time);
        if max_spread - min_spread > EPSILON {
            let angle = context.rng.roll_dice_f32(min_spread, max_spread) * 0.017_453_292;
            if angle != 0.0 {
                // Tilt the direction by `angle` about a perpendicular axis, then turn the
                // result about the original direction by a random angle.
                let right = direction_right(direction);
                let tilted = Quat::from_axis_angle(right, angle) * direction;
                let spin = context.rng.roll_dice_f32(0.0, std::f32::consts::TAU);
                direction = Quat::from_axis_angle(direction, spin) * tilted;
            }
        }
        let position = context.frame.local_to_global_point(local_origin) + offset;
        let speed = desc.velocity.value(wave_time);
        let velocity = direction * speed;

        let mut p = Particle::new();
        p.origin = position;
        let flags = self.flags;
        if flags & flag::TIME_CREATED != 0 {
            p.time_created = birth;
        }
        if flags & flag::START_ORIGIN != 0 {
            p.start_origin = position;
        }
        if flags & flag::WORLD_ORIGIN != 0 {
            p.world_origin = context.frame.origin;
        }
        if flags & flag::KEYFRAME_FLAGS != 0 {
            p.keyframe_flags = 0;
        }
        if flags & flag::VELOCITY != 0 {
            p.velocity = velocity;
        }
        if flags & flag::POINT_SPEED != 0 {
            p.point_speed = velocity.length();
        }
        if flags & flag::POINT_INDEX != 0 {
            p.point_index = 0xFFFF;
        }
        if flags & flag::SCALE != 0 {
            p.scale = desc.particle_scale.value(wave_time);
        }
        if flags & flag::STREAK != 0 {
            p.streak = desc.streak.value(wave_time);
        }
        if flags & flag::STREAK_ORIGIN != 0 {
            p.streak_origin = position;
        }
        if flags & flag::ROTATE_VELOCITY != 0 {
            p.rotate_velocity = vec_value(&desc.rotate_velocity, wave_time);
            p.rotation = Vec3::ZERO;
            p.world_rotation = Vec3::ZERO;
        }
        if flags & flag::ROTATION != 0 {
            p.rotation = vec_value(&desc.rotation, wave_time);
            if flags & flag::START_ROTATION != 0 {
                p.start_rotation = p.rotation;
            }
        }
        if flags & flag::WORLD_ROTATION != 0 {
            p.world_rotation = vec_value(&desc.world_rotation, wave_time);
            if flags & flag::START_WORLD_ROTATION != 0 {
                p.start_world_rotation = p.world_rotation;
            }
        }
        if flags & flag::KEYFRAME_INFO != 0 {
            p.key = KeyInfo {
                current: None,
                last: [None; 4],
            };
        }
        // Aged to the last think: the particle was born in the past.
        let aged = self.last_think;
        let quantum = (self.last_think - birth) as f32;
        self.particle_think(context, quantum, aged, &mut p);
        if flags & flag::PREVIOUS_ORIGIN != 0 {
            p.previous_origin = p.origin;
        }
        if flags & flag::PREVIOUS_STREAK_ORIGIN != 0 {
            p.previous_streak_origin = p.streak_origin;
        }
        self.particles.push(p);
    }

    /// `ParticleEmitter::Think`: the particles through the time since the last think, once it is
    /// at least a physics time step, dropping the ones that have died.
    pub(crate) fn think(&mut self, context: &mut SystemContext<'_>, now: f64) {
        if (now - self.last_think) < f64::from(context.desc.physics_time_step) {
            return;
        }
        let mut quantum = (now - self.last_think) as f32;
        let mut index = 0;
        while index < self.particles.len() {
            let mut p = std::mem::replace(&mut self.particles[index], Particle::new());
            if self.particle_think(context, quantum, now, &mut p) {
                self.particles[index] = p;
                // Retail leaves `fQuantum` at a denormal after the first live particle, so only
                // that one is re-anchored after a long stall.
                quantum = f32::MIN_POSITIVE * 1.0e-8;
                index += 1;
            } else {
                self.particles.swap_remove(index);
            }
        }
        self.last_think = now;
    }

    /// `ParticleEmitter::ParticleThink`: false when the particle is past its lifespan.
    pub(crate) fn particle_think(
        &mut self,
        context: &mut SystemContext<'_>,
        quantum: f32,
        now: f64,
        p: &mut Particle,
    ) -> bool {
        let system = context.desc;
        let desc = &system.emitters[self.index];
        let base = context.start_time + f64::from(desc.start_time);
        let lifespan = desc.lifespan.value(p.time_created - base);
        let time_alive = (now - p.time_created) as f32;
        if time_alive > lifespan {
            return false;
        }
        let (mut controller, mut controller_time) = (0, 0.0);
        if self.flags & flag::KEYFRAME_INFO != 0 {
            self.apply_keyframes(context, p, now);
            if let Some(key) = p.key.last[3] {
                let keyframe = &desc.keyframes[self.keys[key].desc];
                controller = keyframe.controller;
                controller_time = keyframe.time;
            }
        }
        if self.flags & flag::PREVIOUS_ORIGIN != 0 {
            p.previous_origin = p.origin;
        }
        if self.flags & flag::PREVIOUS_STREAK_ORIGIN != 0 {
            p.previous_streak_origin = p.streak_origin;
        }
        let relative = time_alive - controller_time;
        match controller {
            CONTROLLER_PHYSICS => self.do_physics(desc, relative, now, p),
            CONTROLLER_WAVEFORM => self.do_waveform(relative, p),
            // Repulsors are not modelled: a repulse particle stays where it was.
            CONTROLLER_REPULSE => {}
            CONTROLLER_POINT_FILE => self.do_point_file(context, p),
            CONTROLLER_POINT_FILE_LERP => {
                self.point_file_lerp(context, p, (now - self.last_think) as f32)
            }
            _ => {}
        }
        if self.flags & flag::ROTATE_VELOCITY != 0 {
            let turned = p.rotate_velocity * relative;
            if self.flags & flag::ROTATION != 0 {
                p.rotation = p.start_rotation + turned;
            }
            if self.flags & flag::WORLD_ROTATION != 0 {
                p.world_rotation = p.start_world_rotation + turned;
            }
        }
        if quantum > 0.25 {
            if self.flags & flag::PREVIOUS_ORIGIN != 0 {
                p.previous_origin = p.origin;
            }
            if self.flags & flag::PREVIOUS_STREAK_ORIGIN != 0 {
                p.previous_streak_origin = p.streak_origin;
            }
        }
        true
    }
}

/// A waveform vector's value at `time`.
pub(crate) fn vec_value(vector: &Ac2WaveformVec3, time: f64) -> Vec3 {
    Vec3::new(
        vector.x.value(time),
        vector.y.value(time),
        vector.z.value(time),
    )
}

/// A unit vector perpendicular to `direction` (`Matrix4::DirectionMatrix`'s right axis, with the
/// world's up as the up hint).
fn direction_right(direction: Vec3) -> Vec3 {
    let up = Vec3::Z - direction * direction.dot(Vec3::Z);
    let up = if up.length() < EPSILON {
        // Pointing along the up hint: use the next axis.
        let alt = Vec3::Y - direction * direction.dot(Vec3::Y);
        if alt.length() < EPSILON {
            return Vec3::X;
        }
        alt.normalize()
    } else {
        up.normalize()
    };
    up.cross(direction).normalize_or_zero()
}

/// `ParticleEmitter::DetermineDataFlags`: which channels the emitter's particles carry, and
/// whether it responds to repulsors.
fn data_flags(system: &Ac2ParticleSystemDesc, desc: &Ac2ParticleEmitterDesc) -> (u32, bool) {
    let mut flags = 8;
    if system.world_space {
        flags = 0x10_0008;
    }
    if !desc.keyframes.is_empty() {
        flags |= 0x92;
    }
    let mut respond = false;
    let controllers = |kind| desc.keyframes.iter().any(|k| k.controller == kind);
    if controllers(CONTROLLER_WAVEFORM) {
        flags |= 0x116;
    }
    if controllers(CONTROLLER_PHYSICS) {
        flags |= 0x74;
    }
    if controllers(CONTROLLER_REPULSE) {
        flags |= 0x74;
        respond = true;
    }
    if controllers(CONTROLLER_POINT_FILE) {
        flags |= 0x4_0054;
    }
    if controllers(CONTROLLER_POINT_FILE_LERP) {
        flags |= 0xC_0054;
    }
    if !is_none(&desc.streak, 0.0) {
        flags |= 0xE000;
    }
    if !is_none(&desc.particle_scale, 1.0) {
        flags |= 0x200;
    }
    if vec_is_zero(&desc.world_rotation) {
        if !vec_is_zero(&desc.rotation) {
            flags |= flag::ROTATION;
        }
    } else {
        flags |= flag::WORLD_ROTATION;
    }
    if !vec_is_zero(&desc.rotate_velocity) {
        if flags & flag::WORLD_ROTATION != 0 {
            return (flags | 0x2_1000, respond);
        }
        flags |= 0x1C00;
    }
    (flags, respond)
}
