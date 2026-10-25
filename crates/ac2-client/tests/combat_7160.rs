//! Combat with the player's script on the VM (`AC2_DAT_DIR`): a skill used from the shortcut bar
//! against the selected creature sends the special attack request as the VM packs it, and the
//! double's answers (the attack state, the attack's end, a maneuver's toggle) reach the bar's
//! notices. The start of a special attack has no server forwarder, so the bar never sees one
//! here (see `ac2_net::protocol::CombatMessage`); damage and death are not answered yet.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{CombatMessage, ScriptCall, SpecialAttackRequest};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, CREATURE_ONE, MELEE_AUTO_ATTACK, PLAYER};

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// The script with the player attached and the double's shortcut bar on the VM (slot 0 holds
/// Melee Auto Attack, a skill).
fn with_script() -> Option<(ac2_client::Client, Shared)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
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
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    server.borrow_mut().send_init_player();
    run(&mut client, &server, Vec::new());
    Some((client, server))
}

/// Runs the commands for ten frames; the notices the frames raised, in order.
fn run(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut notices = Vec::new();
    for i in 0..10 {
        let frame = if i == 0 { &input } else { &idle };
        for event in step(client, server, frame) {
            if let SessionEvent::Notice(notice) = event {
                notices.push(notice);
            }
        }
    }
    notices
}

/// The player selects the creature (the target of a special attack).
fn select_creature(client: &mut ac2_client::Client, server: &Shared) {
    run(client, server, vec![Command::Select(Some(CREATURE_ONE))]);
}

#[test]
fn a_skill_from_the_shortcut_bar_requests_a_special_attack_with_its_bytes() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    let before = server.borrow().script_bytes.len();
    assert_eq!(client.use_shortcut(0), Ok(1));
    let notices = run(&mut client, &server, Vec::new());
    let sent = server.borrow().script_bytes[before..].to_vec();
    // The VM's `WM_Combat` 0x17 call: the attack (1), the maneuver (the skill, 6), the target (the
    // selection), each packed as a PackBuffer tag then value (ints tagged 1, the long tagged 2).
    let expected: Vec<u8> = [1u32, 1, 1, 6, 2]
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .chain(
            [0x0A01u32, 0x8000_0000]
                .iter()
                .flat_map(|w| w.to_le_bytes()),
        )
        .collect();
    assert_eq!(sent.len(), 1, "one request: {sent:?}");
    assert_eq!(sent[0].0, 0x8204_000B);
    assert_eq!(sent[0].1, expected);
    // The double reads the request back as the codec's layout.
    let request = server
        .borrow()
        .script_calls
        .iter()
        .find_map(SpecialAttackRequest::from_call);
    assert_eq!(
        request,
        Some(SpecialAttackRequest {
            attack: 1,
            maneuver: MELEE_AUTO_ATTACK,
            target: CREATURE_ONE,
        })
    );
    // The bar's maneuver goes pending at the request; the double's answers follow: the attack
    // state up, then the attack ends.
    assert_eq!(
        notices,
        vec![
            GameplayNotice::RequestSpecialAttack {
                maneuver: MELEE_AUTO_ATTACK,
                attack: 1,
            },
            GameplayNotice::UpdateAttackState(true),
            GameplayNotice::EndSpecialAttack(1),
        ]
    );
    assert!(client.take_script_problems().is_empty());
}

/// With nothing selected the VM still sends the request, with no target (the zero long int); the
/// double refuses a target it does not know, so the bar only sees its own pending maneuver.
#[test]
fn a_skill_without_a_selection_requests_no_target_and_is_not_answered() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let before = server.borrow().script_bytes.len();
    assert_eq!(client.use_shortcut(0), Ok(1));
    let notices = run(&mut client, &server, Vec::new());
    let sent = server.borrow().script_bytes[before..].to_vec();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, 0x8204_000B);
    let request = server
        .borrow()
        .script_calls
        .iter()
        .find_map(SpecialAttackRequest::from_call);
    assert_eq!(request.map(|r| r.target), Some(0));
    assert_eq!(
        notices,
        vec![GameplayNotice::RequestSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            attack: 1,
        }]
    );
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn a_maneuver_toggle_reaches_the_bar() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    server
        .borrow_mut()
        .send_combat(&CombatMessage::ToggleSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            toggled: true,
        });
    let notices = run(&mut client, &server, Vec::new());
    assert_eq!(
        notices,
        vec![GameplayNotice::ToggleSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            toggled: true,
        }]
    );
    server
        .borrow_mut()
        .send_combat(&CombatMessage::ToggleSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            toggled: false,
        });
    let notices = run(&mut client, &server, Vec::new());
    assert_eq!(
        notices,
        vec![GameplayNotice::ToggleSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            toggled: false,
        }]
    );
    assert!(client.take_script_problems().is_empty());
}

/// The ScriptCall form of the request is the one the client's codec builds too.
#[test]
fn the_request_is_the_codecs_call() {
    let request = SpecialAttackRequest {
        attack: 1,
        maneuver: MELEE_AUTO_ATTACK,
        target: CREATURE_ONE,
    };
    let call: ScriptCall = request.to_call();
    assert_eq!(call.function, 0x8204_000B);
    assert_eq!(SpecialAttackRequest::from_call(&call), Some(request));
}
