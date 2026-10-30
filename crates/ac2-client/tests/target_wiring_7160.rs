//! The target managers of the objects (`CPhysicsObj::m_pTargetManager`) against the Legions 7160
//! server double: a move-to object registers the player as a voyeur of its target
//! (`MoveToManager::MoveToEvent`'s `SetTarget`), the target leaving the world ends the move-to
//! through its `ExitWorld` notice (`CPhysicsObj::HandleLeaveWorld`), and a teleport lets go of the
//! target and ends the move-to (`CPhysicsObj::HandleTeleportStart`, `LeaveWorld`).

mod common;

use ac2_client::FrameInput;
use common::*;
use legions_server_double::{
    create_object_bytes, move_to_object_bytes, object_event_bytes, ObjectSpawn, Spawn, PLAYER,
};

const NPC: u64 = 0x8000_0000_0000_0042;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
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
        name: "Birch",
    })
}

type Rig = (
    ac2_client::Client,
    std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
);

/// The player and an NPC `distance` metres east of it, the NPC in the world.
fn rig(distance: f32) -> Rig {
    let (mut client, server) = entered(spawn());
    let idle = FrameInput::default();
    server
        .borrow_mut()
        .send_event(&npc([70.0 + distance, 70.0, GROUND]));
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    (client, server)
}

fn frames(rig: &mut Rig, count: usize) {
    let idle = FrameInput::default();
    for _ in 0..count {
        step(&mut rig.0, &rig.1, &idle);
    }
}

#[test]
fn a_move_to_object_makes_the_player_a_voyeur_of_its_target_until_it_is_interrupted() {
    let mut rig = rig(20.0);
    assert_eq!(rig.0.target_of(PLAYER), None);
    rig.1
        .borrow_mut()
        .send_event(&move_to_object_bytes(PLAYER, 9, NPC, 2));
    frames(&mut rig, 4);
    assert!(rig.0.player_moving_to());
    assert_eq!(rig.0.target_of(PLAYER), Some(NPC));
    assert_eq!(rig.0.voyeurs_of(NPC), vec![PLAYER]);
    // The server's move-to is persistent: the player reaches the NPC and keeps following it.
    frames(&mut rig, 64 * 8);
    assert!(rig.0.player_moving_to());
    assert_eq!(rig.0.target_of(PLAYER), Some(NPC));
    // The player interrupts it (`InterruptMoveTo`): the move-to stops, and the target manager
    // lets go of the target at the next quantum (`MoveToManager::Stop`'s `ClearTarget`).
    let interrupt = FrameInput {
        interrupt_move_to: true,
        ..FrameInput::default()
    };
    step(&mut rig.0, &rig.1, &interrupt);
    frames(&mut rig, 20);
    assert!(!rig.0.player_moving_to());
    assert_eq!(rig.0.target_of(PLAYER), None);
    assert!(rig.0.voyeurs_of(NPC).is_empty());
}

#[test]
fn a_target_leaving_the_world_tells_its_voyeur_and_ends_the_move_to() {
    let mut rig = rig(30.0);
    rig.1
        .borrow_mut()
        .send_event(&move_to_object_bytes(PLAYER, 9, NPC, 2));
    frames(&mut rig, 16);
    assert!(rig.0.player_moving_to());
    // `RecvEvt_LeaveWorld` (a newer position stamp).
    rig.1
        .borrow_mut()
        .send_event(&object_event_bytes(0x0001_00BF, NPC, 3, &[11]));
    frames(&mut rig, 120);
    // The target's manager told the player `ExitWorld`: the move-to ended (error 0x37) and the
    // player let go of the target.
    assert!(!rig.0.player_moving_to());
    assert_eq!(rig.0.target_of(PLAYER), None);
}

#[test]
fn a_teleport_lets_go_of_the_target_and_ends_the_move_to() {
    let mut rig = rig(30.0);
    rig.1
        .borrow_mut()
        .send_event(&move_to_object_bytes(PLAYER, 9, NPC, 2));
    frames(&mut rig, 16);
    assert!(rig.0.player_moving_to());
    assert_eq!(rig.0.target_of(PLAYER), Some(NPC));
    rig.1
        .borrow_mut()
        .teleport_player(CELL, [60.0, 60.0, GROUND], 5, 6);
    frames(&mut rig, 8);
    assert!(!rig.0.player_moving_to());
    assert_eq!(rig.0.target_of(PLAYER), None);
    assert!(rig.0.voyeurs_of(NPC).is_empty());
}

#[test]
fn a_target_that_is_not_in_the_world_times_out_after_ten_seconds() {
    // No NPC was made: `GetObjectA` finds nothing, so the voyeur is never added and the
    // move-to waits with its target status undefined until `TargetManager::HandleTargetting`
    // times the target out (ten seconds without a word) and the move-to ends with error 0x37.
    let (client, server) = entered(spawn());
    let mut rig: Rig = (client, server);
    frames(&mut rig, 4);
    rig.1
        .borrow_mut()
        .send_event(&move_to_object_bytes(PLAYER, 9, NPC, 2));
    frames(&mut rig, 64 * 5);
    assert!(rig.0.player_moving_to(), "waiting for the target");
    assert_eq!(rig.0.target_of(PLAYER), Some(NPC));
    frames(&mut rig, 64 * 10);
    assert!(!rig.0.player_moving_to(), "timed out");
}
