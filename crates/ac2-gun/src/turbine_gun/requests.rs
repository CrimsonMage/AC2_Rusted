//! The `Request*` wrappers.
//!
//! Common shape (confirmed in the instructions of every wrapper): the guard (`m_fConnected`, the
//! interface pointer, and a non-NULL `ctx` unless the function says otherwise) else return 0;
//! `AddRef(ctx)`; the `IGun*` call; `PrintCmdHRESULT(name, hr)`; return its bool, i.e.
//! `hr >= 0`. A negative `hr` leaves the context's `AddRef` undone. All ten interface pointers
//! are the one [`crate::GunObject`], so each guard is `gun.is_some()`. `ctx` is `Option` where
//! retail tests it for NULL.

use std::rc::Rc;

use super::TurbineGun;
use crate::ct::{
    CtCallCreateResult, CtFlagsResult, CtFriendAddResult, CtFriendDeleteResult,
    CtGetFriendsListResult, CtGetIgnoredListResult, CtIgnoredAddResult, CtIgnoredDeleteResult,
    CtLocationResult, CtSendToRoomResult, CtSendToUserResult, PendingContext,
};
use crate::interfaces::{FlagOperation, FlagWord, SupportCallType, TagBlob};
use crate::turbine_gun_utils::print_cmd_hresult;

impl TurbineGun {
    /// The guard shared by the wrappers: `m_fConnected != 0` and the interface pointer.
    fn can_request(&self) -> bool {
        self.connected.get() && self.gun_rc().is_some()
    }

