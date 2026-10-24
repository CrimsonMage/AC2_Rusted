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
use std::time::{Duration, Instant};

use ac2_net::protocol::{
    interrogation_response, Build, CellPosition, CharacterSet, ClientEvent, DatEvent, Qualities,
    ServerEvent, QUEUE_DAT, QUEUE_EVENTS,
};
use ac2_net::{
    Credential, Logon, NetClient, NetClientConfig, NetError, NetEvent, ServerPublicKey,
    SessionCrypto,
};

use crate::objects::ObjectTable;
use crate::reporter::{BodyState, PositionReporter};
use crate::world::{World, WorldEvent, WorldState};

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
    /// The server moved the player (portal, recall or correction).
    PlayerMoved {
        position: CellPosition,
        heading_degrees: f32,
    },
    ObjectAdded(u64),
    ObjectMoved(u64),
    ObjectRemoved(u64),
    /// `ECM_Login::RecvEvt_PlayerDesc`: the player's qualities (`CWRepository::SetQualities`,
    /// then `SendNotice_LoginFinished`).
    PlayerQualities(Box<Qualities>),
    /// The character left the world (`CharExitGame`).
    LeftWorld,
    /// The server refused something; the session may continue.
    Rejected(String),
    /// The session ended.
    Disconnected(String),
}

/// The client side of one online session.
pub struct OnlineSession {
    client: NetClient,
    account: String,
    choice: CharacterChoice,
    world: World,
    reporter: Option<PositionReporter>,
    entered: bool,
    scene_ready_sent: bool,
    closed: bool,
    /// Events [`OnlineSession::wait_for_world`] saw, for the next [`OnlineSession::poll`].
    pending: Vec<SessionEvent>,
}

impl OnlineSession {
    /// Starts the logon; progress arrives from [`OnlineSession::poll`].
    pub fn connect(config: OnlineConfig) -> Result<Self, NetError> {
        let build = Build::from_version(&config.version);
        let crypto = config
            .server_key
            .as_ref()
            .map(SessionCrypto::new)
            .transpose()?;
        let client = NetClient::connect(NetClientConfig {
            server: config.server,
            logon: Logon {
                version: config.version,
                account: config.account.clone(),
                credential: config.credential,
                connection_sequence: ac2_net::real_time() as u32,
                crypto,
            },
        })?;
        Ok(Self {
            client,
            account: config.account,
            choice: config.character,
            world: World::new(build),
            reporter: None,
            entered: false,
            scene_ready_sent: false,
            closed: false,
            pending: Vec::new(),
        })
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
    pub fn poll(&mut self) -> Vec<SessionEvent> {
        let mut out = std::mem::take(&mut self.pending);
        for event in self.client.poll() {
            self.handle(event, &mut out);
        }
        out
    }

    /// Runs frames until the player is in the world, or fails after `timeout`. The other
    /// events it sees come back from the next [`OnlineSession::poll`].
    pub fn wait_for_world(&mut self, timeout: Duration) -> Result<WorldEntry, String> {
        let deadline = Instant::now() + timeout;
        let mut seen = Vec::new();
        while Instant::now() < deadline {
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
            std::thread::sleep(Duration::from_millis(10));
        }
        Err("timed out waiting for world entry".into())
    }

    fn handle(&mut self, event: NetEvent, out: &mut Vec<SessionEvent>) {
        match event {
            NetEvent::Connected { .. } => out.push(SessionEvent::Connected),
            NetEvent::LoggedOn | NetEvent::TimeSync(_) | NetEvent::ServerSwitch { .. } => {}
            NetEvent::Disconnected(error) => {
                self.closed = true;
                out.push(SessionEvent::Disconnected(error.to_string()));
            }
            NetEvent::Blob(blob) if blob.queue == QUEUE_EVENTS => {
                let cur_time = self.cur_time();
                for event in self.world.receive(blob, cur_time) {
                    self.handle_world(event, out);
                }
            }
            NetEvent::Blob(blob) if blob.queue == QUEUE_DAT => {
                // `CLCache::UseTime`: data-download events. This client patches no DATs, so an
                // interrogation is answered with no iteration lists (retail lists each DAT).
                if let Ok(ServerEvent::Dat(DatEvent::Interrogation(_))) =
                    ServerEvent::decode(&blob.data, Build::default())
                {
                    let response = interrogation_response(0, &[]);
                    self.client.net_mut().send_dat_event(response);
                }
            }
            // Queue 2 is discarded and queue 4 carries chat (`ProcessLogonEventQueue`).
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
                if Some(update.sender.id) == self.world.player() {
                    out.push(SessionEvent::PlayerMoved {
                        position: update.position,
                        heading_degrees: update.heading_degrees,
                    });
                } else {
                    out.push(SessionEvent::ObjectMoved(update.sender.id));
                }
            }
            ServerEvent::DestroyObject(sender) => out.push(SessionEvent::ObjectRemoved(sender.id)),
            ServerEvent::PlayerDescription(qualities) => {
                out.push(SessionEvent::PlayerQualities(qualities.clone()))
            }
            ServerEvent::CharExitGame => {
                self.world.leave_world();
                self.reporter = None;
                self.entered = false;
                self.scene_ready_sent = false;
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

    /// `gmGamePlayUI::RecvNotice_NextFrameDrawn`: the scene around the player has drawn; once
    /// per world entry.
    pub fn scene_ready(&mut self) {
        if self.entered && !self.scene_ready_sent {
            self.scene_ready_sent = true;
            self.send(ClientEvent::SceneRenderingComplete);
        }
    }

    /// `PlayerMovementModule::UseTime`: report the player's body when retail would.
    pub fn report_body(&mut self, body: &BodyState) {
        let cur_time = self.cur_time();
        let Some(report) = self
            .reporter
            .as_mut()
            .and_then(|r| r.report(cur_time, body))
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
            let deadline = Instant::now() + wait;
            'wait: while Instant::now() < deadline && !self.closed {
                for event in self.poll() {
                    left |= event == SessionEvent::LeftWorld;
                    if matches!(
                        event,
                        SessionEvent::LeftWorld | SessionEvent::Disconnected(_)
                    ) {
                        break 'wait;
                    }
                }
                std::thread::sleep(Duration::from_millis(10));
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
