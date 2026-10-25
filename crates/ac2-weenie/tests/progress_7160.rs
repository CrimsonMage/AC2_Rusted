//! Skills, shortcuts and the social panels' state against the Legions server double: the
//! `WM_Skill` repository and skills, the login options' shortcut bar, a shortcut set byte for
//! byte, and fellowship notices kept for the panels.

use std::time::Duration;

use ac2_net::protocol::{
    entry_flags, ntype, GameplayOptionsProfile, HashKey, HeapObject, NativeObject, PackValue,
    PackedEntry, PackedReference, QualityUpdate, QualityValue, ScriptCall, ScriptHash,
    ShortcutInfo,
};
use ac2_net::Credential;
use ac2_weenie::{
    CharacterChoice, FellowInfo, FellowshipInfo, GameplayNotice, OnlineConfig, OnlineSession,
    SessionEvent,
};
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

type Double = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

fn settle(session: &mut OnlineSession, state: &Double) -> Vec<GameplayNotice> {
    let mut notices = Vec::new();
    for _ in 0..8 {
        for e in session.poll() {
            if let SessionEvent::Notice(n) = e {
                notices.push(n);
            }
        }
        state.borrow_mut().elapsed += 0.05;
    }
    notices
}

fn instance(pkg_id: u32, slots: Vec<u32>, tags: Vec<u8>) -> PackedEntry {
    PackedEntry::Heap {
        flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id,
            native: None,
            slots,
            tags,
        },
    }
}

/// A `SkillInfo` (package 0x231): enum in slot 14, flags in 7, experience in 5-6.
fn skill(skill: u32, flags: u32, xp: u32) -> PackedEntry {
    let mut slots = vec![0; 15];
    (slots[14], slots[7], slots[5]) = (skill, flags, xp);
    instance(0x231, slots, vec![2; 15])
}

