//! Selection and examination against the Legions server double: the client's script calls go
//! out byte for byte as the `WM_Player` / `WM_Examination` stubs pack them, and the double's
//! answers come back as the notices `CPlayer` raises.

use std::time::Duration;

use ac2_net::protocol::{
    ExaminationDataNode, ExaminationProfile, ExaminationRequest, ExaminationValue, ScriptRequest,
    SelectionInfo,
};
use ac2_net::Credential;
use ac2_weenie::{
    CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent, NO_SELECTION,
};
use legions_server_double::{addr, double, words, Spawn, SERVER};

const TARGET: u64 = 0x8000_0002_0000_0042;

fn config() -> OnlineConfig {
    OnlineConfig {
        server: addr(SERVER),
        account: "local".into(),
        credential: Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::First,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    }
}

fn notices(events: &[SessionEvent]) -> Vec<GameplayNotice> {
    events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n.clone()),
            _ => None,
        })
        .collect()
}

/// Polls until the double's answers arrived.
fn settle(
    session: &mut OnlineSession,
    state: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) -> Vec<SessionEvent> {
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }
    events
}

#[test]
fn selecting_and_examining_an_object_round_trips_through_the_double() {
    let (transport, state) = double(Spawn::default());
    let info = SelectionInfo {
        kind: SelectionInfo::CREATURE,
        cur_health: 40,
        pk_damage: 3,
        max_health: 50,
        cur_vigor: 20,
        pk_vigor_loss: 1,
        max_vigor: 25,
    };
    let profile = ExaminationProfile {
        request: ExaminationRequest::default(),
        nodes: vec![ExaminationDataNode {
            order: 1,
            appearance: 0x11,
            value: ExaminationValue::Int(12),
        }],
        available: 0x203,
    };
    {
        let mut s = state.borrow_mut();
        s.script_world.selection_infos.insert(TARGET, info);
        s.script_world.profiles.insert(TARGET, profile.clone());
    }
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();

    // `CPlayer::SetSelection`: the target goes out, the selection and the examination request
    // change locally (the examination panel is closed, so the server is not asked).
    let local = session.select(TARGET);
    assert_eq!(
        notices(&local),
        vec![
            GameplayNotice::SelectionChanged(TARGET),
            GameplayNotice::ExaminationRequestChanged(ExaminationRequest::object(TARGET)),
        ]
    );
    let answered = settle(&mut session, &state);
    let shown = SelectionInfo {
        pk_damage: 0,
        pk_vigor_loss: 0,
        ..info
    };
    assert_eq!(
        notices(&answered),
        vec![GameplayNotice::UpdateSelectionInfo {
            id: TARGET,
            info: shown
        }]
    );
    assert_eq!(session.player_script().selection(), Some(TARGET));

    // The examination panel opens: the request goes out and the profile comes back.
    session.start_examination(0x203);
    let answered = settle(&mut session, &state);
    let expected = ExaminationProfile {
        request: ExaminationRequest::object(TARGET),
        ..profile
    };
    assert_eq!(
        notices(&answered),
        vec![GameplayNotice::UpdateExaminationProfile(expected)]
    );
    // It closes: a cleared request stops the server's examination.
    session.stop_examination();
    session.select(NO_SELECTION);
    settle(&mut session, &state);

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    assert_eq!(
        s.script_requests,
        vec![
            ScriptRequest::SetTarget(TARGET),
            ScriptRequest::Examine(ExaminationRequest::object(TARGET)),
            ScriptRequest::Examine(ExaminationRequest::default()),
            ScriptRequest::SetTarget(NO_SELECTION),
        ]
    );
    // Byte for byte: the `WM_Player` stub's one long int, and the `WM_Examination` stub's
    // `ExaminationRequest` reference (one loaded native heap object, type 0x9A).
    let interp: Vec<&Vec<u8>> = s
        .blobs
        .iter()
        .filter(|(_, queue, data)| *queue == 3 && data[..4] == [0x53, 0, 1, 0])
        .map(|(_, _, data)| data)
        .collect();
    let mut target = Vec::new();
    words(&mut target, &[0x0001_0053, 0x8212_0016, 12, 2]);
    target.extend(TARGET.to_le_bytes());
    assert_eq!(interp[0], &target);
    let mut examine = Vec::new();
    words(
        &mut examine,
        &[0x0001_0053, 0x820A_0007, 48, 5, 1, 1, 0x4400_0100, 0],
    );
    words(&mut examine, &[0xFFFF_009A, 0x4000_0002, 0]);
    examine.extend(TARGET.to_le_bytes());
    words(&mut examine, &[0x203, 0]);
    assert_eq!(interp[1], &examine);
    let mut clear = Vec::new();
    words(
        &mut clear,
        &[0x0001_0053, 0x820A_0007, 36, 5, 1, 1, 0x4400_0100, 0],
    );
    words(&mut clear, &[0xFFFF_009A, 0, 0, 0]);
    assert_eq!(interp[2], &clear);
}
