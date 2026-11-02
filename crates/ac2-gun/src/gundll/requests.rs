//! The request-sending interfaces of `CGun`: `IGunFriends`, `IGunChat`, `IGunSupport`, `IGunPersona`,
//! `IGunBrowser`. Each method is a `CGun::X` of the decompile
//! (`decompiled-pdb/gundll/GunDll/GunDll.dll.c`): the connection test, the argument checks, the
//! request class's `Init` (which stores the context as `Cx` and duplicates the arguments) and
//! `SendRequest`. Addresses are the `gun_interfaces.csv` ones.
//!
//! `this+0x38` of the friends interface, `this+0x3c` of chat, `this+0x30` of the browser, `this+0x40`
//! of support and `this+0x24` of persona all name `CGun+0x4c`: the connection is tested for being
//! present, not for being connected. A `&str` argument cannot be NULL, so the `p == NULL ->
//! 0x80070057` branches of retail have no counterpart where the trait takes `&str`.

use super::catalog::by_id;
use super::cgun::CGun;
use super::msg::GunMsg;
use crate::interfaces::*;
use crate::zt::*;
use crate::Hresult;

const E_INVALIDARG: Hresult = Hresult(0x8007_0057);

impl CGun {
    pub(super) fn new_request(&self, id: u32, ctx: Context) -> GunMsg {
        let mut m = GunMsg::new(by_id(id).expect("request class is catalogued"));
        m.cx = ctx.0;
        m
    }

    /// `if (conn == 0) ZT_E_NOT_CONNECTED; ... Init; SendRequest`.
    pub(super) fn send_if_connected(&self, m: GunMsg) -> Hresult {
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        self.send_request(&m)
    }

    fn user_request(&self, id: u32, attr: &str, user: &str, ctx: Context) -> Hresult {
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        let mut m = self.new_request(id, ctx);
        m.set_str(attr, Some(user));
        self.send_request(&m)
    }

    fn plain_request(&self, id: u32, ctx: Context) -> Hresult {
        self.send_if_connected(self.new_request(id, ctx))
    }
}

/// `wcslen` in UTF-16 units.
pub(super) fn wcslen(s: &str) -> usize {
    s.encode_utf16().count()
}

impl IGunFriends for CGun {
    /// `CGun::GetFriendsList` (0x1000dc40): `<GetFriendsList>` (0x4100).
    fn get_friends_list(&self, context: Context) -> Hresult {
        self.plain_request(0x4100, context)
    }
    /// `CGun::FriendAdd` (0x1000dcd0): `<FriendAdd User>` (0x4101).
    fn friend_add(&self, user: &str, context: Context) -> Hresult {
        self.user_request(0x4101, "User", user, context)
    }
    /// `CGun::FriendDelete` (0x1000dd70): 0x4102.
    fn friend_delete(&self, user: &str, context: Context) -> Hresult {
        self.user_request(0x4102, "User", user, context)
    }
    /// `CGun::GetIgnoredList` (0x1000de10): 0x4103.
    fn get_ignored_list(&self, context: Context) -> Hresult {
        self.plain_request(0x4103, context)
    }
    /// `CGun::IgnoredAdd` (0x1000dea0): 0x4104.
    fn ignored_add(&self, user: &str, context: Context) -> Hresult {
        self.user_request(0x4104, "User", user, context)
    }
    /// `CGun::IgnoredDelete` (0x1000df40): 0x4105.
    fn ignored_delete(&self, user: &str, context: Context) -> Hresult {
        self.user_request(0x4105, "User", user, context)
    }
    /// `CGun::Flags` (0x1000dfe0): `<Flags Op Word Flags>` (0x4108), `Init(ctx, op, word, bits)`.
    fn flags(
        &self,
        op: FlagOperation,
        which_flag: FlagWord,
        bits_to_set_or_clear: u32,
        context: Context,
    ) -> Hresult {
        let mut m = self.new_request(0x4108, context);
        m.set_dec("Op", op as i32);
        m.set_dec("Word", which_flag as i32);
        m.set_dec("Flags", bits_to_set_or_clear as i32);
        self.send_if_connected(m)
    }
    /// `CGun::WhereIs` (0x1000e080): 0x4109.
    fn where_is(&self, a: &str, context: Context) -> Hresult {
        self.user_request(0x4109, "User", a, context)
    }
    /// `CGun::SetLocation` (0x1000e120): `<SetLoc Flags Location>` (0x410a), `Init(ctx, mode,
    /// location)`.
    fn set_location(&self, mode: u32, location: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x410a, context);
        m.set_dec("Flags", mode as i32);
        m.set_str("Location", Some(location));
        self.send_if_connected(m)
    }
    /// `CGun::AcceptAddFriend` (0x1000e1c0): `<AcceptAddFriend Reply User>` (0x410b), `Reply` the bool.
    fn accept_add_friend(&self, a: bool, b: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x410b, context);
        m.set_dec("Reply", a as i32);
        m.set_str("User", Some(b));
        self.send_if_connected(m)
    }
}

