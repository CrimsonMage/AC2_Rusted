//! `ParticleKeyframe`, `ParticleKeyframeInfo` and `ParticleWrapper`: a particle's state, and the
//! keyframes that change its scale, colour, mass and motion controller over its life
//! (`ParticleEmitter::SetupKeyframes`, `ApplyKeyframes`, `ApplyKeyframe`, `GetMass`).

use ac2_assets::{Ac2ParticleEmitterDesc, Ac2WaveformVec3};
use ac2_math::Vec3;

use super::emitter::Emitter;
use super::{SystemContext, EPSILON};

/// `PEKeyframeDesc::m_KeyFlags`.
pub(crate) const KEY_SCALE_INTERPOLATES: u32 = 0x1;
pub(crate) const KEY_COLOR_INTERPOLATES: u32 = 0x2;
pub(crate) const KEY_MASS_INTERPOLATES: u32 = 0x4;
pub(crate) const KEY_SETS_SCALE: u32 = 0x8;
pub(crate) const KEY_SETS_COLOR: u32 = 0x10;
pub(crate) const KEY_SETS_MASS: u32 = 0x20;
pub(crate) const KEY_SETS_CONTROLLER: u32 = 0x80;
pub(crate) const KEY_SETS_FORCES: u32 = 0x100;

/// `PEKeyframeDesc::m_PCType`: the motion controller a keyframe starts.
pub(crate) const CONTROLLER_PHYSICS: u32 = 1;
pub(crate) const CONTROLLER_WAVEFORM: u32 = 2;
pub(crate) const CONTROLLER_REPULSE: u32 = 3;
pub(crate) const CONTROLLER_POINT_FILE: u32 = 4;
pub(crate) const CONTROLLER_POINT_FILE_LERP: u32 = 5;

/// `ParticleFormatInfo::m_flags`: which channels a particle of an emitter carries.
pub(crate) mod flag {
    pub const TIME_CREATED: u32 = 0x2;
    pub const START_ORIGIN: u32 = 0x4;
    pub const PREVIOUS_ORIGIN: u32 = 0x10;
    pub const VELOCITY: u32 = 0x20;
    pub const KEYFRAME_FLAGS: u32 = 0x40;
    pub const KEYFRAME_INFO: u32 = 0x80;
    pub const SCALE: u32 = 0x200;
    pub const ROTATION: u32 = 0x400;
    pub const START_ROTATION: u32 = 0x800;
    pub const ROTATE_VELOCITY: u32 = 0x1000;
    pub const STREAK: u32 = 0x2000;
    pub const STREAK_ORIGIN: u32 = 0x4000;
    pub const PREVIOUS_STREAK_ORIGIN: u32 = 0x8000;
    pub const WORLD_ROTATION: u32 = 0x10000;
    pub const START_WORLD_ROTATION: u32 = 0x20000;
    pub const POINT_INDEX: u32 = 0x40000;
    pub const POINT_SPEED: u32 = 0x80000;
    pub const WORLD_ORIGIN: u32 = 0x100000;
}

/// One entry of `m_aKeyframes`: a keyframe of the emitter's description and the next keys that
/// carry a scale (0), mass (1) and colour (2) interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KeyEntry {
    pub desc: usize,
    pub next: [usize; 3],
    /// `m_nNextPointIndex`: where the next particle takes the point file.
    pub next_point_index: u16,
}

/// `ParticleKeyframeInfo`: the last keys applied to a particle for scale, mass, colour and the
/// motion controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct KeyInfo {
    pub current: Option<usize>,
    pub last: [Option<usize>; 4],
}

/// One particle (`ParticleWrapper` over a packed `ParticleBuffer` entry).
#[derive(Debug, Clone)]
pub(crate) struct Particle {
    pub time_created: f64,
    pub start_origin: Vec3,
    pub world_origin: Vec3,
    pub origin: Vec3,
    pub previous_origin: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec3,
    pub start_rotation: Vec3,
    pub rotate_velocity: Vec3,
    pub world_rotation: Vec3,
    pub start_world_rotation: Vec3,
    pub keyframe_flags: u32,
    pub key: KeyInfo,
    pub waveform: Option<Ac2WaveformVec3>,
    pub streak: f32,
    pub streak_origin: Vec3,
    pub previous_streak_origin: Vec3,
    pub scale: f32,
    pub point_index: u16,
    pub point_speed: f32,
}

