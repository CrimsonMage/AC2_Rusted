//! The scene in linear light and its one tone-map pass to the display.
//!
//! A deliberate deviation from retail (`AGENTS.md`): the world, its sky and the UI viewports'
//! scenes draw into an [`HdrTarget`] (`Rgba16Float`) in linear light, with each layer's colour
//! unclamped, so stacked additive sky layers keep their gradients instead of saturating as they
//! did in the D3D8 8-bit back buffer. [`ToneMap`] then writes the display colour (the Khronos PBR Neutral
//! curve, then sRGB encode) once, before the UI's
//! surfaces draw over it. What is drawn, in which order and with which blend stays retail's.

use crate::ao::{AoCamera, AoPipelines};
use crate::post::{PostChain, PostPipelines};
use crate::{Gpu, DEPTH_FORMAT};

/// The scene colour format: linear light, unclamped.
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// A scene colour buffer and its depth, the size of the output it tone maps into.
pub struct HdrTarget {
    pub width: u32,
    pub height: u32,
    pub color_view: wgpu::TextureView,
    pub depth_view: wgpu::TextureView,
    group: wgpu::BindGroup,
}

impl HdrTarget {
    pub fn new(gpu: &Gpu, tonemap: &ToneMap, width: u32, height: u32) -> Self {
        let (width, height) = (width.max(1), height.max(1));
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = |format, usage, label| {
            gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let color = texture(
            HDR_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            "ac2 hdr scene colour",
        );
        let depth = texture(
            DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            "ac2 hdr scene depth",
        );
        let color_view = color.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());
        let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ac2 tone map source"),
            layout: &tonemap.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth_view),
                },
            ],
        });
        Self {
            width,
            height,
            color_view,
            depth_view,
            group,
        }
    }

    /// Whether the target matches an output of `width` x `height`.
    pub fn fits(&self, width: u32, height: u32) -> bool {
        (self.width, self.height) == (width.max(1), height.max(1))
    }

    /// Records one pass into the scene target: colour cleared to `clear` (linear, with the
    /// coverage alpha) and depth to far, then `record`, limited to `viewport` (x, y, width,
    /// height in pixels; `UIScene::Render`'s viewport) when there is one. Timed as `scene`.
    pub fn render(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        clear: [f32; 4],
        viewport: Option<[u32; 4]>,
        record: impl FnOnce(&mut wgpu::RenderPass<'_>),
    ) {
        let timestamps = gpu.pass_timestamps("scene");
        let [r, g, b, a] = clear;
        let load = wgpu::LoadOp::Clear(wgpu::Color {
            r: f64::from(r),
            g: f64::from(g),
            b: f64::from(b),
            a: f64::from(a),
        });
        let mut pass = begin(
            encoder,
            &self.color_view,
            &self.depth_view,
            load,
            timestamps,
        );
        if let Some([x, y, w, h]) = viewport {
            pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
            pass.set_scissor_rect(x, y, w, h);
        }
        record(&mut pass);
    }

    /// Records one more pass over what the scene target holds: colour kept, depth cleared to far,
    /// so a second scene seen from its own camera (the portal space's, drawn over the world while
    /// it fades) draws over the first without its depth. Timed as `scene`.
    pub fn render_over(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        record: impl FnOnce(&mut wgpu::RenderPass<'_>),
    ) {
        let timestamps = gpu.pass_timestamps("scene");
        let mut pass = begin(
            encoder,
            &self.color_view,
            &self.depth_view,
            wgpu::LoadOp::Load,
            timestamps,
        );
        record(&mut pass);
    }
}

fn begin<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color: &'a wgpu::TextureView,
    depth: &'a wgpu::TextureView,
    color_load: wgpu::LoadOp<wgpu::Color>,
    timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'a>>,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ac2 hdr scene"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: color_load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

/// The tone-map pass into an output of one colour format.
pub struct ToneMap {
    opaque: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    /// The post chain's inputs: bloom, its sampler and the exposure texel (`post.rs`).
    post_layout: wgpu::BindGroupLayout,
    /// Inputs for a scene without the post chain: no bloom, EV 0, no vignette.
    neutral: wgpu::BindGroup,
}

/// The tone map's post inputs: `bloom` (sampled with `sampler`), the one-texel `exposure` and
/// the ambient occlusion `ao`.
pub(crate) fn post_inputs(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    bloom: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    exposure: &wgpu::TextureView,
    ao: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ac2 tone map post"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(bloom),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(exposure),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(ao),
            },
        ],
    })
}

