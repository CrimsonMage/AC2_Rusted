//! Combat with the player's script on the VM (`AC2_DAT_DIR`). A skill used from the shortcut bar
//! against the selected creature sends the special attack request as the VM packs it, and the
//! double's answers reach the bar's notices. An attack exchanges blows on the timer: the player's
//! hit and the creature's blow back, each a physics story whose damage hook runs
//! `gmCWInterface::DoUIDamage` (IFun 0xB400C0) on the VM, which raises the chat line and the
//! floating damage text. Vitals and the selection panel's health follow the blows, and a creature
//! at zero health leaves the world. A special attack's start has no server forwarder, so the bar
//! never sees one here (see `ac2_net::protocol::CombatMessage`).

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{CombatMessage, ScriptRequest, SpecialAttackRequest};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{
    lifestone, Spawn, CORPSE, CORPSE_ITEM, CORPSE_SKIN, CREATURE_ONE, MELEE_AUTO_ATTACK, PLAYER,
};

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// Seconds between a fight's exchanges (`legions_server_double::combat::HIT_INTERVAL`).
const HIT_INTERVAL: f64 = 2.0;
/// The creature's and the player's health the double starts them at.
const CREATURE_HEALTH: u32 = 30;
const PLAYER_HEALTH: u32 = 40;
/// The blows: the player's and the creature's.
const PLAYER_DAMAGE: u32 = 10;
const CREATURE_DAMAGE: u32 = 3;
/// `Health` (the vital's code in `GameplayNotice::UpdateVital`).
const HEALTH_VITAL: u32 = 0x4000_0001;

/// The script with the player attached, the double's shortcut bar on the VM (slot 0 holds Melee
/// Auto Attack, a skill; slot 1 "/e waves"), the double's two creatures spawned with their
/// scripts, and the fight started at full health.
fn with_script() -> Option<(ac2_client::Client, Shared)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset.clone(), store.clone());
    data.preload_tables();
    // As the binary does: the string tables and the chat command line.
    let names = DatScriptData::new(dataset, store);
    let (mut client, server) = entered(spawn());
    client
        .session_mut()
        .set_string_tables(ac2_client::string_table_ids(&names));
    client
        .session_mut()
        .set_chat_commands(ac2_client::chat_commands(&names));
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(PLAYER);
        // These tests count the blows' health exactly: no regeneration between them.
        state.vitals.regen_paused = true;
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    server.borrow_mut().send_init_player();
    server.borrow_mut().spawn_radar_objects();
    server.borrow_mut().start_fight();
    // The creatures' scripts start on their loads; the fight's messages go out on the way.
    advance(&mut client, &server, 1.0);
    Some((client, server))
}

/// Runs `secs` seconds of frames, the commands on the first; every event the frames raised.
fn events_for(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
    frames: usize,
) -> Vec<SessionEvent> {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut events = Vec::new();
    for i in 0..frames {
        let frame = if i == 0 { &input } else { &idle };
        events.extend(step(client, server, frame));
    }
    events
}

/// Runs the commands for ten frames; the notices the frames raised, in order.
fn run(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    notices(events_for(client, server, commands, 10))
}

/// The frames of `secs` seconds (the double's clock moves a step a frame).
fn advance(client: &mut ac2_client::Client, server: &Shared, secs: f64) -> Vec<SessionEvent> {
    let frames = (secs * 64.0).round() as usize;
    events_for(client, server, Vec::new(), frames)
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

/// The player selects the creature (the target of a special attack).
fn select_creature(client: &mut ac2_client::Client, server: &Shared) {
    run(client, server, vec![Command::Select(Some(CREATURE_ONE))]);
}

/// The combat notices (the bar's requests, the attack state, the ends and the toggles).
fn combat_notices(notices: &[GameplayNotice]) -> Vec<GameplayNotice> {
    notices
        .iter()
        .filter(|n| {
            matches!(
                n,
                GameplayNotice::RequestSpecialAttack { .. }
                    | GameplayNotice::UpdateAttackState(_)
                    | GameplayNotice::EndSpecialAttack(_)
                    | GameplayNotice::ToggleSpecialAttack { .. }
            )
        })
        .cloned()
        .collect()
}

/// The damage text's objects, in order (`Do3DDamage`).
fn damage_texts(events: &[SessionEvent]) -> Vec<u64> {
    events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::Do3DDamage { object, .. }) => Some(*object),
            _ => None,
        })
        .collect()
}

