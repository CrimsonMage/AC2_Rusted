//! A creature the server creates on top of the player does not trap the player. AC2e's training
//! world puts an Idol Spawn (visual `0x1F0003D1`, two collision spheres of 1.5 m) in the Menhir
//! Vault 0.8 m from the human start. The player is a physics object in the cell's shadow lists
//! like any other (`CPhysicsObj::add_to_cell`), so the creature's own world entry
//! (`CPhysicsObj::EnterWorld` with `SLIDE_SPF`, `CTransition::find_placement_position`) tests the
//! player's spheres and slides to a free spot; the player never starts inside a solid creature
//! (inside one, every step is a slide that stays inside, `CSphere::slide_sphere`, and
//! `CTransition::validate_transition` takes the body back to where it was).

mod common;

use std::sync::Arc;

use ac2_client::{ClientFrame, FrameInput};
use ac2_weenie::{
    ActiveAction, ACTION_CLASS_MOVEMENT, ACTION_STRAFE_NEGATIVE, ACTION_STRAFE_POSITIVE,
};
use common::*;
use legions_server_double::Spawn;

const INTERIOR_CELL: u32 = 0x0299_0109;
const INTERIOR_ORIGIN: [f32; 3] = [59.060_577, 240.199, -44.606_205];
/// The Idol Spawn AC2e authors at the start, 0.8 m from the player.
const IDOL_ORIGIN: [f32; 3] = [59.0, 241.0, -45.005_77];
const IDOL: u64 = 0x8140_0000_0003_0000;
const IDOL_VISUAL: u32 = 0x1F00_03D1;

fn dat_world() -> Option<(Spawn, Arc<dyn ac2_client::WorldData>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
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
    Some((spawn, Arc::new(world)))
}

fn player_of(frame: &ClientFrame) -> [f32; 3] {
    frame.player.as_ref().expect("the player").origin
}

#[test]
fn a_creature_created_on_the_player_slides_off_and_the_player_walks() {
    let Some((spawn, world)) = dat_world() else {
        eprintln!("skipped: AC2_DAT_DIR is unset");
        return;
    };
    let (mut client, server) = entered_with(spawn, world);
    let idle = FrameInput::default();
    for _ in 0..600 {
        step(&mut client, &server, &idle);
        if !client.frame().portal.in_portal_space {
            break;
        }
    }
    // The teleport to the vault; the player is placed there before the creature comes (AC2e
    // creates the world's creatures after the player's entry).
    server
        .borrow_mut()
        .teleport_player(INTERIOR_CELL, INTERIOR_ORIGIN, 5, 6);
    let mut frames = 0;
    loop {
        step(&mut client, &server, &idle);
        let frame = client.frame();
        if frame.in_world
            && !frame.portal.teleporting
            && frame
                .player
                .as_ref()
                .is_some_and(|p| p.cell == INTERIOR_CELL)
        {
            break;
        }
        frames += 1;
        assert!(frames < 3000, "the player was not placed in the vault");
    }
    for _ in 0..16 {
        step(&mut client, &server, &idle);
    }
    server
        .borrow_mut()
        .spawn_creature(IDOL, IDOL_VISUAL, INTERIOR_CELL, IDOL_ORIGIN);
    let mut frames = 0;
    while !client
        .frame()
        .objects
        .iter()
        .any(|o| o.id == IDOL && o.in_world)
    {
        step(&mut client, &server, &idle);
        frames += 1;
        assert!(frames < 3000, "the creature did not enter the world");
    }
    for _ in 0..64 {
        step(&mut client, &server, &idle);
    }
    let frame = client.frame();
    let start = player_of(&frame);
    let idol = frame
        .objects
        .iter()
        .find(|o| o.id == IDOL)
        .expect("the creature");
    let apart = ((idol.origin[0] - start[0]).powi(2) + (idol.origin[1] - start[1]).powi(2)).sqrt();
    // The creature's spheres are 1.5 m wide (3 m across at their widest), the player's 0.55 m:
    // placed clear of the player it stands further than 0.8 m off.
    assert!(
        apart > 1.5,
        "the creature at {:?} still overlaps the player at {start:?} ({apart} m)",
        idol.origin
    );

    // The creature stands in front of the player once it has slid clear, so the player walks
    // sideways, both ways.
    for (what, action) in [
        ("strafing right", ACTION_STRAFE_POSITIVE),
        ("strafing left", ACTION_STRAFE_NEGATIVE),
    ] {
        let before = player_of(&client.frame());
        let walk = FrameInput {
            actions: vec![ActiveAction {
                class: ACTION_CLASS_MOVEMENT,
                enum_value: action,
                map_extent: 1.0,
                extent: 1.0,
            }],
            ..FrameInput::default()
        };
        for _ in 0..64 {
            step(&mut client, &server, &walk);
        }
        let after = player_of(&client.frame());
        let moved = ((after[0] - before[0]).powi(2) + (after[1] - before[1]).powi(2)).sqrt();
        assert!(
            moved > 1.0,
            "walking {what} moved the body {moved} m, from {before:?} to {after:?}"
        );
    }
}
