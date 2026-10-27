//! The assistance page's call on the real script library (`AC2_DAT_DIR`): the call is a GUN
//! request (`RequestGUN_CallCreate`), queued only with a GUN service; without one the request
//! answers false and queues nothing, which the page shows as its "can't connect" box.

mod common;

use std::sync::Arc;

use ac2_client::{Client, FrameInput};
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::{ChatRequest, ScriptHost};
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use common::*;
use legions_server_double::{Spawn, PLAYER};

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// The player's script attached, the login's idle frames run, the GUN service off or on.
fn attached(gun: bool) -> Option<Client> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(PLAYER);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    client.set_gun_service(gun);
    server.borrow_mut().send_init_player();
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    client.take_gun_requests();
    Some(client)
}

#[test]
fn the_call_is_a_gun_request_only_with_a_gun_service() {
    let Some(mut client) = attached(false) else {
        return;
    };
    // The panel's send without a GUN service: refused, nothing queued.
    assert!(!client.call_create(0xE000, "Legions", "Ilse", "Speeding"));
    assert!(client.take_gun_requests().is_empty());

    let Some(mut client) = attached(true) else {
        return;
    };
    assert!(client.call_create(0xE000, "Legions", "Ilse", "Speeding"));
    assert_eq!(
        client.take_gun_requests(),
        vec![ChatRequest::CallCreate {
            call_type: 0xE000,
            world: "Legions".into(),
            target: "Ilse".into(),
            description: "Speeding".into(),
        }]
    );
    assert!(client.take_script_problems().is_empty());
}