    /// `NSTurbineGun::TurbineGun::RequestFriendAdd` (admin 0x00606EC0, Legions 0x005C7D60):
    /// `IGunFriends::FriendAdd(LocalToGlobalName(name), ctx)` (`+0x10`), "FriendAdd".
    pub fn request_friend_add(&self, name: &str, ctx: Option<Rc<dyn CtFriendAddResult>>) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let global = self.local_to_global_name(name);
        let id = self.add_ref(PendingContext::FriendAdd(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult("FriendAdd", gun.friend_add(&global, id))
    }

    /// `NSTurbineGun::TurbineGun::RequestFriendDelete` (admin 0x00606F50, Legions 0x005C7DF0):
    /// `IGunFriends::FriendDelete(LocalToGlobalName(name), ctx)` (`+0x14`), "FriendDelete".
    pub fn request_friend_delete(
        &self,
        name: &str,
        ctx: Option<Rc<dyn CtFriendDeleteResult>>,
    ) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let global = self.local_to_global_name(name);
        let id = self.add_ref(PendingContext::FriendDelete(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult("FriendDelete", gun.friend_delete(&global, id))
    }

    /// `NSTurbineGun::TurbineGun::RequestIgnoredAdd` (admin 0x00606FE0, Legions 0x005C7E80):
    /// `IGunFriends::IgnoredAdd(LocalToGlobalName(name), ctx)` (`+0x1C`), "IgnoredAdd".
    pub fn request_ignored_add(&self, name: &str, ctx: Option<Rc<dyn CtIgnoredAddResult>>) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let global = self.local_to_global_name(name);
        let id = self.add_ref(PendingContext::IgnoredAdd(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult("IgnoredAdd", gun.ignored_add(&global, id))
    }

    /// `NSTurbineGun::TurbineGun::RequestIgnoredDelete` (admin 0x00607070, Legions 0x005C7F10):
    /// `IGunFriends::IgnoredDelete(LocalToGlobalName(name), ctx)` (`+0x20`), "IgnoredDelete".
    pub fn request_ignored_delete(
        &self,
        name: &str,
        ctx: Option<Rc<dyn CtIgnoredDeleteResult>>,
    ) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let global = self.local_to_global_name(name);
        let id = self.add_ref(PendingContext::IgnoredDelete(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult("IgnoredDelete", gun.ignored_delete(&global, id))
    }

    /// `NSTurbineGun::TurbineGun::RequestGetFriendsList` (admin 0x00605770, Legions 0x005C6650):
    /// `IGunFriends::GetFriendsList(ctx)` (`+0xC`), "GetFriendsList".
    pub fn request_get_friends_list(&self, ctx: Option<Rc<dyn CtGetFriendsListResult>>) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let id = self.add_ref(PendingContext::GetFriendsList(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult("GetFriendsList", gun.get_friends_list(id))
    }

    /// `NSTurbineGun::TurbineGun::RequestGetIgnoredList` (admin 0x006057D0, Legions 0x005C66B0):
    /// `IGunFriends::GetIgnoredList(ctx)` (`+0x18`), "GetIgnoredList".
    pub fn request_get_ignored_list(&self, ctx: Option<Rc<dyn CtGetIgnoredListResult>>) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let id = self.add_ref(PendingContext::GetIgnoredList(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult("GetIgnoredList", gun.get_ignored_list(id))
    }

    /// `NSTurbineGun::TurbineGun::RequestFlags` (admin 0x00605830, Legions 0x005C6710):
    /// `RequestFlags(FLAG_WORD which, FLAG_OPERATION op, ulong bits, CTFlagsResult* ctx)` calls
    /// `IGunFriends::Flags(op, which, bits, ctx)` (`+0x24`; the arguments swap), "Flags".
    /// `RequestGUN_SetStateFlags` calls it with `which = STATE`, `op = SET`.
    pub fn request_flags(
        &self,
        which: FlagWord,
        op: FlagOperation,
        bits: u32,
        ctx: Option<Rc<dyn CtFlagsResult>>,
    ) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let id = self.add_ref(PendingContext::Flags(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult("Flags", gun.flags(op, which, bits, id))
    }

    /// `NSTurbineGun::TurbineGun::RequestSetLocation` (admin 0x006058A0, Legions 0x005C6780):
    /// `RequestSetLocation(ulong mode, PStringBase<ushort>* location, CTLocationResult* ctx)`.
    /// The guard is only `m_fConnected` and the interface: there is NO NULL check of `ctx`
    /// (`AddRef(ctx)` is unconditional, so a NULL one crashes in retail; `ctx` is not optional
    /// here). `IGunFriends::SetLocation(mode, location, ctx)` (`+0x2C`), "SetLocation"; the
    /// location is NOT name-converted.
    pub fn request_set_location(
        &self,
        mode: u32,
        location: &str,
        ctx: Rc<dyn CtLocationResult>,
    ) -> bool {
        if !self.can_request() {
            return false;
        }
        let id = self.add_ref(PendingContext::Location(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult("SetLocation", gun.set_location(mode, location, id))
    }

    /// `NSTurbineGun::TurbineGun::RequestSendToRoomEx` (admin 0x006056F0, Legions 0x005C65D0):
    /// `RequestSendToRoomEx(ulong roomId, PStringBase<ushort>* message, tagBLOB* extra,
    /// CTSendToRoomResult* ctx)`. Guard plus `ctx`; a NULL `extra` is a local `{0, 0}` blob;
    /// `IGunChat::SendToRoomEx(roomId, message, extra, ctx)` (`+0x54`), "SendToRoomEx". The room
    /// id and the text are not name-converted.
    pub fn request_send_to_room_ex(
        &self,
        room_id: u32,
        message: &str,
        extra: Option<&TagBlob>,
        ctx: Option<Rc<dyn CtSendToRoomResult>>,
    ) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let id = self.add_ref(PendingContext::SendToRoom(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        let blob = extra.map_or(&[][..], TagBlob::bytes);
        print_cmd_hresult(
            "SendToRoomEx",
            gun.send_to_room_ex(room_id, message, blob, id),
        )
    }

    /// `NSTurbineGun::TurbineGun::RequestSendToUserEx` (admin 0x00606DF0, Legions 0x005C7C90):
    /// `RequestSendToUserEx(PStringBase<ushort>* user, PStringBase<ushort>* message, tagBLOB*
    /// extra, CTSendToUserResult* ctx)`. Guard plus `ctx`; a NULL `extra` is `{0, 0}`;
    /// `IGunChat::SendToUserEx(LocalToGlobalName(user), message, extra, ctx)` (`+0x58`),
    /// "SendToUserEx".
    pub fn request_send_to_user_ex(
        &self,
        user: &str,
        message: &str,
        extra: Option<&TagBlob>,
        ctx: Option<Rc<dyn CtSendToUserResult>>,
    ) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let global = self.local_to_global_name(user);
        let id = self.add_ref(PendingContext::SendToUser(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        let blob = extra.map_or(&[][..], TagBlob::bytes);
        print_cmd_hresult(
            "SendToUserEx",
            gun.send_to_user_ex(&global, message, blob, id),
        )
    }

    /// `NSTurbineGun::TurbineGun::RequestCallCreate` (admin 0x00607100, Legions 0x005C7FA0):
    /// `RequestCallCreate(ulong callType, PStringBase<ushort>* world, PStringBase<ushort>*
    /// target, PStringBase<ushort>* description, CTCallCreateResult* ctx)`. The guard is
    /// `m_fConnected`, `m_pGunSupport` and `ctx`. `AddRef(ctx)`; `desc` is NULL when the
    /// description is empty (`m_len == 1`), else its text; `tgt` is NULL when the target is
    /// empty, else `LocalToGlobalName(target)`; `IGunSupport::CallCreate(callType, tgt, desc,
    /// &{0, 0}, GetUserDefaultLCID(), ctx)` (`+0xC`), "CallCreate". The `world` argument is
    /// ignored (not a parameter here).
    pub fn request_call_create(
        &self,
        call_type: u32,
        target: &str,
        description: &str,
        ctx: Option<Rc<dyn CtCallCreateResult>>,
    ) -> bool {
        if !self.can_request() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let id = self.add_ref(PendingContext::CallCreate(ctx));
        let tgt = if target.is_empty() {
            None
        } else {
            Some(self.local_to_global_name(target))
        };
        let desc = if description.is_empty() {
            None
        } else {
            Some(description)
        };
        let lcid = self.user_default_lcid;
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        print_cmd_hresult(
            "CallCreate",
            gun.call_create(
                SupportCallType(call_type),
                tgt.as_deref(),
                desc,
                &[],
                lcid,
                id,
            ),
        )
    }
}
