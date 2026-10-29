//! The `gCWRepository` calls that tie a weenie's qualities to its physics object
//! (`CWRepository::InqEtherealType`, `SetEtherealType`, `SetEdgeSlides`, `SetScale`,
//! `LoadDataIDW`; `portal/shared/src/weenie/repository/wrepository.cpp`). Without qualities
//! (`m_qualities == NULL`) they answer 0.
//!
//! `SetEdgeSlides` and `SetScale` also reach the body (`CWPhysicsObject::_SetEdgeSlide` and
//! `_SetScale`, queued as `BodyState`s).

use ac2_net::protocol::QualityValue;
use ac2_physics::PhysicsTypeMask;

use super::repository::owner_qualities;
use super::{narrow, out_native, Call, INT_RESULT};
use crate::host::body_state::BodyState;
use crate::native::NativeObject;
use crate::vm::{Context, EFuns, InterpStatus, Vm};

/// `STypeBool` 7: the weenie slides along edges.
const EDGE_SLIDES: u32 = 7;
/// `STypeFloat` 0x12A: the object's scale.
const SCALE: u32 = 0x12A;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0081_000B, inq_ethereal_type);
    efuns.register(0x0081_000C, set_ethereal_type);
    efuns.register(0x0081_000D, set_edge_slides);
    efuns.register(0x0081_0052, set_scale);
    efuns.register(0x0081_0054, load_data_id);
}

/// `InqEtherealType(EtherealPhysicsType, low int stype, high int stype)`: when both ints are
/// present the type's mask becomes `high << 32 | low`; the result is whether they were.
fn inq_ethereal_type(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, out, low, high) = (call.this(), call.word(2), call.word(3), call.word(4));
    let found = owner_qualities(vm, this).and_then(|q| Some((q.inq_int(low)?, q.inq_int(high)?)));
    if let Some((low, high)) = found {
        let target = out_native(vm, ctx, out);
        match vm.native_mut(target) {
            Some(NativeObject::EtherealPhysicsType(mask)) => {
                *mask = PhysicsTypeMask::from_bits(
                    u64::from(high as u32) << 32 | u64::from(low as u32),
                );
            }
            _ => return vm.bad_this(target),
        }
    }
    Call::new(ctx).ret(5, u32::from(found.is_some()), INT_RESULT);
    InterpStatus::Success
}

/// `SetEtherealType(EtherealPhysicsType, low int stype, high int stype)`: the mask's two words
/// go into the two ints.
fn set_ethereal_type(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, arg, low, high) = (call.this(), call.word(2), call.word(3), call.word(4));
    let source = out_native(vm, ctx, arg);
    let bits = match vm.native(source) {
        Some(NativeObject::EtherealPhysicsType(mask)) => mask.bits(),
        _ => return vm.bad_this(source),
    };
    let ok = owner_qualities(vm, this).is_some_and(|q| {
        q.set(low, &QualityValue::Int(bits as u32 as i32));
        q.set(high, &QualityValue::Int((bits >> 32) as u32 as i32));
        true
    });
    Call::new(ctx).ret(5, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

/// `SetEdgeSlides(flag)`: the bool quality 7.
fn set_edge_slides(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, flag) = (call.this(), call.word(2));
    let ok = owner_qualities(vm, this).is_some_and(|q| {
        q.set(EDGE_SLIDES, &QualityValue::Bool(flag as i32));
        true
    });
    // `CWPhysicsObject::_SetEdgeSlide` -> `CPhysicsObj::set_edge_slide`, on the weenie's body.
    if ok {
        if let Some(&id) = vm.owners.get(&this) {
            vm.body_states.push((id, BodyState::EdgeSlide(flag != 0)));
        }
    }
    Call::new(ctx).ret(3, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

/// `SetScale(float)`: the float quality 0x12A.
fn set_scale(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, scale) = (call.this(), f32::from_bits(call.word(2)));
    let ok = owner_qualities(vm, this).is_some_and(|q| {
        q.set(SCALE, &QualityValue::Float(scale));
        true
    });
    // `CWPhysicsObject::_SetScale` -> `CPhysicsObj::SetScale(float)`, clamped to 0.1..10.
    if ok {
        if let Some(&id) = vm.owners.get(&this) {
            vm.body_states
                .push((id, BodyState::Scale(scale.clamp(0.1, 10.0))));
        }
    }
    Call::new(ctx).ret(3, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

/// `LoadDataIDW(data id stype, file name)`: the DID the data file name gives
/// (`FileArray::GetDIDForDummies`) becomes the quality; a name with no file answers 0.
fn load_data_id(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, stype, name) = (call.this(), call.word(2), call.word(3));
    let name = match vm.wstring(name) {
        Some(text) => narrow(text),
        None => return vm.bad_this(name),
    };
    let did = vm.data().file_did(&name, 0).filter(|&did| did != 0);
    let ok = did.is_some_and(|did| {
        owner_qualities(vm, this).is_some_and(|q| {
            q.set(stype, &QualityValue::DataId(did));
            true
        })
    });
    Call::new(ctx).ret(4, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}
