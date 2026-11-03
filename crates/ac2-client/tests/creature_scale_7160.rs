//! A creature created with a `Scale` quality (`gmWeenieDesc` bit 0x80, 0.75 for Pyrealium's Idol Spawn)
//! has that scale on its body: `CWPhysicsObject::InitPhysics` hands the weenie's scale
//! (`CWRepository::BuildPhysObjProfile`) to `CPhysicsObj::ApplyPhysObjProfile`, which calls
//! `CPhysicsObj::SetScale(float)`. The body is drawn at it and its collision spheres are scaled
//! by it, so the player stops nearer a scaled creature than an unscaled one.

mod common;

use common::*;

use ac2_client::FrameInput;
use ac2_weenie::{ActiveAction, ACTION_CLASS_MOVEMENT, ACTION_FORWARD};
use legions_server_double::Spawn;

const IDOL: u64 = 0x8140_0000_0003_0000;
const IDOL_VISUAL: u32 = 0x1F00_03D1;

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

/// The creature's drawn scale and the distance (on the ground) at which the player, walking
/// north towards it from 6 m away, stops.
fn walk_into(scale: Option<f32>) -> ([f32; 3], f32) {
    let (spawn, world) = dat_world().expect("the DATs");
    let (mut client, server) = entered_with(spawn, world);
    let idle = FrameInput::default();
    for _ in 0..200 {
        step(&mut client, &server, &idle);
    }
    let start = client.frame().player.expect("player").origin;
    let at = [start[0], start[1] + 6.0, start[2]];
    match scale {
        Some(scale) => {
            server
                .borrow_mut()
                .spawn_scaled_creature(IDOL, IDOL_VISUAL, spawn.cell, at, scale)
        }
        None => server
            .borrow_mut()
            .spawn_creature(IDOL, IDOL_VISUAL, spawn.cell, at),
    }
    for _ in 0..128 {
        step(&mut client, &server, &idle);
    }
    let drawn = client
        .frame()
        .objects
        .iter()
        .find(|o| o.id == IDOL)
        .expect("the creature is in the frame")
        .body_scale;
    let walk = FrameInput {
        actions: vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: ACTION_FORWARD,
            map_extent: 1.0,
            extent: 1.0,
        }],
        ..FrameInput::default()
    };
    for _ in 0..256 {
        step(&mut client, &server, &walk);
    }
    let frame = client.frame();
    let stopped = frame.player.expect("player").origin;
    let creature = frame.objects.iter().find(|o| o.id == IDOL).unwrap().origin;
    (
        drawn,
        (creature[0] - stopped[0]).hypot(creature[1] - stopped[1]),
    )
}

#[test]
fn a_creature_with_a_scale_quality_is_drawn_and_collides_at_that_scale() {
    if ac2_dat::test_support::env_dataset().is_none() {
        eprintln!("skipped: AC2_DAT_DIR is unset");
        return;
    }
    let (plain_drawn, plain_gap) = walk_into(None);
    let (scaled_drawn, scaled_gap) = walk_into(Some(0.75));
    assert_eq!(plain_drawn, [1.0, 1.0, 1.0]);
    assert_eq!(scaled_drawn, [0.75, 0.75, 0.75]);
    assert!(
        plain_gap > 1.0,
        "the unscaled creature stopped the player at {plain_gap}"
    );
    // The spheres shrink with the scale: 1.53 m of radius is 1.15 m, so the player gets closer.
    assert!(
        scaled_gap < plain_gap - 0.15,
        "stopped {scaled_gap} m from the scaled creature, {plain_gap} m from the plain one"
    );
}
