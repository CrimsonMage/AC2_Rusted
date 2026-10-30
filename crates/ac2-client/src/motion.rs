//! The server's motion events as the client reads them: the `ObjectAction` conversion into the
//! physics `MotionEvent`s (`ac2_physics::apply_motion`), the behaviour, move-to and position
//! unpacking, and the `PhysicsDesc` animation.

use ac2_anim::Ac2BehaviorParams;
use ac2_net::protocol::{BehaviorParams, MoveTarget, MovementParameters, ObjectAction};
#[cfg(test)]
use ac2_physics::apply_motion;
use ac2_physics::{
    apply_motion_owned, DescSlider, Frame, MotionEvent, PackedMovementParameters, PhysicsBody,
    Position, Quaternion, Vector,
};

/// `BehaviorParams::StreamPack` over `BehaviorParams::SetDefaults`: the fields the pack bitfield
/// names, the rest at their defaults.
pub(crate) fn behavior(params: &BehaviorParams) -> Ac2BehaviorParams {
    let has = |bit: u32| params.bitfield & bit != 0;
    let mut out = Ac2BehaviorParams::new(if has(0x1) { params.behavior } else { 0 });
    if has(0x2) {
        out.mode = params.mode;
    }
    if has(0x4) {
        out.time_scale = params.time_scale;
    }
    if has(0x10) {
        out.hold_cycles = params.hold_cycles;
    }
    if has(0x10_0000) {
        out.fx = params.fx;
    }
    let flag = |bit: u32, value: u32, default: bool| if has(bit) { value != 0 } else { default };
    out.early_callback = flag(0x8, params.early_callback, out.early_callback);
    out.move_to_cancels = flag(0x20, params.moveto_cancels, out.move_to_cancels);
    out.lock_actions = flag(0x800, params.lock_actions, out.lock_actions);
    out.send_event = flag(0x200, params.send_event, out.send_event);
    out.leave_idle_alone = flag(0x2_0000, params.leave_idle_alone, out.leave_idle_alone);
    out.destroy_on_completion = flag(
        0x8000,
        params.destroy_on_completion,
        out.destroy_on_completion,
    );
    out.fade_children = flag(0x100, params.fade_children, out.fade_children);
    out.camera_parent = flag(0x80, params.camera_parent, out.camera_parent);
    out.camera_target = flag(0x40, params.camera_target_flag, out.camera_target);
    out.camera_hold = flag(0x4_0000, params.camera_hold, out.camera_hold);
    out.camera_restore = flag(0x8_0000, params.camera_restore, out.camera_restore);
    if has(0x20_0000) {
        out.camera_target_id = params.camera_target;
        out.camera_behavior = params.camera_behavior;
    }
    if has(0x2000) {
        out.target = params.target;
    }
    if has(0x1_0000) {
        out.vdesc_to_clone = params.vdesc_to_clone;
        out.cloned_apr = params.cloned_apr;
        out.cloned_app_apr = params.cloned_app_apr.clone();
    }
    // `RecvEvt_DoBehavior` runs a copy whose context is 1.
    out.context = 1;
    out
}

/// A `Position` as it travels: cell, origin and rotation (w, x, y, z).
fn position(p: &ac2_net::protocol::Position) -> Position {
    let [w, x, y, z] = p.rotation;
    Position::new(
        p.cell,
        Frame::new(
            Vector::new(p.origin[0], p.origin[1], p.origin[2]),
            Quaternion::new(w, x, y, z),
        ),
    )
}

/// A server `TargetPosition` (`TargetPosition::StreamPack`) as the missile manager holds it.
pub(crate) fn target_position(
    t: &ac2_net::protocol::TargetPosition,
) -> ac2_physics::TargetPosition {
    ac2_physics::TargetPosition {
        target_id: t.target,
        offset_pos: position(&t.offset),
        target_velocity: Vector::new(t.velocity[0], t.velocity[1], t.velocity[2]),
        initialized: t.initialized != 0,
    }
}

/// `MovementParameters` from a server `MoveTo` (`MovementParameters::StreamPack` over the
/// defaults).
pub(crate) fn movement_parameters(p: &MovementParameters) -> ac2_physics::MovementParameters {
    let (target_object, target_position) = match &p.target {
        MoveTarget::Object(id) => (Some(*id), None),
        MoveTarget::Position(at) => (None, Some(position(at))),
    };
    ac2_physics::MovementParameters::unpack(PackedMovementParameters {
        flags: p.flags,
        desired_velocity: p.desired_velocity,
        fail_distance: p.fail_distance,
        desired_distance: p.desired_distance,
        max_success_distance: p.max_success_distance,
        min_success_distance: p.min_success_distance,
        target_object,
        target_position,
        offset: p.offset,
        offset_id: p.offset_id,
        positions: p.positions.iter().map(position).collect(),
        context: p.context_id,
    })
}

