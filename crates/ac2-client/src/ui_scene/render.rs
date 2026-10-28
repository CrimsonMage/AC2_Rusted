//! Drawing the UI's surfaces with the viewport scenes among them: each scene goes after its
//! root's surface (`UIViewportObject::Render` in the surface objects' order), in a pass of its
//! own that clears the depth (`UIScene::Render`'s clear) and sets the device viewport to the
//! root's rectangle.
//!
//! The world and each scene draw in linear light into the presenter's HDR target and reach the
//! window through its tone map (`ac2_gpu::ToneMap`); a scene's tone map composites over the UI
//! surfaces behind it where the scene drew. The UI surfaces themselves draw straight to the
//! window.

use ac2_gpu::{Gpu, ScenePresenter, UiOverlay};

use super::UiScenes;
use crate::ui_scale::UiScale;
use ac2_ui::SceneViewport;

/// Draws into `color` (cleared to `clear`, an sRGB colour) on the window `scale` fits the UI
/// to: `world` (when there is one) through the tone map, then `first` (drawn to the window
/// format), then `overlay`'s layers (whose roots are `layer_roots`, in drawing order) with each
/// of `scenes`' prepared viewports after its root's layer.
#[allow(clippy::too_many_arguments)]
pub fn render_with_scenes(
    gpu: &Gpu,
    presenter: &ScenePresenter,
    color: &wgpu::TextureView,
    depth: &wgpu::TextureView,
    scale: &UiScale,
    clear: [f32; 3],
    overlay: &UiOverlay,
    layer_roots: &[usize],
    scenes: &[&UiScenes],
    world: Option<&dyn Fn(&mut wgpu::RenderPass<'_>)>,
    first: impl FnOnce(&mut wgpu::RenderPass<'_>),
) {
    let mut views: Vec<(usize, usize, SceneViewport)> = scenes
        .iter()
        .enumerate()
        .flat_map(|(owner, scenes)| {
            scenes
                .drawn()
                .into_iter()
                .map(move |viewport| (owner, viewport))
        })
        .filter_map(|(owner, viewport)| {
            let layer = layer_roots.iter().position(|&r| r == viewport.root)?;
            Some((layer, owner, viewport))
        })
        .collect();
    views.sort_by_key(|(layer, ..)| *layer);
    let (width, height) = scale.window;
    let hdr = (world.is_some() || !views.is_empty()).then(|| presenter.target(gpu, width, height));
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    // `Turbine::Client::UseTime`'s clear, then the world in the scene target.
    if let (Some(world), Some(hdr)) = (world, &hdr) {
        let [r, g, b] = clear.map(|c| ac2_gpu::srgb_to_linear(c.clamp(0.0, 1.0)));
        hdr.render(gpu, &mut encoder, [r, g, b, 1.0], None, world);
    }
    let mut pass_clear = Some(clear);
    let mut first = Some(first);
    let mut next = 0;
    let mut pending: Option<[u32; 4]> = None;
    for (layer, owner, viewport) in views.iter() {
        {
            let mut pass = begin_pass(gpu, &mut encoder, color, depth, pass_clear.take());
            output_head(
                &mut pass,
                presenter,
                hdr.as_deref(),
                world.is_some(),
                &mut first,
            );
            if let (Some(rect), Some(hdr)) = (pending.take(), &hdr) {
                presenter.tonemap().draw(&mut pass, hdr, Some(rect));
            }
            overlay.draw_range(&mut pass, next..layer + 1);
            next = layer + 1;
        }
        // The viewport's UI rectangle in window pixels.
        let (left, top) = scale.to_window(f64::from(viewport.x), f64::from(viewport.y));
        let (right, bottom) = scale.to_window(
            f64::from(viewport.x) + f64::from(viewport.width),
            f64::from(viewport.y) + f64::from(viewport.height),
        );
        let x0 = (left.round() as i32).clamp(0, width as i32) as u32;
        let y0 = (top.round() as i32).clamp(0, height as i32) as u32;
        let x1 = (right.round() as i32).clamp(0, width as i32) as u32;
        let y1 = (bottom.round() as i32).clamp(0, height as i32) as u32;
        let Some(hdr) = &hdr else { continue };
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        let rect = [x0, y0, x1 - x0, y1 - y0];
        // `UIScene::Render`: the scene alone, over nothing (coverage 0) and a cleared depth.
        hdr.render(gpu, &mut encoder, [0.0; 4], Some(rect), |pass| {
            scenes[*owner].draw(viewport.root, pass);
        });
        pending = Some(rect);
    }
    {
        let mut pass = begin_pass(gpu, &mut encoder, color, depth, pass_clear.take());
        output_head(
            &mut pass,
            presenter,
            hdr.as_deref(),
            world.is_some(),
            &mut first,
        );
        if let (Some(rect), Some(hdr)) = (pending.take(), &hdr) {
            presenter.tonemap().draw(&mut pass, hdr, Some(rect));
        }
        overlay.draw_range(&mut pass, next..layer_roots.len().max(next));
    }
    drop(hdr);
    gpu.queue.submit([encoder.finish()]);
}

/// The start of the first window pass: the world's tone map (when it drew), then `first`.
fn output_head(
    pass: &mut wgpu::RenderPass<'_>,
    presenter: &ScenePresenter,
    hdr: Option<&ac2_gpu::HdrTarget>,
    world: bool,
    first: &mut Option<impl FnOnce(&mut wgpu::RenderPass<'_>)>,
) {
    let Some(first) = first.take() else {
        return;
    };
    if let (true, Some(hdr)) = (world, hdr) {
        presenter.tonemap().draw(pass, hdr, None);
    }
    first(pass);
}

/// A pass over `color` (cleared to `clear`, else kept) with its depth cleared.
fn begin_pass<'a>(
    gpu: &'a Gpu,
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
        timestamp_writes: gpu.pass_timestamps("ui"),
        occlusion_query_set: None,
        multiview_mask: None,
    })
}
