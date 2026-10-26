//! The status bar's effects (`gmCWInterface::InqEffectList`, `AC2_DAT_DIR`): the double's
//! login carries an `EffectRegistry` with a timed and a pulsed effect, and the player's script
//! (`CPlayer` 0x52 -> `CEffectSystem` 0x7) lists them.

mod common;

use std::sync::Arc;

use ac2_client::FrameInput;
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::native::NativeObject;
use ac2_script::reference::PackedEntry;
use ac2_script::vm::Program;
use ac2_script::wstate::WState;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use common::*;
use legions_server_double::{EffectGrant, Spawn, EFFECT_PULSED, EFFECT_TIMED};

/// The first `n` effects in the DATs the strip shows: `Effect` (package 0x14D) states whose
/// flags (slots 7-8) lack 0x800 (`Effect` 0xB0, hidden).
fn shown_effects(dataset: &ac2_dat::DatDataset, n: usize) -> Vec<u32> {
    let mut found = Vec::new();
    for did in dataset.ids_in_range(DatFileKind::Portal, 0x6F00_0000, 0x7FFF_FFFE) {
        let Ok(Some(bytes)) = dataset.read_packed(DatFileKind::Portal, did) else {
            continue;
        };
        let Ok(state) = WState::unpack(&bytes) else {
            continue;
        };
        let Some(PackedEntry::Heap { object, .. }) = state
            .references
            .as_ref()
            .ok()
            .and_then(|r| r.entries.first())
        else {
            continue;
        };
        let shown = |slot: usize| object.slots.get(slot).is_some_and(|f| f & 0x800 == 0);
        if state.pid == Some(0x14D) && shown(7) && shown(8) {
            found.push(did);
            if found.len() == n {
                break;
            }
        }
    }
    found
}

/// The icon (slot 19) and name (slot 29, a `StringInfo` reference) of an `Effect` state.
fn effect_icon_and_name(dataset: &ac2_dat::DatDataset, did: u32) -> (u32, String) {
    let bytes = dataset
        .read_packed(DatFileKind::Portal, did)
        .unwrap()
        .unwrap();
    let state = WState::unpack(&bytes).unwrap();
    let refs = state.references.as_ref().unwrap();
    let PackedEntry::Heap { object, .. } = &refs.entries[0] else {
        panic!("effect {did:x} is not a package instance");
    };
    let name = match refs.entries.get(object.slots[29] as usize) {
        Some(PackedEntry::Heap { object, .. }) => match &object.native {
            Some(NativeObject::StringInfo(info)) => {
                String::from_utf16_lossy(info.literal.as_deref().unwrap_or_default())
            }
            other => panic!("effect name is {other:?}"),
        },
        other => panic!("effect name is {other:?}"),
    };
    (object.slots[19], name)
}

#[test]
fn the_effect_strip_lists_a_timed_and_a_pulsed_effect() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let effects = shown_effects(&dataset, 2);
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset.clone(), store);
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
    // Nothing on the player yet: an empty strip.
    server.borrow_mut().send_init_player();
    for _ in 0..6 {
        step(&mut client, &server, &idle);
    }
    assert_eq!(client.effect_list(), Ok(Vec::new()));
    // Cast now: the double takes a timed effect off the player once its time has run out.
    let cast = server.borrow().server_time();
    server.borrow_mut().give_effects(vec![
        EffectGrant {
            eid: 0x11,
            effect: effects[0],
            time_cast: cast,
            duration: 30.0,
            flags: EFFECT_TIMED,
        },
        EffectGrant {
            eid: 0x12,
            effect: effects[1],
            time_cast: 0.0,
            duration: 0.0,
            flags: EFFECT_PULSED,
        },
    ]);
    server.borrow_mut().send_init_player();
    for _ in 0..6 {
        step(&mut client, &server, &idle);
    }
    let list = client.effect_list();
    eprintln!(
        "effects {effects:x?}: {list:?} {:?}",
        client.take_script_problems()
    );
    let list = list.unwrap();
    assert_eq!(list.len(), 2);
    // Entry order is the registry's; the timed effect (flags 1: enabled) has its
    // `SetDurationInfo` values, the pulsed one (flags 5) keeps the -1 defaults.
    let timed = list.iter().find(|n| n.eid == 0x11).unwrap();
    let (icon, name) = effect_icon_and_name(&dataset, effects[0]);
    assert_eq!(
        (timed.name.as_str(), timed.icon, timed.flags),
        (name.as_str(), icon, 1)
    );
    assert_eq!((timed.time_cast, timed.duration), (cast, 30.0));
    assert!(!timed.is_pulsed() && !timed.desc.is_empty());
    let pulsed = list.iter().find(|n| n.eid == 0x12).unwrap();
    let (icon, name) = effect_icon_and_name(&dataset, effects[1]);
    assert_eq!((pulsed.name.as_str(), pulsed.icon), (name.as_str(), icon));
    assert!(pulsed.is_pulsed());
    assert_eq!((pulsed.time_cast, pulsed.duration), (-1.0, -1.0));
}
