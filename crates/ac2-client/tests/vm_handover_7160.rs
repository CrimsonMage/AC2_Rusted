//! The calls handed to the script VM (`SCRIPT_VM_FUNCTIONS`: the login, the selection info,
//! `SetSelection`) raise what the session's stand-ins raised, against the Legions server double
//! (`AC2_DAT_DIR` for the script library): the same vitals, inventory refresh, selection and
//! examination notices, the same `SetTarget`, the same panels.

mod common;

use std::sync::Arc;

use ac2_client::{ClientFrame, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{script_function, PackValue, ScriptCall, ScriptRequest, SelectionInfo};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{vital, GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, PLAYER};

const NPC: u64 = 0x8000_0000_0000_0042;

/// The login, then selecting a creature: the notices, the server's requests, the frame.
type Run = (
    Vec<GameplayNotice>,
    Vec<ScriptRequest>,
    Vec<ScriptCall>,
    ClientFrame,
);

fn login_and_select(owned: &[u32]) -> Option<Run> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    client.attach_scripts(ScriptHost::new(program, Box::new(data)), owned);
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    assert!(client.script_host().unwrap().instance(PLAYER).is_some());
    server.borrow_mut().send_init_player();
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(step(&mut client, &server, &idle));
    }
    let info = SelectionInfo {
        kind: SelectionInfo::CREATURE,
        cur_health: 7,
        max_health: 9,
        ..Default::default()
    };
    server
        .borrow_mut()
        .script_world
        .selection_infos
        .insert(NPC, info);
    server
        .borrow_mut()
        .send_event(&legions_server_double::create_object_bytes(
            &legions_server_double::ObjectSpawn {
                id: NPC,
                instance: 3,
                visual: 0x1F00_0023,
                cell: CELL,
                origin: [80.0, 70.0, GROUND],
                rotation: [1.0, 0.0, 0.0, 0.0],
                stamps: [10, 1, 1, 0],
                name: "Birch",
            },
        ));
    for _ in 0..4 {
        events.extend(step(&mut client, &server, &idle));
    }
    let select = FrameInput {
        commands: vec![ac2_client::Command::Select(Some(NPC))],
        ..FrameInput::default()
    };
    events.extend(step(&mut client, &server, &select));
    for _ in 0..8 {
        events.extend(step(&mut client, &server, &idle));
    }
    // The bar's attack buttons, then its set.
    for command in [
        ac2_client::Command::SetAttacking(true),
        ac2_client::Command::SetAttacking(false),
        ac2_client::Command::SetShortcutSet(1),
    ] {
        let input = FrameInput {
            commands: vec![command],
            ..FrameInput::default()
        };
        events.extend(step(&mut client, &server, &input));
    }
    for _ in 0..8 {
        events.extend(step(&mut client, &server, &idle));
    }
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
    // The skill panel's checks run on the VM: a skill the player lacks cannot be raised
    // (`Skill_NotTrained`), and every check answers.
    if owned.contains(&script_function::INIT_PLAYER) {
        let skill = 0x41;
        assert_eq!(
            client.can_skill_be_raised(skill),
            Ok(ac2_weenie::skill_error::NOT_TRAINED)
        );
        assert!(client.can_skill_be_trained(skill).is_ok());
        assert!(client.can_skill_be_untrained(skill).is_ok());
        assert!(client.skill_cost_to_train(skill).is_ok());
        assert!(client.is_hero_skill(skill).is_ok());
    }
    let notices = events
        .into_iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n),
            _ => None,
        })
        .collect();
    let requests = server.borrow().script_requests.clone();
    let calls = server.borrow().script_calls.clone();
    Some((notices, requests, calls, client.frame().clone()))
}

fn of_kind(notices: &[GameplayNotice], keep: fn(&GameplayNotice) -> bool) -> Vec<GameplayNotice> {
    notices.iter().filter(|n| keep(n)).cloned().collect()
}

#[test]
fn the_vm_raises_what_the_stand_ins_raised() {
    let stood_in = &ac2_client::SCRIPT_VM_FUNCTIONS[..3];
    let Some((before, before_requests, before_calls, before_frame)) = login_and_select(stood_in)
    else {
        return;
    };
    let (after, after_requests, after_calls, after_frame) =
        login_and_select(ac2_client::SCRIPT_VM_FUNCTIONS).unwrap();
    assert!(ac2_client::SCRIPT_VM_FUNCTIONS.contains(&script_function::INIT_PLAYER));

    // The login's vitals: the same, and the VM goes on to the experience bar (`CPlayer` 0x65's
    // last step, which the stand-in left out).
    let vitals = |n: &GameplayNotice| {
        matches!(
            n,
            GameplayNotice::UpdateVital { .. } | GameplayNotice::UpdateHeroStatus { .. }
        )
    };
    let mut expected = of_kind(&before, vitals);
    expected.push(GameplayNotice::UpdateVital {
        vital: vital::EXPERIENCE,
        current: 0,
        max: 40,
    });
    assert_eq!(of_kind(&after, vitals), expected);
    let inventory = |n: &GameplayNotice| {
        matches!(
            n,
            GameplayNotice::UpdateContainer { .. } | GameplayNotice::UpdateInventory { .. }
        )
    };
    assert_eq!(of_kind(&after, inventory), of_kind(&before, inventory));
    // The selection and the examination request it changes, and the one `SetTarget`.
    let selection = |n: &GameplayNotice| {
        matches!(
            n,
            GameplayNotice::SelectionChanged(_) | GameplayNotice::ExaminationRequestChanged(_)
        )
    };
    assert_eq!(of_kind(&after, selection), of_kind(&before, selection));
    assert_eq!(after_requests, before_requests);
    assert_eq!(after_requests, [ScriptRequest::SetTarget(NPC)]);
    // Attacking the selection, then stopping: `WM_Combat` 0x16 with its id, 0x14.
    let combat = vec![
        ScriptCall::new(0x8204_0008, vec![PackValue::LongInt(NPC)]).unwrap(),
        ScriptCall::new(0x8204_0010, vec![]).unwrap(),
    ];
    assert_eq!(before_calls, combat);
    assert_eq!(after_calls, combat);
    assert_eq!(after_frame.progress.shortcuts.set, 1);
    // `UpdateSelectionInfo` is raised on the player's weenie (retail's notice names no other
    // object); the panel matches it on the current selection either way.
    assert!(after.iter().any(|n| matches!(
        n,
        GameplayNotice::UpdateSelectionInfo { id, .. } if *id == PLAYER
    )));
    assert_eq!(after_frame.selection, before_frame.selection);
    assert_eq!(after_frame.selection.unwrap().info.unwrap().cur_health, 7);
    // The vitals panel and the inventory follow the VM's notices and the session's mirrors.
    let mut vitals = before_frame.player_vitals;
    vitals.experience = after_frame.player_vitals.experience;
    assert_eq!(after_frame.player_vitals, vitals);
    assert_eq!(after_frame.inventory, before_frame.inventory);
    assert_eq!(after_frame.progress, before_frame.progress);
}
