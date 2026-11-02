//! The connection and the ticket: `RequestConnect`, `RcvConnectEvent` (with its reconnect
//! rule), `Set/GetAuthenticationTicket`, `RequestPresentAuthenticationTicket`,
//! `RequestFinishWithTicket`.

use std::rc::Rc;

use super::{TurbineGun, APP_GUID, E_PENDING};
use crate::ct::{CtTicketResult, EConnectEvent, PendingContext};
use crate::interfaces::TagBlob;
use crate::message::Message;
use crate::turbine_gun_utils::print_cmd_hresult;
use crate::zt::ZT_E_BUFFER_TOO_SMALL;
use crate::Hresult;

/// The dword `SetAuthenticationTicket` writes after the ticket bytes to see whether the DLL
/// overran the buffer: `ef fe ad de`.
const TICKET_CANARY: u32 = 0xDEAD_FEEF;

impl TurbineGun {
    /// `NSTurbineGun::TurbineGun::RequestConnect` (admin 0x00605AC0, Legions 0x005C69A0):
    /// `int RequestConnect(PStringBase<char>* server, ushort port, ushort portLocal,
    /// const wchar_t* gameVersion, const wchar_t* cdKey, ulong locale)`. `gmClient::StartupGUN`
    /// passes `L"1.0"`, `L"XXXX-YYYY-ZZZZ"` and locale 0.
    ///
    /// 1. `m_fPersistentConnection = 0` (before anything else).
    /// 2. No `m_pGunConnect`: return 0.
    /// 3. (`StringFromGUID2(app GUID, buf, 0x50)`: the result is unused.)
    /// 4. `locale == 0` -> `GetUserDefaultLCID()`.
    /// 5. `hr = IGunConnection::Init(appGuid, gameVersion, locale, cdKey, portLocal)`;
    ///    `PrintCmdHRESULT("Init", hr)` false (`hr < 0`) returns 0.
    /// 6. `hr = IGunConnection::Connect(server, port)`; `m_gunServerName = *server`;
    ///    `m_port = port`; `m_fConnecting = 1` (it stays 1 after a failed Connect). `E_PENDING`
    ///    returns 1; any other `hr` is `PrintCmdHRESULT("Connect", hr)` and returns 0.
    pub fn request_connect(
        &self,
        server: &str,
        port: u16,
        port_local: u16,
        game_version: &str,
        cd_key: &str,
        locale: u32,
    ) -> bool {
        self.persistent_connection.set(false);
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        let locale = if locale == 0 {
            self.user_default_lcid
        } else {
            locale
        };
        let hr = gun.init(&APP_GUID, game_version, locale, cd_key, port_local);
        if !print_cmd_hresult("Init", hr) {
            return false;
        }
        let hr = gun.connect(server, port);
        *self.gun_server_name.borrow_mut() = server.to_owned();
        self.port.set(port);
        self.connecting.set(true);
        if hr == E_PENDING {
            return true;
        }
        print_cmd_hresult("Connect", hr);
        false
    }

    /// `NSTurbineGun::TurbineGun::SetAuthenticationTicket` (admin 0x00605BA0, Legions
    /// 0x005C6A80): `int SetAuthenticationTicket(PStringBase<char> ticket)`, the command-line
    /// ticket string.
    ///
    /// No `m_pGunTicket`: return 0. `cb = ticket.m_len` (the character count plus the
    /// terminating NUL); `buf = new uchar[cb + 4]` holding the `cb` bytes (the NUL included) and
    /// then the dword `0xDEADFEEF`; `hr = IGunTicket::SetTicket({cb, buf})`. If the dword is no
    /// longer `0xDEADFEEF` (the DLL overran the buffer) return 0 (the buffer leaks); else free it
    /// and return `hr == 0`. The DLL is handed the `cb` bytes only, so the canary cannot change
    /// here.
    pub fn set_authentication_ticket(&self, ticket: &str) -> bool {
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        let cb = ticket.len() + 1;
        let mut buf = vec![0u8; cb + 4];
        buf[..ticket.len()].copy_from_slice(ticket.as_bytes());
        buf[cb..].copy_from_slice(&TICKET_CANARY.to_le_bytes());
        // `{cb, buf}`: the DLL sees `cb` bytes (its repack never writes past them) and the canary
        // dword follows them in the same allocation
        let mut blob = TagBlob {
            cb_size: cb as u32,
            data: Some(buf),
        };
        let hr = gun.set_ticket(&mut blob);
        let buf = blob.data.take().unwrap_or_default();
        let canary = u32::from_le_bytes([
            buf.get(cb).copied().unwrap_or(0),
            buf.get(cb + 1).copied().unwrap_or(0),
            buf.get(cb + 2).copied().unwrap_or(0),
            buf.get(cb + 3).copied().unwrap_or(0),
        ]);
        if canary != TICKET_CANARY {
            return false;
        }
        hr == Hresult::S_OK
    }

    /// `NSTurbineGun::TurbineGun::GetAuthenticationTicket` (admin 0x00604A20, Legions
    /// 0x005C5900): `int GetAuthenticationTicket(tagBLOB* pTicket)`. Needs `m_fConnected`,
    /// `m_pGunTicket` and `pTicket`, else 0. `hr = IGunTicket::GetTicket(pTicket)`; on
    /// `ZT_E_BUFFER_TOO_SMALL`: `pTicket->pBlobData = new uchar[pTicket->cbSize]` (no zero-size
    /// guard) and `GetTicket` again with the same blob. Returns `hr == 0` of the last call.
    pub fn get_authentication_ticket(&self, ticket: Option<&mut TagBlob>) -> bool {
        if !self.connected.get() {
            return false;
        }
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        let Some(ticket) = ticket else {
            return false;
        };
        let mut hr = gun.get_ticket(ticket);
        if hr == ZT_E_BUFFER_TOO_SMALL {
            ticket.data = Some(vec![0u8; ticket.cb_size as usize]);
            hr = gun.get_ticket(ticket);
        }
        hr == Hresult::S_OK
    }

