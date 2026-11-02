//! `ggmCWInterface::CallEFun_RequestGUN_*` (Legions 0x00455850-0x00455C20, admin 0x004549F0-
//! 0x00454F30): the chat-service requests scripts make. Each wrapper looks `this` and its
//! string arguments up in the object table, calls `gmCWInterface::RequestGUN_*`
//! (`ac2_weenie::gun::interface`, which reaches the global `S_pTurbineGun`), and stores the
//! `int` it returns in the result slot (tag 4, BOOL). Only the first argument slot (`this`,
//! `fp-4`) is checked: an undefined one crashes the interpreter (admin asserts first,
//! `"index != Undef_ReferenceID"`; Legions does not).
//!
//! Slot maps (admin source; ints are raw words, strings reference ids): CallCreate this -4,
//! callType -8, world -0xC, target -0x10, description -0x14, result -0x18; SendToRoom room -8,
//! message -0xC, flags -0x10, result -0x14; SendToUser user -8, message -0xC, flags -0x10,
//! result -0x14; SetLocation mode -8, location -0xC, result -0x10; FriendAdd / FriendDelete /
//! IgnoredAdd / IgnoredDelete name -8, result -0xC; GetFriendsList / GetIgnoredList
//! userRequested -8, result -0xC; SetStateFlags flags -8, result -0xC. EFun ids (Legions
//! dispatcher 0x466116-0x4661FE): SendToUser 0xB40032 (0x455B50), SendToRoom 0xB40033
//! (0x455AF0), SetStateFlags 0xB40034 (0x455C20), FriendAdd 0xB40035 (0x4558D0), FriendDelete
//! 0xB40036 (0x455930), GetFriendsList 0xB40037 (0x455990), IgnoredAdd 0xB40038 (0x455A30),
//! IgnoredDelete 0xB40039 (0x455A90), GetIgnoredList 0xB4003A (0x4559E0), CallCreate 0xB4003B
//! (0x455850), SetLocation 0xB400EB (0x455BC0).

use ac2_weenie::gun::interface as gmcw;

use crate::glue::{word, Call};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x00B4_0032, |vm, c| {
        let (user, message, flags) = (text(vm, c, 2), text(vm, c, 3), word(c, 4));
        finish(vm, c, 5, |_| {
            gmcw::request_gun_send_to_user(&user, &message, flags)
        })
    });
    efuns.register(0x00B4_0033, |vm, c| {
        let (room, message, flags) = (word(c, 2), text(vm, c, 3), word(c, 4));
        finish(vm, c, 5, |_| {
            gmcw::request_gun_send_to_room(room, &message, flags)
        })
    });
    efuns.register(0x00B4_0034, |vm, c| {
        let flags = word(c, 2);
        finish(vm, c, 3, |_| gmcw::request_gun_set_state_flags(flags))
    });
    efuns.register(0x00B4_0035, |vm, c| {
        let name = text(vm, c, 2);
        finish(vm, c, 3, |_| gmcw::request_gun_friend_add(&name))
    });
    efuns.register(0x00B4_0036, |vm, c| {
        let name = text(vm, c, 2);
        finish(vm, c, 3, |_| gmcw::request_gun_friend_delete(&name))
    });
    efuns.register(0x00B4_0037, |vm, c| {
        let user_requested = word(c, 2);
        finish(vm, c, 3, |_| {
            gmcw::request_gun_get_friends_list(user_requested)
        })
    });
    efuns.register(0x00B4_0038, |vm, c| {
        let name = text(vm, c, 2);
        finish(vm, c, 3, |_| gmcw::request_gun_ignored_add(&name))
    });
    efuns.register(0x00B4_0039, |vm, c| {
        let name = text(vm, c, 2);
        finish(vm, c, 3, |_| gmcw::request_gun_ignored_delete(&name))
    });
    efuns.register(0x00B4_003A, |vm, c| {
        let user_requested = word(c, 2);
        finish(vm, c, 3, |_| {
            gmcw::request_gun_get_ignored_list(user_requested)
        })
    });
    efuns.register(0x00B4_003B, |vm, c| {
        let call_type = word(c, 2);
        let (world, target, description) = (text(vm, c, 3), text(vm, c, 4), text(vm, c, 5));
        finish(vm, c, 6, |_| {
            gmcw::request_gun_call_create(call_type, &world, &target, &description)
        })
    });
    efuns.register(0x00B4_00EB, |vm, c| {
        let (mode, location) = (word(c, 2), text(vm, c, 3));
        finish(vm, c, 4, |_| {
            gmcw::request_gun_set_location(mode, &location)
        })
    });
}

fn text(vm: &Vm, ctx: &Context, n: usize) -> String {
    String::from_utf16_lossy(vm.wstring(word(ctx, n)).unwrap_or(&[]))
}

/// Stores `call`'s result in slot `ret` as a BOOL, crashing first on an undefined `this`.
fn finish(
    vm: &mut Vm,
    ctx: &mut Context,
    ret: usize,
    call: impl FnOnce(&mut Vm) -> u32,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    if this == u32::MAX {
        return vm.bad_this(this);
    }
    let result = call(vm);
    Call::new(ctx).ret(ret, result, tag::BOOL);
    InterpStatus::Success
}