/// The `UpdateVital` health currents, in order.
fn health_currents(events: &[SessionEvent]) -> Vec<u64> {
    let mut currents: Vec<u64> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::UpdateVital { vital, current, .. })
                if *vital == HEALTH_VITAL =>
            {
                Some(*current)
            }
            _ => None,
        })
        .collect();
    // The current and the max each raise a notice on a change of health; one value per blow.
    currents.dedup();
    currents
}

#[test]
fn a_skill_from_the_shortcut_bar_requests_a_special_attack_with_its_bytes() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    let before = server.borrow().script_bytes.len();
    assert_eq!(client.use_shortcut(0), Ok(1));
    let notices = run(&mut client, &server, Vec::new());
    let sent = server.borrow().script_bytes[before..].to_vec();
    // The VM's `WM_Combat` 0x17 call: the attack (1), the maneuver (the skill, 6), the target (the
    // selection), each packed as a PackBuffer tag then value (ints tagged 1, the long tagged 2).
    let expected: Vec<u8> = [1u32, 1, 1, 6, 2]
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .chain(
            [0x0A01u32, 0x8000_0000]
                .iter()
                .flat_map(|w| w.to_le_bytes()),
        )
        .collect();
    assert_eq!(sent.len(), 1, "one request: {sent:?}");
    assert_eq!(sent[0].0, 0x8204_000B);
    assert_eq!(sent[0].1, expected);
    // The double reads the request back as the codec's layout.
    let request = server
        .borrow()
        .script_calls
        .iter()
        .find_map(SpecialAttackRequest::from_call);
    assert_eq!(
        request,
        Some(SpecialAttackRequest {
            attack: 1,
            maneuver: MELEE_AUTO_ATTACK,
            target: CREATURE_ONE,
        })
    );
    // The bar's maneuver goes pending at the request; the double's answers follow: the attack
    // state up, then the attack ends.
    assert_eq!(
        combat_notices(&notices),
        vec![
            GameplayNotice::RequestSpecialAttack {
                maneuver: MELEE_AUTO_ATTACK,
                attack: 1,
            },
            GameplayNotice::UpdateAttackState(true),
            GameplayNotice::EndSpecialAttack(1),
        ]
    );
    assert!(client.take_script_problems().is_empty());
}

/// With nothing selected the VM still sends the request, with no target (the zero long int); the
/// double refuses a target it does not know, so the bar only sees its own pending maneuver.
#[test]
fn a_skill_without_a_selection_requests_no_target_and_is_not_answered() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let before = server.borrow().script_bytes.len();
    assert_eq!(client.use_shortcut(0), Ok(1));
    let notices = run(&mut client, &server, Vec::new());
    let sent = server.borrow().script_bytes[before..].to_vec();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, 0x8204_000B);
    let request = server
        .borrow()
        .script_calls
        .iter()
        .find_map(SpecialAttackRequest::from_call);
    assert_eq!(request.map(|r| r.target), Some(0));
    assert_eq!(
        combat_notices(&notices),
        vec![GameplayNotice::RequestSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            attack: 1,
        }]
    );
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn a_maneuver_toggle_reaches_the_bar() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    server
        .borrow_mut()
        .send_combat(&CombatMessage::ToggleSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            toggled: true,
        });
    let notices = run(&mut client, &server, Vec::new());
    assert_eq!(
        notices,
        vec![GameplayNotice::ToggleSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            toggled: true,
        }]
    );
    server
        .borrow_mut()
        .send_combat(&CombatMessage::ToggleSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            toggled: false,
        });
    let notices = run(&mut client, &server, Vec::new());
    assert_eq!(
        notices,
        vec![GameplayNotice::ToggleSpecialAttack {
            maneuver: MELEE_AUTO_ATTACK,
            toggled: false,
        }]
    );
    assert!(client.take_script_problems().is_empty());
}

