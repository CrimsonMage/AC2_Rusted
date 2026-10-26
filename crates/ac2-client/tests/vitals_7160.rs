//! Vitals, skill costs and regeneration against the double, with the player's script on the VM
//! (`AC2_DAT_DIR`).

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::ShortcutInfo;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{default_options, Spawn, CREATURE_ONE, PLAYER};

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// Hum_Ma_FlameBolt: skill 0x59, `WState` package 0x172D (its slot 0 holds the skill id).
const FLAME_BOLT: u32 = 0x59;
const FLAME_BOLT_DID: u32 = 0x7000_172D;
/// The shortcut slot the test puts the skill in.
const SLOT: u32 = 2;

/// The script with the player attached and logged in (no fight started); `fight` also spawns the
/// double's creatures and starts the fight, and the shortcut bar holds Flame Bolt in slot 2.
fn with_script(fight: bool) -> Option<(ac2_client::Client, Shared)> {
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
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    {
        let mut state = server.borrow_mut();
        let mut options = default_options();
        options.shortcuts[SLOT as usize] = ShortcutInfo::Enum {
            kind: 0x4000_0001,
            value: FLAME_BOLT,
        };
        state.script_world.options = Some(options);
        state
            .script_world
            .character
            .skills
            .push((FLAME_BOLT, true, 0));
    }
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    server.borrow_mut().send_init_player();
    if fight {
        server.borrow_mut().spawn_radar_objects();
        server.borrow_mut().start_fight();
    }
    advance(&mut client, &server, 0.5);
    Some((client, server))
}

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

/// The frames of `secs` seconds (the double's clock moves a step a frame).
fn advance(client: &mut ac2_client::Client, server: &Shared, secs: f64) -> Vec<SessionEvent> {
    events_for(client, server, Vec::new(), (secs * 64.0).round() as usize)
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

/// The (current, max) of each `UpdateVital` for `vital`, in order, repeats folded.
fn vital_values(notices: &[GameplayNotice], vital: u32) -> Vec<(u64, u64)> {
    let mut values: Vec<(u64, u64)> = notices
        .iter()
        .filter_map(|n| match n {
            GameplayNotice::UpdateVital {
                vital: v,
                current,
                max,
            } if *v == vital => Some((*current, *max)),
            _ => None,
        })
        .collect();
    values.dedup();
    values
}

/// What the shown skill costs in vigor, by the examination the skill's own record builds
/// (`ActiveSkill` 0xA0): `F2I(max vigor * slot 64)` minus the flat amount in slot 60 (a negative
/// number), with the max vigor `VitalSystem` 0x4 gives.
fn flame_bolt_cost(max_vigor: u32) -> u32 {
    let dataset = ac2_dat::test_support::env_dataset().unwrap();
    let bytes = dataset
        .read_packed(ac2_dat::DatFileKind::Portal, FLAME_BOLT_DID)
        .unwrap()
        .unwrap();
    let state = ac2_script::wstate::WState::unpack(&bytes).unwrap();
    let Some(ac2_script::reference::PackedEntry::Heap { object, .. }) =
        state.references.unwrap().entries.into_iter().next()
    else {
        panic!("the skill's record");
    };
    assert_eq!(object.slots[0], FLAME_BOLT, "the record names the skill");
    let percent = f32::from_bits(object.slots[64]);
    let flat = object.slots[60] as i32;
    ((max_vigor as f32 * percent) as i32 - flat) as u32
}

/// The text a chat line renders to (`StringInfo::GetString` over the DAT's string tables).
fn text(info: &ac2_net::protocol::StringInfo) -> String {
    let dataset = ac2_dat::test_support::env_dataset().expect("dataset");
    let store = ac2_db::ResourceStore::new(dataset);
    let tables = ac2_db::StoreStringTables::new(store, 0x5800_0000);
    let rendered = ac2_db::get_string(&tables, &ac2_client::chat_text::render_info(info), false);
    String::from_utf16_lossy(&rendered)
}

/// The vitals bar's strings: `ID_Vital_DisplayText` as `current/max`.
struct Strings;

impl ac2_ui::FellowStrings for Strings {
    fn render(&self, _: u32, vars: &[(u32, ac2_ui::SkillVar)]) -> String {
        let number = |v: &ac2_ui::SkillVar| match v {
            ac2_ui::SkillVar::UInt(n) => n.to_string(),
            other => format!("{other:?}"),
        };
        vars.iter()
            .map(|(_, v)| number(v))
            .collect::<Vec<_>>()
            .join("/")
    }
}

/// Words as the little-endian bytes of an event.
fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|w| w.to_le_bytes()).collect()
}

#[test]
fn the_player_logs_in_with_full_health_and_vigor() {
    let Some((client, _server)) = with_script(false) else {
        return;
    };
    let frame = client.frame();
    let vitals = frame.player_vitals;
    // `VitalSystem` 0x1 / 0x4 over the login's `Health_CachedMax` / `Vigor_CachedMax`, the
    // current qualities at the max (the double's starting values: 50 and 60).
    let vital = |current, max| Some(ac2_weenie::Vital { current, max });
    assert_eq!(vitals.health, vital(50, 50));
    assert_eq!(vitals.vigor, vital(60, 60));
    assert!(!vitals.hero);
    // The status bar (`gmVitalsUI`) on the real layout shows both, and the PK meters sit at 0.
    let dataset = ac2_dat::test_support::env_dataset().unwrap();
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = ac2_ui::build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let shown = |tree: &ac2_ui::UiTree, name: &str| {
        let id = ac2_ui::tokenize(name);
        let element = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id))
            .unwrap();
        tree.get(element).unwrap().text().unwrap().to_string()
    };
    let text = |cur, max| ac2_ui::vital_text(&Strings, cur, max);
    hud.vitals.apply_snapshot(tree, &vitals, &text, &Strings);
    assert_eq!(shown(tree, "HealthText"), "50/50");
    assert_eq!(shown(tree, "VigorText"), "60/60");
}

