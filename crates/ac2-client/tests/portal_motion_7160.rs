//! The portal space's motion drawn headless on the real DATs (`AC2_DAT_DIR`): the effect
//! (`PortalSpace::Effect`) is driven on its own clock at 60 frames a second through its whole run
//! (the fade in, the tunnel, the exit's white fade), and frames spaced through it are drawn
//! without a world through the view's portal pass, the post chain and the tone map. Writes
//! `target/ac2-snapshots/portal_motion_<nn>_<phase>_<ms>.png`; skipped without a GPU adapter.

#![cfg(feature = "window")]

use std::path::PathBuf;
use std::sync::Arc;

use ac2_client::view::ClientView;
use ac2_client::{ClientFrame, DatWorld, PortalFrame};
use ac2_core::Random;
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_render::portal::{Phase, PortalSpace};

const SIZE: (u32, u32) = (640, 480);
const STEP: f64 = 1.0 / 60.0;

fn write_png(name: &str, rgba: &[u8]) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).expect("snapshot directory");
    let file = std::fs::File::create(dir.join(name)).expect("png file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), SIZE.0, SIZE.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(rgba)
        .unwrap();
}

fn mean_rgb(rgba: &[u8]) -> [f32; 3] {
    let n = (rgba.len() / 4) as f32;
    let mut sum = [0.0f32; 3];
    for px in rgba.chunks_exact(4) {
        for c in 0..3 {
            sum[c] += f32::from(px[c]);
        }
    }
    sum.map(|s| s / n)
}

fn frame_of(effect: &PortalSpace) -> ClientFrame {
    let draw = *effect.draw();
    ClientFrame {
        portal: PortalFrame {
            in_portal_space: effect.in_portal_space(),
            phase: effect.phase(),
            draw,
            teleporting: false,
            camera: effect.camera(),
            tunnel: draw.tunnels.then(|| effect.tunnel_frame()),
            clouds: effect.cloud_geometry().clone(),
        },
        ..ClientFrame::default()
    }
}

#[test]
fn the_portal_space_runs_its_fade_tunnel_and_white_exit() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
    let world = Arc::clone(dat_world.world());
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    let mut view = ClientView::new(world, &gpu, COLOR_FORMAT, 1, SIZE);
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;

    let mut effect = PortalSpace::new();
    effect.set_clouds(ac2_render::portal::load_clouds_desc(&store), 0.0);
    // The client's generator after two draws: a tunnel that bends little (as `portal_view_7160`).
    let mut random = Random::default();
    random.rand();
    random.rand();
    effect.enter(false, 0.0, &mut random);
    // The fade in (1 s), 4 s of tunnel, then the exit's two phases (1 s each).
    let exit_at = 5.0;
    let end = exit_at + 2.2;
    let shots = [
        0.25, 0.5, 0.75, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.25, 5.5, 5.75, 6.25, 6.5, 6.75,
    ];
    let mut next_shot = 0;
    let mut frames = 0u32;
    let mut whites = Vec::new();
    let mut tick = 0u32;
    loop {
        let now = f64::from(tick) * STEP;
        if now > end {
            break;
        }
        if (now - exit_at).abs() < STEP / 2.0 {
            effect.exit(false, now);
        }
        effect.think(now, &mut random);
        effect.render_clouds(now, &mut random);
        let frame = frame_of(&effect);
        // Draw every frame so the eye adaptation follows the run; keep the shots.
        let shot = next_shot < shots.len() && now + STEP / 2.0 >= shots[next_shot];
        if shot || tick.is_multiple_of(4) {
            view.prepare(&gpu, &frame, aspect, now);
            view.render(&gpu, &target.color_view, &target.depth_view, false);
            frames += 1;
        }
        if shot {
            let rgba = target.read_rgba(&gpu.device, &gpu.queue);
            let phase = match effect.phase() {
                Phase::StartupPhase0 => "fadein",
                Phase::Active => "tunnel",
                Phase::ShutdownPhase0 => "exit0",
                Phase::ShutdownPhase1 => "exit1",
                _ => "other",
            };
            let name = format!(
                "portal_motion_{next_shot:02}_{phase}_{:04}.png",
                (now * 1000.0).round() as u32
            );
            let mean = mean_rgb(&rgba);
            eprintln!(
                "{name}: mean {mean:?} camera {:?}",
                effect.camera().map(|c| (c.eye, c.rotation))
            );
            if effect.phase() == Phase::ShutdownPhase0 || effect.phase() == Phase::ShutdownPhase1 {
                whites.push((effect.draw().fader_color[3], mean));
            }
            write_png(&name, &rgba);
            next_shot += 1;
        }
        tick += 1;
    }
    assert!(frames > 0);
    assert_eq!(effect.phase(), Phase::Inactive, "the effect ran out");
    // `Think`'s shutdown fader is `RGBAColor` (1, 1, 1, t): the exit fades through white, so a
    // frame under a strong fader is grey, never tinted.
    let (alpha, mean) =
        whites.iter().copied().fold(
            (0.0, [0.0; 3]),
            |best, f| if f.0 > best.0 { f } else { best },
        );
    assert!(alpha > 0.7, "a strongly faded exit frame ({alpha})");
    let spread = mean.iter().copied().fold(f32::MIN, f32::max)
        - mean.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        spread < 12.0,
        "the exit's fader is white, not tinted (mean {mean:?} at alpha {alpha})"
    );
}
