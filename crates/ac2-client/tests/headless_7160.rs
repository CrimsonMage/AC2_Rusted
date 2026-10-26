//! The headless client end to end against the Legions 7160 server double, on synthetic world
//! data: log on, spawn on flat ground, walk north into a pillar (physics collision), take a
//! server force-position, and log off; every movement report is checked byte for byte.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::*;

use ac2_client::{Client, FrameInput, Notice};
use ac2_net::protocol::{ClientEvent, ClientPosition};
use ac2_weenie::{
    ActiveAction, CellPosition, OnlineSession, ACTION_CLASS_MOVEMENT, ACTION_FORWARD,
};
use legions_server_double::{double, Spawn};

/// `ShouldSendPositionEvent`, written out: a do-motion change, or more than 0.2 s and a moved
/// body (the heading never changes here).
fn should_report(
    last: Option<(f64, CellPosition, [f32; 3])>,
    now: (f64, CellPosition, [f32; 3]),
) -> bool {
    let Some((time, position, motion)) = last else {
        return true;
    };
    let differs = |a: f32, b: f32| f64::from(0.0002f32) <= (f64::from(a) - f64::from(b)).abs();
    if (0..3).any(|i| differs(now.2[i], motion[i])) {
        return true;
    }
    if now.0 - time <= 0.2 {
        return false;
    }
    now.1.cell != position.cell || (0..3).any(|i| differs(now.1.origin[i], position.origin[i]))
}

#[test]
fn the_headless_client_walks_into_a_pillar_and_takes_a_forced_position() {
    let spawn = Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (transport, server) = double(spawn);
    let session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    let mut client = Client::new(session, Arc::new(Synthetic));
    client.set_load_budget(Duration::from_secs(5));
    let idle = FrameInput::default();
    let mut frames = 0;
    let mut notices = Vec::new();
    while !client.frame().in_world {
        // Every load asked for in a frame is in by the next one.
        client.settle_loads(Duration::from_secs(5)).unwrap();
        client.use_time(&idle).unwrap();
        let frame = client.frame();
        // `IsBlockingForCells` holds the body out of the world (and its physics) until the
        // cells around it are in.
        if frame.blocking_for_cells {
            assert!(!frame.in_world);
        }
        notices.extend(client.take_notices());
        server.borrow_mut().elapsed += STEP;
        frames += 1;
        assert!(frames < 600, "no world entry");
    }
    // World entry: ready to enter the world, then the frame-drawn notice in the same frame,
    // which sent `SceneRenderingComplete` once.
    assert_eq!(
        notices,
        vec![Notice::PlayerReadyToEnterWorld, Notice::NextFrameDrawn]
    );
    assert_eq!(client.frame().blocks.len(), 9);
    client.use_time(&idle).unwrap();
    server.borrow_mut().elapsed += STEP;
    let scene_complete = ClientEvent::SceneRenderingComplete.encode().1;
    let sent = |state: &legions_server_double::State| {
        state
            .blobs
            .iter()
            .filter(|(_, _, b)| *b == scene_complete)
            .count()
    };
    assert_eq!(sent(&server.borrow()), 1);
    // Settle onto the ground for a second.
    for _ in 0..64 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }
    let start = client.frame().player.unwrap();
    assert!(start.contact, "{start:?}");
    // The camera sits 4.5 m behind the player at the default 0.25 rad pitch, lifted by its
    // height, looking at three quarters of its height.
    let camera = client.frame().camera.expect("camera");
    let pitch = 0.25_f64;
    let behind = [
        start.origin[0] as f64,
        start.origin[1] as f64 - 4.5 * pitch.cos(),
        start.origin[2] as f64 + 4.5 * pitch.sin() + 1.8,
    ];
    for axis in 0..3 {
        assert!(
            (camera.origin[axis] as f64 - behind[axis]).abs() < 0.05,
            "{camera:?} {behind:?}"
        );
    }
    // It looks at the top of the player (its height above the origin).
    let head = [start.origin[0], start.origin[1], start.origin[2] + 1.8];
    for (target, head) in camera.target.iter().zip(head) {
        assert!((target - head).abs() < 0.01, "{camera:?}");
    }
    assert_eq!(camera.fov, 65.0);
    // `GetGlobalDirection`: the negated viewer offset (0, -1, pitch) the movement module steers
    // by.
    assert_eq!(client.camera().global_direction(), [0.0, 1.0, -0.25]);
    let reports_before = position_reports(&server.borrow());

    // Walk north for three seconds: the pillar stops the body short of its surface.
    let walk = FrameInput {
        actions: vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: ACTION_FORWARD,
            map_extent: 1.0,
            extent: 1.0,
        }],
        ..FrameInput::default()
    };
    let mut expected = Vec::new();
    let mut last = None;
    for _ in 0..192 {
        // The report reads the body as the previous frame left it, with this frame's motion.
        let before = client.player_movement_state().unwrap();
        client.use_time(&walk).unwrap();
        let time = client.session().cur_time();
        let position = CellPosition {
            cell: before.cell_id,
            origin: before.origin,
        };
        let now = (time, position, [0.0, 1.0, 0.0]);
        if should_report(last, now) {
            last = Some(now);
            expected.push(
                ClientEvent::Position(ClientPosition {
                    time,
                    position,
                    heading_degrees: before.heading_degrees,
                    motion: [0.0, 1.0, 0.0],
                    cancel_moveto: true,
                    instance_stamp: 9,
                    teleport_stamp: 5,
                    force_position_stamp: 6,
                    moveto_stamp: 7,
                    jump_velocity: None,
                })
                .encode()
                .1,
            );
        }
        server.borrow_mut().elapsed += STEP;
    }
    let stopped = client.frame().player.unwrap();
    let face = PILLAR[1] - PILLAR_RADIUS - 0.4;
    assert!(stopped.origin[1] < face + 0.01, "{stopped:?}");
    assert!(stopped.origin[1] > face - 0.25, "{stopped:?}");
    assert!((stopped.origin[0] - 70.0).abs() < 0.01, "{stopped:?}");
    let reports = position_reports(&server.borrow());
    assert_eq!(&reports[reports_before.len()..], &expected[..]);
    // About 1.4 s of walking at 4 m/s reaches the pillar; stuck, the body no longer reports.
    assert!(expected.len() >= 5, "{}", expected.len());

    // A newer force-position stamp moves the body (`RecvEvt_Position`, player branch).
    server
        .borrow_mut()
        .send_player_position(CELL, [60.0, 60.0, GROUND], [20, 5, 7]);
    for _ in 0..4 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }
    // The body keeps the momentum it had (it may coast a little north on the next quantum).
    let moved = client.frame().player.unwrap();
    assert!((moved.origin[0] - 60.0).abs() < 0.01, "{moved:?}");
    assert!((0.0..0.5).contains(&(moved.origin[1] - 60.0)), "{moved:?}");

    assert!(client.session_mut().log_off(Duration::from_secs(3)));
    assert!(server.borrow().logged_off);
}

fn position_reports(state: &legions_server_double::State) -> Vec<Vec<u8>> {
    state
        .blobs
        .iter()
        .filter(|(_, _, b)| b[..4] == 0x0001_008Eu32.to_le_bytes())
        .map(|(_, _, b)| b.clone())
        .collect()
}
