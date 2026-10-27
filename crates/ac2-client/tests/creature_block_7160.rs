//! A server object in the player's path stops the player: the creature the double creates in
//! front of it is in the world's object lists (`CObjCell::AddObject`), and the player's
//! transition tests it like a static object (`CPhysicsObj::FindObjCollisions`). The player walks
//! north into it, stops at its surface, and does not walk through it.

mod common;

use common::*;

use ac2_client::FrameInput;
use ac2_weenie::{ActiveAction, ACTION_CLASS_MOVEMENT, ACTION_FORWARD};
use legions_server_double::{Spawn, BLOCKING_CREATURE};

/// The player enters west of the pillar, so its path north is clear of the pillar.
fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [60.0, 60.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

#[test]
fn a_creature_in_the_path_stops_the_player() {
    let (mut client, server) = entered(spawn());
    let idle = FrameInput::default();
    // The creature is created 6 m north of the player; its Setup loads on a loader thread.
    server
        .borrow_mut()
        .spawn_blocking_creature([60.0, 66.0, GROUND]);
    for _ in 0..64 {
        step(&mut client, &server, &idle);
    }
    let remote = client
        .frame()
        .objects
        .iter()
        .any(|o| o.id == BLOCKING_CREATURE);
    assert!(remote, "the creature is in the frame");

    // Settle the player on the ground, then walk north for 3 s.
    for _ in 0..64 {
        step(&mut client, &server, &idle);
    }
    let walk = FrameInput {
        actions: vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: ACTION_FORWARD,
            map_extent: 1.0,
            extent: 1.0,
        }],
        ..FrameInput::default()
    };
    for _ in 0..192 {
        step(&mut client, &server, &walk);
    }
    let stopped = client.frame().player.expect("player");
    // The creature's surface is its radius plus the player's sphere radius (0.4 m) south of its
    // centre.
    let face = 66.0 - CREATURE_RADIUS - 0.4;
    assert!(
        stopped.origin[1] < face + 0.01,
        "walked through: {stopped:?}"
    );
    assert!(
        stopped.origin[1] > face - 0.25,
        "stopped short: {stopped:?}"
    );
    assert!((stopped.origin[0] - 60.0).abs() < 0.01, "{stopped:?}");
}
