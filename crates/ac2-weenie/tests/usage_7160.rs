//! Using objects against the Legions server double: the `WM_Usage` 0x22 stub's `UsageDesc`
//! goes out byte for byte, and the double's `WM_Usage` 0x21 answer comes back as the notices
//! `CUsageSystem` 0xA raises.

use std::time::Duration;

use ac2_net::protocol::{ScriptRequest, StringInfo, UsageDesc};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent};
use legions_server_double::{addr, double, words, Spawn, PLAYER, SERVER};

const ITEM: u64 = 0x8000_0002_0000_0042;
const TARGET: u64 = 0x8000_0002_0000_0043;

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
fn uses_go_out_as_usage_descs_and_their_outcomes_come_back() {
    let (transport, state) = double(Spawn::default());
    let message = StringInfo {
        literal: Some("You drink the potion.".into()),
        ..StringInfo::default()
    };
    state.borrow_mut().script_world.usage_message = Some(message.clone());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();

    // Nothing selected: `CPlayer` 0x79 says so and sends nothing.
    let events = session.use_selection();
    assert!(matches!(
        &notices(&events)[..],
        [GameplayNotice::DisplayStringInfo { kind: 8, text }] if text.string_id == 0xAE62_433B
    ));
    // Using the player itself sends nothing either.
    session.use_object(PLAYER);

    // A use: the outcome's message shows as a usage line.
    session.use_object(ITEM);
    let events = settle(&mut session, &state);
    assert_eq!(
        notices(&events),
        vec![GameplayNotice::DisplayStringInfo {
            kind: 8,
            text: message
        }]
    );
    // A use on a target that fails.
    state.borrow_mut().script_world.usage_error = 0x2711;
    session.use_with_target(ITEM, TARGET);
    let events = settle(&mut session, &state);
    assert_eq!(
        notices(&events),
        vec![GameplayNotice::UsageError {
            error: 0x2711,
            item: ITEM,
            target: TARGET
        }]
    );

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    assert_eq!(
        s.script_requests,
        vec![
            ScriptRequest::Use(UsageDesc::new(PLAYER, ITEM, 0)),
            ScriptRequest::Use(UsageDesc::new(PLAYER, ITEM, TARGET)),
        ]
    );
    // Byte for byte: `InterpSEvent`, the function, the `PackBuffer` size, then the reference
    // graph: the `UsageDesc` instance (package 0x245, twenty slots) and its message.
    let uses: Vec<&Vec<u8>> = s
        .blobs
        .iter()
        .filter(|(_, queue, data)| *queue == 3 && data[..8] == [0x53, 0, 1, 0, 0x16, 0, 0x16, 0x82])
        .map(|(_, _, data)| data)
        .collect();
    assert_eq!(uses.len(), 2);
    let mut expected = Vec::new();
    words(&mut expected, &[0x0001_0053, 0x8216_0016, 160]);
    words(
        &mut expected,
        &[5, 2, 1, 2, 0x4400_0100, 0, 0x0245_0000, 20],
    );
    words(
        &mut expected,
        &[
            2,
            u32::MAX,
            0x42,
            0x8000_0002,
            u32::MAX,
            0,
            1,
            0x5000_0000,
            0xBF80_0000,
            0x43,
            0x8000_0002,
            0,
            0,
            0,
            0,
            u32::MAX,
            0,
            0x13C,
            0,
            0,
        ],
    );
    expected.extend([
        1, 1, 0x82, 0xCD, 1, 2, 0x82, 0xCD, 3, 0x82, 0xCD, 2, 0x82, 0xCD, 2, 1, 2, 2, 2, 2,
    ]);
    // The message: a loaded native StringInfo, 0x4E1AEC40 of table 0x25000014, no variables,
    // no literal; no slots.
    words(
        &mut expected,
        &[0x4400_0100, 0, 0xFFFF_006A, 0x4E1A_EC40, 0x2500_0014, 0, 0],
    );
    assert_eq!(uses[1], &expected);
}
