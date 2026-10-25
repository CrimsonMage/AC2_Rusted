//! The ambient sound records: `CSoundDesc` (`0x10`, the region's ambient sound types) with its
//! `CAmbientSoundInfo`s, each naming an `EnvInfo` (`0x29`, the reverb) and its continuous and
//! intermittent sounds. [`AmbientData`] is everything the ambient sound reads, loaded on the
//! loader thread.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};

use crate::data::{flags, MusicInfo, SoundInfo};

const MAX_ITEMS: usize = 4096;

/// `CContinSoundInfo`: a sound that plays while its environment is around.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinSoundInfo {
    /// `m_bitField`: 1 a `SoundInfo`, 2 a `MusicInfo`, 4 only while dominant, 8 never stopped
    /// when the environment fades, 0x10 four placed copies.
    pub bit_field: u32,
    /// `m_obj_did`.
    pub sound: u32,
    pub volume: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl ContinSoundInfo {
    /// `CContinSoundInfo::TestBitMask`.
    pub fn test(&self, mask: u32) -> bool {
        self.bit_field & mask != 0
    }
}

/// `CIntermitSoundInfo`: a sound fired now and then around the player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntermitSoundInfo {
    /// `m_bitField`: 1 a `SoundInfo`, 2 a `MusicInfo`, 4 only while dominant.
    pub bit_field: u32,
    pub sound: u32,
    pub volume: f32,
    pub probability: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub min_time: f32,
    pub max_time: f32,
}

impl IntermitSoundInfo {
    pub fn test(&self, mask: u32) -> bool {
        self.bit_field & mask != 0
    }
}

/// `CAmbientSoundInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct AmbientSoundInfo {
    /// `m_sound_type`: the key cells and land points name.
    pub sound_type: u32,
    /// `m_env_did`: the `EnvInfo` reverb of the environment.
    pub env: u32,
    pub intermit: Vec<IntermitSoundInfo>,
    pub contin: Vec<ContinSoundInfo>,
}

/// `CSoundDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct SoundDesc {
    pub sounds: Vec<AmbientSoundInfo>,
}

impl SoundDesc {
    /// `CSoundDesc::InqSoundInfo`: the first entry of `sound_type`.
    pub fn sound_info(&self, sound_type: u32) -> Option<&AmbientSoundInfo> {
        self.sounds.iter().find(|s| s.sound_type == sound_type)
    }
}

fn count(cursor: &mut ArchiveCursor<'_>) -> ResourceResult<usize> {
    let n = cursor.read_u32()? as usize;
    Ok(n.min(MAX_ITEMS + 1))
}

impl ResourcePackedDecode for SoundDesc {
    const TYPE_NAME: &'static str = "AC2 SoundDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    /// The header (DID, then a zero word), then `PSmartArray<CAmbientSoundInfo>`:
    /// `CAmbientSoundInfo::StreamPack` (type, env, intermittent sounds, continuous sounds).
    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let bad = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(bad("embedded DID did not match DAT identity"));
        }
        cursor.read_u32()?;
        let n = count(&mut cursor)?;
        if n > MAX_ITEMS {
            return Err(bad("too many sound types"));
        }
        let mut sounds = Vec::with_capacity(n);
        for _ in 0..n {
            let sound_type = cursor.read_u32()?;
            let env = cursor.read_u32()?;
            let mut intermit = Vec::new();
            for _ in 0..count(&mut cursor)? {
                intermit.push(IntermitSoundInfo {
                    bit_field: cursor.read_u32()?,
                    sound: cursor.read_u32()?,
                    volume: cursor.read_f32()?,
                    probability: cursor.read_f32()?,
                    min_pitch: cursor.read_f32()?,
                    max_pitch: cursor.read_f32()?,
                    min_height: cursor.read_f32()?,
                    max_height: cursor.read_f32()?,
                    min_time: cursor.read_f32()?,
                    max_time: cursor.read_f32()?,
                });
            }
            let mut contin = Vec::new();
            for _ in 0..count(&mut cursor)? {
                contin.push(ContinSoundInfo {
                    bit_field: cursor.read_u32()?,
                    sound: cursor.read_u32()?,
                    volume: cursor.read_f32()?,
                    min_pitch: cursor.read_f32()?,
                    max_pitch: cursor.read_f32()?,
                    min_height: cursor.read_f32()?,
                    max_height: cursor.read_f32()?,
                });
            }
            sounds.push(AmbientSoundInfo {
                sound_type,
                env,
                intermit,
                contin,
            });
        }
        if cursor.remaining() != 0 {
            return Err(bad("trailing bytes after SoundDesc"));
        }
        Ok(Self { sounds })
    }

    fn cache_retained_bytes(&self) -> usize {
        self.sounds.len() * std::mem::size_of::<AmbientSoundInfo>()
    }
}

/// The ambient sound's data: the region's `CSoundDesc` and, for each sound it names, whether it
/// is placed in 3D (`ContinMusicObject::Is3D` / `IntermitMusicObject::Is3D`: the `MusicInfo`'s
/// `0x10` bit; `ContinSoundObject::Is3D` / `IntermitSoundObject::Is3D`: the `SoundInfo`'s `4`).
#[derive(Debug, Clone, PartialEq)]
pub struct AmbientData {
    pub desc: SoundDesc,
    three_d: HashMap<u32, bool>,
}

impl AmbientData {
    pub fn new(desc: SoundDesc, three_d: HashMap<u32, bool>) -> Self {
        Self { desc, three_d }
    }

    /// The region's `CSoundDesc`: its sound types and their continuous and intermittent sounds.
    pub fn desc(&self) -> &SoundDesc {
        &self.desc
    }

    /// The sound's `Is3D` (false when its record is missing, as retail's missing-data path).
    pub fn is_3d(&self, sound: u32) -> bool {
        self.three_d.get(&sound).copied().unwrap_or(false)
    }

    /// Reads the region's `CSoundDesc` `did` and the records of the sounds it names.
    pub fn load(store: &ResourceStore, did: u32) -> ResourceResult<Option<Arc<Self>>> {
        let get = |did: u32| TypedResourceRef::new(DatFileKind::Portal, DataId::new(did), 0);
        let Some(desc) = store.load_cached_packed::<SoundDesc>(get(did))? else {
            return Ok(None);
        };
        let desc = (*desc).clone();
        let mut three_d = HashMap::new();
        let mut sounds = Vec::new();
        for info in &desc.sounds {
            sounds.extend(info.contin.iter().map(|c| (c.sound, c.test(2))));
            sounds.extend(info.intermit.iter().map(|c| (c.sound, c.test(2))));
        }
        for (sound, music) in sounds {
            let is_3d = if music {
                store
                    .load_cached_packed::<MusicInfo>(get(sound))?
                    .is_some_and(|m| m.test(0x10))
            } else {
                store
                    .load_cached_packed::<SoundInfo>(get(sound))?
                    .is_some_and(|s| s.test(flags::THREE_D))
            };
            three_d.insert(sound, is_3d);
        }
        Ok(Some(Arc::new(Self { desc, three_d })))
    }
}
