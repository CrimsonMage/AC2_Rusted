//! The `Rcv*Result` handlers (vtable slots 10-17), which `RcvResult` routes by message family.
//!
//! Common prologue of every handler: with `m_bInited == 0`, `hr` and the returned status are both
//! `ZT_E_NOT_INITED` (0xC1120008); else `hr = res->m_hr` and the status is 0. A handler returns
//! the status unless it says otherwise. `PrintCmdHRESULT` is applied to `hr` for diagnostics
//! (console output only, not reproduced). The context a handler calls is the pending entry for
//! `res->m_Context` (`GetContext`), and only for an inited message. A context of another class
//! than the id's (retail would call whatever object the pointer is) is not called.

use super::TurbineGun;
use crate::ct::{ETicketResult, PendingContext};
use crate::message::{Message, MessageBody};
use crate::zt::{
    ZT_E_ACCOUNT_BANNED, ZT_E_ALREADY_TICKETED, ZT_E_INTERNAL, ZT_E_MAC_BANNED, ZT_E_NOT_INITED,
    ZT_E_NOT_TICKETED,
};
use crate::Hresult;

impl TurbineGun {
    /// The prologue: `(hr, status)`.
    fn result_prologue(res: &Message) -> (Hresult, Hresult) {
        if !res.inited {
            (ZT_E_NOT_INITED, ZT_E_NOT_INITED)
        } else {
            // A result always carries its header; none reads as the zeroed one.
            (res.result.map_or(Hresult::S_OK, |h| h.hr), Hresult::S_OK)
        }
    }

    /// `Gun::CBaseResult::GetContext`: the pending context of an inited result.
    fn result_context(&self, res: &Message) -> Option<PendingContext> {
        if !res.inited {
            return None;
        }
        let id = res.result?.context.0;
        if id == 0 {
            return None;
        }
        self.contexts.borrow().get(&id).cloned()
    }

    /// `NSTurbineGun::TurbineGun::RcvAccountsResult` (admin 0x00604BB0, Legions 0x005C5A90; slot
    /// 10): prints "Getting account event"; 0x2200 (`LoginRes`) sets `m_fLoggedIn = 0`; returns
    /// the prologue status.
    pub fn rcv_accounts_result(&self, res: &Message) -> Hresult {
        let (_hr, status) = Self::result_prologue(res);
        if res.id.0 == 0x2200 {
            self.logged_in.set(false);
        }
        status
    }

    /// `NSTurbineGun::TurbineGun::RcvNewsResult` (admin 0x00604CA0, Legions 0x005C5B80; slot
    /// 11): 0x3200 prints "GetNews", any other id "UnknownNewsResult"; returns the prologue
    /// status.
    pub fn rcv_news_result(&self, res: &Message) -> Hresult {
        Self::result_prologue(res).1
    }

    /// `NSTurbineGun::TurbineGun::RcvMatchResult` (admin 0x00605440, Legions 0x005C6320; slot
    /// 14): 0x7202 GetViewList, 0x7203 GetHeaderRow, 0x7204 GetPage, 0x7205 GetRowPage print only;
    /// returns the prologue status. The client never requests these.
    pub fn rcv_match_result(&self, res: &Message) -> Hresult {
        Self::result_prologue(res).1
    }

    /// `NSTurbineGun::TurbineGun::RcvSupportResult` (admin 0x00604C10, Legions 0x005C5AF0; slot
    /// 17). `hr = inited ? res->m_hr : ZT_E_NOT_INITED`. 0x10200 `CallCreateRes`: `queueId`,
    /// `queuePos` are `m_QueueID` (+0x24) and `m_QueuePos` (+0x28) when `hr == 0` and inited,
    /// else 0, 0; with an inited message and a context, `ctx.vtbl[+8](queueId, queuePos)`
    /// (`CCliCallCreateResult::HandleCallCreateResult`). 0x10201 `CallGetStatusRes` prints.
    /// Returns 0 always.
    pub fn rcv_support_result(&self, res: &Message) -> Hresult {
        let (hr, _status) = Self::result_prologue(res);
        if res.id.0 == 0x10200 {
            let (queue_id, queue_pos) = match &res.body {
                MessageBody::CallCreate {
                    queue_id,
                    queue_pos,
                } if hr == Hresult::S_OK && res.inited => (*queue_id, *queue_pos),
                _ => (0, 0),
            };
            if let Some(PendingContext::CallCreate(ctx)) = self.result_context(res) {
                ctx.handle_call_create_result(queue_id, queue_pos);
            }
        }
        Hresult::S_OK
    }

