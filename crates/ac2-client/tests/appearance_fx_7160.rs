//! An FX that changes its emitter's appearance (`FXManager::DoAppearanceFX`), on the real DATs
//! (`AC2_DAT_DIR`) against the Legions 7160 server double: the server's `DoFX` on the player starts
//! an FX of the character's FX table whose nodes name appearance tables, and the player is drawn
//! with them applied (`CPhysicsObj::DoAppearanceFX`: the effects description merged onto the
//! local one); the server's `StopFX` removes them (`CPhysicsObj::RemoveAppearanceFX`).

mod common;

use std::sync::Arc;

use ac2_client::{Client, DatWorld, FrameInput};
use ac2_db::ResourceStore;
use ac2_net::protocol::Opcode;
use common::*;
use legions_server_double::{object_event_bytes, Spawn, PLAYER};

/// An FX of the base character visual's table (`0x1F000000`) whose nodes apply appearance
/// tables with key 2: `0x200001E8` among them.
const APPEARANCE_FX: u32 = 0x4100_0002;
const TABLE: u32 = 0x2000_01E8;
const KEY: u32 = 2;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// The scalar of the appearance key on the player as drawn, when applied.
fn applied(client: &Client) -> Option<f32> {
    client.frame().player.as_ref()?.appearance.applied_scalar(
        ac2_physics::GLOBAL_PART_GROUP,
        TABLE,
        KEY,
    )
}

#[test]
fn an_fx_applies_its_appearance_to_the_player_until_it_stops() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let world = Arc::new(DatWorld::new(Arc::clone(&store)).unwrap());
    let (mut client, server) = entered_with(spawn(), world);
    server.borrow_mut().send_init_player();
    // The character's body (and so its FX table) loads on the loader thread.
    for _ in 0..60 {
        step(&mut client, &server, &FrameInput::default());
    }
    assert_eq!(applied(&client), None, "not applied before the FX");
    let fx = |opcode: u32, words: &[u32]| object_event_bytes(opcode, PLAYER, 9, words);
    server
        .borrow_mut()
        .send_event(&fx(Opcode::DO_FX, &[APPEARANCE_FX, 1.0f32.to_bits()]));
    let mut on = None;
    for _ in 0..30 {
        step(&mut client, &server, &FrameInput::default());
        on = applied(&client);
        if on.is_some() {
            break;
        }
    }
    assert_eq!(
        on,
        Some(1.0),
        "the FX's appearance is applied at its scalar"
    );
    server
        .borrow_mut()
        .send_event(&fx(Opcode::STOP_FX, &[APPEARANCE_FX]));
    for _ in 0..10 {
        step(&mut client, &server, &FrameInput::default());
    }
    assert_eq!(applied(&client), None, "StopFX removes it");
}

/// `ECM_Physics::RecvEvt_DoFX_Private`: the server's private FX list plays on the player, each FX
/// at full scalar (`CPhysicsObj::RecvEvt_DoFX_Private`), so its appearance is applied as `DoFX`'s.
#[test]
fn a_private_fx_plays_on_the_player() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let world = Arc::new(DatWorld::new(Arc::clone(&store)).unwrap());
    let (mut client, server) = entered_with(spawn(), world);
    server.borrow_mut().send_init_player();
    for _ in 0..60 {
        step(&mut client, &server, &FrameInput::default());
    }
    // The message: its opcode, then `PSmartArray<unsigned long>` (count, then the FX ids).
    let message: Vec<u8> = [Opcode::DO_FX_PRIVATE, 1, APPEARANCE_FX]
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect();
    server.borrow_mut().send_event(&message);
    let mut on = None;
    for _ in 0..30 {
        step(&mut client, &server, &FrameInput::default());
        on = applied(&client);
        if on.is_some() {
            break;
        }
    }
    assert_eq!(
        on,
        Some(1.0),
        "the private FX plays on the player at full scalar"
    );
}
