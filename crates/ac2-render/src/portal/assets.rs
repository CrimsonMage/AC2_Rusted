//! What `PortalSpace::Effect::Initialize` loads for the tunnel, its backdrop and its clouds: the
//! tunnels' `MaterialInstance`s (`DBObj::GetByEnum(0x40000013 / 0x40000014, 0x40000002, ...)`,
//! `m_pTunnelMaterial` and `m_pOuterTunnelMaterial`), the background `CDataMesh`
//! (`CDataMesh::GetByEnum(0x40000016, 0x40000002)`) and the clouds' `PSDesc`
//! (`FileArray::GetDIDFromEnum(0x40000015, 0x40000002)`, made into a system by
//! `PSManager::CreateParticleSystem`), as models the object renderer draws. Read at setup, never
//! on the frame thread.

use std::sync::Arc;

use ac2_assets::Ac2ParticleSystemDesc;
use ac2_core::DataId;
use ac2_db::{ResourceStore, TypedResourceRef};

use crate::object::{
    load_mesh_model, particle_material_model, ObjectModel, ObjectVertex, RenderSettings,
};

use super::TunnelVertex;

/// The group `Initialize` reads its DIDs from (`0x40000002`).
const DID_GROUP: u32 = 0x4000_0002;
/// `Initialize`'s enum for the clouds' `PSDesc`.
const CLOUDS_ENUM: u32 = 0x4000_0015;
/// `Initialize`'s enum for the background mesh.
const BACKGROUND_ENUM: u32 = 0x4000_0016;
/// `Initialize`'s enums for the inner and outer tunnels' materials.
pub const INNER_TUNNEL_ENUM: u32 = 0x4000_0013;
pub const OUTER_TUNNEL_ENUM: u32 = 0x4000_0014;

/// The settings the portal space's materials are prepared with: a class-2 device at high detail
/// and no fog (`SceneTool::SetFogProperties(FOGMODE_NONE, ...)` in `Render`).
pub fn portal_settings() -> RenderSettings {
    RenderSettings {
        fog_linear: false,
        ..RenderSettings::default()
    }
}

/// The DID an `Initialize` enum value names; `None` where the DAT has none.
fn enum_did(store: &ResourceStore, value: u32) -> Option<DataId> {
    ac2_assets::ac2_enum_did(store, DID_GROUP, value)
        .ok()
        .flatten()
        .filter(|did| did.raw() != 0)
}

/// The clouds' particle description (`m_pTunnelCloudsParticleSystem`'s `PSDesc`), if the DAT has
/// one with a material and emitters.
pub fn load_clouds_desc(store: &ResourceStore) -> Option<Arc<Ac2ParticleSystemDesc>> {
    let did = enum_did(store, CLOUDS_ENUM)?;
    let desc = store
        .load_cached_packed::<Ac2ParticleSystemDesc>(TypedResourceRef::new(
            ac2_dat::DatFileKind::Portal,
            did,
            0,
        ))
        .ok()
        .flatten()?;
    let desc = Arc::new((*desc).clone());
    (desc.material.is_some() && !desc.emitters.is_empty()).then_some(desc)
}

/// The models the tunnel's backdrop and clouds draw through.
#[derive(Debug, Default)]
pub struct PortalModels {
    /// `m_pBackgroundMesh`.
    pub background: Option<ObjectModel>,
    /// The empty model the clouds' material draws through (`ParticleSystem::Render`).
    pub clouds: Option<ObjectModel>,
    /// The empty models the inner and outer tunnels' materials draw through: `Render` passes
    /// each tunnel's vertex buffer to `RenderIndexedUsingMaterial` with the material and its
    /// instance's modified material, so every layer, stage, blend and animated modifier applies.
    pub inner_tunnel: Option<ObjectModel>,
    pub outer_tunnel: Option<ObjectModel>,
}

/// Loads the background mesh and the clouds' material model.
pub fn load_portal_models(store: &ResourceStore) -> PortalModels {
    let settings = portal_settings();
    let background = enum_did(store, BACKGROUND_ENUM)
        .and_then(|did| load_mesh_model(store, did, &settings).ok().flatten());
    let clouds = load_clouds_desc(store)
        .and_then(|desc| desc.material)
        .and_then(|material| particle_material_model(store, material, &settings));
    let tunnel = |value| {
        enum_did(store, value).and_then(|did| particle_material_model(store, did, &settings))
    };
    PortalModels {
        background,
        clouds,
        inner_tunnel: tunnel(INNER_TUNNEL_ENUM),
        outer_tunnel: tunnel(OUTER_TUNNEL_ENUM),
    }
}

/// A tunnel's vertices as the object renderer takes them: `UpdateGeometry`'s position, its
/// diffuse colour (white, the ring's fade in alpha; packed ARGB) and its texture coordinates (the
/// ring's position along the tunnel, the point's sixteenth around it), for every coordinate set a
/// stage may read. The tunnel has no normals: its materials draw unlit.
pub fn tunnel_object_vertices(vertices: &[TunnelVertex]) -> Vec<ObjectVertex> {
    vertices
        .iter()
        .map(|v| {
            let channel = |shift: u32| ((v.color >> shift) & 0xFF) as f32 / 255.0;
            ObjectVertex {
                position: v.position,
                normal: [0.0, 0.0, 1.0],
                color: [channel(16), channel(8), channel(0), channel(24)],
                uvs: [v.uv; 4],
                tangents: [[0.0; 3]; 2],
            }
        })
        .collect()
}
