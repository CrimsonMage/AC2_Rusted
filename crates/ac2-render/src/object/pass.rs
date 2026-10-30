//! One `MaterialLayer` as the device state retail sets before drawing it:
//! `RenderMaterial::GetSubObjects` keeps the layers of the device's hardware class,
//! `RenderVertexBuffer::RenderIndexedUsingMaterial` gates them by detail and fog, and
//! `RenderDeviceD3D::SetupState` turns the survivors into material colours, blend/depth/alpha-test
//! state and texture stages. `MaterialLayer::BuildCompilerFlags` picks the vertex program that
//! `fragment.dll` links for the layer.

use std::sync::Arc;

use ac2_assets::{Ac2LayerModifier, Ac2MaterialLayer};
use ac2_core::DataId;
use ac2_math::Mat4;

use super::pixel_shader::PixelProgram;

/// `BlendMode`.
pub mod blend {
    pub const ZERO: u32 = 1;
    pub const ONE: u32 = 2;
    pub const SRCALPHA: u32 = 5;
    pub const INVSRCALPHA: u32 = 6;
    pub const DSTALPHA: u32 = 7;
    pub const INVDSTALPHA: u32 = 8;
}

const ALPHATEST_ON: u32 = 1;
const MAX_STAGES: usize = 4;

/// The device a material is prepared for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSettings {
    /// `RenderDevice::caps.m_nHardwareClass`: 1 without pixel shaders, 2 with vertex and pixel
    /// shaders and four texture stages. Layers flagged for another class are dropped.
    pub hardware_class: u32,
    /// Detail setting passed to `RenderIndexedUsingMaterial`: 0 low, 1 medium, 2 high.
    pub detail: u32,
    /// Whether the scene fog is `FOGMODE_LINEAR`.
    pub fog_linear: bool,
}

impl Default for RenderSettings {
    /// High detail outdoors (linear fog) on a class-2 device, which every GPU with vertex and
    /// pixel shaders and four texture stages is (`RenderDeviceD3D` caps check).
    fn default() -> Self {
        Self {
            hardware_class: 2,
            detail: 2,
            fog_linear: true,
        }
    }
}

/// `RenderMaterial::GetSubObjects` (hardware class bits `0x100000..0x400000`, layers without
/// stages) and `RenderIndexedUsingMaterial` (detail bits `0x10000..0x40000`, linear-fog bit
/// `0x80000`): whether retail draws `layer`.
pub fn layer_drawn(layer: &Ac2MaterialLayer, settings: &RenderSettings) -> bool {
    let (on, off) = (layer.true_flags, layer.false_flags);
    let gate = |bit: u32, active: bool| (on & bit == 0 || active) && (off & bit == 0 || !active);
    !layer.stages.is_empty()
        && gate(0x10_0000, settings.hardware_class == 1)
        && gate(0x20_0000, settings.hardware_class == 2)
        && gate(0x40_0000, settings.hardware_class == 3)
        && gate(0x1_0000, settings.detail == 0)
        && gate(0x2_0000, settings.detail == 1)
        && gate(0x4_0000, settings.detail == 2)
        && gate(0x8_0000, settings.fog_linear)
}

/// The vertex program `FragmentGetCompiler`'s compile links for a layer, from the
/// `BuildCompilerFlags` bits: unlit, lit, lit with vertex diffuse (`dynamic | vertex diffuse`),
/// and the two per-pixel lighting programs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexProgram {
    Unlit,
    Lit,
    PixelLit,
    LitVertexDiffuse,
    PixelLitMultiple,
}

/// Per-layer switches of the linked vertex program (`BuildCompilerFlags`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProgramFlags {
    pub material_diffuse: bool,
    pub vertex_diffuse: bool,
    pub modulate_vertex_diffuse: bool,
    pub specular: bool,
    pub depth_fog: bool,
    pub diffuse_fog_amount: bool,
    pub alpha_fog: bool,
    pub ignore_directional: bool,
    pub ignore_point: bool,
    pub pixel_specular: bool,
    /// `m_Options & 1`: the per-pixel programs end with `f_OutputColor1_Color1` instead of
    /// `f_OutputPreModulatedColor1_Color1`.
    pub unmodulated_color1: bool,
    /// `m_bUseDynamicLighting`: the device lights (and the pixel shader's light colour) are on.
    pub dynamic: bool,
    /// `Ac2LayerLighting::display_lit`: lit in display space (the AC1 creatures).
    pub display_lit: bool,
}

