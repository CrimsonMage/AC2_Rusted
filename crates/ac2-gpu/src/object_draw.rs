//! The per-draw uniform block of the object pass (`RenderDeviceD3D::SetupVertexShaderConstants`
//! and `SetupState` for one layer), the per-instance block its vertex shader reads, and the buffers
//! that hold the frame's draws, instances and bone matrices. One draw covers a run of instances
//! that share a layer and its uniform, so the uniform carries only what all of them share.

use ac2_math::{Mat4, Vec3, Vec4};
use ac2_render::object::{LayerPass, PointLight, VertexProgram, MAX_POINT_LIGHTS};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct DrawUniform {
    ambient: [f32; 4],
    diffuse: [f32; 4],
    specular: [f32; 4],
    texture_factor: [f32; 4],
    program: [u32; 4],
    color_ops: [[u32; 4]; 4],
    alpha_ops: [[u32; 4]; 4],
    matrices: [[f32; 4]; 8],
    point_count: [u32; 4],
    /// Per point light: origin and range^2, diffuse, specular, attenuation. Read by the pixel
    /// stage too, so it stays in the uniform rather than the vertex-only instance block.
    points: [[f32; 4]; 16],
}

/// One instance of a draw (`Instance` in `object.wgsl`, read by the vertex stage only, at
/// `@builtin(instance_index)`).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct InstanceData {
    model: [[f32; 4]; 4],
    /// The part's origin relative to the eye (`model`'s, minus the eye, built before the part's
    /// own offset is added), so the clip position never sums world-sized numbers.
    rel: [f32; 4],
    /// A frill's normal for every vertex (xyz) when w is 1 (`RefreshFrillMesh`).
    pub(crate) frill: [f32; 4],
    /// x: the part's first matrix in the bone buffer, y: 1 when its vertices blend.
    pub(crate) skin: [u32; 4],
    /// x: the landscape self shadow under a merged frill's origin (1 when unshadowed); y, z, w:
    /// its growth ring (near and far radius, random value; zero far radius: drawn as authored).
    pub(crate) frill_shade: [f32; 4],
}

/// Bytes of one [`InstanceData`] in the instance storage buffer.
pub(crate) const INSTANCE_BYTES: u64 = std::mem::size_of::<InstanceData>() as u64;

/// A part drawn at `model` (mesh to world), rigid and not a frill.
pub(crate) fn instance_data(model: Mat4, rel: Vec4) -> InstanceData {
    InstanceData {
        model: model.to_cols_array_2d(),
        rel: rel.to_array(),
        frill: [0.0; 4],
        skin: [0; 4],
        frill_shade: [1.0, 0.0, 0.0, 0.0],
    }
}

/// `VSLightState` constants of up to four point lights.
pub(crate) fn set_point_lights(uniform: &mut DrawUniform, lights: &[PointLight]) {
    let lights = &lights[..lights.len().min(MAX_POINT_LIGHTS)];
    uniform.point_count = [lights.len() as u32, 0, 0, 0];
    for (index, light) in lights.iter().enumerate() {
        let rgb = |c: [f32; 3]| [c[0], c[1], c[2], 1.0];
        uniform.points[index * 4] = light.origin.extend(light.range * light.range).to_array();
        uniform.points[index * 4 + 1] = rgb(light.diffuse);
        uniform.points[index * 4 + 2] = rgb(light.specular);
        let [a, b, c] = light.attenuation;
        uniform.points[index * 4 + 3] = [a, b, c, 0.0];
    }
}

/// The frame's draw uniforms (`draws` bytes, read at a dynamic offset) and instance blocks
/// (`instances` bytes), and their bind group.
pub(crate) struct DrawStorage {
    pub(crate) draws: wgpu::Buffer,
    pub(crate) instances: wgpu::Buffer,
    pub(crate) group: wgpu::BindGroup,
}

pub(crate) fn draw_storage(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    draws: u64,
    instances: u64,
) -> DrawStorage {
    let draws = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ac2 object draws"),
        size: draws,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let instances = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ac2 object instances"),
        size: instances,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ac2 object draws"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &draws,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<DrawUniform>() as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: instances.as_entire_binding(),
            },
        ],
    });
    DrawStorage {
        draws,
        instances,
        group,
    }
}

/// The storage buffer of the frame's bone matrices.
pub(crate) fn bone_storage(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    size: u64,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ac2 object bones"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ac2 object bones"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (buffer, group)
}

/// `FLAG_DISPLAY_LIT` in `object.wgsl` (bits 14 to 17 are the sky's).
const FLAG_DISPLAY_LIT_BIT: u32 = 18;
/// `FLAG_DISPLAY_GLOW` in `object.wgsl`.
const FLAG_DISPLAY_GLOW_BIT: u32 = 19;
/// `FLAG_SKY` in `object.wgsl`: the draw is a DAT sky object.
const FLAG_SKY: u32 = 1 << 14;
/// `FLAG_SKY_BACKDROP`: the sky object is the backdrop dome ([`crate::sky_backdrop`]).
const FLAG_SKY_BACKDROP: u32 = 1 << 15;

