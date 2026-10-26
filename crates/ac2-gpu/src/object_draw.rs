//! The per-draw uniform block of the object pass (`RenderDeviceD3D::SetupVertexShaderConstants`
//! and `SetupState` for one layer) and the buffers that hold the frame's draws and bone matrices.

use ac2_math::Mat4;
use ac2_render::object::{LayerPass, PointLight, VertexProgram, MAX_POINT_LIGHTS};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct DrawUniform {
    model: [[f32; 4]; 4],
    ambient: [f32; 4],
    diffuse: [f32; 4],
    specular: [f32; 4],
    texture_factor: [f32; 4],
    program: [u32; 4],
    color_ops: [[u32; 4]; 4],
    alpha_ops: [[u32; 4]; 4],
    matrices: [[f32; 4]; 8],
    /// A frill's normal for every vertex (xyz) when w is 1 (`RefreshFrillMesh`).
    pub(crate) frill: [f32; 4],
    point_count: [u32; 4],
    /// Per point light: origin and range^2, diffuse, specular, attenuation.
    points: [[f32; 4]; 16],
    /// x: the part's first matrix in the bone buffer, y: 1 when its vertices blend.
    pub(crate) skin: [u32; 4],
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

pub(crate) fn draw_storage(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    size: u64,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ac2 object draws"),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ac2 object draws"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<DrawUniform>() as u64),
            }),
        }],
    });
    (buffer, group)
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

/// `FLAG_SKY` in `object.wgsl`: the draw is a DAT sky object.
const FLAG_SKY: u32 = 1 << 14;
/// `FLAG_SKY_BACKDROP`: the sky object is the backdrop dome ([`crate::sky_backdrop`]).
const FLAG_SKY_BACKDROP: u32 = 1 << 15;

/// `FLAG_SKY_EMISSIVE`: an additive (`SRCALPHA`, `ONE`) sky layer, light in the sky.
const FLAG_SKY_EMISSIVE: u32 = 1 << 16;

/// Marks a draw as a DAT sky object (`DrawGroup::sky`), as its backdrop dome, and as
/// emissive when it adds light (`SRCALPHA`, `ONE`).
pub(crate) fn mark_sky(uniform: &mut DrawUniform, pass: &LayerPass, backdrop: bool) {
    uniform.program[1] |= FLAG_SKY;
    if backdrop {
        uniform.program[1] |= FLAG_SKY_BACKDROP;
    } else if pass.source_blend == 5 && pass.dest_blend == 2 {
        uniform.program[1] |= FLAG_SKY_EMISSIVE;
    }
}

pub(crate) fn draw_uniform(model: Mat4, pass: &LayerPass) -> DrawUniform {
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
    .fold(0_u32, |acc, (bit, on)| acc | (u32::from(*on) << bit));
    let mut uniform = DrawUniform {
        model: model.to_cols_array_2d(),
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
        frill: [0.0; 4],
        point_count: [0; 4],
        points: [[0.0; 4]; 16],
        skin: [0; 4],
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
