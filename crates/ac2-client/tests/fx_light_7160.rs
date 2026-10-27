//! FX lights on the real DATs (`AC2_DAT_DIR`, skipped without it) against the Legions 7160 server
//! double. A lamp (visual `0x1F001B6D`) has a startup FX (`0x81000069`) whose node places a light.
//! Retail places it when the object is created (`CPhysicsObj::InitObjectEnd` ->
//! `PHYSICSOBJREP::ApplyStartupFX` -> `FXManager::DoStartupFX` -> `DoLightSourceFX`), gated by
//! `Render::m_Prefs.LightSourceEffects`. `FXManager::StopFX` stops the FX's particle systems and
//! sounds; its light is not stopped, since a `CPartArray` light carries no FX id, so it runs out
//! by its lifespan.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use ac2_audio::RecordingSink;
use ac2_client::{ClientPrefs, DatWorld, FrameInput, ObjectLightStart};
use ac2_core::CellId;
use ac2_db::ResourceStore;
use ac2_net::protocol::{Opcode, VisualDesc};
use ac2_prefs::Value;
use ac2_render::prefs::LIGHT_SOURCE_EFFECTS;
use common::{entered_with, step};
use legions_server_double::{create_object_with_visual, object_event_bytes, ObjectSpawn, Spawn};

/// A lamp's visual, with a startup FX that places one light.
const LAMP: u32 = 0x1F00_1B6D;
const LAMP_FX: u32 = 0x8100_0069;
/// The lamp object beside the player.
const NPC: u64 = 0x8000_0000_0000_0060;
/// The town block CB4D, where the lamp's visual stands.
const TOWN: u32 = 0xCB4D_0025;
/// The frames the lamp has to load its body and start its FX.
const FRAMES: usize = 200;

/// The lights the lamp's FX started, with the client's preferences as they are (the registered
/// default, or `LightSourceEffects` off), after the lamp was created and, optionally, its FX was
/// stopped.
fn lamp_lights(light_effects: bool, stop_fx: bool) -> (Vec<ObjectLightStart>, Vec<u64>) {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return (Vec::new(), Vec::new());
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let world = DatWorld::new(Arc::clone(&store)).expect("world data opens");
    let top = world
        .world()
        .load_landblock(CellId::new(0xCB4D_0001))
        .unwrap()
        .unwrap()
        .height_grid()
        .into_iter()
        .fold(0.0, f32::max);
    let spawn = Spawn {
        cell: TOWN,
        origin: [96.0, 96.0, top + 2.0],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (mut client, server) = entered_with(spawn, Arc::new(world));
    client.attach_audio(ac2_client::sound_manager(
        Arc::clone(&store),
        Box::new(RecordingSink::new()),
    ));

    // `LightSourceEffects` from the client's own preferences (the DAT's names, registered).
    let mut prefs = ClientPrefs::load(
        PathBuf::from("fx_light_test_unused.ini"),
        Arc::clone(&store),
    )
    .expect("preferences register");
    if !light_effects {
        prefs
            .bank_mut()
            .find_mut(LIGHT_SOURCE_EFFECTS)
            .expect("LightSourceEffects is registered")
            .value = Value::Bool(false);
    }
    client.apply_light_preferences(prefs.bank());

    let lamp = ObjectSpawn {
        id: NPC,
        instance: 3,
        visual: LAMP,
        cell: TOWN,
        origin: [98.0, 96.0, top + 2.0],
        rotation: [1.0, 0.0, 0.0, 0.0],
        stamps: [10, 1, 1, 0],
        name: "Lamp",
    };
    let visual = VisualDesc {
        parent: Some(LAMP),
        ..VisualDesc::default()
    };
    server
        .borrow_mut()
        .send_event(&create_object_with_visual(&lamp, &visual));

    let idle = FrameInput::default();
    let mut lights = Vec::new();
    for _ in 0..FRAMES {
        step(&mut client, &server, &idle);
        lights.extend(client.take_object_light_starts());
    }
    if stop_fx {
        server
            .borrow_mut()
            .send_event(&object_event_bytes(Opcode::STOP_FX, NPC, 3, &[LAMP_FX]));
        for _ in 0..FRAMES {
            step(&mut client, &server, &idle);
            lights.extend(client.take_object_light_starts());
        }
    }
    let gone = client.take_object_lights_gone();
    (lights, gone)
}

/// The lamp's startup light starts when the lamp is created (`DoStartupFX` on its object).
#[test]
fn a_lamp_places_its_startup_light_when_it_is_created() {
    if ac2_dat::test_support::env_dataset().is_none() {
        return;
    }
    let (lights, _) = lamp_lights(true, false);
    let placed: Vec<_> = lights.iter().filter(|start| start.emitter == NPC).collect();
    assert_eq!(
        placed.len(),
        1,
        "the startup FX's one light starts once: {lights:?}"
    );
}

/// `Render::m_Prefs.LightSourceEffects` off: `DoLightSourceFX` returns without a light, so the
/// lamp's startup FX starts none.
#[test]
fn light_source_effects_off_places_no_fx_light() {
    if ac2_dat::test_support::env_dataset().is_none() {
        return;
    }
    let (lights, _) = lamp_lights(false, false);
    assert!(
        lights.iter().all(|start| start.emitter != NPC),
        "no FX light with the preference off: {lights:?}"
    );
}

/// `FXManager::StopFX` on the startup FX does not stop its light: the light stays with the lamp
/// until its lifespan runs out (no light is placed again, and none leaves with the stop).
#[test]
fn stop_fx_leaves_the_startup_light_running() {
    if ac2_dat::test_support::env_dataset().is_none() {
        return;
    }
    let (lights, gone) = lamp_lights(true, true);
    let placed = lights.iter().filter(|start| start.emitter == NPC).count();
    assert_eq!(
        placed, 1,
        "the light starts once, and StopFX places none: {lights:?}"
    );
    assert!(
        !gone.contains(&NPC),
        "the lamp still stands, so its lights do not go with it"
    );
}
