//! Drawing the UI's surfaces with the viewport scenes among them: each scene goes after its
//! root's surface (`UIViewportObject::Render` in the surface objects' order), in a pass of its
//! own that clears the depth (`UIScene::Render`'s clear) and sets the device viewport to the
//! root's rectangle.

use ac2_gpu::{Gpu, UiOverlay};

use super::UiScenes;

/// Draws into `color` (cleared to `clear`) on a `size` target: `first` (the world, when there
/// is one), then `overlay`'s layers (whose roots are `layer_roots`, in drawing order) with each
/// of `scenes`' prepared viewports after its root's layer.
#[allow(clippy::too_many_arguments)]
pub fn render_with_scenes(
    gpu: &Gpu,
    color: &wgpu::TextureView,
    depth: &wgpu::TextureView,
    size: (u32, u32),
    clear: [f32; 3],
    overlay: &UiOverlay,
    layer_roots: &[usize],
    scenes: &[&UiScenes],
    first: impl FnOnce(&mut wgpu::RenderPass<'_>),
) {
    let mut views: Vec<(usize, usize, usize, ac2_ui::SceneViewport)> = scenes
        .iter()
        .enumerate()
        .flat_map(|(owner, scenes)| {
            scenes
                .drawn()
                .into_iter()
                .map(move |(view, viewport)| (owner, view, viewport))
        })
        .filter_map(|(owner, view, viewport)| {
            let layer = layer_roots.iter().position(|&r| r == viewport.root)?;
            Some((layer, owner, view, viewport))
        })
        .collect();
    views.sort_by_key(|(layer, ..)| *layer);
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    let mut first = Some(first);
    let mut next = 0;
    let (width, height) = size;
    for (i, (layer, owner, view, viewport)) in views.iter().enumerate() {
        let mut pass = begin_pass(&mut encoder, color, depth, (i == 0).then_some(clear));
        if let Some(first) = first.take() {
            first(&mut pass);
        }
        overlay.draw_range(&mut pass, next..layer + 1);
        next = layer + 1;
        let x0 = viewport.x.clamp(0, width as i32) as u32;
        let y0 = viewport.y.clamp(0, height as i32) as u32;
        let x1 = (viewport.x + viewport.width as i32).clamp(0, width as i32) as u32;
        let y1 = (viewport.y + viewport.height as i32).clamp(0, height as i32) as u32;
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        pass.set_viewport(
            x0 as f32,
            y0 as f32,
            (x1 - x0) as f32,
            (y1 - y0) as f32,
            0.0,
            1.0,
        );
        scenes[*owner].draw(*view, &mut pass);
    }
    {
        let mut pass = begin_pass(
            &mut encoder,
            color,
            depth,
            views.is_empty().then_some(clear),
        );
        if let Some(first) = first.take() {
            first(&mut pass);
        }
        overlay.draw_range(&mut pass, next..layer_roots.len().max(next));
    }
    gpu.queue.submit([encoder.finish()]);
}

/// A pass over `color` (cleared to `clear`, else kept) with its depth cleared.
fn begin_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color: &wgpu::TextureView,
    depth: &wgpu::TextureView,
    clear: Option<[f32; 3]>,
) -> wgpu::RenderPass<'a> {
    let load = match clear {
        Some([r, g, b]) => wgpu::LoadOp::Clear(wgpu::Color {
            r: f64::from(r),
            g: f64::from(g),
            b: f64::from(b),
            a: 1.0,
        }),
        None => wgpu::LoadOp::Load,
    };
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ac2 ui scenes"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load,
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
    })
}
