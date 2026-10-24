//! Asheron's Call 2 env cell surfaces: the drawable fragments of a `CCellMesh` (`0x01`) under
//! the fixed-function material layer retail draws them with.
//!
//! `CEnvCell::DrawCellMesh` draws the cell mesh through
//! `CMeshInstance::DrawMesh` at full detail. Cell materials are `MaterialInstance`s of templates
//! with six layers keyed by `MaterialLayer::trueFlags`: `0x0001_0000`/`0x0002_0000`/`0x0004_0000`
//! are the low/medium/high material detail, `0x0010_0000` the fixed-function and `0x0020_0000`
//! the pixel-shader path. The fixed-function high layer merges two textures: stage 0 selects the
//! base texture, stage 1 `ADDSIGNED`s a detail texture over it (`LM_UVScale` tiles the detail
//! stage, 10x on the Legions dungeon templates) and the last stage `MODULATE2X`es the result
//! with the vertex diffuse that `CEnvCell::LightIndices` bakes. Layers with one stage modulate
//! the base texture directly.

use std::sync::Arc;

use ac2_dat::DatFileKind;
use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore, TypedResourceRef};

use crate::ac2_appearance::{load_ac2_material_with_atoms, Ac2MaterialAtom};
use crate::ac2_mesh::Ac2Mesh;
use crate::{
    Ac2LayerModifier, Ac2MaterialLayer, DecodedMeshVertex, DecodedSurfaceBlendMode,
    DecodedSurfaceCullMode,
};

/// Every Legions cell mesh is `CBaseMesh` type 2 (appearance atoms address mesh types).
const CELL_MESH_TYPE: u32 = 2;
const LAYER_FIXED_FUNCTION: u32 = 0x0010_0000;
const LAYER_DETAIL_HIGH: u32 = 0x0004_0000;
const LAYER_DETAIL_MEDIUM: u32 = 0x0002_0000;

// `D3DTEXTUREOP` and `D3DTA_*` values of `LayerStage`.
const TOP_DISABLE: u32 = 1;
const TOP_MODULATE: u32 = 4;
const TOP_MODULATE2X: u32 = 5;
const TOP_MODULATE4X: u32 = 6;
const TOP_ADDSIGNED: u32 = 8;
const TA_SELECTMASK: u32 = 0x0F;
const TA_DIFFUSE: u32 = 0;
const TA_TEXTURE: u32 = 2;
const BLEND_ONE: u32 = 2;
const BLEND_SRCALPHA: u32 = 5;
const BLEND_INVSRCALPHA: u32 = 6;
const CULLMODE_NONE: u32 = 1;
const CULLMODE_CCW: u32 = 3;

/// One drawable fragment of a cell mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2CellSurface {
    /// The mesh's material slot (a `MaterialInstance` DID).
    pub material: DataId,
    /// `RenderTexture` chain of the base stage.
    pub base_texture: DataId,
    pub base_uv_scale: [f32; 2],
    /// `RenderTexture` chain `ADDSIGNED` over the base, with its stage UV scale.
    pub detail_texture: Option<DataId>,
    pub detail_uv_scale: [f32; 2],
    /// Scale of the final diffuse modulation (`MODULATE2X` is 2).
    pub diffuse_scale: f32,
    pub blend_mode: DecodedSurfaceBlendMode,
    pub alpha_test: bool,
    pub cull_mode: DecodedSurfaceCullMode,
    pub indices: Arc<[u32]>,
}

/// The drawable surfaces of one cell mesh in cell-local space (AC axes).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2CellSurfaces {
    pub vertices: Arc<[DecodedMeshVertex]>,
    pub surfaces: Vec<Ac2CellSurface>,
}

/// Decodes a `CCellMesh` into its full-detail surfaces, each material resolved with the cell's
/// appearance atoms. Fragments whose material has no textured stage are not drawn.
pub fn decode_ac2_cell_surfaces(
    store: &ResourceStore,
    cell_mesh: DataId,
    atoms: &[Ac2MaterialAtom],
) -> ResourceResult<Option<Ac2CellSurfaces>> {
    let Some(mesh) = store.load_cached_packed::<Ac2Mesh>(TypedResourceRef::new(
        DatFileKind::Portal,
        cell_mesh,
        0,
    ))?
    else {
        return Ok(None);
    };
    let degrade = mesh.highest_detail_degrade();
    let mut surfaces = Vec::new();
    for fragment in mesh
        .fragments
        .iter()
        .filter(|fragment| fragment.degrade == degrade)
    {
        let Some(material_did) = mesh.materials.get(fragment.material_index).copied() else {
            continue;
        };
        let Some(material) =
            load_ac2_material_with_atoms(store, material_did, CELL_MESH_TYPE, atoms)?
        else {
            continue;
        };
        let Some(layer) = fixed_function_layer(&material.layers) else {
            continue;
        };
        let Some(stages) = layer_stages(layer) else {
            continue;
        };
        let (blend_mode, alpha_test) = blend(layer);
        surfaces.push(Ac2CellSurface {
            material: material_did,
            base_texture: stages.base,
            base_uv_scale: stage_uv_scale(layer, stages.base_stage),
            detail_texture: stages.detail,
            detail_uv_scale: stages
                .detail_stage
                .map_or([1.0; 2], |stage| stage_uv_scale(layer, stage)),
            diffuse_scale: stages.diffuse_scale,
            blend_mode,
            alpha_test,
            cull_mode: match layer.cull_mode {
                CULLMODE_NONE => DecodedSurfaceCullMode::None,
                CULLMODE_CCW => DecodedSurfaceCullMode::CounterClockwise,
                _ => DecodedSurfaceCullMode::Clockwise,
            },
            indices: Arc::clone(&fragment.indices),
        });
    }
    Ok(Some(Ac2CellSurfaces {
        vertices: Arc::clone(&mesh.vertices),
        surfaces,
    }))
}

