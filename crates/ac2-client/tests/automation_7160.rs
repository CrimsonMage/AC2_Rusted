//! The automatic tools against the Legions server double (`AC2_DAT_DIR`): each sends exactly
//! what the player's own gestures send. The auto-butcher uses the knife on the corpse as the
//! inventory's double-click and the targeting click do (`butchery_7160`), the auto-rebuff casts
//! Defensive Stance from its shortcut bar slot as the bar's press does (`buffs_7160`): the same
//! script requests, byte for byte, as the manual run of the same scene.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_macro::automation::{RebuffEntry, Settings};
use ac2_net::protocol::ShortcutInfo;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::native::NativeObject;
use ac2_script::reference::PackedEntry;
use ac2_script::vm::Program;
use ac2_script::wstate::WState;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::SessionEvent;
use common::*;
use legions_server_double::{
    default_options, Spawn, BUTCHER_KNIFE, BUTCHER_SKIN, CORPSE, CREATURE_ONE, PLAYER,
};

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;
type Dataset = Arc<ac2_dat::DatDataset>;
const SKILL_DID: u32 = 0x7000_1768;

const HIT_INTERVAL: f64 = 2.0;
const DEFENSIVE_STANCE: u32 = 0xA8;
const SLOT: u32 = 3;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

fn frames(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
    count: usize,
) -> Vec<SessionEvent> {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut events = Vec::new();
    for i in 0..count {
        events.extend(step(client, server, if i == 0 { &input } else { &idle }));
    }
    events
}

/// A client with the script attached, in the world at the double's spawn; `setup` prepares the
/// double first.
fn scripted(
    setup: impl FnOnce(&mut legions_server_double::State),
) -> Option<(ac2_client::Client, Shared)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset.clone(), store.clone());
    data.preload_tables();
    let names = DatScriptData::new(dataset, store);
    let (mut client, server) = entered(spawn());
    client
        .session_mut()
        .set_string_tables(ac2_client::string_table_ids(&names));
    client
        .session_mut()
        .set_chat_commands(ac2_client::chat_commands(&names));
    setup(&mut server.borrow_mut());
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    step(&mut client, &server, &FrameInput::default());
    server.borrow_mut().send_init_player();
    Some((client, server))
}

/// `butchery_7160`'s scene: the knife in the pack and the creature killed, its corpse beside the
/// player.
fn with_corpse() -> Option<(ac2_client::Client, Shared)> {
    let (mut client, server) = scripted(|state| {
        state.give_starter_items();
        state.send_contents(PLAYER);
    })?;
    server.borrow_mut().spawn_radar_objects();
    server.borrow_mut().start_fight();
    server.borrow_mut().give_butcher_tool();
    frames(&mut client, &server, Vec::new(), 64);
    frames(
        &mut client,
        &server,
        vec![Command::Select(Some(CREATURE_ONE))],
        10,
    );
    frames(&mut client, &server, vec![Command::SetAttacking(true)], 1);
    frames(
        &mut client,
        &server,
        Vec::new(),
        (3.0 * HIT_INTERVAL * 64.0) as usize + 64,
    );
    assert!(server.borrow().fight.dead.contains(&CREATURE_ONE));
    frames(
        &mut client,
        &server,
        vec![Command::SetAttacking(false)],
        ((legions_server_double::DEATH_SECONDS + 0.5) * 64.0) as usize,
    );
    Some((client, server))
}

/// The script requests the double received since `mark`.
fn requests_since(server: &Shared, mark: usize) -> Vec<(u32, Vec<u8>)> {
    server.borrow().script_bytes[mark..].to_vec()
}

