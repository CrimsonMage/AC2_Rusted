//! A teleport on the real DATs (`AC2_DAT_DIR`), headless, against the Legions 7160 server double.
//!
//! The player stands in the town block. The double teleports it (one `PositionPack` with a newer
//! teleport stamp) into the 0299 interior, another landblock: the body leaves the world, the
//! portal space fades the world out, the destination's cells load on the loader threads, and the
//! player is placed there. The portal space then shuts down, and the frame drawn after the
//! placement is acknowledged with `SceneRenderingComplete`.

mod common;

use std::sync::Arc;

use ac2_client::{Client, ClientFrame, FrameInput, Notice};
use ac2_render::portal::Phase;
use common::*;
use legions_server_double::{Spawn, State, TeleportMessage};

/// The town block's interior destination (landblock 0299) and its origin.
const INTERIOR_CELL: u32 = 0x0299_0109;
const INTERIOR_ORIGIN: [f32; 3] = [59.0, 240.0, -44.5];
const INTERIOR_BLOCK: u32 = 0x0299_0000;
/// `SceneRenderingComplete` (`0x000100D1`), little-endian, the whole payload.
const SCENE_RENDERING_COMPLETE: [u8; 4] = [0xD1, 0x00, 0x01, 0x00];

/// The DAT world and the town spawn (the ambient music test's landblock), or `None` without DATs.
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

/// One frame; the notices it raised.
fn frame(
    client: &mut Client,
    server: &std::rc::Rc<std::cell::RefCell<State>>,
    notices: &mut Vec<Notice>,
) -> ClientFrame {
    step(client, server, &FrameInput::default());
    notices.extend(client.take_notices());
    client.frame()
}

/// The frame drawn acknowledgements the double has received.
fn acks(server: &std::rc::Rc<std::cell::RefCell<State>>) -> usize {
    server
        .borrow()
        .blobs
        .iter()
        .filter(|(_, _, data)| data.as_slice() == SCENE_RENDERING_COMPLETE)
        .count()
}