#[test]
fn skills_shortcuts_and_fellowship_reach_the_panels() {
    let (transport, state) = double(Spawn::default());
    state.borrow_mut().script_world.options = Some(GameplayOptionsProfile {
        pack_bitfield: 0x2 | 0x4 | 0x1000,
        shortcuts: vec![ShortcutInfo::Empty(0); 3],
        which_shortcut_set: 1,
        shortcut_height: 2,
        ..GameplayOptionsProfile::default()
    });
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();

    // The login options: the shortcut bar.
    state.borrow_mut().send_init_player();
    let notices = settle(&mut session, &state);
    assert!(notices.contains(&GameplayNotice::ReloadOptions));
    let bar = &session.progress().shortcuts;
    assert_eq!((bar.set, bar.height, bar.slots.len()), (1, 2, 3));

    // `WM_Skill` 0xE: the repository (credits 3, hero credits 1, two skills).
    let mut repo_slots = vec![0u32; 8];
    let mut repo_tags = vec![2u8; 8];
    (repo_slots[0], repo_slots[3], repo_slots[5]) = (3, 1, 0x41);
    (repo_slots[7], repo_tags[7]) = (2, 1);
    let table = PackedEntry::Heap {
        flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id: u32::MAX,
            native: Some(NativeObject::Hash(ScriptHash {
                native_type: ntype::AR_HASH,
                bucket_index: 0,
                keys: vec![HashKey::Value(0x41), HashKey::Value(0x40)],
                values: vec![3, 4],
            })),
            slots: vec![],
            tags: vec![],
        },
    };
    let repository = PackedReference {
        ids: vec![1, 2, 3, 4],
        entries: vec![
            instance(0x233, repo_slots, repo_tags),
            table,
            skill(0x41, 1, 700),
            skill(0x40, 0, 0),
        ],
    };
    {
        let mut s = state.borrow_mut();
        s.send_quality(&QualityUpdate {
            object: None,
            quality: 301,
            value: QualityValue::LongInt(1000),
        });
        s.send_script_call(0x8214_000E, &[PackValue::Reference(repository)]);
    }
    let notices = settle(&mut session, &state);
    assert!(notices.contains(&GameplayNotice::UpdateSkillPanel {
        available_xp: 1000,
        skill_credits: 3,
        hero_skill_credits: 1
    }));
    let skills = &session.progress().skills;
    assert_eq!((skills.credits, skills.hero_credits), (3, 1));
    assert_eq!(
        skills
            .skills
            .iter()
            .map(|s| (s.skill, s.trained(), s.xp))
            .collect::<Vec<_>>(),
        [(0x40, false, 0), (0x41, true, 700)]
    );
    // The skill panel's queries over the repository.
    assert!(skills.is_trained(0x41) && !skills.is_trained(0x40) && !skills.is_trained(0x99));
    assert!(skills.is_being_untrained(0x41) && !skills.is_being_untrained(0x40));
    assert_eq!((skills.xp(0x41), skills.xp(0x99)), (700, 0));
    // `WM_Skill` 0xD: one skill trained.
    let one = PackedReference {
        ids: vec![1],
        entries: vec![skill(0x40, 1, 5)],
    };
    state
        .borrow_mut()
        .send_script_call(0x8214_000D, &[PackValue::Reference(one)]);
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::UpdateSkill(0x40)]
    );
    assert!(session.progress().skills.skills[0].trained());

    // A shortcut into slot 1; slot 3 is past the bar.
    let shortcut = ShortcutInfo::Instance(0x8000_0002_0000_0042);
    assert!(session.set_shortcut(1, shortcut.clone()));
    assert!(!session.set_shortcut(3, shortcut.clone()));
    settle(&mut session, &state);
    assert_eq!(session.progress().shortcuts.slots[1], shortcut);

    // Fellowship notices (raised by the script VM's glue) are kept for the panel.
    session.apply_notices(&[
        GameplayNotice::UpdateFellowship(FellowshipInfo {
            name: "Wanderers".into(),
            leader: PLAYER,
            fellows: vec![(PLAYER, FellowInfo::default())],
            ..FellowshipInfo::default()
        }),
        GameplayNotice::UpdateFellowLevel {
            fellow: PLAYER,
            level: 12,
        },
    ]);
    let fellowship = session.progress().social.fellowship.as_ref().unwrap();
    assert_eq!(fellowship.fellows[0].1.level, 12);

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    assert_eq!(
        s.script_calls,
        vec![ScriptCall::new(
            0x8212_0018,
            vec![
                PackValue::native(NativeObject::ShortcutInfo(shortcut)),
                PackValue::Int(1)
            ]
        )
        .unwrap()]
    );
    let blob = s
        .blobs
        .iter()
        .find(|(_, q, d)| *q == 3 && d[4..8] == 0x8212_0018u32.to_le_bytes())
        .map(|(_, _, d)| d.clone())
        .unwrap();
    let mut expected = Vec::new();
    words(
        &mut expected,
        &[0x0001_0053, 0x8212_0018, 48, 5, 1, 1, 0x4400_0100],
    );
    words(
        &mut expected,
        &[0, 0xFFFF_00AC, 0x4000_0002, 0x42, 0x8000_0002, 0, 1, 1],
    );
    assert_eq!(blob, expected);
}

#[test]
fn skill_requests_go_out_as_the_wm_skill_stubs() {
    use ac2_weenie::SkillRequest;
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    let requests = [
        SkillRequest::Train(0x41),
        SkillRequest::Raise(0x42),
        SkillRequest::Untrain(0x43),
        SkillRequest::CancelUntrain,
    ];
    for request in requests {
        session.request_skill(request);
    }
    settle(&mut session, &state);
    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    let calls: Vec<ScriptCall> = requests.iter().map(|r| r.call()).collect();
    for call in &calls {
        assert!(s.script_calls.contains(call), "{call:?}");
    }
    // Byte for byte: `InterpSEvent`, the function, the buffer (an int tag and the enum).
    let sent = |function: u32| {
        s.blobs
            .iter()
            .find(|(_, q, d)| *q == 3 && d[4..8] == function.to_le_bytes())
            .map(|(_, _, d)| d.clone())
            .unwrap()
    };
    for (function, skill) in [
        (0x8214_0002, 0x41),
        (0x8214_000C, 0x42),
        (0x8214_0007, 0x43),
    ] {
        let mut expected = Vec::new();
        words(&mut expected, &[0x0001_0053, function, 8, 1, skill]);
        assert_eq!(sent(function), expected);
    }
    let mut cancel = Vec::new();
    words(&mut cancel, &[0x0001_0053, 0x8214_0001, 0]);
    assert_eq!(sent(0x8214_0001), cancel);
}