/// A text shortcut (`/e waves`, slot 1) runs its line through the chat shell, which asks the
/// server for the emote (efun 0x7F0052 `ProcessText` -> the shell's `DoEmote`).
#[test]
fn a_text_shortcut_runs_its_line_through_the_chat_shell() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let before = server.borrow().script_requests.len();
    assert!(client.use_shortcut(1).is_ok());
    run(&mut client, &server, Vec::new());
    let requests: Vec<ScriptRequest> = server.borrow().script_requests[before..].to_vec();
    assert!(
        requests
            .iter()
            .any(|r| matches!(r, ScriptRequest::EmoteText(text) if text == "waves")),
        "{requests:?}"
    );
    assert!(client.take_script_problems().is_empty());
}

/// The attack on the creature: the first exchange (after the interval) is the player's blow and
/// the creature's blow back. Each blow is raised both ways: the chat line and the floating damage
/// text over the damaged object, the player's own (the creature) and the creature's (the player).
#[test]
fn damage_is_noticed_both_ways() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    let started = events_for(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    assert!(combat_notices(&notices(started)).is_empty());
    let events = advance(&mut client, &server, HIT_INTERVAL + 0.5);
    // The player's blow damages the creature (its text over the creature), the creature's blow
    // back damages the player (its text over the player).
    assert_eq!(
        damage_texts(&events),
        vec![CREATURE_ONE, PLAYER],
        "{events:?}"
    );
    let lines = notices(events)
        .iter()
        .filter(|n| matches!(n, GameplayNotice::DisplayStringInfo { .. }))
        .count();
    assert!(lines >= 2, "the chat line for each blow: {lines}");
    assert!(client.take_script_problems().is_empty());
}

/// The player's blows bring the creature's health down on the selection panel, and the creature's
/// blows bring the player's health down through the vitals (`CPlayer` 0x85, `UpdateVital`).
#[test]
fn vitals_and_the_selection_health_drop_with_the_blows() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    events_for(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    let events = advance(&mut client, &server, HIT_INTERVAL + 0.5);
    assert_eq!(
        health_currents(&events),
        vec![u64::from(PLAYER_HEALTH - CREATURE_DAMAGE)],
        "{events:?}"
    );
    let creature = client
        .frame()
        .selection
        .as_ref()
        .and_then(|s| s.info.as_ref())
        .map(|info| (info.cur_health, info.max_health));
    assert_eq!(
        creature,
        Some((CREATURE_HEALTH - PLAYER_DAMAGE, CREATURE_HEALTH))
    );
    assert!(client.take_script_problems().is_empty());
}

/// Three exchanges kill the creature (30 health, 10 a blow): it leaves the world, the attack
/// ends, and the creature's last blow is not made. It is destroyed after its out-of-view lifetime
/// (`OUT_OF_VIEW_LIFETIME`, 25 s), as an object that leaves the world is.
#[test]
fn the_creature_dies_at_zero_health() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    events_for(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    let fight = advance(&mut client, &server, 3.0 * HIT_INTERVAL + 0.5);
    assert!(!fight
        .iter()
        .any(|e| matches!(e, SessionEvent::ObjectRemoved(_))));
    let all = notices(fight.clone());
    assert!(combat_notices(&all).ends_with(&[GameplayNotice::UpdateAttackState(false)]));
    // The creatures' blows: two made (the third exchange kills before the creature strikes).
    assert_eq!(
        health_currents(&fight),
        vec![
            u64::from(PLAYER_HEALTH - CREATURE_DAMAGE),
            u64::from(PLAYER_HEALTH - 2 * CREATURE_DAMAGE),
        ]
    );
    assert!(server.borrow().fight.dead.contains(&CREATURE_ONE));
    // The creature's death plays out, it leaves the world, and the object goes once its
    // lifetime runs out.
    let later = advance(
        &mut client,
        &server,
        26.0 + legions_server_double::DEATH_SECONDS,
    );
    assert!(later
        .iter()
        .any(|e| matches!(e, SessionEvent::ObjectRemoved(id) if *id == CREATURE_ONE)));
    assert!(client.frame().objects.iter().all(|o| o.id != CREATURE_ONE));
    assert!(client.take_script_problems().is_empty());
}