impl IGunChat for CGun {
    /// `CGun::GetListOfRooms` (0x1000e260): 0x5100.
    fn get_list_of_rooms(&self, context: Context) -> Hresult {
        self.plain_request(0x5100, context)
    }
    /// `CGun::JoinRoom` (0x1000e2f0): `<JoinRoom Room Password>` (0x5101); a NULL password is `L""`.
    fn join_room(&self, a: &str, b: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x5101, context);
        m.set_str("Room", Some(a));
        m.set_str("Password", Some(b));
        self.send_if_connected(m)
    }
    /// `CGun::LeaveRoom` (0x1000e3a0): `<LeaveRoom RoomId>` (0x5102).
    fn leave_room(&self, a: u32, context: Context) -> Hresult {
        let mut m = self.new_request(0x5102, context);
        m.set_dec("RoomId", a as i32);
        self.send_if_connected(m)
    }
    /// `CGun::LeaveAllRooms` (0x1000e430): 0x5103.
    fn leave_all_rooms(&self, context: Context) -> Hresult {
        self.plain_request(0x5103, context)
    }
    /// `CGun::GetUsersInRoom` (0x1000e4c0): 0x5104.
    fn get_users_in_room(&self, a: u32, context: Context) -> Hresult {
        let mut m = self.new_request(0x5104, context);
        m.set_dec("RoomId", a as i32);
        self.send_if_connected(m)
    }
    /// `CGun::SendToRoom` (0x1000e550): `<SendToRoom RoomId Text>` (0x5105).
    fn send_to_room(&self, a: u32, b: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x5105, context);
        m.set_dec("RoomId", a as i32);
        m.set_str("Text", Some(b));
        self.send_if_connected(m)
    }
    /// `CGun::SendToUser` (0x1000e6b0): `<SendToUser User Text>` (0x5106).
    fn send_to_user(&self, a: &str, b: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x5106, context);
        m.set_str("User", Some(a));
        m.set_str("Text", Some(b));
        self.send_if_connected(m)
    }
    /// `CGun::GetSquelchList` (0x1000e820): 0x5107.
    fn get_squelch_list(&self, context: Context) -> Hresult {
        self.plain_request(0x5107, context)
    }
    /// `CGun::SquelchAdd` (0x1000e8b0): 0x5108.
    fn squelch_add(&self, a: &str, context: Context) -> Hresult {
        self.user_request(0x5108, "User", a, context)
    }
    /// `CGun::SquelchDelete` (0x1000e950): 0x5109 (no NULL test in retail).
    fn squelch_delete(&self, a: &str, context: Context) -> Hresult {
        self.user_request(0x5109, "User", a, context)
    }
    /// `CGun::CreateRoom` (0x1000e9e0): a zero limit (`p4`) is `0x80070057`; `<CreateRoom Room Topic
    /// Mode Limit Password>` (0x510a).
    fn create_room(&self, a: &str, b: &str, c: u32, d: u32, e: &str, context: Context) -> Hresult {
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        if d == 0 {
            return E_INVALIDARG;
        }
        let mut m = self.new_request(0x510a, context);
        m.set_str("Room", Some(a));
        m.set_str("Topic", Some(b));
        m.set_dec("Mode", c as i32);
        m.set_dec("Limit", d as i32);
        m.set_str("Password", Some(e));
        self.send_request(&m)
    }
    /// `CGun::GetRoomOwner` (0x1000eaa0): `<RoomOwner RoomId>` (0x510b).
    fn get_room_owner(&self, a: u32, context: Context) -> Hresult {
        let mut m = self.new_request(0x510b, context);
        m.set_dec("RoomId", a as i32);
        self.send_if_connected(m)
    }
    /// `CGun::AssignOwnership` (0x1000eb30): `<AssignOwnership RoomId User>` (0x510c).
    fn assign_ownership(&self, a: u32, b: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x510c, context);
        m.set_dec("RoomId", a as i32);
        m.set_str("User", Some(b));
        self.send_if_connected(m)
    }
    /// `CGun::SetRoomTopic` (0x1000ebd0): `<SetRoomTopic RoomId Text>` (0x510e).
    fn set_room_topic(&self, a: u32, b: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x510e, context);
        m.set_dec("RoomId", a as i32);
        m.set_str("Text", Some(b));
        self.send_if_connected(m)
    }
    /// `CGun::SetRoomMode` (0x1000ec70): `<SetRoomMode RoomId Mode Password>` (0x510d).
    fn set_room_mode(&self, a: u32, b: u32, c: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x510d, context);
        m.set_dec("RoomId", a as i32);
        m.set_dec("Mode", b as i32);
        m.set_str("Password", Some(c));
        self.send_if_connected(m)
    }
    /// `CGun::InviteUser` (0x1000ed10): `<InviteUser RoomId User Text>` (0x510f).
    fn invite_user(&self, a: u32, b: &str, c: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x510f, context);
        m.set_dec("RoomId", a as i32);
        m.set_str("User", Some(b));
        m.set_str("Text", Some(c));
        self.send_if_connected(m)
    }
    /// `CGun::KickOutUser` (0x1000edc0): `<KickOutUser RoomId User Text>` (0x5110).
    fn kick_out_user(&self, a: u32, b: &str, c: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x5110, context);
        m.set_dec("RoomId", a as i32);
        m.set_str("User", Some(b));
        m.set_str("Text", Some(c));
        self.send_if_connected(m)
    }
    /// `CGun::RetrieveChatLog` (0x1000ee70): `<RetrieveChatLog Persona1 Persona2 MaxChatLogLength>`
    /// (0x511e) with the constant 30000.
    fn retrieve_chat_log(&self, a: &str, b: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x511e, context);
        m.set_str("Persona1", Some(a));
        m.set_str("Persona2", Some(b));
        m.set_dec("MaxChatLogLength", 30000);
        self.send_if_connected(m)
    }
    /// `CGun::SendToRoomEx` (0x1000e5f0): `<SendToRoomEx RoomId Text><Extra/>` (0x511c). A blob with a
    /// length but no data is `0x80070057` in retail; a slice always has its data.
    fn send_to_room_ex(
        &self,
        room_id: u32,
        message: &str,
        blob: &[u8],
        context: Context,
    ) -> Hresult {
        let mut m = self.new_request(0x511c, context);
        m.set_dec("RoomId", room_id as i32);
        m.set_str("Text", Some(message));
        m.set_blob("Extra", Some(blob));
        self.send_if_connected(m)
    }
    /// `CGun::SendToUserEx` (0x1000e760): `<SendToUserEx User Text><Extra/>` (0x511d).
    fn send_to_user_ex(&self, user: &str, message: &str, blob: &[u8], context: Context) -> Hresult {
        let mut m = self.new_request(0x511d, context);
        m.set_str("User", Some(user));
        m.set_str("Text", Some(message));
        m.set_blob("Extra", Some(blob));
        self.send_if_connected(m)
    }
}