    /// `NSTurbineGun::TurbineGun::RcvChatResult` (admin 0x00604CF0, Legions 0x005C5BD0; slot 12):
    /// a jump table on `id - 0x5200`.
    ///
    /// | id | action |
    /// |---|---|
    /// | 0x5200 GetListOfRooms | print only (retail computes `GetRoomModeText` per room, debug) |
    /// | 0x5201 JoinRoom, 0x520A CreateRoom | `(roomId, serverId)` = `m_RoomId`, `m_ServerId` when `hr == 0`, else 0, 0; with a context `ctx.vtbl[+8](roomId, serverId)` |
    /// | 0x5202 LeaveRoom, 0x5203 LeaveAllRooms | with a context `ctx.vtbl[+8]()` |
    /// | 0x5204 GetUsersInRoom, 0x5207-0x5209 squelch, 0x520B-0x5210 | print only |
    /// | 0x5205 SendToRoom, 0x521C SendToRoomEx | with a context `ctx.vtbl[+8]()` |
    /// | 0x5206 SendToUser, 0x521D SendToUserEx | with a context `ctx.vtbl[+8]()` |
    /// | 0x5211 SayRes | nothing |
    /// | 0x5212 CreatePersistentChatChannel | with a context `ctx->m_hResult = hr`, `ctx.vtbl[+0xC](0)` |
    /// | 0x5213 DeletePersistentChatChannel | with a context `ctx->m_hResult = hr`, `ctx.vtbl[+8]()` |
    /// | 0x5214-0x521B, above 0x521D, below 0x5200 | status `ZT_E_INTERNAL` |
    ///
    /// Returns the prologue status otherwise.
    pub fn rcv_chat_result(&self, res: &Message) -> Hresult {
        let (hr, status) = Self::result_prologue(res);
        let room = || match &res.body {
            MessageBody::JoinRoom { room_id, server_id }
            | MessageBody::CreateRoom { room_id, server_id }
                if hr == Hresult::S_OK =>
            {
                (*room_id, *server_id)
            }
            _ => (0, 0),
        };
        match res.id.0 {
            0x5200 | 0x5204 | 0x5207..=0x5209 | 0x520b..=0x5211 => {}
            0x5201 => {
                let (room_id, server_id) = room();
                if let Some(PendingContext::JoinRoom(ctx)) = self.result_context(res) {
                    ctx.handle_join_room_result(room_id, server_id);
                }
            }
            0x520a => {
                let (room_id, server_id) = room();
                if let Some(PendingContext::CreateRoom(ctx)) = self.result_context(res) {
                    ctx.handle_create_room_result(room_id, server_id);
                }
            }
            0x5202 => {
                if let Some(PendingContext::LeaveRoom(ctx)) = self.result_context(res) {
                    ctx.handle_leave_room_result();
                }
            }
            0x5203 => {
                if let Some(PendingContext::LeaveAllRooms(ctx)) = self.result_context(res) {
                    ctx.handle_leave_all_rooms_result();
                }
            }
            0x5205 | 0x521c => {
                if let Some(PendingContext::SendToRoom(ctx)) = self.result_context(res) {
                    ctx.handle_send_to_room_result();
                }
            }
            0x5206 | 0x521d => {
                if let Some(PendingContext::SendToUser(ctx)) = self.result_context(res) {
                    ctx.handle_send_to_user_result();
                }
            }
            0x5212 => {
                if let Some(ctx) = self.result_context(res) {
                    ctx.as_result().set_h_result(hr);
                    if let PendingContext::CreatePersistentChatChannel(ctx) = ctx {
                        ctx.handle_create_persistent_chat_channel_result(0);
                    }
                }
            }
            0x5213 => {
                if let Some(ctx) = self.result_context(res) {
                    ctx.as_result().set_h_result(hr);
                    if let PendingContext::DeletePersistentChatChannel(ctx) = ctx {
                        ctx.handle_delete_persistent_chat_channel_result();
                    }
                }
            }
            _ => return ZT_E_INTERNAL,
        }
        status
    }

