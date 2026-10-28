//! The client's conversion of the server's motion events (`ObjectAction` into `MotionEvent`,
//! the behaviour bitfield, the `MoveTo` unpacking, the `PhysicsDesc` animation) and the route
//! a `MoveTo` takes to the player alone. The body's own response is tested in `ac2-physics`.

use std::sync::Arc;

use ac2_net::protocol::{BehaviorParams, MoveTarget, MovementParameters, ObjectAction};
use ac2_physics::{Frame, MotionEvent, PhysicsBody, Position, Quaternion, Vector};

use super::{apply, behavior, describe, event, movement_parameters};
use crate::world_data::{DatWorld, WorldData};

/// The human visual (the double's player).
const HUMAN: u32 = 0x1F00_0023;
const OTHER_MODE: u32 = 0x4100_0009;
/// The `dead` mode and the `death` behaviour.
const DEAD: u32 = 0x4100_0005;
const DEATH: u32 = 0x4100_001F;
/// Behaviours of the peace mode on the ground.
const WAVE: u32 = 0x4000_000A;
/// A network slider.
const NETWORK_SLIDER: u32 = 0x4000_000A;

fn at(cell: u32, origin: [f32; 3]) -> Position {
    Position::new(
        cell,
        Frame::new(
            Vector::new(origin[0], origin[1], origin[2]),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        ),
    )
}

/// A human body with its behaviour table, the player's or another object's.
fn human(player: bool) -> Option<PhysicsBody> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let world = DatWorld::new(Arc::new(ac2_db::ResourceStore::new(dataset))).unwrap();
    let data = world.body(Some(HUMAN)).unwrap();
    let mut body = PhysicsBody::new(
        at(0x0299_0109, [59.0, 240.0, -44.5]),
        data.body.shape,
        data.body.motion,
    );
    body.set_player(player);
    body.set_radius(data.body.radius);
    body.set_animation(data.body.animation.expect("the human animates"));
    Some(body)
}

fn animate(body: &mut PhysicsBody, seconds: f32) {
    let mut random = ac2_core::Random::default();
    for step in 0..(seconds * 20.0) as u32 {
        body.update_animation(0.05, 1.0 + f64::from(step) * 0.05, &mut random);
    }
}

fn params(bitfield: u32, behavior: u32, mode: u32) -> Box<BehaviorParams> {
    Box::new(BehaviorParams {
        bitfield,
        behavior,
        mode,
        ..BehaviorParams::default()
    })
}

/// `BehaviorParams::StreamPack`: the bitfield names the fields it sends, the rest keep their
/// defaults, and the copy runs in context 1.
#[test]
fn the_behaviour_bitfield_names_the_fields_it_sends() {
    let named = behavior(&BehaviorParams {
        bitfield: 0x1 | 0x2 | 0x800,
        behavior: DEATH,
        mode: DEAD,
        lock_actions: 1,
        ..BehaviorParams::default()
    });
    assert_eq!(named.behavior, DEATH);
    assert_eq!(named.mode, DEAD);
    assert!(named.lock_actions);
    assert_eq!(named.context, 1);
    let bare = behavior(&BehaviorParams {
        bitfield: 0,
        behavior: DEATH,
        ..BehaviorParams::default()
    });
    assert_ne!(bare.behavior, DEATH, "no bit 1, no behaviour");
    assert_eq!(bare.mode, behavior(&BehaviorParams::default()).mode);
}

/// The camera and child-fade parameters travel with the behaviour when their bits are set:
/// `mFadeChildren` 0x100, `mCameraParent` 0x80, `mCameraTarget` 0x40, `mCameraHold` 0x40000,
/// `mCameraRestore` 0x80000, and `mCameraTargetID` with `mCameraBehavior` 0x200000.
#[test]
fn the_behaviour_bitfield_carries_the_camera_and_fade_parameters() {
    let named = behavior(&BehaviorParams {
        bitfield: 0x100 | 0x80 | 0x40 | 0x4_0000 | 0x8_0000 | 0x20_0000,
        fade_children: 1,
        camera_parent: 1,
        camera_target_flag: 1,
        camera_hold: 1,
        camera_restore: 1,
        camera_target: 0x77,
        camera_behavior: 0x4000_0099,
        ..BehaviorParams::default()
    });
    assert!(named.fade_children && named.camera_parent && named.camera_target);
    assert!(named.camera_hold && named.camera_restore);
    assert_eq!(
        (named.camera_target_id, named.camera_behavior),
        (0x77, 0x4000_0099)
    );
    let bare = behavior(&BehaviorParams {
        bitfield: 0,
        fade_children: 1,
        camera_target: 0x77,
        ..BehaviorParams::default()
    });
    assert!(!bare.fade_children && bare.camera_target_id == 0);
}

