//! The world's post chain on a uniform scene: the bloom of a flat field is the field, the
//! vignette leaves the centre alone, so the centre pixel shows only the eye adaptation's exposure
//! (`post.rs`: half the metered difference from the key, within one stop either way).

use ac2_gpu::{FrameTarget, Gpu, ScenePresenter, COLOR_FORMAT};

const SIZE: (u32, u32) = (160, 96);
/// `post.rs`'s `KEY_LOG2`, `ADAPT_STRENGTH` and `EV_LIMITS`.
const KEY_LOG2: f64 = -3.0;
const ADAPT_STRENGTH: f64 = 0.5;

/// `tonemap.wgsl`'s Khronos PBR Neutral curve on a grey channel, then the sRGB encode, as 0-255.
fn display(x: f64) -> f64 {
    let offset = if x < 0.08 { x - 6.25 * x * x } else { 0.04 };
    let c = x - offset;
    let c = if c < 0.76 {
        c
    } else {
        let d = 1.0 - 0.76;
        1.0 - d * d / (c + d - 0.76)
    };
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    s.clamp(0.0, 1.0) * 255.0
}

fn centre_after_post(gpu: &Gpu, presenter: &ScenePresenter, luminance: f32) -> u8 {
    let (width, height) = SIZE;
    let output = FrameTarget::new(&gpu.device, width, height);
    let hdr = presenter.target(gpu, width, height);
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    hdr.render(
        gpu,
        &mut encoder,
        [luminance, luminance, luminance, 1.0],
        None,
        |_| {},
    );
    presenter.post(gpu, &mut encoder, &hdr);
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("post test output"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output.color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &output.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        presenter.draw_world(&mut pass, &hdr);
    }
    gpu.queue.submit([encoder.finish()]);
    let rgba = output.read_rgba(&gpu.device, &gpu.queue);
    let at = ((height / 2 * width + width / 2) * 4) as usize;
    rgba[at + 1]
}

#[test]
fn a_uniform_scene_takes_half_its_metered_difference_within_a_stop() {
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("skipped: no GPU adapter");
        return;
    };
    // A fresh presenter's first frame takes its exposure at once.
    for (log2, expected_ev) in [(-3.0, 0.0), (-4.0, 0.5), (-1.0, -1.0), (-8.0, 1.0)] {
        let presenter = ScenePresenter::new(&gpu, COLOR_FORMAT);
        let luminance = 2.0_f64.powf(log2);
        let ev = ((KEY_LOG2 - log2) * ADAPT_STRENGTH).clamp(-1.0, 1.0);
        assert_eq!(ev, expected_ev);
        let got = f64::from(centre_after_post(&gpu, &presenter, luminance as f32));
        let want = display(luminance * 2.0_f64.powf(ev));
        assert!(
            (got - want).abs() <= 2.0,
            "luminance 2^{log2}: centre {got}, expected {want:.1} at EV {ev}"
        );
    }
}
