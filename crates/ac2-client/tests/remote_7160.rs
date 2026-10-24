//! Objects other than the player against the Legions 7160 server double: an object's create,
//! its positions (in range, out of physics range, into a cell that is not resident and back),
//! its mode, the 25 s destruction of an object out of view, the server's destroy, and an
//! object that comes back into view when its block becomes resident.

mod common;

use ac2_client::{ClientFrame, FrameInput, ObjectFrame};
use ac2_net::protocol::{heading_to_u8, motion_to_u24};
use ac2_weenie::SessionEvent;
use common::*;
use legions_server_double::{
    create_object_bytes, object_event_bytes, position_bytes, ObjectPosition, ObjectSpawn, Spawn,
};

const NPC: u64 = 0x7000_0000_0000_0042;
const OTHER: u64 = 0x7000_0000_0000_0043;
/// Two blocks north of the spawn block: outside the resident 3 x 3.
const FAR_CELL: u32 = 0x7F81_001C;

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

fn npc_spawn(id: u64, origin: [f32; 3]) -> ObjectSpawn {
    ObjectSpawn {
        id,
        instance: 3,
        visual: 0x1F00_0023,
        cell: CELL,
        origin,
        rotation: [1.0, 0.0, 0.0, 0.0],
        stamps: [10, 1, 1, 0],
    }
}

/// A position for `id` with position stamp `stamp` (teleport and force stamps 1).
fn position(id: u64, stamp: u16, cell: u32, origin: [f32; 3], heading: f32) -> Vec<u8> {
    position_bytes(&ObjectPosition {
        id,
        instance: 3,
        time: 0.0,
        cell,
        origin,
        heading_and_motion: u32::from(heading_to_u8(heading)) << 24
            | motion_to_u24([0.0, 1.0, 0.0]),
        flags: 4,
        stamps: [stamp, 1, 1],
        velocity: None,
    })
}

fn frames(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    count: usize,
) -> Vec<SessionEvent> {
    let idle = FrameInput::default();
    (0..count)
        .flat_map(|_| step(client, server, &idle))
        .collect()
}

