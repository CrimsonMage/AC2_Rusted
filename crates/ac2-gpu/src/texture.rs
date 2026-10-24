//! `RenderTexture` uploads, one GPU texture per AC2 DID, shared by every batch that samples it.

use std::collections::HashMap;

use ac2_core::DataId;
use ac2_render::Image;

/// What a texture is for: a `RenderMaterial`'s base texture or an alpha mask `RenderTexture`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureKey {
    Material(DataId),
    Mask(DataId),
}

/// Uploaded textures by key; a key whose image cannot be loaded maps to a 1x1 white texture.
pub struct TextureCache {
    views: HashMap<TextureKey, wgpu::TextureView>,
    white: wgpu::TextureView,
}

impl TextureCache {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let white = upload(
            device,
            queue,
            &Image {
                levels: vec![ac2_render::ImageLevel {
                    width: 1,
                    height: 1,
                    rgba: [255_u8; 4].into(),
                }],
            },
        );
        Self {
            views: HashMap::new(),
            white,
        }
    }

    pub fn white(&self) -> &wgpu::TextureView {
        &self.white
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

fn upload(device: &wgpu::Device, queue: &wgpu::Queue, image: &Image) -> wgpu::TextureView {
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
    }
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
