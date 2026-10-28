//! `CWPhysicsObject::LookAtSelection` after the selection UI's `SetSelection`: the player looks at
//! its new selection and `ECM_Physics::SendEvt_CLookAt` (0x000100C7) carries the target; with the
//! camera's look-at-selection preference off nothing is sent until a look needs clearing.

mod common;

use ac2_client::{Command, FrameInput};
use common::*;
use legions_server_double::{Spawn, CREATED};

const CELL: u32 = 0x7D64_0011;
const GROUND: f32 = 0.0;
const CLIENT_LOOK_AT: u32 = 0x0001_00C7;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// The targets of every `CLookAt` the client sent since blob `since`.
fn looks(server: &std::cell::RefCell<legions_server_double::State>, since: usize) -> Vec<u64> {
    server.borrow().blobs[since..]
        .iter()
        .filter(|(_, _, data)| data.len() >= 12 && data[..4] == CLIENT_LOOK_AT.to_le_bytes())
        .map(|(_, _, data)| u64::from_le_bytes(data[4..12].try_into().unwrap()))
        .collect()
}

#[test]
fn a_selection_by_the_selection_ui_sends_the_look() {
    let (mut client, server) = entered(spawn());
    let since = server.borrow().blobs.len();
    let input = FrameInput {
        commands: vec![
            Command::Select(Some(CREATED)),
            Command::LookAtSelection(Some(CREATED)),
        ],
        ..FrameInput::default()
    };
    step(&mut client, &server, &input);
    for _ in 0..4 {
        step(&mut client, &server, &FrameInput::default());
    }
    assert_eq!(looks(&server, since), vec![CREATED]);
    // Cleared: the look ends and the empty target goes out.
    let since = server.borrow().blobs.len();
    let input = FrameInput {
        commands: vec![Command::Select(None), Command::LookAtSelection(None)],
        ..FrameInput::default()
    };
    step(&mut client, &server, &input);
    for _ in 0..4 {
        step(&mut client, &server, &FrameInput::default());
    }
    assert_eq!(looks(&server, since), vec![0]);
}
