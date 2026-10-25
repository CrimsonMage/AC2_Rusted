//! The online session: logon, character choice, world entry, the objects the server reports,
//! movement reports and logoff, in the order the retail client performs them.
//!
//! World entry (`gmCharacterSelectionUI` -> `Proto_UI::SendEnterWorld`): the event queue starts
//! entering the world, ordered stamps restart, `EnterWorld` goes on queue 4; the server's
//! `CreatePlayer` finishes entering (`GlobalEventHandler::RecvEvt_CreatePlayer`); the player's
//! `CreateObject` places it. Once the scene has drawn, `SceneRenderingComplete` goes on queue 3
//! (`gmGamePlayUI::RecvNotice_NextFrameDrawn`). Leaving: `LogOffCharacter` on queue 4
//! (`gmGamePlayUI::EndSession`), the server's `CharExitGame`, then the transport logoff.

use std::net::SocketAddr;
use std::time::Duration;

/// Pause between frames while waiting.
const FRAME: Duration = Duration::from_millis(10);

use ac2_net::protocol::{
    interrogation_response, Build, CellPosition, CharacterSet, ClientEvent, DatEvent, ObjectAction,
    Qualities, ServerEvent, ServerPosition, StringInfo, QUEUE_DAT, QUEUE_EVENTS,
};
use ac2_net::{
    Credential, Logon, NetClient, NetClientConfig, NetError, NetEvent, ServerPublicKey,
    SessionCrypto,
};

use crate::objects::ObjectTable;
use crate::objects::{STAMP_FORCE_POSITION, STAMP_MOVE_TO, STAMP_TELEPORT};
use crate::player_script::{GameplayNotice, PlayerScript};
use crate::reporter::{BodyState, PositionReporter, ReportStamps};
use crate::world::{World, WorldEvent, WorldState};

mod gameplay;

/// Which listed character to enter the world with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterChoice {
    First,
    Name(String),
    Id(u64),
}

#[derive(Debug, Clone)]
pub struct OnlineConfig {
    pub server: SocketAddr,
    pub account: String,
    pub credential: Credential,
    /// Data version sent in the logon: `ac2_net::LEGIONS_VERSION` (`"7160"`) for the DVD
    /// client; servers that only accept the admin build take `"7148"`.
    pub version: String,
    pub character: CharacterChoice,
    /// The RSA key the session key is exported under: the one the client executable carries
    /// (`ServerPublicKey::find_in_client`). `None` logs on without a session key.
    pub server_key: Option<ServerPublicKey>,
    /// `DBCache::GetLocalLanguage`: the language id the local DAT was opened for.
    pub language: u32,
    /// The DATs' file ids and iterations (`crate::dat_iterations`) for the interrogation
    /// response; empty when no DATs are open.
    pub dat_iterations: Vec<(u64, Vec<i32>)>,
}

