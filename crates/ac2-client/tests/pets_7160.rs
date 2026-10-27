//! The pet panel's requests on the player's script (`AC2_DAT_DIR`): `gmCWInterface::RequestPet*`
//! and `RequestSetPetName` run their IFuns on the VM, which sends the script's calls. The IFun's
//! values are pushed as `ggmCWInterface::CallIFun_*` pushes them (the context first, then the
//! mode or the name), so the packed bytes are the script's own packing of those values.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_net::protocol::{PackValue, ScriptCall};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use common::*;
use legions_server_double::{Spawn, PLAYER};

/// The pet the requests name: a long id whose low word is 0x0A01 (a tag-2 long on the wire).
const PET: u64 = 0x8000_0000_0000_0A01;
/// `PetAttack`'s, `PetMode`'s and `PetRename`'s wire calls (`CALLS`).
const REQUEST_PET_RETURN: u32 = 0x8212_0037;
const REQUEST_SET_PET_NAME: u32 = 0x8212_0038;
const REQUEST_PET_BUFF: u32 = 0x8212_003A;
const REQUEST_PET_ATTACK: u32 = 0x8212_003B;
const REQUEST_PET_HEAL: u32 = 0x8212_003C;
const REQUEST_PET_MODE: u32 = 0x8212_0073;
/// `gmPetUI`'s Defensive mode (`ModeTable`).
const DEFENSIVE: u32 = 0x4000_0004;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// The player with the script attached and the double's world entered.
fn with_script() -> Option<(ac2_client::Client, Shared)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset.clone(), store.clone());
    data.preload_tables();
    let names = DatScriptData::new(dataset, store);
    let (mut client, server) = entered(spawn());
    client
        .session_mut()
        .set_string_tables(ac2_client::string_table_ids(&names));
    client
        .session_mut()
        .set_chat_commands(ac2_client::chat_commands(&names));
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(PLAYER);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    step(&mut client, &server, &FrameInput::default());
    server.borrow_mut().send_init_player();
    server.borrow_mut().spawn_radar_objects();
    Some((client, server))
}

/// Runs the command for ten frames; the `(function, bytes)` the VM sent meanwhile.
fn sent_by(
    client: &mut ac2_client::Client,
    server: &Shared,
    command: Command,
) -> Vec<(u32, Vec<u8>)> {
    let before = server.borrow().script_bytes.len();
    let input = FrameInput {
        commands: vec![command],
        ..FrameInput::default()
    };
    for i in 0..10 {
        let frame = if i == 0 {
            input.clone()
        } else {
            FrameInput::default()
        };
        step(client, server, &frame);
    }
    server.borrow().script_bytes[before..].to_vec()
}

/// A long on the wire: the tag 2, then the id's low and high words.
fn long_bytes(id: u64) -> Vec<u8> {
    [2u32, id as u32, (id >> 32) as u32]
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect()
}

#[test]
fn pet_attack_sends_the_pet_as_a_long() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let sent = sent_by(&mut client, &server, Command::PetAttack(PET));
    assert_eq!(sent, vec![(REQUEST_PET_ATTACK, long_bytes(PET))]);
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn pet_buff_heal_and_return_send_their_calls_with_no_values() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    for (command, function) in [
        (Command::PetBuff, REQUEST_PET_BUFF),
        (Command::PetHeal, REQUEST_PET_HEAL),
        (Command::PetReturn, REQUEST_PET_RETURN),
    ] {
        let sent = sent_by(&mut client, &server, command);
        assert_eq!(sent, vec![(function, Vec::new())]);
    }
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn pet_mode_sends_the_mode_then_the_pet() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let sent = sent_by(
        &mut client,
        &server,
        Command::PetMode {
            pet: PET,
            mode: DEFENSIVE,
        },
    );
    assert_eq!(sent.len(), 1, "{sent:?}");
    let (function, bytes) = &sent[0];
    assert_eq!(*function, REQUEST_PET_MODE);
    // The VM's packing: the mode as an int (tag 1), then the pet as a long.
    let values = ScriptCall::decode(*function, bytes)
        .expect("a pet mode call")
        .expect("decodes");
    assert_eq!(
        values.arguments,
        vec![PackValue::Int(DEFENSIVE), PackValue::LongInt(PET)]
    );
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn pet_rename_sends_the_name_then_the_pet() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let sent = sent_by(
        &mut client,
        &server,
        Command::PetRename {
            pet: PET,
            name: "Rex".into(),
        },
    );
    assert_eq!(sent.len(), 1, "{sent:?}");
    let (function, bytes) = &sent[0];
    assert_eq!(*function, REQUEST_SET_PET_NAME);
    let call = ScriptCall::decode(*function, bytes)
        .expect("a rename call")
        .expect("decodes");
    // The name is a string reference the VM created (`WString "Rex"`), then the pet's long.
    assert_eq!(call.arguments.len(), 2, "{call:?}");
    assert!(
        format!("{:?}", call.arguments[0]).contains("\"Rex\""),
        "{call:?}"
    );
    assert_eq!(call.arguments[1], PackValue::LongInt(PET));
    assert!(client.take_script_problems().is_empty());
}
