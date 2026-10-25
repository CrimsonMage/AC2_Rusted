//! Asheron's Call 2 FX records: `FXNode`/`FXData` (embedded in `PartGroupDataDesc` FX overrides)
//! and `DBFXTable` (`0x1E`), decoded in `FXNode::StreamPack` / `DBFXTable::StreamPack` order.
//! The particle-emitter facts of each node are retained (`FXManager::DoParticleFX`), and its
//! sound facts (`FXManager::DoTargettedFX`'s sound branch); camera, appearance, streak, decal,
//! repulsor and light payloads are parsed to stay aligned.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::Vec3;

pub(crate) type Malformed = &'static str;
const MAX_ITEMS: usize = 4096;

/// The particle part of one `FXNode`: the `PSDesc` list (`GetParticleID` picks one at random) and
/// where the system attaches (`PHYSICSOBJREP::CreateParticleSystem`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxParticleNode {
    pub particle_descs: Arc<[DataId]>,
    /// `m_holding_location`: a Setup holding point id (takes precedence over the bone).
    pub holding_location: u32,
    /// `m_bone_name`: a Setup bone name (`CPartGroup::FindBoneType`).
    pub bone_name: u32,
    /// `m_bone_offset`, translated in the bone's (or the object's) space.
    pub bone_offset: Vec3,
    /// `m_remove_rotation`: keep only the attachment translation.
    pub remove_rotation: bool,
    /// `m_water_plane`: the system rides the cell's water surface.
    pub water_plane: bool,
}

/// One FX id's data list (`PSmartArray<FXData>`), reduced to its particle nodes.
pub type Ac2FxEntry = (u32, Arc<[Ac2FxParticleNode]>);

/// What `FXManager::DoTargettedFX` and `UseTime` read of one `FXNode` for sound. Every node is
/// kept (its sound list may be empty), as each node steps the FX's volume ramp.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxSoundNode {
    /// `m_sound_list`: `SoundInfo` / `MusicInfo` DIDs; `FXNode::GetSoundID` picks one.
    pub sounds: Arc<[u32]>,
    /// `m_ramp_time`: seconds for the volume to reach the FX's scalar.
    pub ramp_time: f32,
    /// `m_fire_and_forget`: a looping sound is not kept to be stopped with the FX.
    pub fire_and_forget: bool,
    /// `m_player_only`.
    pub player_only: bool,
    /// `m_day_time` / `m_night_time`: when the node plays.
    pub day_time: bool,
    pub night_time: bool,
    /// `m_channel_keys` is not empty (`FXManager::ChannelMissmatch` consults the channel prefs).
    pub has_channels: bool,
}

/// One FX id's sound nodes.
pub type Ac2FxSoundEntry = (u32, Arc<[Ac2FxSoundNode]>);

/// A decoded FX map: the particle view and the sound view of the same entries.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2FxMap {
    pub particles: Arc<[Ac2FxEntry]>,
    pub sounds: Arc<[Ac2FxSoundEntry]>,
}

/// `DBFXTable` (`0x1E`): FX id -> `FXData` list, plus sub-tables searched after this one.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2FxTable {
    pub entries: Arc<[Ac2FxEntry]>,
    pub sounds: Arc<[Ac2FxSoundEntry]>,
    pub sub_tables: Arc<[DataId]>,
}

impl ResourcePackedDecode for Ac2FxTable {
    const TYPE_NAME: &'static str = "AC2 DBFXTable";
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
        if word(&mut cursor).map_err(malformed)? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let map = read_fx_map(&mut cursor).map_err(malformed)?;
        let count = count(&mut cursor, 4).map_err(malformed)?;
        let sub_tables = (0..count)
            .map(|_| word(&mut cursor).map(DataId::new))
            .collect::<Result<Vec<_>, _>>()
            .map_err(malformed)?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after DBFXTable"));
        }
        Ok(Self {
            entries: map.particles,
            sounds: map.sounds,
            sub_tables: Arc::from(sub_tables),
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.sub_tables.len() * 4
            + self
                .entries
                .iter()
                .map(|(_, nodes)| 8 + nodes.len() * std::mem::size_of::<Ac2FxParticleNode>())
                .sum::<usize>()
            + self
                .sounds
                .iter()
                .map(|(_, nodes)| 8 + nodes.len() * std::mem::size_of::<Ac2FxSoundNode>())
                .sum::<usize>()
    }
}

