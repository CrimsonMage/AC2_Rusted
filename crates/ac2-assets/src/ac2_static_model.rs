//! Asheron's Call 2 visuals as the engine's `DecodedStaticModel`: a `VisualDesc` (`0x1F`) or
//! `CSetup` (`0x04`) resolved to its meshes in the Setup's default placement frame, one surface
//! batch per mesh fragment, textured from each fragment's resolved material.

use std::collections::BTreeMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{
    CachedResource, ResourcePackedDecode, ResourceResult, ResourceStore, TypedResourceRef,
};
use ac2_math::{BBox, Mat4, Vec3};

use crate::ac2_mesh::Ac2Mesh;
use crate::ac2_texture::{ac2_texture_has_cutout_alpha, load_ac2_decoded_texture};
use crate::ac2_visual_desc::Ac2VisualDesc;
use crate::{
    load_ac2_material, Ac2MaterialLayer, DecodedConstructedSurfaceDelay, DecodedMeshVertex,
    DecodedStaticModel, DecodedStaticSurfaceBatch, DecodedStaticSurfaceMaterial,
    DecodedSurfaceBlendMode, DecodedSurfaceCullMode, DecodedTextureAsset,
};
use ac2_anim::Ac2Animation;
use ac2_anim::{Ac2Setup, Ac2Transform};

const SETUP_PREFIX: u32 = 0x0400_0000;
const VISUAL_DESC_PREFIX: u32 = 0x1F00_0000;
/// `VisualDesc` parent chains are short; this bounds a malformed cycle.
const MAX_VISUAL_DESC_DEPTH: usize = 16;

// `BlendMode`, `CullMode` and `AlphaTestMode` values of `MaterialLayer`.
const BLEND_ONE: u32 = 2;
const BLEND_SRCALPHA: u32 = 5;
const BLEND_INVSRCALPHA: u32 = 6;
/// `D3DTEXTUREOP` and `D3DTA_*` texture stage arguments.
const TOP_DISABLE: u32 = 1;
const TOP_SELECTARG1: u32 = 2;
const TOP_SELECTARG2: u32 = 3;
const TA_SELECTMASK: u32 = 0x0F;
const TA_TEXTURE: u32 = 2;
const CULLMODE_NONE: u32 = 1;
const CULLMODE_CCW: u32 = 3;

