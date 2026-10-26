//! Smooth drawing of other objects between server positions, against the Legions 7160 server
//! double. A remote creature walks at 4 m/s, reported every 0.25 s one metre apart, while the
//! client steps at 60 frames per second; the drawn position is read from `client.frame()` on
//! every frame.

mod common;

use ac2_client::{ClientFrame, FrameInput, ObjectFrame};
use ac2_net::protocol::{heading_to_u8, motion_to_u24};
use common::*;
use legions_server_double::{
    create_object_bytes, position_bytes, ObjectPosition, ObjectSpawn, Spawn,
};

const NPC: u64 = 0x8000_0000_0000_0051;
const FPS: f64 = 60.0;
/// One server position every 0.25 s: 15 frames at 60 fps.
const FRAMES_PER_POSITION: usize = 15;
const POSITIONS: usize = 24;
/// The drawn motion is measured once the object is up to speed (after the first second).
const WARM_UP_FRAMES: usize = 60;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

fn npc_spawn(origin: [f32; 3]) -> ObjectSpawn {
    ObjectSpawn {
        id: NPC,
        instance: 3,
        visual: 0x1F00_0023,
        cell: CELL,
        origin,
        rotation: [1.0, 0.0, 0.0, 0.0],
        stamps: [10, 1, 1, 0],
        name: "Walker",
    }
}

/// A forward-moving position heading 90 degrees (+x), stamped `stamp`, at server time `time`.
fn walking(time: f64, stamp: u16, origin: [f32; 3]) -> Vec<u8> {
    position_bytes(&ObjectPosition {
        id: NPC,
        instance: 3,
        time,
        cell: CELL,
        origin,
        heading_and_motion: u32::from(heading_to_u8(90.0)) << 24 | motion_to_u24([0.0, 1.0, 0.0]),
        flags: 4,
        stamps: [stamp, 1, 1],
        velocity: None,
    })
}

fn object(frame: &ClientFrame, id: u64) -> Option<ObjectFrame> {
    frame.objects.iter().find(|o| o.id == id).cloned()
}

/// One frame at 60 fps: the loads settle, the client runs `use_time`, the clocks step 1/60 s.
fn frame_step(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) {
    client
        .settle_loads(std::time::Duration::from_secs(5))
        .unwrap();
    client.use_time(&FrameInput::default()).unwrap();
    server.borrow_mut().elapsed += 1.0 / FPS;
    server.borrow_mut().combat_tick();
}

/// The drawn x (east) of the object on each frame, from the frame it is in the world.
fn drawn_track(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    frames: usize,
) -> Vec<[f32; 2]> {
    let mut stamp = 11u16;
    let mut track = Vec::with_capacity(frames);
    for frame in 0..frames {
        if frame % FRAMES_PER_POSITION == 0 && frame / FRAMES_PER_POSITION < POSITIONS {
            let k = (frame / FRAMES_PER_POSITION) as f32;
            let now = client.session().cur_time();
            server
                .borrow_mut()
                .send_event(&walking(now, stamp, [50.0 + k, 60.0, GROUND]));
            stamp += 1;
        }
        frame_step(client, server);
        let o = object(&client.frame(), NPC).expect("the object");
        track.push([o.origin[0], o.origin[1]]);
    }
    track
}

/// Per-frame steps over the measured window: the average and the extremes, and the frames that
/// do not move while the object travels (step below 1 mm).
#[derive(Debug)]
struct Jitter {
    mean: f32,
    min: f32,
    max: f32,
    still: usize,
}

fn jitter(track: &[[f32; 2]]) -> Jitter {
    let steps: Vec<f32> = track
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .collect();
    let window = &steps[WARM_UP_FRAMES..];
    let mean = window.iter().sum::<f32>() / window.len() as f32;
    Jitter {
        mean,
        min: window.iter().copied().fold(f32::MAX, f32::min),
        max: window.iter().copied().fold(0.0, f32::max),
        still: window.iter().filter(|s| **s < 1e-3).count(),
    }
}

#[test]
fn a_remote_walker_draws_smoothly_between_server_positions() {
    let (mut client, server) = entered(spawn());
    server
        .borrow_mut()
        .send_event(&create_object_bytes(&npc_spawn([50.0, 60.0, GROUND])));
    let frames = POSITIONS * FRAMES_PER_POSITION + 60;
    let track = drawn_track(&mut client, &server, frames);
    let j = jitter(&track);
    let first = track.first().unwrap()[0];
    let last = track.last().unwrap()[0];
    eprintln!(
        "smooth motion: x {first:.2} -> {last:.2}, mean step {:.4} m/frame, \
         min step {:.4} m, max step {:.4} m, still frames {}",
        j.mean, j.min, j.max, j.still
    );
    assert!(last - first > 10.0, "the walker moved: {first} -> {last}");
    // The walker never stops while it walks at 4 m/s, so no frame is still.
    assert_eq!(j.still, 0, "frames with no movement");
    // Each frame's step is within 20% of the average: the drawn motion is at display rate, not
    // at the 10 Hz physics quantum.
    assert!(j.min >= 0.8 * j.mean && j.max <= 1.2 * j.mean, "{j:?}");
}