impl IGunSupport for CGun {
    /// `CGun::CallCreate` (0x1000ef20): NULL strings are `L""` and a NULL blob an empty one; the
    /// target is at most 50 and the description at most 250 characters (`ZT_E_STR_TOO_LONG`), the
    /// blob at most 5000 bytes, the type word must satisfy `(t & 0x2000) == 0 || (t & 0x4000) != 0`,
    /// `(t & 0x8000) == 0 || description non-empty`, `(t & 0x4000) == 0 || target non-empty` and
    /// `(t & 0xff) < 9` (all `0x80070057`), in that order; then `<CallCreate CallType
    /// TargetPersonaName Description LangID><CallData/>` (0x10100).
    fn call_create(
        &self,
        call_type: SupportCallType,
        a: Option<&str>,
        b: Option<&str>,
        c: &[u8],
        d: u32,
        context: Context,
    ) -> Hresult {
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        let target = a.unwrap_or("");
        let description = b.unwrap_or("");
        let t = call_type.0;
        if wcslen(target) >= 0x33 {
            return ZT_E_STR_TOO_LONG;
        }
        if wcslen(description) >= 0xfb {
            return ZT_E_STR_TOO_LONG;
        }
        if c.len() >= 0x1389 {
            return E_INVALIDARG;
        }
        if (t & 0x2000) != 0 && (t & 0x4000) == 0 {
            return E_INVALIDARG;
        }
        if (t & 0x8000) != 0 && wcslen(description) == 0 {
            return E_INVALIDARG;
        }
        if (t & 0x4000) != 0 && wcslen(target) == 0 {
            return E_INVALIDARG;
        }
        if (t & 0xff) >= 9 {
            return E_INVALIDARG;
        }
        let mut m = self.new_request(0x10100, context);
        m.set_dec("CallType", t as i32);
        m.set_str("TargetPersonaName", Some(target));
        m.set_str("Description", Some(description));
        m.set_blob("CallData", Some(c));
        m.set_dec("LangID", d as i32);
        self.send_request(&m)
    }
    /// `CGun::CallGetStatus` (0x1000f0e0): `<CallGetStatus QueueID>` (0x10101).
    fn call_get_status(&self, a: u32, context: Context) -> Hresult {
        let mut m = self.new_request(0x10101, context);
        m.set_dec("QueueID", a as i32);
        self.send_if_connected(m)
    }
}

