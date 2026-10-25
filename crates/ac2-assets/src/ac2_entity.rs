//! Asheron's Call 2 `EntityDesc` (`0x47`, and embedded in `CLandBlockInfo`, `EntityGroupDesc`
//! and FX lights), decoded in `EntityDesc::StreamPack` order under its pack bitfield.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};
use ac2_math::{Quat, Vec3};

use ac2_db::skip_property_collection;

/// The placement and visual facts of an entity description.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2EntityDesc {
    pub entity_type: u32,
    /// `VisualDesc` for a template, or the template `EntityDesc` for a placed instance.
    pub data: Option<DataId>,
    pub origin: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    /// The packed `PropertyCollection`, decoded on demand with `ac2_db::read_ac2_property_groups`
    /// (it needs the `MasterProperty` table installed).
    pub properties: Option<std::sync::Arc<[u8]>>,
    /// `m_strWbName`: the workbench name (`UISceneEntity::SetID` maps it to the entity id).
    pub workbench_name: Option<String>,
}

impl Default for Ac2EntityDesc {
    fn default() -> Self {
        Self {
            entity_type: 0,
            data: None,
            origin: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            properties: None,
            workbench_name: None,
        }
    }
}

impl ResourcePackedDecode for Ac2EntityDesc {
    const TYPE_NAME: &'static str = "AC2 EntityDesc";
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
        let desc = read_ac2_entity_desc(&mut cursor).map_err(malformed)?;
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after EntityDesc"));
        }
        Ok(desc)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.properties.as_ref().map_or(0, |p| p.len())
    }
}

/// Reads one `EntityDesc`: DID, type, runtime id, data id, offset `Frame` (origin, quaternion and
/// its cached 3x3 matrix), scale, properties, version and workbench name.
pub fn read_ac2_entity_desc(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2EntityDesc, &'static str> {
    let bits = word(cursor)?;
    let mut desc = Ac2EntityDesc::default();
    if bits & 0x1 != 0 {
        word(cursor)?;
    }
    if bits & 0x2 != 0 {
        desc.entity_type = word(cursor)?;
    }
    if bits & 0x4 != 0 {
        // `InstanceID` (64-bit).
        skip(cursor, 8)?;
    }
    if bits & 0x8 != 0 {
        desc.data = Some(DataId::new(word(cursor)?)).filter(|did| !did.is_null());
    }
    if bits & 0x10 != 0 {
        let f =
            |cursor: &mut ArchiveCursor<'_>| cursor.read_f32().map_err(|_| "record ended early");
        desc.origin = Vec3::new(f(cursor)?, f(cursor)?, f(cursor)?);
        let w = f(cursor)?;
        let rotation = Quat::from_xyzw(f(cursor)?, f(cursor)?, f(cursor)?, w);
        desc.rotation = if rotation.length_squared() > 0.0 {
            rotation.normalize()
        } else {
            Quat::IDENTITY
        };
        skip(cursor, 9 * 4)?;
    }
    if bits & 0x20 != 0 {
        let f =
            |cursor: &mut ArchiveCursor<'_>| cursor.read_f32().map_err(|_| "record ended early");
        desc.scale = Vec3::new(f(cursor)?, f(cursor)?, f(cursor)?);
    }
    if bits & 0x80 != 0 {
        let start = cursor.position();
        skip_property_collection(cursor)?;
        desc.properties = Some(cursor.bytes()[start..cursor.position()].into());
    }
    if bits & 0x40 != 0 {
        word(cursor)?;
    }
    if bits & 0x100 != 0 {
        // `PStringBase<char>::UnPack`: a u16 length (0xFFFF: a u32 follows), the scrambled
        // characters, alignment.
        let mut len = u32::from(cursor.read_u16().map_err(|_| "record ended early")?);
        if len == 0xFFFF {
            len = word(cursor)?;
        }
        let mut name = cursor
            .read_bytes(len as usize)
            .map_err(|_| "record ended early")?
            .to_vec();
        ac2_core::pstring::unmunge(&mut name);
        desc.workbench_name = Some(String::from_utf8_lossy(&name).into_owned());
        cursor.align_to(4).map_err(|_| "record ended early")?;
    }
    Ok(desc)
}

fn word(cursor: &mut ArchiveCursor<'_>) -> Result<u32, &'static str> {
    cursor.read_u32().map_err(|_| "record ended early")
}

fn skip(cursor: &mut ArchiveCursor<'_>, len: usize) -> Result<(), &'static str> {
    cursor
        .read_bytes(len)
        .map(|_| ())
        .map_err(|_| "record ended early")
}
