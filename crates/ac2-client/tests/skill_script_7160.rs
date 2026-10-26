//! Skill training, raising and experience spending against the Legions server double on the
//! script VM (`AC2_DAT_DIR`): the skill panel's requests leave as the `WM_Skill` stubs' bytes
//! (`RequestTrainSkill` 0x13, `RequestRaiseSkill` 0x10), the double spends the credits and XP its
//! rules set (`set_skill_cost`) and answers with the repository (0xE) and the skill (0xD), and the
//! panel's queries (the `GetSkillCredits` and kin IFuns) read the result.

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use ac2_client::{Client, Command, FrameInput, SkillLevel};
use ac2_core::DataId;
use ac2_net::protocol::{pack_values, PackValue, SkillMessage, SkillRecord, SkillRepository};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent, SkillRequest};
use common::*;
use legions_server_double::{SkillCost, Spawn, State, MELEE_ATTACK, MELEE_AUTO_ATTACK};

/// A skill the character may train: its training costs 2 credits and its first raise 7 XP (the
/// client's own queries, `GetSkillCostToTrainByEnum` and `GetSkillExperienceNeededToLevelByEnum`).
const SKILL: u32 = 0xF9;
const TRAIN_CREDITS: u32 = 2;
const RAISE_XP: u64 = 7;
/// The XP the server grants the character before the skills are spent.
const GRANTED_XP: u64 = 100;
/// The `WM_Skill` stubs' wire ids (`RequestTrainSkill`, `RequestRaiseSkill`).
const REQUEST_TRAIN: u32 = 0x8214_0002;
const REQUEST_RAISE: u32 = 0x8214_000C;

fn settle(client: &mut Client, server: &Rc<RefCell<State>>) -> Vec<GameplayNotice> {
    let mut events = Vec::new();
    for _ in 0..6 {
        events.extend(step(client, server, &FrameInput::default()));
    }
    notices(events)
}

fn notices(events: Vec<SessionEvent>) -> Vec<GameplayNotice> {
    events
        .into_iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n),
            _ => None,
        })
        .collect()
}

/// Runs `commands` for one frame, then settles; the notices they raised, in order.
fn run(
    client: &mut Client,
    server: &Rc<RefCell<State>>,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut events = step(client, server, &input);
    events.extend(settle_events(client, server));
    notices(events)
}

fn settle_events(client: &mut Client, server: &Rc<RefCell<State>>) -> Vec<SessionEvent> {
    let mut events = Vec::new();
    for _ in 0..6 {
        events.extend(step(client, server, &FrameInput::default()));
    }
    events
}

fn logged_in(owned: &[u32]) -> Option<(Client, Rc<RefCell<State>>)> {
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
    step(&mut client, &server, &FrameInput::default());
    server.borrow_mut().send_init_player();
    settle(&mut client, &server);
    Some((client, server))
}

/// The packed argument bytes the client sent for `function`, in order.
fn sent(server: &Rc<RefCell<State>>, function: u32) -> Vec<Vec<u8>> {
    server
        .borrow()
        .script_bytes
        .iter()
        .filter(|(f, _)| *f == function)
        .map(|(_, bytes)| bytes.clone())
        .collect()
}

fn record(skill: u32, flags: u32, xp: u64) -> SkillRecord {
    SkillRecord { skill, flags, xp }
}