impl Particle {
    pub(crate) fn new() -> Self {
        Self {
            time_created: 0.0,
            start_origin: Vec3::ZERO,
            world_origin: Vec3::ZERO,
            origin: Vec3::ZERO,
            previous_origin: Vec3::ZERO,
            velocity: Vec3::ZERO,
            rotation: Vec3::ZERO,
            start_rotation: Vec3::ZERO,
            rotate_velocity: Vec3::ZERO,
            world_rotation: Vec3::ZERO,
            start_world_rotation: Vec3::ZERO,
            keyframe_flags: 0,
            key: KeyInfo {
                current: None,
                last: [None; 4],
            },
            waveform: None,
            streak: 0.0,
            streak_origin: Vec3::ZERO,
            previous_streak_origin: Vec3::ZERO,
            scale: 1.0,
            point_index: 0xFFFF,
            point_speed: 0.0,
        }
    }
}

/// `ParticleEmitter::SetupKeyframes`: one entry per keyframe, each pointing at the next key that
/// interpolates scale, mass and colour (itself when there is none).
pub(crate) fn setup_keyframes(desc: &Ac2ParticleEmitterDesc) -> Vec<KeyEntry> {
    let count = desc.keyframes.len();
    let mut keys = Vec::with_capacity(count);
    if count == 0 {
        return keys;
    }
    let find = |from: usize, bits: u32| {
        desc.keyframes[from..]
            .iter()
            .position(|key| key.key_flags & bits != 0)
            .map(|offset| from + offset)
    };
    if count != 1 {
        for index in 0..count - 1 {
            keys.push(KeyEntry {
                desc: index,
                next: [
                    find(index + 1, KEY_SCALE_INTERPOLATES).unwrap_or(index),
                    find(index + 1, KEY_MASS_INTERPOLATES).unwrap_or(index),
                    find(index + 1, KEY_COLOR_INTERPOLATES).unwrap_or(index),
                ],
                next_point_index: 0,
            });
        }
    }
    keys.push(KeyEntry {
        desc: count - 1,
        next: [count - 1; 3],
        next_point_index: 0,
    });
    keys
}

/// The interpolation fraction between two keys at `time_alive`: 0 before the first, 1 at or
/// after the second, and 1 when they are less than `F_EPSILON` apart.
pub(crate) fn blend(from_time: f32, to_time: f32, time_alive: f32) -> f32 {
    let span = to_time - from_time;
    if span <= EPSILON {
        return 1.0;
    }
    ((time_alive - from_time) / span).clamp(0.0, 1.0)
}