/// One texture stage after `SetupState`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StagePass {
    /// `None` for an untextured stage and for the frame-buffer and distortion-map special
    /// textures, which this renderer does not produce.
    pub texture: Option<DataId>,
    pub color_op: u32,
    pub color_arg1: u32,
    pub color_arg2: u32,
    pub alpha_op: u32,
    pub alpha_arg1: u32,
    pub alpha_arg2: u32,
    pub address_u: u32,
    pub address_v: u32,
    pub min_filter: u32,
    pub mag_filter: u32,
    /// `LayerStage::m_nTexCoord`: a vertex coordinate set (0-7) or a generated source
    /// (`0x100` view reflection, `0x300` clip position, `0x400` view position, ...).
    pub tex_coord: u32,
    /// Rows 0 and 1 of the stage's transposed texture matrix (`m_TTextureMatrices`), which the
    /// vertex program dots with `(u, v, 0, 1)`, when a modifier targets the stage.
    pub texture_matrix: Option<[[f32; 4]; 2]>,
}

/// Device state for drawing one layer.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerPass {
    pub program: VertexProgram,
    pub flags: ProgramFlags,
    /// `D3DMATERIAL8` ambient, diffuse (tinted) and specular (tinted), and the specular power.
    pub ambient: [f32; 4],
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
    pub power: f32,
    pub source_blend: u32,
    pub dest_blend: u32,
    pub depth_write: bool,
    pub depth_test: u32,
    pub cull_mode: u32,
    /// `D3DRS_ALPHATESTENABLE` with `ALPHAFUNC` GREATER and this reference (0-255).
    pub alpha_test: Option<u32>,
    /// `D3DRS_SPECULARENABLE`: the vertex program's second colour is added after the stages.
    pub specular_enable: bool,
    /// `SetDepthFog`: range fog from the vertex program.
    pub fog: bool,
    /// `D3DRS_TEXTUREFACTOR` as the stages read it.
    pub texture_factor: [f32; 4],
    pub stages: Vec<StagePass>,
    /// The layer's authored pixel shader, which replaces the stage cascade.
    pub pixel_program: Option<Arc<PixelProgram>>,
}

/// `RenderDeviceD3D::SetupState` for `layer` at client time `time`, tinted by `tint`.
pub fn layer_pass(layer: &Ac2MaterialLayer, tint: [f32; 4], time: f64) -> LayerPass {
    layer_pass_forced(layer, tint, time, false)
}