impl ToneMap {
    pub fn new(gpu: &Gpu, output: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let entry = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 tone map"),
            entries: &[
                entry(0, wgpu::TextureSampleType::Float { filterable: false }),
                entry(1, wgpu::TextureSampleType::Depth),
            ],
        });
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ac2 tone map post"),
            entries: &[
                entry(0, wgpu::TextureSampleType::Float { filterable: true }),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                entry(2, wgpu::TextureSampleType::Float { filterable: true }),
                entry(3, wgpu::TextureSampleType::Float { filterable: true }),
            ],
        });
        // One black texel serves as both the bloom and the exposure: EV 0, bloom and vignette 0.
        let black = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("ac2 tone map neutral"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: HDR_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
            .create_view(&Default::default());
        // And one white texel: no occlusion.
        let white = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ac2 tone map no occlusion"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::ao::AO_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            white.as_image_copy(),
            &[255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: None,
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let neutral = post_inputs(
            device,
            &post_layout,
            &black,
            &device.create_sampler(&Default::default()),
            &black,
            &white.create_view(&Default::default()),
        );
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 tone map"),
            source: wgpu::ShaderSource::Wgsl(include_str!("tonemap.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 tone map"),
            bind_group_layouts: &[Some(&layout), Some(&post_layout)],
            immediate_size: 0,
        });
        let pipeline = |entry_point, blend| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ac2 tone map"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entry_point),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: output,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Always),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            opaque: pipeline("fs", None),
            // The scene's colour is already weighted by its coverage (drawn over a clear of 0).
            composite: pipeline(
                "fs_composite",
                Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            ),
            layout,
            post_layout,
            neutral,
        }
    }

    /// Records the tone map of `source` into `pass` (an output pass the size of `source`, with
    /// its depth attachment), replacing the output. A `viewport` instead composites that
    /// rectangle of the scene over the output where the scene drew (`fs_composite`).
    pub fn draw(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        source: &HdrTarget,
        viewport: Option<[u32; 4]>,
    ) {
        self.draw_with(pass, source, viewport, &self.neutral);
    }

    fn draw_with(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        source: &HdrTarget,
        viewport: Option<[u32; 4]>,
        post: &wgpu::BindGroup,
    ) {
        if let Some([x, y, w, h]) = viewport {
            pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
            pass.set_scissor_rect(x, y, w, h);
        }
        pass.set_pipeline(match viewport {
            Some(_) => &self.composite,
            None => &self.opaque,
        });
        pass.set_bind_group(0, &source.group, &[]);
        pass.set_bind_group(1, post, &[]);
        pass.draw(0..3, 0..1);
        if viewport.is_some() {
            pass.set_viewport(
                0.0,
                0.0,
                source.width as f32,
                source.height as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(0, 0, source.width, source.height);
        }
    }
}

/// A tone map for one output format with the HDR target it reads and the world's post chain,
/// remade when the output's size changes (the window resizes freely).
pub struct ScenePresenter {
    tonemap: ToneMap,
    target: std::cell::RefCell<Option<HdrTarget>>,
    post_pipelines: PostPipelines,
    ao_pipelines: AoPipelines,
    post: std::cell::RefCell<Option<PostChain>>,
    /// The world camera's projection for the occlusion, once the view has set it.
    camera: std::cell::Cell<Option<AoCamera>>,
    /// The eye adaptation is held: the exposure does not move ([`Self::hold_adaptation`]).
    held: std::cell::Cell<bool>,
}

impl ScenePresenter {
    pub fn new(gpu: &Gpu, output: wgpu::TextureFormat) -> Self {
        Self {
            tonemap: ToneMap::new(gpu, output),
            target: std::cell::RefCell::new(None),
            post_pipelines: PostPipelines::new(gpu),
            ao_pipelines: AoPipelines::new(gpu),
            post: std::cell::RefCell::new(None),
            camera: std::cell::Cell::new(None),
            held: std::cell::Cell::new(false),
        }
    }

    /// Holds the eye adaptation where it is (`true`) or lets it follow the scene again: renders
    /// compared pixel for pixel keep one exposure.
    pub fn hold_adaptation(&self, held: bool) {
        self.held.set(held);
    }

