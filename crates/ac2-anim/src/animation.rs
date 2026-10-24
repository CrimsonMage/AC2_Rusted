//! Asheron's Call 2 animation: `AnimMap` (`0x03`, motion -> `DBAnimator`) and `DBAnimator`
//! (`0x05`) keyframe assets (`AnimAsset::StreamPackInternal`: period, then per bone scale, offset,
//! rotation and weight keyframe tables). Switch animators (`AMSwitchME`) are followed to their
//! first asset.

use std::sync::Arc;

use ac2_dat::ArchiveCursor;
use ac2_dat::DatFileKind;
use ac2_core::{DataId, QualifiedDataId};
use ac2_math::{Quat, Vec3};
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};

use crate::setup::Ac2Transform;

const ANIMATOR_PREFIX: u32 = 0x0500_0000;
const ANIM_ASSET: u32 = 0x4000_0001;
const SWITCH_ANIMATOR: u32 = 0x4000_0008;
const MAX_BONES: usize = 256;
const MAX_KEYS: usize = 1 << 14;
const MAX_SWITCH_DEPTH: usize = 4;

/// Keyframe tracks of one bone: absolute local transform components over time.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2BoneTrack {
    pub scale: Vec<(f32, Vec3)>,
    pub offset: Vec<(f32, Vec3)>,
    pub rotation: Vec<(f32, Quat)>,
}

/// A decoded keyframe animation (`AnimAsset`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2Animation {
    pub period: f32,
    pub bones: Arc<[Ac2BoneTrack]>,
}

impl Ac2Animation {
    /// Local bone transforms at `time` (wrapped into the period). Components a bone has no keys
    /// for keep the rest-pose value from `rest`.
    pub fn sample(&self, time: f32, rest: &[Ac2Transform]) -> Vec<Ac2Transform> {
        let time = if self.period > 0.0 {
            time.rem_euclid(self.period)
        } else {
            0.0
        };
        rest.iter()
            .enumerate()
            .map(|(index, rest)| {
                let Some(track) = self.bones.get(index) else {
                    return *rest;
                };
                Ac2Transform {
                    offset: sample_vec(&track.offset, time).unwrap_or(rest.offset),
                    scale: sample_vec(&track.scale, time).unwrap_or(rest.scale),
                    rotation: sample_quat(&track.rotation, time).unwrap_or(rest.rotation),
                }
            })
            .collect()
    }
}

impl ResourcePackedDecode for Ac2Animation {
    const TYPE_NAME: &'static str = "AC2 AnimAsset";
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
        if cursor.read_u32()? != ANIM_ASSET {
            return Err(malformed("animator is not a keyframe asset"));
        }
        let period = cursor.read_f32()?;
        let bone_count = cursor.read_u32_count()?;
        if bone_count > MAX_BONES {
            return Err(malformed("bone count"));
        }
        let mut bones = Vec::with_capacity(bone_count);
        for _ in 0..bone_count {
            let mut track = Ac2BoneTrack::default();
            for _ in 0..keys(&mut cursor, 16).ok_or_else(|| malformed("scale keys"))? {
                let value = read_vec(&mut cursor)?;
                track.scale.push((cursor.read_f32()?, value));
            }
            for _ in 0..keys(&mut cursor, 16).ok_or_else(|| malformed("offset keys"))? {
                let value = read_vec(&mut cursor)?;
                track.offset.push((cursor.read_f32()?, value));
            }
            for _ in 0..keys(&mut cursor, 20).ok_or_else(|| malformed("rotation keys"))? {
                let w = cursor.read_f32()?;
                let rotation = Quat::from_xyzw(
                    cursor.read_f32()?,
                    cursor.read_f32()?,
                    cursor.read_f32()?,
                    w,
                );
                let time = cursor.read_f32()?;
                track.rotation.push((time, rotation.normalize()));
            }
            // Blend weight keys (weight, time) do not affect a single clip's pose.
            let weights = keys(&mut cursor, 8).ok_or_else(|| malformed("weight keys"))?;
            cursor.read_bytes(weights * 8)?;
            bones.push(track);
        }
        // An optional `AnimInfo` (hooks and events) follows; it does not shape the pose.
        Ok(Self {
            period,
            bones: Arc::from(bones),
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .bones
                .iter()
                .map(|bone| (bone.scale.len() + bone.offset.len()) * 16 + bone.rotation.len() * 20)
                .sum::<usize>()
    }
}

/// `AnimMap`: motion key -> `DBAnimator` DID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ac2AnimMap {
    pub entries: Arc<[(u32, DataId)]>,
}

