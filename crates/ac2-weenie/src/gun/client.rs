//! The GUN half of `gmClient` (`ac2/client/src/main/gmclient.cpp`): `Init`, `StartupGUN`,
//! `AuthenticateWithGUN`, `RegisterForGunEvents`, `InitAuth`, `Connect`'s order, `UseTime`'s poll,
//! the notice receivers and the destructor. Legions 0x004xxxxx and admin addresses are on each
//! item; where the builds differ Legions wins (no `GunTicketGen` block in `Init`, no
//! `FATAL ERROR` reports; the error results are the same).
//!
//! `StartupGUN` blocks in retail (`while (...) { KeepUIAlive(); CheckForMessages(); }`, no
//! timeout). Here the client is one frame loop, so [`GunClient::connect`] is that loop one
//! iteration per call and answers [`ConnectPoll::Pending`] while retail would still be spinning.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ac2_gun::ct::{CtConnectEvent, CtTicketResult, ETicketResult};
use ac2_gun::turbine_gun_utils::GunLoader;
use ac2_gun::{TagBlob, TurbineGun};
use ac2_net::Credential;

use super::handlers::{CCliChatEvent, CCliConnectEvent, CCliFriendsEvent, CCliTicketResult};
use super::options::GunOptions;
use super::output::GunOutput;
use super::shared::{shared, GunShared};

/// The UI string table every GUN error is in (`NetError::m_tableID`, enum 0x41000018).
pub const UI_TABLE: u32 = 0x4100_0018;
/// The table of the re-authentication strings (enum 0x4100000E).
pub const REAUTH_TABLE: u32 = 0x4100_000E;

/// A `NetError`: the string id by name (`Tokenize(name)`) and its table enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GunError {
    pub string: &'static str,
    pub table: u32,
}

/// `Tokenize("ID_DataPatch_GUNFailed")`, table 0x41000018: every `StartupGUN` and
/// `AuthenticateWithGUN` failure but a banned account.
pub const GUN_FAILED: GunError = GunError {
    string: "ID_DataPatch_GUNFailed",
    table: UI_TABLE,
};
/// A banned account (`ETicketResult::BANNED`, 0xC1121005).
pub const GUN_ACCOUNT_BANNED: GunError = GunError {
    string: "ID_DataPatch_GUNAccountBanned",
    table: UI_TABLE,
};
/// `AuthenticateWithGUN(true)`'s own failure string when it is on disk.
pub const GUN_REAUTH_FAILED: GunError = GunError {
    string: "ID_GunReAuthenticationFailed",
    table: REAUTH_TABLE,
};

/// What [`GunClient::connect`] answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectPoll {
    /// `StartupGUN` is still waiting (a connect event or the ticket answer): ask again next frame.
    Pending,
    /// `StartupGUN` succeeded and `InitAuth` made the logon's credential: `Turbine::Client::Connect`
    /// (`ClientNet::Init`) goes on.
    Ready(Credential),
    /// `StartupGUN` failed: `Connect` returns this `NetError`.
    Failed(GunError),
    /// `InitAuth` returned false: `Turbine::Client::Connect` returns the connection error in
    /// Legions `DAT_00AF596C/70` = `ID_ConnectionError_Generic`.
    InitAuthFailed,
}

enum Phase {
    /// `Connect` not called yet.
    Idle,
    /// `StartupGUN` step 6: waiting for the first connect event.
    WaitConnect(Rc<CCliConnectEvent>),
    /// Step 8: waiting for `PresentTicketRes`.
    WaitTicket,
    /// Finished (the answer was given).
    Done,
    /// `StartupGUN` failed: `gmClient::Connect` returned its `NetError` and never calls
    /// `Turbine::Client::Connect`. Every later `connect` answers the same.
    Failed(GunError),
    /// `InitAuth` returned false: `Connect` returned the connection error and stops there.
    InitAuthFailed,
}

