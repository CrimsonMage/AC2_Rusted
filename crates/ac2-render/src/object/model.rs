//! A `VisualDesc` as drawable parts: each Setup mesh in its model-space placement, its top degrade's
//! triangle lists, and per list the material layers retail draws on this device.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{Ac2Setup, Ac2Transform};
use ac2_assets::{
    load_ac2_material_with_atoms, Ac2Atoms, Ac2MaterialLayer, Ac2Mesh, Ac2VisualDesc,
};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourcePackedDecode, ResourceResult, ResourceStore, TypedResourceRef,
};
use ac2_math::Mat4;

use super::appearance::resolve_drawn_groups;
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
    /// `CGfxObj::mesh_type` (the selection's `meshType`; 0 for cells, water and the far mesh).
    pub mesh_type: u32,
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
    /// The root group's model-space bones (`CPartArray::m_boneMatrices`, no scale), for the
    /// particle systems and objects a bone holds (`ObjectSkeleton::root_bones`).
    pub bones: Vec<Ac2Transform>,
}

impl ObjectModel {
    /// Index and vertex counts of the first part's first triangle list (`FrillRender` budgets).
    pub fn first_list(&self) -> (u32, u32) {
        self.parts.first().map_or((0, 0), |part| {
            let indices = part.batches.first().map_or(0, |b| b.indices.len());
            (indices as u32, part.vertices.len() as u32)
        })
    }

    /// Every stage texture the model's drawn layers sample, once each.
    pub fn texture_keys(&self) -> Vec<crate::TextureKey> {
        let mut keys = Vec::new();
        for batch in self.parts.iter().flat_map(|part| &part.batches) {
            for pass in batch.material.passes([1.0; 4], 0.0) {
                for stage in &pass.stages {
                    if let Some(did) = stage.texture {
                        let key = crate::TextureKey::Texture(did);
                        if !keys.contains(&key) {
                            keys.push(key);
                        }
                    }
                }
            }
        }
        keys
    }

    /// The model's parts with its root group's bones at `root` (parent-relative bone transforms,
    /// `CPartArray::m_boneTransforms` as the animation leaves them) and every other group at its
    /// default placement. `None` without a skeleton.
    pub fn pose(&self, root: Option<&[Ac2Transform]>) -> Option<ObjectPose> {
        self.rig()?.pose(root)
    }

    /// What posing the model needs, without its vertices.
    pub fn rig(&self) -> Option<PoseRig> {
        Some(PoseRig {
            skeleton: Arc::clone(self.skeleton.as_ref()?),
            parts: self
                .parts
                .iter()
                .map(|part| (part.slot, part.skin.is_some(), part.transform))
                .collect(),
        })
    }
}

/// A model's skeleton and each part's slot, skinning and default transform: enough to pose it
/// (`ObjectModel::pose`) where the drawn model lives on the GPU.
#[derive(Debug, Clone, PartialEq)]
pub struct PoseRig {
    pub skeleton: Arc<ObjectSkeleton>,
    pub parts: Vec<((usize, usize), bool, Mat4)>,
}

impl PoseRig {
    /// [`ObjectModel::pose`].
    pub fn pose(&self, root: Option<&[Ac2Transform]>) -> Option<ObjectPose> {
        let groups = self.skeleton.pose(root);
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
            .map(|&((group, index), skinned, transform)| {
                let Some(pose) = groups.get(group) else {
                    return (transform, None);
                };
                (pose.part_matrix(index), starts[group].filter(|_| skinned))
            })
            .unzip();
        Some(ObjectPose {
            parts,
            skins,
            palette,
            bones: self.skeleton.root_bones(root),
        })
    }
}

/// Loads `visual` (a `VisualDesc` or `CSetup`) with the layers `settings` draws, as
/// [`load_object_model_with`] without a local description.
pub fn load_object_model(
    store: &ResourceStore,
    visual: DataId,
    settings: &RenderSettings,
) -> ResourceResult<Option<ObjectModel>> {
    load_object_model_with(store, visual, None, settings)
}

