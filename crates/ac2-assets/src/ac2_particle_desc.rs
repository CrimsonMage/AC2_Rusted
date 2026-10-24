//! Asheron's Call 2 particle systems: `PSDesc` (`0x39`, `DB_TYPE_PSDESC`) with its `PEmitterDesc`
//! emitters and their `PEKeyframeDesc` keyframes, decoded in `PSDesc::StreamPack` order. Every Legions record decodes to its exact length.

use std::sync::Arc;

use ac2_dat::ArchiveCursor;
use ac2_core::{DataId, QualifiedDataId};
use ac2_math::Vec3;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

use crate::ac2_material::read_waveform;
use crate::Ac2Waveform;

const MAX_EMITTERS: usize = 64;
const MAX_KEYFRAMES: usize = 256;
const MAX_POINTS: usize = 1 << 16;

/// `WaveformVector3`: one waveform per axis.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ac2WaveformVec3 {
    pub x: Ac2Waveform,
    pub y: Ac2Waveform,
    pub z: Ac2Waveform,
}

/// `PEKeyframeDesc`: a particle-lifetime key. `key_flags` selects which channels it sets (`0x8`
/// scale, `0x10` colour, `0x20` mass, `0x80` motion controller, `0x100` force flags) and which
/// interpolate toward it (`0x1` scale, `0x2` colour, `0x4` mass).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2ParticleKeyframe {
    pub time: f32,
    pub key_flags: u32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: [f32; 4],
    pub mass: f32,
    /// Motion controller from this key on: 1 physics, 2 waveform, 3 repulse, 4 point file,
    /// 5 point-file interpolation.
    pub controller: u32,
    /// Force flags (`0x1` gravity, `0x2` wind) applied by the physics controller.
    pub force_flags: u32,
    pub position: Ac2WaveformVec3,
    pub points: Arc<[Vec3]>,
}

/// `PEmitterDesc`: one emitter of a particle system.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2ParticleEmitterDesc {
    pub max_particles: u32,
    pub origin: Ac2WaveformVec3,
    /// Emission shape: 0 point, 1 square, 2 circle, 3 sphere, 4 swept line.
    pub shape: u32,
    /// Particle primitive: 3 triangle, 4 quad.
    pub particle_shape: u32,
    pub scale: Ac2Waveform,
    pub particle_scale: Ac2Waveform,
    pub exploding_dir: bool,
    pub inclusive_shape: bool,
    pub active: bool,
    /// Seconds between blasts (`m_tNextBirthRateDischargeTime += m_BirthRate`).
    pub birth_rate: f32,
    pub lifespan: Ac2Waveform,
    pub velocity: Ac2Waveform,
    pub streak: Ac2Waveform,
    pub rotation: Ac2WaveformVec3,
    pub world_rotation: Ac2WaveformVec3,
    pub rotate_velocity: Ac2WaveformVec3,
    pub direction: Ac2WaveformVec3,
    pub min_spread: Ac2Waveform,
    pub max_spread: Ac2Waveform,
    pub emission_limit: u32,
    pub blast_count: u32,
    pub start_time: f32,
    pub time_limit: f32,
    pub emission_distance: f32,
    pub fade_in: f32,
    pub fade_out: f32,
    pub particle_snap: Ac2Waveform,
    pub constrain_axes: [bool; 3],
    pub keyframes: Arc<[Ac2ParticleKeyframe]>,
}

/// `PSDesc` (`0x39`): a particle system and the `MaterialInstance` its particles draw with.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2ParticleSystemDesc {
    pub max_particles: u32,
    /// `2` multiplies the object scale by the visual's particle scale.
    pub scale_type: u32,
    pub physics_time_step: f32,
    pub fast_forward_time: f32,
    pub physics_duration: f32,
    pub start_fade_distance: f32,
    pub stop_fade_distance: f32,
    pub world_space: bool,
    pub force_draw: bool,
    pub emitters: Arc<[Ac2ParticleEmitterDesc]>,
    pub material: Option<DataId>,
}