impl Ac2AnimMap {
    pub fn animator(&self, motion: u32) -> Option<DataId> {
        self.entries
            .iter()
            .find(|(key, _)| *key == motion)
            .map(|(_, did)| *did)
    }
}

impl ResourcePackedDecode for Ac2AnimMap {
    const TYPE_NAME: &'static str = "AC2 AnimMap";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context: "embedded DID did not match DAT identity",
            });
        }
        let count = (cursor.read_u32()? & 0xFFFF) as usize;
        let entries = (0..count)
            .map(|_| Ok((cursor.read_u32()?, DataId::new(cursor.read_u32()?))))
            .collect::<ResourceResult<Arc<[_]>>>()?;
        Ok(Self { entries })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.entries.len() * 8
    }
}

/// Loads the keyframe animation an `AnimMap` plays for `motion`, following switch animators.
pub fn load_ac2_motion_animation(
    store: &ResourceStore,
    anim_map: DataId,
    motion: u32,
) -> ResourceResult<Option<Arc<Ac2Animation>>> {
    let Some(map) = load_portal::<Ac2AnimMap>(store, anim_map)? else {
        return Ok(None);
    };
    let Some(mut animator) = map.animator(motion) else {
        return Ok(None);
    };
    for _ in 0..MAX_SWITCH_DEPTH {
        let Some(header) = load_portal::<Ac2AnimatorHeader>(store, animator)? else {
            return Ok(None);
        };
        match (header.kind, header.first_child) {
            (ANIM_ASSET, _) => {
                return Ok(load_portal::<Ac2Animation>(store, animator)?
                    .map(|animation| Arc::new((*animation).clone())));
            }
            (SWITCH_ANIMATOR, Some(child)) => animator = child,
            _ => return Ok(None),
        }
    }
    Ok(None)
}

/// `DBAnimator` type word and, for switch animators, the first animator they select.
#[derive(Debug)]
struct Ac2AnimatorHeader {
    kind: u32,
    first_child: Option<DataId>,
}

impl ResourcePackedDecode for Ac2AnimatorHeader {
    const TYPE_NAME: &'static str = "AC2 DBAnimator header";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        _qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let words = bytes
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]));
        let kind = words.clone().nth(1).unwrap_or(0);
        let first_child = (kind == SWITCH_ANIMATOR)
            .then(|| {
                words
                    .skip(2)
                    .find(|word| word & 0xFF00_0000 == ANIMATOR_PREFIX)
            })
            .flatten()
            .map(DataId::new);
        Ok(Self { kind, first_child })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

fn keys(cursor: &mut ArchiveCursor<'_>, item: usize) -> Option<usize> {
    let count = cursor.read_u32_count().ok()?;
    (count <= MAX_KEYS && count * item <= cursor.remaining()).then_some(count)
}

fn read_vec(cursor: &mut ArchiveCursor<'_>) -> ResourceResult<Vec3> {
    Ok(Vec3::new(
        cursor.read_f32()?,
        cursor.read_f32()?,
        cursor.read_f32()?,
    ))
}

fn bracket<T: Copy>(keys: &[(f32, T)], time: f32) -> Option<(T, T, f32)> {
    let first = keys.first()?;
    if keys.len() == 1 || time <= first.0 {
        return Some((first.1, first.1, 0.0));
    }
    for pair in keys.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if time <= b.0 {
            let span = (b.0 - a.0).max(f32::EPSILON);
            return Some((a.1, b.1, ((time - a.0) / span).clamp(0.0, 1.0)));
        }
    }
    let last = keys[keys.len() - 1].1;
    Some((last, last, 0.0))
}

fn sample_vec(keys: &[(f32, Vec3)], time: f32) -> Option<Vec3> {
    bracket(keys, time).map(|(a, b, t)| a.lerp(b, t))
}

fn sample_quat(keys: &[(f32, Quat)], time: f32) -> Option<Quat> {
    bracket(keys, time).map(|(a, b, t)| a.slerp(b, t))
}

pub(crate) fn load_portal<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<ac2_db::CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}