#[test]
fn a_skill_is_trained_and_raised_through_the_player_script() {
    let Some((mut client, server)) = logged_in(ac2_client::SCRIPT_VM_FUNCTIONS) else {
        return;
    };
    server.borrow_mut().grant_xp(GRANTED_XP);
    settle(&mut client, &server);
    server.borrow_mut().set_skill_cost(
        SKILL,
        SkillCost {
            credits: TRAIN_CREDITS,
            xp: RAISE_XP,
        },
    );

    // Before: the character has its two skills and two credits; the panel's queries say what
    // training and raising take.
    assert_eq!(client.skill_credits(), Ok(2));
    assert_eq!(client.hero_skill_credits(), Ok(0));
    assert_eq!(client.skill_cost_to_train(SKILL), Ok(TRAIN_CREDITS));
    assert_eq!(client.skill_experience_to_level(SKILL), Ok(RAISE_XP));
    assert_eq!(client.can_skill_be_trained(SKILL), Ok(0));
    assert_eq!(client.is_skill_trained(SKILL), Ok(false));
    assert_eq!(client.skill_xp(SKILL), Ok(0));
    assert_eq!(client.can_skill_be_raised(SKILL), Ok(30009));

    // Train: the stub sends `WM_Skill` 0x13 with the skill, and the double answers with the
    // repository, the credits spent and the skill trained.
    let trained = run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Train(SKILL))],
    );
    assert_eq!(
        sent(&server, REQUEST_TRAIN),
        vec![pack_values(&[PackValue::Int(SKILL)])]
    );
    let repository = |credits, skills: Vec<SkillRecord>| {
        SkillMessage::SetRepository(SkillRepository {
            credits,
            hero_credits: 0,
            untraining: 0,
            skills,
        })
    };
    let melee = [record(MELEE_AUTO_ATTACK, 1, 0), record(MELEE_ATTACK, 1, 0)];
    let mut after_train = melee.to_vec();
    after_train.push(record(SKILL, 1, 0));
    assert_eq!(
        server.borrow().skill_answers,
        vec![
            repository(0, after_train.clone()),
            SkillMessage::UpdateSkill(record(SKILL, 1, 0)),
        ]
    );
    assert!(trained.contains(&GameplayNotice::UpdateSkillPanel {
        available_xp: GRANTED_XP,
        skill_credits: 0,
        hero_skill_credits: 0,
    }));
    // The panel reads the result: the credits, the training, and the raise's state (trained,
    // enough XP).
    assert_eq!(client.frame().progress.skills.credits, 0);
    assert_eq!(client.skill_credits(), Ok(0));
    assert_eq!(client.is_skill_trained(SKILL), Ok(true));
    assert_eq!(client.can_skill_be_raised(SKILL), Ok(0));
    assert_ne!(client.can_skill_be_trained(SKILL), Ok(0));

    // A second training is refused by the double (the skill is trained already, and the client
    // says so); the answers and the credits stay.
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Train(SKILL))],
    );
    assert_eq!(server.borrow().skill_answers.len(), 2);
    assert_eq!(client.skill_credits(), Ok(0));

    // Raise: `WM_Skill` 0x10 with the skill. The double spends the XP from `AvailableXP` into the
    // skill's experience, answering with the skill (0xD) and the quality update.
    let raised = run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Raise(SKILL))],
    );
    assert_eq!(
        sent(&server, REQUEST_RAISE),
        vec![pack_values(&[PackValue::Int(SKILL)])]
    );
    assert_eq!(
        server.borrow().skill_answers.last(),
        Some(&SkillMessage::UpdateSkill(record(SKILL, 1, RAISE_XP)))
    );
    let available = GRANTED_XP - RAISE_XP;
    assert!(raised.contains(&GameplayNotice::UpdateSkill(SKILL)));
    assert!(raised.contains(&GameplayNotice::UpdateSkillPanel {
        available_xp: available,
        skill_credits: 0,
        hero_skill_credits: 0,
    }));
    // The spent XP: the skill's experience and the available XP, both from the queries and the
    // qualities the panel reads; the total stays.
    assert_eq!(client.skill_xp(SKILL), Ok(RAISE_XP));
    assert_eq!(client.frame().progress.experience.available_xp, available);
    assert_eq!(client.frame().progress.experience.total_xp, GRANTED_XP);
    assert_eq!(client.skill_credits(), Ok(0));
    assert_eq!(client.is_skill_being_untrained(SKILL), Ok(false));
    assert!(client.is_skill_usable_by_inventory(SKILL).is_ok());
    assert_eq!(client.is_skill_harmful(SKILL), Ok(false));

    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}

#[test]
fn a_skill_request_is_refused_without_the_credits_or_the_xp() {
    let Some((mut client, server)) = logged_in(ac2_client::SCRIPT_VM_FUNCTIONS) else {
        return;
    };
    server.borrow_mut().set_skill_cost(
        SKILL,
        SkillCost {
            credits: TRAIN_CREDITS + 1,
            xp: RAISE_XP,
        },
    );
    // Not enough credits to train: the request goes out, the double answers nothing.
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Train(SKILL))],
    );
    assert!(server.borrow().skill_answers.is_empty());
    assert_eq!(client.skill_credits(), Ok(2));
    assert_eq!(client.is_skill_trained(SKILL), Ok(false));
    // Not trained: a raise is refused by the client's check (not trained) and the double.
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Raise(SKILL))],
    );
    assert!(server.borrow().skill_answers.is_empty());
    assert_eq!(sent(&server, REQUEST_RAISE).len(), 1);
    // The untrain requests go out as their stubs' calls: `WM_Skill` 0x12 with the skill, and the
    // cancel `WM_Skill` 0xF with no values.
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Untrain(SKILL))],
    );
    assert_eq!(
        sent(&server, 0x8214_0007),
        vec![pack_values(&[PackValue::Int(SKILL)])]
    );
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::CancelUntrain)],
    );
    assert_eq!(sent(&server, 0x8214_0001), vec![pack_values(&[])]);
}