#[test]
fn the_auto_butcher_sends_what_the_knife_and_the_targeting_click_send() {
    // By hand: the knife's double-click, then the click on the corpse.
    let Some((mut manual, manual_server)) = with_corpse() else {
        return;
    };
    let mark = manual_server.borrow().script_bytes.len();
    frames(
        &mut manual,
        &manual_server,
        vec![Command::Select(Some(BUTCHER_KNIFE)), Command::UseSelection],
        10,
    );
    frames(
        &mut manual,
        &manual_server,
        vec![Command::Select(Some(CORPSE)), Command::UseSelection],
        64,
    );
    let by_hand = requests_since(&manual_server, mark);
    assert!(!by_hand.is_empty());
    assert!(manual.frame().inventory.pack.items.contains(&BUTCHER_SKIN));

    // The tool, switched on in the same scene.
    let (mut client, server) = with_corpse().unwrap();
    let mark = server.borrow().script_bytes.len();
    let mut settings = Settings::default();
    settings.butcher.enabled = true;
    // Any corpse: the gestures alone, compared byte for byte (the killer check is below).
    settings.butcher.only_my_kills = false;
    client.set_automation_settings(settings);
    frames(&mut client, &server, Vec::new(), 4 * 64);
    assert_eq!(
        requests_since(&server, mark),
        by_hand,
        "the same requests as by hand"
    );
    assert!(client.frame().inventory.pack.items.contains(&BUTCHER_SKIN));
    // The corpse is not tried again, and the tool stays on.
    frames(&mut client, &server, Vec::new(), 8 * 64);
    assert_eq!(requests_since(&server, mark), by_hand);
    assert!(client.automation_settings().butcher.enabled);
    assert!(client.take_script_problems().is_empty());
}

fn state(dataset: &Dataset, did: u32) -> ac2_script::reference::PackedReference {
    let bytes = dataset
        .read_packed(DatFileKind::Portal, did)
        .unwrap()
        .unwrap();
    WState::unpack(&bytes).unwrap().references.unwrap()
}

/// The entry a reference slot names (`ids` maps the packed id to the entry).
fn entry(refs: &ac2_script::reference::PackedReference, id: u32) -> Option<&PackedEntry> {
    refs.ids
        .iter()
        .position(|&i| i == id)
        .map(|i| &refs.entries[i])
}

/// The items of the list a slot of the root object names.
fn list_slot(refs: &ac2_script::reference::PackedReference, slot: usize) -> Vec<u64> {
    let PackedEntry::Heap { object, .. } = &refs.entries[0] else {
        panic!("root");
    };
    match entry(refs, object.slots[slot]) {
        Some(PackedEntry::Heap {
            object:
                ac2_script::reference::HeapObject {
                    native: Some(NativeObject::List(list)),
                    ..
                },
            ..
        }) => list.items.clone(),
        other => panic!("slot {slot}: {other:?}"),
    }
}

/// What the skill's record says it applies: the `Effect`'s DID (the skill's slot 36 names a
/// list of effects) and its duration (the first pair in the effect's own list of
/// `FloatScaleDuple`s, slot 0; the double's pick of how long a cast lasts).
fn skill_effect(dataset: &Dataset) -> (u32, f32) {
    let skill = state(dataset, SKILL_DID);
    let ids = list_slot(&skill, 36);
    let Some(PackedEntry::Package { did, .. }) = entry(&skill, ids[0] as u32) else {
        panic!("the effect");
    };
    let effect = state(dataset, *did);
    let pairs = list_slot(&effect, 0);
    let Some(PackedEntry::Heap { object, .. }) = entry(&effect, pairs[0] as u32) else {
        panic!("the pair");
    };
    (*did, f32::from_bits(object.slots[0]))
}

/// `buffs_7160`'s scene: Defensive Stance on bar slot 3, its effect lasting 120 s.
fn with_stance() -> Option<(ac2_client::Client, Shared)> {
    let (mut client, server) = scripted(|state| {
        let mut options = default_options();
        options.shortcuts[SLOT as usize] = ShortcutInfo::Enum {
            kind: 0x4000_0001,
            value: DEFENSIVE_STANCE,
        };
        state.script_world.options = Some(options);
        state
            .script_world
            .character
            .skills
            .push((DEFENSIVE_STANCE, true, 0));
    })?;
    // The effect the skill's record applies, as `buffs_7160` reads it.
    let (did, duration) = skill_effect(&ac2_dat::test_support::env_dataset()?);
    server
        .borrow_mut()
        .set_skill_effect(DEFENSIVE_STANCE, did, duration);
    server.borrow_mut().set_vigor_cost(DEFENSIVE_STANCE, 1);
    frames(&mut client, &server, Vec::new(), 30);
    Some((client, server))
}