/// Where the player entered the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldEntry {
    pub player: u64,
    pub instance: u16,
    pub position: CellPosition,
    /// Orientation quaternion (w, x, y, z).
    pub orientation: [f32; 4],
    pub visual: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    Connected,
    Characters(CharacterSet),
    /// The chosen character is entering; its id.
    Entering(u64),
    EnteredWorld(WorldEntry),
    /// The server moved the player (`CPhysicsObj::RecvEvt_Position`'s player branch: a newer
    /// force-position or teleport stamp, or an impulse). Place the body there
    /// (`SyncPositionSimple(pos, 1)`), keeping its own heading unless `teleported`
    /// (`SmartBox::HandleTeleport` follows a teleport), then apply `impulse`.
    PlayerMoved {
        position: CellPosition,
        heading_degrees: f32,
        teleported: bool,
        impulse: Option<[f32; 3]>,
    },
    /// `CObjectMaint::RecvEvt_CreateObject` for an object other than the player (its data is in
    /// [`OnlineSession::objects`]).
    ObjectAdded(u64),
    /// `CPhysicsObj::RecvEvt_Position` accepted a newer position for an object other than the
    /// player: the update, and whether its teleport stamp was newer.
    ObjectMoved {
        object: u64,
        update: ServerPosition,
        teleported: bool,
    },
    /// A per-object physics event reached its object (`CPhysicsObj::RecvEvt_*`), after the
    /// object table's bookkeeping.
    ObjectAction {
        object: u64,
        action: ObjectAction,
    },
    /// `ECM_Physics::RecvEvt_Contain`: the object went into the player's inventory (out of the
    /// world).
    ObjectContained(u64),
    /// `CObjectMaint::DestroyObjectInternal`: the object is gone (destroyed by the server, or
    /// out of view for 25 s).
    ObjectRemoved(u64),
    /// `ECM_Login::RecvEvt_PlayerDesc`: the player's qualities (`CWRepository::SetQualities`,
    /// then `SendNotice_LoginFinished`).
    PlayerQualities(Box<Qualities>),
    /// `ECM_Login::SendNotice_LoginFinished` (notice `0x10401`), sent by
    /// `CWInterface::RecvEvt_PlayerDesc` once the player's qualities are stored.
    LoginFinished,
    /// `CWRepository::RecvEvt_Update*_Private`'s change notice: the object, the table code
    /// (`QualityValue::table`) and the quality type id.
    QualityChanged {
        object: u64,
        table: u32,
        quality: u32,
    },
    /// The character left the world (`CharExitGame`).
    LeftWorld,
    /// The server refused something; the session may continue.
    Rejected(String),
    /// `ECM_Login::RecvEvt_CharGenVerification`: the server's answer to a character creation
    /// (its response, the character, the weenie's result).
    CharGenVerification {
        response: u32,
        character: u64,
        name: String,
        result: u32,
    },
    /// Queue 4 chat data the chat client could not read (`IChatClientImpl::OnNetworkData`
    /// refused it).
    Chat(Vec<u8>),
    /// A script event (`CWDistributor::RecvEvt_InterpCEvent_Private`): the object (the player
    /// for broadcasts), the function id and its packed arguments.
    Interp {
        object: u64,
        function: u32,
        arguments: Vec<u8>,
    },
    /// `GlobalEventHandler::RecvEvt_DisplayStringInfo`: a display type and its text.
    SystemMessage {
        kind: u32,
        text: StringInfo,
    },
    /// `GlobalEventHandler::RecvEvt_WorldName`.
    WorldName(StringInfo),
    /// `GlobalEventHandler::RecvEvt_AccountBooted`: the reason.
    AccountBooted(StringInfo),
    /// The server gave a reason for closing the connection (`Net_LoginConnectionError` /
    /// `Net_WorldConnectionError`); [`SessionEvent::ServerDied`] follows once the disconnect
    /// handshake ran.
    ConnectionError(String),
    /// `ECM_Login::SendNotice_ServerDied` (`ClientNet::RemoveConnection`): the logon or world
    /// connection went away, closed by the server or silent for 140 s.
    /// [`SessionEvent::Disconnected`] follows.
    ServerDied,
    /// The session ended.
    Disconnected(String),
    /// A notice the player's scripts raised (`CM_*::SendNotice_*`) for the UI.
    Notice(GameplayNotice),
}

/// The client side of one online session.
pub struct OnlineSession {
    client: NetClient,
    account: String,
    choice: CharacterChoice,
    world: World,
    reporter: Option<PositionReporter>,
    entered: bool,
    closed: bool,
    /// Events [`OnlineSession::wait_for_world`] saw, for the next [`OnlineSession::poll`].
    pending: Vec<SessionEvent>,
    language: u32,
    dat_iterations: Vec<(u64, Vec<i32>)>,
    /// The world's verdict on the position update about to be dispatched.
    last_position: Option<(u64, crate::objects::PositionApplied)>,
    /// Net events received by [`OnlineSession::net_use_time`], for
    /// [`OnlineSession::dispatch_events`].
    net_events: Vec<NetEvent>,
    /// The player's weenie scripts (`CPlayer`), natively until the script VM runs them.
    script: PlayerScript,
    /// `IChatClientImpl`'s last context id (`AddPendingMethod` numbers requests from 1).
    chat_context: u32,
}

impl OnlineSession {
    /// Starts the logon; progress arrives from [`OnlineSession::poll`].
    pub fn connect(config: OnlineConfig) -> Result<Self, NetError> {
        let transport = ac2_net::UdpTransport::bind(config.server)?;
        Self::with_transport(config, Box::new(transport))
    }