/// Resolves an AC2 `VisualDesc` (following its parent chain) or `CSetup` to the Setup it draws.
pub fn resolve_ac2_visual_setup(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<DataId>> {
    let mut current = did;
    for _ in 0..MAX_VISUAL_DESC_DEPTH {
        match current.raw() & 0xFF00_0000 {
            SETUP_PREFIX => return Ok(Some(current)),
            VISUAL_DESC_PREFIX => {
                let Some(desc) = load::<Ac2VisualDesc>(store, current)? else {
                    return Ok(None);
                };
                if let Some(setup) = desc.root_setup() {
                    return Ok(Some(setup));
                }
                let Some(parent) = desc.parent else {
                    return Ok(None);
                };
                current = parent;
            }
            _ => return Ok(None),
        }
    }
    Ok(None)
}

/// The `VisualDesc` scale a visual applies to its parts (`PHYSICSOBJREP::GetVdescScale`): the
/// nearest description in the parent chain that sets one, else one (a bare `CSetup` has none).
pub fn resolve_ac2_visual_scale(store: &ResourceStore, did: DataId) -> ResourceResult<Vec3> {
    let mut current = did;
    for _ in 0..MAX_VISUAL_DESC_DEPTH {
        if current.raw() & 0xFF00_0000 != VISUAL_DESC_PREFIX {
            break;
        }
        let Some(desc) = load::<Ac2VisualDesc>(store, current)? else {
            break;
        };
        if let Some(scale) = desc.scale {
            return Ok(scale);
        }
        let Some(parent) = desc.parent else {
            break;
        };
        current = parent;
    }
    Ok(Vec3::ONE)
}

/// The Setups a `VisualDesc` (or a bare `CSetup`) draws and each one's root transform. Part
/// groups attach to their parent group's Setup at the connection point they name: the child root
/// is the parent's bone in its default pose followed by the connection transform.
pub fn resolve_ac2_visual_parts(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Vec<(DataId, Ac2Transform)>> {
    Ok(place_parts(store, did, None)?
        .into_iter()
        .map(|part| (part.setup, part.root))
        .collect())
}

/// Every mesh a `VisualDesc` (or bare `CSetup`) draws in its default placement, with its
/// model-space transform: skinned meshes are authored in Setup space and ride the part root; rigid
/// mesh `i` rides bone `i` of the rest pose (the root when the Setup has fewer bones than meshes).
pub fn resolve_ac2_visual_meshes(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Vec<(DataId, Ac2Transform)>> {
    let mut meshes = Vec::new();
    for part in place_parts(store, did, None)? {
        let Some(setup) = load::<Ac2Setup>(store, part.setup)? else {
            continue;
        };
        let bind = setup.rest_pose();
        for (index, mesh_did) in setup.meshes.iter().enumerate() {
            let skinned =
                load::<Ac2Mesh>(store, *mesh_did)?.is_some_and(|mesh| mesh.skin.is_some());
            let local = if skinned {
                Ac2Transform::IDENTITY
            } else {
                bind.get(index)
                    .or_else(|| bind.first())
                    .copied()
                    .unwrap_or(Ac2Transform::IDENTITY)
            };
            meshes.push((*mesh_did, part.root.compose(&local)));
        }
    }
    Ok(meshes)
}

/// A Setup placed by a visual: its root transform and, for the animated root group, the posed
/// model-space bone transforms that drive its skin.
struct PlacedPart {
    setup: DataId,
    root: Ac2Transform,
    posed_bones: Option<Vec<Ac2Transform>>,
}

/// An animation sampled at a time, applied to a visual's root part group.
#[derive(Debug, Clone, Copy)]
pub struct Ac2Pose<'a> {
    pub animation: &'a Ac2Animation,
    pub time: f32,
}

fn place_parts(
    store: &ResourceStore,
    did: DataId,
    pose: Option<Ac2Pose<'_>>,
) -> ResourceResult<Vec<PlacedPart>> {
    if did.raw() & 0xFF00_0000 == SETUP_PREFIX {
        let posed_bones = pose
            .map(|pose| posed_bones(store, did, pose))
            .transpose()?
            .flatten();
        return Ok(vec![PlacedPart {
            setup: did,
            root: Ac2Transform::IDENTITY,
            posed_bones,
        }]);
    }
    let mut current = did;
    let mut groups = None;
    for _ in 0..MAX_VISUAL_DESC_DEPTH {
        if current.raw() & 0xFF00_0000 != VISUAL_DESC_PREFIX {
            break;
        }
        let Some(desc) = load::<Ac2VisualDesc>(store, current)? else {
            break;
        };
        if desc.part_groups.iter().any(|group| group.setup.is_some()) {
            groups = Some(Arc::clone(&desc.part_groups));
            break;
        }
        let Some(parent) = desc.parent else {
            break;
        };
        current = parent;
    }
    let Some(groups) = groups else {
        return Ok(Vec::new());
    };
    // (group key, part) of every placed group, parents before children.
    let mut placed: Vec<(u32, PlacedPart)> = Vec::new();
    let mut pending: Vec<_> = groups
        .iter()
        .filter(|group| group.setup.is_some())
        .collect();
    while !pending.is_empty() {
        let before = pending.len();
        let mut index = 0;
        while index < pending.len() {
            let group = pending[index];
            let root = if group.parent_key == 0 {
                Some(Ac2Transform::IDENTITY)
            } else if let Some((_, parent)) =
                placed.iter().find(|(key, _)| *key == group.parent_key)
            {
                let attachment = load::<Ac2Setup>(store, parent.setup)?.and_then(|setup| {
                    let point = setup.connection_point(group.connection_point)?;
                    let bone = match &parent.posed_bones {
                        Some(posed) => posed.get(point.bone).copied()?,
                        None => setup.rest_pose().get(point.bone).copied()?,
                    };
                    Some(parent.root.compose(&bone).compose(&point.transform))
                });
                Some(attachment.unwrap_or(parent.root))
            } else {
                None
            };
            match (root, group.setup) {
                (Some(root), Some(setup)) => {
                    let posed_bones = match pose {
                        Some(pose) if group.parent_key == 0 => posed_bones(store, setup, pose)?,
                        _ => None,
                    };
                    placed.push((
                        group.key,
                        PlacedPart {
                            setup,
                            root,
                            posed_bones,
                        },
                    ));
                    pending.swap_remove(index);
                }
                _ => index += 1,
            }
        }
        if pending.len() == before {
            // Groups whose parent never resolves are not drawn.
            break;
        }
    }
    Ok(placed.into_iter().map(|(_, part)| part).collect())
}

/// Model-space bones of `setup` with `pose` sampled over its rest locals.
fn posed_bones(
    store: &ResourceStore,
    setup: DataId,
    pose: Ac2Pose<'_>,
) -> ResourceResult<Option<Vec<Ac2Transform>>> {
    Ok(load::<Ac2Setup>(store, setup)?.map(|setup| {
        let locals = pose.animation.sample(pose.time, &setup.rest_locals());
        setup.world_pose(&locals)
    }))
}

/// Decodes an AC2 `VisualDesc` (with its attached part groups) or `CSetup` into a static model in
/// model space (AC axes: x east, y north, z up). Fragments whose material has no texture are not
/// drawn.
pub fn decode_ac2_static_model(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<DecodedStaticModel>> {
    decode_ac2_posed_model(store, did, None)
}

/// [`decode_ac2_static_model`] with the root part group's skeleton posed by an animation: skinned
/// vertices follow `posed * bind^-1` of their bones, rigid meshes and attached groups follow the
/// posed bones.
pub fn decode_ac2_posed_model(
    store: &ResourceStore,
    did: DataId,
    pose: Option<Ac2Pose<'_>>,
) -> ResourceResult<Option<DecodedStaticModel>> {
    let parts = place_parts(store, did, pose)?;
    let mut textures = BTreeMap::<DataId, Option<(Arc<DecodedTextureAsset>, bool)>>::new();
    let mut batches = Vec::new();
    let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let mut mesh_ordinal = 0;
    for part in parts {
        let Some(setup) = load::<Ac2Setup>(store, part.setup)? else {
            continue;
        };
        let root = part.root;
        let bind = setup.rest_pose();
        let pose = part.posed_bones.clone().unwrap_or_else(|| bind.clone());
        let skinning = part
            .posed_bones
            .as_ref()
            .map(|posed| skin_matrices(&bind, posed));
        for (mesh_index, mesh_did) in setup.meshes.iter().enumerate() {
            let ordinal = mesh_ordinal;
            mesh_ordinal += 1;
            let Some(mesh) = load::<Ac2Mesh>(store, *mesh_did)? else {
                continue;
            };
            // Skinned meshes are authored in Setup space; rigid meshes ride their bone (mesh i on
            // bone i, or the root when the Setup has fewer bones than meshes).
            let local = if mesh.skin.is_some() {
                Ac2Transform::IDENTITY
            } else {
                pose.get(mesh_index)
                    .or_else(|| pose.first())
                    .copied()
                    .unwrap_or(Ac2Transform::IDENTITY)
            };
            let transform = root.compose(&local);
            let vertices = match (&skinning, &mesh.skin) {
                (Some(matrices), Some(skin)) => skinned_vertices(&mesh, skin, matrices, &root),
                _ => mesh
                    .vertices
                    .iter()
                    .map(|vertex| DecodedMeshVertex {
                        position: transform.apply(vertex.position),
                        normal: (transform.rotation * vertex.normal).normalize_or_zero(),
                        uv: vertex.uv,
                    })
                    .collect::<Arc<[_]>>(),
            };
            for vertex in vertices.iter() {
                min = min.min(vertex.position);
                max = max.max(vertex.position);
            }
            append_fragments(
                store,
                &mesh,
                ordinal,
                &vertices,
                &mut textures,
                &mut batches,
            )?;
        }
    }
    if batches.is_empty() {
        return Ok(None);
    }
    Ok(Some(DecodedStaticModel {
        model_id: did,
        bounds: BBox::new(min, max),
        surface_batches: Arc::from(batches),
    }))
}

/// `posed * bind^-1` for every bone.
fn skin_matrices(bind: &[Ac2Transform], posed: &[Ac2Transform]) -> Vec<Mat4> {
    bind.iter()
        .zip(posed)
        .map(|(bind, posed)| posed.to_mat4() * bind.to_mat4().inverse())
        .collect()
}

/// Linear-blend skinning of a mesh's bind-pose vertices, then the part's root transform.
fn skinned_vertices(
    mesh: &Ac2Mesh,
    skin: &[[(u8, f32); 4]],
    matrices: &[Mat4],
    root: &Ac2Transform,
) -> Arc<[DecodedMeshVertex]> {
    let root = root.to_mat4();
    mesh.vertices
        .iter()
        .zip(skin)
        .map(|(vertex, influences)| {
            let mut blended = Mat4::ZERO;
            let mut total = 0.0;
            for (bone, weight) in influences {
                if *weight > 0.0 {
                    if let Some(matrix) = matrices.get(usize::from(*bone)) {
                        blended += *matrix * *weight;
                        total += weight;
                    }
                }
            }
            let skin = if total > 0.0 {
                root * (blended * (1.0 / total))
            } else {
                root
            };
            DecodedMeshVertex {
                position: skin.transform_point3(vertex.position),
                normal: skin.transform_vector3(vertex.normal).normalize_or_zero(),
                uv: vertex.uv,
            }
        })
        .collect()
}

/// One surface batch per drawn fragment (top detail level) of `mesh`, already placed.
fn append_fragments(
    store: &ResourceStore,
    mesh: &Ac2Mesh,
    mesh_ordinal: usize,
    vertices: &Arc<[DecodedMeshVertex]>,
    textures: &mut BTreeMap<DataId, Option<(Arc<DecodedTextureAsset>, bool)>>,
    batches: &mut Vec<DecodedStaticSurfaceBatch>,
) -> ResourceResult<()> {
    let degrade = mesh.highest_detail_degrade();
    for (subset_ordinal, fragment) in mesh.fragments.iter().enumerate() {
        if fragment.degrade != degrade {
            continue;
        }
        let Some(material_did) = mesh.materials.get(fragment.material_index).copied() else {
            continue;
        };
        let Some(material) = load_ac2_material(store, material_did)? else {
            continue;
        };
        let (Some(chain), Some(layer)) = (material.base_texture(), material.layers.first()) else {
            continue;
        };
        if !textures.contains_key(&chain) {
            let texture = match load_ac2_decoded_texture(store, chain)? {
                Some(texture) => Some((
                    Arc::new(texture),
                    ac2_texture_has_cutout_alpha(store, chain)?,
                )),
                None => None,
            };
            textures.insert(chain, texture);
        }
        let Some((texture, cutout_alpha)) = textures.get(&chain).cloned().flatten() else {
            continue;
        };
        let (static_material, delay) = surface_material(layer, cutout_alpha);
        batches.push(DecodedStaticSurfaceBatch {
            mesh_ordinal,
            subset_ordinal,
            surface_did: material_did,
            cull_mode: cull_mode(layer.cull_mode),
            material: static_material,
            constructed_delay: delay,
            stippled: false,
            texture,
            vertices: Arc::clone(vertices),
            indices: Arc::clone(&fragment.indices),
        });
    }
    Ok(())
}

/// AC2 layers blend `SRCALPHA, INVSRCALPHA` even for opaque surfaces, and retail enables the
/// alpha test for every `SRCALPHA` layer (`RenderDeviceD3D::SetupState`). Foliage and grass
/// modulate texture alpha and, with depth writes on, draw as cut-outs. Layers left at the
/// `LayerStage::Begin` default alpha source (select diffuse) draw opaque when their texture alpha
/// is a gloss or skin mask, and cut out when the texture's alpha is authored silhouette coverage
/// (`cutout_alpha`; ferns and leaf sprays keep the default). Depth-write-off layers are sorted
/// translucency.
fn surface_material(
    layer: &Ac2MaterialLayer,
    cutout_alpha: bool,
) -> (DecodedStaticSurfaceMaterial, DecodedConstructedSurfaceDelay) {
    let writes_depth = layer.depth_write != 0;
    let cuts_out = layer.alpha_test_mode != 0 || samples_texture_alpha(layer) || cutout_alpha;
    let blends = match (layer.source_blend, layer.dest_blend) {
        (BLEND_SRCALPHA, BLEND_INVSRCALPHA) => Some(DecodedSurfaceBlendMode::Alpha),
        (BLEND_SRCALPHA, BLEND_ONE) => Some(DecodedSurfaceBlendMode::AlphaAdditive),
        (BLEND_ONE, BLEND_ONE) => Some(DecodedSurfaceBlendMode::Additive),
        _ => None,
    };
    let (blend_mode, alpha_test, delay) = match blends {
        Some(DecodedSurfaceBlendMode::Alpha) if writes_depth && cuts_out => (
            DecodedSurfaceBlendMode::ClipMap,
            true,
            DecodedConstructedSurfaceDelay::Clip,
        ),
        Some(DecodedSurfaceBlendMode::Alpha) if writes_depth => (
            DecodedSurfaceBlendMode::Opaque,
            false,
            DecodedConstructedSurfaceDelay::Immediate,
        ),
        Some(mode) => (
            mode,
            layer.alpha_test_mode != 0,
            DecodedConstructedSurfaceDelay::Alpha,
        ),
        None if layer.alpha_test_mode != 0 => (
            DecodedSurfaceBlendMode::ClipMap,
            true,
            DecodedConstructedSurfaceDelay::Clip,
        ),
        None => (
            DecodedSurfaceBlendMode::Opaque,
            false,
            DecodedConstructedSurfaceDelay::Immediate,
        ),
    };
    let material = DecodedStaticSurfaceMaterial {
        base_color: layer.diffuse,
        blend_mode,
        alpha_test,
        ..DecodedStaticSurfaceMaterial::default()
    };
    (material, delay)
}

/// Whether the first texture stage's alpha operation reads the texture (`D3DTA_TEXTURE`).
fn samples_texture_alpha(layer: &Ac2MaterialLayer) -> bool {
    let Some(stage) = layer.stages.first() else {
        return false;
    };
    let texture = |arg: u32| arg & TA_SELECTMASK == TA_TEXTURE;
    match stage.alpha_op {
        TOP_DISABLE => false,
        TOP_SELECTARG1 => texture(stage.alpha_arg1),
        TOP_SELECTARG2 => texture(stage.alpha_arg2),
        _ => texture(stage.alpha_arg1) || texture(stage.alpha_arg2),
    }
}

fn cull_mode(mode: u32) -> DecodedSurfaceCullMode {
    match mode {
        CULLMODE_NONE => DecodedSurfaceCullMode::None,
        CULLMODE_CCW => DecodedSurfaceCullMode::CounterClockwise,
        _ => DecodedSurfaceCullMode::Clockwise,
    }
}

fn load<T: ResourcePackedDecode + Send + Sync + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<T>>> {
    store.load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))
}
