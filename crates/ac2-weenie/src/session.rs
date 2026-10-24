use std::net::SocketAddr;
use std::time::{Duration, Instant};

use ac2_net::protocol::{
    CellPosition, CharacterSet, ClientEvent, CreateObject, ServerEvent, QUEUE_EVENTS,
};
use ac2_net::{Credential, Logon, NetClient, NetClientConfig, NetError, NetEvent};

use crate::objects::ObjectTable;
use crate::reporter::PositionReporter;

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
    /// Data version sent in the logon; servers accept the builds they support (e.g. `7148`).
    pub version: String,
    pub character: CharacterChoice,
    pub connect_timeout: Duration,
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
    /// The server refused something; the session may continue.
    Rejected(String),
    /// The session ended.
    Disconnected(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Connecting,
    Selecting,
    Entering { character: u64 },
    Spawning { player: u64 },
    InWorld { player: u64 },
    Closed,
}

/// The client side of one online session.
pub struct OnlineSession {
    client: NetClient,
    account: String,
    choice: CharacterChoice,
    phase: Phase,
    objects: ObjectTable,
    reporter: Option<PositionReporter>,
    scene_ready_sent: bool,
}

impl OnlineSession {
    /// Starts the logon; progress arrives from [`OnlineSession::poll`].
    pub fn connect(config: OnlineConfig) -> Result<Self, NetError> {
        let client = NetClient::connect(NetClientConfig {
            server: config.server,
            logon: Logon {
                version: config.version,
                account: config.account.clone(),
                credential: config.credential,
                connection_sequence: ac2_net::real_time() as u32,
            },
        })?;
        Ok(Self {
            client,
            account: config.account,
            choice: config.character,
            phase: Phase::Connecting,
            objects: ObjectTable::default(),
            reporter: None,
            scene_ready_sent: false,
        })
    }

    pub fn objects(&self) -> &ObjectTable {
        &self.objects
    }

    /// The player's object id once in (or entering) the world.
    pub fn player(&self) -> Option<u64> {
        match self.phase {
            Phase::Spawning { player } | Phase::InWorld { player } => Some(player),
            _ => None,
        }
    }

    pub fn is_closed(&self) -> bool {
        self.phase == Phase::Closed
    }

    fn send(&mut self, event: ClientEvent) {
        let (queue, bytes) = event.encode();
        self.client.send(queue, bytes);
    }