/// `FLAG_SKY_EMISSIVE`: an additive (`SRCALPHA`, `ONE`) sky layer, light in the sky.
const FLAG_SKY_EMISSIVE: u32 = 1 << 16;
/// `FLAG_SKY_VEIL`: a dome-sized alpha sky layer that tints the atmosphere
/// ([`crate::object_state::sky_veil`]).
const FLAG_SKY_VEIL: u32 = 1 << 17;
/// Sky objects at least this wide (metres from their centre) are domes over the whole sky; the
/// glow band and the sun are smaller.
pub(crate) const SKY_DOME_RADIUS: f32 = 300.0;
/// A dome is centred on the viewer: its vertices' mean lies within this fraction of its radius
/// (a hemisphere's is about half; the night layer `0x040000B4`'s is 0.36). The moon and the sun
/// are flat meshes set out at their radius (the moon `0x040000B6` spans 738 to 972 m, mean
/// 802 m), so their mean lies near it and they are never domes.
const SKY_DOME_CENTRED: f32 = 0.75;

/// What a sky object's layer is in the modern sky.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkyRole {
    /// The day dome (`crate::sky_backdrop`): only its texture's detail.
    Backdrop,
    /// A dome-sized alpha layer centred on the viewer (the night layer `0x040000B4`): a tint over
    /// the atmosphere.
    Veil,
    /// Drawn as authored; additive layers shine (`FLAG_SKY_EMISSIVE`).
    Layer,
}

impl SkyRole {
    /// The role of `pass` of a sky object whose vertices reach `radius` from its origin and
    /// average at `center` (model space). Only a viewer-centred dome is a veil: a distant
    /// billboard such as the moon keeps its authored `SRCALPHA`, `INVSRCALPHA` blend, so it covers
    /// the sky objects `LScapeSky::Draw` drew before it (the glow band) as retail did.
    pub(crate) fn of(pass: &LayerPass, backdrop: bool, radius: f32, center: Vec3) -> Self {
        if backdrop {
            Self::Backdrop
        } else if pass.source_blend == 5 && pass.dest_blend == 6 && sky_dome(radius, center) {
            Self::Veil
        } else {
            Self::Layer
        }
    }
}

/// Whether a sky object whose vertices reach `radius` from its origin and average at `center` is a
/// dome over the whole sky: wide, and centred on the viewer it is placed at.
pub(crate) fn sky_dome(radius: f32, center: Vec3) -> bool {
    radius >= SKY_DOME_RADIUS && center.length() <= radius * SKY_DOME_CENTRED
}

/// Marks a draw as a DAT sky object (`DrawGroup::sky`) in its `role`.
pub(crate) fn mark_sky(uniform: &mut DrawUniform, pass: &LayerPass, role: SkyRole) {
    uniform.program[1] |= FLAG_SKY;
    uniform.program[1] |= match role {
        SkyRole::Backdrop => FLAG_SKY_BACKDROP,
        SkyRole::Veil => FLAG_SKY_VEIL,
        SkyRole::Layer if pass.source_blend == 5 && pass.dest_blend == 2 => FLAG_SKY_EMISSIVE,
        SkyRole::Layer => 0,
    };
}

pub(crate) fn draw_uniform(pass: &LayerPass) -> DrawUniform {
    let program = match pass.program {
        VertexProgram::Unlit => 0,
        VertexProgram::Lit => 1,
        VertexProgram::PixelLit => 2,
        VertexProgram::LitVertexDiffuse => 3,
        VertexProgram::PixelLitMultiple => 4,
    };
    let f = &pass.flags;
    let bits = [
        f.material_diffuse,
        f.vertex_diffuse,
        f.modulate_vertex_diffuse,
        f.specular,
        pass.fog,
        f.diffuse_fog_amount,
        f.alpha_fog,
        f.ignore_directional,
        pass.specular_enable,
        f.depth_fog,
        f.pixel_specular,
        f.unmodulated_color1,
        f.dynamic,
        f.ignore_point,
    ]
    .iter()
    .enumerate()
    .fold(0_u32, |acc, (bit, on)| acc | (u32::from(*on) << bit))
        | (u32::from(f.display_lit) << FLAG_DISPLAY_LIT_BIT)
        | (u32::from(f.display_glow) << FLAG_DISPLAY_GLOW_BIT);
    let mut uniform = DrawUniform {
        ambient: pass.ambient,
        diffuse: pass.diffuse,
        specular: pass.specular,
        texture_factor: pass.texture_factor,
        program: [
            program,
            bits,
            pass.alpha_test.unwrap_or(256).min(256),
            pass.power.to_bits(),
        ],
        color_ops: [[1, 2, 1, 1]; 4],
        alpha_ops: [[2, 1, 0xFFFF_FFFF, 0]; 4],
        matrices: [[0.0; 4]; 8],
        point_count: [0; 4],
        points: [[0.0; 4]; 16],
    };
    for (index, stage) in pass.stages.iter().enumerate().take(4) {
        uniform.color_ops[index] = [
            stage.color_op,
            stage.color_arg1,
            stage.color_arg2,
            stage.alpha_op,
        ];
        uniform.alpha_ops[index] = [
            stage.alpha_arg1,
            stage.alpha_arg2,
            stage.tex_coord,
            u32::from(stage.texture_matrix.is_some()),
        ];
        if let Some([row0, row1]) = stage.texture_matrix {
            uniform.matrices[index * 2] = row0;
            uniform.matrices[index * 2 + 1] = row1;
        }
    }
    uniform
}