/// The fixed-function layer at the highest material detail the material authors.
fn fixed_function_layer(layers: &[Ac2MaterialLayer]) -> Option<&Ac2MaterialLayer> {
    let fixed = |detail: u32| {
        layers.iter().find(|layer| {
            layer.true_flags & LAYER_FIXED_FUNCTION != 0 && layer.true_flags & detail != 0
        })
    };
    fixed(LAYER_DETAIL_HIGH)
        .or_else(|| fixed(LAYER_DETAIL_MEDIUM))
        .or_else(|| {
            layers
                .iter()
                .find(|layer| layer.true_flags & LAYER_FIXED_FUNCTION != 0)
        })
        .or_else(|| layers.first())
}

struct LayerStages {
    base: DataId,
    base_stage: usize,
    detail: Option<DataId>,
    detail_stage: Option<usize>,
    diffuse_scale: f32,
}

/// Reads the stage chain: the first textured stage is the base; a later textured `ADDSIGNED`
/// stage is the detail merge; the stage modulating by the diffuse sets its scale.
fn layer_stages(layer: &Ac2MaterialLayer) -> Option<LayerStages> {
    let texture_arg = |arg: u32| arg & TA_SELECTMASK == TA_TEXTURE;
    let diffuse_arg = |arg: u32| arg & TA_SELECTMASK == TA_DIFFUSE;
    let mut base = None;
    let mut detail = None;
    let mut diffuse_scale = 1.0;
    for (index, stage) in layer.stages.iter().enumerate() {
        if stage.color_op == TOP_DISABLE {
            break;
        }
        let samples = texture_arg(stage.color_arg1) || texture_arg(stage.color_arg2);
        if let (Some(texture), true) = (stage.texture, samples) {
            if base.is_none() {
                base = Some((texture, index));
            } else if stage.color_op == TOP_ADDSIGNED && detail.is_none() {
                detail = Some((texture, index));
            }
        }
        if diffuse_arg(stage.color_arg1) || diffuse_arg(stage.color_arg2) {
            diffuse_scale = match stage.color_op {
                TOP_MODULATE => 1.0,
                TOP_MODULATE2X => 2.0,
                TOP_MODULATE4X => 4.0,
                _ => diffuse_scale,
            };
        }
    }
    let (base, base_stage) = base?;
    Some(LayerStages {
        base,
        base_stage,
        detail: detail.map(|(texture, _)| texture),
        detail_stage: detail.map(|(_, stage)| stage),
        diffuse_scale,
    })
}

/// `LM_UVScale` base values of the modifier addressing `stage`.
fn stage_uv_scale(layer: &Ac2MaterialLayer, stage: usize) -> [f32; 2] {
    layer
        .modifiers
        .iter()
        .find_map(|modifier| match modifier {
            Ac2LayerModifier::UvScale { tex_coord, u, v } if *tex_coord as usize == stage => {
                let scale = |value: f32| {
                    if value.is_finite() && value != 0.0 {
                        value
                    } else {
                        1.0
                    }
                };
                Some([scale(u.base), scale(v.base)])
            }
            _ => None,
        })
        .unwrap_or([1.0; 2])
}

/// Opaque, cut-out (alpha test) or depth-sorted translucent, from the layer's raster state.
fn blend(layer: &Ac2MaterialLayer) -> (DecodedSurfaceBlendMode, bool) {
    if layer.alpha_test_mode != 0 {
        return (DecodedSurfaceBlendMode::ClipMap, true);
    }
    match (layer.source_blend, layer.dest_blend, layer.depth_write != 0) {
        (BLEND_SRCALPHA, BLEND_INVSRCALPHA, false) => (DecodedSurfaceBlendMode::Alpha, false),
        (BLEND_SRCALPHA, BLEND_ONE, _) => (DecodedSurfaceBlendMode::AlphaAdditive, false),
        (BLEND_ONE, BLEND_ONE, _) => (DecodedSurfaceBlendMode::Additive, false),
        _ => (DecodedSurfaceBlendMode::Opaque, false),
    }
}
