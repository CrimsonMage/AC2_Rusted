use ac2_physics::{Frame, Position, Quaternion, Vector};

use super::*;
use crate::camera_fx::CameraFx;
use crate::targeting::{CameraObjects, CAMERA_TARGET_BONE};

const PLAYER: u64 = 0x5000_0001;
const NPC: u64 = 0x7000_0042;

fn at(x: f32, y: f32, z: f32) -> Position {
    Position::new(
        0x7F7F_001C,
        Frame::new(Vector::new(x, y, z), Quaternion::new(1.0, 0.0, 0.0, 0.0)),
    )
}

fn subject(id: u64, position: Position) -> CameraSubject {
    CameraSubject {
        id,
        position,
        height: 1.8,
        radius: 0.4,
        scale_z: 1.0,
        local_velocity: Vector::new(0.0, 0.0, 0.0),
        element: 0,
        no_draw: false,
    }
}

/// An NPC 10 m north of the player, with a bone type 0x40000001 at its head; the player has the
/// camera target bone 2 m above it.
struct World;

impl CameraObjects for World {
    fn subject(&self, id: u64) -> Option<CameraSubject> {
        (id == NPC).then(|| subject(NPC, at(70.0, 80.0, 50.0)))
    }
    fn bone_position(&self, id: u64, bone: u32, offset: Vector) -> Option<Position> {
        match (id, bone) {
            (NPC, 0x4000_0001) => Some(at(70.0, 80.0, 51.7)),
            (NPC, 0x4000_0002) => Some(at(70.0, 82.0, 51.7)),
            (PLAYER, CAMERA_TARGET_BONE) => {
                Some(at(70.0 + offset.x, 70.0 + offset.y, 52.0 + offset.z))
            }
            _ => None,
        }
    }
}

fn rig() -> (CameraManager, CameraSubject) {
    let player = subject(PLAYER, at(70.0, 70.0, 50.0));
    let mut camera = CameraManager::new();
    camera.set_player(&player, 0.75);
    (camera, player)
}

fn step(camera: &mut CameraManager, player: &CameraSubject, quantum: f32) -> Position {
    camera.update_camera_with(quantum, Some(player), &World, &player.position)
}

#[test]
fn a_targeted_camera_looks_from_the_npcs_bone_and_returns_to_the_player() {
    let (mut camera, player) = rig();
    for _ in 0..30 {
        step(&mut camera, &player, 0.1);
    }
    let behind = step(&mut camera, &player, 0.1);
    assert!(behind.frame.origin.y < 70.0, "{behind:?}");
    let view = CameraView {
        player: Some(&player),
        objects: &World,
    };
    camera.set_targeted_camera(&view, NPC, (0x4000_0002, 0x4000_0001), 2.0, 0.5);
    assert!(camera.no_ray_trace());
    assert!(camera.check_env_only());
    for _ in 0..10 {
        step(&mut camera, &player, 0.1);
    }
    // At the parent bone lifted by the NPC's height (retail adds it to a bone too), looking
    // from there at the target bone, south and down.
    let shot = step(&mut camera, &player, 0.1);
    assert_eq!(shot.frame.origin, Vector::new(70.0, 82.0, 53.5));
    let look = shot.frame.vector_heading();
    assert!(look.y < -0.7 && look.z < -0.6, "{look:?}");
    // Past the period less the ramp, back to the player over 2 s.
    for _ in 0..40 {
        step(&mut camera, &player, 0.1);
    }
    assert!(camera.targeting.target_is_player());
    assert_eq!(camera.targeting.parent_id, INVALID_IID);
    let back = step(&mut camera, &player, 0.1);
    assert!((back.frame.origin.y - behind.frame.origin.y).abs() < 1e-3);
}

#[test]
fn a_held_anim_target_stays_until_restored() {
    let (mut camera, player) = rig();
    let view = CameraView {
        player: Some(&player),
        objects: &World,
    };
    camera.set_camera_anim_target(&view, -1.0, 0.0);
    assert_eq!(camera.targeting.target_id, CAMERA_TARGET_ID);
    for _ in 0..50 {
        step(&mut camera, &player, 0.1);
    }
    assert_eq!(camera.targeting.target_id, CAMERA_TARGET_ID, "held");
    camera.restore_player_camera(&view);
    assert!(camera.targeting.target_is_player());
    // With no camera parent bone on the player, no parent dummy: nothing moves.
    camera.set_camera_anim_parent(&view, 1.0, 0.0);
    assert_eq!(camera.targeting.parent_id, INVALID_IID);
}

#[test]
fn camera_fx_for_the_player_only_ignore_other_sources() {
    let (mut camera, player) = rig();
    let view = CameraView {
        player: Some(&player),
        objects: &World,
    };
    let fx = CameraFx {
        shake: true,
        has_fov: true,
        player_only: true,
        intensity: 20.0,
        ..CameraFx::default()
    };
    camera.do_camera_fx(&fx, 1.0, NPC, &view);
    assert!(camera.fx.tracks.is_empty());
    camera.do_camera_fx(&fx, 1.0, PLAYER, &view);
    assert_eq!(camera.fx.tracks.len(), 2);
    let mut current = player.position;
    let mut roll = |_: f32, _: f32| 2.3;
    camera.use_time(10.0, &mut current, &mut |_| Some(0.0), &mut roll);
    assert_eq!(camera.fov(), 75.0, "the first call runs no time: 65 + 10");
    camera.use_time(10.5, &mut current, &mut |_| Some(0.0), &mut roll);
    assert_eq!(camera.fov(), 85.0);
    assert_ne!(current.frame.origin, player.position.frame.origin, "shaken");
    camera.stop_all_camera_fx();
    camera.use_time(10.6, &mut current, &mut |_| Some(0.0), &mut roll);
    assert_eq!(camera.fov(), 75.0);
}

#[test]
fn a_dynamic_first_person_camera_rides_the_head_bone() {
    let (mut camera, player) = rig();
    camera.set_first_person(true);
    camera.dynamic_first_person = true;
    let at_head = step(&mut camera, &player, 0.1);
    assert_eq!(at_head.frame.origin, Vector::new(70.0, 70.2, 52.2));
}

/// With the player as the camera's parent and an NPC as its target, the camera stands back from
/// the player along the line from the NPC, and the player's head turns to the NPC.
#[test]
fn a_player_parent_turns_the_players_head_to_the_target() {
    let (mut camera, player) = rig();
    let view = CameraView {
        player: Some(&player),
        objects: &World,
    };
    camera.set_parent_object(&view, PLAYER, -1.0, 0.0);
    camera.set_target_object(&view, NPC, -1.0, 0.0);
    for _ in 0..5 {
        step(&mut camera, &player, 0.1);
    }
    assert_eq!(camera.targeting.take_look_at(), Some(NPC));
    assert_eq!(camera.targeting.take_look_at(), None, "taken once");
    let shot = step(&mut camera, &player, 0.1);
    // Behind the player (south of it), looking north towards the NPC.
    assert!(shot.frame.origin.y < 70.0, "{shot:?}");
    assert!(shot.frame.vector_heading().y > 0.9, "{shot:?}");
}
