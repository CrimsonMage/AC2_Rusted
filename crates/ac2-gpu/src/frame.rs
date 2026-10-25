//! Offscreen colour and depth targets, and their read-back for headless snapshots.

use std::sync::mpsc;

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// A colour target with its depth buffer.
pub struct FrameTarget {
    pub width: u32,
    pub height: u32,
    color: wgpu::Texture,
    pub color_view: wgpu::TextureView,
    pub depth_view: wgpu::TextureView,
}

impl FrameTarget {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = |format, usage, label| {
            device.create_texture(&wgpu::TextureDescriptor {
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
            COLOR_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            "ac2 frame colour",
        );
        let depth = texture(
            DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            "ac2 frame depth",
        );
        Self {
            width,
            height,
            color_view: color.create_view(&Default::default()),
            depth_view: depth.create_view(&Default::default()),
            color,
        }
    }

    /// Clears the target to `clear` and records one render pass into it.
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        clear: [f32; 3],
        record: impl FnOnce(&mut wgpu::RenderPass<'_>),
    ) {
        render_to(
            device,
            queue,
            &self.color_view,
            &self.depth_view,
            clear,
            record,
        );
    }

    /// Copies the colour target to the CPU as tightly packed RGBA8 rows.
    pub fn read_rgba(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u8> {
        let row = self.width * 4;
        let padded =
            row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ac2 frame read-back"),
            size: u64::from(padded) * u64::from(self.height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.color,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
        let (sender, receiver) = mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        receiver
            .recv()
            .expect("map callback ran")
            .expect("read-back buffer maps");
        let mapped = buffer
            .slice(..)
            .get_mapped_range()
            .expect("read-back buffer is mapped");
        let mut rgba = Vec::with_capacity((row * self.height) as usize);
        for y in 0..self.height as usize {
            let start = y * padded as usize;
            rgba.extend_from_slice(&mapped[start..start + row as usize]);
        }
        rgba
    }
}

/// The next frame of `surface`, reconfigured with `config` when it is outdated or suboptimal;
/// `None` when the frame is skipped (occluded, timed out, lost) and the caller tries again.
pub fn acquire_frame(
    surface: &wgpu::Surface<'_>,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> Option<wgpu::SurfaceTexture> {
    match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(frame) => Some(frame),
        wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
            surface.configure(device, config);
            Some(frame)
        }
        wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => None,
        _ => {
            surface.configure(device, config);
            None
        }
    }
}

/// Clears `color` and `depth`, then records one render pass into them.
pub fn render_to(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    color: &wgpu::TextureView,
    depth: &wgpu::TextureView,
    clear: [f32; 3],
    record: impl FnOnce(&mut wgpu::RenderPass<'_>),
) {
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ac2 frame"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(clear[0]),
                        g: f64::from(clear[1]),
                        b: f64::from(clear[2]),
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        record(&mut pass);
    }
    queue.submit([encoder.finish()]);
}
