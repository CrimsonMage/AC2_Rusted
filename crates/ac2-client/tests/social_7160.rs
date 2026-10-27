//! The fellowship and allegiance actions against the Legions server double (`AC2_DAT_DIR`
//! for the script library): each `Command::Social` sends the stub's call as the `CPlayer` IFun
//! makes it, the same on the script VM as from the session's stand-in, and the double's
//! answers come back as the fellowship notices; the attack buttons get the server's state.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{NativeObject, PackValue, ScriptCall};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent, SocialRequest};
use common::*;
use legions_server_double::{Spawn, FELLOW, PLAYER};

const TARGET: u64 = 0x2000_0000_0000_0003;

fn requests() -> Vec<SocialRequest> {
    vec![
        SocialRequest::FellowshipCreate {
            name: "Wanderers".into(),
            social: 0,
            looting: 2,
        },
        SocialRequest::FellowshipRecruit(TARGET),
        SocialRequest::FellowshipDismiss(TARGET),
        SocialRequest::FellowshipRecruitAccept,
        SocialRequest::FellowshipRecruitDecline(0x138D),
        SocialRequest::FellowshipLeave,
        SocialRequest::FellowshipDisband,
        SocialRequest::SwearAllegiance,
        SocialRequest::BreakAllegiance(TARGET),
        SocialRequest::AcceptVassal,
        SocialRequest::DeclineVassal(0x138D),
        SocialRequest::QueryAllegianceProfile(PLAYER),
    ]
}

/// The login, then every action and the attack buttons, one per frame: the notices and the
/// calls the double received.
/// The notices of each action, the calls the double received, the XP shares after creating.
type Run = (
    Vec<Vec<GameplayNotice>>,
    Vec<ScriptCall>,
    Option<Vec<(u64, f32)>>,
);

fn run(owned: &[u32]) -> Option<Run> {
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
    server.borrow_mut().send_init_player();
    for _ in 0..8 {
        step(&mut client, &server, &idle);
    }
    let mut commands: Vec<Command> = requests().into_iter().map(Command::Social).collect();
    commands.extend([Command::SetAttacking(true), Command::SetAttacking(false)]);
    let mut notices = Vec::new();
    let mut shares = None;
    for command in commands {
        let input = FrameInput {
            commands: vec![command],
            ..FrameInput::default()
        };
        let mut events = step(&mut client, &server, &input);
        for _ in 0..6 {
            events.extend(step(&mut client, &server, &idle));
        }
        // Right after the create: the XP shares of the (sharing) fellowship.
        if notices.is_empty() {
            shares = client.fellow_xp_hash().ok();
        }
        notices.push(
            events
                .into_iter()
                .filter_map(|e| match e {
                    SessionEvent::Notice(n) => Some(n),
                    _ => None,
                })
                .collect(),
        );
    }
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
    let calls = server.borrow().script_calls.clone();
    Some((notices, calls, shares))
}

