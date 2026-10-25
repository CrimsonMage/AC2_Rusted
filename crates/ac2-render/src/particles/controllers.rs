//! The motion controllers a keyframe selects (`PEKeyframeDesc::m_PCType`): physics under
//! gravity and wind, waveforms, and point files (`ParticleEmitter::DoPhysics`, `DoWaveform`,
//! `DoPointFile`, `DoPointFileLerp`).

use ac2_assets::Ac2ParticleEmitterDesc;
use ac2_math::Vec3;

use super::emitter::{vec_value, Emitter};
use super::keyframes::{flag, Particle};
use super::{SystemContext, EPSILON};

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
