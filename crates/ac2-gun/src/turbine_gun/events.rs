//! The chat, friends, accounts and persona event handlers.
//!
//! open: every event accessor (`GetCount`, `GetUserName`, ...) returns `ZT_E_NOT_INITED` while
//! `m_bInited` is 0 and leaves its out-variable alone, so retail's handlers go on with
//! uninitialised stack values (`0c_client.md` section 3.3, the "latent bug"). Those values are
//! not determinable, so a handler for an event that is not inited, or whose body is not its
//! class's, does nothing here.

use super::TurbineGun;
use crate::message::{Message, MessageBody};
use crate::zt::ZT_E_INTERNAL;
use crate::Hresult;

impl TurbineGun {
    /// `NSTurbineGun::TurbineGun::RcvChatEvent` (admin 0x00606710, Legions 0x005C75B0; slot 5).
    /// Always returns 0; ids outside 0x5314-0x531A, and no listeners, do nothing. For every
    /// listener of `m_ChatEventList` (newest first) `IncEventsHandled()` then the call:
    ///
    /// | id | call |
    /// |---|---|
    /// | 0x5314 RoomEnter | `+0x0C (roomId, name)` |
    /// | 0x5315 RoomLeave | `+0x10 (roomId, name)` |
    /// | 0x5316 RoomSpeak | `+0x14 HearRoom(roomId, user, speech, extra)` |
    /// | 0x5317 UserSpeak | `+0x20 HearUser(user, speech, extra)`, the user NOT stripped |
    /// | 0x5318 KickedOut | `+0x18 (roomId, reason)` |
    /// | 0x5319 ReceiveOwnership | `+0x1C (roomId)` |
    /// | 0x531A Invited | `+0x08 (roomName, invitation)` (`m_UserName` ignored) |
    ///
    /// The user name of Enter, Leave and RoomSpeak goes through `GlobalToLocalName` when
    /// `S_fStripWorldNameFromRoomChatEvents` is set, else a copy. `m_Type` is ignored; 0x531B
    /// UserSay is not handled. `extra` is the event's `tagBLOB*` as it is.
    pub fn rcv_chat_event(&self, ev: &Message) -> Hresult {
        if !ev.inited {
            return Hresult::S_OK;
        }
        let strip = Self::strip_world_name_from_room_chat_events();
        let user_name = |name: &str| {
            if strip {
                self.global_to_local_name(name)
            } else {
                name.to_owned()
            }
        };
        let listeners = self.chat_event_list.borrow().clone();
        match &ev.body {
            MessageBody::RoomEnter {
                room_id,
                user_name: n,
            } if ev.id.0 == 0x5314 => {
                let name = user_name(n);
                for l in &listeners {
                    l.inc_events_handled();
                    l.room_enter(*room_id, &name);
                }
            }
            MessageBody::RoomLeave {
                room_id,
                user_name: n,
            } if ev.id.0 == 0x5315 => {
                let name = user_name(n);
                for l in &listeners {
                    l.inc_events_handled();
                    l.room_leave(*room_id, &name);
                }
            }
            MessageBody::RoomSpeak {
                room_id,
                user_name: n,
                text,
                extra,
                ..
            } if ev.id.0 == 0x5316 => {
                let user = user_name(n);
                for l in &listeners {
                    l.inc_events_handled();
                    l.hear_room(*room_id, &user, text, extra.as_ref());
                }
            }
            MessageBody::UserSpeak {
                user_name: n,
                text,
                extra,
                ..
            } if ev.id.0 == 0x5317 => {
                for l in &listeners {
                    l.inc_events_handled();
                    l.hear_user(n, text, extra.as_ref());
                }
            }
            MessageBody::KickedOut { room_id, reason } if ev.id.0 == 0x5318 => {
                for l in &listeners {
                    l.inc_events_handled();
                    l.kicked_out(*room_id, reason);
                }
            }
            MessageBody::ReceiveOwnership { room_id } if ev.id.0 == 0x5319 => {
                for l in &listeners {
                    l.inc_events_handled();
                    l.receive_ownership(*room_id);
                }
            }
            MessageBody::Invited {
                room_name,
                invitation,
                ..
            } if ev.id.0 == 0x531a => {
                for l in &listeners {
                    l.inc_events_handled();
                    l.invited(room_name, invitation);
                }
            }
            _ => {}
        }
        Hresult::S_OK
    }

    /// `NSTurbineGun::TurbineGun::RcvFriendsEvent` (admin 0x00606C20, Legions 0x005C7AC0; slot
    /// 6). Always returns 0.
    ///
    /// * 0x430C `FriendStart`: with a count other than 0 and listeners, every listener of
    ///   `m_FriendsEventList` gets `IncEventsHandled()` then `FriendStart(count, array)` (`+8`).
    ///   A count of 0 DROPS the event: no empty list is ever delivered.
    /// * 0x430D `FriendStatus`: every listener gets `IncEventsHandled()` then
    ///   `FriendStatus(GlobalToLocalName(name), mode)` (`+0xC`), the name always stripped (not
    ///   gated by `S_fStripWorldNameFromRoomChatEvents`). Afterwards `GetModeText(buf, mode, 0xFF)`
    ///   is computed and discarded (not reproduced).
    /// * 0x430E `FriendRequest`, 0x430F `FriendAddComplete` and any other id: ignored.
    pub fn rcv_friends_event(&self, ev: &Message) -> Hresult {
        if !ev.inited {
            return Hresult::S_OK;
        }
        let listeners = self.friends_event_list.borrow().clone();
        match &ev.body {
            MessageBody::FriendStart(statuses) if ev.id.0 == 0x430c => {
                if !statuses.is_empty() {
                    for l in &listeners {
                        l.inc_events_handled();
                        l.friend_start(statuses);
                    }
                }
            }
            MessageBody::FriendStatus { user_name, mode } if ev.id.0 == 0x430d => {
                let name = self.global_to_local_name(user_name);
                for l in &listeners {
                    l.inc_events_handled();
                    l.friend_status(&name, *mode);
                }
            }
            _ => {}
        }
        Hresult::S_OK
    }

    /// `NSTurbineGun::TurbineGun::RcvAccountsEvent` (admin 0x00604B70, Legions 0x005C5A50; slot
    /// 7): 0x2301 `LogoutEv` returns 0 (the event is not used), else `ZT_E_INTERNAL`.
    pub fn rcv_accounts_event(&self, ev: &Message) -> Hresult {
        if ev.id.0 == 0x2301 {
            Hresult::S_OK
        } else {
            ZT_E_INTERNAL
        }
    }

    /// `NSTurbineGun::TurbineGun::RcvPersonaEvent` (admin 0x00604B90, Legions 0x005C5A70; slot
    /// 8): 0x8300 and 0x8301 return 0, else `ZT_E_INTERNAL`. No other effect.
    pub fn rcv_persona_event(&self, ev: &Message) -> Hresult {
        if ev.id.0 == 0x8300 || ev.id.0 == 0x8301 {
            Hresult::S_OK
        } else {
            ZT_E_INTERNAL
        }
    }
}
