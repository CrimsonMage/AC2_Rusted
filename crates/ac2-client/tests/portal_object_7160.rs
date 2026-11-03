//! A portal the server creates is drawn by its startup FX. Pyrealium's portals use the visual
//! `it_portal_shard.vd` (`0x1F00031E`): its one mesh is an invisible click volume
//! (`it_portal_shard_portal.mesh`, material `invisible_click.mi`), and what the player sees are
//! the looping particle systems of its startup FX (`pt_objfx_portal_a` and `_b`). Retail starts
//! them when the object's part array is made (`PHYSICSOBJREP::InitInternal` ->
//! `ApplyStartupFX(rep, NULL)` -> `FXManager::DoStartupFX` -> `DoParticleFX`), for a server
//! object as for a landblock's static one.

mod common;

use common::*;

use ac2_client::FrameInput;
use legions_server_double::Spawn;

const PORTAL: u64 = 0x8140_0000_0003_0002;
/// `it_portal_shard.vd`.
const PORTAL_VISUAL: u32 = 0x1F00_031E;
/// `pt_objfx_portal_a.psdesc` and `pt_objfx_portal_b.psdesc`, the startup FX's systems.
const PORTAL_SYSTEMS: [u32; 2] = [0x3900_00C9, 0x3900_00C8];

fn dat_world() -> Option<(Spawn, std::sync::Arc<dyn ac2_client::WorldData>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ac2_db::ResourceStore::new(dataset));
    let world = ac2_client::DatWorld::new(store).ok()?;
    let top = world
        .world()
        .load_landblock(ac2_core::CellId::new(0xCB4D_0001))
        .ok()??
        .height_grid()
        .into_iter()
        .fold(0.0, f32::max);
    let spawn = Spawn {
        cell: 0xCB4D_0025,
        origin: [96.0, 96.0, top + 2.0],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    Some((spawn, std::sync::Arc::new(world)))
}

#[test]
fn a_server_portal_starts_its_startup_particle_systems() {
    let Some((spawn, world)) = dat_world() else {
        return;
    };
    let (mut client, server) = entered_with(spawn, world);
    // As the binary does: a sound manager (silent here); the FX's particle parts run with it.
    client.attach_audio(ac2_client::sound_manager(
        std::sync::Arc::new(ac2_db::ResourceStore::new(
            ac2_dat::test_support::env_dataset().unwrap(),
        )),
        Box::new(ac2_audio::NullSink),
    ));
    let idle = FrameInput::default();
    for _ in 0..16 {
        step(&mut client, &server, &idle);
    }
    client.take_object_particle_starts();
    let me = client.frame().player.expect("player").origin;
    server.borrow_mut().spawn_creature(
        PORTAL,
        PORTAL_VISUAL,
        spawn.cell,
        [me[0], me[1] + 6.0, me[2]],
    );
    let mut starts = Vec::new();
    for _ in 0..64 {
        step(&mut client, &server, &idle);
        starts.extend(client.take_object_particle_starts());
    }
    let mut portal: Vec<u32> = starts
        .iter()
        .filter(|s| s.emitter == PORTAL)
        .map(|s| s.desc.raw())
        .collect();
    portal.sort_unstable();
    let mut expected = PORTAL_SYSTEMS.to_vec();
    expected.sort_unstable();
    assert_eq!(
        portal, expected,
        "the portal's startup FX start its two looping systems once: {starts:?}"
    );
    assert!(
        starts
            .iter()
            .filter(|s| s.emitter == PORTAL)
            .all(|s| s.looping && s.handle != 0),
        "the systems loop and are kept on their FX"
    );
    // They run on: nothing stops them while the portal stays.
    for _ in 0..64 {
        step(&mut client, &server, &idle);
    }
    let handles: Vec<u64> = starts
        .iter()
        .filter(|s| s.emitter == PORTAL)
        .map(|s| s.handle)
        .collect();
    assert!(client
        .take_object_particle_stops()
        .iter()
        .all(|h| !handles.contains(h)));
}
