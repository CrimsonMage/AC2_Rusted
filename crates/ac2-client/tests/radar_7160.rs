//! The radar's data in the frame (`gmRadarUI`, `gmNavigationUI`): the double's creatures and
//! NPC as objects with script processes, the ranges the `RadarControl` script gives (read on the
//! VM), the heading, the groove and the map notes (`AC2_DAT_DIR`).

mod common;

use std::sync::Arc;

use ac2_client::FrameInput;
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use common::*;
use legions_server_double::{Spawn, CREATURE_ONE, CREATURE_TWO, NPC};

/// The player enters on the land cell (outdoors); the double's radar objects are placed a few
/// metres from there.
fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

#[test]
fn the_frame_carries_the_radar() {
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
    let (mut client, server) = entered(spawn());
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    server.borrow_mut().spawn_radar_objects();
    let idle = FrameInput::default();
    for _ in 0..40 {
        step(&mut client, &server, &idle);
    }
    let frame = client.frame();

    // The three objects, placed where the double put them (the NPC's name is "Scout").
    let object = |id| frame.objects.iter().find(|o| o.id == id);
    let creature = object(CREATURE_ONE).unwrap_or_else(|| panic!("{:?}", frame.objects));
    assert_eq!(creature.name.as_deref(), Some("Creature"));
    assert_eq!(creature.cell, CELL);
    assert_eq!(creature.origin, [78.0, 70.0, GROUND]);
    assert_eq!(
        object(CREATURE_TWO).map(|o| o.origin),
        Some([70.0, 82.0, GROUND])
    );
    assert_eq!(
        object(NPC).and_then(|o| o.name.clone()).as_deref(),
        Some("Scout")
    );
    // Their scripts run (the radar asks them for their looks).
    let host = client.script_host().expect("attached");
    for id in [CREATURE_ONE, CREATURE_TWO, NPC] {
        assert!(host.instance(id).is_some(), "{id:#x} has no process");
    }

    // `gRadarControl::CallIFun_InqRadarRanges` on the DAT: the outdoor range where the player
    // stands (the indoor one is 50), and the player's heading (its rotation is the identity).
    let radar = &frame.radar;
    assert_eq!(radar.range, 75.0);
    assert!(radar.heading_degrees.is_some_and(|h| h.abs() < 1e-3));
    assert_eq!(radar.groove, 0);
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );
    // The looks (`gmCWInterface::InqRadarLook`, the templates' override): the creatures red
    // (shape 0x40000004), the NPC yellow (0x40000007), each selectable and in range, in
    // creation order.
    let looks: Vec<_> = radar
        .blips
        .iter()
        .map(|b| (b.id, b.color, b.shape, b.offset, b.in_range))
        .collect();
    assert_eq!(
        looks,
        [
            (
                CREATURE_ONE,
                [1.0, 0.0, 0.0, 1.0],
                0x4000_0004,
                [8.0, 0.0, 0.0],
                true
            ),
            (
                CREATURE_TWO,
                [1.0, 0.0, 0.0, 1.0],
                0x4000_0004,
                [0.0, 12.0, 0.0],
                true
            ),
            (
                NPC,
                [1.0, 1.0, 0.0, 1.0],
                0x4000_0007,
                [-6.0, -6.0, 0.0],
                true
            ),
        ]
    );
    assert!(frame.markers.is_empty());
}
