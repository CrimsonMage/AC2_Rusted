//! The gameplay HUD's GPU memory is stable over many frames: after a warm-up, a thousand more
//! frames of the HUD (the radar redrawn each frame, the layers recomposed when they change) add
//! no live textures, buffers, bind groups or views, and no allocated memory beyond the allocator's
//! own noise. The rig submits each frame's queued uploads as the window loop does; a loop that
//! updates the HUD without submitting grew about 2 MB a frame. Needs `AC2_DAT_DIR` and a GPU adapter.
#![cfg(feature = "window")]

mod hud;

use hud::Rig;

const WARM_UP: usize = 300;
const FRAMES: usize = 1000;
/// Allocator noise between two reports (sub-allocation blocks); the leak grew ~2 MB a frame.
const BYTES_SLACK: u64 = 64 << 20;

/// The live GPU objects the HUD holds, and the bytes the allocator reports (Vulkan only).
#[derive(Debug, PartialEq, Eq)]
struct Live {
    textures: usize,
    texture_views: usize,
    buffers: usize,
    bind_groups: usize,
    samplers: usize,
}

fn live(rig: &Rig) -> (Live, Option<u64>) {
    let report = rig
        .gpu
        .instance
        .generate_report()
        .expect("a wgpu-core instance reports its objects");
    let hub = &report.hub;
    let live = Live {
        textures: hub.textures.num_kept_from_user,
        texture_views: hub.texture_views.num_kept_from_user,
        buffers: hub.buffers.num_kept_from_user,
        bind_groups: hub.bind_groups.num_kept_from_user,
        samplers: hub.samplers.num_kept_from_user,
    };
    let bytes = rig
        .gpu
        .device
        .generate_allocator_report()
        .map(|report| report.total_allocated_bytes);
    (live, bytes)
}

#[test]
fn hud_gpu_memory_is_stable_over_many_frames() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.open_inventory();
    rig.idle(WARM_UP);
    let (start, start_bytes) = live(&rig);
    rig.idle(FRAMES);
    let (end, end_bytes) = live(&rig);

    assert_eq!(end, start, "live GPU objects after {FRAMES} more frames");
    if let (Some(start), Some(end)) = (start_bytes, end_bytes) {
        assert!(
            end <= start + BYTES_SLACK,
            "allocated GPU bytes grew from {start} to {end} over {FRAMES} frames"
        );
    }
}