/// A client that has the fight under way and the shortcut bar's Flame Bolt selected on the
/// creature, with Flame Bolt's cost set to what its record shows.
fn ready_to_cast() -> Option<(ac2_client::Client, Shared, u32)> {
    let (mut client, server) = with_script(true)?;
    events_for(
        &mut client,
        &server,
        vec![Command::Select(Some(CREATURE_ONE))],
        10,
    );
    let cost = flame_bolt_cost(60);
    server.borrow_mut().set_vigor_cost(FLAME_BOLT, cost);
    Some((client, server, cost))
}

#[test]
fn a_skill_use_drains_vigor_and_the_server_says_so() {
    let Some((mut client, server, cost)) = ready_to_cast() else {
        return;
    };
    assert_eq!(cost, 32);
    let (sent, events) = {
        let before = (
            server.borrow().script_bytes.len(),
            server.borrow().events_sent.len(),
        );
        assert_eq!(client.use_shortcut(SLOT), Ok(1));
        let events = events_for(&mut client, &server, Vec::new(), 10);
        let state = server.borrow();
        (
            (
                state.script_bytes[before.0..].to_vec(),
                state.events_sent[before.1..].to_vec(),
            ),
            events,
        )
    };
    // The request: `WM_Combat` 0x17 with the attack (1), the skill, and the target (a long).
    assert_eq!(
        sent.0,
        vec![(
            0x8204_000B,
            words(&[1, 1, 1, FLAME_BOLT, 2, 0x0A01, 0x8000_0000])
        )]
    );
    // The first thing the server sends back is the player's vigor quality (Vigor_CurrentLevel
    // 0x103, private int update 0x10069): 60 less the cost.
    assert_eq!(sent.1[0], words(&[0x0001_0069, 0x103, 60 - cost]));
    // The client's script turns it into the bar's vigor notice.
    let vigor = vital_values(&notices(events), 0x4000_0002);
    assert_eq!(vigor, vec![(u64::from(60 - cost), 60)]);
    assert_eq!(server.borrow().vitals.vigor, (60 - cost, 60));
}

#[test]
fn a_skill_use_without_the_vigor_is_refused_and_costs_nothing() {
    let Some((mut client, server, cost)) = ready_to_cast() else {
        return;
    };
    server.borrow_mut().vitals.vigor.0 = cost - 1;
    let before = server.borrow().events_sent.len();
    let health = server.borrow().fight.creatures[&CREATURE_ONE];
    assert_eq!(client.use_shortcut(SLOT), Ok(1));
    let events = events_for(&mut client, &server, Vec::new(), 10);
    let state = server.borrow();
    // `WM_Combat` 0x1A: the skill, the player (a long int) and `Combat_NotEnoughVigor` (0x4E20);
    // the only thing the server sends, and nothing was spent or struck.
    assert_eq!(
        state.events_sent[before..],
        [words(&[
            0x0001_0066,
            0x8204_001A,
            28,
            1,
            FLAME_BOLT,
            2,
            PLAYER as u32,
            (PLAYER >> 32) as u32,
            1,
            0x4E20
        ])]
    );
    assert_eq!(state.vitals.vigor.0, cost - 1);
    assert_eq!(state.fight.creatures[&CREATURE_ONE], health);
    // The client's `CCombatSystem` 0x11 turns it into a chat line (display type 1).
    let lines: Vec<(u32, String)> = notices(events)
        .iter()
        .filter_map(|n| match n {
            GameplayNotice::DisplayStringInfo { kind, text: info } => Some((*kind, text(info))),
            _ => None,
        })
        .collect();
    eprintln!("refusal lines: {lines:?}");
    assert!(
        lines
            .iter()
            .any(|(_, l)| l.to_lowercase().contains("vigor")),
        "{lines:?}"
    );
}

#[test]
fn vigor_and_health_regenerate_to_their_maxima() {
    let Some((mut client, server)) = with_script(true) else {
        return;
    };
    // Spent to 10 of 60 vigor; the fight started the player at 40 of 50 health.
    {
        let mut state = server.borrow_mut();
        state.vitals.vigor.0 = 10;
        state.fight.player.0 = 40;
        state.vitals.regen_due = state.elapsed + 1.0;
    }
    let events = advance(&mut client, &server, 3.5);
    let all = notices(events);
    // Each step is a private quality update: 4 vigor and 1 health a second at rest (the double's
    // rates, sent as the Vigor_RegenRate / Health_RegenRate floats).
    let vigor = vital_values(&all, 0x4000_0002);
    assert_eq!(vigor, vec![(14, 60), (18, 60), (22, 60)]);
    let health = vital_values(&all, 0x4000_0001);
    assert_eq!(health, vec![(41, 50), (42, 50), (43, 50)]);
    // It stops at the max and sends nothing more.
    let events = advance(&mut client, &server, 20.0);
    let all = notices(events);
    assert_eq!(vital_values(&all, 0x4000_0002).last(), Some(&(60, 60)));
    assert_eq!(vital_values(&all, 0x4000_0001).last(), Some(&(50, 50)));
    let quiet = notices(advance(&mut client, &server, 3.0));
    assert!(vital_values(&quiet, 0x4000_0002).is_empty());
    assert!(vital_values(&quiet, 0x4000_0001).is_empty());
}