/// The conversion keeps the motion actions and drops the rest; a targeted FX plays its
/// untargeted sound branch.
#[test]
fn the_server_actions_convert_to_their_motion_events() {
    let Some(MotionEvent::DoBehavior(converted)) =
        event(&ObjectAction::DoBehavior(params(3, DEATH, DEAD)))
    else {
        panic!("DoBehavior converts to a behaviour event");
    };
    assert_eq!((converted.behavior, converted.mode), (DEATH, DEAD));
    assert_eq!(
        event(&ObjectAction::DoTargettedFx {
            fx: 0x4000_0010,
            scalar: 0.5,
            target: 9,
        }),
        Some(MotionEvent::DoFx {
            fx: 0x4000_0010,
            scalar: 0.5,
        })
    );
    assert_eq!(
        event(&ObjectAction::AddTargetedImpulse {
            target: 9,
            magnitude: 2.0,
            falloff: 1,
            vertical: 0.0,
        }),
        Some(MotionEvent::AddTargetedImpulse {
            magnitude: 2.0,
            falloff: true,
            vertical: 0.0,
        })
    );
    assert_eq!(event(&ObjectAction::LeaveWorld(0)), None);
    assert_eq!(
        event(&ObjectAction::AddTerrainModifier {
            terrain: 0x1234,
            modifier: 9,
        }),
        Some(MotionEvent::AddTerrainModifier {
            terrain: 0x1234,
            modifier: 9,
        })
    );
    assert_eq!(
        event(&ObjectAction::ClearTerrainModifiers),
        Some(MotionEvent::ClearTerrainModifiers)
    );
}

#[test]
fn a_physics_desc_sets_the_mode_behaviours_and_sliders_of_a_new_body() {
    let Some(mut body) = human(false) else {
        return;
    };
    describe(
        &mut body,
        OTHER_MODE,
        &[*params(1, WAVE, 0)],
        &[(NETWORK_SLIDER, 0.5, 0.0)],
    );
    animate(&mut body, 5.0);
    assert_eq!(body.mode(), OTHER_MODE);
    assert_eq!(body.slider_value(NETWORK_SLIDER), 0.5);
}

fn move_to(target: MoveTarget) -> ObjectAction {
    ObjectAction::MoveTo {
        params: Box::new(MovementParameters {
            flags: 1 | 0x40,
            desired_velocity: Some(1.0),
            fail_distance: Some(50.0),
            desired_distance: Some(0.5),
            max_success_distance: Some(1.0),
            min_success_distance: None,
            target,
            offset: [0.0; 3],
            offset_id: 0,
            positions: Vec::new(),
            context_id: 7,
        }),
        stamp: 3,
    }
}

#[test]
fn only_the_player_moves_to() {
    let (Some(mut remote), Some(mut player)) = (human(false), human(true)) else {
        return;
    };
    let target = MoveTarget::Position(ac2_net::protocol::Position {
        cell: 0x0299_0109,
        origin: [59.0, 244.0, -44.5],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    assert!(!apply(&mut remote, &move_to(target)));
    assert!(!remote.is_moving_to());
    assert!(apply(&mut player, &move_to(target)));
    assert!(player.is_moving_to());
}

#[test]
fn a_server_move_to_unpacks_into_the_physics_parameters() {
    let ObjectAction::MoveTo { params, .. } = move_to(MoveTarget::Object(0x7000_0000_0000_0042))
    else {
        unreachable!();
    };
    let unpacked = movement_parameters(&params);
    assert_eq!(unpacked.flags, 0x41, "the flags as sent");
    assert_eq!(unpacked.target_object, Some(0x7000_0000_0000_0042));
    assert_eq!(
        (
            unpacked.desired_velocity,
            unpacked.fail_distance,
            unpacked.desired_distance,
            unpacked.max_success_distance,
            unpacked.min_success_distance,
        ),
        (1.0, 50.0, 0.5, 1.0, 100.0),
        "the minimum is not sent: its default stays"
    );
    assert_eq!(unpacked.context, 7);
}