/// The first blow's text, over the creature, in the blow's own notice: the damage text's object is
/// the creature the player hit.
#[test]
fn a_special_attack_hit_puts_its_text_over_the_creature() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    let events = events_for(&mut client, &server, Vec::new(), 1);
    assert!(damage_texts(&events).is_empty());
    assert_eq!(client.use_shortcut(0), Ok(1));
    let events = events_for(&mut client, &server, Vec::new(), 10);
    assert_eq!(damage_texts(&events), vec![CREATURE_ONE], "{events:?}");
    assert!(client.take_script_problems().is_empty());
}

/// The `death` behaviour, the `dead` mode and the default mode the bodies return to.
const DEATH_BEHAVIOR: u32 = 0x4100_001F;
const DEAD_MODE: u32 = 0x4100_0005;
const PEACE_MODE: u32 = 0x4000_0001;

/// The (behaviour, mode) of each `DoBehavior` event `object` received.
fn do_behaviors(events: &[SessionEvent], object: u64) -> Vec<(u32, u32)> {
    events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::ObjectAction {
                object: id,
                action: ac2_net::protocol::ObjectAction::DoBehavior(p),
            } if *id == object => Some((p.behavior, p.mode)),
            _ => None,
        })
        .collect()
}

/// The packed `UpdateDeathState` flags, in order.
fn death_flags(events: &[SessionEvent]) -> Vec<bool> {
    events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::UpdateDeathState(dead)) => Some(*dead),
            _ => None,
        })
        .collect()
}

/// The creature dies (three exchanges), leaves the world, and leaves its corpse where it stood:
/// the corpse opens, lists its trophy skin and its item, and Loot All takes them into the pack.
#[test]
fn the_creature_dies_and_its_corpse_is_looted() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    events_for(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    // Three exchanges, then the death plays out before the creature leaves for its corpse.
    let fight = advance(
        &mut client,
        &server,
        3.0 * HIT_INTERVAL + 0.5 + legions_server_double::DEATH_SECONDS,
    );
    assert!(server.borrow().fight.dead.contains(&CREATURE_ONE));
    assert!(fight.iter().any(|e| matches!(
        e,
        SessionEvent::ObjectAction {
            object,
            action: ac2_net::protocol::ObjectAction::LeaveWorld(_),
        } if *object == CREATURE_ONE
    )));
    // The double sets the creature's Dead bool (quality 0x117) before it leaves; the client's
    // quality update carries it to the creature's script.
    assert!(fight.iter().any(|e| matches!(
        e,
        SessionEvent::QualityChanged { object, table: 1, quality: 0x117 } if *object == CREATURE_ONE
    )), "{fight:?}");
    // And its death presentation: `DoBehavior` with the `death` behaviour and the `dead` mode
    // (`gmbehaviors.emp` 0x4100001F, `gmmodes.emp` 0x41000005), before the creature leaves.
    let behaviors = do_behaviors(&fight, CREATURE_ONE);
    assert_eq!(behaviors, vec![(DEATH_BEHAVIOR, DEAD_MODE)], "{fight:?}");
    // The corpse is placed at the creature's position (its cell, where the creature died).
    let corpse = client
        .frame()
        .objects
        .iter()
        .find(|o| o.id == CORPSE)
        .map(|o| (o.cell, o.origin));
    let creature_origin = legions_server_double::radar_objects(&spawn())
        .into_iter()
        .find(|(o, _)| o.id == CREATURE_ONE)
        .map(|(o, _)| (o.cell, o.origin));
    assert_eq!(
        corpse, creature_origin,
        "the corpse lies where the creature died"
    );

    // Open the corpse: it lists its skin and item.
    run(
        &mut client,
        &server,
        vec![Command::SetContainerOpen {
            container: CORPSE,
            open: true,
        }],
    );
    let listed = client
        .frame()
        .inventory
        .containers
        .iter()
        .find(|c| c.id == CORPSE)
        .map(|c| c.items.to_vec());
    assert_eq!(listed, Some(vec![CORPSE_SKIN, CORPSE_ITEM]));
    // Loot All: both come into the pack.
    run(&mut client, &server, vec![Command::LootAll(CORPSE)]);
    let pack = client.frame().inventory.pack.items.clone();
    assert!(
        pack.contains(&CORPSE_SKIN) && pack.contains(&CORPSE_ITEM),
        "{pack:?}"
    );
    assert!(client.take_script_problems().is_empty());
}

