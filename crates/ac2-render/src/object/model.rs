//! A `VisualDesc` as drawable parts: each Setup mesh in its model-space placement, its top degrade's
//! triangle lists, and per list the material layers retail draws on this device.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{Ac2Setup, Ac2Transform};
use ac2_assets::{load_ac2_material, resolve_ac2_visual_groups, Ac2MaterialLayer, Ac2Mesh};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourcePackedDecode, ResourceResult, ResourceStore, TypedResourceRef,
};
use ac2_math::Mat4;

use super::pass::{layer_drawn, layer_pass, LayerPass, RenderSettings};
use super::pixel_shader::{parse_pixel_shader, PixelProgram};
use super::skeleton::{ObjectSkeleton, SkeletonGroup};

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
    /// The part's group in the model's skeleton and its index in that group's Setup.
    pub slot: (usize, usize),
    /// Per-vertex (bone, weight) influences when the mesh carries matrix weights.
    pub skin: Option<Arc<[[(u8, f32); 4]]>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectModel {
    pub visual: DataId,
    pub parts: Vec<ObjectPart>,
    /// The visual's part-group tree, for posing; `None` for cells, water and the far mesh.
    pub skeleton: Option<Arc<ObjectSkeleton>>,
}

/// A model posed for one draw: what `CPartArray::Draw` hands the device.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjectPose {
    /// `CPhysicsPart::m_mObject` of every part, in model part order (mesh to model space).
    pub parts: Vec<Mat4>,
    /// Per part, the first of its group's matrices in `palette` when its vertices blend.
    pub skins: Vec<Option<u32>>,
    /// The deform matrices of every skinned group (`SceneTool::SetMatrixStack`).
    pub palette: Vec<Mat4>,
}

impl ObjectModel {
    /// The model's parts with its root group's bones at `root` (parent-relative bone transforms,
    /// `CPartArray::m_boneTransforms` as the animation leaves them) and every other group at its
    /// default placement. `None` without a skeleton.
    pub fn pose(&self, root: Option<&[Ac2Transform]>) -> Option<ObjectPose> {
        let skeleton = self.skeleton.as_ref()?;
        let groups = skeleton.pose(root);
        let mut palette = Vec::new();
        let mut starts = Vec::with_capacity(groups.len());
        for group in &groups {
            starts.push(group.blend_matrices().map(|matrices| {
                let start = palette.len() as u32;
                palette.extend(matrices);
                start
            }));
        }
        let (parts, skins) = self
            .parts
            .iter()
            .map(|part| {
                let (group, index) = part.slot;
                let Some(pose) = groups.get(group) else {
                    return (part.transform, None);
                };
                let skin = part.skin.as_ref().and(starts[group]);
                (pose.part_matrix(index), skin)
            })
            .unzip();
        Some(ObjectPose {
            parts,
            skins,
            palette,
        })
    }
}

/// Loads `visual` (a `VisualDesc` or `CSetup`) with the layers `settings` draws: every part group
/// of the visual (`CPartGroup`), each Setup mesh at its default placement. `None` when it names no
/// mesh with a drawn layer.
pub fn load_object_model(
    store: &ResourceStore,
    visual: DataId,
    settings: &RenderSettings,
) -> ResourceResult<Option<ObjectModel>> {
    let mut materials = HashMap::<DataId, Option<Arc<ObjectMaterial>>>::new();
    let mut skeleton = ObjectSkeleton::default();
    let mut setups: Vec<CachedResource<Ac2Setup>> = Vec::new();
    let mut meshes = Vec::new();
    // Skeleton index of each resolved group that loaded.
    let mut kept: Vec<Option<usize>> = Vec::new();
    for group in resolve_ac2_visual_groups(store, visual)? {
        let Some(setup) = load::<Ac2Setup>(store, group.setup)? else {
            kept.push(None);
            continue;
        };
        let mut group_meshes = Vec::with_capacity(setup.meshes.len());
        for (index, mesh_did) in setup.meshes.iter().enumerate() {
            if let Some(mesh) = load::<Ac2Mesh>(store, *mesh_did)? {
                group_meshes.push((index, *mesh_did, mesh));
            }
        }
        // `CPartArray::CacheHasMatrixWeights`.
        let weighted = group_meshes.iter().any(|(_, _, mesh)| mesh.skin.is_some());
        // A group whose parent's Setup failed to load is not drawn.
        let parent = match group.parent {
            None => None,
            Some((parent, connection)) => match kept.get(parent).copied().flatten() {
                Some(parent) => Some((parent, &*setups[parent], connection)),
                None => {
                    kept.push(None);
                    continue;
                }
            },
        };
        let index = skeleton.groups.len();
        kept.push(Some(index));
        skeleton
            .groups
            .push(SkeletonGroup::new(group.setup, &setup, weighted, parent));
        setups.push(setup);
        meshes.push((index, group_meshes));
    }
    let rest = skeleton.pose(None);
    let mut parts = Vec::new();
    for (group, group_meshes) in meshes {
        for (index, mesh_did, mesh) in group_meshes {
            let degrade = mesh.highest_detail_degrade();
            let mut batches = Vec::new();
            for fragment in mesh.fragments.iter().filter(|f| f.degrade == degrade) {
                let Some(material_did) = mesh.materials.get(fragment.material_index).copied()
                else {
                    continue;
                };
                let material = match materials.get(&material_did) {
                    Some(material) => material.clone(),
                    None => {
                        let material =
                            load_ac2_material(store, material_did)?.and_then(|material| {
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
                transform: rest[group].part_matrix(index),
                vertices: object_vertices(&mesh),
                batches,
                slot: (group, index),
                skin: mesh.skin.clone(),
            });
        }
    }
    Ok((!parts.is_empty()).then_some(ObjectModel {
        visual,
        parts,
        skeleton: Some(Arc::new(skeleton)),
    }))
}

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
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
