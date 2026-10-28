//! A missile through `Remotes::step_quantum`: spawned, flying, detonating on the world, fading
//! and removed.

use ac2_core::Random;
use ac2_physics::{
    BodyShape, Frame, MissileEvent, PhysicsBody, Position, TargetPosition, Terrain, Vector,
};

use crate::missile_follow::FADE_SLIDER;
use crate::remotes::{Remote, Remotes};

/// A landblock cell 70 m in from the block's corner on both axes.
const CELL: u32 = 0x7F7F_001C;
const BLOCK: u32 = 0x7F7F_FFFF;
const MISSILE: u64 = 0x7000_0000_0000_0021;

fn flat_ground(height: f32) -> Terrain {
    let mut terrain = Terrain::new();
    terrain.insert_heights(BLOCK, &[height; 17 * 17], |_, _| false);
    terrain
}

fn missile_remote(z: f32) -> Remote {
    let shape = BodyShape {
        spheres: vec![ac2_physics::Sphere::new(Vector::new(0.0, 0.0, 0.2), 0.2)],
        height: 0.4,
        step_up_height: 0.1,
        step_down_height: 0.1,
        scale: 1.0,
        allow_free_heading: false,
    };
    let at = Position::new(
        CELL,
        Frame::new(
            Vector::new(70.0, 70.0, z),
            ac2_physics::Quaternion::IDENTITY,
        ),
    );
    Remote {
        visual: None,
        body: Some(PhysicsBody::new(at, shape, None)),
        failed: false,
        stored: None,
        in_world: true,
        lost_cell: None,
        active: true,
        player_distance: 0.0,
        object: None,
        shadow: None,
        ethereal: false,
        alpha_moves: None,
        fx_follow: None,
        listed_cell: None,
    }
}

#[test]
fn a_missile_flies_hits_the_ground_fades_and_is_removed() {
    let mut terrain = flat_ground(50.0);
    let mut remotes = Remotes::<()>::default();
    remotes.insert(MISSILE, missile_remote(52.0));
    let mut random = Random::default();
    let mut now = 10.0;

    // Spawned: the server's launch velocity and a fixed point on the ground ahead.
    let aim = Position::new(
        CELL,
        Frame::new(
            Vector::new(70.0, 90.0, 50.0),
            ac2_physics::Quaternion::IDENTITY,
        ),
    );
    let target = TargetPosition {
        target_id: 0,
        offset_pos: aim,
        target_velocity: Vector::ZERO,
        initialized: true,
    };
    let body = remotes.body_mut(MISSILE).expect("body");
    body.set_time(now);
    body.recv_initialize_missile(&terrain, [0.0, 20.0, -4.0], &target);
    assert!(body.has_missile_manager());
    let start = body.position().frame.origin;
    let launch: Vec<_> = remotes
        .body_mut(MISSILE)
        .map(|b| b.take_missile_events())
        .unwrap_or_default()
        .into_iter()
        .map(|e| (MISSILE, e))
        .collect();
    let first = remotes.follow_missiles(&launch);
    assert_eq!(first.streaks, vec![(MISSILE, true)], "the streak starts");

    // Flies, then hits: each quantum's events are followed until the delete is asked for.
    let mut follow = None;
    let mut flown = start;
    for _ in 0..80 {
        now += 0.1;
        let events = remotes.step_quantum(&mut terrain, now, &mut random, &[]);
        flown = remotes
            .get(MISSILE)
            .and_then(|r| r.body())
            .map_or(flown, |b| b.position().frame.origin);
        let done = remotes.follow_missiles(&events);
        if !done.destroy.is_empty() {
            follow = Some(done);
            break;
        }
    }
    let follow = follow.expect("the missile detonated");
    assert!(flown.y > start.y + 5.0, "it flew: {start:?} -> {flown:?}");
    assert!(
        follow.streaks.contains(&(MISSILE, false)),
        "the streak stops"
    );
    assert_eq!(
        follow.destroy,
        vec![(MISSILE, 0.5)],
        "DeleteSelf(0.5, 0): destroyed when the half second fade ends"
    );
    assert!(follow.stick.is_empty());

    // Fading: the opacity slider moves toward clear.
    let body = remotes.body_mut(MISSILE).expect("body");
    assert!(
        body.do_slider(0, FADE_SLIDER, 0.0, 0.5),
        "the fade slider exists"
    );

    // Removed: the object maintenance deletes it when the time has come.
    remotes.removed(MISSILE, &mut terrain);
    assert!(!remotes.tracks(MISSILE));
    let _ = MissileEvent::Streak { on: true };
}