/// `gmClient`'s GUN members and functions.
pub struct GunClient {
    options: GunOptions,
    /// `m_pPrimer` (+0x3C): the ticket text of `-glsticketdirect`, the account's primer.
    primer: String,
    /// `GetUserDefaultLCID()` for `TurbineGun::new`.
    lcid: u32,
    shared: Rc<GunShared>,
    loader: RefCell<Box<dyn GunLoader>>,
    /// `m_pTurbineGun` (+0xC0).
    gun: RefCell<Option<Rc<TurbineGun>>>,
    /// `m_gunPortLocal` (+0xD0).
    gun_port_local: Cell<u32>,
    /// `G_pGunTicketHandler` (Legions 0x00A44690): persists across re-authentications.
    ticket_handler: RefCell<Option<Rc<CCliTicketResult>>>,
    /// `m_fTryingReauthentication` (+0xD8).
    trying_reauthentication: Cell<bool>,
    phase: RefCell<Phase>,
    /// `StringInfo::IsOnDisk` for a string of a table enum.
    string_on_disk: RefCell<Box<dyn Fn(&str, u32) -> bool>>,
    shut_down: Cell<bool>,
}

impl GunClient {
    /// `gmClient`'s GUN state after the constructor and the command line. Call
    /// [`GunClient::init`] next (`gmClient::Init`).
    pub fn new(
        options: GunOptions,
        tchat: bool,
        primer: &str,
        user_default_lcid: u32,
        loader: Box<dyn GunLoader>,
    ) -> Rc<Self> {
        let shared = shared();
        let client = Rc::new(Self {
            options,
            primer: primer.to_owned(),
            lcid: user_default_lcid,
            shared,
            loader: RefCell::new(loader),
            gun: RefCell::new(None),
            gun_port_local: Cell::new(0),
            ticket_handler: RefCell::new(None),
            trying_reauthentication: Cell::new(false),
            phase: RefCell::new(Phase::Idle),
            string_on_disk: RefCell::new(Box::new(|_, _| false)),
            shut_down: Cell::new(false),
        });
        client.shared.set_client(Rc::downgrade(&client));
        if tchat {
            // `EvaluateCommandLineArg` for `-tchat` (admin 0x00403AE0, Legions 0x00403A50).
            client.shared.set_use_turbine_chat(true);
            TurbineGun::set_strip_world_name_from_room_chat_events(false);
        }
        client
    }

    /// The options.
    pub fn options(&self) -> &GunOptions {
        &self.options
    }

    /// `m_pTurbineGun`.
    pub fn turbine_gun(&self) -> Option<Rc<TurbineGun>> {
        self.gun.borrow().clone()
    }

    /// `G_pGunTicketHandler`.
    pub fn ticket_handler(&self) -> Option<Rc<CCliTicketResult>> {
        self.ticket_handler.borrow().clone()
    }

    /// How the program answers `StringInfo::IsOnDisk` for `(string, table enum)`.
    pub fn set_string_on_disk(&self, on_disk: Box<dyn Fn(&str, u32) -> bool>) {
        *self.string_on_disk.borrow_mut() = on_disk;
    }

    /// Whether `StartupGUN` is running (retail's `UseTime` does not run during its loops).
    pub fn starting_up(&self) -> bool {
        matches!(
            *self.phase.borrow(),
            Phase::WaitConnect(_) | Phase::WaitTicket
        )
    }

    /// Creates the `TurbineGun` and gives it to `S_pTurbineGun` (the constructor sets it).
    fn make_gun(&self) -> Option<Rc<TurbineGun>> {
        let gun = Rc::new(TurbineGun::new(self.lcid));
        self.shared.set_turbine_gun(Some(Rc::clone(&gun)));
        let ok = gun.init(
            &self.options.gun_client_dll,
            &mut **self.loader.borrow_mut(),
        );
        if ok {
            *self.gun.borrow_mut() = Some(Rc::clone(&gun));
            Some(gun)
        } else {
            // `delete g` (vtable slot 1): the destructor disconnects and releases. Retail
            // leaves `S_pTurbineGun` dangling; here it is NULL.
            self.shared.set_turbine_gun(None);
            None
        }
    }

    /// `gmClient::Init`'s GUN steps (admin 0x00404410, Legions 0x00404340):
    /// 2. with a `-gunserver`, `new TurbineGun` and `Init(m_gundll)`; a failure deletes it and
    ///    `m_pTurbineGun` is NULL (Legions reports nothing);
    /// 3. with a `TurbineGun` and a `-gunticketoverride`, `SetAuthenticationTicket(override)`
    ///    (the result is ignored). Legions has no `GunTicketGen.dll` block (admin step 4).
    ///
    /// Step 5 (the `LaunchTicketData` registry read and `CheckForBillingMigration` against
    /// `-migrationurl`) is not ported.
    // open: gmClient::Init step 5 (registry LaunchTicket billing-migration check, its HTTPS
    // call with migration1/2Timeout): Turbine's billing migration service is gone; the options
    // are parsed and kept, nothing reads them.
    pub fn init(&self) {
        if self.options.gun_enabled() {
            self.make_gun();
        }
        let gun = self.turbine_gun();
        if let Some(gun) = gun {
            if !self.options.gun_ticket_override.is_empty() {
                gun.set_authentication_ticket(&self.options.gun_ticket_override);
            }
        }
    }