    /// `NSTurbineGun::TurbineGun::RequestPresentAuthenticationTicket` (admin 0x00605640,
    /// Legions 0x005C6520): `int RequestPresentAuthenticationTicket(CTTicketResult* ctx)`. Needs
    /// `m_fConnected`, `m_pGunTicket`, `ctx` and `ctx->GetTicketBlob() != NULL`; then
    /// `AddRef(ctx)` and `hr = IGunTicket::PresentTicket(ctx->m_pTicket, ctx)`. Returns
    /// `hr == 0`; there is no `PrintCmdHRESULT` and a failing `hr` leaves the `AddRef` undone.
    pub fn request_present_authentication_ticket(
        &self,
        ctx: Option<Rc<dyn CtTicketResult>>,
    ) -> bool {
        if !self.connected.get() || self.gun_rc().is_none() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let Some(blob) = ctx.get_ticket_blob() else {
            return false;
        };
        let id = self.add_ref(PendingContext::Ticket(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        gun.present_ticket(blob.bytes(), id) == Hresult::S_OK
    }

    /// `NSTurbineGun::TurbineGun::RequestFinishWithTicket` (admin 0x006056A0, Legions
    /// 0x005C6580): needs `m_fConnected`, `m_pGunTicket` and `ctx`; `AddRef(ctx)`;
    /// `hr = IGunTicket::ReleaseTicket(ctx)`; returns `hr == 0`. Called from `~gmClient`.
    pub fn request_finish_with_ticket(&self, ctx: Option<Rc<dyn CtTicketResult>>) -> bool {
        if !self.connected.get() || self.gun_rc().is_none() {
            return false;
        }
        let Some(ctx) = ctx else {
            return false;
        };
        let id = self.add_ref(PendingContext::Ticket(ctx));
        let Some(gun) = self.gun_rc() else {
            return false;
        };
        gun.release_ticket(id) == Hresult::S_OK
    }

    /// Tells every `m_ConnectEventList` listener, newest first: `IncEventsHandled()` (vtable
    /// `+4`) then `HandleConnectEvent(code)` (`+8`).
    fn notify_connect(&self, code: EConnectEvent) {
        let listeners = self.connect_event_list.borrow().clone();
        for listener in listeners {
            listener.inc_events_handled();
            listener.handle_connect_event(code);
        }
    }

    /// `NSTurbineGun::TurbineGun::RcvConnectEvent` (admin 0x00605CA0, Legions 0x005C6B80; slot
    /// 4). Always returns 0.
    ///
    /// * 0x1300 `ConnectEv`: `m_fConnected = 1`; code `RECONNECTED` if `m_fPersistentConnection`
    ///   is set, else `CONNECT`; `m_fPersistentConnection = 1`; `m_fConnecting = 0`; notify.
    /// * 0x1301 `DisconnectEv`: `m_fConnected = 0`, `m_fLoggedIn = 0`; notify `DISCONNECT`; then
    ///   the reconnect rule.
    /// * 0x1302 `ConnectFailedEv`: `m_fConnected = 0`, `m_fLoggedIn = 0`; notify
    ///   `CONNECT_FAILED`; then the reconnect rule.
    /// * any other id: notify `UNDEF` (no state change).
    ///
    /// Reconnect rule (0x1301 and 0x1302): with `m_fPersistentConnection` set,
    /// `hr = IGunConnection::Connect(m_gunServerName, m_port)`, `m_fPersistentConnection = 1`,
    /// and an `hr` other than `E_PENDING` is `PrintCmdHRESULT("Connect", hr)`. There is no delay,
    /// retry counter or attempt limit; the next attempt is the next 0x1301/0x1302.
    /// `m_fConnecting` is not reset, and `IGunConnection::Init` is not called again.
    pub fn rcv_connect_event(&self, ev: &Message) -> Hresult {
        match ev.id.0 {
            0x1300 => {
                self.connected.set(true);
                let code = if self.persistent_connection.get() {
                    EConnectEvent::Reconnected
                } else {
                    EConnectEvent::Connect
                };
                self.persistent_connection.set(true);
                self.connecting.set(false);
                self.notify_connect(code);
            }
            0x1301 | 0x1302 => {
                self.connected.set(false);
                self.logged_in.set(false);
                let code = if ev.id.0 == 0x1301 {
                    EConnectEvent::Disconnect
                } else {
                    EConnectEvent::ConnectFailed
                };
                self.notify_connect(code);
                if self.persistent_connection.get() {
                    self.reconnect();
                }
            }
            _ => self.notify_connect(EConnectEvent::Undef),
        }
        Hresult::S_OK
    }

    /// The reconnect rule's `IGunConnection::Connect` call (`RcvConnectEvent`).
    pub(crate) fn reconnect(&self) {
        let Some(gun) = self.gun_rc() else {
            return;
        };
        let server = self.gun_server_name.borrow().clone();
        let hr = gun.connect(&server, self.port.get());
        self.persistent_connection.set(true);
        if hr != E_PENDING {
            print_cmd_hresult("Connect", hr);
        }
    }
}