    /// [`OnlineSession::connect`] over `transport` (a scripted server in tests).
    pub fn with_transport(
        config: OnlineConfig,
        mut transport: Box<dyn ac2_net::Transport>,
    ) -> Result<Self, NetError> {
        let connection_sequence = transport.clock().1 as u32;
        let build = Build::from_version(&config.version);
        let crypto = config
            .server_key
            .as_ref()
            .map(SessionCrypto::new)
            .transpose()?;
        let client = NetClient::with_transport(
            NetClientConfig {
                server: config.server,
                logon: Logon {
                    version: config.version,
                    account: config.account.clone(),
                    credential: config.credential,
                    connection_sequence,
                    crypto,
                },
            },
            transport,
        );
        Ok(Self {
            client,
            account: config.account,
            choice: config.character,
            world: World::new(build),
            reporter: None,
            entered: false,
            closed: false,
            pending: Vec::new(),
            language: config.language,
            dat_iterations: config.dat_iterations,
            last_position: None,
            net_events: Vec::new(),
            script: PlayerScript::default(),
            chat_context: 0,
        })
    }

    /// [`World::set_property_types`] for this session's world.
    pub fn set_property_types(
        &mut self,
        types: Box<dyn ac2_net::protocol::PropertyTypes + Send + Sync>,
    ) {
        self.world.set_property_types(types);
    }

    pub fn objects(&self) -> &ObjectTable {
        &self.world.objects
    }

