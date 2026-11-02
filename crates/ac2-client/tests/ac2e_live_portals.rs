//! The start room's portals against a running AC2e server (`AC2E_LIVE`, see `tests/live/mod.rs`):
//! AC2e's training world (`training-world.tsv`) places an "Arwic Outland Portal" (entity
//! `0x4700084D`, visual `0x1F00031E`) beside a new character's start in the Menhir Vault. The
//! portal is drawn (its visual's one mesh is an invisible click volume; the portal is the looping
//! particle systems of its startup FX), and walking into it takes the player through, as using it
//! does (the server's portal collision: AC2e `portals::Contacts`).
//!
//! Run with `AC2E_LIVE_TEST=ac2e_live_portals scripts/ac2e-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_weenie::{ActiveAction, ACTION_CLASS_MOVEMENT, ACTION_FORWARD, ACTION_TURN_POSITIVE};
use live::LiveRig;

/// The Menhir Vault: the start room's cell.
const START_CELL: u32 = 0x0299_0109;
/// The portal's entity DID (`training-world.tsv`).
const PORTAL: u32 = 0x4700_084D;

fn origin(rig: &LiveRig) -> [f32; 3] {
    rig.client
        .frame()
        .player
        .as_ref()
        .expect("the player")
        .origin
}

fn cell(rig: &LiveRig) -> u32 {
    rig.client.frame().player.as_ref().expect("the player").cell
}

fn portal(rig: &LiveRig) -> u64 {
    let me = origin(rig);
    let frame = rig.client.frame();
    rig.client
        .session()
        .objects()
        .iter()
        .filter(|o| o.entity == Some(PORTAL))
        .filter_map(|o| {
            let f = frame.objects.iter().find(|f| f.id == o.id && f.in_world)?;
            Some((o.id, (f.origin[0] - me[0]).hypot(f.origin[1] - me[1])))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("a portal in the start room")
        .0
}

fn hold(action: u32) -> Vec<ActiveAction> {
    vec![ActiveAction {
        class: ACTION_CLASS_MOVEMENT,
        enum_value: action,
        map_extent: 1.0,
        extent: 1.0,
    }]
}

/// The player's facing on the ground: the body's +y axis turned by its rotation (w, x, y, z).
fn facing(rig: &LiveRig) -> [f32; 2] {
    let [w, x, y, z] = rig.client.frame().player.as_ref().unwrap().rotation;
    [2.0 * (x * y - w * z), 1.0 - 2.0 * (x * x + z * z)]
}

/// Turns the player in place until it faces `id`.
fn face(rig: &mut LiveRig, id: u64) {
    let at = place(rig, id);
    for _ in 0..400 {
        let me = origin(rig);
        let (dx, dy) = (at[0] - me[0], at[1] - me[1]);
        let len = dx.hypot(dy);
        let f = facing(rig);
        if (f[0] * dx + f[1] * dy) / len > 0.995 {
            break;
        }
        rig.held = hold(ACTION_TURN_POSITIVE);
        rig.frame(&[]);
    }
    rig.held.clear();
    rig.run(0.5);
}

fn place(rig: &LiveRig, id: u64) -> [f32; 3] {
    rig.client
        .frame()
        .objects
        .iter()
        .find(|o| o.id == id)
        .expect("the portal")
        .origin
}

fn start() -> Option<LiveRig> {
    let mut rig = LiveRig::new_with_sound("Porter")?;
    rig.run(3.0);
    assert!(rig.client.frame().in_world, "the player is in the world");
    assert_eq!(cell(&rig), START_CELL);
    Some(rig)
}

/// Lays the view out for three seconds of frames (the particles run on its clock) and gives the
/// object particle systems and live particles of the last frame.
fn draw_frames(rig: &mut LiveRig) -> (usize, usize) {
    let frame = rig.client.frame();
    let size = live::SIZE;
    let aspect = size.0 as f32 / size.1 as f32;
    rig.view.settle(&rig.gpu, &frame, aspect);
    for i in 0..90 {
        assert!(rig
            .view
            .prepare(&rig.gpu, &frame, aspect, f64::from(i) / 30.0));
    }
    rig.view.object_particles()
}

fn snapshot(rig: &mut LiveRig, name: &str) {
    let Ok(out) = std::env::var("AC2_PORTAL_SNAPSHOTS") else {
        return;
    };
    std::fs::create_dir_all(&out).unwrap();
    let size = live::SIZE;
    let target = ac2_gpu::FrameTarget::new(&rig.gpu.device, size.0, size.1);
    rig.view
        .render(&rig.gpu, &target.color_view, &target.depth_view, true);
    let rgba = target.read_rgba(&rig.gpu.device, &rig.gpu.queue);
    let file = std::fs::File::create(std::path::Path::new(&out).join(name)).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), size.0, size.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&rgba)
        .unwrap();
}

/// The portal's visual is drawn: its object frame names the visual and the view draws parts
/// for it.
#[test]
fn the_start_rooms_portal_is_drawn() {
    let Some(mut rig) = start() else {
        return;
    };
    let id = portal(&rig);
    rig.command(Command::Select(Some(id)));
    face(&mut rig, id);
    rig.settle_view();
    let frame = rig.client.frame();
    let object = frame.objects.iter().find(|o| o.id == id).unwrap();
    eprintln!(
        "portal {id:#x}: visual {:x?} setup {:x?} at {:?} in {:08x}; parts posed {}; player at {:?}",
        object.visual,
        object.setup,
        object.origin,
        object.cell,
        object.pose.len(),
        origin(&rig)
    );
    let particles = draw_frames(&mut rig);
    eprintln!(
        "loaded objects {} of {}; object particle systems and live particles {particles:?}",
        rig.view.loaded_objects(&frame),
        frame.objects.len()
    );
    snapshot(&mut rig, "portal_start_room.png");
    assert!(
        particles.0 > 0 && particles.1 > 0,
        "the portal's startup particle systems draw: {particles:?}"
    );
}

/// Walking into the portal takes the player out of the vault, as using it does.
#[test]
fn walking_into_the_portal_travels() {
    let Some(mut rig) = start() else {
        return;
    };
    let id = portal(&rig);
    let at = place(&rig, id);
    face(&mut rig, id);
    let from = origin(&rig);
    // Forward through the portal's place and a little past it, then stop.
    rig.held = hold(ACTION_FORWARD);
    let left = rig.until(4.0, |r| {
        let frame = r.client.frame();
        !frame.in_world
            || frame
                .player
                .as_ref()
                .is_some_and(|p| p.cell >> 16 != START_CELL >> 16)
    });
    rig.held.clear();
    let closest = origin(&rig);
    let left = left
        || rig.until(10.0, |r| {
            let frame = r.client.frame();
            !frame.in_world
                || frame
                    .player
                    .as_ref()
                    .is_some_and(|p| p.cell >> 16 != START_CELL >> 16)
        });
    assert!(
        left,
        "walking from {from:?} into the portal {id:#x} at {at:?} stopped at {closest:?}; the player stays at {:?} in {:08x}",
        origin(&rig),
        cell(&rig)
    );
}
