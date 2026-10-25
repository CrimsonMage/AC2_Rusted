//! The server moving the player to an object (`CPhysicsObj::RecvEvt_MoveTo` with an object
//! target) against the Legions 7160 server double: the player walks to the object, following
//! it while it stands there, and stops when it is reached; a target that leaves the world is
//! lost and the move-to ends.

mod common;

use ac2_client::{ClientFrame, FrameInput, ObjectFrame};
use common::*;
use legions_server_double::{create_object_bytes, object_event_bytes, ObjectSpawn, Spawn, PLAYER};

const NPC: u64 = 0x7000_0000_0000_0042;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

fn object(frame: &ClientFrame, id: u64) -> Option<ObjectFrame> {
    frame.objects.iter().find(|o| o.id == id).cloned()
}

/// `MovementParameters::StreamPack` for a move towards an object (flags 1 | 0x20 | 0x40, use
/// cylinders), then the move-to stamp, after the player as sender.
fn move_to_object(target: u64, stamp: u16) -> Vec<u8> {
    let mut b = 0x0001_0083u32.to_le_bytes().to_vec();
    b.extend(PLAYER.to_le_bytes());
    b.extend(9u16.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    b.extend(0x61u32.to_le_bytes());
    // Desired velocity, fail distance, desired distance, maximum success distance.
    for v in [1.0f32, 50.0, 0.5, 1.0] {
        b.extend(v.to_le_bytes());
    }
    b.extend(target.to_le_bytes());
    // No waypoints; context 7.
    b.extend(0u32.to_le_bytes());
    b.extend(7u32.to_le_bytes());
    b.extend(stamp.to_le_bytes());
    b.extend([0, 0]);
    b
}

fn npc(origin: [f32; 3]) -> Vec<u8> {
    create_object_bytes(&ObjectSpawn {
        id: NPC,
        instance: 3,
        visual: 0x1F00_0023,
        cell: CELL,
        origin,
        rotation: [1.0, 0.0, 0.0, 0.0],
        stamps: [10, 1, 1, 0],
    })
}

fn distance(client: &ac2_client::Client) -> f32 {
    let frame = client.frame();
    let player = frame.player.as_ref().unwrap().origin;
    let npc = object(&frame, NPC).unwrap().origin;
    (player[0] - npc[0]).hypot(player[1] - npc[1])
}

#[test]
fn the_server_moves_the_player_to_an_object() {
    let (mut client, server) = entered(spawn());
    let idle = FrameInput::default();
    server.borrow_mut().send_event(&npc([76.0, 70.0, GROUND]));
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert!(object(&client.frame(), NPC).unwrap().in_world);
    assert!((distance(&client) - 6.0).abs() < 0.01);
    server.borrow_mut().send_event(&move_to_object(NPC, 2));
    // It walks over (the motion interpreter's run, not a snap), then stands within the
    // success distance of the object (cylinders: measured between the bodies' edges).
    let mut arrived = None;
    for frame in 0..64 * 10 {
        step(&mut client, &server, &idle);
        if arrived.is_none() && distance(&client) < 1.0 {
            arrived = Some(frame);
        }
    }
    let arrived = arrived.expect("reached the object");
    assert!(arrived > 64, "walked, in {arrived} frames");
    let last = distance(&client);
    assert!(last > 0.1, "stopped short of the object's centre: {last}");
    let before = client.frame().player.unwrap().origin;
    for _ in 0..64 {
        step(&mut client, &server, &idle);
    }
    let after = client.frame().player.unwrap().origin;
    let moved = (after[0] - before[0]).hypot(after[1] - before[1]);
    assert!(
        moved < 0.01,
        "the move-to ended and the player stands: {moved}"
    );
}

#[test]
fn a_target_that_leaves_the_world_ends_the_move_to() {
    let (mut client, server) = entered(spawn());
    let idle = FrameInput::default();
    server.borrow_mut().send_event(&npc([90.0, 70.0, GROUND]));
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    server.borrow_mut().send_event(&move_to_object(NPC, 2));
    for _ in 0..32 {
        step(&mut client, &server, &idle);
    }
    let walking = distance(&client);
    assert!(walking < 19.9, "it set off: {walking}");
    // `RecvEvt_LeaveWorld` (a newer position stamp): the target is lost.
    server
        .borrow_mut()
        .send_event(&object_event_bytes(0x0001_00BF, NPC, 3, &[11]));
    for _ in 0..64 {
        step(&mut client, &server, &idle);
    }
    let before = client.frame().player.unwrap().origin;
    for _ in 0..64 {
        step(&mut client, &server, &idle);
    }
    let after = client.frame().player.unwrap().origin;
    let moved = (after[0] - before[0]).hypot(after[1] - before[1]);
    assert!(moved < 0.01, "stopped: {moved}");
    assert!(after[0] < 89.0, "short of where the target was: {after:?}");
}
