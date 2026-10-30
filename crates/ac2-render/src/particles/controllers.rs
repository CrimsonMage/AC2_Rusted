//! The motion controllers a keyframe selects (`PEKeyframeDesc::m_PCType`): physics under
//! gravity and wind, waveforms, and point files (`ParticleEmitter::DoPhysics`, `DoWaveform`,
//! `DoPointFile`, `DoPointFileLerp`).

use ac2_assets::Ac2ParticleEmitterDesc;
use ac2_math::Vec3;

use super::emitter::{vec_value, Emitter};
use super::keyframes::{flag, Particle};
use super::{SystemContext, EPSILON};

/// `scale * vector`, each component evaluated in f64 and stored as the float retail keeps.
fn scaled(vector: Vec3, scale: f64) -> Vec3 {
    Vec3::new(
        (scale * f64::from(vector.x)) as f32,
        (scale * f64::from(vector.y)) as f32,
        (scale * f64::from(vector.z)) as f32,
    )
}

impl Emitter {
    /// `ParticleEmitter::DoPhysics`: the start origin plus the displacement under the forces the
    /// particle's keyframe flags enable, `relative` seconds after its controller key.
    pub(crate) fn do_physics(
        &self,
        desc: &Ac2ParticleEmitterDesc,
        relative: f32,
        now: f64,
        p: &mut Particle,
    ) {
        let time_alive = (now - p.time_created) as f32;
        let mass = self.mass(desc, time_alive, p);
        let mut acceleration = Vec3::ZERO;
        if p.keyframe_flags & 1 != 0 {
            acceleration += self.gravity * mass;
        }
        if p.keyframe_flags & 2 != 0 {
            acceleration += self.wind * mass;
        }
        let displacement = |t: f32| p.velocity * t + acceleration * (0.5 * t * t);
        p.origin = p.start_origin + displacement(relative);
        if self.flags & flag::STREAK_ORIGIN != 0 {
            let behind = (relative - p.streak).max(0.0);
            p.streak_origin = p.start_origin + displacement(behind);
        }
    }

    /// `ParticleEmitter::DoRepulse`: the particle is stepped in physics-time-step slices from the
    /// emitter's last think (at most a quarter second back) to `now`. Each slice takes the
    /// forces its keyframe flags enable (gravity, wind), the push of every nearby repulsor whose
    /// range holds the particle (the acceleration at the centre, falling linearly to nothing at
    /// the range; reversed for an attractor; constrained axes zeroed), then moves the particle by
    /// its velocity and the acceleration, snaps it back toward its start origin by the emitter's
    /// `ParticleSnap` waveform, and keeps 95 percent of the step's velocity.
    pub(crate) fn do_repulse(&self, context: &SystemContext<'_>, now: f64, p: &mut Particle) {
        let desc = &context.desc.emitters[self.index];
        let time_alive = (now - p.time_created) as f32;
        let mass = f64::from(self.mass(desc, time_alive, p));
        let step = context.desc.physics_time_step;
        let step64 = f64::from(step);
        let inverse = (1.0 / step64) as f32;
        let mut last = self.last_think;
        if now - last > 0.25 {
            last = now - 0.25;
        }
        while step64 <= now - last {
            last += step64;
            let mut origin = p.origin;
            let mut acceleration = Vec3::ZERO;
            if p.keyframe_flags & 1 != 0 {
                acceleration += scaled(self.gravity, mass);
            }
            if p.keyframe_flags & 2 != 0 {
                acceleration += scaled(self.wind, mass);
            }
            for repulsor in context.repulsors {
                let from = origin - repulsor.position;
                let distance = (f64::from(from.x) * f64::from(from.x)
                    + f64::from(from.y) * f64::from(from.y)
                    + f64::from(from.z) * f64::from(from.z))
                .sqrt() as f32;
                if distance >= repulsor.range {
                    continue;
                }
                let mut push = if distance.abs() < EPSILON {
                    Vec3::ZERO
                } else {
                    let scale = ((1.0 / f64::from(distance))
                        * (1.0 - f64::from(distance) / f64::from(repulsor.range))
                        * f64::from(repulsor.acceleration)) as f32;
                    from * scale
                };
                if repulsor.attractor {
                    push = -push;
                }
                for (axis, constrained) in desc.constrain_axes.iter().enumerate() {
                    if *constrained {
                        push[axis] = 0.0;
                    }
                }
                acceleration += push;
            }
            let moved = |a: f32, v: f32| (f64::from(a) * step64 + step64 * f64::from(v)) as f32;
            let delta = Vec3::new(
                moved(acceleration.x, p.velocity.x),
                moved(acceleration.y, p.velocity.y),
                moved(acceleration.z, p.velocity.z),
            );
            let to_start = p.start_origin - origin;
            let snap = (f64::from(desc.particle_snap.value(f64::from(time_alive))) * step64) as f32;
            let snap64 = f64::from(snap);
            origin = Vec3::new(
                (f64::from(to_start.x) * snap64 + f64::from(delta.x) + f64::from(origin.x)) as f32,
                (f64::from(to_start.y) * snap64 + f64::from(delta.y) + f64::from(origin.y)) as f32,
                (f64::from(to_start.z) * snap64 + f64::from(delta.z) + f64::from(origin.z)) as f32,
            );
            p.origin = origin;
            let keep = |d: f32| (f64::from(d) * f64::from(inverse) * 0.95) as f32;
            p.velocity = Vec3::new(keep(delta.x), keep(delta.y), keep(delta.z));
        }
    }

