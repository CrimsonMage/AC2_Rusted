//! Retail render-state enums as wgpu state: `BlendMode`, `DepthTestMode`, `CullMode` for the
//! pipelines, `TexAddress` and `TexFilterMode` for the stage samplers.

use ac2_render::object::{blend, LayerPass, StagePass};

use crate::DEPTH_FORMAT;

/// Byte stride of one draw's uniform block (the dynamic-offset alignment).
pub const DRAW_STRIDE: u64 = 512;

/// Fixed-function state that selects a pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    source_blend: u32,
    dest_blend: u32,
    depth_write: bool,
    depth_test: u32,
    cull_mode: u32,
}

pub fn pipeline_key(pass: &LayerPass) -> PipelineKey {
    PipelineKey {
        source_blend: pass.source_blend,
        dest_blend: pass.dest_blend,
        depth_write: pass.depth_write,
        depth_test: pass.depth_test,
        cull_mode: pass.cull_mode,
    }
}

impl PipelineKey {
    pub fn create(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        module: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        // `SetSourceBlend`: ALPHABLENDENABLE unless the blend is ONE, ZERO.
        let blending = !(self.source_blend == blend::ONE && self.dest_blend == blend::ZERO);
        let component = wgpu::BlendComponent {
            src_factor: blend_factor(self.source_blend),
            dst_factor: blend_factor(self.dest_blend),
            operation: wgpu::BlendOperation::Add,
        };
        let stride = (3 + 3 + 4 + 8) * 4;
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ac2 object"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3, 1 => Float32x3, 2 => Float32x4,
                        3 => Float32x2, 4 => Float32x2, 5 => Float32x2, 6 => Float32x2
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: blending.then_some(wgpu::BlendState {
                        color: component,
                        alpha: component,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                // The retail view (AC to D3D axes, left-handed camera) shows the same screen winding
                // as this right-handed one, so `CULLMODE_CW` culls clockwise (back) faces.
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: match self.cull_mode {
                    2 => Some(wgpu::Face::Back),
                    3 => Some(wgpu::Face::Front),
                    _ => None,
                },
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: self.depth_write,
                depth_compare: depth_compare(self.depth_test),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        })
    }
}

fn blend_factor(mode: u32) -> wgpu::BlendFactor {
    use wgpu::BlendFactor as F;
    match mode {
        1 => F::Zero,
        2 => F::One,
        3 => F::Src,
        4 => F::OneMinusSrc,
        5 => F::SrcAlpha,
        6 => F::OneMinusSrcAlpha,
        7 => F::DstAlpha,
        8 => F::OneMinusDstAlpha,
        9 => F::Dst,
        10 => F::OneMinusDst,
        11 => F::SrcAlphaSaturated,
        _ => F::One,
    }
}

fn depth_compare(mode: u32) -> wgpu::CompareFunction {
    use wgpu::CompareFunction as C;
    match mode {
        1 => C::Never,
        2 => C::Less,
        3 => C::Equal,
        5 => C::Greater,
        6 => C::NotEqual,
        7 => C::GreaterEqual,
        8 => C::Always,
        _ => C::LessEqual,
    }
}

/// A stage's addressing and filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerKey {
    address_u: u32,
    address_v: u32,
    min_filter: u32,
    mag_filter: u32,
}

pub fn sampler_key(stage: Option<&StagePass>) -> SamplerKey {
    stage.map_or(
        SamplerKey {
            address_u: 1,
            address_v: 1,
            min_filter: 2,
            mag_filter: 2,
        },
        |stage| SamplerKey {
            address_u: stage.address_u,
            address_v: stage.address_v,
            min_filter: stage.min_filter,
            mag_filter: stage.mag_filter,
        },
    )
}

impl SamplerKey {
    /// `D3DTSS_MIPFILTER` is LINEAR when the trilinear preference is on (taken as on here).
    pub fn create(&self, device: &wgpu::Device) -> wgpu::Sampler {
        let address = |mode: u32| match mode {
            2 => wgpu::AddressMode::MirrorRepeat,
            3 | 4 => wgpu::AddressMode::ClampToEdge,
            _ => wgpu::AddressMode::Repeat,
        };
        let filter = |mode: u32| {
            if mode <= 1 {
                wgpu::FilterMode::Nearest
            } else {
                wgpu::FilterMode::Linear
            }
        };
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ac2 stage sampler"),
            address_mode_u: address(self.address_u),
            address_mode_v: address(self.address_v),
            mag_filter: filter(self.mag_filter),
            min_filter: filter(self.min_filter),
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })
    }
}