#[test]
fn social_actions_go_out_and_the_fellowship_comes_back() {
    let social = ac2_weenie::SocialRequest::FellowshipCreate {
        name: String::new(),
        social: 0,
        looting: 0,
    };
    let stood_in: Vec<u32> = ac2_client::SCRIPT_VM_FUNCTIONS
        .iter()
        .copied()
        .filter(|f| *f >> 8 != social.ifun() >> 8)
        .collect();
    let Some((before, before_calls, _)) = run(&stood_in) else {
        return;
    };
    let (after, after_calls, shares) = run(ac2_client::SCRIPT_VM_FUNCTIONS).unwrap();

    // The VM sends what the stand-in sends. The name's reference differs only where the
    // stand-in cannot follow: its id is the VM's reference-table index, and the VM leaves out
    // the reserved-reference bit `ReferenceTable::AddReference` sets (0x04000000).
    let wanted: Vec<ScriptCall> = requests()
        .iter()
        .map(|r| r.call(ac2_weenie::SocialContext::default()))
        .collect();
    assert_eq!(&before_calls[..12], &wanted[..]);
    assert_eq!(after_calls[1..], before_calls[1..]);
    assert_eq!(after_calls[0].function, 0x820C_001B);
    assert_eq!(
        after_calls[0].arguments[..2],
        [PackValue::Int(2), PackValue::Int(0)]
    );
    assert_eq!(
        after_calls[0].arguments[2].as_native(),
        Some(&NativeObject::WString("Wanderers".into()))
    );
    // `WM_Fellowship` 0x22 byte for byte: the looting method, the social flag (int tags),
    // then the name as one native `wpstring` (type 0x76).
    let (queue, bytes) = before_calls[0].to_event().encode();
    assert_eq!(queue, 3);
    let mut expected = Vec::new();
    for w in [
        0x0001_0053u32,
        0x820C_001B,
        (bytes.len() - 12) as u32,
        1,
        2,
        1,
        0,
        5,
        1,
        1,
    ] {
        expected.extend(w.to_le_bytes());
    }
    for w in [0x4400_0100u32, 0, 0xFFFF_0076] {
        expected.extend(w.to_le_bytes());
    }
    ac2_net::pack_wide_string(&mut expected, 0, "Wanderers");
    expected.extend(0u32.to_le_bytes());
    assert_eq!(bytes, expected);

    // The double's answers, through the VM: the fellowship (two members, the player leading,
    // Birch first in the hash), the recruit, the dismissal, the leave.
    let fellowship = after[0]
        .iter()
        .find_map(|n| match n {
            GameplayNotice::UpdateFellowship(f) => Some(f.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{:?}", after[0]));
    assert_eq!(fellowship.name, "Wanderers");
    assert_eq!(fellowship.leader, PLAYER);
    assert_eq!(
        fellowship.fellows.iter().map(|f| f.0).collect::<Vec<_>>(),
        [FELLOW, PLAYER]
    );
    let birch = &fellowship.fellows[0].1;
    assert_eq!(birch.name.literal.as_deref(), Some("Birch"));
    assert_eq!(
        (
            birch.level,
            birch.health,
            birch.max_health,
            birch.vigor,
            birch.max_vigor,
            birch.cell
        ),
        (4, 40, 50, 20, 30, CELL)
    );
    assert!(after[1]
        .iter()
        .any(|n| matches!(n, GameplayNotice::UpdateFellow { fellow, .. } if *fellow == TARGET)));
    assert!(after[2].contains(&GameplayNotice::ClearFellow(TARGET)));
    assert!(after[5].contains(&GameplayNotice::ClearFellowship));
    // The XP shares (`GetFellowXPHash` -> `FellowshipSystem` 0x41): a non-social fellowship
    // of two within five levels gives each member 0.6 (`FellowshipControl` 0x1).
    let mut shares = shares.unwrap();
    shares.sort_by_key(|s| s.0);
    assert_eq!(shares, [(PLAYER, 0.6), (FELLOW, 0.6)]);
    // The attack buttons: the server's state, on and off, once each (the VM's `CPlayer` 0x2).
    let attack_states = |n: &[GameplayNotice]| {
        n.iter()
            .filter(|n| matches!(n, GameplayNotice::UpdateAttackState(_)))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        attack_states(&after[12]),
        [GameplayNotice::UpdateAttackState(true)]
    );
    assert_eq!(
        attack_states(&after[13]),
        [GameplayNotice::UpdateAttackState(false)]
    );
    assert_eq!(
        attack_states(&before[12]),
        [GameplayNotice::UpdateAttackState(true)]
    );
}

/// The looting method the drop-down chose goes out with Create: the `DoFellowshipCreate` call
/// packs it as the second int (`WM_Fellowship` 0x22), byte for byte, with no server needed.
#[test]
fn the_chosen_looting_method_goes_out_with_create_byte_for_byte() {
    let create = SocialRequest::FellowshipCreate {
        name: "Wanderers".into(),
        social: 0,
        looting: 3,
    };
    let call = create.call(ac2_weenie::SocialContext::default());
    assert_eq!(call.arguments[..2], [PackValue::Int(3), PackValue::Int(0)]);
    let (queue, bytes) = call.to_event().encode();
    assert_eq!(queue, 3);
    let mut expected = Vec::new();
    for w in [
        0x0001_0053u32,
        0x820C_001B,
        (bytes.len() - 12) as u32,
        1,
        3,
        1,
        0,
        5,
        1,
        1,
    ] {
        expected.extend(w.to_le_bytes());
    }
    for w in [0x4400_0100u32, 0, 0xFFFF_0076] {
        expected.extend(w.to_le_bytes());
    }
    ac2_net::pack_wide_string(&mut expected, 0, "Wanderers");
    expected.extend(0u32.to_le_bytes());
    assert_eq!(bytes, expected);
}
