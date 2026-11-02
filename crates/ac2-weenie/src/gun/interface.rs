//! `gmCWInterface::RequestGUN_*` (Legions 0x00453320-0x00453F50, admin 0x004521E0-0x00452E40):
//! the calls the UI and the scripts (`ggmCWInterface::CallEFun_RequestGUN_*`) make. Legions and
//! admin run the same code. All but `RequestGUN_SendToRoom` follow one shape:
//!
//! ```text
//! if (S_pTurbineGun == NULL) return 0;
//! ctx = new CCliXResult(args);          // reference count 1
//! r = S_pTurbineGun->RequestX(args, ctx); // AddRefs ctx itself
//! release(ctx);                          // the creator's reference
//! return r;                              // connected && interface && ctx && hr >= 0
//! ```
//!
//! The functions return the `int` the script stores (BOOL) or the UI ignores.

use ac2_gun::{FlagOperation, FlagWord, TagBlob};

use super::chat_extra::{gun_chat_extra_info, pack_into_blob};
use super::handlers::{
    CCliCallCreateResult, CCliFlagsResult, CCliFriendAddResult, CCliFriendDeleteResult,
    CCliGetFriendsListResult, CCliGetIgnoredListResult, CCliIgnoredAddResult,
    CCliIgnoredDeleteResult, CCliLocationResult, CCliSendToRoomResult, CCliSendToUserResult,
};
use super::shared::{shared, TurbineRoomSend};

/// What `RequestGUN_SendToRoom` returns when Turbine chat is off and there is no `TurbineGun`:
/// retail's left-over register is `arg2`, the `PStringBase<ushort>*` of the message, a heap
/// address and never 0. There is no address here; scripts only test the BOOL for non-zero.
pub const MESSAGE_POINTER: u32 = 1;

fn flag(ok: bool) -> u32 {
    u32::from(ok)
}

/// `RequestGUN_FriendAdd(name)` (admin 0x004521E0, Legions 0x00453320): `CCliFriendAddResult(name)`
/// [0x1C bytes], `RequestFriendAdd(name, ctx)`.
pub fn request_gun_friend_add(name: &str) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    flag(gun.request_friend_add(name, Some(CCliFriendAddResult::new(name))))
}

/// `RequestGUN_FriendDelete(name)` (admin 0x00452250, Legions 0x00453390).
pub fn request_gun_friend_delete(name: &str) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    flag(gun.request_friend_delete(name, Some(CCliFriendDeleteResult::new(name))))
}

/// `RequestGUN_GetFriendsList(userRequested)` (admin 0x004522C0, Legions 0x00453400).
pub fn request_gun_get_friends_list(user_requested: u32) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    flag(gun.request_get_friends_list(Some(CCliGetFriendsListResult::new(user_requested))))
}

/// `RequestGUN_IgnoredAdd(name)` (admin 0x00452330, Legions 0x00453470).
pub fn request_gun_ignored_add(name: &str) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    flag(gun.request_ignored_add(name, Some(CCliIgnoredAddResult::new(name))))
}

/// `RequestGUN_IgnoredDelete(name)` (admin 0x004523A0, Legions 0x004534E0).
pub fn request_gun_ignored_delete(name: &str) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    flag(gun.request_ignored_delete(name, Some(CCliIgnoredDeleteResult::new(name))))
}

/// `RequestGUN_GetIgnoredList(userRequested)` (admin 0x00452410, Legions 0x00453550).
pub fn request_gun_get_ignored_list(user_requested: u32) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    flag(gun.request_get_ignored_list(Some(CCliGetIgnoredListResult::new(user_requested))))
}

/// `RequestGUN_CallCreate(callType, world, target, description)` (admin 0x00452480, Legions
/// 0x004535C0; INDEX records 0x004535D2, a mid-function address): `CCliCallCreateResult(callType,
/// target, description)` [0x24 bytes], the world string is not stored and `RequestCallCreate`
/// ignores its `world`. The assistance page (Legions 0x44458B) and the scripts call it.
pub fn request_gun_call_create(
    call_type: u32,
    _world: &str,
    target: &str,
    description: &str,
) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    let ctx = CCliCallCreateResult::new(call_type, target, description);
    flag(gun.request_call_create(call_type, target, description, Some(ctx)))
}

