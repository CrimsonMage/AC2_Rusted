//! The retail globals the GUN classes reach: `NSTurbineGun::TurbineGun::S_pTurbineGun` (Legions
//! 0x00AF1B58, admin 0x00B9CF34), `S_fUseTurbineChatInsteadOfGUN` (Legions 0x00A51438, admin
//! bss; 0 until set), `ClientGunInterface::GetPlayerInterface()` (the player weenie's
//! `gmCWInterface`, `CObjectMaint::objMaint->player_id`) and the gmClient that receives the
//! notices. The client is single threaded, so the globals are per thread (the script VM runs on
//! the frame thread; a test thread gets its own).

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::{Rc, Weak};

use ac2_gun::TurbineGun;

use std::collections::HashMap;

use ac2_gun::ct::{Bullet, CtResult, CtSendToRoomResult};
use ac2_gun::Hresult;

use super::client::GunClient;
use super::handlers::CCliSendToRoomResult;
use super::output::{GunNotice, GunOutput};

/// A `CWBookKeeper::CSendToTurbineRoomByID` call `RequestGUN_SendToRoom` made (Legions
/// 0x00512250, admin 0x0052DDB0): the chat client is the session's, so the program carries it
/// out (`OnlineSession::send_to_room_with_extra`) and gives the chat request's context back
/// ([`GunShared::register_turbine_context`]).
#[derive(Clone)]
pub struct TurbineRoomSend {
    pub room: u32,
    pub text: String,
    /// The packed `GunChatExtraInfo` (`remoteBlob`).
    pub extra: Vec<u8>,
    /// The `CCliSendToRoomResult` the call passes as `localBlob {4, ctx}`.
    pub ctx: Rc<CCliSendToRoomResult>,
}

/// The GUN globals.
#[derive(Default)]
pub struct GunShared {
    gun: RefCell<Option<Rc<TurbineGun>>>,
    use_turbine_chat: Cell<bool>,
    outbox: RefCell<VecDeque<GunOutput>>,
    turbine_sends: RefCell<Vec<TurbineRoomSend>>,
    player: Cell<bool>,
    player_name: RefCell<String>,
    client: RefCell<Weak<GunClient>>,
    chat_client: Cell<bool>,
    turbine_contexts: RefCell<HashMap<u32, Rc<CCliSendToRoomResult>>>,
}

thread_local! {
    static SHARED: Rc<GunShared> = Rc::new(GunShared::default());
}

/// The thread's GUN globals.
pub fn shared() -> Rc<GunShared> {
    SHARED.with(Rc::clone)
}

impl GunShared {
    /// `S_pTurbineGun`: `None` is NULL (no `-gunserver`, or `Init` failed).
    pub fn turbine_gun(&self) -> Option<Rc<TurbineGun>> {
        self.gun.borrow().clone()
    }

    /// Sets `S_pTurbineGun`. Deviation: retail never clears it (not even in the destructor, so a
    /// deleted `TurbineGun` stays reachable); here `None` is stored when the object goes.
    pub fn set_turbine_gun(&self, gun: Option<Rc<TurbineGun>>) {
        *self.gun.borrow_mut() = gun;
    }

    /// `S_fUseTurbineChatInsteadOfGUN`.
    pub fn use_turbine_chat(&self) -> bool {
        self.use_turbine_chat.get()
    }

    pub fn set_use_turbine_chat(&self, on: bool) {
        self.use_turbine_chat.set(on);
    }

    /// `ClientGunInterface::GetPlayerInterface() != NULL`: the program sets it while the
    /// player's weenie exists.
    pub fn set_player(&self, present: bool) {
        self.player.set(present);
    }

    pub fn player_present(&self) -> bool {
        self.player.get()
    }

    /// `gmCWInterface::InqName` (string stat 0x100) of the player's weenie, the source of the
    /// meta letters (`GunChatExtraInfo::GunChatExtraInfo`).
    pub fn set_player_name(&self, name: &str) {
        *self.player_name.borrow_mut() = name.to_owned();
    }

    pub fn player_name(&self) -> String {
        self.player_name.borrow().clone()
    }

    /// The gmClient that has registered for the notices (`gmClient::Init`).
    pub fn set_client(&self, client: Weak<GunClient>) {
        *self.client.borrow_mut() = client;
    }

    /// Records a call for the program ([`GunOutput`]).
    pub fn push(&self, output: GunOutput) {
        self.outbox.borrow_mut().push_back(output);
    }

    /// Everything recorded since the last call, in order.
    pub fn take_outputs(&self) -> Vec<GunOutput> {
        self.outbox.borrow_mut().drain(..).collect()
    }

    /// `AC2ClientChatManager::GetIChatClient() != NULL`: the program sets it while the Turbine
    /// chat client exists (in the world).
    pub fn set_chat_client(&self, present: bool) {
        self.chat_client.set(present);
    }

    pub fn chat_client_present(&self) -> bool {
        self.chat_client.get()
    }

    /// The chat request `send` made got the session's context id `context`: its result is
    /// answered by [`GunShared::answer_turbine_send`].
    pub fn register_turbine_context(&self, context: u32, send: &TurbineRoomSend) {
        self.turbine_contexts
            .borrow_mut()
            .insert(context, Rc::clone(&send.ctx));
    }

    /// `gmChatInterfaceProvider::OnSendToRoomByIDResult` (admin 0x00402240): `ctx.m_hr = hr;`
    /// the context's handler (`CCliSendToRoomResult::HandleSendToRoomResult` ->
    /// `Result_SendToRoom`), then the context is released.
    pub fn answer_turbine_send(&self, context: u32, hr: u32) {
        let ctx = self.turbine_contexts.borrow_mut().remove(&context);
        if let Some(ctx) = ctx {
            ctx.set_h_result(Hresult(hr));
            ctx.inc_events_handled();
            ctx.handle_send_to_room_result();
        }
    }

    pub(super) fn push_turbine_send(&self, send: TurbineRoomSend) {
        self.turbine_sends.borrow_mut().push(send);
    }

    /// The `CSendToTurbineRoomByID` calls made since the last call.
    pub fn take_turbine_sends(&self) -> Vec<TurbineRoomSend> {
        std::mem::take(&mut *self.turbine_sends.borrow_mut())
    }

    /// `CM_UI::SendNotice_Gun*`: the handlers registered for the notice run at once, as retail's
    /// do. 0x46B and 0x46C go to the gmClient; 0x46A to `gmUIFlow::RecvNotice_GunConnectionLost`
    /// (recorded for the program, which owns the UI flow); 0x469 has no handler.
    pub fn send_notice(&self, notice: GunNotice) {
        let client = self.client.borrow().upgrade();
        match (notice, client) {
            (GunNotice::Reconnected, Some(client)) => client.recv_notice_gun_reconnected(),
            (GunNotice::Authenticated(ok), Some(client)) => {
                client.recv_notice_gun_authenticated(ok)
            }
            (GunNotice::ConnectionLost, _) => self.push(GunOutput::Notice(notice)),
            _ => {}
        }
    }
}