/// Loads what an object draws: the part groups of its total `VisualDesc` (`visual`'s DAT chain,
/// or `local`'s parent chain with `local` on top; `appearance::resolve_drawn_groups`), each Setup
/// mesh at its default placement after the group's mesh replacements
/// (`CPartArray::ApplyMeshReplacement`), its materials under the group's material atoms
/// (`CPartArray::ApplyMaterialModifier`), with the layers `settings` draws. `None` when a group
/// fails to build (`CPartGroup::Init`) or nothing has a drawn layer.
pub fn load_object_model_with(
    store: &ResourceStore,
    visual: DataId,
    local: Option<&Ac2VisualDesc>,
    settings: &RenderSettings,
) -> ResourceResult<Option<ObjectModel>> {
    let Some(groups) = resolve_drawn_groups(store, visual, local)? else {
        return Ok(None);
    };
    let mut materials = HashMap::<(DataId, u32, usize), Option<Arc<ObjectMaterial>>>::new();
    let mut skeleton = ObjectSkeleton::default();
    let mut setups: Vec<CachedResource<Ac2Setup>> = Vec::new();
    let mut meshes = Vec::new();
    for (group_index, group) in groups.iter().enumerate() {
        // `CPartArray::SetSetupID` fails the build without its Setup.
        let Some(setup) = load::<Ac2Setup>(store, group.setup)? else {
            return Ok(None);
        };
        let mut group_meshes = Vec::with_capacity(setup.meshes.len());
        for (index, mesh_did) in setup.meshes.iter().enumerate() {
            if let Some((did, mesh)) = replaced_mesh(store, *mesh_did, &group.atoms)? {
                group_meshes.push((index, did, mesh));
            }
        }
        // `CPartArray::CacheHasMatrixWeights`.
        let weighted = group_meshes.iter().any(|(_, _, mesh)| mesh.skin.is_some());
        let parent = match group.parent {
            None => None,
            Some((parent, connection)) => {
                // `CPartGroup::AddChild` fails without the parent's connection point
                // (`ResetParentConnection` / `SetParentOffset`).
                let parent_setup = &*setups[parent];
                let attached = parent_setup
                    .connection_point(connection)
                    .is_some_and(|point| point.bone < parent_setup.bones.len());
                if !attached {
                    return Ok(None);
                }
                Some((parent, parent_setup, connection))
            }
        };
        skeleton
            .groups
            .push(SkeletonGroup::new(group.setup, &setup, weighted, parent));
        setups.push(setup);
        meshes.push((group_index, group_meshes));
    }
    let rest = skeleton.pose(None);
    let mut parts = Vec::new();
    for (group, group_meshes) in meshes {
        let atoms = &groups[group].atoms.materials;
        for (index, mesh_did, mesh) in group_meshes {
            let degrade = mesh.highest_detail_degrade();
            let mut batches = Vec::new();
            for fragment in mesh.fragments.iter().filter(|f| f.degrade == degrade) {
                let Some(material_did) = mesh.materials.get(fragment.material_index).copied()
                else {
                    continue;
                };
                // Materials depend on the group's atoms and the mesh type they select by.
                let key = (
                    material_did,
                    mesh.mesh_type,
                    if atoms.is_empty() { 0 } else { group },
                );
                let material = match materials.get(&key) {
                    Some(material) => material.clone(),
                    None => {
                        let material = load_ac2_material_with_atoms(
                            store,
                            material_did,
                            mesh.mesh_type,
                            atoms,
                        )?
                        .and_then(|material| {
                            let layers: Vec<_> = material
                                .layers
                                .into_iter()
                                .filter(|layer| layer_drawn(layer, settings))
                                .collect();
                            (!layers.is_empty()).then(|| Arc::new(ObjectMaterial::new(layers)))
                        });
                        materials.insert(key, material.clone());
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
                mesh_type: mesh.mesh_type,
            });
        }
    }
    Ok((!parts.is_empty()).then_some(ObjectModel {
        visual,
        parts,
        skeleton: Some(Arc::new(skeleton)),
    }))
}

/// `CMeshInstance::PerformMeshReplacement` for each mesh atom in order: an atom of category 1, or
/// of the current mesh's `mesh_type`, swaps the mesh (category 0 never does). `None` when the
/// final mesh does not load.
fn replaced_mesh(
    store: &ResourceStore,
    mesh_did: DataId,
    atoms: &Ac2Atoms,
) -> ResourceResult<Option<(DataId, CachedResource<Ac2Mesh>)>> {
    let mut did = mesh_did;
    let Some(mut mesh) = load::<Ac2Mesh>(store, did)? else {
        return Ok(None);
    };
    for atom in &atoms.meshes {
        if atom.mesh_type != 0 && (atom.mesh_type == 1 || atom.mesh_type == mesh.mesh_type) {
            did = atom.mesh;
            let Some(next) = load::<Ac2Mesh>(store, did)? else {
                return Ok(None);
            };
            mesh = next;
        }
    }
    Ok(Some((did, mesh)))
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