    /// Processes everything the server sent since the last call.
    pub fn poll(&mut self) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        for event in self.client.poll() {
            self.handle(event, &mut out);
        }
        out
    }

    /// Polls until the player is in the world, or fails after `timeout`.
    pub fn wait_for_world(&mut self, timeout: Duration) -> Result<WorldEntry, String> {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err("timed out waiting for world entry".into());
            }
            let Some(event) = self
                .client
                .next_event(remaining.min(Duration::from_millis(100)))
            else {
                continue;
            };
            let mut events = Vec::new();
            self.handle(event, &mut events);
            for event in events {
                match event {
                    SessionEvent::EnteredWorld(entry) => return Ok(entry),
                    SessionEvent::Disconnected(reason) => return Err(reason),
                    SessionEvent::Rejected(reason) if self.phase == Phase::Closed => {
                        return Err(reason)
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle(&mut self, event: NetEvent, out: &mut Vec<SessionEvent>) {
        match event {
            NetEvent::Connected { .. } => {
                self.phase = Phase::Selecting;
                out.push(SessionEvent::Connected);
            }
            NetEvent::TimeSync(_) | NetEvent::LoggedOn | NetEvent::ServerSwitch { .. } => {}
            NetEvent::Disconnected(error) => {
                self.phase = Phase::Closed;
                out.push(SessionEvent::Disconnected(error.to_string()));
            }
            NetEvent::Blob(blob) => match ServerEvent::decode(&blob.data) {
                Ok(event) => self.handle_event(event, blob.queue, out),
                Err(error) => {
                    let opcode = blob
                        .data
                        .get(..4)
                        .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                    out.push(SessionEvent::Rejected(format!(
                        "undecoded event {opcode:#010x} on queue {}: {error}",
                        blob.queue
                    )));
                }
            },
        }
    }

    fn handle_event(&mut self, event: ServerEvent, queue: u16, out: &mut Vec<SessionEvent>) {
        match event {
            ServerEvent::CharacterSet(set) if queue == QUEUE_EVENTS => {
                let chosen = set
                    .characters
                    .iter()
                    .find(|character| match &self.choice {
                        CharacterChoice::First => true,
                        CharacterChoice::Name(name) => character.name.eq_ignore_ascii_case(name),
                        CharacterChoice::Id(id) => character.id == *id,
                    })
                    .map(|character| character.id);
                out.push(SessionEvent::Characters(set));
                if self.phase != Phase::Selecting {
                    return;
                }
                match chosen {
                    Some(character) => {
                        self.send(ClientEvent::EnterWorld {
                            character,
                            account: self.account.clone(),
                        });
                        self.phase = Phase::Entering { character };
                        out.push(SessionEvent::Entering(character));
                    }
                    None => {
                        self.phase = Phase::Closed;
                        out.push(SessionEvent::Rejected(format!(
                            "no character matching {:?} on this account",
                            self.choice
                        )));
                    }
                }
            }
            ServerEvent::CharacterError(reason) => {
                if let Phase::Entering { .. } = self.phase {
                    self.phase = Phase::Closed;
                }
                out.push(SessionEvent::Rejected(format!("character error {reason}")));
            }
            ServerEvent::CreatePlayer { id, .. } => {
                self.objects.clear();
                self.phase = Phase::Spawning { player: id };
            }
            ServerEvent::CreateObject(object) => self.create_object(&object, out),
            ServerEvent::UpdatePosition(update) => {
                if Some(update.id) == self.player() {
                    if let Some(reporter) = self.reporter.as_mut() {
                        // Adopt the server's forced-position and teleport stamps.
                        reporter.stamps[2] = update.stamps[1];
                        reporter.stamps[1] = update.stamps[2];
                    }
                    out.push(SessionEvent::PlayerMoved {
                        position: update.position,
                        heading_degrees: update.heading_degrees,
                    });
                } else if self.objects.update(&update) {
                    out.push(SessionEvent::ObjectMoved(update.id));
                }
            }
            ServerEvent::DestroyObject { id, instance } => {
                if self.objects.destroy(id, instance) {
                    out.push(SessionEvent::ObjectRemoved(id));
                }
            }
            ServerEvent::CharExitGame => {
                self.objects.clear();
                self.reporter = None;
                self.scene_ready_sent = false;
                self.phase = Phase::Selecting;
            }
            _ => {}
        }
    }

    fn create_object(&mut self, object: &CreateObject, out: &mut Vec<SessionEvent>) {
        if let Phase::Spawning { player } = self.phase {
            if object.id == player {
                if let Some((position, orientation)) = object.physics.position {
                    self.reporter = Some(PositionReporter::new(
                        object.physics.instance_stamp,
                        Instant::now(),
                    ));
                    self.phase = Phase::InWorld { player };
                    out.push(SessionEvent::EnteredWorld(WorldEntry {
                        player,
                        instance: object.physics.instance_stamp,
                        position,
                        orientation,
                        visual: object.visual.parent,
                    }));
                }
                return;
            }
        }
        if Some(object.id) == self.player() {
            return;
        }
        self.objects.create(object);
        out.push(SessionEvent::ObjectAdded(object.id));
    }

    /// Tells the server the scene around the player is loaded, once per world entry. The
    /// server publishes the player to others only after this.
    pub fn scene_ready(&mut self) {
        if matches!(self.phase, Phase::InWorld { .. }) && !self.scene_ready_sent {
            self.scene_ready_sent = true;
            self.send(ClientEvent::SceneRenderingComplete);
        }
    }

    /// Reports the player's body state; sent only when it changed enough (see
    /// [`PositionReporter`]).
    pub fn report_position(
        &mut self,
        position: CellPosition,
        heading_degrees: f32,
        motion: [f32; 3],
        jump_velocity: Option<[f32; 3]>,
    ) {
        if !self.scene_ready_sent {
            return;
        }
        let Some(reporter) = self.reporter.as_mut() else {
            return;
        };
        if let Some(report) = reporter.report(
            Instant::now(),
            position,
            heading_degrees,
            motion,
            jump_velocity,
        ) {
            self.send(ClientEvent::Position(report));
        }
    }

    /// Leaves the world (if in it) and closes the connection.
    pub fn log_off(&mut self) {
        if let Some(character) = self.player() {
            self.send(ClientEvent::LogOffCharacter { character });
        }
        self.phase = Phase::Closed;
        self.client.disconnect();
    }
}

impl Drop for OnlineSession {
    fn drop(&mut self) {
        if self.phase != Phase::Closed {
            self.log_off();
        }
    }
}
