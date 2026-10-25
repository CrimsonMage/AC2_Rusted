//! Asheron's Call 2 `VisualDesc` (`0x1F`), decoded in `VisualDesc::StreamPack` order. Every field
//! is gated by the leading pack bitfield; part groups (`PartGroupDataDesc`) name the `CSetup` and
//! `AnimMap` of each visual part. The global part group's FX (`VisualDesc::FillFXTable` /
//! `GetStartupFX`) keeps its particle facts (`crate::ac2_fx`).

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::Vec3;

use crate::ac2_fx::{
    count, float, hash_count, read_fx_map, skip, word, Ac2FxEntry, Ac2FxSoundEntry, Malformed,
};

/// One visual part group: its key, parent group, connection point and authored resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ac2PartGroup {
    pub key: u32,
    pub parent_key: u32,
    pub connection_point: u32,
    pub setup: Option<DataId>,
    pub anim_map: Option<DataId>,
}

/// FX of the global part group (`m_global_app_mods`): the static `DBFXTable`, the startup FX ids
/// with their scalars, and the per-id overrides that shadow the static table.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2VisualFx {
    pub fx_table: Option<DataId>,
    pub startup: Arc<[(u32, f32)]>,
    pub overrides: Arc<[Ac2FxEntry]>,
    /// The overrides' sound nodes.
    pub sound_overrides: Arc<[Ac2FxSoundEntry]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ac2VisualDesc {
    pub parent: Option<DataId>,
    /// `m_midesc_did`: the `MotionInterpDesc` the object moves by.
    pub motion_interp: Option<DataId>,
    pub behavior_table: Option<DataId>,
    pub modes: Option<DataId>,
    pub scale: Option<Vec3>,
    pub child_scale: Option<f32>,
    /// `m_particle_scale` (`CPhysicsObj::GetParticleScale`).
    pub particle_scale: Option<f32>,
    pub part_groups: Arc<[Ac2PartGroup]>,
    /// `m_app_hash` of each part group, in `part_groups` order.
    pub part_group_appearance: Arc<[Ac2AppearanceHash]>,
    /// `m_global_app_mods`' appearance hash.
    pub global_appearance: Ac2AppearanceHash,
    pub fx: Ac2VisualFx,
}

impl Ac2VisualDesc {
    /// The root part group (no parent key) that names a Setup.
    pub fn root_setup(&self) -> Option<DataId> {
        self.part_groups
            .iter()
            .find(|group| group.parent_key == 0 && group.setup.is_some())
            .or_else(|| self.part_groups.iter().find(|group| group.setup.is_some()))
            .and_then(|group| group.setup)
    }
}

impl ResourcePackedDecode for Ac2VisualDesc {
    const TYPE_NAME: &'static str = "AC2 VisualDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        decode(&mut ArchiveCursor::new(bytes), qdid).map_err(|context| {
            ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context,
            }
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.part_groups.len() * std::mem::size_of::<Ac2PartGroup>()
            + self.fx.startup.len() * 8
            + self
                .fx
                .overrides
                .iter()
                .map(|(_, nodes)| 8 + nodes.len() * 64)
                .sum::<usize>()
    }
}

