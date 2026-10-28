//! The static-object pass: one pipeline per blend/depth/cull state (`SetSourceBlend`,
//! `SetDestBlend`, `SetDepthWrite`, `SetDepthTest`, `D3DRS_CULLMODE`), one draw per material layer
//! (`RenderIndexedUsingMaterial` draws each drawn layer in turn), with that layer's material,
//! program and texture stages in a dynamic-offset uniform. A posed instance draws each part at its
//! `CPhysicsPart::m_mObject`, and parts with matrix weights blend their vertices by their group's
//! deform matrices (`SceneTool::SetMatrixStack`), read from a per-frame storage buffer.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;

use ac2_math::{Mat4, Vec3};
use ac2_render::object::{
    LayerPass, ObjectEnvironment, ObjectMaterial, ObjectModel, ObjectPose, ObjectVertex, PointLight,
};
use ac2_render::{Camera, Image};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::object_draw::{
    bone_storage, draw_storage, draw_uniform, mark_sky, set_point_lights, DrawUniform, SkyRole,
};
use crate::object_pipelines::PipelineCompiler;
use crate::object_state::{
    pipeline_key, sampler_key, sky_depth, sky_veil, PipelineKey, SamplerKey, DRAW_STRIDE,
};
use crate::pixel_program::shade_function;
use crate::shading::FRAME_SHADING;
use crate::texture::{TextureCache, TextureKey};
use crate::Gpu;

#[path = "object_shadow.rs"]
mod shadow;

#[path = "object_prepare.rs"]
mod prepare;
#[path = "object_record.rs"]
mod record;
#[path = "object_setup.rs"]
mod setup;
#[path = "object_upload.rs"]
mod upload;

#[cfg(test)]
#[path = "object_stream_tests.rs"]
mod stream_tests;

#[cfg(test)]
#[path = "decal_draw_tests.rs"]
mod decal_draw_tests;
#[cfg(test)]
#[path = "streak_strip_tests.rs"]
mod streak_strip_tests;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    uvs: [[f32; 2]; 4],
    tangents: [[f32; 3]; 2],
    /// Up to four (bone, weight) influences; all-zero weights for a rigid part.
    bones: [u8; 4],
    weights: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FrameUniform {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    eye: [f32; 4],
    to_viewer: [f32; 4],
    sun_inv_dir: [f32; 4],
    sun_diffuse: [f32; 4],
    sun_specular: [f32; 4],
    ambient: [f32; 4],
    fog_color: [f32; 4],
    fog: [f32; 4],
    shading: [f32; 4],
    /// World to the landscape's static shadow map (`crate::StaticShadowMap`).
    shadow: [[f32; 4]; 4],
}

/// Byte stride of one draw group's frame uniform (the dynamic-offset alignment; the frame
/// uniform is 272 bytes).
const FRAME_STRIDE: u64 = 512;
/// Draw groups one `prepare` lays out.
const MAX_GROUPS: usize = 16;
/// Bytes of one bone matrix in the storage buffer.
const BONE_BYTES: u64 = 64;

/// One drawn layer of a batch: its state and the bind group of its stage textures.
struct GpuPass {
    pass: LayerPass,
    /// Shader module index: 0 for the stage cascade, else the layer's pixel shader.
    program: u32,
    textures: wgpu::BindGroup,
}

struct GpuBatch {
    range: Range<u32>,
    material: Arc<ObjectMaterial>,
    passes: Vec<GpuPass>,
}

struct GpuPart {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    transform: Mat4,
    batches: Vec<GpuBatch>,
    /// The mesh carries matrix weights.
    weighted: bool,
    /// The buffers were made by `set_geometry` (`COPY_DST`, power-of-two capacity) and are
    /// refilled in place from then on.
    streamed: bool,
}

/// An uploaded `ObjectModel`.
pub struct GpuObjectModel {
    parts: Vec<GpuPart>,
    /// Model-space centre of the vertices, for back-to-front ordering of translucent layers.
    center: Vec3,
    /// Index and vertex counts of the first part's first triangle list (`FrillRender` budgets).
    first_list: (u32, u32),
    /// The farthest vertex from the model's origin.
    radius: f32,
}

/// Every `GpuVertex` is a whole number of 4-byte words, so `write_buffer` sizes need no padding.
const _: () = assert!(std::mem::size_of::<GpuVertex>().is_multiple_of(4));