    /// The world camera's projection for the next [`Self::post`] (its ambient occlusion).
    pub fn set_camera(&self, camera: AoCamera) {
        self.camera.set(Some(camera));
    }

    /// The post chain for `hdr`, made when the target's size changes.
    fn chain(&self, gpu: &Gpu, hdr: &HdrTarget) -> std::cell::Ref<'_, Option<PostChain>> {
        let mut post = self.post.borrow_mut();
        if !post.as_ref().is_some_and(|chain| chain.fits(hdr)) {
            *post = Some(PostChain::new(
                gpu,
                &self.post_pipelines,
                &self.ao_pipelines,
                &self.tonemap.post_layout,
                hdr,
            ));
        }
        drop(post);
        self.post.borrow()
    }

    /// Records the ambient occlusion from `hdr`'s depth as it stands (the world's camera when
    /// `world`, else none: no occlusion). Call it once per frame after the world drew and before
    /// anything clears the depth (the portal space's pass), then [`Self::post_chain`].
    pub fn occlusion(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        hdr: &HdrTarget,
        world: bool,
    ) {
        let camera = if world { self.camera.get() } else { None };
        if let Some(chain) = self.chain(gpu, hdr).as_ref() {
            chain.record_occlusion(gpu, &self.ao_pipelines, encoder, camera);
        }
    }

    /// Records the bloom and eye adaptation over `hdr`, once per frame after everything drew
    /// into it and before [`Self::draw_world`].
    pub fn post_chain(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, hdr: &HdrTarget) {
        if let Some(chain) = self.chain(gpu, hdr).as_ref() {
            chain.record(gpu, &self.post_pipelines, encoder, self.held.get());
        }
    }

    /// The whole post chain for a world drawn alone: [`Self::occlusion`] then
    /// [`Self::post_chain`].
    pub fn post(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, hdr: &HdrTarget) {
        self.occlusion(gpu, encoder, hdr, true);
        self.post_chain(gpu, encoder, hdr);
    }

    /// Tone maps the world in `hdr` over the whole output with the post chain's bloom and
    /// exposure (neutral until [`Self::post`] has run at this size).
    pub fn draw_world(&self, pass: &mut wgpu::RenderPass<'_>, hdr: &HdrTarget) {
        let post = self.post.borrow();
        match post.as_ref().filter(|chain| chain.fits(hdr)) {
            Some(chain) => self
                .tonemap
                .draw_with(pass, hdr, None, chain.tonemap_inputs()),
            None => self.tonemap.draw(pass, hdr, None),
        }
    }

    pub fn tonemap(&self) -> &ToneMap {
        &self.tonemap
    }

    /// The HDR target for an output of `width` x `height`.
    pub fn target(&self, gpu: &Gpu, width: u32, height: u32) -> std::cell::Ref<'_, HdrTarget> {
        let fits = self
            .target
            .borrow()
            .as_ref()
            .is_some_and(|t| t.fits(width, height));
        if !fits {
            *self.target.borrow_mut() = Some(HdrTarget::new(gpu, &self.tonemap, width, height));
        }
        std::cell::Ref::map(self.target.borrow(), |t| t.as_ref().expect("made above"))
    }
}

#[cfg(test)]
mod tests {
    /// `tonemap.wgsl`'s Khronos PBR Neutral curve on one grey channel.
    fn neutral(x: f32) -> f32 {
        let offset = if x < 0.08 { x - 6.25 * x * x } else { 0.04 };
        let c = x - offset;
        if c < 0.76 {
            return c;
        }
        let d = 1.0 - 0.76;
        1.0 - d * d / (c + d - 0.76)
    }

    #[test]
    fn neutral_keeps_black_and_mid_tones_and_rolls_highlights_off_below_one() {
        assert_eq!(neutral(0.0), 0.0);
        assert!((neutral(0.18) - 0.14).abs() < 1e-3, "{}", neutral(0.18));
        assert!(neutral(1.0) < 0.95 && neutral(1.0) > 0.85);
        // Two stacked additive layers at 1.0 each stay distinguishable from one.
        assert!(neutral(2.0) - neutral(1.0) > 0.03);
        assert!(neutral(50.0) < 1.0);
    }
}
