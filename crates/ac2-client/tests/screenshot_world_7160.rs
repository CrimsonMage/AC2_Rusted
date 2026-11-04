//! `SmartBox::RenderScreenshot` (Legions 0x00543580) on the HUD rig over the Legions server double
//! (`AC2_DAT_DIR`, a GPU adapter): the screenshot is the world alone, so where the HUD covers the
//! presented frame the capture shows the world, and where nothing covers it the two agree.

#![cfg(feature = "window")]

mod hud;

use std::time::Duration;

use ac2_gpu::{screenshot, FrameTarget, COLOR_FORMAT};
use hud::Rig;

#[test]
fn the_screenshot_is_the_world_without_the_hud() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the HUD", |r| r.view.hud_tree().is_some());
    rig.idle(4);
    let (width, height) = (1024, 768);
    rig.view.resize(width, height);
    let frame = rig.client.frame();
    let aspect = width as f32 / height as f32;
    rig.view.update(&rig.gpu, &frame, Duration::ZERO);
    rig.view.settle(&rig.gpu, &frame, aspect);
    assert!(rig
        .view
        .prepare(&rig.gpu, &frame, aspect, frame.server_time));
    let target = FrameTarget::new(&rig.gpu.device, width, height);
    let capture = screenshot::capture_target(&rig.gpu, COLOR_FORMAT, width, height);
    let capture_view = capture.create_view(&Default::default());
    rig.view.render_capturing(
        &rig.gpu,
        &target.color_view,
        &target.depth_view,
        true,
        Some(&capture_view),
    );
    let presented = target.read_rgba(&rig.gpu.device, &rig.gpu.queue);
    let shot = screenshot::read_surface(&rig.gpu, &capture, COLOR_FORMAT, width, height)
        .expect("the capture reads back");
    assert_eq!(shot.len(), presented.len());
    let differing = presented
        .chunks_exact(4)
        .zip(shot.chunks_exact(4))
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    let pixels = (width * height) as usize;
    // The HUD covers part of the presented frame, not all of it.
    assert!(differing > 0, "the HUD is not in the shot");
    assert!(differing < pixels / 2, "{differing} of {pixels} differ");
    assert!(
        shot.chunks_exact(4).any(|p| p[..3] != [0, 0, 0]),
        "the shot holds the world"
    );
}