    /// The player's object id once entering the world.
    pub fn player(&self) -> Option<u64> {
        self.world.player()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// The server clock (`Timer::cur_time`).
    pub fn cur_time(&self) -> f64 {
        self.client.net().timer().cur_time()
    }

    fn send(&mut self, event: ClientEvent) {
        let (queue, bytes) = event.encode();
        self.client.send(queue, bytes);
    }

    /// One frame: everything the server sent since the last call.
    /// [`OnlineSession::net_use_time`] then [`OnlineSession::dispatch_events`].
    pub fn poll(&mut self) -> Vec<SessionEvent> {
        self.net_use_time();
        self.dispatch_events()
    }

    /// `Timer::update_time` and `ClientNet::UseTime`: datagrams in and out; what arrived waits
    /// for [`OnlineSession::dispatch_events`] (`Client::UseTime` runs the movement module and
    /// the SmartBox in between).
    pub fn net_use_time(&mut self) {
        let events = self.client.poll();
        self.net_events.extend(events);
    }

    /// `CEventQueue::UseTime` and the logon and DDD queues: dispatch what the network received.
    pub fn dispatch_events(&mut self) -> Vec<SessionEvent> {
        let mut out = std::mem::take(&mut self.pending);
        for event in std::mem::take(&mut self.net_events) {
            self.handle(event, &mut out);
        }
        out
    }

    /// `CObjectMaint::UseTime` (from `WorldManager::Update`): held event queues that waited too
    /// long for their object ask for it to be created and are dropped.
    pub fn object_maint_use_time(&mut self) -> Vec<SessionEvent> {
        let cur_time = self.cur_time();
        let mut out = Vec::new();
        for event in self.world.use_time(cur_time) {
            self.handle_world(event, &mut out);
        }
        out
    }

    /// `CBookKeep::AddObjectToBeDestroyed`: the client lost sight of the object (it left the
    /// world or its cell is not resident); it is destroyed in 25 s unless it comes back.
    pub fn add_object_to_be_destroyed(&mut self, id: u64) {
        self.world.add_object_to_be_destroyed(id);
    }

    /// `CBookKeep::RemoveObjectToBeDestroyed` (`CBookKeep::HandleEnterWorld`).
    pub fn remove_object_to_be_destroyed(&mut self, id: u64) {
        self.world.remove_object_to_be_destroyed(id);
    }

    /// Whether the object waits to be destroyed for being out of view.
    pub fn is_to_be_destroyed(&self, id: u64) -> bool {
        self.world.is_to_be_destroyed(id)
    }

    /// `Timer::local_time` (the physics clock, `CPhysics::GetTickCount`).
    pub fn local_time(&self) -> f64 {
        self.client.net().timer().local_time()
    }

    /// The transport's seconds since start.
    pub fn elapsed(&mut self) -> f64 {
        self.client.elapsed()
    }

    /// Waits on the transport's clock.
    pub fn sleep(&mut self, duration: Duration) {
        self.client.sleep(duration);
    }

    /// Runs frames until the player is in the world, or fails after `timeout`. The other
    /// events it sees come back from the next [`OnlineSession::poll`].
    pub fn wait_for_world(&mut self, timeout: Duration) -> Result<WorldEntry, String> {
        let deadline = self.client.elapsed() + timeout.as_secs_f64();
        let mut seen = Vec::new();
        while self.client.elapsed() < deadline {
            let mut entered = None;
            for event in self.poll() {
                match event {
                    SessionEvent::EnteredWorld(entry) if entered.is_none() => entered = Some(entry),
                    SessionEvent::Disconnected(reason) => return Err(reason),
                    SessionEvent::Rejected(reason) if self.closed => return Err(reason),
                    other => seen.push(other),
                }
            }
            if let Some(entry) = entered {
                self.pending = seen;
                return Ok(entry);
            }
            self.client.sleep(FRAME);
        }
        Err("timed out waiting for world entry".into())
    }

    fn handle(&mut self, event: NetEvent, out: &mut Vec<SessionEvent>) {
        match event {
            NetEvent::Connected { .. } => out.push(SessionEvent::Connected),
            NetEvent::LoggedOn | NetEvent::TimeSync(_) | NetEvent::ServerSwitch { .. } => {}
            NetEvent::ConnectionError { error, .. } => {
                out.push(SessionEvent::ConnectionError(error.to_string()))
            }
            NetEvent::Disconnected(error) => {
                self.closed = true;
                // The session is over: its world goes as at `CharExitGame`.
                self.world.leave_world();
                self.reporter = None;
                self.entered = false;
                if error == NetError::ServerDied {
                    out.push(SessionEvent::ServerDied);
                }
                out.push(SessionEvent::Disconnected(error.to_string()));
            }
            NetEvent::Blob(blob) if blob.queue == QUEUE_EVENTS => {
                let cur_time = self.cur_time();
                for event in self.world.receive(blob, cur_time) {
                    self.handle_world(event, out);
                }
            }
            NetEvent::Blob(blob) if blob.queue == QUEUE_DAT => {
                // `CLCache::UseTime` -> `CLCache::OnServerInterrogation`: the language and each
                // open DAT's iterations. This client patches no DATs beyond that answer.
                if let Ok(ServerEvent::Dat(DatEvent::Interrogation(_))) =
                    ServerEvent::decode(&blob.data, Build::default())
                {
                    let response = interrogation_response(self.language, &self.dat_iterations);
                    self.client.net_mut().send_dat_event(response);
                }
            }
            // `Turbine::Client::ProcessLogonEventQueue`: queue 4 hands `0x100D3` to the chat
            // client and drops everything else; queue 2 is discarded.
            NetEvent::Blob(blob) if blob.queue == 4 => {
                if let Ok(ServerEvent::Chat(data)) =
                    ServerEvent::decode(&blob.data, Build::default())
                {
                    let events = self.receive_chat(data);
                    out.extend(events);
                }
            }
            NetEvent::Blob(_) => {}
        }
    }

    fn handle_world(&mut self, event: WorldEvent, out: &mut Vec<SessionEvent>) {
        let event = match event {
            WorldEvent::Dispatched(event) => event,
            WorldEvent::Undecoded { opcode, error } => {
                out.push(SessionEvent::Rejected(format!(
                    "undecoded event {opcode:#010x}: {error}"
                )));
                return;
            }
            WorldEvent::ForceCreate(object) => {
                self.send(ClientEvent::ForceCreate { object });
                return;
            }
            WorldEvent::PositionApplied { id, applied } => {
                self.last_position = Some((id, applied));
                return;
            }
            WorldEvent::Destroyed(id) => {
                self.script.forget_object(id);
                out.push(SessionEvent::ObjectRemoved(id));
                return;
            }
        };
        match event {
            ServerEvent::CharacterSet(set) => self.choose_character(set, out),
            ServerEvent::CharacterError(reason) => {
                if self.world.state() == WorldState::EnteringWorld {
                    self.world.leave_world();
                }
                out.push(SessionEvent::Rejected(format!("character error {reason}")));
            }
            ServerEvent::CreateObject(object) => {
                if Some(object.id) == self.world.player() {
                    if !self.entered {
                        if let Some(position) = object.physics.position {
                            self.entered = true;
                            self.reporter = Some(PositionReporter::new(0));
                            out.push(SessionEvent::EnteredWorld(WorldEntry {
                                player: object.id,
                                instance: object.physics.instance_stamp,
                                position: CellPosition {
                                    cell: position.cell,
                                    origin: position.origin,
                                },
                                orientation: position.rotation,
                                visual: object.visual.parent,
                            }));
                        }
                    }
                } else {
                    out.push(SessionEvent::ObjectAdded(object.id));
                }
            }
            ServerEvent::UpdatePosition(update) | ServerEvent::PositionCell(update) => {
                let applied = self
                    .last_position
                    .take()
                    .filter(|(id, _)| *id == update.sender.id)
                    .map(|(_, applied)| applied);
                let Some(applied) = applied else {
                    return;
                };
                if Some(update.sender.id) == self.world.player() {
                    // `CPhysicsObj::RecvEvt_Position`, the player: only a newer force-position
                    // or teleport stamp, or an impulse, moves the body the client drives.
                    if applied.forced || applied.teleported || update.impulse {
                        out.push(SessionEvent::PlayerMoved {
                            position: update.position,
                            heading_degrees: update.heading_degrees,
                            teleported: applied.teleported,
                            impulse: update.impulse.then_some(update.velocity).flatten(),
                        });
                    }
                } else {
                    out.push(SessionEvent::ObjectMoved {
                        object: update.sender.id,
                        update,
                        teleported: applied.teleported,
                    });
                }
            }
            ServerEvent::Contain(child) => {
                if self
                    .world
                    .objects
                    .get(child.id)
                    .is_some_and(|o| o.contained)
                {
                    out.push(SessionEvent::ObjectContained(child.id));
                }
            }
            ServerEvent::Object(event) => out.push(SessionEvent::ObjectAction {
                object: event.sender.id,
                action: event.action,
            }),
            ServerEvent::PlayerDescription(qualities) => {
                out.push(SessionEvent::PlayerQualities(qualities.clone()));
                out.push(SessionEvent::LoginFinished);
            }
            ServerEvent::CharGenVerification {
                response,
                identity,
                weenie_result,
            } => out.push(SessionEvent::CharGenVerification {
                response,
                character: identity.id,
                name: identity.name,
                result: weenie_result,
            }),
            ServerEvent::Interp {
                object,
                function,
                arguments,
            } => {
                let target = object.map(|o| o.id).or(self.world.player());
                if target.is_some() && target == self.world.player() {
                    if let Some(events) = self.receive_script(function, &arguments) {
                        out.extend(events);
                        return;
                    }
                }
                if let Some(object) = target {
                    out.push(SessionEvent::Interp {
                        object,
                        function,
                        arguments,
                    });
                }
            }
            ServerEvent::DisplayStringInfo { kind, text } => {
                out.push(SessionEvent::SystemMessage { kind, text: *text })
            }
            ServerEvent::WorldName(text) => out.push(SessionEvent::WorldName(*text)),
            ServerEvent::AccountBooted(text) => out.push(SessionEvent::AccountBooted(*text)),
            ServerEvent::Quality(update) => {
                if let Some(object) = update.object.map(|o| o.id).or(self.world.player()) {
                    out.push(SessionEvent::QualityChanged {
                        object,
                        table: update.value.table(),
                        quality: update.quality,
                    });
                }
            }
            ServerEvent::CharExitGame => {
                self.world.leave_world();
                self.reporter = None;
                self.entered = false;
                out.push(SessionEvent::LeftWorld);
            }
            _ => {}
        }
    }

    /// `gmCharacterSelectionUI` picking a character, then `Proto_UI::SendEnterWorld`.
    fn choose_character(&mut self, set: CharacterSet, out: &mut Vec<SessionEvent>) {
        let chosen = set
            .characters
            .iter()
            .find(|c| match &self.choice {
                CharacterChoice::First => true,
                CharacterChoice::Name(name) => c.name.eq_ignore_ascii_case(name),
                CharacterChoice::Id(id) => c.id == *id,
            })
            .map(|c| c.id);
        out.push(SessionEvent::Characters(set));
        if self.world.state() != WorldState::NotInWorld {
            return;
        }
        match chosen {
            Some(character) => {
                self.world.start_entering_world();
                self.client.net_mut().enter_world();
                let account = self.account.clone();
                self.send(ClientEvent::EnterWorld { character, account });
                out.push(SessionEvent::Entering(character));
            }
            None => {
                self.closed = true;
                out.push(SessionEvent::Rejected(format!(
                    "no character matching {:?} on this account",
                    self.choice
                )));
            }
        }
    }

    /// `gmGamePlayUI::RecvNotice_NextFrameDrawn`: the first frame drawn after the player was
    /// ready to enter the world (`RecvNotice_PlayerReadyToEnterWorld` asked for the notice);
    /// sent for every entry, a teleport's included.
    pub fn scene_ready(&mut self) {
        if self.entered {
            self.send(ClientEvent::SceneRenderingComplete);
        }
    }

    /// `PlayerMovementModule::UseTime`: report the player's body when retail would.
    /// `ChatInterfaceProvider::SendData`: chat data out on queue 4.
    pub fn send_chat(&mut self, data: &[u8]) {
        self.client.net_mut().send_chat(data);
    }

    /// A script event to the server (`ECM_Interp::SendEvt_InterpSEvent[Encrypt]`).
    pub fn send_interp(&mut self, function: u32, arguments: Vec<u8>, encrypted: bool) {
        self.send(ClientEvent::Interp {
            function,
            arguments,
            encrypted,
        });
    }

    /// The stamps come from the player's object as the server last set them
    /// (`CPhysicsObj::get_instance_stamp`, `get_update_time`).
    pub fn report_body(&mut self, body: &BodyState) {
        let cur_time = self.cur_time();
        let stamps = self
            .world
            .player()
            .and_then(|id| self.world.objects.get(id))
            .map(|o| ReportStamps {
                instance: o.instance,
                teleport: o.update_times[STAMP_TELEPORT],
                force_position: o.update_times[STAMP_FORCE_POSITION],
                moveto: o.update_times[STAMP_MOVE_TO],
            })
            .unwrap_or_default();
        let Some(report) = self
            .reporter
            .as_mut()
            .and_then(|r| r.report(cur_time, body, stamps))
        else {
            return;
        };
        self.send(ClientEvent::Position(report));
    }

    /// `PlayerMovementModule::SendLookAtEvent`.
    pub fn report_head(&mut self, looking_at_object: bool, head: (f32, f32)) {
        let cur_time = self.cur_time();
        if let Some((x, y)) = self
            .reporter
            .as_mut()
            .and_then(|r| r.look(cur_time, looking_at_object, head))
        {
            self.send(ClientEvent::LookAtDir { x, y });
        }
    }

    /// `gmGamePlayUI::EndSession` (`LogOffCharacter`), the server's `CharExitGame` for up to
    /// `wait`, then `ClientNet::LogOffServer`. Returns whether the character left the world.
    pub fn log_off(&mut self, wait: Duration) -> bool {
        let mut left = false;
        if let (Some(character), true) = (self.world.player(), self.entered) {
            self.send(ClientEvent::LogOffCharacter { character });
            let deadline = self.client.elapsed() + wait.as_secs_f64();
            'wait: while self.client.elapsed() < deadline && !self.closed {
                for event in self.poll() {
                    left |= event == SessionEvent::LeftWorld;
                    if matches!(
                        event,
                        SessionEvent::LeftWorld | SessionEvent::Disconnected(_)
                    ) {
                        break 'wait;
                    }
                }
                self.client.sleep(FRAME);
            }
        }
        self.closed = true;
        self.client.disconnect();
        left
    }
}

impl Drop for OnlineSession {
    fn drop(&mut self) {
        if !self.closed {
            self.log_off(Duration::from_secs(1));
        }
    }
}
