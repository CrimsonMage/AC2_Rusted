//! The server double's character on the script VM (`AC2_DAT_DIR`): its race, the skills the
//! repository carries before the login message and the shortcut bar its options profile holds
//! reach the player's script, so the skill panels and the bar answer from real data.

mod common;

use std::sync::Arc;

use ac2_client::FrameInput;
use ac2_core::DataId;
use ac2_net::protocol::ShortcutInfo;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use common::*;
use legions_server_double::{Spawn, MELEE_ATTACK, MELEE_AUTO_ATTACK, PLAYER};

#[test]
fn the_doubles_character_reaches_the_player_script() {
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
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
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

    // The race (int 290, Human): the melee root panel is the Human one (enum mapper
    // 0x4100001D: RootMeleePanel 0x80000000 -> HumanRootMeleePanel 0x8000000C).
    assert_eq!(
        client.root_panel_from_race(0x8000_0000),
        Ok(Some(0x8000_000C))
    );

    // The repository: both skills trained, credits 2, in the session's mirror and the VM's.
    let skills = client.frame().progress.skills;
    assert!(skills.is_trained(MELEE_AUTO_ATTACK) && skills.is_trained(MELEE_ATTACK));
    assert_eq!(skills.credits, 2);
    // The VM's player has them: training answers `Skill_AlreadyTrained`, raising something
    // other than `Skill_NotTrained`.
    assert_eq!(client.can_skill_be_trained(MELEE_ATTACK), Ok(30000));
    assert_ne!(client.can_skill_be_raised(MELEE_ATTACK), Ok(30009));

    // The bar: ten slots, set 0, one row; Melee Auto Attack first, a chat line second.
    let bar = client.shortcut_array().unwrap();
    assert_eq!(bar.len(), 10);
    assert_eq!(
        bar[0],
        ShortcutInfo::Enum {
            kind: 0x4000_0001,
            value: MELEE_AUTO_ATTACK
        }
    );
    assert_eq!(bar[1], ShortcutInfo::Text("/e waves".into()));
    let settings = client.shortcut_settings().unwrap();
    assert_eq!((settings.which_set, settings.height), (0, 1));
    assert_eq!(client.frame().progress.shortcuts.slots, bar);
}
