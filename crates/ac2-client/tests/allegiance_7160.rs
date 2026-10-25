//! The allegiance actions and the invitations against the Legions server double on the
//! script VM (`AC2_DAT_DIR`): a vassal asks and is accepted, the player swears to its
//! selection and breaks again, a fellowship invitation is accepted; each answer comes back as
//! the `CM_Allegiance` / `CM_Fellowship` notices.

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use ac2_client::{Client, Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{PackValue, ScriptCall};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent, SocialRequest};
use common::*;
use legions_server_double::{Spawn, State, PLAYER};

const VASSAL: u64 = 0x5000_0000_0000_0051;
const PATRON: u64 = 0x7000_0000_0000_0042;
const RECRUITER: u64 = 0x5000_0000_0000_0052;

/// One frame with `commands`, then a few idle ones: the notices.
fn frames(
    client: &mut Client,
    server: &Rc<RefCell<State>>,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut events = step(client, server, &input);
    for _ in 0..6 {
        events.extend(step(client, server, &FrameInput::default()));
    }
    events
        .into_iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n),
            _ => None,
        })
        .collect()
}

fn profile(notices: &[GameplayNotice]) -> ac2_weenie::AllegianceProfile {
    notices
        .iter()
        .find_map(|n| match n {
            GameplayNotice::UpdateAllegianceProfile(p) => Some((**p).clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{notices:?}"))
}

#[test]
fn allegiance_and_invitations_answer_through_the_vm() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    step(&mut client, &server, &FrameInput::default());
    server.borrow_mut().send_init_player();
    frames(&mut client, &server, Vec::new());

    // A vassal asks (the double's character accepts requests), and is accepted.
    server.borrow_mut().ask_to_swear(VASSAL, "Ash");
    let asked = frames(&mut client, &server, Vec::new());
    assert!(
        asked.iter().any(|n| matches!(
            n,
            GameplayNotice::RequestVassal { vassal, name }
                if *vassal == VASSAL && name.literal.as_deref() == Some("Ash")
        )),
        "{asked:?}"
    );
    let accepted = frames(
        &mut client,
        &server,
        vec![Command::Social(SocialRequest::AcceptVassal)],
    );
    assert!(accepted.contains(&GameplayNotice::PatronSwearDone {
        vassal: VASSAL,
        kind: 0
    }));
    let p = profile(&accepted);
    assert_eq!(p.member.id, PLAYER);
    assert_eq!(p.vassals.len(), 1);
    assert_eq!(p.vassals[0].0.id, VASSAL);
    assert_eq!(p.vassals[0].0.name.literal.as_deref(), Some("Ash"));

    // Swearing to the selection, then breaking.
    server
        .borrow_mut()
        .send_event(&legions_server_double::create_object_bytes(
            &legions_server_double::ObjectSpawn {
                id: PATRON,
                instance: 3,
                visual: 0x1F00_0023,
                cell: CELL,
                origin: [80.0, 70.0, GROUND],
                rotation: [1.0, 0.0, 0.0, 0.0],
                stamps: [10, 1, 1, 0],
                name: "Birch",
            },
        ));
    frames(&mut client, &server, vec![]);
    frames(&mut client, &server, vec![Command::Select(Some(PATRON))]);
    let sworn = frames(
        &mut client,
        &server,
        vec![Command::Social(SocialRequest::SwearAllegiance)],
    );
    assert!(sworn.contains(&GameplayNotice::VassalSwearDone {
        patron: PATRON,
        kind: 0
    }));
    let p = profile(&sworn);
    assert_eq!((p.patron.id, p.monarch.id), (PATRON, PATRON));
    assert!(p.patron_online && p.monarch_online);
    assert_eq!(p.total, 3);
    let broken = frames(
        &mut client,
        &server,
        vec![Command::Social(SocialRequest::BreakAllegiance(PATRON))],
    );
    // No patron any more: the adaptor keeps its default member (`AllegianceProfile` 0x2 skips
    // a null patron).
    let p = profile(&broken);
    assert_ne!(p.patron.id, PATRON);
    assert!(!p.patron_online);

    // A fellowship invitation, accepted: the recruiter's fellowship, the recruiter leading.
    server
        .borrow_mut()
        .invite_to_fellowship(RECRUITER, "Oak", "Grove");
    let invited = frames(&mut client, &server, Vec::new());
    assert!(
        invited.iter().any(|n| matches!(
            n,
            GameplayNotice::RequestRecruitment { leader, .. } if *leader == RECRUITER
        )),
        "{invited:?}"
    );
    let joined = frames(
        &mut client,
        &server,
        vec![Command::Social(SocialRequest::FellowshipRecruitAccept)],
    );
    let fellowship = joined
        .iter()
        .find_map(|n| match n {
            GameplayNotice::UpdateFellowship(f) => Some(f.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{joined:?}"));
    assert_eq!(
        (fellowship.name.as_str(), fellowship.leader),
        ("Grove", RECRUITER)
    );

    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
    // The calls, as the VM's `CPlayer` IFuns sent them: the ids the requests stored.
    let calls = server.borrow().script_calls.clone();
    for want in [
        ScriptCall::new(0x8203_001C, vec![PackValue::LongInt(VASSAL)]).unwrap(),
        ScriptCall::new(0x8203_000A, vec![PackValue::LongInt(PATRON)]).unwrap(),
        ScriptCall::new(0x8203_0009, vec![PackValue::LongInt(PATRON)]).unwrap(),
        ScriptCall::new(0x820C_000D, vec![PackValue::LongInt(RECRUITER)]).unwrap(),
    ] {
        assert!(calls.contains(&want), "{want:?} not in {calls:?}");
    }
}
