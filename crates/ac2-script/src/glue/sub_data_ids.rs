//! `GetSubDataIDs`: the data ids a string or a scene depends on, added to a
//! `QualifiedDataIDArray` (`gStringInfo::CallEFun_GetSubDataIDs`,
//! `ggmSceneInfo::CallEFun_GetSubDataIDs`; the arguments are the DB type filter and the array).

use super::enums::add_data_id;
use super::{out_native, word, Call};
use crate::native::{NativeObject, StringInfo, StringVarValue};
use crate::vm::{Context, EFuns, InterpStatus, Vm};
use crate::wire::string_info_from_net;

/// The DB type `gmKeyframe::GetSubDataIDs` leaves the track ids out for (`DB_TYPE_DATA_FILE`).
const TYPE_DATA_FILE: u32 = 0x4000_0008;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x006A_001C, string_info_sub_data_ids);
    efuns.register(0x00A3_0002, scene_info_sub_data_ids);
}

/// `StringInfo::GetSubDataIDs`: a string that is not a literal adds its table, then its nested
/// `StringInfo` variables (kind 9) add theirs.
fn collect(vm: &mut Vm, set: u32, info: &StringInfo) {
    if info.literal.is_some() {
        return;
    }
    add_data_id(vm, set, info.table_id);
    for var in &info.variables {
        if let StringVarValue::StringInfo(nested) = &var.value {
            if var.kind == 9 {
                collect(vm, set, nested);
            }
        }
    }
}

/// The set the array argument names, crashing the script when it is not one.
fn array_arg(vm: &mut Vm, ctx: &Context, reference: u32) -> Result<u32, InterpStatus> {
    let set = out_native(vm, ctx, reference);
    match vm.native(set) {
        Some(NativeObject::DataIds(_)) => Ok(set),
        _ => Err(vm.bad_this(set)),
    }
}

/// `gStringInfo::CallEFun_GetSubDataIDs(type, array)`.
fn string_info_sub_data_ids(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let array = word(ctx, 3);
    let info = match vm.native(this) {
        Some(NativeObject::StringInfo(s)) => s.clone(),
        _ => return vm.bad_this(this),
    };
    match array_arg(vm, ctx, array) {
        Ok(set) => {
            collect(vm, set, &info);
            InterpStatus::Success
        }
        Err(status) => status,
    }
}

/// `gmSceneInfo::GetSubDataIDs`: each keyframe adds its video, audio and voice ids (unless the
/// filter is the data-file type) and its subtitle's.
fn scene_info_sub_data_ids(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let (filter, array) = (word(ctx, 2), word(ctx, 3));
    let frames = match vm.native(this) {
        Some(NativeObject::SceneInfo(s)) => s.keyframes.clone(),
        _ => return vm.bad_this(this),
    };
    let set = match array_arg(vm, ctx, array) {
        Ok(set) => set,
        Err(status) => return status,
    };
    for frame in &frames {
        if filter != TYPE_DATA_FILE {
            for did in [frame.video, frame.audio, frame.voice] {
                add_data_id(vm, set, did);
            }
        }
        collect(vm, set, &string_info_from_net(&frame.subtitle));
    }
    InterpStatus::Success
}
