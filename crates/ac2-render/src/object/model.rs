//! A `VisualDesc` as drawable parts: each Setup mesh in its model-space placement, its top degrade's
//! triangle lists, and per list the material layers retail draws on this device.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_assets::{load_ac2_material, resolve_ac2_visual_meshes, Ac2MaterialLayer, Ac2Mesh};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};
use ac2_math::Mat4;

use super::pass::{layer_drawn, layer_pass, LayerPass, RenderSettings};
use super::pixel_shader::{parse_pixel_shader, PixelProgram};

/// One vertex as the vertex programs read it (`v0` position, `v1` normal, `v2` diffuse,
/// `v6..` texture coordinate sets).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    /// The `D3DFVF_DIFFUSE` colour as RGBA, white when the mesh has none.
    pub color: [f32; 4],
    /// Texture coordinate sets 0-3 (missing sets read zero).
    pub uvs: [[f32; 2]; 4],
    /// Bump tangents S and T (`v3`, `v4`), zero when the mesh has none.
    pub tangents: [[f32; 3]; 2],
}

/// The layers of one material that this device draws.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectMaterial {
    layers: Vec<Ac2MaterialLayer>,
    /// Each layer's parsed authored pixel shader. A shader that does not parse leaves the layer
    /// on its texture stages.
    programs: Vec<Option<Arc<PixelProgram>>>,
    animated: bool,
}

impl ObjectMaterial {
    pub(crate) fn new(layers: Vec<Ac2MaterialLayer>) -> Self {
        let animated = layers.iter().any(layer_animated);
        let programs = layers
            .iter()
            .map(|layer| {
                (!layer.pixel_shader.is_empty())
                    .then(|| parse_pixel_shader(&layer.pixel_shader).ok().map(Arc::new))
                    .flatten()
            })
            .collect();
        Self {
            layers,
            programs,
            animated,
        }
    }

    /// `SetupState` of every drawn layer, in draw order.
    pub fn passes(&self, tint: [f32; 4], time: f64) -> Vec<LayerPass> {
        self.layers
            .iter()
            .zip(&self.programs)
            .map(|(layer, program)| LayerPass {
                pixel_program: program.clone(),
                ..layer_pass(layer, tint, time)
            })
            .collect()
    }

    /// Whether any pass depends on client time (waveform specular power or alpha reference, or
    /// UV modifiers).
    pub fn animated(&self) -> bool {
        self.animated
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
}

/// One triangle list and the material it draws with.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectBatch {
    pub indices: Arc<[u32]>,
    pub material: Arc<ObjectMaterial>,
}

/// One mesh of the visual.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPart {
    pub mesh: DataId,
    /// Mesh to model space.
    pub transform: Mat4,
    pub vertices: Arc<[ObjectVertex]>,
    pub batches: Vec<ObjectBatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectModel {
    pub visual: DataId,
    pub parts: Vec<ObjectPart>,
}

/// Loads `visual` (a `VisualDesc` or `CSetup`) with the layers `settings` draws. `None` when it
/// names no mesh with a drawn layer.
pub fn load_object_model(
    store: &ResourceStore,
    visual: DataId,
    settings: &RenderSettings,
) -> ResourceResult<Option<ObjectModel>> {
    let mut materials = HashMap::<DataId, Option<Arc<ObjectMaterial>>>::new();
    let mut parts = Vec::new();
    for (mesh_did, transform) in resolve_ac2_visual_meshes(store, visual)? {
        let Some(mesh) = store.load_cached_packed::<Ac2Mesh>(TypedResourceRef::new(
            DatFileKind::Portal,
            mesh_did,
            0,
        ))?
        else {
            continue;
        };
        let degrade = mesh.highest_detail_degrade();
        let mut batches = Vec::new();
        for fragment in mesh.fragments.iter().filter(|f| f.degrade == degrade) {
            let Some(material_did) = mesh.materials.get(fragment.material_index).copied() else {
                continue;
            };
            let material = match materials.get(&material_did) {
                Some(material) => material.clone(),
                None => {
                    let material = load_ac2_material(store, material_did)?.and_then(|material| {
                        let layers: Vec<_> = material
                            .layers
                            .into_iter()
                            .filter(|layer| layer_drawn(layer, settings))
                            .collect();
                        (!layers.is_empty()).then(|| Arc::new(ObjectMaterial::new(layers)))
                    });
                    materials.insert(material_did, material.clone());
                    material
                }
            };
            if let Some(material) = material {
                batches.push(ObjectBatch {
                    indices: Arc::clone(&fragment.indices),
                    material,
                });
            }
        }
        if batches.is_empty() {
            continue;
        }
        parts.push(ObjectPart {
            mesh: mesh_did,
            transform: transform.to_mat4(),
            vertices: object_vertices(&mesh),
            batches,
        });
    }
    Ok((!parts.is_empty()).then_some(ObjectModel { visual, parts }))
}

fn object_vertices(mesh: &Ac2Mesh) -> Arc<[ObjectVertex]> {
    mesh.vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            let mut uvs = [[0.0; 2]; 4];
            uvs[0] = vertex.uv.to_array();
            for (set, extra) in mesh.extra_uvs.iter().take(3).enumerate() {
                uvs[set + 1] = extra[index].to_array();
            }
            ObjectVertex {
                position: vertex.position.to_array(),
                normal: vertex.normal.to_array(),
                color: mesh
                    .colors
                    .as_ref()
                    .map_or([1.0; 4], |colors| d3d_color(colors[index])),
                uvs,
                tangents: mesh.tangents.as_ref().map_or([[0.0; 3]; 2], |t| {
                    [t[index][0].to_array(), t[index][1].to_array()]
                }),
            }
        })
        .collect()
}

/// A `D3DCOLOR` (`0xAARRGGBB`) as the vertex unit reads it.
fn d3d_color(color: u32) -> [f32; 4] {
    let channel = |shift: u32| ((color >> shift) & 0xFF) as f32 / 255.0;
    [channel(16), channel(8), channel(0), channel(24)]
}

fn layer_animated(layer: &Ac2MaterialLayer) -> bool {
    use ac2_assets::{Ac2LayerModifier, Ac2Waveform};
    let varies =
        |w: &Ac2Waveform| w.kind > ac2_math::waveform::kind::NONE || w.base_velocity != 0.0;
    varies(&layer.specular_power)
        || varies(&layer.alpha_test_ref)
        || layer.modifiers.iter().any(|modifier| match modifier {
            Ac2LayerModifier::UvTranslate { u, v, .. } | Ac2LayerModifier::UvScale { u, v, .. } => {
                varies(u) || varies(v)
            }
            Ac2LayerModifier::UvRotate { rotate, .. } => varies(rotate),
            Ac2LayerModifier::UvTransform { .. } => false,
        })
}
