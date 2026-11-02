//! Polling and routing: `CheckForMessages`, `Read`, `RcvEvent`, `RcvResult`, the listener
//! registration and `UnRegisterEvents`.

use std::rc::Rc;

use super::TurbineGun;
use crate::ct::{CtChatEvent, CtConnectEvent, CtFriendsEvent};
use crate::interfaces::IGunQueueRead;
use crate::message::{GunQueueMessage, Message};
use crate::turbine_gun_utils::print_cmd_hresult;
use crate::zt::ZT_E_INTERNAL;
use crate::{Hresult, MessageKind};

impl TurbineGun {
    /// `NSTurbineGun::TurbineGun::CheckForMessages` (admin 0x006049F0, Legions 0x005C58D0): with
    /// an `m_pGunConnect`, `hr = IGunConnection::DispatchQueue(this as IGUNQueueRead)`; `hr != 0`
    /// is `PrintCmdHRESULT("DispatchQueue", hr)` and returns 0; otherwise (also without a
    /// connection object) returns 1. `gmClient::UseTime` calls it every frame.
    ///
    /// `DispatchQueue` re-enters this object and the GunDll object from `Read` (the reconnect
    /// `Connect`, the RECONNECTED handler's `GetTicket` / `PresentTicket`), in retail's order:
    /// no borrow is held across the call.
    pub fn check_for_messages(&self) -> bool {
        let Some(gun) = self.gun_rc() else {
            return true;
        };
        let hr = gun.dispatch_queue(self);
        if hr != Hresult::S_OK {
            print_cmd_hresult("DispatchQueue", hr);
            return false;
        }
        true
    }

    /// `NSTurbineGun::TurbineGun::Read` (admin 0x00604A80, Legions 0x005C5960; vtable slot 0):
    /// the sink `DispatchQueue` calls once per queued message. `obj = msg->pvBuffer`; by
    /// `obj->m_msgId & 0xF00`: 0x200 (a result) is `RcvResult`, 0x300 (an event) is `RcvEvent`,
    /// anything else `ZT_E_INTERNAL`. The other `GUNQueueMessage` members are not read.
    pub fn read_message(&self, msg: &GunQueueMessage) -> Hresult {
        match msg.message.id.kind() {
            MessageKind::Result => self.rcv_result(&msg.message),
            MessageKind::Event => self.rcv_event(&msg.message),
            _ => ZT_E_INTERNAL,
        }
    }

    /// `NSTurbineGun::TurbineGun::RcvEvent` (admin 0x00604AD0, Legions 0x005C59B0; slot 3): by
    /// `m_msgId & 0xFF000`: 0x1000 `RcvConnectEvent`, 0x2000 `RcvAccountsEvent`, 0x4000
    /// `RcvFriendsEvent`, 0x5000 `RcvChatEvent`, 0x8000 `RcvPersonaEvent`, else `ZT_E_INTERNAL`.
    pub fn rcv_event(&self, ev: &Message) -> Hresult {
        match ev.id.family() {
            0x1000 => self.rcv_connect_event(ev),
            0x2000 => self.rcv_accounts_event(ev),
            0x4000 => self.rcv_friends_event(ev),
            0x5000 => self.rcv_chat_event(ev),
            0x8000 => self.rcv_persona_event(ev),
            _ => ZT_E_INTERNAL,
        }
    }

