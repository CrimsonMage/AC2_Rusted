//! The assistance page's call on the real script library (`AC2_DAT_DIR`): the call is a GUN
//! request (`RequestGUN_CallCreate`) to `TurbineGun`; without one the request answers false,
//! which the page shows as its "can't connect" box.

mod common;

use std::sync::Arc;

use ac2_client::{Client, FrameInput};
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
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
fn attached(gun: bool) -> Option<(Client, ac2_gun::test_support::TestGunHandle)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let handle = ac2_gun::test_support::TestGunHandle::new();
    ac2_weenie::gun::shared().set_turbine_gun(None);
    if gun {
        let tg = std::rc::Rc::new(ac2_gun::TurbineGun::new(0x0409));
        let mut loader = ac2_gun::test_support::TestLoader::new(handle.gun());
        assert!(tg.init("", &mut loader));
        assert!(tg.request_connect("gun", 2310, 0, "1.0", "K", 0));
        handle.queue_message(ac2_gun::GunQueueMessage {
            flags: 4,
            gmid: 1,
            param: 0,
            cb: 0,
            message: ac2_gun::Message {
                inited: true,
                id: ac2_gun::message_id::CONNECT_EVENT,
                result: None,
                body: ac2_gun::MessageBody::None,
            },
        });
        tg.check_for_messages();
        ac2_weenie::gun::shared().set_turbine_gun(Some(tg));
    }
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
    server.borrow_mut().send_init_player();
    server.borrow_mut().send_world_name("Legions");
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    handle.clear_calls();
    Some((client, handle))
}

#[test]
fn the_call_is_a_gun_request_only_with_a_turbine_gun() {
    let Some((mut client, handle)) = attached(false) else {
        return;
    };
    // The panel's send without a TurbineGun: refused.
    assert!(!client.call_create(0xE000, "Legions", "Ilse", "Speeding"));
    assert!(handle.calls().is_empty());

    let Some((mut client, handle)) = attached(true) else {
        return;
    };
    assert!(client.call_create(0xE000, "Legions", "Ilse", "Speeding"));
    let calls = handle.calls();
    assert!(
        matches!(&calls[0], ac2_gun::test_support::Call::CallCreate { .. }),
        "{calls:?}"
    );
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn the_world_name_notice_reaches_the_client_and_the_turbine_gun() {
    let Some((client, _)) = attached(true) else {
        return;
    };
    // `Turbine::Client::SetWorldName` from the notice; `TurbineGun::SetWorldName` is the
    // gmClient's, attached with `attach_gun` (not here), so the script's name is empty.
    assert_eq!(client.world_name(), "Legions");
    assert_eq!(ac2_gun::turbine_gun_utils::get_world_name(), "");
}
