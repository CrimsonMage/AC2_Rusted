//! The server's motion events on a human body from the real DATs (`AC2_DAT_DIR`; skipped
//! without): what each `CPhysicsObj::RecvEvt_*` does to the body, and the `MoveTo` unpacking.

use std::sync::Arc;

use ac2_net::protocol::{BehaviorParams, MoveTarget, MovementParameters, ObjectAction};
use ac2_physics::{Frame, PhysicsBody, Position, Quaternion, Vector};

use super::{apply, describe, movement_parameters};
use crate::world_data::{DatWorld, WorldData};

/// The human visual (the double's player).
const HUMAN: u32 = 0x1F00_0023;
/// `MODE_DEFAULT` (peace) and a ground mode its behaviour table can switch to.
const PEACE: u32 = 0x4000_0001;
const OTHER_MODE: u32 = 0x4100_0009;
/// Behaviours of the peace mode on the ground.
const WAVE: u32 = 0x4000_000A;
const OTHER_BEHAVIOR: u32 = 0x4000_000B;
/// A slider the human's `CharTemplate` leaves to the client, and a network one.
const CLIENT_SLIDER: u32 = 0x4000_0001;
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
        data.shape,
        data.motion,
    );
    body.set_player(player);
    body.set_radius(data.radius);
    body.set_animation(data.animation.expect("the human animates"));
    Some(body)
}

fn animate(body: &mut PhysicsBody, seconds: f32) {
    for step in 0..(seconds * 20.0) as u32 {
        body.update_animation(0.05, 1.0 + f64::from(step) * 0.05);
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

#[test]
fn modes_behaviours_and_layers_reach_the_body() {
    let Some(mut body) = human(false) else {
        return;
    };
    assert_eq!(body.mode(), PEACE);
    // `RecvEvt_SetMode`: at once.
    apply(&mut body, &ObjectAction::SetMode(OTHER_MODE));
    assert_eq!(body.mode(), OTHER_MODE);
    // `RecvEvt_DoMode` -> `DoMode(0, mode)`: back to peace through the table's transitions.
    apply(&mut body, &ObjectAction::DoMode(PEACE));
    animate(&mut body, 5.0);
    assert_eq!(body.mode(), PEACE);
    // `RecvEvt_DoBehavior` (bit 1: the behaviour), then `RecvEvt_StopBehavior`.
    apply(&mut body, &ObjectAction::DoBehavior(params(1, WAVE, 0)));
    assert!(body.is_doing_behavior(WAVE));
    apply(&mut body, &ObjectAction::StopBehavior(WAVE));
    animate(&mut body, 1.0);
    assert!(!body.is_doing_behavior(WAVE));
    // Bit 2 names the mode the behaviour enters first (`BehaviorParams::mModeID`).
    apply(
        &mut body,
        &ObjectAction::DoBehavior(params(3, OTHER_BEHAVIOR, PEACE)),
    );
    assert!(body.is_doing_behavior(OTHER_BEHAVIOR));
    apply(&mut body, &ObjectAction::StopAllWeenieBehaviors);
    apply(&mut body, &ObjectAction::StopMoveToCancelsBehaviors);
    apply(&mut body, &ObjectAction::StopLayer(0));
    animate(&mut body, 1.0);
    assert_eq!(body.mode(), PEACE);
}

#[test]
fn the_player_keeps_its_client_sliders_and_others_take_every_slider() {
    let (Some(mut remote), Some(mut player)) = (human(false), human(true)) else {
        return;
    };
    let slide = |slider| ObjectAction::DoSlider {
        slider,
        value: 0.75,
        time: 0.0,
    };
    for body in [&mut remote, &mut player] {
        apply(body, &slide(CLIENT_SLIDER));
        apply(body, &slide(NETWORK_SLIDER));
    }
    assert_eq!(remote.slider_desired_value(CLIENT_SLIDER), 0.75);
    assert_eq!(remote.slider_desired_value(NETWORK_SLIDER), 0.75);
    assert_ne!(player.slider_desired_value(CLIENT_SLIDER), 0.75);
    assert_eq!(player.slider_desired_value(NETWORK_SLIDER), 0.75);
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
    assert!(!apply(&mut remote, &move_to(target.clone())));
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