#[test]
fn a_teleport_runs_the_portal_space_and_places_the_player_at_the_destination() {
    let Some((spawn, world)) = dat_world() else {
        eprintln!("skipped: AC2_DAT_DIR is unset");
        return;
    };
    let (mut client, server) = entered_with(spawn, world);
    let mut notices = Vec::new();
    let mut settled = client.frame();
    assert!(settled.in_world, "the player stands in the town");
    // The login's portal space: the world's entry enters it (`gmGamePlayUI::RecvNotice_PlayerReady
    // ToEnterWorld` while the weenie is not ready), and the description's `LoginFinished` lets the
    // weenie out, so the gate exits once the player is placed. Its fade runs out before play.
    let mut login_frames = 0;
    while settled.portal.in_portal_space {
        settled = frame(&mut client, &server, &mut notices);
        login_frames += 1;
        assert!(
            login_frames < 2000,
            "the login's portal space did not run out"
        );
    }
    assert!(settled.in_world, "the player stands in the town");
    assert!(!settled.portal.in_portal_space && settled.portal.world_drawn());
    let acks_before = acks(&server);

    // The teleport: the server's position with a newer teleport stamp (5 -> 6).
    server
        .borrow_mut()
        .teleport_player(INTERIOR_CELL, INTERIOR_ORIGIN, 5, 6);
    let start = frame(&mut client, &server, &mut notices);
    // `SmartBox::HandleTeleport`: the body leaves the world, the portal space is wanted, and the
    // destination is not resident yet (its cells load on the loader threads). The effect's
    // `Think` runs in the SmartBox step, before the events, so its phase starts the next frame.
    assert!(!start.in_world, "the body left the world");
    assert!(start.portal.in_portal_space && start.portal.teleporting);
    assert!(!start.blocks.contains(&INTERIOR_BLOCK));
    assert!(
        notices.contains(&Notice::PlayerTeleportStart),
        "{notices:?}"
    );
    let fading = frame(&mut client, &server, &mut notices);
    assert_eq!(fading.portal.phase, Phase::StartupPhase0);
    assert!(
        fading.portal.world_drawn(),
        "the world fades out, still drawn"
    );

    // The destination's cells come in (the harness settles the loads, so this is a few frames),
    // the player is placed there (the end), and the effect's phases run out. Record every frame.
    let mut phases = vec![fading.portal.phase];
    let mut drawn_while_active = Vec::new();
    let mut placed = fading;
    let mut frames_left = 2000;
    while placed.portal.teleporting || placed.portal.phase != Phase::Inactive {
        placed = frame(&mut client, &server, &mut notices);
        if phases.last() != Some(&placed.portal.phase) {
            phases.push(placed.portal.phase);
        }
        if placed.portal.phase == Phase::Active {
            drawn_while_active.push(placed.portal.world_drawn());
        }
        frames_left -= 1;
        assert!(
            frames_left > 0,
            "the portal space did not run out: {:?}",
            placed.blocks
        );
        if !placed.portal.teleporting && placed.in_world {
            // Placed: the destination is resident and the body is where the server put it.
            assert!(placed.blocks.contains(&INTERIOR_BLOCK));
            let player = placed.player.as_ref().expect("the player's body");
            assert_eq!(player.cell, INTERIOR_CELL);
            // The body is placed with sliding (`EnterWorldSimple(pos, sliding = 1)`): x and y are
            // the server's, z settles onto the floor under them.
            let [x, y, z] = player.origin;
            assert!((x - INTERIOR_ORIGIN[0]).abs() < 1e-3 && (y - INTERIOR_ORIGIN[1]).abs() < 1e-3);
            assert!((z - INTERIOR_ORIGIN[2]).abs() < 1.0, "{:?}", player.origin);
        }
    }
    // The tunnel phase: the world is not drawn while it runs.
    assert_eq!(
        phases,
        vec![
            Phase::StartupPhase0,
            Phase::Active,
            Phase::ShutdownPhase0,
            Phase::ShutdownPhase1,
            Phase::Inactive
        ],
        "the phases of the portal space"
    );
    assert_eq!(
        drawn_while_active,
        vec![false],
        "the world is not drawn in the tunnel"
    );
    assert!(notices.contains(&Notice::TeleportEnd), "{notices:?}");
    assert!(placed.in_world && !placed.portal.in_portal_space && placed.portal.world_drawn());

    // The frame drawn after the placement is acknowledged, byte for byte.
    assert_eq!(acks(&server), acks_before + 1);
    let last_ack = server
        .borrow()
        .blobs
        .iter()
        .rev()
        .find(|(_, _, data)| data.as_slice() == SCENE_RENDERING_COMPLETE)
        .map(|(_, _, data)| data.clone());
    assert_eq!(last_ack.as_deref(), Some(&SCENE_RENDERING_COMPLETE[..]));

    // The double's trace: the teleport is one position pack, in order.
    assert_eq!(
        server.borrow().teleport_trace,
        vec![TeleportMessage::PositionPack {
            cell: INTERIOR_CELL,
            origin: INTERIOR_ORIGIN,
            teleport_stamp: 6,
        }]
    );
}

/// `gmGamePlayUI::gmGamePlayUI`: the gameplay UI starts in portal space at once
/// (`SmartBox::EnterPortalSpace(1)`), before the world's entry, and the login still lets the
/// player out.
#[test]
fn the_gameplay_ui_starts_in_portal_space_and_the_login_ends_it() {
    let Some((spawn, world)) = dat_world() else {
        eprintln!("skipped: AC2_DAT_DIR is unset");
        return;
    };
    let (transport, server) = legions_server_double::double(spawn);
    let session = ac2_weenie::OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    let mut client = Client::new(session, world);
    client.set_load_budget(std::time::Duration::from_secs(5));
    assert!(!client.frame().portal.in_portal_space);
    client.gameplay_ui_started();
    assert!(client.frame().portal.in_portal_space);
    let mut notices = Vec::new();
    let mut frames = 0;
    let mut current = client.frame();
    while !current.in_world || current.portal.in_portal_space {
        current = frame(&mut client, &server, &mut notices);
        frames += 1;
        assert!(frames < 3000, "the portal space did not run out");
    }
    assert!(current.portal.world_drawn());
}