/// [`layer_pass`] under `SceneTool::m_bForceDisableAlphaTest` (`CPartArray::Draw` sets it for
/// objects farther than 40 units): a faded alpha-tested or `SRCALPHA` layer drops its alpha test
/// and depth write whatever its alpha.
pub fn layer_pass_forced(
    layer: &Ac2MaterialLayer,
    tint: [f32; 4],
    time: f64,
    force_disable_alpha_test: bool,
) -> LayerPass {
    let lighting = &layer.lighting;
    let power = layer.specular_power.value(time);
    let specular_on = (power != 0.0 && lighting.specular_highlights) || lighting.pixel_specular;
    let base_diffuse = if lighting.material_diffuse {
        layer.diffuse
    } else {
        [1.0; 4]
    };
    let diffuse = mul4(base_diffuse, tint);
    let specular = mul4(
        if specular_on {
            layer.specular
        } else {
            [0.0; 4]
        },
        tint,
    );

    let mut depth_write = layer.depth_write != 0;
    let mut alpha_mode = layer.alpha_test_mode;
    let alpha_test_on = alpha_mode != 0 || layer.source_blend == blend::SRCALPHA;
    if alpha_test_on
        && diffuse[3] < 1.0
        && (force_disable_alpha_test || (alpha_mode == 0 && diffuse[3] < 0.8))
    {
        depth_write = false;
        alpha_mode = 0;
    }
    let alpha_ref = if alpha_mode == ALPHATEST_ON {
        (layer.alpha_test_ref.value(time) * 255.0) as i32 as u32
    } else {
        0
    };
    let program = vertex_program(layer);
    let mut matrices = texture_matrices(&layer.modifiers, time);
    let stages = layer
        .stages
        .iter()
        .take(MAX_STAGES)
        .enumerate()
        .map(|(index, stage)| StagePass {
            texture: (stage.special_texture == 0)
                .then_some(stage.texture)
                .flatten(),
            color_op: stage.color_op,
            color_arg1: stage.color_arg1,
            color_arg2: stage.color_arg2,
            alpha_op: stage.alpha_op,
            alpha_arg1: stage.alpha_arg1,
            alpha_arg2: stage.alpha_arg2,
            address_u: stage.address_u,
            address_v: stage.address_v,
            min_filter: stage.min_filter,
            mag_filter: stage.mag_filter,
            tex_coord: stage.tex_coord,
            texture_matrix: matrices[index]
                .take()
                .map(|m| [m.row(0).to_array(), m.row(1).to_array()]),
        })
        .collect();
    LayerPass {
        program,
        flags: ProgramFlags {
            material_diffuse: lighting.material_diffuse,
            vertex_diffuse: lighting.vertex_diffuse,
            modulate_vertex_diffuse: lighting.modulate_vertex_diffuse,
            specular: lighting.specular_highlights,
            depth_fog: lighting.depth_fog,
            diffuse_fog_amount: lighting.diffuse_fog_amount,
            alpha_fog: lighting.depth_alpha_fog,
            ignore_directional: lighting.ignore_directional,
            ignore_point: lighting.ignore_point,
            pixel_specular: lighting.pixel_specular,
            unmodulated_color1: layer.options & 1 != 0,
            dynamic: lighting.dynamic,
            display_lit: lighting.display_lit,
        },
        ambient: layer.ambient,
        diffuse,
        specular,
        power: if specular_on { power } else { 0.0 },
        // The back buffer carries no destination alpha, so DSTALPHA blends read source alpha.
        source_blend: without_dest_alpha(layer.source_blend),
        dest_blend: without_dest_alpha(layer.dest_blend),
        depth_write,
        depth_test: layer.depth_test,
        cull_mode: layer.cull_mode,
        alpha_test: alpha_test_on.then_some(alpha_ref),
        specular_enable: specular_on && layer.pixel_shader.is_empty(),
        fog: lighting.depth_fog && !lighting.diffuse_fog_amount,
        texture_factor: texture_factor(diffuse),
        stages,
        pixel_program: None,
    }
}

/// The program selection at the top of the fragment compile: `dynamic | vertex diffuse` first,
/// then per-pixel lighting (multiple lights or not), then dynamic or not.
fn vertex_program(layer: &Ac2MaterialLayer) -> VertexProgram {
    let lighting = &layer.lighting;
    if lighting.dynamic && lighting.vertex_diffuse {
        VertexProgram::LitVertexDiffuse
    } else if !lighting.per_pixel {
        if lighting.dynamic {
            VertexProgram::Lit
        } else {
            VertexProgram::Unlit
        }
    } else if lighting.pixel_multiple_lights {
        VertexProgram::PixelLitMultiple
    } else {
        VertexProgram::PixelLit
    }
}

fn without_dest_alpha(mode: u32) -> u32 {
    match mode {
        blend::DSTALPHA => blend::SRCALPHA,
        blend::INVDSTALPHA => blend::INVSRCALPHA,
        other => other,
    }
}

/// `SetupVertexShaderConstants` leaves `TEXTUREFACTOR` alone while lighting is on and sets it to
/// the material diffuse (`_ftol2` of each channel times 255) when it is off; the material diffuse
/// stands in for both.
fn texture_factor(diffuse: [f32; 4]) -> [f32; 4] {
    diffuse.map(|c| ((c * 255.0) as i32).clamp(0, 255) as f32 / 255.0)
}

/// Per stage, the product of its modifiers' `ApplyTextureTransform` in layer order
/// (`m = m * modifier` on D3DX row-vector matrices; these are the column-vector transposes), or
/// `None` when no modifier targets it.
pub fn texture_matrices(modifiers: &[Ac2LayerModifier], time: f64) -> [Option<Mat4>; MAX_STAGES] {
    let mut out = [None; MAX_STAGES];
    for modifier in modifiers {
        let (stage, transform) = super::texture_matrix::modifier_transform(modifier, time);
        if let Some(slot) = out.get_mut(stage as usize) {
            let current: Mat4 = slot.unwrap_or(Mat4::IDENTITY);
            // Row-vector `current * transform` is column-vector `transform * current`.
            *slot = Some(transform * current);
        }
    }
    out
}

fn mul4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}
