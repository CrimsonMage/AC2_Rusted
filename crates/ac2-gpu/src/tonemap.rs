//! The scene in linear light and its one tone-map pass to the display.
//!
//! A deliberate deviation from retail (`AGENTS.md`): the world, its sky and the UI viewports'
//! scenes draw into an [`HdrTarget`] (`Rgba16Float`) in linear light, with each layer's colour
//! unclamped, so stacked additive sky layers keep their gradients instead of saturating as they
//! did in the D3D8 8-bit back buffer. [`ToneMap`] then writes the display colour (exposure and
//! contrast about middle grey, ACES filmic, sRGB encode, saturation) once, before the UI's
//! surfaces draw over it. What is drawn, in which order and with which blend stays retail's.

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
    /// height in pixels; `UIScene::Render`'s viewport) when there is one.
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        clear: [f32; 4],
        viewport: Option<[u32; 4]>,
        record: impl FnOnce(&mut wgpu::RenderPass<'_>),
    ) {
        let mut pass = begin(encoder, &self.color_view, &self.depth_view, clear);
        if let Some([x, y, w, h]) = viewport {
            pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
            pass.set_scissor_rect(x, y, w, h);
        }
        record(&mut pass);
    }
}

fn begin<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color: &'a wgpu::TextureView,
    depth: &'a wgpu::TextureView,
    [r, g, b, a]: [f32; 4],
) -> wgpu::RenderPass<'a> {
    let color_load = wgpu::LoadOp::Clear(wgpu::Color {
        r: f64::from(r),
        g: f64::from(g),
        b: f64::from(b),
        a: f64::from(a),
    });
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
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

/// The tone-map pass into an output of one colour format.
pub struct ToneMap {
    opaque: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
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
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ac2 tone map"),
            source: wgpu::ShaderSource::Wgsl(include_str!("tonemap.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ac2 tone map"),
            bind_group_layouts: &[Some(&layout)],
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
        if let Some([x, y, w, h]) = viewport {
            pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
            pass.set_scissor_rect(x, y, w, h);
        }
        pass.set_pipeline(match viewport {
            Some(_) => &self.composite,
            None => &self.opaque,
        });
        pass.set_bind_group(0, &source.group, &[]);
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

/// A tone map for one output format with the HDR target it reads, remade when the output's
/// size changes (the window resizes freely).
pub struct ScenePresenter {
    tonemap: ToneMap,
    target: std::cell::RefCell<Option<HdrTarget>>,
}

impl ScenePresenter {
    pub fn new(gpu: &Gpu, output: wgpu::TextureFormat) -> Self {
        Self {
            tonemap: ToneMap::new(gpu, output),
            target: std::cell::RefCell::new(None),
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
    /// `tonemap.wgsl`'s ACES fit at known points.
    fn aces(x: f32) -> f32 {
        ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0)
    }

    #[test]
    fn aces_keeps_black_and_rolls_highlights_off_below_one() {
        assert_eq!(aces(0.0), 0.0);
        assert!((aces(0.18) - 0.2672).abs() < 1e-3, "{}", aces(0.18));
        assert!(aces(1.0) < 0.81 && aces(1.0) > 0.79);
        // Two stacked additive layers at 1.0 each stay distinguishable from one.
        assert!(aces(2.0) - aces(1.0) > 0.05);
    }
}