    /// `NSTurbineGun::TurbineGun::RcvResult` (admin 0x00605900, Legions 0x005C67E0; slot 9).
    ///
    /// 1. With `m_bInited` and a nonzero `m_Context`: `ctx->m_hResult = res->m_hr` and
    ///    `ctx->IncEventsHandled()`.
    /// 2. By `m_msgId & 0xFF000`: 0x2000 `RcvAccountsResult`, 0x3000 `RcvNewsResult`, 0x4000
    ///    `RcvFriendsResult`, 0x5000 `RcvChatResult`, 0x7000 `RcvMatchResult`, 0x8000
    ///    `RcvPersonaResult`, 0xF000 `RcvAuthenticateResult`, 0x10000 `RcvSupportResult`, else
    ///    `ZT_E_INTERNAL` (0x1000 and 0x6000 have no handler).
    /// 3. With a context: releases the reference `Request*` added (the object is deleted at
    ///    count 0).
    /// 4. Returns the handler's status.
    ///
    /// A context id with no pending entry reads as NULL (retail would use an unowned pointer).
    pub fn rcv_result(&self, res: &Message) -> Hresult {
        let mut pending_id = None;
        if res.inited {
            if let Some(header) = res.result {
                if header.context.0 != 0 {
                    let found = self.contexts.borrow().get(&header.context.0).cloned();
                    if let Some(ctx) = found {
                        ctx.as_result().set_h_result(header.hr);
                        ctx.as_result().inc_events_handled();
                        pending_id = Some(header.context.0);
                    }
                }
            }
        }
        let status = match res.id.family() {
            0x2000 => self.rcv_accounts_result(res),
            0x3000 => self.rcv_news_result(res),
            0x4000 => self.rcv_friends_result(res),
            0x5000 => self.rcv_chat_result(res),
            0x7000 => self.rcv_match_result(res),
            0x8000 => self.rcv_persona_result(res),
            0xF000 => self.rcv_authenticate_result(res),
            0x10000 => self.rcv_support_result(res),
            _ => ZT_E_INTERNAL,
        };
        if let Some(id) = pending_id {
            self.contexts.borrow_mut().remove(&id);
        }
        status
    }

    /// `NSTurbineGun::TurbineGun::RegisterForConnectEvents` (admin 0x00605E00, Legions
    /// 0x005C6CE0): NULL -> 0; `AddRef(ev)`; `List::insert(list, list.head, &ev)` puts it before
    /// the head, so the newest listener is first; returns the insert's result (1; 0 only if the
    /// node allocation fails, which Rust does not report).
    pub fn register_for_connect_events(&self, ev: Option<Rc<dyn CtConnectEvent>>) -> bool {
        let Some(ev) = ev else {
            return false;
        };
        self.connect_event_list.borrow_mut().insert(0, ev);
        true
    }

    /// `NSTurbineGun::TurbineGun::RegisterForChatEvents` (admin 0x00605E40, Legions 0x005C6D20):
    /// the same for `m_ChatEventList` (+0x18).
    pub fn register_for_chat_events(&self, ev: Option<Rc<dyn CtChatEvent>>) -> bool {
        let Some(ev) = ev else {
            return false;
        };
        self.chat_event_list.borrow_mut().insert(0, ev);
        true
    }

    /// `NSTurbineGun::TurbineGun::RegisterForFriendsEvents` (admin 0x00605E80, Legions
    /// 0x005C6D60): the same for `m_FriendsEventList` (+0x28).
    pub fn register_for_friends_events(&self, ev: Option<Rc<dyn CtFriendsEvent>>) -> bool {
        let Some(ev) = ev else {
            return false;
        };
        self.friends_event_list.borrow_mut().insert(0, ev);
        true
    }

    /// `NSTurbineGun::TurbineGun::UnRegisterEvents` (admin 0x00605A10, Legions 0x005C68F0):
    /// releases one reference of every listener in the three lists (deleting at 0). Retail leaves
    /// the list nodes for the destructor's flush; only the destructor calls this, so the lists
    /// are emptied here.
    pub fn un_register_events(&self) {
        let connect = std::mem::take(&mut *self.connect_event_list.borrow_mut());
        let chat = std::mem::take(&mut *self.chat_event_list.borrow_mut());
        let friends = std::mem::take(&mut *self.friends_event_list.borrow_mut());
        drop((connect, chat, friends));
    }
}

impl IGunQueueRead for TurbineGun {
    /// `IGUNQueueRead::Read`, [`TurbineGun::read_message`].
    fn read(&self, message: &GunQueueMessage) -> Hresult {
        self.read_message(message)
    }
}