impl Emitter {
    /// `ParticleEmitter::GetMass`: the mass between the last mass key and its next.
    pub(crate) fn mass(&self, desc: &Ac2ParticleEmitterDesc, time_alive: f32, p: &Particle) -> f32 {
        if self.flags & flag::KEYFRAME_INFO == 0 {
            return 1.0;
        }
        let Some(last) = p.key.last[1] else {
            return 1.0;
        };
        let from = &desc.keyframes[self.keys[last].desc];
        let to = &desc.keyframes[self.keys[self.keys[last].next[1]].desc];
        let span = to.time - from.time;
        let fraction = if span.abs() >= EPSILON {
            ((time_alive - from.time) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        to.mass * fraction + (1.0 - fraction) * from.mass
    }

    /// `ParticleEmitter::ApplyKeyframes`: applies every key the particle has reached.
    pub(crate) fn apply_keyframes(
        &mut self,
        context: &mut SystemContext<'_>,
        p: &mut Particle,
        now: f64,
    ) {
        if self.flags & flag::KEYFRAME_INFO == 0 {
            return;
        }
        let time_alive = ((now - p.time_created) as f32).max(0.0);
        let mut index = p.key.current.map_or(0, |current| current + 1);
        let system = context.desc;
        let desc = &system.emitters[self.index];
        while index < self.keys.len() {
            let key_time = desc.keyframes[self.keys[index].desc].time;
            if time_alive < key_time {
                p.key.current = index.checked_sub(1);
                break;
            }
            self.apply_keyframe(context, p, now, index);
            index += 1;
        }
        if index == self.keys.len() {
            p.key.current = Some(self.keys.len() - 1);
        }
    }

    /// `ParticleEmitter::ApplyKeyframe`: the keyframe `key` takes effect on the particle. A
    /// keyframe that starts a motion controller first moves the start origin to where the
    /// previous controller had taken the particle, so the new one continues from there.
    fn apply_keyframe(
        &mut self,
        context: &mut SystemContext<'_>,
        p: &mut Particle,
        now: f64,
        key: usize,
    ) {
        let system = context.desc;
        let emitter = &system.emitters[self.index];
        let keyframe = &emitter.keyframes[self.keys[key].desc];
        if self.flags & flag::KEYFRAME_INFO != 0 {
            if keyframe.key_flags & KEY_SETS_CONTROLLER != 0 {
                if let Some(previous) = p.key.last[3] {
                    self.finish_controller(context, p, now, previous, key);
                }
                if keyframe.controller == CONTROLLER_WAVEFORM {
                    p.waveform = Some(keyframe.position);
                }
                p.key.last[3] = Some(key);
            }
            if keyframe.key_flags & (KEY_SETS_SCALE | KEY_SCALE_INTERPOLATES) != 0 {
                p.key.last[0] = Some(key);
            }
            if keyframe.key_flags & (KEY_SETS_COLOR | KEY_COLOR_INTERPOLATES) != 0 {
                p.key.last[2] = Some(key);
            }
            if keyframe.key_flags & (KEY_SETS_MASS | KEY_MASS_INTERPOLATES) != 0 {
                p.key.last[1] = Some(key);
            }
        }
        if self.flags & flag::KEYFRAME_FLAGS != 0 && keyframe.key_flags & KEY_SETS_FORCES != 0 {
            p.keyframe_flags = keyframe.force_flags;
        }
    }

    /// The part of `ApplyKeyframe` that carries the finished controller's motion into the start
    /// origin (and the rotation it accumulated into the start rotation).
    fn finish_controller(
        &mut self,
        context: &mut SystemContext<'_>,
        p: &mut Particle,
        now: f64,
        previous: usize,
        next: usize,
    ) {
        let system = context.desc;
        let emitter = &system.emitters[self.index];
        let previous_desc = &emitter.keyframes[self.keys[previous].desc];
        let dt = emitter.keyframes[self.keys[next].desc].time - previous_desc.time;
        match previous_desc.controller {
            CONTROLLER_PHYSICS => {
                let mass = p.key.last[1].map_or(1.0, |m| emitter.keyframes[self.keys[m].desc].mass);
                let mut acceleration = Vec3::ZERO;
                if p.keyframe_flags & 1 != 0 {
                    acceleration += self.gravity * mass;
                }
                if p.keyframe_flags & 2 != 0 {
                    acceleration += self.wind * mass;
                }
                let displacement = p.velocity * dt + acceleration * (0.5 * dt * dt);
                p.start_origin += displacement;
            }
            CONTROLLER_WAVEFORM => {
                if let Some(wave) = &p.waveform {
                    let t = f64::from(dt);
                    p.start_origin += Vec3::new(wave.x.value(t), wave.y.value(t), wave.z.value(t));
                }
            }
            CONTROLLER_POINT_FILE if !previous_desc.points.is_empty() => {
                let count = previous_desc.points.len();
                if p.point_index == 0xFFFF {
                    p.point_index = self.keys[previous].next_point_index;
                    self.keys[previous].next_point_index =
                        ((u32::from(self.keys[previous].next_point_index) + 1) % count as u32)
                            as u16;
                }
                let local = previous_desc.points[usize::from(p.point_index) % count];
                p.start_origin += context.frame.local_to_global_vector(local);
            }
            CONTROLLER_POINT_FILE_LERP => {
                self.point_file_lerp(context, p, (now - self.last_think) as f32);
                p.start_origin = p.origin;
            }
            _ => {}
        }
        if self.flags & flag::ROTATE_VELOCITY != 0 {
            let turned = p.rotate_velocity * dt;
            if self.flags & flag::START_WORLD_ROTATION != 0 {
                p.start_world_rotation += turned;
            } else if self.flags & flag::START_ROTATION != 0 {
                p.start_rotation += turned;
            }
        }
    }
}
