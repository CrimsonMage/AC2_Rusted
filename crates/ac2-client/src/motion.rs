//! The server's motion events for a body (`CPhysicsObj::RecvEvt_DoMode`, `RecvEvt_SetMode`,
//! the behaviour, layer and slider events, `RecvEvt_MoveTo`) and the animated part pose as
//! plain data.

use ac2_anim::Ac2BehaviorParams;
use ac2_net::protocol::{BehaviorParams, MoveTarget, MovementParameters, ObjectAction};
use ac2_physics::{
    DescSlider, Frame, PackedMovementParameters, PhysicsBody, Position, Quaternion, Vector,
};

/// One Setup bone as the animation left it (`CPartArray::m_boneTransforms`): offset, rotation
/// (w, x, y, z) and scale relative to its parent bone (AC axes). The renderer builds the bone and
/// deform matrices from these (`ac2_render::object::ObjectModel::pose`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartPose {
    pub offset: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl PartPose {
    /// As the `TransformBase` the renderer poses a model with (`ObjectModel::pose`).
    pub fn transform(&self) -> ac2_anim::Ac2Transform {
        let [w, x, y, z] = self.rotation;
        ac2_anim::Ac2Transform {
            offset: self.offset.into(),
            scale: self.scale.into(),
            rotation: ac2_math::Quat::from_xyzw(x, y, z, w),
        }
    }
}

/// Where the body is drawn: `PhysicsBody::drawing_position` at the frame's render interpolation
/// (`CPhysics::sRenderInterpolation`), as the cell and the origin and rotation (w, x, y, z).
pub(crate) fn drawn_place(
    body: &PhysicsBody,
    render_interpolation: f32,
) -> (u32, [f32; 3], [f32; 4]) {
    let place = body.drawing_position(render_interpolation);
    let o = place.frame.origin;
    let q = place.frame.q;
    (place.cell_id, [o.x, o.y, o.z], [q.w, q.x, q.y, q.z])
}

/// The body's bones as the renderer poses them: blended at `drawn_at` (the frame's time) when
/// pose blending is on, else the retail refresh (`PhysicsBody::bone_transforms`).
pub(crate) fn pose(body: &PhysicsBody, drawn_at: Option<f64>) -> Vec<PartPose> {
    let bones = match drawn_at {
        Some(now) => body.blended_bone_transforms(now),
        None => body.bone_transforms(),
    };
    bones
        .iter()
        .map(|t| PartPose {
            offset: [t.offset.x, t.offset.y, t.offset.z],
            rotation: [t.rotation.w, t.rotation.x, t.rotation.y, t.rotation.z],
            scale: [t.scale.x, t.scale.y, t.scale.z],
        })
        .collect()
}

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

/// The motion events a body plays (`CPhysicsObj::RecvEvt_*`): `DoMode` is `DoMode(0, mode)`;
/// `DoBehavior` and `ReleaseBehavior` run copies whose context is 1 (a weenie emote is not run
/// first); `DoSlider` leaves the player's client-controlled sliders alone; `MoveTo` moves the
/// player only; the velocity, acceleration and jump scales and the hover height set the body's
/// tuning; the FX events start and stop the effect's sounds. Returns whether a move-to started.
pub(crate) fn apply(body: &mut PhysicsBody, action: &ObjectAction) -> bool {
    match action {
        ObjectAction::DoMode(mode) => {
            body.do_mode(0, *mode);
        }
        ObjectAction::SetMode(mode) => {
            body.set_mode(*mode);
        }
        ObjectAction::StopLayer(layer) => body.stop_layer(*layer),
        ObjectAction::DoBehavior(params) => {
            body.do_behavior(&behavior(params));
        }
        ObjectAction::StopBehavior(id) => body.stop_behavior(*id),
        ObjectAction::ReleaseBehavior {
            behavior: id,
            params,
        } => {
            body.release_behavior(*id, &behavior(params));
        }
        ObjectAction::StopAllWeenieBehaviors => {
            body.stop_all_weenie_behaviors();
        }
        ObjectAction::StopMoveToCancelsBehaviors => body.stop_move_to_cancels_behaviors(),
        ObjectAction::DoSlider {
            slider,
            value,
            time,
        } => {
            body.recv_do_slider(*slider, *value, *time);
        }
        ObjectAction::MoveTo { params, .. } => {
            return body.recv_move_to(movement_parameters(params));
        }
        // `CPhysicsObj::RecvEvt_AddImpulse`: knockback in world axes.
        ObjectAction::AddImpulse { direction, .. } => body.add_impulse(*direction),
        ObjectAction::SetVelocityScale(scale) => body.set_velocity_scale(*scale),
        ObjectAction::SetAccelerationScale(scale) => body.set_acceleration_scale(*scale),
        ObjectAction::SetJumpScale(scale) => body.set_jump_scale(*scale),
        ObjectAction::SetHoverHeight(height) => body.set_hover_height(*height),
        // `CPhysicsObj::RecvEvt_LookAtDir`: a non-player's head direction.
        ObjectAction::LookAtDir { x, y } => body.look_at_dir(*x, *y),
        ObjectAction::SetGlobalModifier(id) => body.set_global_modifier(*id),
        ObjectAction::DoFxScript(id) => body.do_fx_script(*id),
        ObjectAction::DoFx { fx, scalar } => {
            body.do_fx(*fx, *scalar);
        }
        // `CPhysicsObj::RecvEvt_DoTargettedFX`: the FX manager's sound branch is the untargeted
        // one (`FXManager::DoTargettedFX`); the target only steers the visual part.
        ObjectAction::DoTargettedFx { fx, scalar, .. } => {
            body.do_fx(*fx, *scalar);
        }
        ObjectAction::StopFx(fx) => {
            body.stop_fx(*fx);
        }
        _ => {}
    }
    false
}

/// [`apply_from`] for the object `own`: a look at the object itself ends the look
/// (`CPhysicsObj::LookAt`'s own-id check), which needs the body's id.
pub(crate) fn apply_owned(
    body: &mut PhysicsBody,
    own: u64,
    action: &ObjectAction,
    source: Option<Position>,
) -> bool {
    if let ObjectAction::LookAt(target) = action {
        body.look_at(*target, own);
        return false;
    }
    apply_from(body, action, source)
}

/// [`apply`] with the position a targeted impulse pushes from: the target's position
/// (`CPhysicsObj::AddTargetedImpulse` takes `CPhysics::GetObjectA(target)`). `None` (the target
/// is not in the world, or it is the body itself) leaves the body alone.
pub(crate) fn apply_from(
    body: &mut PhysicsBody,
    action: &ObjectAction,
    source: Option<Position>,
) -> bool {
    if let (
        ObjectAction::AddTargetedImpulse {
            magnitude,
            falloff,
            vertical,
            ..
        },
        Some(source),
    ) = (action, source)
    {
        body.add_targeted_impulse(&source, *magnitude, *falloff != 0, *vertical);
        return false;
    }
    apply(body, action)
}

#[cfg(test)]
#[path = "motion_tests.rs"]
mod tests;