fn decode(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
) -> Result<Ac2VisualDesc, Malformed> {
    let bits = word(cursor)?;
    if bits & 1 != 0 && word(cursor)? != qdid.data_id().raw() {
        return Err("embedded DID did not match DAT identity");
    }
    let did = |cursor: &mut ArchiveCursor<'_>, bit: u32| -> Result<Option<DataId>, Malformed> {
        Ok(if bits & bit != 0 {
            Some(DataId::new(word(cursor)?)).filter(|did| !did.is_null())
        } else {
            None
        })
    };
    let parent = did(cursor, 0x2)?;
    let motion_interp = did(cursor, 0x4)?;
    let behavior_table = did(cursor, 0x8)?;
    let modes = did(cursor, 0x10)?;
    let scale = if bits & 0x20 != 0 {
        Some(Vec3::new(float(cursor)?, float(cursor)?, float(cursor)?))
    } else {
        None
    };
    let child_scale = if bits & 0x40 != 0 {
        Some(float(cursor)?)
    } else {
        None
    };
    let particle_scale = if bits & 0x400 != 0 {
        Some(float(cursor)?)
    } else {
        None
    };
    if bits & 0x80 != 0 {
        // `IconDesc`: layers of (layer id, image DID, shift colour).
        let layers = count(cursor, 12)?;
        skip(cursor, layers * 12)?;
    }
    let mut part_groups = Vec::new();
    let mut part_group_appearance = Vec::new();
    let mut fx = Ac2VisualFx::default();
    let mut global_appearance: Ac2AppearanceHash = Arc::from([]);
    if bits & 0x100 != 0 {
        // `m_global_app_mods` is a part group applied to every part; it names no Setup.
        let global = part_group(cursor)?;
        fx = global.fx;
        global_appearance = global.appearance;
    }
    if bits & 0x200 != 0 {
        for _ in 0..hash_count(cursor)? {
            word(cursor)?;
            let group = part_group(cursor)?;
            part_groups.push(group.group);
            part_group_appearance.push(group.appearance);
        }
    }
    if !cursor.remaining_bytes().is_empty() {
        return Err("trailing bytes after VisualDesc");
    }
    Ok(Ac2VisualDesc {
        parent,
        motion_interp,
        behavior_table,
        modes,
        scale,
        child_scale,
        particle_scale,
        part_groups: Arc::from(part_groups),
        part_group_appearance: Arc::from(part_group_appearance),
        global_appearance,
        fx,
    })
}

/// `AppearanceInfo` hash of a part group (`m_app_hash`): appearance table DID -> its applied
/// appearance keys and their scalars.
pub type Ac2AppearanceHash = Arc<[(DataId, Arc<[(u32, f32)]>)]>;

/// A decoded `PartGroupDataDesc`.
pub(crate) struct DecodedPartGroup {
    pub group: Ac2PartGroup,
    pub fx: Ac2VisualFx,
    pub appearance: Ac2AppearanceHash,
}

/// `PartGroupDataDesc::StreamPack`.
pub(crate) fn part_group(cursor: &mut ArchiveCursor<'_>) -> Result<DecodedPartGroup, Malformed> {
    let bits = word(cursor)?;
    let mut group = Ac2PartGroup::default();
    let mut field = |bit: u32| -> Result<u32, Malformed> {
        if bits & bit != 0 {
            word(cursor)
        } else {
            Ok(0)
        }
    };
    group.key = field(0x1)?;
    group.parent_key = field(0x2)?;
    group.connection_point = field(0x4)?;
    group.setup = Some(DataId::new(field(0x8)?)).filter(|did| !did.is_null());
    group.anim_map = Some(DataId::new(field(0x10)?)).filter(|did| !did.is_null());
    let mut appearance = Vec::new();
    if bits & 0x20 != 0 {
        for _ in 0..hash_count(cursor)? {
            let table = DataId::new(word(cursor)?);
            let keys = (0..hash_count(cursor)?)
                .map(|_| Ok((word(cursor)?, float(cursor)?)))
                .collect::<Result<Vec<_>, Malformed>>()?;
            appearance.push((table, Arc::from(keys)));
        }
    }
    let mut fx = Ac2VisualFx::default();
    if bits & 0x40 != 0 {
        fx.fx_table = Some(DataId::new(word(cursor)?)).filter(|did| !did.is_null());
    }
    if bits & 0x80 != 0 {
        fx.startup = (0..hash_count(cursor)?)
            .map(|_| Ok((word(cursor)?, float(cursor)?)))
            .collect::<Result<Vec<_>, Malformed>>()?
            .into();
    }
    if bits & 0x100 != 0 {
        let map = read_fx_map(cursor)?;
        fx.overrides = map.particles;
        fx.sound_overrides = map.sounds;
    }
    Ok(DecodedPartGroup {
        group,
        fx,
        appearance: Arc::from(appearance),
    })
}