    /// `NSTurbineGun::TurbineGun::RcvFriendsResult` (admin 0x00605140, Legions 0x005C6020; slot
    /// 13): a jump table on `id - 0x4200`; ids 0x4206 and 0x4207 (and any other) are not
    /// handled, and the status is the prologue status for every id (unlike chat, never
    /// `ZT_E_INTERNAL`).
    ///
    /// | id | action |
    /// |---|---|
    /// | 0x4200 GetFriendsList | `(count, array)` from the message when `hr == 0`, else 0, none; with a context `ctx.vtbl[+8](count, array)` |
    /// | 0x4203 GetIgnoredList | the same with the names |
    /// | 0x4201 FriendAdd, 0x4202 FriendDelete, 0x4204 IgnoredAdd, 0x4205 IgnoredDelete, 0x420A SetLocation | with a context `ctx.vtbl[+8]()` |
    /// | 0x4208 Flags | with a context `ctx.vtbl[+8](m_WhichFlag, m_Flags)` (0, 0 unless `hr == 0`) |
    /// | 0x4209 WhereIs, 0x420B AcceptAddFriend | print only (the client never requests WhereIs) |
    pub fn rcv_friends_result(&self, res: &Message) -> Hresult {
        let (hr, status) = Self::result_prologue(res);
        let ok = hr == Hresult::S_OK;
        match res.id.0 {
            0x4200 => {
                let none = Vec::new();
                let statuses = match &res.body {
                    MessageBody::GetFriendsList(v) if ok => v,
                    _ => &none,
                };
                if let Some(PendingContext::GetFriendsList(ctx)) = self.result_context(res) {
                    ctx.handle_get_friends_list_result(statuses);
                }
            }
            0x4203 => {
                let none = Vec::new();
                let names = match &res.body {
                    MessageBody::GetIgnoredList(v) if ok => v,
                    _ => &none,
                };
                if let Some(PendingContext::GetIgnoredList(ctx)) = self.result_context(res) {
                    ctx.handle_get_ignored_list_result(names);
                }
            }
            0x4201 => {
                if let Some(PendingContext::FriendAdd(ctx)) = self.result_context(res) {
                    ctx.handle_friend_add_result();
                }
            }
            0x4202 => {
                if let Some(PendingContext::FriendDelete(ctx)) = self.result_context(res) {
                    ctx.handle_friend_delete_result();
                }
            }
            0x4204 => {
                if let Some(PendingContext::IgnoredAdd(ctx)) = self.result_context(res) {
                    ctx.handle_ignored_add_result();
                }
            }
            0x4205 => {
                if let Some(PendingContext::IgnoredDelete(ctx)) = self.result_context(res) {
                    ctx.handle_ignored_delete_result();
                }
            }
            0x420a => {
                if let Some(PendingContext::Location(ctx)) = self.result_context(res) {
                    ctx.handle_location_result();
                }
            }
            0x4208 => {
                let (which_flag, flags) = match &res.body {
                    MessageBody::Flags { which_flag, flags } if ok => (*which_flag, *flags),
                    _ => (0, 0),
                };
                if let Some(PendingContext::Flags(ctx)) = self.result_context(res) {
                    ctx.handle_flags_result(which_flag, flags);
                }
            }
            _ => {}
        }
        status
    }

