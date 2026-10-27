//! The server's knockback on the player (`ECM_Physics::RecvEvt_AddImpulse` and
//! `RecvEvt_AddTargetedImpulse`) against the Legions 7160 server double: an impulse adds to the
//! player's velocity and the player moves and lands again; a targeted impulse pushes the player
//! away from the target; a target not in the world pushes nothing.

mod common;

use ac2_client::FrameInput;
use common::*;
use legions_server_double::{create_object_bytes, object_event_bytes, ObjectSpawn, Spawn, PLAYER};

const NPC: u64 = 0x8000_0000_0000_0043;
/// `ECM_Physics::RecvEvt_AddImpulse`, `RecvEvt_AddTargetedImpulse`.
const ADD_IMPULSE: u32 = 0x0001_0098;
const ADD_TARGETED_IMPULSE: u32 = 0x0001_0099;
/// The player's physics instance stamp in the double (`move_to_7160`).
const PLAYER_INSTANCE: u16 = 9;

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

/// `AddImpulse`: direction (three floats) and the contact word, after the player as sender.
fn impulse(direction: [f32; 3]) -> Vec<u8> {
    let mut args: Vec<u32> = direction.iter().map(|v| v.to_bits()).collect();
    args.push(0);
    object_event_bytes(ADD_IMPULSE, PLAYER, PLAYER_INSTANCE, &args)
}

/// `AddTargetedImpulse`: the target id (two words), magnitude, falloff and vertical.
fn targeted(target: u64, magnitude: f32, falloff: u32, vertical: f32) -> Vec<u8> {
    let args = [
        target as u32,
        (target >> 32) as u32,
        magnitude.to_bits(),
        falloff,
        vertical.to_bits(),
    ];
    object_event_bytes(ADD_TARGETED_IMPULSE, PLAYER, PLAYER_INSTANCE, &args)
}

fn player(client: &ac2_client::Client) -> [f32; 3] {
    client.frame().player.unwrap().origin
}

fn run(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    frames: u32,
) {
    let idle = FrameInput::default();
    for _ in 0..frames {
        step(client, server, &idle);
    }
}

#[test]
fn an_impulse_knocks_the_player_along_and_it_lands_again() {
    let (mut client, server) = entered(spawn());
    run(&mut client, &server, 8);
    let before = player(&client);
    server.borrow_mut().send_event(&impulse([4.0, 0.0, 3.0]));
    run(&mut client, &server, 8);
    let after = player(&client);
    // The push reaches the body and carries it along +x; the walking brake (20 m/s^2) takes it
    // back toward rest, so the window's travel is short.
    assert!(
        after[0] - before[0] > 0.05,
        "knocked along +x: {before:?} -> {after:?}"
    );
    run(&mut client, &server, 3 * 64);
    let landed = player(&client);
    assert!(
        (landed[2] - GROUND).abs() < 0.05,
        "lands on the ground again: {landed:?}"
    );
}

#[test]
fn a_targeted_impulse_pushes_the_player_away_from_the_target() {
    let (mut client, server) = entered(spawn());
    server.borrow_mut().send_event(&npc([66.0, 70.0, GROUND]));
    run(&mut client, &server, 4);
    let before = player(&client);
    server.borrow_mut().send_event(&targeted(NPC, 4.0, 0, 0.0));
    run(&mut client, &server, 8);
    let after = player(&client);
    assert!(
        after[0] - before[0] > 0.05,
        "pushed east, away from the NPC to the west: {before:?} -> {after:?}"
    );
    assert!((after[1] - before[1]).abs() < 0.05, "no sideways push");
}

#[test]
fn a_targeted_impulse_from_a_target_not_in_the_world_does_nothing() {
    let (mut client, server) = entered(spawn());
    run(&mut client, &server, 4);
    let before = player(&client);
    server.borrow_mut().send_event(&targeted(NPC, 4.0, 0, 0.0));
    run(&mut client, &server, 8);
    let after = player(&client);
    let moved = (after[0] - before[0]).hypot(after[1] - before[1]);
    assert!(moved < 0.01, "no target, no push: {moved}");
}