    /// `gmClient::Connect` (admin 0x00404F80, Legions 0x00404D20) then the credential of
    /// `Turbine::Client::Connect` (`InitAuth`, admin 0x00404E10 / Legions 0x00404BB0): one
    /// iteration per call. `port_local_preference` is `Preferences::InqBool(0x4000000F,
    /// 0x41000001)` read when `m_gunPortLocal` is 0, `ui_alive` is `RealClient::KeepUIAlive()`.
    pub fn connect(&self, port_local_preference: &dyn Fn() -> u32, ui_alive: bool) -> ConnectPoll {
        match &*self.phase.borrow() {
            Phase::Failed(error) => return ConnectPoll::Failed(*error),
            Phase::InitAuthFailed => return ConnectPoll::InitAuthFailed,
            _ => {}
        }
        let begin = matches!(*self.phase.borrow(), Phase::Idle);
        if begin {
            if let Err(error) = self.startup_gun_begin(port_local_preference) {
                return self.fail(error);
            }
        }
        match self.startup_gun_poll(ui_alive) {
            Ok(true) => {}
            Ok(false) => return ConnectPoll::Pending,
            Err(error) => return self.fail(error),
        }
        *self.phase.borrow_mut() = Phase::Done;
        match self.init_auth() {
            Some(credential) => ConnectPoll::Ready(credential),
            None => {
                *self.phase.borrow_mut() = Phase::InitAuthFailed;
                ConnectPoll::InitAuthFailed
            }
        }
    }

    fn fail(&self, error: GunError) -> ConnectPoll {
        *self.phase.borrow_mut() = Phase::Failed(error);
        ConnectPoll::Failed(error)
    }

    /// `gmClient::StartupGUN` steps 1-5 (admin 0x00404A40, Legions 0x00404820): GUN skipped
    /// without a `-gunserver`; a `TurbineGun` is made if `Init` did not; a `CCliConnectEvent`
    /// is registered (on every call: the listeners accumulate); `RequestConnect(server,
    /// gunPort, (ushort)m_gunPortLocal, L"1.0", L"XXXX-YYYY-ZZZZ", 0)`.
    fn startup_gun_begin(&self, port_local_preference: &dyn Fn() -> u32) -> Result<(), GunError> {
        if !self.options.gun_enabled() {
            *self.phase.borrow_mut() = Phase::Done;
            return Ok(());
        }
        let gun = match self.turbine_gun() {
            Some(gun) => gun,
            None => self.make_gun().ok_or(GUN_FAILED)?,
        };
        let event = CCliConnectEvent::new();
        let listener: Rc<dyn CtConnectEvent> = event.clone();
        if !gun.register_for_connect_events(Some(listener)) {
            return Err(GUN_FAILED);
        }
        if self.gun_port_local.get() == 0 {
            self.gun_port_local.set(port_local_preference());
        }
        let connected = gun.request_connect(
            &self.options.gun_server,
            self.options.gun_port,
            self.gun_port_local.get() as u16,
            "1.0",
            "XXXX-YYYY-ZZZZ",
            0,
        );
        if !connected {
            return Err(GUN_FAILED);
        }
        *self.phase.borrow_mut() = Phase::WaitConnect(event);
        Ok(())
    }

