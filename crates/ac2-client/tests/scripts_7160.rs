//! The weenies' scripts on the script VM through the client core, against the Legions server
//! double (`AC2_DAT_DIR` for the script library): the player's process starts from its weenie's
//! package, and a server call handed to the VM comes back as the VM's notice, once.

mod common;

use std::sync::Arc;

use ac2_client::FrameInput;
use ac2_core::DataId;
use ac2_net::protocol::{script_function, ScriptMessage, StringInfo};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, PLAYER};

fn host() -> Option<ScriptHost> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    Some(ScriptHost::new(program, Box::new(data)))
}

#[test]
fn a_call_handed_to_the_vm_comes_back_as_its_notice() {
    let Some(host) = host() else {
        return;
    };
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    // The calls the VM covers are its from now on.
    client.attach_scripts(host, ac2_client::SCRIPT_VM_FUNCTIONS);
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    assert!(
        client.script_host().unwrap().instance(PLAYER).is_some(),
        "{:?}",
        client.take_script_problems()
    );
    let text = StringInfo::literal("Welcome to Dereth");
    server
        .borrow_mut()
        .send_script(&ScriptMessage::DisplayStringInfo {
            function: script_function::DISPLAY_STRING_INFO,
            text: text.clone(),
            kind: 4,
        });
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(step(&mut client, &server, &idle));
    }
    let lines: Vec<&GameplayNotice> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n @ GameplayNotice::DisplayStringInfo { .. }) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(
        lines,
        [&GameplayNotice::DisplayStringInfo {
            kind: 4,
            text: text.clone()
        }],
        "{:?}",
        client.take_script_problems()
    );
    // `WM_Communication` 0x23 and 0x12 (a `wpstring` made a literal) the same way, each once,
    // as the stand-in raised them.
    server
        .borrow_mut()
        .send_script(&ScriptMessage::DisplayStringInfo {
            function: script_function::DISPLAY_STRING_INFO_2,
            text: text.clone(),
            kind: 8,
        });
    server
        .borrow_mut()
        .send_script(&ScriptMessage::DisplayText {
            text: "Welcome to Dereth".into(),
            kind: 16,
        });
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(step(&mut client, &server, &idle));
    }
    let lines: Vec<GameplayNotice> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n @ GameplayNotice::DisplayStringInfo { .. }) => Some(n.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        lines,
        [
            GameplayNotice::DisplayStringInfo {
                kind: 8,
                text: text.clone()
            },
            GameplayNotice::DisplayStringInfo { kind: 16, text }
        ],
        "{:?}",
        client.take_script_problems()
    );
}
