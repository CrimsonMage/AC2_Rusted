//! Movement against a running AC2e server (`AC2E_LIVE`, see `tests/live/mod.rs`): a new character
//! leaves its start in the Menhir Vault, walks to the room's lifestone and binds to it, and walks
//! to its portal and travels. AC2e's training world (`training-world.tsv`) keeps its Idol Spawns
//! (scale 0.75) clear of the start and of those ways; even where a creature stood on the start, its
//! world entry would slide it clear of the player (`CPhysicsObj::EnterWorld`, `SLIDE_SPF`), as
//! the player is listed in the object lists like any object (`CPhysicsObj::add_to_cells`).
//!
//! Run with `AC2E_LIVE_TEST=ac2e_live_movement scripts/ac2e-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_weenie::{
    ACTION_CLASS_MOVEMENT, ACTION_FORWARD, ACTION_STRAFE_NEGATIVE, ACTION_STRAFE_POSITIVE,
    ActiveAction, GameplayNotice, SessionEvent,
};
use live::LiveRig;

/// The Menhir Vault: the start room's cell.
const START_CELL: u32 = 0x0299_0109;
/// `LIFESTONE`'s and the portal's entity DIDs (`training-world.tsv`).
const LIFESTONE: u32 = 0x4700_07DE;
const PORTAL: u32 = 0x4700_084D;
/// `training-world.tsv`: the Idol Spawn.
const IDOL: u32 = 0x4700_07D1;

fn origin(rig: &LiveRig) -> [f32; 3] {
    rig.client
        .frame()
        .player
        .as_ref()
        .expect("the player's body")
        .origin
}

fn cell(rig: &LiveRig) -> u32 {
    rig.client.frame().player.as_ref().expect("the player").cell
}

/// The world objects within 8 m of the player, for a failure message.
fn near(rig: &LiveRig) -> Vec<(u64, [f32; 3], bool)> {
    let me = origin(rig);
    rig.client
        .frame()
        .objects
        .iter()
        .filter(|o| (o.origin[0] - me[0]).hypot(o.origin[1] - me[1]) < 8.0)
        .map(|o| (o.id & 0xFFFF, o.origin, o.in_world))
        .collect()
}

/// Holds `action` for `seconds`, releases it, and gives the metres the body moved on the ground.
fn walk(rig: &mut LiveRig, action: u32, seconds: f64) -> f32 {
    let before = origin(rig);
    rig.held = vec![ActiveAction {
        class: ACTION_CLASS_MOVEMENT,
        enum_value: action,
        map_extent: 1.0,
        extent: 1.0,
    }];
    rig.run(seconds);
    rig.held.clear();
    rig.run(0.5);
    let after = origin(rig);
    (after[0] - before[0]).hypot(after[1] - before[1])
}

/// Where the session says object `id` stands (planar), once it is in the world.
fn place_of(rig: &LiveRig, id: u64) -> Option<[f32; 3]> {
    rig.client
        .frame()
        .objects
        .iter()
        .find(|o| o.id == id && o.in_world)
        .map(|o| o.origin)
}

fn planar(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Holds `action` until the body is within `within` metres of `target` (or `seconds` run out),
/// then lets go and lets it settle. The distance after.
fn walk_towards(
    rig: &mut LiveRig,
    action: u32,
    target: [f32; 3],
    within: f32,
    seconds: f64,
) -> f32 {
    rig.held = vec![ActiveAction {
        class: ACTION_CLASS_MOVEMENT,
        enum_value: action,
        map_extent: 1.0,
        extent: 1.0,
    }];
    let mut run = 0.0;
    while run < seconds && planar(origin(rig), target) > within {
        rig.run(0.05);
        run += 0.05;
    }
    rig.held.clear();
    rig.run(0.5);
    planar(origin(rig), target)
}

/// The chat lines the server said since event `from`.
fn lines(rig: &LiveRig, from: usize) -> Vec<String> {
    rig.seen[from..]
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::DisplayStringInfo { text, .. })
            | SessionEvent::Notice(GameplayNotice::DisplayMessage { text, .. }) => {
                text.literal.clone()
            }
            _ => None,
        })
        .collect()
}

