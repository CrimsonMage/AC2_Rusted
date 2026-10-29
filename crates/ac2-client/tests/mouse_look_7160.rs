//! The mouse-look camera against the Legions 7160 server double: while the player runs with the
//! right button held, the camera keeps its distance from the body as drawn, frame by frame
//! (`AGENTS.md`: the camera follows the drawn position, not the 0.1 s physics quanta).

mod common;

use ac2_client::FrameInput;
use ac2_weenie::{ActiveAction, ACTION_CLASS_MOVEMENT, ACTION_FORWARD};
use common::*;
use legions_server_double::Spawn;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 40.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

#[test]
fn the_mouse_look_camera_moves_with_the_drawn_body_while_running() {
    let (mut client, server) = entered(spawn());
    let look = FrameInput {
        mouse_look_state: Some(true),
        ..FrameInput::default()
    };
    step(&mut client, &server, &look);
    let run = FrameInput {
        actions: vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: ACTION_FORWARD,
            map_extent: 1.0,
            extent: 1.0,
        }],
        ..FrameInput::default()
    };
    // Up to speed first.
    for _ in 0..64 {
        step(&mut client, &server, &run);
    }
    // Each frame (1/64 s, several per 0.1 s quantum): the camera's offset from the drawn body.
    let mut gaps = Vec::new();
    let mut moved = 0.0_f32;
    let mut last = None;
    for _ in 0..64 {
        step(&mut client, &server, &run);
        let frame = client.frame();
        let player = frame.player.as_ref().expect("the player");
        let camera = frame.camera.expect("the camera");
        assert_eq!(camera.cell >> 16, player.cell >> 16, "one landblock");
        let gap = (0..3)
            .map(|i| (camera.origin[i] - player.origin[i]).powi(2))
            .sum::<f32>()
            .sqrt();
        gaps.push(gap);
        if let Some(previous) = last.replace(player.origin) {
            moved += (player.origin[1] - previous[1]).abs();
        }
    }
    assert!(moved > 2.0, "the player ran ({moved} m)");
    let (min, max) = gaps
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &g| (lo.min(g), hi.max(g)));
    // Following the current quantum, the gap swung by up to a quantum's run (about 0.4 m) as
    // the drawn body caught up between quanta.
    assert!(max - min < 0.05, "camera to body gap {min}..{max}");
}