/// The animation half of a `PhysicsDesc` (`CPhysicsObj::set_description`): its mode (0 when
/// the description has none), behaviours and sliders.
pub(crate) fn describe(
    body: &mut PhysicsBody,
    mode: u32,
    behaviors: &[BehaviorParams],
    sliders: &[(u32, f32, f32)],
) {
    let behaviors: Vec<Ac2BehaviorParams> = behaviors.iter().map(behavior).collect();
    let sliders: Vec<DescSlider> = sliders
        .iter()
        .map(|&(slider, value, velocity)| DescSlider {
            slider,
            value,
            velocity,
        })
        .collect();
    body.set_description_animation(mode, &behaviors, &sliders);
}

/// The physics event an `ObjectAction` carries (`CPhysicsObj::RecvEvt_*`); `None` for the
/// actions a body does not play. `DoTargettedFx` plays its untargeted sound branch
/// (`FXManager::DoTargettedFX`): the target only steers the visual part.
pub(crate) fn event(action: &ObjectAction) -> Option<MotionEvent> {
    Some(match action {
        ObjectAction::DoMode(mode) => MotionEvent::DoMode(*mode),
        ObjectAction::SetMode(mode) => MotionEvent::SetMode(*mode),
        ObjectAction::StopLayer(layer) => MotionEvent::StopLayer(*layer),
        ObjectAction::DoBehavior(params) => MotionEvent::DoBehavior(behavior(params)),
        ObjectAction::StopBehavior(id) => MotionEvent::StopBehavior(*id),
        ObjectAction::ReleaseBehavior {
            behavior: id,
            params,
        } => MotionEvent::ReleaseBehavior {
            behavior: *id,
            params: behavior(params),
        },
        ObjectAction::StopAllWeenieBehaviors => MotionEvent::StopAllWeenieBehaviors,
        ObjectAction::StopMoveToCancelsBehaviors => MotionEvent::StopMoveToCancelsBehaviors,
        ObjectAction::DoSlider {
            slider,
            value,
            time,
        } => MotionEvent::DoSlider {
            slider: *slider,
            value: *value,
            time: *time,
        },
        ObjectAction::MoveTo { params, .. } => {
            MotionEvent::MoveTo(Box::new(movement_parameters(params)))
        }
        ObjectAction::AddImpulse { direction, .. } => MotionEvent::AddImpulse {
            direction: *direction,
        },
        ObjectAction::AddTargetedImpulse {
            magnitude,
            falloff,
            vertical,
            ..
        } => MotionEvent::AddTargetedImpulse {
            magnitude: *magnitude,
            falloff: *falloff != 0,
            vertical: *vertical,
        },
        ObjectAction::SetVelocityScale(scale) => MotionEvent::SetVelocityScale(*scale),
        ObjectAction::SetAccelerationScale(scale) => MotionEvent::SetAccelerationScale(*scale),
        ObjectAction::SetJumpScale(scale) => MotionEvent::SetJumpScale(*scale),
        ObjectAction::SetHoverHeight(height) => MotionEvent::SetHoverHeight(*height),
        ObjectAction::LookAt(target) => MotionEvent::LookAt(*target),
        ObjectAction::LookAtDir { x, y } => MotionEvent::LookAtDir { x: *x, y: *y },
        ObjectAction::SetGlobalModifier(id) => MotionEvent::SetGlobalModifier(*id),
        ObjectAction::AddTerrainModifier { terrain, modifier } => MotionEvent::AddTerrainModifier {
            terrain: *terrain,
            modifier: *modifier,
        },
        ObjectAction::ClearTerrainModifiers => MotionEvent::ClearTerrainModifiers,
        ObjectAction::DoFxScript(id) => MotionEvent::DoFxScript(*id),
        ObjectAction::DoFx { fx, scalar } | ObjectAction::DoTargettedFx { fx, scalar, .. } => {
            MotionEvent::DoFx {
                fx: *fx,
                scalar: *scalar,
            }
        }
        ObjectAction::StopFx(fx) => MotionEvent::StopFx(*fx),
        _ => return None,
    })
}

/// [`ac2_physics::apply_motion`] for the server's action; a non-motion action leaves the body
/// alone.
#[cfg(test)]
pub(crate) fn apply(body: &mut PhysicsBody, action: &ObjectAction) -> bool {
    event(action).is_some_and(|event| apply_motion(body, &event))
}

/// [`ac2_physics::apply_motion_owned`] for the server's action.
pub(crate) fn apply_owned(
    body: &mut PhysicsBody,
    own: u64,
    action: &ObjectAction,
    source: Option<Position>,
) -> bool {
    event(action).is_some_and(|event| apply_motion_owned(body, own, &event, source))
}

#[cfg(test)]
#[path = "motion_tests.rs"]
mod tests;
