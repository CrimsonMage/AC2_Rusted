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
        let sun = crate::sun_shadow::SunShadowBinding::layout_entries(3);
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
                sun[0],
                sun[1],
                sun[2],
            ],
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 object draw"),
            entries: &[
                uniform(0, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
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
        let sun_shadow = crate::sun_shadow::SunShadowBinding::placeholder(gpu);
        let frame_group = frame_group(
            device,
            &frame_layout,
            &frame_buffer,
            textures.white(),
            &shadow_sampler,
            &sun_shadow,
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
        let draws = draw_storage(device, &draw_layout, DRAW_STRIDE * 64, INSTANCE_BYTES * 64);
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
            draws,
            draw_capacity: DRAW_STRIDE * 64,
            instance_capacity: INSTANCE_BYTES * 64,
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
            sun_casters: None,
            sun_shadow,
            shadow_view: None,
            scratch_entries: Vec::new(),
            scratch_opaque: Vec::new(),
            scratch_translucent: Vec::new(),
            scratch_instances: Vec::new(),
            scratch_model_ranks: HashMap::new(),
            scratch_order: Vec::new(),
            scratch_waiting: HashSet::new(),
            scratch_last_uniform: None,
            scratch_bones: Vec::new(),
            scratch_bytes: Vec::new(),
            scratch_vertices: Vec::new(),
        }
    }

    /// Uploads a model; `load` supplies images for stage textures not yet on the GPU.
    /// Binds the landscape's static shadow map, which merged frills read their colour from.
    pub fn set_shadow_map(&mut self, gpu: &Gpu, view: &wgpu::TextureView) {
        self.shadow_view = Some(view.clone());
        self.rebind_frame(gpu);
    }

    /// Binds the sun shadow cascades the objects receive.
    pub fn set_sun_shadow(&mut self, gpu: &Gpu, sun: &crate::sun_shadow::SunShadowBinding) {
        self.sun_shadow = sun.clone();
        self.rebind_frame(gpu);
    }

    fn rebind_frame(&mut self, gpu: &Gpu) {
        let white = self.textures.white().clone();
        self.frame_group = frame_group(
            &gpu.device,
            &self.frame_layout,
            &self.frame_buffer,
            self.shadow_view.as_ref().unwrap_or(&white),
            &self.shadow_sampler,
            &self.sun_shadow,
        );
    }

    /// The world-to-shadow-map transform for the next `prepare` (zero without a map).
    pub fn set_shadow_matrix(&mut self, matrix: Mat4) {
        self.shadow_matrix = matrix;
    }
}
