//! The weenies' scripts on the script VM through the client core, against the Legions server
//! double (`AC2_DAT_DIR` for the script library): the player's process starts from its weenie's
//! package, and a server call handed to the VM comes back as the VM's notice, once.

mod common;

use std::sync::Arc;

use ac2_client::FrameInput;
use ac2_core::DataId;
use ac2_net::protocol::{script_function, ScriptMessage, StringInfo};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, PLAYER};

fn host() -> Option<ScriptHost> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    Some(ScriptHost::new(program, Box::new(data)))
}

#[test]
fn a_call_handed_to_the_vm_comes_back_as_its_notice() {
    let Some(host) = host() else {
        return;
    };
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    // The calls the VM covers are its from now on.
    client.attach_scripts(host, ac2_client::SCRIPT_VM_FUNCTIONS);
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    assert!(
        client.script_host().unwrap().instance(PLAYER).is_some(),
        "{:?}",
        client.take_script_problems()
    );
    let text = StringInfo::literal("Welcome to Dereth");
    server
        .borrow_mut()
        .send_script(&ScriptMessage::DisplayStringInfo {
            function: script_function::DISPLAY_STRING_INFO,
            text: text.clone(),
            kind: 4,
        });
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(step(&mut client, &server, &idle));
    }
    let lines: Vec<&GameplayNotice> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n @ GameplayNotice::DisplayStringInfo { .. }) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(
        lines,
        [&GameplayNotice::DisplayStringInfo {
            kind: 4,
            text: text.clone()
        }],
        "{:?}",
        client.take_script_problems()
    );
    // `WM_Communication` 0x23 and 0x12 (a `wpstring` made a literal) the same way, each once,
    // as the stand-in raised them.
    server
        .borrow_mut()
        .send_script(&ScriptMessage::DisplayStringInfo {
            function: script_function::DISPLAY_STRING_INFO_2,
            text: text.clone(),
            kind: 8,
        });
    server
        .borrow_mut()
        .send_script(&ScriptMessage::DisplayText {
            text: "Welcome to Dereth".into(),
            kind: 16,
        });
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(step(&mut client, &server, &idle));
    }
    let lines: Vec<GameplayNotice> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n @ GameplayNotice::DisplayStringInfo { .. }) => Some(n.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        lines,
        [
            GameplayNotice::DisplayStringInfo {
                kind: 8,
                text: text.clone()
            },
            GameplayNotice::DisplayStringInfo { kind: 16, text }
        ],
        "{:?}",
        client.take_script_problems()
    );

    // Selecting (the stand-in's until the VM runs `CPlayer` 0x6A and 0x34 through), once.
    let npc = 0x7000_0000_0000_0042u64;
    let info = ac2_net::protocol::SelectionInfo {
        kind: ac2_net::protocol::SelectionInfo::CREATURE,
        cur_health: 7,
        max_health: 9,
        ..Default::default()
    };
    server
        .borrow_mut()
        .script_world
        .selection_infos
        .insert(npc, info);
    server
        .borrow_mut()
        .send_event(&legions_server_double::create_object_bytes(
            &legions_server_double::ObjectSpawn {
                id: npc,
                instance: 3,
                visual: 0x1F00_0023,
                cell: CELL,
                origin: [80.0, 70.0, GROUND],
                rotation: [1.0, 0.0, 0.0, 0.0],
                stamps: [10, 1, 1, 0],
                name: "Birch",
            },
        ));
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    let select = FrameInput {
        commands: vec![ac2_client::Command::Select(Some(npc))],
        ..FrameInput::default()
    };
    let mut events = step(&mut client, &server, &select);
    for _ in 0..8 {
        events.extend(step(&mut client, &server, &idle));
    }
    // The panel shows the latest `UpdateSelectionInfo` for the current selection (retail's
    // notice names no object), whoever raised it.
    assert!(events.contains(&SessionEvent::Notice(GameplayNotice::SelectionChanged(npc))));
    let selection = client.frame().selection.unwrap();
    assert_eq!((selection.id, selection.info), (npc, Some(info)));
    assert_eq!(
        server.borrow().script_requests,
        [ac2_net::protocol::ScriptRequest::SetTarget(npc)]
    );
}

/// The synthetic world, with the loader threads serving the script VM's missed reads.
struct ScriptWorld(DatScriptData);

impl ac2_client::WorldData for ScriptWorld {
    fn load_block(&self, terrain: &mut ac2_physics::Terrain, block: u32) -> Result<bool, String> {
        Synthetic.load_block(terrain, block)
    }
    fn body(&self, visual: Option<u32>) -> Result<ac2_client::BodyData, String> {
        Synthetic.body(visual)
    }
    fn game_clock(&self) -> Option<ac2_world::GameClock> {
        None
    }
    fn load_script_data(&self, read: ac2_script::vm::DataRequest) {
        self.0.load(read);
    }
}

/// The VM on cache-only data (preloaded first, as `bin/ac2` does): the login message runs
/// through `CPlayer` without a problem, and the UI reaches the VM through the client (the
/// `SetSelection` IFun, the skill panels).
#[test]
fn the_cache_only_vm_runs_the_login_and_answers_the_ui() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    data.preload_states();
    data.preload_skill_states(&program);
    data.set_cache_only(true);
    let (mut client, server) = entered_with(
        Spawn {
            cell: CELL,
            origin: [70.0, 70.0, GROUND],
            rotation: [1.0, 0.0, 0.0, 0.0],
        },
        Arc::new(ScriptWorld(data.clone())),
    );
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    assert!(client.script_host().unwrap().instance(PLAYER).is_some());
    server.borrow_mut().send_init_player();
    for _ in 0..8 {
        step(&mut client, &server, &idle);
    }
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
    let r = client
        .call_ifun(0x00B4_0002, &[ac2_script::host::IFunArg::Long(0x77)])
        .unwrap();
    assert_eq!(r.outcome.status, ac2_script::vm::InterpStatus::Terminated);
    let mut events = Vec::new();
    for _ in 0..4 {
        events.extend(step(&mut client, &server, &idle));
    }
    assert!(
        events.contains(&SessionEvent::Notice(GameplayNotice::SelectionChanged(
            0x77
        )))
    );
    let panel = client.skill_panel(1).unwrap().unwrap();
    assert!(panel.nodes.is_empty());

    // The skill panel's and shortcut bar's questions (skill 0x10; the player has no skills:
    // 30001 = not trainable here, 30009 = not trained).
    assert_eq!(client.can_skill_be_trained(0x10), Ok(30001));
    assert_eq!(client.can_skill_be_untrained(0x10), Ok(30009));
    assert_eq!(client.can_skill_be_raised(0x10), Ok(30009));
    assert_eq!(client.skill_cost_to_train(0x10), Ok(5));
    assert_eq!(client.is_hero_skill(0x10), Ok(false));
    assert_eq!(client.skill_experience_to_level(0x10), Ok(125_000));
    let level = client.skill_level(0x10).unwrap();
    assert_eq!((level.raw, level.enchanted), (0, 0));
    assert!(!client.skill_time_left(0x10).unwrap().ok);
    assert!(!client.shortcut_info(0).unwrap().ok);
    let settings = client.shortcut_settings().unwrap();
    assert_eq!((settings.which_set, settings.height), (0, 1));
    assert!(client.shortcut_array().unwrap().is_empty());
    // The chat windows' defaults (`GameplayOptionsProfile::Clear`, Legions).
    let chat = client.chat_settings().unwrap();
    assert_eq!(chat.filters.get(&1), Some(&0x0060_017F));
    assert_eq!(chat.font_colors.get(&0x100), Some(&9));
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}