/// The player dies under the creature's blows (the creature is made stronger than the player's
/// blows, so the player falls first): the death flag goes up (`UpdateDeathState`), the attack ends,
/// and the respawn request (`RequestResurrect`) puts the player at the lifestone with full health
/// and the flag down.
#[test]
fn the_player_dies_and_respawns_at_a_lifestone() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    // A creature that outlasts the player's blows: 40 health for the player's 40 to go.
    server
        .borrow_mut()
        .fight
        .creatures
        .insert(CREATURE_ONE, (1000, 1000));
    select_creature(&mut client, &server);
    events_for(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    // Fourteen exchanges: the creature's 3 a blow takes the player's 40 to zero.
    let mut events = advance(&mut client, &server, 14.0 * HIT_INTERVAL + 0.5);
    assert_eq!(death_flags(&events), vec![true], "{events:?}");
    assert!(
        events.iter().any(|e| matches!(
            e,
            SessionEvent::QualityChanged { object, table: 1, quality: 0x117 } if *object == PLAYER
        )),
        "{events:?}"
    );
    assert_eq!(
        do_behaviors(&events, PLAYER),
        vec![(DEATH_BEHAVIOR, DEAD_MODE)]
    );
    assert!(client.frame().progress.dead);
    assert_eq!(health_currents(&events).last(), Some(&0));
    assert!(combat_notices(&notices(events.clone()))
        .ends_with(&[GameplayNotice::UpdateAttackState(false)]));

    // The respawn request: the lifestone teleport, the flag down, full health.
    events_for(&mut client, &server, vec![Command::RequestResurrect], 1);
    for _ in 0..40 {
        let more = advance(&mut client, &server, 0.5);
        events.extend(more);
        let frame = client.frame();
        if !frame.progress.dead && frame.in_world && !frame.portal.teleporting {
            break;
        }
    }
    assert_eq!(death_flags(&events), vec![true, false]);
    // The respawn puts the body back in the default mode.
    assert!(events.iter().any(|e| matches!(
        e,
        SessionEvent::ObjectAction { object, action: ac2_net::protocol::ObjectAction::SetMode(PEACE_MODE) } if *object == PLAYER
    )));
    // The respawn's alive branch (`CDeathSystem` 0x3 -> `DeathSystem` 0x7) runs on the VM and
    // shows its two usage messages.
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(
                e,
                SessionEvent::Notice(GameplayNotice::DisplayStringInfo { kind: 8, .. })
            ))
            .count(),
        2,
        "{events:?}"
    );
    assert!(!client.frame().progress.dead);
    assert_eq!(
        health_currents(&events).last(),
        Some(&u64::from(PLAYER_HEALTH))
    );
    let frame = client.frame();
    let player = frame.player.as_ref().expect("the player's body");
    let [x, y, _] = player.origin;
    let [lx, ly, _] = lifestone(&spawn());
    assert!(
        (x - lx).abs() < 1e-3 && (y - ly).abs() < 1e-3,
        "{:?}",
        player.origin
    );
    let problems = client.take_script_problems();
    assert!(problems.is_empty(), "{problems:?}");
}

