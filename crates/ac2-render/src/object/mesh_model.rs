//! Models built from a bare `CDataMesh` or a particle material, with no visual and no skeleton:
//! the portal space's background mesh (`CBaseMesh::DrawMesh` at the eye) and the clouds' particle
//! material (`ParticleSystem::Render`), which the renderer fills with the frame's particles.

use std::sync::Arc;

use ac2_assets::{load_ac2_material, Ac2Mesh};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};
use ac2_math::Mat4;

use super::model::{ObjectBatch, ObjectMaterial, ObjectModel, ObjectPart, ObjectVertex};
use super::pass::{layer_drawn, RenderSettings};

/// A `CDataMesh` drawn on its own: its highest-detail degree's fragments, each batch with its
/// material's drawn layers, as one part at the model's origin. `None` when no fragment has a drawn
/// material.
pub fn load_mesh_model(
    store: &ResourceStore,
    mesh_did: DataId,
    settings: &RenderSettings,
) -> ResourceResult<Option<ObjectModel>> {
    let Some(mesh) = store.load_cached_packed::<Ac2Mesh>(TypedResourceRef::new(
        DatFileKind::Portal,
        mesh_did,
        0,
    ))?
    else {
        return Ok(None);
    };
    let degrade = mesh.highest_detail_degrade();
    let mut batches = Vec::new();
    for fragment in mesh.fragments.iter().filter(|f| f.degrade == degrade) {
        let Some(material_did) = mesh.materials.get(fragment.material_index).copied() else {
            continue;
        };
        let layers: Vec<_> = load_ac2_material(store, material_did)?
            .map(|material| material.layers)
            .unwrap_or_default()
            .into_iter()
            .filter(|layer| layer_drawn(layer, settings))
            .collect();
        if layers.is_empty() {
            continue;
        }
        batches.push(ObjectBatch {
            indices: Arc::clone(&fragment.indices),
            material: Arc::new(ObjectMaterial::new(layers)),
        });
    }
    if batches.is_empty() {
        return Ok(None);
    }
    Ok(Some(ObjectModel {
        visual: mesh_did,
        parts: vec![ObjectPart {
            mesh: mesh_did,
            transform: Mat4::IDENTITY,
            vertices: super::model::object_vertices(&mesh),
            batches,
            degrades: None,
            slot: (0, 0),
            skin: None,
            mesh_type: mesh.mesh_type,
        }],
        degrade: None,
        skeleton: None,
    }))
}

/// The model a particle material draws through: one empty triangle with the material's drawn
/// layers, which the renderer replaces by the frame's particles. `None` when the material has
/// no drawn layer.
pub fn particle_material_model(
    store: &ResourceStore,
    material: DataId,
    settings: &RenderSettings,
) -> Option<ObjectModel> {
    let layers: Vec<_> = load_ac2_material(store, material)
        .ok()
        .flatten()?
        .layers
        .into_iter()
        .filter(|layer| layer_drawn(layer, settings))
        .collect();
    if layers.is_empty() {
        return None;
    }
    let vertex = ObjectVertex {
        position: [0.0; 3],
        normal: [0.0, 0.0, 1.0],
        color: [1.0; 4],
        uvs: [[0.0; 2]; 4],
        tangents: [[0.0; 3]; 2],
    };
    Some(ObjectModel {
        visual: material,
        parts: vec![ObjectPart {
            mesh: material,
            transform: Mat4::IDENTITY,
            vertices: Arc::from(vec![vertex; 3]),
            batches: vec![ObjectBatch {
                indices: Arc::from(vec![0, 1, 2]),
                material: Arc::new(ObjectMaterial::new(layers)),
            }],
            degrades: None,
            slot: (0, 0),
            skin: None,
            mesh_type: 0,
        }],
        degrade: None,
        skeleton: None,
    })
}
