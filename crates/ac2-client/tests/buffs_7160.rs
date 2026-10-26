//! A self-buff from the human skill list, through the double (`AC2_DAT_DIR`): using Defensive
//! Stance (Hum_Me_DefensiveStance, skill 0xA8) from the shortcut bar costs vigor, the server adds
//! its effect to the player (`WM_Effect` 0x17) and the status bar's strip lists it; when its time
//! runs out the server takes it off again (`WM_Effect` 0x16) and the strip empties.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
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
use legions_server_double::{default_options, Spawn};

type Dataset = Arc<ac2_dat::DatDataset>;
type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// Defensive Stance: skill 0xA8, `WState` package 0x1768.
const DEFENSIVE_STANCE: u32 = 0xA8;
const SKILL_DID: u32 = 0x7000_1768;
const SLOT: u32 = 3;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
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

fn with_buff() -> Option<(ac2_client::Client, Shared, Dataset, (u32, f32))> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset.clone(), store.clone());
    data.preload_tables();
    let names = DatScriptData::new(dataset.clone(), store);
    let (mut client, server) = entered(spawn());
    client
        .session_mut()
        .set_string_tables(ac2_client::string_table_ids(&names));
    client
        .session_mut()
        .set_chat_commands(ac2_client::chat_commands(&names));
    let effect = skill_effect(&dataset);
    {
        let mut state = server.borrow_mut();
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
        state.set_skill_effect(DEFENSIVE_STANCE, effect.0, effect.1);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    server.borrow_mut().send_init_player();
    let (mut client, server) = (client, server);
    run(&mut client, &server, Vec::new(), 30);
    Some((client, server, dataset, effect))
}

fn run(
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
        events.extend(step(client, server, if i == 0 { &input } else { &idle }));
    }
    events
}

fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// The `Effect` state's name (slot 29, a `StringInfo` literal) and icon (slot 19).
fn name_and_icon(dataset: &Dataset, did: u32) -> (String, u32) {
    let refs = state(dataset, did);
    let PackedEntry::Heap { object, .. } = &refs.entries[0] else {
        panic!("effect");
    };
    let name = match entry(&refs, object.slots[29]) {
        Some(PackedEntry::Heap {
            object:
                ac2_script::reference::HeapObject {
                    native: Some(NativeObject::StringInfo(info)),
                    ..
                },
            ..
        }) => {
            // Rendered from the DAT's string tables when it names an entry.
            let store = ac2_db::ResourceStore::new(dataset.clone());
            let tables = ac2_db::StoreStringTables::new(store, 0x5800_0000);
            let info = ac2_db::RenderInfo {
                string_id: info.string_id,
                table_id: info.table_id,
                literal: info.literal.clone(),
                variables: Vec::new(),
            };
            String::from_utf16_lossy(&ac2_db::get_string(&tables, &info, false))
        }
        other => panic!("name: {other:?}"),
    };
    (name, object.slots[19])
}

#[test]
fn a_self_buff_costs_vigor_joins_the_strip_and_expires() {
    let Some((mut client, server, dataset, (effect, duration))) = with_buff() else {
        return;
    };
    assert_eq!(duration, 120.0);
    assert_eq!(client.effect_list(), Ok(Vec::new()));
    // The cost the skill's record shows: 1 vigor (`F2I(max vigor * slot 64)` less slot 60).
    server.borrow_mut().set_vigor_cost(DEFENSIVE_STANCE, 1);

    let before = server.borrow().events_sent.len();
    assert_eq!(client.use_shortcut(SLOT), Ok(1));
    let cast_at = server.borrow().server_time();
    run(&mut client, &server, Vec::new(), 12);
    let sent: Vec<Vec<u8>> = server.borrow().events_sent[before..].to_vec();
    // The server answers with the vigor spent, the effect added, and the special attack ended.
    assert_eq!(sent[0], words(&[0x0001_0069, 0x103, 59]));
    assert_eq!(sent[1][..8], words(&[0x0001_0066, 0x8208_0017]));
    assert_eq!(sent[2], words(&[0x0001_0066, 0x8204_0019, 8, 1, 1]));

    // The player's script lists it: the effect's own name and icon, timed and enabled, cast
    // when the double cast it and lasting what the record says.
    let (name, icon) = name_and_icon(&dataset, effect);
    let list = client.effect_list().unwrap();
    assert_eq!(list.len(), 1, "{list:?}");
    let node = &list[0];
    assert_eq!(
        (node.name.as_str(), node.icon, node.flags),
        (name.as_str(), icon, 1)
    );
    assert_eq!(node.duration, f64::from(duration));
    assert!(
        (node.time_cast - cast_at).abs() < 0.2,
        "{} {cast_at}",
        node.time_cast
    );

    // The status bar's strip (`gmEffectsUI`) on the real layout shows it with its name as the
    // tooltip and its meter running.
    let frame = client.frame();
    let hud_store = Arc::new(ac2_db::ResourceStore::new(dataset.clone()));
    let mut hud = ac2_ui::build_gameplay_hud(&hud_store, 1024, 768).unwrap();
    let entries = |nodes: &[ac2_client::EffectNode]| -> Vec<ac2_ui::EffectEntry> {
        nodes
            .iter()
            .map(|n| ac2_ui::EffectEntry {
                eid: n.eid,
                pulsed: n.is_pulsed(),
                name: n.name.clone(),
                icon: n.icon,
                time_cast: n.time_cast,
                duration: n.duration,
            })
            .collect()
    };
    let strip = hud.effects.as_mut().expect("the effect strip");
    hud.tree.global_loop(frame.server_time);
    strip.init(&mut hud.tree, &entries(&list), frame.server_time);
    assert_eq!(strip.shown(&hud.tree), vec![node.eid]);
    let shown = strip.element(&hud.tree, node.eid).unwrap();
    assert_eq!(hud.tree.tooltip_text(shown).as_deref(), Some(name.as_str()));
    assert_eq!(
        hud.tree.get(shown).unwrap().state(),
        ac2_ui::tokenize("active")
    );

    // Ninety seconds on it is still there; past the duration the server takes it off
    // (`WM_Effect` 0x16 with its id) and the strip is empty again.
    run(&mut client, &server, Vec::new(), 90 * 64);
    assert_eq!(client.effect_list().unwrap().len(), 1);
    let before = server.borrow().events_sent.len();
    run(&mut client, &server, Vec::new(), 32 * 64);
    assert_eq!(
        server.borrow().events_sent[before..],
        [words(&[0x0001_0066, 0x8208_0016, 8, 1, node.eid])]
    );
    assert_eq!(server.borrow().effects.expired, vec![node.eid]);
    assert_eq!(client.effect_list(), Ok(Vec::new()));
    let frame = client.frame();
    let strip = hud.effects.as_mut().unwrap();
    strip.init(&mut hud.tree, &[], frame.server_time);
    assert!(strip.shown(&hud.tree).is_empty());
}