impl IGunPersona for CGun {
    /// `CGun::EnterPersona` (0x1000b2e0): not initialised -> `ZT_E_NOT_INITED`; then the connection;
    /// `<EnterPersonaCmd Name Shard>` (0x8100).
    fn enter_persona(&self, a: &str, b: u32, context: Context) -> Hresult {
        if !self.inited() {
            return ZT_E_NOT_INITED;
        }
        let mut m = self.new_request(0x8100, context);
        m.set_str("Name", Some(a));
        m.set_dec("Shard", b as i32);
        self.send_if_connected(m)
    }
    /// `CGun::LeavePersona` (0x1000b390): `<LeavePersonaCmd>` (0x8101).
    fn leave_persona(&self, context: Context) -> Hresult {
        self.plain_request(0x8101, context)
    }
    /// `CGun::CreatePersona` (0x1000b420): `<CreatePersonaCmd Name Type Shard>` (0x8102) with Type 1.
    fn create_persona(&self, a: &str, b: u32, context: Context) -> Hresult {
        let mut m = self.new_request(0x8102, context);
        m.set_str("Name", Some(a));
        m.set_dec("Type", 1);
        m.set_dec("Shard", b as i32);
        self.send_if_connected(m)
    }
    /// `CGun::DeletePersona` (0x1000b4c0): `<DeletePersonaCmd Name Shard>` (0x8103).
    fn delete_persona(&self, a: &str, b: u32, context: Context) -> Hresult {
        let mut m = self.new_request(0x8103, context);
        m.set_str("Name", Some(a));
        m.set_dec("Shard", b as i32);
        self.send_if_connected(m)
    }
    /// `CGun::GetPersonaList` (0x1000b560): `<GetPersonaListCmd Shard>` (0x8104).
    fn get_persona_list(&self, a: u32, context: Context) -> Hresult {
        let mut m = self.new_request(0x8104, context);
        m.set_dec("Shard", a as i32);
        self.send_if_connected(m)
    }
}

impl IGunBrowser for CGun {
    /// `CGun::GetViewList` (0x1000bff0): `<ViewList>` (0x7102).
    fn get_view_list(&self, context: Context) -> Hresult {
        self.plain_request(0x7102, context)
    }
    /// `CGun::GetHeaderRowOfView` (0x1000c080): `<HdrRow Vid>` (0x7103).
    fn get_header_row_of_view(&self, a: u32, context: Context) -> Hresult {
        let mut m = self.new_request(0x7103, context);
        m.set_dec("Vid", a as i32);
        self.send_if_connected(m)
    }
    /// `CGun::GetPage` (0x1000c110): `<Page Vid PageNo Num Str>` (0x7104); a NULL string is `L""`.
    fn get_page(&self, a: u32, b: u32, c: u32, d: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x7104, context);
        m.set_dec("Vid", a as i32);
        m.set_dec("PageNo", b as i32);
        m.set_dec("Num", c as i32);
        m.set_str("Str", Some(d));
        self.send_if_connected(m)
    }
    /// `CGun::GetRowPage` (0x1000c1c0): `<RowPg Vid Rid Num Str>` (0x7105).
    fn get_row_page(&self, a: u32, b: u32, c: u32, d: &str, context: Context) -> Hresult {
        let mut m = self.new_request(0x7105, context);
        m.set_dec("Vid", a as i32);
        m.set_dec("Rid", b as i32);
        m.set_dec("Num", c as i32);
        m.set_str("Str", Some(d));
        self.send_if_connected(m)
    }
}
