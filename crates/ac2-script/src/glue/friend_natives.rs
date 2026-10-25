//! `gTurbineUserStatus` (0xAF): a friend's status the GUN events hand the script
//! (`gturbineuserstatus.cpp.c`). The efuns read the native's fields: `InqName` and `InqLocation`
//! write the wide string into the script's out variable (and return 0, writing nothing, for an
//! empty string), `InqMode` writes the mode. Each reads `this` at the frame's first slot, the out
//! reference after it, and returns an int in the deepest slot.

use ac2_weenie::FriendInfo;

use super::lists::{write_out, Kind};
use super::ui_natives::{native, Native};
use super::{word, Call, INT_RESULT};
use crate::native::NativeObject;
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

impl Native for FriendInfo {
    fn of(n: &NativeObject) -> Option<&Self> {
        match n {
            NativeObject::TurbineUserStatus(f) => Some(f),
            _ => None,
        }
    }
    fn of_mut(n: &mut NativeObject) -> Option<&mut Self> {
        match n {
            NativeObject::TurbineUserStatus(f) => Some(f),
            _ => None,
        }
    }
}

pub(super) fn register(efuns: &mut EFuns) {
    // gTurbineUserStatus::CallEFun_InqName, CallEFun_InqLocation.
    efuns.register(0x00AF_0001, |vm, c| inq_text(vm, c, |f| &f.name));
    efuns.register(0x00AF_0002, |vm, c| inq_text(vm, c, |f| &f.location));
    // gTurbineUserStatus::CallEFun_InqMode.
    efuns.register(0x00AF_0003, inq_mode);
}

/// `TurbineUserStatus::InqName` / `InqLocation`: the text into the out variable, 1; an empty text
/// writes nothing and returns 0 (`m_len == 1`).
fn inq_text(vm: &mut Vm, c: &mut Context, field: fn(&FriendInfo) -> &String) -> InterpStatus {
    let this = Call::new(c).this();
    let Some(text) = native::<FriendInfo>(vm, this).map(|f| field(f).clone()) else {
        return vm.bad_this(this);
    };
    let found = !text.is_empty();
    if found {
        let r = vm.new_wstring(text.encode_utf16().collect());
        vm.write_through(c, word(c, 2), r, tag::REF);
    }
    Call::new(c).ret(3, u32::from(found), INT_RESULT);
    InterpStatus::Success
}

/// `TurbineUserStatus::InqMode`: the mode into the out variable, 1.
fn inq_mode(vm: &mut Vm, c: &mut Context) -> InterpStatus {
    let this = Call::new(c).this();
    let Some(mode) = native::<FriendInfo>(vm, this).map(|f| f.mode) else {
        return vm.bad_this(this);
    };
    write_out(vm, c, word(c, 2), Some(u64::from(mode)), Kind::A);
    Call::new(c).ret(3, 1, INT_RESULT);
    InterpStatus::Success
}