impl Ac2FxTable {
    pub fn entry(&self, fx_id: u32) -> Option<&Arc<[Ac2FxParticleNode]>> {
        self.entries
            .iter()
            .find(|(id, _)| *id == fx_id)
            .map(|(_, nodes)| nodes)
    }
}

/// `PHashTable<ulong, PSmartArray<FXData>>`: the FX map of `DBFXTable` and part-group overrides.
/// Entries keep their id even when no node carries particles, so an override still hides the
/// static table (`FXTable::GetFXData`).
pub(crate) fn read_fx_map(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2FxMap, Malformed> {
    let mut particles = Vec::new();
    let mut sounds = Vec::new();
    for _ in 0..hash_count(cursor)? {
        let fx_id = word(cursor)?;
        let nodes = (0..count(cursor, 8)?)
            .map(|_| read_fx_data(cursor))
            .collect::<Result<Vec<_>, _>>()?;
        let (particle, sound): (Vec<_>, Vec<_>) = nodes.into_iter().unzip();
        particles.push((fx_id, particle.into_iter().flatten().collect::<Arc<[_]>>()));
        sounds.push((fx_id, Arc::from(sound)));
    }
    Ok(Ac2FxMap {
        particles: Arc::from(particles),
        sounds: Arc::from(sounds),
    })
}

/// `FXData::StreamPack`: the default `FXNode`, then a terrain-keyed `FXNode` table. The default
/// node is what `FXData::GetData` returns off the keyed terrains.
pub(crate) fn read_fx_data(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<(Option<Ac2FxParticleNode>, Ac2FxSoundNode), Malformed> {
    let node = read_fx_node(cursor)?;
    for _ in 0..hash_count(cursor)? {
        word(cursor)?;
        read_fx_node(cursor)?;
    }
    Ok(node)
}

/// `FXNode::StreamPack`: a 64-bit pack bitfield gating each field in stream order (absent
/// fields keep `FXNode::SetDefaults`).
fn read_fx_node(
    cursor: &mut ArchiveCursor<'_>,
) -> Result<(Option<Ac2FxParticleNode>, Ac2FxSoundNode), Malformed> {
    let bits = u64::from(word(cursor)?) | (u64::from(word(cursor)?) << 32);
    let has = |bit: u64| bits & bit != 0;
    let field = |cursor: &mut ArchiveCursor<'_>, bit: u64| -> Result<u32, Malformed> {
        if has(bit) {
            word(cursor)
        } else {
            Ok(0)
        }
    };
    let skip_words = |cursor: &mut ArchiveCursor<'_>, mask: &[u64]| -> Result<(), Malformed> {
        for bit in mask {
            field(cursor, *bit)?;
        }
        Ok(())
    };
    let mut sounds = Vec::new();
    if has(0x1) {
        for _ in 0..count(cursor, 4)? {
            sounds.push(word(cursor)?);
        }
    }
    if has(0x80_0000_0000) {
        // The music list.
        let n = count(cursor, 4)?;
        skip(cursor, n * 4)?;
    }
    let mut particle_descs = Vec::new();
    if has(0x2) {
        for _ in 0..count(cursor, 4)? {
            let did = word(cursor)?;
            if did != 0 {
                particle_descs.push(DataId::new(did));
            }
        }
    }
    // Camera FX and appearance ids; the appearance key is (key, scalar modifier).
    skip_words(cursor, &[0x4, 0x8])?;
    if has(0x10) {
        skip(cursor, 8)?;
    }
    // Ramp time and intensity.
    let ramp_time = if has(0x20) { float(cursor)? } else { 0.0 };
    skip_words(cursor, &[0x40])?;
    let holding_location = field(cursor, 0x80)?;
    let bone_name = field(cursor, 0x100)?;
    let bone_offset = if has(0x200) {
        Vec3::new(float(cursor)?, float(cursor)?, float(cursor)?)
    } else {
        Vec3::ZERO
    };
    if has(0x400) {
        // `m_has_streak`, then `StreakFX` (colour, two points, material, five scalars).
        skip(cursor, 4 + 13 * 4)?;
    }
    skip_words(cursor, &[0x800, 0x1000, 0x2000, 0x4000])?;
    if has(0x8000) {
        skip(cursor, 4 + 16)?;
    }
    skip_words(
        cursor,
        &[
            0x1_0000,
            0x2_0000,
            0x4_0000,
            0x20_0000_0000,
            0x8_0000,
            0x10_0000,
            0x20_0000,
            0x40_0000,
            0x80_0000,
            0x8_0000_0000,
            0x100_0000,
        ],
    )?;
    let flag = |cursor: &mut ArchiveCursor<'_>, bit: u64, default: bool| {
        Ok::<_, Malformed>(if has(bit) {
            word(cursor)? != 0
        } else {
            default
        })
    };
    let day_time = flag(cursor, 0x200_0000, true)?;
    let night_time = flag(cursor, 0x400_0000, true)?;
    let mut has_channels = false;
    if has(0x800_0000) {
        let pairs = hash_count(cursor)?;
        has_channels = pairs != 0;
        skip(cursor, pairs * 8)?;
    }
    let fire_and_forget = flag(cursor, 0x1000_0000, false)?;
    let water_plane = field(cursor, 0x2000_0000)? != 0;
    field(cursor, 0x4000_0000)?;
    if has(0x8000_0000) {
        // `m_has_decal`, then `DecalFX`: colour, material, origin, lifespan, three waveforms.
        skip(cursor, 4 + 4 + 4 + 12 + 4 + 3 * 44)?;
    }
    if has(0x2_0000_0000) {
        // `m_has_repulsor`, then `RepulsorFX`: acceleration, range, lifetime, attractor, speed.
        skip(cursor, 4 + 5 * 4)?;
    }
    if has(0x1_0000_0000) {
        // `m_has_light`, then `LightSourceFX`: an `EntityDesc`, lifespan and fade time.
        word(cursor)?;
        crate::ac2_entity::read_ac2_entity_desc(cursor)?;
        skip(cursor, 8)?;
    }
    let player_only = flag(cursor, 0x4_0000_0000, false)?;
    skip_words(cursor, &[0x10_0000_0000])?;
    let remove_rotation = field(cursor, 0x40_0000_0000)? != 0;
    let particle = (!particle_descs.is_empty()).then(|| Ac2FxParticleNode {
        particle_descs: Arc::from(particle_descs),
        holding_location,
        bone_name,
        bone_offset,
        remove_rotation,
        water_plane,
    });
    let sound = Ac2FxSoundNode {
        sounds: Arc::from(sounds),
        ramp_time,
        fire_and_forget,
        player_only,
        day_time,
        night_time,
        has_channels,
    };
    Ok((particle, sound))
}

/// Packed hash tables lead with a word whose low 16 bits are the entry count.
pub(crate) fn hash_count(cursor: &mut ArchiveCursor<'_>) -> Result<usize, Malformed> {
    let n = (word(cursor)? & 0xFFFF) as usize;
    if n * 4 > cursor.remaining() {
        return Err("hash entry count exceeded the record");
    }
    Ok(n)
}

pub(crate) fn count(cursor: &mut ArchiveCursor<'_>, item: usize) -> Result<usize, Malformed> {
    let n = word(cursor)? as usize;
    if n > MAX_ITEMS || n * item > cursor.remaining() {
        return Err("item count exceeded the record");
    }
    Ok(n)
}

pub(crate) fn word(cursor: &mut ArchiveCursor<'_>) -> Result<u32, Malformed> {
    cursor.read_u32().map_err(|_| "record ended early")
}

pub(crate) fn float(cursor: &mut ArchiveCursor<'_>) -> Result<f32, Malformed> {
    cursor.read_f32().map_err(|_| "record ended early")
}

pub(crate) fn skip(cursor: &mut ArchiveCursor<'_>, len: usize) -> Result<(), Malformed> {
    cursor
        .read_bytes(len)
        .map(|_| ())
        .map_err(|_| "record ended early")
}