    /// `ParticleEmitter::DoWaveform`: the start origin plus the keyframe's position waveforms.
    pub(crate) fn do_waveform(&self, relative: f32, p: &mut Particle) {
        let Some(wave) = p.waveform else {
            return;
        };
        p.origin = p.start_origin + vec_value(&wave, f64::from(relative));
        if self.flags & flag::STREAK_ORIGIN != 0 {
            let behind = (relative - p.streak).max(0.0);
            p.streak_origin = p.start_origin + vec_value(&wave, f64::from(behind));
        }
    }

    /// `ParticleEmitter::DoPointFile`: the particle sits on its point, taken in file order (or at
    /// random when its keyframe flags say so).
    pub(crate) fn do_point_file(&mut self, context: &mut SystemContext<'_>, p: &mut Particle) {
        let Some(key) = p.key.last[3] else {
            return;
        };
        let system = context.desc;
        let points = &system.emitters[self.index].keyframes[self.keys[key].desc].points;
        if points.is_empty() {
            p.origin = p.start_origin;
        } else {
            let count = points.len() as u32;
            if p.point_index == 0xFFFF {
                p.point_index = self.keys[key].next_point_index;
                self.keys[key].next_point_index = if p.keyframe_flags & 8 == 0 {
                    ((u32::from(self.keys[key].next_point_index) + 1) % count) as u16
                } else {
                    context.rng.roll_dice(0, count as i32 - 1) as u16
                };
            }
            let point = points[usize::from(p.point_index) % points.len()];
            p.origin = context.frame.local_to_global_vector(point) + p.start_origin;
        }
        if self.flags & flag::STREAK_ORIGIN != 0 {
            p.streak_origin = p.origin;
        }
    }

    /// `ParticleEmitter::DoPointFileLerp`: the particle moves along the point file at its point
    /// speed. As retail compares the distance it may travel with the squared distance to the
    /// next point, so does this.
    pub(crate) fn point_file_lerp(
        &mut self,
        context: &mut SystemContext<'_>,
        p: &mut Particle,
        quantum: f32,
    ) {
        let Some(key) = p.key.last[3] else {
            return;
        };
        let system = context.desc;
        let points = &system.emitters[self.index].keyframes[self.keys[key].desc].points;
        if points.is_empty() {
            p.origin = p.start_origin;
            return;
        }
        let count = points.len();
        if p.point_index == 0xFFFF {
            p.point_index = if p.keyframe_flags & 8 != 0 {
                context.rng.roll_dice(0, count as i32 - 1) as u16
            } else {
                0
            };
            self.do_point_file(context, p);
        }
        let mut budget = p.point_speed * quantum;
        let mut index = usize::from(p.point_index) % count;
        while budget > EPSILON {
            let next = context.frame.local_to_global_vector(points[index]) + p.start_origin;
            let to_next = next - p.origin;
            let distance_squared = to_next.length_squared();
            if budget <= distance_squared {
                p.origin = if distance_squared.sqrt() < EPSILON {
                    next
                } else {
                    p.origin + to_next / distance_squared.sqrt() * budget.sqrt()
                };
                budget = 0.0;
            } else {
                p.origin = next;
                budget -= distance_squared;
            }
            p.point_index = index as u16;
            index = (index + 1) % count;
        }
    }
}