#[test]
fn the_auto_rebuff_sends_what_the_bar_press_sends_and_recasts_a_lapsed_effect() {
    let Some((mut manual, manual_server)) = with_stance() else {
        return;
    };
    let mark = manual_server.borrow().script_bytes.len();
    assert_eq!(manual.use_shortcut(SLOT), Ok(1));
    frames(&mut manual, &manual_server, Vec::new(), 64);
    let by_hand = requests_since(&manual_server, mark);
    assert_eq!(by_hand.len(), 1, "{by_hand:?}");
    let name = manual.effect_list().unwrap()[0].name.clone();

    let (mut client, server) = with_stance().unwrap();
    let mark = server.borrow().script_bytes.len();
    let mut settings = Settings::default();
    settings.rebuff.enabled = true;
    settings.rebuff.entries.push(RebuffEntry {
        effect: name.clone(),
        skill: DEFENSIVE_STANCE,
        enabled: true,
    });
    client.set_automation_settings(settings);
    frames(&mut client, &server, Vec::new(), 64);
    assert_eq!(
        requests_since(&server, mark),
        by_hand,
        "the same request as the bar's press"
    );
    assert_eq!(client.effect_list().unwrap()[0].name, name);
    // Up for 120 s: nothing more while it lasts; when the server takes it off, the same press
    // again. (The double stamps the effect with its own clock, which runs ahead of the client's
    // in this scene, so the lead time is not reached here; the policy's tests cover it.)
    frames(&mut client, &server, Vec::new(), 100 * 64);
    assert_eq!(requests_since(&server, mark).len(), 1);
    frames(&mut client, &server, Vec::new(), 25 * 64);
    let sent = requests_since(&server, mark);
    assert_eq!(sent.len(), 2, "{sent:?}");
    assert_eq!(sent[1].0, by_hand[0].0);
    assert!(client.take_script_problems().is_empty());
}

/// A corpse's profile as Pyrealium sends it: "Corpse of", then "Killed by" (`examination.stt`
/// 0x32A6B06F) with the killer's tagged name in the StringInfo variable 0x23A664C8.
fn corpse_profile(killer: &str) -> ac2_net::protocol::ExaminationProfile {
    use ac2_net::protocol::{
        ExaminationDataNode, ExaminationProfile, ExaminationValue, StringInfo, StringVariable,
    };
    let text = |string_id, variables| ExaminationDataNode {
        order: 2,
        appearance: 0x33BE_4035,
        value: ExaminationValue::Text(StringInfo {
            string_id,
            table_id: 0x2500_0011,
            literal: None,
            variables,
        }),
    };
    let name = StringInfo {
        literal: Some(killer.to_owned()),
        ..StringInfo::default()
    };
    ExaminationProfile {
        nodes: vec![
            text(0x509A_8FF6, vec![]),
            text(
                0x32A6_B06F,
                vec![(0x23A6_64C8, StringVariable::Nested(Box::new(name)))],
            ),
        ],
        ..ExaminationProfile::default()
    }
}

/// With "only my kills" (the default) the tool asks for the corpse's profile first and butchers
/// it only when the profile names the player as its killer.
#[test]
fn the_auto_butcher_butchers_only_the_players_kills() {
    for (mine, expect_skin) in [(true, true), (false, false)] {
        let Some((mut client, server)) = with_corpse() else {
            return;
        };
        let player = client
            .session_mut()
            .character_name(PLAYER)
            .expect("the player's name");
        let killer = if mine {
            format!("{player} [M]")
        } else {
            "Someone Else [F]".to_owned()
        };
        server
            .borrow_mut()
            .script_world
            .profiles
            .insert(CORPSE, corpse_profile(&killer));
        let before = server.borrow().script_requests.len();
        let mut settings = Settings::default();
        settings.butcher.enabled = true;
        client.set_automation_settings(settings);
        frames(&mut client, &server, Vec::new(), 6 * 64);
        let asked = server.borrow().script_requests[before..].to_vec();
        assert_eq!(
            asked
                .iter()
                .filter(|r| **r
                    == ac2_net::protocol::ScriptRequest::Examine(
                        ac2_net::protocol::ExaminationRequest::object(CORPSE)
                    ))
                .count(),
            1,
            "the corpse is examined once ({killer})"
        );
        assert_eq!(
            client.frame().inventory.pack.items.contains(&BUTCHER_SKIN),
            expect_skin,
            "{killer}"
        );
        assert!(client.automation_settings().butcher.enabled);
    }
}