    /// Steps 6-9. `Ok(true)` once `StartupGUN` would return success, `Ok(false)` while a loop
    /// would still spin.
    fn startup_gun_poll(&self, ui_alive: bool) -> Result<bool, GunError> {
        if matches!(*self.phase.borrow(), Phase::Done) {
            return Ok(true);
        }
        let Some(gun) = self.turbine_gun() else {
            return Ok(true);
        };
        let waiting = match &*self.phase.borrow() {
            Phase::WaitConnect(event) => Some(Rc::clone(event)),
            _ => None,
        };
        if let Some(event) = waiting {
            // while (ev->NumEventsHandled() == 0) { if (!KeepUIAlive()) fail; CheckForMessages(); }
            if event.num_events_handled_now() == 0 {
                if !ui_alive {
                    return Err(GUN_FAILED);
                }
                gun.check_for_messages();
                if event.num_events_handled_now() == 0 {
                    return Ok(false);
                }
            }
            // The first event ends the wait, whichever it was; not connected is the failure.
            if !gun.is_connected() {
                return Err(GUN_FAILED);
            }
            if self.options.use_gun_ticket {
                self.authenticate_with_gun(false)?;
                *self.phase.borrow_mut() = Phase::WaitTicket;
            } else {
                return self.finish_startup();
            }
        }
        if matches!(*self.phase.borrow(), Phase::WaitTicket) {
            let Some(handler) = self.ticket_handler() else {
                return Err(GUN_FAILED);
            };
            if !handler.authenticate_request_arrived() {
                if !ui_alive {
                    *self.ticket_handler.borrow_mut() = None;
                    return Err(GUN_FAILED);
                }
                gun.check_for_messages();
                if !handler.authenticate_request_arrived() {
                    return Ok(false);
                }
            }
            let result = handler.authenticate_result();
            if result != ETicketResult::Success {
                // The code also builds a string from `CTResult::GetHResultString` and a
                // sprintf format it never uses.
                *self.ticket_handler.borrow_mut() = None;
                return Err(if result == ETicketResult::Banned {
                    GUN_ACCOUNT_BANNED
                } else {
                    GUN_FAILED
                });
            }
            return self.finish_startup();
        }
        Ok(true)
    }

    /// Step 9: `RegisterForGunEvents()` (its result is not read) and success.
    fn finish_startup(&self) -> Result<bool, GunError> {
        self.register_for_gun_events();
        *self.phase.borrow_mut() = Phase::Done;
        Ok(true)
    }

    /// `gmClient::RegisterForGunEvents` (admin 0x00403C10, Legions 0x00403B80): a `CCliChatEvent`
    /// to `RegisterForChatEvents`, then a `CCliFriendsEvent` to `RegisterForFriendsEvents`; 0 at
    /// the first failure.
    pub fn register_for_gun_events(&self) -> bool {
        let Some(gun) = self.turbine_gun() else {
            return false;
        };
        if !gun.register_for_chat_events(Some(CCliChatEvent::new())) {
            return false;
        }
        gun.register_for_friends_events(Some(CCliFriendsEvent::new()))
    }

    /// `gmClient::AuthenticateWithGUN(reauth)` (admin 0x00402AE0, Legions 0x00402A30).
    pub fn authenticate_with_gun(&self, reauth: bool) -> Result<(), GunError> {
        self.trying_reauthentication.set(reauth);
        let mut error = GUN_FAILED;
        if reauth && (self.string_on_disk.borrow())(GUN_REAUTH_FAILED.string, REAUTH_TABLE) {
            error = GUN_REAUTH_FAILED;
        }
        let handler = {
            let mut slot = self.ticket_handler.borrow_mut();
            Rc::clone(slot.get_or_insert_with(CCliTicketResult::new))
        };
        let Some(gun) = self.turbine_gun() else {
            *self.ticket_handler.borrow_mut() = None;
            return Err(error);
        };
        // `pTicket = new tagBLOB{0, 0}` (leaked by retail on both failure paths).
        let mut ticket = TagBlob::default();
        if !gun.get_authentication_ticket(Some(&mut ticket)) {
            *self.ticket_handler.borrow_mut() = None;
            return Err(error);
        }
        handler.adopt_ticket(ticket);
        let ctx: Rc<dyn CtTicketResult> = handler;
        if !gun.request_present_authentication_ticket(Some(ctx)) {
            *self.ticket_handler.borrow_mut() = None;
            return Err(error);
        }
        Ok(())
    }

    /// `gmClient::InitAuth` (admin 0x00404E10, Legions 0x00404BB0): the credential of the
    /// logon. An empty primer with no `-gunticket` or no ticket handler is
    /// `Turbine::Client::InitAuth` (auth type 1, no data); an empty primer with both is auth type
    /// 0x41000001 with the ticket handler's bytes exactly as `GetTicket` returned them (`None`
    /// when it has no ticket: `InitAuth` returns 0); a primer is auth type 0x40000002 with the
    /// packed primer.
    pub fn init_auth(&self) -> Option<Credential> {
        let primer_empty = self.primer.is_empty();
        let handler = self.ticket_handler();
        if primer_empty && (!self.options.use_gun_ticket || handler.is_none()) {
            return Some(Credential::Account);
        }
        if primer_empty {
            let blob = handler?.get_ticket_blob()?;
            return Some(Credential::GunTicket(blob.bytes().to_vec()));
        }
        Some(Credential::Password(self.primer.clone()))
    }

