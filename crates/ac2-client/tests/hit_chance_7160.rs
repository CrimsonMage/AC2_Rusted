//! Hit chance on name plates (a modern addition, AGENTS.md): the client keeps the hit chance of
//! the creature profiles it receives (the retail profile's "Chance to hit this target: " row),
//! the frame carries it while `ShowHitChanceOnNameplates` is on, and with
//! `ExamineNearbyForHitChance` on the client asks for a nearby creature's profile on its own: the
//! examination panel's object request, then a cleared one, with the selection, the player's
//! examination and the panel left alone.

mod common;

use std::sync::Arc;

use ac2_client::{ClientPrefs, Command, FrameInput};
use ac2_net::protocol::{
    ExaminationDataNode, ExaminationProfile, ExaminationRequest, ExaminationValue, ScriptRequest,
    StringInfo, StringVariable,
};
use ac2_prefs::Value;
use ac2_ui::hit_chance_pref::{EXAMINE_NEARBY, SHOW_HIT_CHANCE};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, BLOCKING_CREATURE, BLOCKING_CREATURE_VISUAL};

const FAR_CREATURE: u64 = 0x8000_0000_0000_0B01;

fn text(
    string_id: u32,
    appearance: u32,
    variables: Vec<(u32, StringVariable)>,
) -> ExaminationDataNode {
    ExaminationDataNode {
        order: 2,
        appearance,
        value: ExaminationValue::Text(StringInfo {
            string_id,
            table_id: 0x2500_0011,
            literal: None,
            variables,
        }),
    }
}

/// A creature's profile as the retail servers ended it: the hit chance row.
fn profile(percent: u32) -> ExaminationProfile {
    ExaminationProfile {
        request: ExaminationRequest::default(),
        nodes: vec![
            text(0x9E78_D75C, 0x33BE_4035, vec![]),
            text(
                0xD819_12CB,
                0xBE5B_6513,
                vec![(
                    0xF662_0CE8,
                    StringVariable::Word {
                        kind: 2,
                        value: percent,
                    },
                )],
            ),
            ExaminationDataNode {
                order: 2,
                appearance: 0,
                value: ExaminationValue::Break,
            },
        ],
        available: 0x203,
    }
}

fn prefs(show: bool, examine: bool) -> Option<ClientPrefs> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let dir = std::env::temp_dir().join(format!(
        "ac2-hit-chance-{}-{show}-{examine}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut prefs = ClientPrefs::load(dir.join("Preferences.ini"), store).unwrap();
    prefs.bank_mut().find_mut(SHOW_HIT_CHANCE).unwrap().value = Value::Bool(show);
    prefs.bank_mut().find_mut(EXAMINE_NEARBY).unwrap().value = Value::Bool(examine);
    Some(prefs)
}

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [60.0, 60.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// Frames with the client's own requests, gathering the events.
fn run(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    frames: usize,
) -> Vec<SessionEvent> {
    let mut events = Vec::new();
    for _ in 0..frames {
        events.extend(step(client, server, &FrameInput::default()));
        let frame = client.frame();
        client.examine_nearby_for_hit_chance(&frame);
    }
    events
}

#[test]
fn nearby_creatures_are_examined_quietly_and_their_plates_carry_the_hit_chance() {
    let Some(prefs) = prefs(true, true) else {
        return;
    };
    let (mut client, server) = entered(spawn());
    client.attach_prefs(prefs);
    {
        let mut server = server.borrow_mut();
        server
            .script_world
            .profiles
            .insert(BLOCKING_CREATURE, profile(64));
        server
            .script_world
            .profiles
            .insert(FAR_CREATURE, profile(30));
        // One 8 m ahead (north), one 40 m ahead: only the first is near enough.
        server.spawn_blocking_creature([60.0, 68.0, GROUND]);
        server.spawn_creature(
            FAR_CREATURE,
            BLOCKING_CREATURE_VISUAL,
            CELL,
            [60.0, 100.0, GROUND],
        );
    }
    let before = server.borrow().script_requests.len();
    let events = run(&mut client, &server, 96);
    let asked = server.borrow().script_requests[before..].to_vec();
    assert_eq!(
        asked,
        [
            ScriptRequest::Examine(ExaminationRequest::object(BLOCKING_CREATURE)),
            ScriptRequest::Examine(ExaminationRequest::default()),
        ],
        "one request for the near creature, then the cleared one"
    );
    let frame = client.frame();
    assert_eq!(frame.hit_chances.get(&BLOCKING_CREATURE), Some(&64));
    assert!(!frame.hit_chances.contains_key(&FAR_CREATURE));
    // Nothing reached the panel or the selection.
    assert!(frame.selection.is_none());
    assert!(!events.iter().any(|e| matches!(
        e,
        SessionEvent::Notice(
            GameplayNotice::UpdateExaminationProfile(_)
                | GameplayNotice::ExaminationRequestChanged(_)
                | GameplayNotice::OpenExaminationPanel
        )
    )));
    assert_eq!(
        client.session().player_script().examination_request(),
        ExaminationRequest::default()
    );
}

#[test]
fn the_hit_chance_is_kept_but_not_shown_or_asked_for_when_off() {
    let Some(prefs) = prefs(false, false) else {
        return;
    };
    let (mut client, server) = entered(spawn());
    client.attach_prefs(prefs);
    {
        let mut server = server.borrow_mut();
        server
            .script_world
            .profiles
            .insert(BLOCKING_CREATURE, profile(64));
        server.spawn_blocking_creature([60.0, 68.0, GROUND]);
    }
    let before = server.borrow().script_requests.len();
    run(&mut client, &server, 96);
    assert_eq!(
        server.borrow().script_requests.len(),
        before,
        "no request of its own"
    );
    // The player's own examination: the profile is shown as retail shows it, and kept.
    let open = FrameInput {
        commands: vec![
            Command::Examine(ExaminationRequest::object(BLOCKING_CREATURE)),
            Command::StartExamination(0x203),
        ],
        ..FrameInput::default()
    };
    let mut shown = step(&mut client, &server, &open);
    shown.extend(run(&mut client, &server, 8));
    assert!(shown.iter().any(|e| matches!(
        e,
        SessionEvent::Notice(GameplayNotice::UpdateExaminationProfile(_))
    )));
    assert_eq!(client.session().hit_chance(BLOCKING_CREATURE), Some(64));
    assert!(
        client.frame().hit_chances.is_empty(),
        "not shown with the switch off"
    );
}