/// `gmCWInterface::StopAttacking(target)` (IFun 0xB40104) runs on the VM, which sends the script's
/// `WM_Combat` 0x25 with the target (a long int). The double stops the attack on that target: the
/// attack state goes down.
#[test]
fn stop_attacking_runs_on_the_vm_for_the_target() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    events_for(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    advance(&mut client, &server, 1.0);
    let before = server.borrow().script_bytes.len();
    let mut events = events_for(
        &mut client,
        &server,
        vec![Command::StopAttacking(CREATURE_ONE)],
        1,
    );
    events.extend(advance(&mut client, &server, 0.5));
    let sent = server.borrow().script_bytes[before..].to_vec();
    // The target packed as the script packs it: the long tag (2), then the id's two words.
    let target: Vec<u8> = [2u32]
        .iter()
        .chain(&[CREATURE_ONE as u32, (CREATURE_ONE >> 32) as u32])
        .flat_map(|w| w.to_le_bytes())
        .collect();
    assert_eq!(sent, vec![(0x8204_0025, target)]);
    assert_eq!(
        combat_notices(&notices(events)),
        vec![GameplayNotice::UpdateAttackState(false)]
    );
    assert!(client.take_script_problems().is_empty());
}

/// `PlayerMovementModule::StopAttackingIfAutoMoving` -> `gmCWInterface::StopAttackingIfAutoMoving`
/// (IFun 0xB400C3) runs on the VM with no arguments: the script decides whether the auto-move
/// attack stops, so the call must reach the VM and finish without a problem.
#[test]
fn stop_attacking_if_auto_moving_runs_on_the_vm() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    select_creature(&mut client, &server);
    events_for(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    advance(&mut client, &server, 1.0);
    events_for(
        &mut client,
        &server,
        vec![Command::StopAttackingIfAutoMoving],
        1,
    );
    advance(&mut client, &server, 0.5);
    assert!(client.take_script_problems().is_empty());
}

/// `gmCWInterface::Dismount` (IFun 0xB400E5) runs on the VM, which sends the script's call with no
/// arguments (id 0x8212_0033, not traced by name).
#[test]
fn dismount_runs_on_the_vm() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let before = server.borrow().script_bytes.len();
    events_for(&mut client, &server, vec![Command::Dismount], 1);
    advance(&mut client, &server, 0.5);
    let sent = server.borrow().script_bytes[before..].to_vec();
    assert_eq!(sent, vec![(0x8212_0033, Vec::new())]);
    assert!(client.take_script_problems().is_empty());
}

/// `gmCWInterface::CanAutoAttackTarget` (IFun 0xB4000A) runs as a query on the VM and answers an
/// int back with no problem. The double's creature gets "no" (the script reads qualities the double
/// does not send; not traced), so the cycling keys keep their own rule for now.
#[test]
fn can_auto_attack_target_queries_the_script() {
    let Some((mut client, _server)) = with_script() else {
        return;
    };
    assert_eq!(client.can_auto_attack_target(CREATURE_ONE), Ok(false));
    assert!(client.take_script_problems().is_empty());
}

/// A declined resurrection offer (`gmCWInterface::ClearResurrectionRequest`) runs the player's
/// script's IFun 0xB400F7 with no problem, as the respawn's IFun 0xB4006B runs.
#[test]
fn a_declined_resurrection_offer_runs_the_clear_ifun_on_the_script() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    events_for(
        &mut client,
        &server,
        vec![Command::ClearResurrectionRequest],
        1,
    );
    assert!(client.take_script_problems().is_empty());
}
