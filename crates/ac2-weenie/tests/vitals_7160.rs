//! The player's vitals against the Legions server double: quality updates and `WM_Player` 0x27
//! reach `CPlayer` (0x85, 0x65), whose vital notices are what `gmVitalsUI` shows.

use std::time::Duration;

use ac2_net::protocol::{QualityUpdate, QualityValue};
use ac2_net::Credential;
use ac2_weenie::{
    vital, CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent, Vital,
};
use legions_server_double::{addr, double, words, Spawn, SERVER};

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

fn int(quality: u32, value: i32) -> QualityUpdate {
    QualityUpdate {
        object: None,
        quality,
        value: QualityValue::Int(value),
    }
}

fn update(vital: u32, current: u64, max: u64) -> GameplayNotice {
    GameplayNotice::UpdateVital {
        vital,
        current,
        max,
    }
}

#[test]
fn vitals_follow_the_login_message_and_quality_updates() {
    let (transport, state) = double(Spawn::default());
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    assert_eq!(session.player_script().vitals().health, None);

    // The player's vitals arrive as private int updates and the vitae float; each raises the
    // vital it feeds (`CPlayer` 0x85).
    {
        let mut s = state.borrow_mut();
        for update in [
            int(0x107, 250),
            int(0x102, 200),
            int(0x108, 120),
            int(0x103, 90),
        ] {
            s.send_quality(&update);
        }
        s.send_quality(&QualityUpdate {
            object: None,
            quality: 0x11E,
            value: QualityValue::Float(95.0),
        });
    }
    let events = settle(&mut session, &state);
    let raised = notices(&events);
    assert_eq!(raised.len(), 2 + 2 + 2 + 2 + 4, "{raised:?}");
    // After the vitae: 250 * 0.95 -> 237, 120 * 0.95 -> 114.
    assert_eq!(
        raised[8..],
        [
            update(vital::HEALTH, 200, 237),
            update(vital::PK_DAMAGE, 0, 237),
            update(vital::VIGOR, 90, 114),
            update(vital::PK_VIGOR_LOSS, 0, 114),
        ]
    );

    // `WM_Player` 0x27 (`CPlayer` 0x65): the skill repository's panel, the inventory
    // arguments (the player's contents, the equipment), the options profile (the double always
    // sends one), then the hero status and every vital again.
    state.borrow_mut().send_init_player();
    let events = settle(&mut session, &state);
    let panel = match notices(&events).first() {
        Some(
            n @ GameplayNotice::UpdateSkillPanel {
                skill_credits: 2, ..
            },
        ) => n.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        notices(&events),
        vec![
            panel,
            GameplayNotice::UpdateContainer {
                container: legions_server_double::PLAYER,
                code: ac2_weenie::INVENTORY_REFRESH,
                item: 0,
                slot: 0
            },
            GameplayNotice::UpdateInventory {
                code: ac2_weenie::INVENTORY_REFRESH,
                item: 0,
                location: 0
            },
            GameplayNotice::ReloadOptions,
            GameplayNotice::UpdateHeroStatus {
                hero: false,
                current: 0,
                max: 100
            },
            update(vital::HEALTH, 200, 237),
            update(vital::PK_DAMAGE, 0, 237),
            update(vital::VIGOR, 90, 114),
            update(vital::PK_VIGOR_LOSS, 0, 114),
            update(vital::FOCUS, 0, 100),
        ]
    );
    // Its other arguments are left to the caller as the raw script event.
    assert!(events.iter().any(|e| matches!(
        e,
        SessionEvent::Interp {
            function: 0x8212_0027,
            ..
        }
    )));

    // Damage, then the PK damage shown.
    state.borrow_mut().send_quality(&int(0x102, 150));
    state.borrow_mut().send_quality(&int(0x10D, 30));
    settle(&mut session, &state);
    let events = session.set_show_pk_damage(true);
    assert_eq!(
        notices(&events)[..2],
        [
            update(vital::HEALTH, 150, 237),
            update(vital::PK_DAMAGE, 30, 237)
        ]
    );
    let vitals = session.player_script().vitals();
    assert_eq!(
        vitals.health,
        Some(Vital {
            current: 150,
            max: 237
        })
    );
    assert_eq!(
        vitals.vigor,
        Some(Vital {
            current: 90,
            max: 114
        })
    );
    assert_eq!(vitals.experience, None);

    // Byte for byte: the private int update is its opcode, the quality, the value.
    let mut expected = Vec::new();
    words(&mut expected, &[0x0001_0069, 0x102, 150]);
    assert_eq!(int(0x102, 150).encode(), expected);
    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
}