/// The id of the session's object of `entity` that is in the world.
fn object_of(rig: &LiveRig, entity: u32) -> Option<u64> {
    rig.client
        .session()
        .objects()
        .iter()
        .find(|o| o.entity == Some(entity))
        .map(|o| o.id)
}

fn said(rig: &LiveRig, from: usize, part: &str) -> bool {
    rig.seen[from..].iter().any(|e| match e {
        SessionEvent::Notice(GameplayNotice::DisplayStringInfo { text, .. })
        | SessionEvent::Notice(GameplayNotice::DisplayMessage { text, .. }) => {
            text.literal.as_deref().is_some_and(|t| t.contains(part))
        }
        _ => false,
    })
}

fn start() -> Option<LiveRig> {
    let mut rig = LiveRig::new("Walker")?;
    // The start: the vault's cell, the player on its floor, the world's creatures placed.
    rig.run(3.0);
    assert!(rig.client.frame().in_world, "the player is in the world");
    assert_eq!(cell(&rig), START_CELL);
    Some(rig)
}

/// Holds `action` for `seconds` from the start and asserts the body moved more than 2 m.
fn leaves_the_start(action: u32, seconds: f64, what: &str) {
    let Some(mut rig) = start() else {
        return;
    };
    let start = origin(&rig);
    let moved = walk(&mut rig, action, seconds);
    assert!(
        moved > 2.0,
        "{what} moved the body {moved} m from {start:?} to {:?}; near: {:?}",
        origin(&rig),
        near(&rig)
    );
}

/// The body leaves the start more than 2 m forward (west, towards the lifestone), ...
#[test]
fn a_new_character_walks_forward_from_its_start() {
    leaves_the_start(ACTION_FORWARD, 0.7, "walking forward");
}

/// ... and sideways, either way (towards the portal, and away from it).
#[test]
fn a_new_character_strafes_right_from_its_start() {
    leaves_the_start(ACTION_STRAFE_POSITIVE, 1.5, "strafing right");
}

#[test]
fn a_new_character_strafes_left_from_its_start() {
    leaves_the_start(ACTION_STRAFE_NEGATIVE, 1.5, "strafing left");
}

/// The training Idol Spawns stand at their scale (0.75, the weenie's `Scale` quality) and the
/// start is clear of every object.
#[test]
fn the_start_is_clear_and_the_idols_are_at_their_scale() {
    let Some(rig) = start() else {
        return;
    };
    let me = origin(&rig);
    let frame = rig.client.frame();
    let mut idols = 0;
    for object in &frame.objects {
        let entity = rig
            .client
            .session()
            .objects()
            .get(object.id)
            .and_then(|o| o.entity);
        let apart = (object.origin[0] - me[0]).hypot(object.origin[1] - me[1]);
        if !object.in_world || apart > 8.0 {
            continue;
        }
        assert!(
            apart > 1.5,
            "object {:#x} ({entity:x?}) stands {apart} m from the start {me:?}",
            object.id
        );
        if entity == Some(IDOL) {
            idols += 1;
            assert_eq!(
                object.body_scale,
                [0.75, 0.75, 0.75],
                "the Idol {:#x} is drawn at its scale",
                object.id
            );
        }
    }
    assert_eq!(idols, 3, "the three training Idols are in the room");
}

/// The player walks to the lifestone, 4 m west (forward from the start), and uses it: the server
/// binds it, within four metres of the stone.
#[test]
fn the_player_walks_to_the_lifestone_and_binds() {
    let Some(mut rig) = start() else {
        return;
    };
    let stone = object_of(&rig, LIFESTONE).expect("the lifestone");
    let at = place_of(&rig, stone).expect("the lifestone is in the world");
    let start = origin(&rig);
    let gap = walk_towards(&mut rig, ACTION_FORWARD, at, 3.0, 5.0);
    assert!(
        gap <= 3.5,
        "walking forward from {start:?} reached {:?}, {gap} m from the stone at {at:?}; near: {:?}",
        origin(&rig),
        near(&rig)
    );
    // The server judges the range by the position the client last reported.
    rig.run(1.5);
    let from = rig.seen.len();
    rig.command(Command::Use(stone));
    let bound = rig.until(10.0, |r| said(r, from, "bound"));
    assert!(
        bound,
        "the lifestone did not bind; the player at {:?}, the server said {:?}; near: {:?}",
        origin(&rig),
        lines(&rig, from),
        near(&rig)
    );
}