    /// `gmClient::UseTime` (admin 0x00401E30, Legions 0x00401DA0): `if (m_pTurbineGun)
    /// CheckForMessages()`, the only per-frame poll. Not called while `StartupGUN` spins.
    pub fn use_time(&self) {
        if self.starting_up() {
            return;
        }
        if let Some(gun) = self.turbine_gun() {
            gun.check_for_messages();
        }
    }

    /// `gmClient::RecvNotice_WorldName` (admin 0x00403FF0, Legions 0x00403F40; notice 0x10416),
    /// the GUN half: `TurbineGun::SetWorldName(name)` (the client's own is the caller's).
    pub fn recv_notice_world_name(&self, name: &str) {
        if let Some(gun) = self.turbine_gun() {
            gun.set_world_name(name);
        }
    }

    /// `gmClient::RecvNotice_CharacterSet`'s GUN half (Legions 0x00401D70): a character set with
    /// `m_fUseTurbineChat` sets `S_fUseTurbineChatInsteadOfGUN = 1` and
    /// `S_fStripWorldNameFromRoomChatEvents = false`, then calls the client's slot +0x9C,
    /// `Turbine::Client::StartupTurbineChat` (Legions 0x006A2A30: `Client+0xB0` and the chat
    /// manager), whose chat client is built in here (`GunShared::set_chat_client`).
    pub fn recv_notice_character_set(&self, use_turbine_chat: bool) {
        if use_turbine_chat {
            self.shared.set_use_turbine_chat(true);
            TurbineGun::set_strip_world_name_from_room_chat_events(false);
        }
    }

    /// `gmClient::RecvNotice_GunReconnected` (admin 0x00404080, Legions 0x00403FD0; notice
    /// 0x46B): `AuthenticateWithGUN(true)` unconditionally (even without `-gunticket`); an error
    /// is printed through the UI sink as `NetError::GetLogString`.
    pub fn recv_notice_gun_reconnected(&self) {
        if let Err(error) = self.authenticate_with_gun(true) {
            self.shared.push(GunOutput::UiNetError(error));
        }
    }

    /// `gmClient::RecvNotice_GunAuthenticated(result)` (admin 0x004029D0, Legions 0x00402920;
    /// notice 0x46C): nothing unless `m_fTryingReauthentication`; then it is cleared and a
    /// failure shows `ID_GunReAuthenticationFailed` (when on disk); a success calls
    /// `gmCWInterface::GUNHandleReauthentication` if the player exists, else shows
    /// `ID_GunReAuthenticationSuccess`.
    pub fn recv_notice_gun_authenticated(&self, result: bool) {
        if !self.trying_reauthentication.get() {
            return;
        }
        self.trying_reauthentication.set(false);
        if !result {
            if (self.string_on_disk.borrow())(GUN_REAUTH_FAILED.string, REAUTH_TABLE) {
                self.shared.push(GunOutput::UiString {
                    name: GUN_REAUTH_FAILED.string,
                    table: REAUTH_TABLE,
                });
            }
        } else if self.shared.player_present() {
            self.shared.push(GunOutput::HandleReauthentication);
        } else {
            self.shared.push(GunOutput::UiString {
                name: "ID_GunReAuthenticationSuccess",
                table: REAUTH_TABLE,
            });
        }
    }

    /// `gmClient::~gmClient` (admin 0x00402650, Legions 0x004025A0): unregister the four notices;
    /// with a `TurbineGun` and a ticket handler `RequestFinishWithTicket(handler)` and release
    /// the handler; then delete the `TurbineGun` (its destructor disconnects).
    pub fn shutdown(&self) {
        if self.shut_down.replace(true) {
            return;
        }
        self.shared.set_client(std::rc::Weak::new());
        let gun = self.turbine_gun();
        let handler = self.ticket_handler.borrow_mut().take();
        if let (Some(gun), Some(handler)) = (&gun, handler) {
            let ctx: Rc<dyn CtTicketResult> = handler;
            gun.request_finish_with_ticket(Some(ctx));
        }
        *self.gun.borrow_mut() = None;
        self.shared.set_turbine_gun(None);
        drop(gun);
    }
}

impl Drop for GunClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}