impl ResourcePackedDecode for Ac2ParticleSystemDesc {
    const TYPE_NAME: &'static str = "AC2 PSDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let max_particles = cursor.read_u32()?;
        let scale_type = cursor.read_u32()?;
        let physics_time_step = cursor.read_f32()?;
        let fast_forward_time = cursor.read_f32()?;
        let physics_duration = cursor.read_f32()?;
        let start_fade_distance = cursor.read_f32()?;
        let stop_fade_distance = cursor.read_f32()?;
        let world_space = cursor.read_u32()? != 0;
        let force_draw = cursor.read_u32()? != 0;
        let count = cursor.read_u32()? as usize;
        if count > MAX_EMITTERS {
            return Err(malformed("emitter count exceeded the record"));
        }
        let emitters = (0..count)
            .map(|_| read_emitter(&mut cursor, malformed))
            .collect::<ResourceResult<Vec<_>>>()?;
        let material = Some(DataId::new(cursor.read_u32()?)).filter(|did| !did.is_null());
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after PSDesc"));
        }
        Ok(Self {
            max_particles,
            scale_type,
            physics_time_step,
            fast_forward_time,
            physics_duration,
            start_fade_distance,
            stop_fade_distance,
            world_space,
            force_draw,
            emitters: Arc::from(emitters),
            material,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .emitters
                .iter()
                .map(|emitter| {
                    std::mem::size_of::<Ac2ParticleEmitterDesc>()
                        + emitter
                            .keyframes
                            .iter()
                            .map(|key| {
                                std::mem::size_of::<Ac2ParticleKeyframe>() + key.points.len() * 12
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}

/// The emitter fields in `PSDesc::StreamPack` stream order (not `PEmitterDesc` layout order).
fn read_emitter(
    cursor: &mut ArchiveCursor<'_>,
    malformed: impl Fn(&'static str) -> ResourceError + Copy,
) -> ResourceResult<Ac2ParticleEmitterDesc> {
    let max_particles = cursor.read_u32()?;
    let origin = read_vec3(cursor)?;
    let shape = cursor.read_u32()?;
    let particle_shape = cursor.read_u32()?;
    let scale = read_waveform(cursor)?;
    let particle_scale = read_waveform(cursor)?;
    let exploding_dir = cursor.read_u32()? != 0;
    let inclusive_shape = cursor.read_u32()? != 0;
    let active = cursor.read_u32()? != 0;
    let birth_rate = cursor.read_f32()?;
    let lifespan = read_waveform(cursor)?;
    let velocity = read_waveform(cursor)?;
    let streak = read_waveform(cursor)?;
    let rotation = read_vec3(cursor)?;
    let world_rotation = read_vec3(cursor)?;
    let rotate_velocity = read_vec3(cursor)?;
    let direction = read_vec3(cursor)?;
    let min_spread = read_waveform(cursor)?;
    let max_spread = read_waveform(cursor)?;
    let emission_limit = cursor.read_u32()?;
    let blast_count = cursor.read_u32()?;
    let start_time = cursor.read_f32()?;
    let time_limit = cursor.read_f32()?;
    let emission_distance = cursor.read_f32()?;
    let fade_in = cursor.read_f32()?;
    let fade_out = cursor.read_f32()?;
    let particle_snap = read_waveform(cursor)?;
    let constrain_axes = [
        cursor.read_u32()? != 0,
        cursor.read_u32()? != 0,
        cursor.read_u32()? != 0,
    ];
    let count = cursor.read_u32()? as usize;
    if count > MAX_KEYFRAMES {
        return Err(malformed("keyframe count exceeded the record"));
    }
    let keyframes = (0..count)
        .map(|_| read_keyframe(cursor, malformed))
        .collect::<ResourceResult<Vec<_>>>()?;
    Ok(Ac2ParticleEmitterDesc {
        max_particles,
        origin,
        shape,
        particle_shape,
        scale,
        particle_scale,
        exploding_dir,
        inclusive_shape,
        active,
        birth_rate,
        lifespan,
        velocity,
        streak,
        rotation,
        world_rotation,
        rotate_velocity,
        direction,
        min_spread,
        max_spread,
        emission_limit,
        blast_count,
        start_time,
        time_limit,
        emission_distance,
        fade_in,
        fade_out,
        particle_snap,
        constrain_axes,
        keyframes: Arc::from(keyframes),
    })
}

fn read_keyframe(
    cursor: &mut ArchiveCursor<'_>,
    malformed: impl Fn(&'static str) -> ResourceError,
) -> ResourceResult<Ac2ParticleKeyframe> {
    let time = cursor.read_f32()?;
    let key_flags = cursor.read_u32()?;
    let scale_x = cursor.read_f32()?;
    let scale_y = cursor.read_f32()?;
    let color = [
        cursor.read_f32()?,
        cursor.read_f32()?,
        cursor.read_f32()?,
        cursor.read_f32()?,
    ];
    let mass = cursor.read_f32()?;
    let controller = cursor.read_u32()?;
    let force_flags = cursor.read_u32()?;
    let position = read_vec3(cursor)?;
    let count = cursor.read_u32()? as usize;
    if count > MAX_POINTS || count * 12 > cursor.remaining() {
        return Err(malformed("keyframe point count exceeded the record"));
    }
    let points = (0..count)
        .map(|_| {
            Ok(Vec3::new(
                cursor.read_f32()?,
                cursor.read_f32()?,
                cursor.read_f32()?,
            ))
        })
        .collect::<ResourceResult<Vec<_>>>()?;
    // `STREAMPACK_ALIGN`: every keyframe field is a 4-byte word, so this never moves the cursor.
    cursor.align_to(4)?;
    Ok(Ac2ParticleKeyframe {
        time,
        key_flags,
        scale_x,
        scale_y,
        color,
        mass,
        controller,
        force_flags,
        position,
        points: Arc::from(points),
    })
}

fn read_vec3(cursor: &mut ArchiveCursor<'_>) -> ResourceResult<Ac2WaveformVec3> {
    Ok(Ac2WaveformVec3 {
        x: read_waveform(cursor)?,
        y: read_waveform(cursor)?,
        z: read_waveform(cursor)?,
    })
}
