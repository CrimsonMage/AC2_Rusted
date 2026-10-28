//! The `gCWPhysicsObject` calls that set or read a weenie's body state directly
//! (`CWPhysicsObject` -> `CPhysicsObj`): the movement and animation freezes, the edge-slide flag
//! (`gCWRepository::SetEdgeSlides`), the marker toggle, and the reads the body answers
//! (`IsOutside`, `GetCollisionState`). The client owns the bodies, so the changes are queued
//! ([`BodyState`]) and the reads answer from a snapshot of the body it shares each frame
//! ([`BodySnapshot`], `ScriptHost::set_body_snapshot`).

use ac2_net::protocol::ObjectAction;

use super::interface::this_weenie;
use super::ScriptHost;
use crate::glue::{out_native, Call};
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

/// `LandDefs::cellid_mask`: the low 16 bits of a cell id.
const CELLID_MASK: u32 = 0xFFFF;
/// `LandDefs::first_envcell_id`: cells below it are landscape cells.
const FIRST_ENVCELL_ID: u32 = 0x100;

/// A state change a script made on a weenie's physics object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyState {
    /// `CPhysicsObj::set_movement_frozen` (`MOVEMENT_FROZEN_PS`).
    MovementFrozen(bool),
    /// `CPhysicsObj::set_animation_frozen` (`ANIMATION_FROZEN_PS`).
    AnimationFrozen(bool),
    /// `CPhysicsObj::set_edge_slide` (`EDGE_SLIDE_PS`).
    EdgeSlide(bool),
}

/// What the scripts read of a weenie's body: its cell (`CPhysicsObj::GetPosition`) and its
/// collision state (`CPhysicsObj::GetCollisionState`).
#[derive(Debug, Clone, PartialEq)]
pub struct BodySnapshot {
    pub cell: u32,
    pub collision: ac2_physics::CollisionParameters,
    /// The cell material's property groups (`CPhysicsObj::InqCurrentProperties`), when the body
    /// has one.
    pub properties: Option<std::sync::Arc<ac2_physics::PhysicsMaterial>>,
}

impl ScriptHost {
    /// The body state of object `id` as the client holds it, or `None` when it has no body.
    pub fn set_body_snapshot(&mut self, id: u64, snapshot: Option<BodySnapshot>) {
        match snapshot {
            Some(s) => {
                self.vm.bodies.insert(id, s);
            }
            None => {
                self.vm.bodies.remove(&id);
            }
        }
    }

    /// The body state changes the scripts asked for since the last call, in order.
    pub fn take_body_states(&mut self) -> Vec<(u64, BodyState)> {
        std::mem::take(&mut self.vm.body_states)
    }
}

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0080_0054, |vm, ctx| {
        freeze(vm, ctx, BodyState::MovementFrozen)
    });
    efuns.register(0x0080_0056, |vm, ctx| {
        freeze(vm, ctx, BodyState::AnimationFrozen)
    });
    efuns.register(0x0080_002B, toggle_place_marker_objects);
    efuns.register(0x0080_0036, is_outside);
    efuns.register(0x0080_000B, get_collision_state);
    efuns.register(0x0080_004A, inq_current_properties);
    efuns.register(0x0080_001C, do_behavior_non_latent);
}

/// `gCWPhysicsObject::CallEFun_SetMovementFrozen` / `SetAnimationFrozen(frozen)` ->
/// `CWPhysicsObject::Set*Frozen` -> `CPhysicsObj::set_*_frozen`: answers 1 for a weenie with a
/// body (0 without).
pub(crate) fn freeze(vm: &mut Vm, ctx: &mut Context, state: fn(bool) -> BodyState) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let frozen = Call::new(ctx).word(2) != 0;
    if id != 0 {
        vm.body_states.push((id, state(frozen)));
    }
    Call::new(ctx).ret(3, u32::from(id != 0), tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_TogglePlaceMarkerObjects` -> `CPhysicsObj::TogglePlaceMarkerObjects`
/// (`m_bPlaceMarkers` flips) then `IsPlacingMarkerObjects`: answers the new flag. The marker
/// objects are a debugging aid the client does not draw, so only the flag is kept. 0 without a
/// body.
fn toggle_place_marker_objects(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let placing = id != 0 && {
        if !vm.place_markers.remove(&id) {
            vm.place_markers.insert(id);
        }
        vm.place_markers.contains(&id)
    };
    Call::new(ctx).ret(2, u32::from(placing), tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_IsOutside` -> `CWPhysicsObject::IsOutside`: whether the body's
/// cell is a landscape cell (`cell & cellid_mask < first_envcell_id`); 0 without a body.
fn is_outside(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let outside = vm
        .bodies
        .get(&id)
        .is_some_and(|b| b.cell & CELLID_MASK < FIRST_ENVCELL_ID);
    Call::new(ctx).ret(2, u32::from(outside), tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_GetCollisionState(CollisionParameters *)` ->
/// `CPhysicsObj::GetCollisionState`: the parameters take the body's physics type and its
/// placement and movement physics masks (and their bits). Answers 1; 0 without a body.
fn get_collision_state(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let params = Call::new(ctx).word(2);
    let target = out_native(vm, ctx, params);
    let state = vm.bodies.get(&id).map(|b| b.collision.clone());
    match vm.native_mut(target) {
        Some(NativeObject::CollisionParameters(p)) => {
            if let Some(state) = &state {
                p.sets.set_physics_type(state.physics_type());
                p.sets.set_physics_placement_type(state.placement_physics());
                p.sets.set_physics_movement_type(state.movement_physics());
            }
        }
        _ => return vm.bad_this(target),
    }
    Call::new(ctx).ret(3, u32::from(state.is_some()), tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_InqCurrentProperties(PropertyCollection *)` ->
/// `CWPhysicsObject::InqCurrentProperties` -> `CPhysicsObj::InqCurrentProperties`: the collection
/// becomes a copy of the body's cell material collection and the efun answers 1; 0 (the
/// collection untouched) while the body has no cell material or no body exists.
fn inq_current_properties(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let reference = Call::new(ctx).word(2);
    let target = out_native(vm, ctx, reference);
    let properties = vm.bodies.get(&id).and_then(|b| b.properties.clone());
    if let Some(material) = &properties {
        match vm.native_mut(target) {
            Some(NativeObject::PropertyCollection(collection)) => {
                **collection = ac2_db::PropertyCollection::default();
                for property in material.groups.iter().flat_map(|g| g.properties.iter()) {
                    collection.add_property(&property.to_base_property());
                }
            }
            _ => return vm.bad_this(target),
        }
    }
    Call::new(ctx).ret(3, u32::from(properties.is_some()), tag::INT);
    InterpStatus::Success
}

/// `gCWPhysicsObject::CallEFun_DoBehaviorNonLatent(params)` -> `CWPhysicsObject::
/// DoBehaviorNonLatent`: the behaviour starts with its context id cleared, so no completion
/// resumes a script. Queued as the server's `DoBehavior`; answers 0 for a weenie with a body
/// (8 without one).
fn do_behavior_non_latent(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let id = match this_weenie(vm, ctx) {
        Ok(id) => id,
        Err(status) => return status,
    };
    let reference = Call::new(ctx).word(2);
    let params = match vm.native(reference) {
        Some(NativeObject::BehaviorParams(p)) => p.clone(),
        _ => return vm.bad_this(reference),
    };
    if id != 0 {
        vm.physics.push((id, ObjectAction::DoBehavior(params)));
    }
    Call::new(ctx).ret(3, if id != 0 { 0 } else { 8 }, tag::INT);
    InterpStatus::Success
}