/// `WM_Skill` 0x12 (the untrain stub) and 0xF (the cancel), answered by the repository (0xE, its
/// untraining slot names the skill being untrained, cleared by the cancel). The panel's queries
/// read the trained skill: `CanSkillBeUntrainedByEnum` gives the `gmWErrorType` why not
/// (`Skill_StartingSkill` 30008 for the trained skill, `Skill_NotUntrainable` 30007 for a starting
/// melee skill, `Skill_NotTrained` 30009 for a skill the character lacks).
#[test]
fn untrain_and_cancel_are_answered_with_the_repository() {
    let Some((mut client, server)) = logged_in(ac2_client::SCRIPT_VM_FUNCTIONS) else {
        return;
    };
    server.borrow_mut().grant_xp(GRANTED_XP);
    settle(&mut client, &server);
    server.borrow_mut().set_skill_cost(
        SKILL,
        SkillCost {
            credits: TRAIN_CREDITS,
            xp: RAISE_XP,
        },
    );
    assert_eq!(client.can_skill_be_untrained(SKILL), Ok(30009));
    assert_eq!(client.is_skill_being_untrained(SKILL), Ok(false));
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Train(SKILL))],
    );
    assert_eq!(client.is_skill_trained(SKILL), Ok(true));

    // The queries on the trained skill: its level and timer, the hero flag, and the inventory's
    // use. The wlib answers with the pack as it is (no weapon): the auto-attack is usable, the
    // weapon skills are not.
    assert_eq!(client.can_skill_be_untrained(SKILL), Ok(30008));
    assert_eq!(client.can_skill_be_untrained(MELEE_ATTACK), Ok(30007));
    assert_eq!(client.can_skill_be_untrained(0x41), Ok(30009));
    assert_eq!(
        client.skill_level(SKILL),
        Ok(SkillLevel {
            raw: 0,
            enchanted: 0,
            ok: true,
        })
    );
    let time = client.skill_time_left(SKILL).expect("the timer query");
    assert!(!time.ok);
    assert_eq!((time.total, time.left), (0.0, 0.0));
    assert_eq!(client.is_hero_skill(SKILL), Ok(false));
    assert_eq!(
        client.is_skill_usable_by_inventory(MELEE_AUTO_ATTACK),
        Ok(true)
    );
    assert_eq!(client.is_skill_usable_by_inventory(MELEE_ATTACK), Ok(false));
    assert_eq!(client.is_skill_usable_by_inventory(SKILL), Ok(false));

    // Untrain: `WM_Skill` 0x12 with the skill; the repository answer names it as untraining.
    let repository = |untraining, skills: Vec<SkillRecord>| {
        SkillMessage::SetRepository(SkillRepository {
            credits: 0,
            hero_credits: 0,
            untraining,
            skills,
        })
    };
    let mut after_train = vec![record(MELEE_AUTO_ATTACK, 1, 0), record(MELEE_ATTACK, 1, 0)];
    after_train.push(record(SKILL, 1, 0));
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Untrain(SKILL))],
    );
    assert_eq!(
        sent(&server, 0x8214_0007),
        vec![pack_values(&[PackValue::Int(SKILL)])]
    );
    assert_eq!(
        server.borrow().skill_answers.last(),
        Some(&repository(SKILL, after_train.clone()))
    );
    assert_eq!(client.is_skill_being_untrained(SKILL), Ok(true));
    // A second untrain while one is under way is refused by the double: no further answer.
    let answers = server.borrow().skill_answers.len();
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::Untrain(SKILL))],
    );
    assert_eq!(server.borrow().skill_answers.len(), answers);

    // Cancel: `WM_Skill` 0xF with no values; the repository answer clears the untraining slot.
    run(
        &mut client,
        &server,
        vec![Command::Skill(SkillRequest::CancelUntrain)],
    );
    assert_eq!(sent(&server, 0x8214_0001), vec![pack_values(&[])]);
    assert_eq!(
        server.borrow().skill_answers.last(),
        Some(&repository(0, after_train))
    );
    assert_eq!(client.is_skill_being_untrained(SKILL), Ok(false));
    assert_eq!(client.is_skill_trained(SKILL), Ok(true));
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}
