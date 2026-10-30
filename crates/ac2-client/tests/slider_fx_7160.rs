//! An FX node with a slider (`FXManager::DoSliderFX`), on the real DATs (`AC2_DAT_DIR`) against
//! the Legions 7160 server double: the server's `DoFX` on the player starts an FX of the
//! character's FX table whose node moves the colour-alpha slider, and the player is drawn with
//! that alpha (`PHYSICSOBJREP::Draw`'s colour scale).

mod common;

use std::sync::Arc;

use ac2_client::{DatWorld, FrameInput};
use ac2_db::ResourceStore;
use ac2_net::protocol::Opcode;
use common::*;
use legions_server_double::{object_event_bytes, Spawn, PLAYER};

/// The FX ids of the tables that carry the alpha slider: `0x8100028D` moves it to 1, `0x8100028E`
/// to 0.
const FADE_OUT: u32 = 0x8100_028E;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

#[test]
fn a_slider_fx_moves_the_players_alpha() {
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
    let alpha = |client: &ac2_client::Client| client.frame().player.as_ref().unwrap().anim_scale[3];
    assert_eq!(alpha(&client), 1.0);
    server.borrow_mut().send_event(&object_event_bytes(
        Opcode::DO_FX,
        PLAYER,
        9,
        &[FADE_OUT, 1.0f32.to_bits()],
    ));
    let mut seen = Vec::new();
    for _ in 0..40 {
        step(&mut client, &server, &FrameInput::default());
        seen.push(alpha(&client));
    }
    assert!(
        seen.iter().any(|a| *a < 1.0),
        "the slider FX fades the player"
    );
    assert_eq!(seen.last(), Some(&0.0), "the slider ends at its intensity");
}