/// `RequestGUN_SendToUser(user, message, weenieFlags)` (admin 0x00452710, Legions 0x00453820):
/// 0 for an empty user or message or no `TurbineGun`; else `GunChatExtraInfo(this, flags)`
/// packed to a blob, `CCliSendToUserResult(user, message)` [0x20 bytes],
/// `RequestSendToUserEx(user, message, &blob, ctx)`. Turbine chat is not consulted for tells.
pub fn request_gun_send_to_user(user: &str, message: &str, weenie_flags: u32) -> u32 {
    let shared = shared();
    let gun = shared.turbine_gun();
    let Some(gun) = gun.filter(|_| !user.is_empty() && !message.is_empty()) else {
        return 0;
    };
    let blob = pack_into_blob(&gun_chat_extra_info(&shared, weenie_flags));
    let ctx = CCliSendToUserResult::new(user, message);
    flag(gun.request_send_to_user_ex(user, message, Some(&blob), Some(ctx)))
}

/// `RequestGUN_SendToRoom(roomId, message, weenieFlags)` (admin 0x004527C0, Legions 0x004538D0):
/// 0 for an empty message or room 0. Packs `GunChatExtraInfo(this, flags)` and makes
/// `CCliSendToRoomResult(roomId, message)` [0x20 bytes]. With `S_fUseTurbineChatInsteadOfGUN`:
/// `CWBookKeeper::CSendToTurbineRoomByID(roomId, message, {4, ctx}, blob)` and its value (1 when
/// the chat client took it, 0 when there is none; the creator's reference stays with that call and
/// the chat reply answers the context, see [`super::shared::GunShared::answer_turbine_send`]); else with a
/// `TurbineGun`: `RequestSendToRoomEx(roomId, message, &blob, ctx)`; else the left-over register,
/// `arg2` ([`MESSAGE_POINTER`]).
///
/// The Turbine chat send is recorded ([`super::shared::GunShared::take_turbine_sends`]) for the
/// program, which owns the chat client; the session's send cannot fail, so the value is 1 exactly
/// when a chat client exists ([`super::shared::GunShared::set_chat_client`]).
pub fn request_gun_send_to_room(room_id: u32, message: &str, weenie_flags: u32) -> u32 {
    if message.is_empty() || room_id == 0 {
        return 0;
    }
    let shared = shared();
    let blob = pack_into_blob(&gun_chat_extra_info(&shared, weenie_flags));
    if shared.use_turbine_chat() {
        // `CSendToTurbineRoomByID`: 0 without a chat client, else `hr == 0` of the send.
        if !shared.chat_client_present() {
            return 0;
        }
        shared.push_turbine_send(TurbineRoomSend {
            room: room_id,
            text: message.to_owned(),
            extra: blob.bytes().to_vec(),
            ctx: CCliSendToRoomResult::new(room_id, message),
        });
        return 1;
    }
    match shared.turbine_gun() {
        Some(gun) => {
            let ctx = CCliSendToRoomResult::new(room_id, message);
            flag(gun.request_send_to_room_ex(room_id, message, Some(&blob), Some(ctx)))
        }
        None => MESSAGE_POINTER,
    }
}

/// `RequestGUN_SetStateFlags(flags)` (admin 0x00452DD0, Legions 0x00453EE0): `CCliFlagsResult`
/// [0x18 bytes], `RequestFlags(STATE, SET, flags, ctx)`.
pub fn request_gun_set_state_flags(flags: u32) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    flag(gun.request_flags(
        FlagWord::State,
        FlagOperation::Set,
        flags,
        Some(CCliFlagsResult::new()),
    ))
}

/// `RequestGUN_SetLocation(mode, location)` (admin 0x00452E40, Legions 0x00453F50):
/// `CCliLocationResult` [0x18 bytes], `RequestSetLocation(mode, location, ctx)`.
pub fn request_gun_set_location(mode: u32, location: &str) -> u32 {
    let Some(gun) = shared().turbine_gun() else {
        return 0;
    };
    flag(gun.request_set_location(mode, location, CCliLocationResult::new()))
}

/// The blob `GunChatExtraInfo::PackIntoBlob` makes for `weenie_flags` from the player's name.
pub fn chat_extra_blob(weenie_flags: u32) -> TagBlob {
    pack_into_blob(&gun_chat_extra_info(&shared(), weenie_flags))
}
