//! Experience and level against the Legions server double on the script VM (`AC2_DAT_DIR`):
//! the server's quality updates reach the player's script (`CWInterface::OnStatChanged` ->
//! `CPlayer` 0x85), which raises the XP bar and the skill panel; a quality the stand-ins also
//! answered (health) raises what they raised.

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use ac2_client::{Client, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{QualityUpdate, QualityValue};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{vital, GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, State};

fn settle(client: &mut Client, server: &Rc<RefCell<State>>) -> Vec<GameplayNotice> {
    let mut events = Vec::new();
    for _ in 0..6 {
        events.extend(step(client, server, &FrameInput::default()));
    }
    events
        .into_iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n),
            _ => None,
        })
        .collect()
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

fn of_vital(notices: &[GameplayNotice], wanted: u32) -> Vec<(u64, u64)> {
    notices
        .iter()
        .filter_map(|n| match n {
            GameplayNotice::UpdateVital {
                vital,
                current,
                max,
            } if *vital == wanted => Some((*current, *max)),
            _ => None,
        })
        .collect()
}

#[test]
fn experience_and_levels_reach_the_ui_through_the_player_script() {
    let Some((mut client, server)) = logged_in(ac2_client::SCRIPT_VM_FUNCTIONS) else {
        return;
    };
    // 1000 XP: the XP bar (`CPlayer` 0x20, against the level's range from the advancement
    // table), the vitae, and the skill panel's available XP (`CPlayer` 0x85 on long 301).
    server.borrow_mut().grant_xp(1000);
    let granted = settle(&mut client, &server);
    let bar = of_vital(&granted, vital::EXPERIENCE);
    assert_eq!(bar.len(), 1, "{granted:?}");
    assert_eq!(bar[0].0, 1000);
    assert!(granted.contains(&GameplayNotice::UpdateSkillPanel {
        available_xp: 1000,
        skill_credits: 2,
        hero_skill_credits: 0
    }));
    // Level 2: the XP bar again, over the new level's range.
    server.borrow_mut().set_level(2);
    let levelled = settle(&mut client, &server);
    let bar2 = of_vital(&levelled, vital::EXPERIENCE);
    assert_eq!(bar2.len(), 1, "{levelled:?}");
    assert_ne!(bar2, bar);
    // The panels read them: the XP bar from the notices, total / available / level from the
    // qualities.
    let frame = client.frame();
    let xp = frame.player_vitals.experience.unwrap();
    assert_eq!((xp.current, xp.max), bar2[0]);
    let e = frame.progress.experience;
    assert_eq!((e.total_xp, e.available_xp, e.level), (1000, 1000, 2));
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
}

#[test]
fn a_quality_update_raises_what_the_stand_ins_raised() {
    let stood_in: Vec<u32> = ac2_client::SCRIPT_VM_FUNCTIONS
        .iter()
        .copied()
        .filter(|f| *f != ac2_weenie::IFUN_ON_STAT_CHANGED)
        .collect();
    let health = |owned: &[u32]| {
        let (mut client, server) = logged_in(owned)?;
        // Health 7 (int 0x102): `CPlayer` 0x85 -> 0x23, health and the PK damage over it.
        server.borrow_mut().send_quality(&QualityUpdate {
            object: None,
            quality: 0x102,
            value: QualityValue::Int(7),
        });
        let notices = settle(&mut client, &server);
        Some((
            of_vital(&notices, vital::HEALTH),
            of_vital(&notices, vital::PK_DAMAGE),
        ))
    };
    let Some(before) = health(&stood_in) else {
        return;
    };
    let after = health(ac2_client::SCRIPT_VM_FUNCTIONS).unwrap();
    assert_eq!(before.0, [(7, 1)]);
    assert_eq!(after, before);
}
