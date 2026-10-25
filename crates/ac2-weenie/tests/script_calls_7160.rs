//! The skill, shortcut, fellowship and allegiance calls against the Legions server double:
//! the client's calls go out byte for byte with their script objects (natives and package
//! instances in packed reference graphs) and decode on the server's side; a server call with a
//! package instance comes back as the raw script event and decodes the same way.

use std::time::Duration;

use ac2_net::protocol::{
    entry_flags, ntype, HashKey, HeapObject, NativeObject, PackValue, PackedEntry, PackedReference,
    ScriptCall, ScriptHash, ShortcutInfo,
};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent};
use legions_server_double::{addr, double, words, Spawn, PLAYER, SERVER};

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

fn chat_filter(pairs: &[(u64, u64)]) -> PackValue {
    PackValue::native(NativeObject::Hash(ScriptHash {
        native_type: ntype::AA_HASH,
        bucket_index: 0,
        keys: pairs.iter().map(|(k, _)| HashKey::Value(*k)).collect(),
        values: pairs.iter().map(|(_, v)| *v).collect(),
    }))
}

/// A `SkillInfo` instance (package 0x231, fifteen slots) whose slot 3 names a native
/// `StringInfo`: the shape of graph `WM_Skill` 0xD carries.
fn skill_info() -> PackedReference {
    let flags = entry_flags::LOADED | entry_flags::RECURSE | entry_flags::KIND_HEAP;
    let mut slots = vec![0u32; 15];
    let mut tags = vec![2u8; 15];
    slots[0] = 0x41;
    (slots[3], tags[3]) = (2, 1);
    PackedReference {
        ids: vec![1, 2],
        entries: vec![
            PackedEntry::Heap {
                flags,
                object: HeapObject {
                    pkg_state_id: 0,
                    pkg_id: 0x231,
                    native: None,
                    slots,
                    tags,
                },
            },
            PackedEntry::Heap {
                flags,
                object: HeapObject {
                    pkg_state_id: 0,
                    pkg_id: u32::MAX,
                    native: Some(NativeObject::WString("Sword".into())),
                    slots: vec![],
                    tags: vec![],
                },
            },
        ],
    }
}

#[test]
fn script_objects_cross_the_wire_byte_exact() {
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();

    let calls = vec![
        // `RequestTrainSkill`, `SetShortcut`, `DoFellowshipCreate`, `UploadChatOptions`.
        ScriptCall::new(0x8214_0002, vec![PackValue::Int(0x41)]).unwrap(),
        ScriptCall::new(
            0x8212_0018,
            vec![
                PackValue::native(NativeObject::ShortcutInfo(ShortcutInfo::Instance(PLAYER))),
                PackValue::Int(2),
            ],
        )
        .unwrap(),
        ScriptCall::new(
            0x820C_001B,
            vec![
                PackValue::Int(1),
                PackValue::Int(0),
                PackValue::native(NativeObject::WString("Wanderers".into())),
            ],
        )
        .unwrap(),
        ScriptCall::new(
            0x8212_002D,
            vec![
                chat_filter(&[(1, 0x80100)]),
                chat_filter(&[]),
                chat_filter(&[(4, 10)]),
                chat_filter(&[]),
                chat_filter(&[(4, 1)]),
            ],
        )
        .unwrap(),
    ];
    for call in &calls {
        session.send_script_call(call).unwrap();
    }
    // A server call cannot be sent.
    let server_call = ScriptCall::new(0x8214_0014, vec![PackValue::Int(0x41)]).unwrap();
    assert!(session.send_script_call(&server_call).is_err());

    // `WM_Skill` 0xD: a `SkillInfo` graph to the player.
    let skill = ScriptCall::new(0x8214_000D, vec![PackValue::Reference(skill_info())]).unwrap();
    state
        .borrow_mut()
        .send_script_call(skill.function, &skill.arguments);
    let events = settle(&mut session, &state);
    let received: Vec<ScriptCall> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Interp {
                object,
                function,
                arguments,
            } if *object == PLAYER => ScriptCall::decode(*function, arguments)?.ok(),
            _ => None,
        })
        .collect();
    // The skill, and the double's answer to `DoFellowshipCreate` (`WM_Fellowship` 0x2B, the
    // new fellowship).
    assert!(received.contains(&skill), "{received:?}");
    assert!(received.iter().any(|c| c.function == 0x820C_002B));
    // And the double's answer to `RequestTrainSkill` (the repository with the skill, `WM_Skill`
    // 0xE): the character has the credits for it.
    assert!(received.iter().any(|c| c.function == 0x8214_000E));
    assert_eq!(received.len(), 3);

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    // All of them, each once (a packet's blobs need not arrive in send order).
    let mut sent = calls.clone();
    let mut got = s.script_calls.clone();
    sent.sort_by_key(|c| c.function);
    got.sort_by_key(|c| c.function);
    assert_eq!(got, sent);
    // Byte for byte: `DoFellowshipCreate`'s two ints, then the name as a native `wpstring`.
    let create = s
        .blobs
        .iter()
        .find(|(_, q, d)| *q == 3 && d[4..8] == 0x820C_001Bu32.to_le_bytes())
        .map(|(_, _, d)| d.clone())
        .unwrap();
    let mut expected = Vec::new();
    words(&mut expected, &[0x0001_0053, 0x820C_001B, 64, 1, 1, 1, 0]);
    words(&mut expected, &[5, 1, 1, 0x4400_0100, 0, 0xFFFF_0076]);
    ac2_net::pack_wide_string(&mut expected, 0, "Wanderers");
    words(&mut expected, &[0]);
    assert_eq!(create, expected);
}
