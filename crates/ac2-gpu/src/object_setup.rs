//! Setup of the object pass: the pipeline and bind-group layouts, and the shadow map bindings.

use super::*;

impl ObjectRenderer {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let uniform = |binding, dynamic| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        };
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object frame"),
            entries: &[
                uniform(0, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object draw"),
            entries: &[uniform(0, true)],
        });
        let mut texture_entries = Vec::new();
        for binding in 0..4 {
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
        }
        for binding in 4..8 {
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object stages"),
            entries: &texture_entries,
        });
        let bone_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object bones"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 object frame uniform"),
            size: FRAME_STRIDE * MAX_GROUPS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let textures = TextureCache::new(device, &gpu.queue);
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 object shadow map"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let frame_group = frame_group(
            device,
            &frame_layout,
            &frame_buffer,
            textures.white(),
            &shadow_sampler,
        );
        let module = object_module(device, include_str!("object_stages.wgsl"));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 object"),
            bind_group_layouts: &[
                Some(&frame_layout),
                Some(&draw_layout),
                Some(&texture_layout),
                Some(&bone_layout),
            ],
            immediate_size: 0,
        });
        let (draw_buffer, draw_group) = draw_storage(device, &draw_layout, DRAW_STRIDE * 64);
        let (bone_buffer, bone_group) = bone_storage(device, &bone_layout, BONE_BYTES * 64);
        let pipelines = PipelineCompiler::new(gpu, &layout, format, module);
        Self {
            program_ids: HashMap::new(),
            pipelines,
            stream_pipelines: false,
            frame_buffer,
            frame_group,
            frame_layout,
            shadow_sampler,
            shadow_matrix: Mat4::ZERO,
            draw_layout,
            texture_layout,
            draw_buffer,
            draw_group,
            draw_capacity: DRAW_STRIDE * 64,
            bone_layout,
            bone_buffer,
            bone_group,
            bone_capacity: BONE_BYTES * 64,
            samplers: HashMap::new(),
            textures,
            time: 0.0,
            queue: Vec::new(),
            groups: Vec::new(),
            casters: None,
            shadow_format: crate::SHADOW_MAP_FORMAT,
            scratch_entries: Vec::new(),
            scratch_opaque: Vec::new(),
            scratch_translucent: Vec::new(),
            scratch_bones: Vec::new(),
            scratch_bytes: Vec::new(),
            scratch_vertices: Vec::new(),
        }
    }

    /// Uploads a model; `load` supplies images for stage textures not yet on the GPU.
    /// Binds the landscape's static shadow map, which merged frills read their colour from.
    pub fn set_shadow_map(&mut self, gpu: &Gpu, view: &wgpu::TextureView) {
        self.frame_group = frame_group(
            &gpu.device,
            &self.frame_layout,
            &self.frame_buffer,
            view,
            &self.shadow_sampler,
        );
    }

    /// The world-to-shadow-map transform for the next `prepare` (zero without a map).
    pub fn set_shadow_matrix(&mut self, matrix: Mat4) {
        self.shadow_matrix = matrix;
    }
}