    /// `NSTurbineGun::TurbineGun::RcvAuthenticateResult` (admin 0x006054D0, Legions 0x005C63B0;
    /// slot 16). `ctx = inited ? m_Context : NULL`, a `CCliTicketResult`, used WITHOUT a NULL
    /// check in retail (here no context means no call). `hr = inited ? m_hr :
    /// ZT_E_NOT_INITED`.
    ///
    /// * 0xF200 `CheckTicketRes`, 0xF201 `PresentTicketRes`: `ctx.vtbl[+0xC]` (`HandleAuthentication
    ///   Result`) with `ALREADY_TICKETED` -> `OLD_TICKET_STILL_EXISTS`, `ACCOUNT_BANNED` ->
    ///   `BANNED`, 0 -> `SUCCESS`, anything else `FAILURE`.
    /// * 0xF202 `ReleaseTicketRes`: `ctx.vtbl[+0x10]` with `NOT_TICKETED` -> `NO_TICKET`, 0 ->
    ///   `SUCCESS`, else `FAILURE`.
    /// * 0xF203 `GetTicketRes`: `hr != 0` -> `ctx.vtbl[+8]` (`HandleTicketArrived`)(`FAILURE`);
    ///   else the ticket accessor (inited, so 0), `ctx.CopyTicket(m_Ticket)` then
    ///   `ctx.vtbl[+8](SUCCESS)`. (Its `NOT_TICKETED` -> `NO_TICKET` and other-status ->
    ///   `FAILURE` branches are dead: the accessor only fails on an uninited message, which has
    ///   `hr != 0`.) The client has no request that produces 0xF203.
    /// * 0xF205 `SetGameServerRes`: the prologue status. 0xF204 and any other id:
    ///   `ZT_E_INTERNAL`.
    ///
    /// open: for 0xF203 retail passes `m_Ticket` to `CopyTicket` unchecked (a NULL ticket
    /// crashes it); a message without a ticket does nothing here.
    pub fn rcv_authenticate_result(&self, res: &Message) -> Hresult {
        let (hr, status) = Self::result_prologue(res);
        let ctx = match self.result_context(res) {
            Some(PendingContext::Ticket(ctx)) => Some(ctx),
            _ => None,
        };
        match res.id.0 {
            0xf200 | 0xf201 => {
                let result = if hr == ZT_E_ALREADY_TICKETED {
                    ETicketResult::OldTicketStillExists
                } else if hr == ZT_E_ACCOUNT_BANNED {
                    ETicketResult::Banned
                } else if hr == Hresult::S_OK {
                    ETicketResult::Success
                } else {
                    ETicketResult::Failure
                };
                if let Some(ctx) = ctx {
                    ctx.handle_authentication_result(result);
                }
            }
            0xf202 => {
                let result = if hr == ZT_E_NOT_TICKETED {
                    ETicketResult::NoTicket
                } else if hr == Hresult::S_OK {
                    ETicketResult::Success
                } else {
                    ETicketResult::Failure
                };
                if let Some(ctx) = ctx {
                    ctx.handle_release_ticket_result(result);
                }
            }
            0xf203 => {
                if let Some(ctx) = ctx {
                    if hr != Hresult::S_OK {
                        ctx.handle_ticket_arrived(ETicketResult::Failure);
                    } else if let MessageBody::GetTicket(Some(ticket)) = &res.body {
                        ctx.copy_ticket(ticket);
                        ctx.handle_ticket_arrived(ETicketResult::Success);
                    }
                }
            }
            0xf205 => {}
            _ => return ZT_E_INTERNAL,
        }
        status
    }

    /// `NSTurbineGun::TurbineGun::RcvPersonaResult` (admin 0x006071F0, Legions 0x005C8090; slot
    /// 15). `ctx = inited ? m_Context : NULL`, used without a NULL check in retail (here no
    /// context means no call). 0x8200 `EnterPersona`: `ctx.vtbl[+8](m_Type, m_ServerId)`
    /// (0, 0 unless `hr == 0`); 0x8201 `LeavePersona`: `+0xC`; 0x8202 `CreatePersona`: `+0x10`;
    /// 0x8203 `DeletePersona`: `+0x14`; 0x8204 `GetPersonaList`: `+0x18` with the names
    /// (`hr == 0`, else none); any other id `ZT_E_INTERNAL`; the prologue status otherwise.
    pub fn rcv_persona_result(&self, res: &Message) -> Hresult {
        let (hr, status) = Self::result_prologue(res);
        let ctx = match self.result_context(res) {
            Some(PendingContext::Persona(ctx)) => Some(ctx),
            _ => None,
        };
        match res.id.0 {
            0x8200 => {
                let (kind, server_id) = match &res.body {
                    MessageBody::EnterPersona { kind, server_id } if hr == Hresult::S_OK => {
                        (*kind, *server_id)
                    }
                    _ => (0, 0),
                };
                if let Some(ctx) = ctx {
                    ctx.handle_enter_persona_result(kind, server_id);
                }
            }
            0x8201 => {
                if let Some(ctx) = ctx {
                    ctx.handle_leave_persona_result();
                }
            }
            0x8202 => {
                if let Some(ctx) = ctx {
                    ctx.handle_create_persona_result();
                }
            }
            0x8203 => {
                if let Some(ctx) = ctx {
                    ctx.handle_delete_persona_result();
                }
            }
            0x8204 => {
                let none = Vec::new();
                let names = match &res.body {
                    MessageBody::GetPersonaList(v) if hr == Hresult::S_OK => v,
                    _ => &none,
                };
                if let Some(ctx) = ctx {
                    ctx.handle_get_persona_list_result(names);
                }
            }
            _ => return ZT_E_INTERNAL,
        }
        status
    }
}

// The failure code the client uses for "the ticket request failed" is MAC_BANNED's value.
const _: () = assert!(ETicketResult::Failure as u32 == ZT_E_MAC_BANNED.0);