#[test]
fn an_object_appears_moves_leaves_view_and_is_destroyed() {
    let (mut client, server) = entered(spawn());
    server
        .borrow_mut()
        .send_event(&create_object_bytes(&npc_spawn(NPC, [90.0, 70.0, GROUND])));
    let events = frames(&mut client, &server, 4);
    assert!(
        events.contains(&SessionEvent::ObjectAdded(NPC)),
        "{events:?}"
    );
    // `FinishEntityCreation` once its Setup loaded: in the world where it was created.
    let npc = object(&client.frame(), NPC).expect("the object");
    assert!(npc.in_world && npc.active, "{npc:?}");
    // Placement puts it in the land cell under it (4, 3).
    assert_eq!(npc.cell, BLOCK | 0x24);
    assert!((npc.origin[0] - 90.0).abs() < 1e-3 && (npc.origin[2] - GROUND).abs() < 0.05);

    // A newer position moves it there, facing the pack's heading.
    server
        .borrow_mut()
        .send_event(&position(NPC, 11, CELL, [95.0, 72.0, GROUND], 90.0));
    let events = frames(&mut client, &server, 2);
    assert!(
        events.iter().any(|e| matches!(
            e,
            SessionEvent::ObjectMoved {
                object: NPC,
                teleported: false,
                ..
            }
        )),
        "{events:?}"
    );
    let npc = object(&client.frame(), NPC).unwrap();
    assert!(
        (npc.origin[0] - 95.0).abs() < 1e-3 && (npc.origin[1] - 72.0).abs() < 1e-3,
        "{npc:?}"
    );
    // The do-motion values as `UnitVectorFromUInt24` gives them back.
    assert_eq!(
        npc.motion,
        ac2_net::protocol::motion_from_u24(motion_to_u24([0.0, 1.0, 0.0]))
    );
    // An older stamp is ignored.
    server
        .borrow_mut()
        .send_event(&position(NPC, 9, CELL, [10.0, 10.0, GROUND], 0.0));
    frames(&mut client, &server, 2);
    assert!((object(&client.frame(), NPC).unwrap().origin[0] - 95.0).abs() < 1e-3);

    // `SetMode` reaches the object.
    server
        .borrow_mut()
        .send_event(&object_event_bytes(0x0001_00A9, NPC, 3, &[0x4100_0003]));
    frames(&mut client, &server, 2);
    assert_eq!(
        object(&client.frame(), NPC).unwrap().mode,
        Some(0x4100_0003)
    );

    // Out of physics range (more than 96 m from the player) it is inactive.
    server
        .borrow_mut()
        .send_event(&position(NPC, 12, 0x807F_001C, [70.0, 70.0, GROUND], 0.0));
    frames(&mut client, &server, 2);
    let npc = object(&client.frame(), NPC).unwrap();
    assert!(npc.in_world && !npc.active, "{npc:?}");
    assert_eq!(npc.cell, 0x807F_001C);

    // Into a cell that is not resident: out of the world, waiting in its lost cell, queued for
    // destruction.
    server
        .borrow_mut()
        .send_event(&position(NPC, 13, FAR_CELL, [70.0, 70.0, GROUND], 0.0));
    frames(&mut client, &server, 2);
    let npc = object(&client.frame(), NPC).unwrap();
    assert!(!npc.in_world, "{npc:?}");
    assert_eq!(npc.lost_cell, Some(FAR_CELL));
    assert!(client.session().is_to_be_destroyed(NPC));

    // Back in a resident cell: in the world again and no longer to be destroyed.
    server
        .borrow_mut()
        .send_event(&position(NPC, 14, CELL, [80.0, 80.0, GROUND], 0.0));
    frames(&mut client, &server, 2);
    let npc = object(&client.frame(), NPC).unwrap();
    assert!(npc.in_world && npc.lost_cell.is_none(), "{npc:?}");
    assert!(!client.session().is_to_be_destroyed(NPC));

    // Out of view again, 25 s later it is gone.
    server
        .borrow_mut()
        .send_event(&position(NPC, 15, FAR_CELL, [70.0, 70.0, GROUND], 0.0));
    frames(&mut client, &server, 2);
    assert!(client.session().is_to_be_destroyed(NPC));
    let mut removed = false;
    for _ in 0..30 {
        server.borrow_mut().elapsed += 1.0;
        removed |= frames(&mut client, &server, 1).contains(&SessionEvent::ObjectRemoved(NPC));
    }
    assert!(removed);
    assert!(object(&client.frame(), NPC).is_none());
}

#[test]
fn a_server_destroy_removes_the_object_at_the_next_object_maintenance() {
    let (mut client, server) = entered(spawn());
    server
        .borrow_mut()
        .send_event(&create_object_bytes(&npc_spawn(
            OTHER,
            [60.0, 60.0, GROUND],
        )));
    frames(&mut client, &server, 4);
    assert!(object(&client.frame(), OTHER).is_some());
    server
        .borrow_mut()
        .send_event(&object_event_bytes(0x0001_00C2, OTHER, 3, &[]));
    let events = frames(&mut client, &server, 3);
    assert!(
        events.contains(&SessionEvent::ObjectRemoved(OTHER)),
        "{events:?}"
    );
    assert!(object(&client.frame(), OTHER).is_none());
}

#[test]
fn an_object_lost_in_a_far_block_comes_back_when_the_block_becomes_resident() {
    let (mut client, server) = entered(spawn());
    // Created where no block is resident: it waits in its lost cell.
    let mut far = npc_spawn(NPC, [70.0, 70.0, GROUND]);
    far.cell = FAR_CELL;
    server.borrow_mut().send_event(&create_object_bytes(&far));
    frames(&mut client, &server, 4);
    let npc = object(&client.frame(), NPC).unwrap();
    assert!(!npc.in_world && npc.lost_cell == Some(FAR_CELL), "{npc:?}");
    // The player teleports one block north: the window moves over the far block
    // (`InitObjCell`), and the object re-enters (`ReEnterVisibility`).
    server
        .borrow_mut()
        .send_player_position(0x7F80_001C, [70.0, 70.0, GROUND], [20, 6, 6]);
    let mut frames_left = 300;
    while !object(&client.frame(), NPC).unwrap().in_world {
        frames(&mut client, &server, 1);
        frames_left -= 1;
        assert!(frames_left > 0, "{:?}", client.frame().blocks);
    }
    assert!(!client.session().is_to_be_destroyed(NPC));
}
