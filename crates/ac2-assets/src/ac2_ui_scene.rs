//! `UIScene` (`DB_TYPE_UI_SCENE` 0x4000003F, DIDs 0x21000000..0x21FFFFFE in the portal DAT): the
//! small 3D scene a UI viewport shows (`UIElement_Viewport`'s `uiscene`), its entities' camera,
//! lights and physics objects (`UIScene::AddEntity` by `EntityDesc::m_type`) and its fog.
//!
//! `UIScene::StreamPack`: the `DBObj` DID (`UnPack_Type`), `PSmartArray<EntityDesc>` (a u32
//! count, then each `EntityDesc::StreamPack`), the fog colour (`RGBAColor::UnPack`: one packed
//! 32-bit colour), `m_bWantFog`, `m_fFogNear`, `m_fFogFar`.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

use crate::ac2_entity::{read_ac2_entity_desc, Ac2EntityDesc};

/// `UIScene::AddEntity`'s entity types.
pub const UI_SCENE_PHYS_OBJ: u32 = 0x4000_0001;
pub const UI_SCENE_PHYS_OBJ_ALT: u32 = 0x4000_0007;
pub const UI_SCENE_CAMERA: u32 = 0x4000_0005;
pub const UI_SCENE_LIGHT: u32 = 0x4000_0006;

/// A `UIScene`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2UiScene {
    pub entities: Vec<Ac2EntityDesc>,
    /// `m_FogColor` as packed by `RGBAColor::GetColor32` (ARGB).
    pub fog_color: u32,
    pub want_fog: bool,
    pub fog_near: f32,
    pub fog_far: f32,
}

impl ResourcePackedDecode for Ac2UiScene {
    const TYPE_NAME: &'static str = "AC2 UIScene";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        let word =
            |c: &mut ArchiveCursor<'_>| c.read_u32().map_err(|_| malformed("record ended early"));
        if word(&mut cursor)? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let count = word(&mut cursor)? as usize;
        if count > 4096 {
            return Err(malformed("entity count"));
        }
        let entities = (0..count)
            .map(|_| read_ac2_entity_desc(&mut cursor).map_err(malformed))
            .collect::<ResourceResult<Vec<_>>>()?;
        let fog_color = word(&mut cursor)?;
        let want_fog = word(&mut cursor)? != 0;
        let fog_near = f32::from_bits(word(&mut cursor)?);
        let fog_far = f32::from_bits(word(&mut cursor)?);
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after UIScene"));
        }
        Ok(Self {
            entities,
            fog_color,
            want_fog,
            fog_near,
            fog_far,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.entities.len() * std::mem::size_of::<Ac2EntityDesc>()
    }
}

/// The UI scene `did` from the portal DAT.
pub fn load_ac2_ui_scene(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<Ac2UiScene>>> {
    store.load_cached_packed::<Ac2UiScene>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}

/// An `EntityDesc` record (`0x47`) from the portal DAT.
pub fn load_ac2_entity_desc(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<Ac2EntityDesc>>> {
    store.load_cached_packed::<Ac2EntityDesc>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}
