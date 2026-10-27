//! The BC upload against the RGBA8 one: seeded random DXT1, DXT3 and DXT5 blocks (both DXT1
//! colour modes, every interpolant) uploaded as blocks and as the CPU decoder's RGBA8, read back
//! texel for texel. Needs a GPU adapter that samples BC formats.
//!
//! Hardware BC decoders round the 1/3, 2/3 and 1/2 interpolants their own way (the graphics APIs
//! leave them implementation-defined within a few percent), so the GPU and the CPU decoder may
//! differ by a few levels on interpolated texels; the endpoints and DXT3's explicit alpha match.
//! Neither is the retail reference: retail uploaded DXT straight to D3D8, so players saw their
//! card's decoder, which is what this path keeps. Measured on a desktop Vulkan GPU over random
//! blocks: 30-70% of the texels differ, by at most 5 levels (6 for BC3's interpolated alpha). In
//! the lit, tone-mapped world-viewer scene 89% of the channels are identical and the worst is 4.

use ac2_render::{BlockCompressedTextureFormat, BlockImage, BlockLevel, Image, ImageLevel};

use super::upload;
use crate::frame::{FrameTarget, COLOR_FORMAT, DEPTH_FORMAT};
use crate::Gpu;

const SIZE: u32 = 32;
/// About 3% of the channel range: the interpolant rounding hardware decoders are allowed.
const TOLERANCE: u8 = 8;

const SHADER: &str = "
@group(0) @binding(0) var source: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(source, vec2<i32>(at.xy), 0);
}
";

/// A xorshift stream, so the blocks are the same on every run.
fn random_bytes(seed: u64, count: usize) -> Vec<u8> {
    let mut state = seed;
    (0..count)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}

/// `view`'s texels, copied texel for texel into an RGBA8 target and read back.
fn read_texels(gpu: &Gpu, view: &wgpu::TextureView) -> Vec<u8> {
    let device = &gpu.device;
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("texel copy"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("texel copy"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(COLOR_FORMAT.into())],
        }),
        primitive: Default::default(),
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
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("texel copy"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(view),
        }],
    });
    let target = FrameTarget::new(device, SIZE, SIZE);
    target.render(device, &gpu.queue, [0.0; 3], |pass| {
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    });
    target.read_rgba(device, &gpu.queue)
}

#[test]
fn bc_uploads_match_the_cpu_decode_within_the_interpolant_rounding() {
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    if !gpu.block_compression() {
        eprintln!("the adapter does not sample BC formats; skipping");
        return;
    }
    let formats = [
        BlockCompressedTextureFormat::Bc1Rgba,
        BlockCompressedTextureFormat::Bc2Rgba,
        BlockCompressedTextureFormat::Bc3Rgba,
    ];
    for (seed, format) in formats.into_iter().enumerate() {
        let blocks_per_side = (SIZE / 4) as usize;
        let bytes = blocks_per_side * blocks_per_side * ac2_render::block_bytes(format);
        let blocks = random_bytes(0x9E37_79B9_7F4A_7C15 ^ seed as u64, bytes);
        let rgba = ac2_assets::decode_block_compressed_texture(
            format,
            SIZE as usize,
            SIZE as usize,
            &blocks,
        )
        .expect("whole blocks decode");
        let as_blocks = Image {
            levels: Vec::new(),
            blocks: Some(BlockImage {
                format,
                levels: vec![BlockLevel {
                    width: SIZE,
                    height: SIZE,
                    blocks: blocks.into(),
                }],
            }),
        };
        let as_rgba = Image::rgba(vec![ImageLevel {
            width: SIZE,
            height: SIZE,
            rgba: rgba.into(),
        }]);
        let gpu_decoded = read_texels(&gpu, &upload(&gpu.device, &gpu.queue, &as_blocks));
        let cpu_decoded = read_texels(&gpu, &upload(&gpu.device, &gpu.queue, &as_rgba));
        let worst = gpu_decoded
            .iter()
            .zip(&cpu_decoded)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        let differing = gpu_decoded
            .chunks(4)
            .zip(cpu_decoded.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        eprintln!("{format:?}: {differing} texels differ, worst channel {worst}");
        assert!(
            worst <= TOLERANCE,
            "{format:?}: a channel differs by {worst}"
        );
    }
}