/// A buffer for `set_geometry`: at least one copy alignment unit, rounded up to a power of two.
fn stream_buffer(
    device: &wgpu::Device,
    label: &str,
    usage: wgpu::BufferUsages,
    bytes: u64,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.max(wgpu::COPY_BUFFER_ALIGNMENT).next_power_of_two(),
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

impl GpuObjectModel {
    /// The farthest vertex from the model's origin (its bounding sphere about the origin).
    pub(crate) fn bounding_radius(&self) -> f32 {
        self.radius
    }

    /// Index and vertex counts of the first part's first triangle list.
    pub fn first_list(&self) -> (u32, u32) {
        self.first_list
    }
}

/// The sky's backdrop among a group of DAT sky objects: the largest, the dome `LScapeSky` draws
/// first (`0x04000022` in the shipped region). The modern sky takes its colour from the
/// atmosphere and grades it by this object's tint; the dome itself adds only its texture's
/// detail (`FLAG_SKY_BACKDROP`).
pub fn sky_backdrop(instances: &[ObjectInstance<'_>]) -> Option<usize> {
    instances
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.model.radius.total_cmp(&b.1.model.radius))
        .map(|(index, _)| index)
}

/// One draw of the frame: pipeline, uniform offset, and instance/part/batch/layer indices.
#[derive(Clone, Copy)]
struct QueuedDraw {
    key: PipelineKey,
    /// The instance waits for a pipeline still compiling, so none of its draws record.
    deferred: bool,
    offset: u32,
    instance: usize,
    part: usize,
    batch: usize,
    layer: usize,
}

/// One placed model.
#[derive(Clone, Copy)]
pub struct ObjectInstance<'a> {
    pub model: &'a GpuObjectModel,
    pub world: Mat4,
    /// Material tint (`CPhysicsObj` diffuse colour and opacity); white leaves it unchanged.
    pub tint: [f32; 4],
    /// A grass frill merged by `FrillRender`: every vertex takes this world normal and the
    /// landscape shadow colour under its origin (`FrillRenderType::AddPos`).
    pub frill_normal: Option<Vec3>,
    /// The landscape self shadow under a merged frill's origin (`GetShadowColor`'s self term);
    /// 1 leaves it unshadowed.
    pub frill_shade: f32,
    /// The device's point lights for this object (`SceneTool::CacheStaticLights`), at most 4.
    pub point_lights: &'a [PointLight],
    /// The model posed for this draw (`ObjectModel::pose`); `None` draws its default placement.
    pub pose: Option<&'a ObjectPose>,
}

impl<'a> ObjectInstance<'a> {
    /// An untinted instance.
    pub fn new(model: &'a GpuObjectModel, world: Mat4) -> Self {
        Self {
            model,
            world,
            tint: [1.0; 4],
            frill_normal: None,
            frill_shade: 1.0,
            point_lights: &[],
            pose: None,
        }
    }
}

/// Instances laid out together; `ordered` keeps their order instead of sorting translucency.
#[derive(Clone, Copy)]
pub struct DrawGroup<'a> {
    pub instances: &'a [ObjectInstance<'a>],
    pub ordered: bool,
    /// The environment the group draws in (`WorldRender::SetupEnvironment` before it); `None`
    /// uses the frame's.
    pub environment: Option<&'a ObjectEnvironment>,
    /// The instances are DAT sky objects (`LScapeSky::Draw`): no depth test or write.
    pub sky: bool,
}

pub struct ObjectRenderer {
    /// Shader module indices by translated pixel shader: 0 is the stage cascade, then one per
    /// distinct program, compiled with the pipelines on the compiler's worker.
    program_ids: HashMap<String, u32>,
    pipelines: PipelineCompiler,
    /// Whether `prepare` leaves out an instance whose pipelines are still compiling (the
    /// running client) rather than waiting for them (headless renders and tests).
    stream_pipelines: bool,
    frame_buffer: wgpu::Buffer,
    frame_group: wgpu::BindGroup,
    frame_layout: wgpu::BindGroupLayout,
    shadow_sampler: wgpu::Sampler,
    /// World to the static shadow map, for the frills' landscape shadow colour.
    shadow_matrix: Mat4,
    draw_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    draw_buffer: wgpu::Buffer,
    draw_group: wgpu::BindGroup,
    draw_capacity: u64,
    bone_layout: wgpu::BindGroupLayout,
    bone_buffer: wgpu::Buffer,
    bone_group: wgpu::BindGroup,
    bone_capacity: u64,
    samplers: HashMap<SamplerKey, wgpu::Sampler>,
    pub textures: TextureCache,
    time: f64,
    /// Laid out by `prepare`, recorded by `draw`.
    queue: Vec<QueuedDraw>,
    /// Ranges of `queue` by draw group.
    groups: Vec<Range<usize>>,
    /// The static shadow map's caster pipeline, made on first use.
    casters: Option<shadow::CasterPass>,
    shadow_format: wgpu::TextureFormat,
    /// `prepare`'s working storage, cleared and refilled each frame so steady frames allocate
    /// nothing. Never shrunk.
    scratch_entries: Vec<(DrawUniform, QueuedDraw)>,
    scratch_opaque: Vec<(DrawUniform, QueuedDraw)>,
    scratch_translucent: Vec<(f32, (DrawUniform, QueuedDraw))>,
    scratch_bones: Vec<[[f32; 4]; 4]>,
    scratch_bytes: Vec<u8>,
    /// `set_geometry`'s converted vertices.
    scratch_vertices: Vec<GpuVertex>,
}

/// The object shader with a `shade` function appended.
fn frame_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    shadow_map: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ac2 object frame"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<FrameUniform>() as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(shadow_map),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

pub(crate) fn object_module(device: &wgpu::Device, shade: &str) -> wgpu::ShaderModule {
    let source = format!(
        "{}\n{}\n{}\n{}",
        include_str!("shading.wgsl"),
        include_str!("object.wgsl"),
        include_str!("object_modern.wgsl"),
        shade
    );
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ac2 object"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}
