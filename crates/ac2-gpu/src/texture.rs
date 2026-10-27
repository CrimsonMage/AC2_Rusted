//! `RenderTexture` uploads, one GPU texture per AC2 DID, shared by every batch that samples it.

use std::collections::HashMap;

use std::sync::atomic::{AtomicU64, Ordering};

use ac2_render::{BlockCompressedTextureFormat, Image};

pub use ac2_render::TextureKey;

/// Uploaded textures by key; a key whose image cannot be loaded maps to a 1x1 white texture.
pub struct TextureCache {
    views: HashMap<TextureKey, wgpu::TextureView>,
    white: wgpu::TextureView,
    neutral: wgpu::TextureView,
}

impl TextureCache {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let white = upload(
            device,
            queue,
            &Image::rgba(vec![ac2_render::ImageLevel {
                width: 1,
                height: 1,
                rgba: [255_u8; 4].into(),
            }]),
        );
        let neutral = upload(
            device,
            queue,
            &Image::rgba(vec![ac2_render::ImageLevel {
                width: 1,
                height: 1,
                rgba: [128, 128, 128, 255].into(),
            }]),
        );
        Self {
            views: HashMap::new(),
            white,
            neutral,
        }
    }

    pub fn white(&self) -> &wgpu::TextureView {
        &self.white
    }

    /// Opaque mid grey (0.5): what an authored pixel shader's `tex` reads from a stage with no
    /// texture ([`crate::ObjectRenderer`]'s stage binding).
    pub fn neutral(&self) -> &wgpu::TextureView {
        &self.neutral
    }

    /// The view for `key`, uploading the image `load` returns on first use.
    pub fn get(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: TextureKey,
        load: &mut dyn FnMut(TextureKey) -> Option<Image>,
    ) -> wgpu::TextureView {
        if let Some(view) = self.views.get(&key) {
            return view.clone();
        }
        let view = match load(key) {
            Some(image) => upload(device, queue, &image),
            None => self.white.clone(),
        };
        self.views.insert(key, view.clone());
        view
    }
}

/// GPU bytes of every texture uploaded so far, and of those kept as BC blocks.
static UPLOADED_BYTES: AtomicU64 = AtomicU64::new(0);
static UPLOADED_BLOCK_BYTES: AtomicU64 = AtomicU64::new(0);

/// `(all, block-compressed)` texture bytes uploaded since the start (bench statistics).
pub fn uploaded_texture_bytes() -> (u64, u64) {
    (
        UPLOADED_BYTES.load(Ordering::Relaxed),
        UPLOADED_BLOCK_BYTES.load(Ordering::Relaxed),
    )
}

fn upload(device: &wgpu::Device, queue: &wgpu::Queue, image: &Image) -> wgpu::TextureView {
    if let Some(blocks) = &image.blocks {
        return upload_blocks(device, queue, blocks);
    }
    let size = wgpu::Extent3d {
        width: image.width(),
        height: image.height(),
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ac2 texture"),
        size,
        mip_level_count: image.levels.len() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Gamma-space texels, sampled and blended without sRGB conversion like D3D8.
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (mip, level) in image.levels.iter().enumerate() {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: mip as u32,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &level.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(level.width * 4),
                rows_per_image: Some(level.height),
            },
            wgpu::Extent3d {
                width: level.width,
                height: level.height,
                depth_or_array_layers: 1,
            },
        );
        UPLOADED_BYTES.fetch_add(level.rgba.len() as u64, Ordering::Relaxed);
    }
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// A DXT chain uploaded as BC1/BC2/BC3 (the non-sRGB formats: gamma-space texels as the RGBA8
/// path and D3D8 sample them). Each level is copied at its physical size, whole blocks, so a
/// mip below 4 x 4 is one padded block.
fn upload_blocks(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &ac2_render::BlockImage,
) -> wgpu::TextureView {
    let format = match image.format {
        BlockCompressedTextureFormat::Bc1Rgba => wgpu::TextureFormat::Bc1RgbaUnorm,
        BlockCompressedTextureFormat::Bc2Rgba => wgpu::TextureFormat::Bc2RgbaUnorm,
        BlockCompressedTextureFormat::Bc3Rgba => wgpu::TextureFormat::Bc3RgbaUnorm,
    };
    let block = ac2_render::block_bytes(image.format) as u32;
    let first = &image.levels[0];
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ac2 texture (BC)"),
        size: wgpu::Extent3d {
            width: first.width,
            height: first.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: image.levels.len() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (mip, level) in image.levels.iter().enumerate() {
        let (columns, rows) = (level.width.div_ceil(4), level.height.div_ceil(4));
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: mip as u32,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &level.blocks,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(columns * block),
                rows_per_image: Some(rows),
            },
            wgpu::Extent3d {
                width: columns * 4,
                height: rows * 4,
                depth_or_array_layers: 1,
            },
        );
        UPLOADED_BYTES.fetch_add(level.blocks.len() as u64, Ordering::Relaxed);
        UPLOADED_BLOCK_BYTES.fetch_add(level.blocks.len() as u64, Ordering::Relaxed);
    }
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

#[cfg(test)]
#[path = "texture_tests.rs"]
mod tests;
