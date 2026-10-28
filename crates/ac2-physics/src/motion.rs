//! The motion events a body plays (`CPhysicsObj::RecvEvt_*`) as physics data: the mode,
//! behaviour, layer and slider events, `RecvEvt_MoveTo`, the impulses, the velocity,
//! acceleration and jump scales and the hover height, the look events and the FX events. The
//! client turns the server's `ObjectAction` into these (`ac2-client`'s `motion` module).

use ac2_anim::Ac2BehaviorParams;

use crate::body::PhysicsBody;
use crate::moveto::MovementParameters;
use crate::position::Position;

/// One motion event for a body, as the server's `ObjectAction` carries it, with the behaviour
/// parameters and move-to parameters already unpacked.
#[derive(Debug, Clone, PartialEq)]
pub enum MotionEvent {
    /// `RecvEvt_DoMode`: `DoMode(0, mode)`.
    DoMode(u32),
    /// `RecvEvt_SetMode`: at once.
    SetMode(u32),
    StopLayer(u32),
    /// `RecvEvt_DoBehavior`: a copy whose context is 1.
    DoBehavior(Ac2BehaviorParams),
    StopBehavior(u32),
    /// `RecvEvt_ReleaseBehavior`: the behaviour id and a copy whose context is 1.
    ReleaseBehavior {
        behavior: u32,
        params: Ac2BehaviorParams,
    },
    StopAllWeenieBehaviors,
    StopMoveToCancelsBehaviors,
    /// `RecvEvt_DoSlider`: the player's client-controlled sliders are left alone.
    DoSlider {
        slider: u32,
        value: f32,
        time: f32,
    },
    /// `RecvEvt_MoveTo`: only the player moves to.
    MoveTo(Box<MovementParameters>),
    /// `RecvEvt_AddImpulse`: knockback in world axes.
    AddImpulse {
        direction: [f32; 3],
    },
    /// `RecvEvt_AddTargetedImpulse`: pushes from the source given to `apply_motion_from`.
    AddTargetedImpulse {
        magnitude: f32,
        falloff: bool,
        vertical: f32,
    },
    SetVelocityScale(f32),
    SetAccelerationScale(f32),
    SetJumpScale(f32),
    SetHoverHeight(f32),
    /// `RecvEvt_LookAt`: the target object; the body's own id ends the look (see
    /// `apply_motion_owned`).
    LookAt(u64),
    /// `RecvEvt_LookAtDir`: a non-player's head direction.
    LookAtDir {
        x: f32,
        y: f32,
    },
    SetGlobalModifier(u32),
    /// `RecvEvt_AddTerrainModifier`: the terrain and the modifier it takes.
    AddTerrainModifier {
        terrain: u32,
        modifier: u32,
    },
    /// `RecvEvt_ClearTerrainModifiers`: every registered terrain modifier goes.
    ClearTerrainModifiers,
    DoFxScript(u32),
    /// `RecvEvt_DoFX` and the untargeted sound branch of `RecvEvt_DoTargettedFX`
    /// (`FXManager::DoTargettedFX`).
    DoFx {
        fx: u32,
        scalar: f32,
    },
    StopFx(u32),
}

/// Plays `event` on `body`. Returns whether a move-to started.
pub fn apply_motion(body: &mut PhysicsBody, event: &MotionEvent) -> bool {
    match event {
        MotionEvent::DoMode(mode) => {
            body.do_mode(0, *mode);
        }
        MotionEvent::SetMode(mode) => {
            body.set_mode(*mode);
        }
        MotionEvent::StopLayer(layer) => body.stop_layer(*layer),
        MotionEvent::DoBehavior(params) => {
            body.do_behavior(params);
        }
        MotionEvent::StopBehavior(id) => body.stop_behavior(*id),
        MotionEvent::ReleaseBehavior {
            behavior: id,
            params,
        } => {
            body.release_behavior(*id, params);
        }
        MotionEvent::StopAllWeenieBehaviors => {
            body.stop_all_weenie_behaviors();
        }
        MotionEvent::StopMoveToCancelsBehaviors => body.stop_move_to_cancels_behaviors(),
        MotionEvent::DoSlider {
            slider,
            value,
            time,
        } => {
            body.recv_do_slider(*slider, *value, *time);
        }
        MotionEvent::MoveTo(params) => {
            return body.recv_move_to(params.as_ref().clone());
        }
        MotionEvent::AddImpulse { direction } => body.add_impulse(*direction),
        MotionEvent::SetVelocityScale(scale) => body.set_velocity_scale(*scale),
        MotionEvent::SetAccelerationScale(scale) => body.set_acceleration_scale(*scale),
        MotionEvent::SetJumpScale(scale) => body.set_jump_scale(*scale),
        MotionEvent::SetHoverHeight(height) => body.set_hover_height(*height),
        MotionEvent::LookAtDir { x, y } => body.look_at_dir(*x, *y),
        MotionEvent::SetGlobalModifier(id) => body.set_global_modifier(*id),
        MotionEvent::AddTerrainModifier { terrain, modifier } => {
            body.add_terrain_modifier(*terrain, *modifier);
        }
        MotionEvent::ClearTerrainModifiers => body.clear_terrain_modifiers(),
        MotionEvent::DoFxScript(id) => body.do_fx_script(*id),
        MotionEvent::DoFx { fx, scalar } => {
            body.do_fx(*fx, *scalar);
        }
        MotionEvent::StopFx(fx) => {
            body.stop_fx(*fx);
        }
        // Only the targeted forms and a look at an object need more than the body (see
        // `apply_motion_from` and `apply_motion_owned`).
        MotionEvent::AddTargetedImpulse { .. } | MotionEvent::LookAt(_) => {}
    }
    false
}

/// [`apply_motion_from`] for the object `own`: a look at the object itself ends the look
/// (`CPhysicsObj::LookAt`'s own-id check), which needs the body's id.
pub fn apply_motion_owned(
    body: &mut PhysicsBody,
    own: u64,
    event: &MotionEvent,
    source: Option<Position>,
) -> bool {
    if let MotionEvent::LookAt(target) = event {
        body.look_at(*target, own);
        return false;
    }
    apply_motion_from(body, event, source)
}

/// [`apply_motion`] with the position a targeted impulse pushes from: the target's position
/// (`CPhysicsObj::AddTargetedImpulse` takes `CPhysics::GetObjectA(target)`). `None` (the target
/// is not in the world, or it is the body itself) leaves the body alone.
pub fn apply_motion_from(
    body: &mut PhysicsBody,
    event: &MotionEvent,
    source: Option<Position>,
) -> bool {
    if let (
        MotionEvent::AddTargetedImpulse {
            magnitude,
            falloff,
            vertical,
        },
        Some(source),
    ) = (event, source)
    {
        body.add_targeted_impulse(&source, *magnitude, *falloff, *vertical);
        return false;
    }
    apply_motion(body, event)
}