/// The player walks to the portal, 4 m north (a strafe from the start), and uses it: the body
/// leaves the vault.
#[test]
fn the_player_walks_to_the_portal_and_travels() {
    let Some(mut rig) = start() else {
        return;
    };
    let portal = object_of(&rig, PORTAL).expect("the portal");
    let at = place_of(&rig, portal).expect("the portal is in the world");
    // The strafe key that goes north is the one that brings the body nearer.
    let start = origin(&rig);
    let mut gap = walk_towards(&mut rig, ACTION_STRAFE_POSITIVE, at, 3.0, 0.4);
    if planar(origin(&rig), at) >= planar(start, at) {
        gap = walk_towards(&mut rig, ACTION_STRAFE_NEGATIVE, at, 3.0, 5.0);
    } else {
        gap = gap.min(walk_towards(&mut rig, ACTION_STRAFE_POSITIVE, at, 3.0, 5.0));
    }
    assert!(
        gap <= 3.5,
        "strafing from {start:?} reached {:?}, {gap} m from the portal at {at:?}; near: {:?}",
        origin(&rig),
        near(&rig)
    );
    rig.run(1.5);
    let from = rig.seen.len();
    rig.command(Command::Use(portal));
    let left = rig.until(30.0, |r| {
        let frame = r.client.frame();
        frame.in_world && frame.player.as_ref().is_some_and(|p| p.cell != START_CELL)
    });
    assert!(
        left,
        "the portal did not take the player from {START_CELL:08x}; at {:?} in cell {:08x}; the server said {:?}",
        origin(&rig),
        cell(&rig),
        lines(&rig, from)
    );
}

/// The player and the Idol Spawns stand on the vault's floor, not above it: each body is in
/// contact on a walkable surface with no height above it (`CPhysicsObj::GetHeightAboveWalkable`
/// zero, `CPhysicsObj::on_ground`), and stays at one height over several frames (nothing falls,
/// nothing hovers or climbs). The mentor (a Pectar) is not asked: it is a flier that hovers 2 m
/// (`ac2e_live_quests`).
#[test]
fn the_player_and_the_idols_stand_on_the_floor() {
    let Some(mut rig) = start() else {
        return;
    };
    let player = rig.client.frame().player.as_ref().expect("the player").id;
    let idols = |rig: &LiveRig| -> Vec<u64> {
        rig.client
            .frame()
            .objects
            .iter()
            .filter(|o| o.in_world)
            .filter(|o| {
                rig.client
                    .session()
                    .objects()
                    .get(o.id)
                    .and_then(|x| x.entity)
                    == Some(IDOL)
            })
            .filter(|o| {
                let me = origin(rig);
                (o.origin[0] - me[0]).hypot(o.origin[1] - me[1]) < 8.0
            })
            .map(|o| o.id)
            .collect()
    };
    let bodies: Vec<u64> = std::iter::once(player).chain(idols(&rig)).collect();
    assert!(bodies.len() > 1, "the room's Idols are in the world");
    let mut seen: Vec<(u64, f32)> = Vec::new();
    for frame in 0..5 {
        for &id in &bodies {
            let footing = rig.client.footing(id).expect("the body is in the world");
            let z = if id == player {
                origin(&rig)[2]
            } else {
                place_of(&rig, id).expect("in the world")[2]
            };
            assert!(
                footing.surface_found && footing.on_ground && footing.height.abs() < 0.05,
                "body {id:#x} at z {z} does not stand on the floor at sample {frame}: {footing:?}"
            );
            match seen.iter().find(|(seen_id, _)| *seen_id == id) {
                Some((_, first)) => assert!(
                    (z - first).abs() < 0.01,
                    "body {id:#x} moved from z {first} to {z} while standing"
                ),
                None => seen.push((id, z)),
            }
        }
        rig.run(0.4);
    }
}
