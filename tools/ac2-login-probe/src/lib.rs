//! A headless logon probe (test tooling): sends the retail logon to an AC2 server, answers the
//! DAT interrogation, waits for the character set, and logs off the way the client does when
//! the user quits at character selection. Each stage is reported as it happens, so a run
//! against a real server shows how far the logon gets.
//!
//! Quitting at character selection is `Turbine::Client::Disconnect` (Legions 0x006A0C40),
//! which calls `ClientNet::LogOffServer` (0x004BB9D0): a logoff header alone to every
//! connection, then silence. [`ac2_weenie::OnlineSession::log_off`] does exactly that.
//!
//! The probe never prints or formats the password or ticket: [`ProbeConfig`] has a `Debug`
//! that leaves the credential out, and [`Stage`] has none.

mod reason;

use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

use ac2_net::protocol::CharacterSet;
use ac2_net::{flags, Credential, Logon, NetError, PacketHeader, Transport};
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent, SessionTrace};

pub use reason::{describe as describe_reason, name as reason_name};

/// Pause between frames while waiting.
const FRAME: Duration = Duration::from_millis(10);

/// What to probe.
#[derive(Clone)]
pub struct ProbeConfig {
    pub server: SocketAddr,
    /// Sent as given; the command line lowercases it ([`retail_account`]).
    pub account: String,
    pub credential: Credential,
    /// The data version in the logon: `"7160"` (Legions) or `"7148"` (admin).
    pub version: String,
    /// The DATs' ids and iterations for the interrogation response (empty without DATs).
    pub dat_iterations: Vec<(u64, Vec<i32>)>,
    /// How long to wait for the character set after the first frame.
    pub timeout: Duration,
}

impl fmt::Debug for ProbeConfig {
    /// Leaves the credential out: it may hold a password or ticket.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProbeConfig")
            .field("server", &self.server)
            .field("account", &self.account)
            .field("version", &self.version)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

/// `Turbine::Client::EvaluateCommandLineArg` (Legions 0x006A1EB0) lowercases the account name
/// (`_strlwr`) before it reaches the logon.
pub fn retail_account(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// The first IPv4 address `host` resolves to. A tiny private resolver: `host` is a name or a
/// dotted address.
pub fn resolve_ipv4(host: &str, port: u16) -> Result<SocketAddr, String> {
    (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("{host}: {e}"))?
        .find(SocketAddr::is_ipv4)
        .ok_or_else(|| format!("{host}: no IPv4 address"))
}

/// A logon stage, reported as it happens. Carries no credential.
#[derive(Clone, PartialEq, Eq)]
pub enum Stage {
    /// The logon datagram left (`ClientNet::SendLoginRequest`). `resend` counts repeats.
    LogonSent {
        version: String,
        auth_type: u32,
        bytes: usize,
        resend: u32,
    },
    /// The first datagram of any kind from the server.
    FirstReply { from: SocketAddr, bytes: usize },
    /// The server's connect header (`ClientNet::HandleConnectionRequest`).
    Connected { net_id: u32 },
    /// `Net_LoginConnected`: the first packet after the connect-ack.
    LoggedOn,
    /// `CLCache::OnServerInterrogation` arrived.
    Interrogation { payload_len: usize },
    /// The interrogation response went out.
    InterrogationAnswered { language: u32, dat_files: usize },
    /// `ECM_Login::RecvEvt_CharacterSet`.
    CharacterSet {
        account: String,
        allowed: u32,
        status: u32,
        characters: Vec<(u64, String)>,
        deleted: usize,
    },
    /// A server message this client could not decode, by opcode and length.
    Undecoded {
        opcode: u32,
        length: usize,
        error: String,
    },
    /// The server is closing the connection for this reason.
    ConnectionError { world: bool, reason: String },
    /// The session ended.
    Disconnected { reason: String },
    /// The logoff left (`ClientNet::LogOffServer`).
    LogoffSent,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LogonSent {
                version,
                auth_type,
                bytes,
                resend: 0,
            } => write!(
                f,
                "logon sent: version {version}, auth type {auth_type:#010x}, no session key (auth flags 0), {bytes} bytes"
            ),
            Self::LogonSent { resend, bytes, .. } => {
                write!(f, "logon sent again (#{resend}): {bytes} bytes")
            }
            Self::FirstReply { from, bytes } => write!(f, "server replied: {bytes} bytes from {from}"),
            Self::Connected { net_id } => write!(f, "connect: server accepted the logon (net id {net_id:#x})"),
            Self::LoggedOn => write!(f, "logged on: first packet after the connect-ack"),
            Self::Interrogation { payload_len } => {
                write!(f, "interrogation: server asks for the DAT iterations ({payload_len} bytes)")
            }
            Self::InterrogationAnswered { language, dat_files } => write!(
                f,
                "interrogation answered: language {language}, {dat_files} DAT file(s) listed"
            ),
            Self::CharacterSet {
                account,
                allowed,
                status,
                characters,
                deleted,
            } => {
                write!(
                    f,
                    "character set: account {account:?}, status {status}, {} of {allowed} slots, {deleted} deleted",
                    characters.len()
                )?;
                for (id, name) in characters {
                    write!(f, "\n  character {id:#018x} {name:?}")?;
                }
                Ok(())
            }
            Self::Undecoded {
                opcode,
                length,
                error,
            } => write!(
                f,
                "undecodable server message: opcode {opcode:#010x}, {length} bytes ({error})"
            ),
            Self::ConnectionError { world, reason } => write!(
                f,
                "server closing the {} connection: {reason}",
                if *world { "world" } else { "logon" }
            ),
            Self::Disconnected { reason } => write!(f, "disconnected: {reason}"),
            Self::LogoffSent => write!(f, "logoff sent (ClientNet::LogOffServer)"),
        }
    }
}

/// Which stage failed; each has its own process exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// Usage, DAT or exe setup problems before any network use.
    Setup,
    /// The host name did not resolve to an IPv4 address.
    Resolve,
    /// No reply (or no character set) before the timeout, or the logon retries ran out.
    Timeout,
    /// The server answered and refused or closed the logon with a reason.
    Refused,
    /// The server's answer did not decode, or the session broke the protocol.
    Protocol,
}

impl FailureKind {
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Setup => 1,
            Self::Resolve => 2,
            Self::Timeout => 3,
            Self::Refused => 4,
            Self::Protocol => 5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Resolve => "resolve",
            Self::Timeout => "no reply / timeout",
            Self::Refused => "refused",
            Self::Protocol => "protocol error",
        }
    }
}

/// Why the probe stopped short of the character set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub kind: FailureKind,
    pub detail: String,
}

impl Failure {
    pub fn new(kind: FailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

/// What the net layer and the transport saw, in order.
enum Seen {
    Trace(SessionTrace),
    Sent(Vec<u8>),
    Received(SocketAddr, usize),
}

/// Reports each datagram that leaves or arrives, then defers to `inner`.
struct Tracing {
    inner: Box<dyn Transport>,
    seen: Sender<Seen>,
}

impl Transport for Tracing {
    fn receive(&mut self) -> Result<Vec<(SocketAddr, Vec<u8>)>, NetError> {
        let inbound = self.inner.receive()?;
        for (from, bytes) in &inbound {
            let _ = self.seen.send(Seen::Received(*from, bytes.len()));
        }
        Ok(inbound)
    }

    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), NetError> {
        let _ = self.seen.send(Seen::Sent(datagram.to_vec()));
        self.inner.send(to, datagram)
    }

    fn clock(&mut self) -> (f64, i64) {
        self.inner.clock()
    }

    fn sleep(&mut self, duration: Duration) {
        self.inner.sleep(duration)
    }
}

/// The failure a final `NetError` stands for.
fn classify(error: &NetError) -> FailureKind {
    match error {
        NetError::Timeout => FailureKind::Timeout,
        NetError::Server { .. } | NetError::ServerDied => FailureKind::Refused,
        _ => FailureKind::Protocol,
    }
}

fn error_text(error: &NetError) -> String {
    match error {
        NetError::Server {
            string_id,
            table_id,
        } => describe_reason(*string_id, *table_id),
        NetError::Timeout => "the server did not answer the logon".into(),
        NetError::ServerDied => "the server closed the connection".into(),
        other => other.to_string(),
    }
}

/// Runs the probe over a UDP socket to `config.server`.
pub fn run_udp(
    config: ProbeConfig,
    report: &mut dyn FnMut(&Stage),
) -> Result<CharacterSet, Failure> {
    let transport = ac2_net::UdpTransport::bind(config.server)
        .map_err(|e| Failure::new(FailureKind::Setup, e.to_string()))?;
    run(config, Box::new(transport), report)
}

/// Runs the probe over `transport`: logon, interrogation, character set, logoff. Every stage
/// goes to `report` as it happens.
pub fn run(
    config: ProbeConfig,
    transport: Box<dyn Transport>,
    report: &mut dyn FnMut(&Stage),
) -> Result<CharacterSet, Failure> {
    let (tx, rx) = channel();
    let tracing = Tracing {
        inner: transport,
        seen: tx.clone(),
    };
    let timeout = config.timeout;
    let version = config.version.clone();
    let mut session = OnlineSession::with_transport(
        OnlineConfig {
            server: config.server,
            account: config.account,
            credential: config.credential,
            version: config.version,
            // Stop at the character set, as the character selection screen does.
            character: CharacterChoice::Wait,
            // The RSA session key is never sent: auth flags 0 and an empty blob.
            server_key: None,
            language: 0,
            dat_iterations: config.dat_iterations,
        },
        Box::new(tracing),
    )
    .map_err(|e| Failure::new(FailureKind::Protocol, e.to_string()))?;
    session.set_observer(Box::new(move |trace| {
        let _ = tx.send(Seen::Trace(trace.clone()));
    }));

    let deadline = session.elapsed() + timeout.as_secs_f64();
    let mut logons = 0u32;
    let mut replied = false;
    let mut last_error: Option<NetError> = None;
    let outcome = loop {
        let events = session.poll();
        while let Ok(seen) = rx.try_recv() {
            match seen {
                Seen::Sent(datagram) => {
                    if let Some(stage) = logon_stage(&datagram, &version, &mut logons) {
                        report(&stage);
                    }
                }
                Seen::Received(from, bytes) => {
                    if !replied {
                        replied = true;
                        report(&Stage::FirstReply { from, bytes });
                    }
                }
                Seen::Trace(trace) => {
                    if let Some(stage) = trace_stage(&trace, &mut last_error) {
                        report(&stage);
                    }
                }
            }
        }
        for event in events {
            match event {
                SessionEvent::Characters(set) => {
                    report(&Stage::CharacterSet {
                        account: set.account.clone(),
                        allowed: set.allowed,
                        status: set.status,
                        characters: set
                            .characters
                            .iter()
                            .map(|c| (c.id, c.name.clone()))
                            .collect(),
                        deleted: set.deleted.len(),
                    });
                    quit_at_selection(&mut session, report, &rx);
                    return Ok(set);
                }
                // An undecodable message is reported as a stage and does not stop the probe.
                SessionEvent::Rejected(reason) if session.is_closed() => {
                    return Err(Failure::new(FailureKind::Protocol, reason));
                }
                _ => {}
            }
        }
        if session.is_closed() {
            break match last_error.take() {
                Some(error) => Failure::new(classify(&error), error_text(&error)),
                None => Failure::new(FailureKind::Protocol, "the session closed"),
            };
        }
        if session.elapsed() >= deadline {
            break Failure::new(
                FailureKind::Timeout,
                if replied {
                    "the server answered but sent no character set in time"
                } else {
                    "no reply from the server in time"
                },
            );
        }
        session.sleep(FRAME);
    };
    Err(outcome)
}

/// The retail quit at character selection: `ClientNet::LogOffServer`.
fn quit_at_selection(
    session: &mut OnlineSession,
    report: &mut dyn FnMut(&Stage),
    rx: &std::sync::mpsc::Receiver<Seen>,
) {
    session.log_off(Duration::ZERO);
    while let Ok(seen) = rx.try_recv() {
        if let Seen::Sent(datagram) = seen {
            if PacketHeader::decode(&datagram).is_some_and(|h| h.flags & flags::LOGOFF != 0) {
                report(&Stage::LogoffSent);
            }
        }
    }
}

/// The stage for a datagram the client sent, if it is a logon.
fn logon_stage(datagram: &[u8], version: &str, count: &mut u32) -> Option<Stage> {
    let header = PacketHeader::decode(datagram)?;
    if header.flags & flags::LOGON == 0 {
        return None;
    }
    let (_, auth_type, ..) = Logon::decode_payload(datagram.get(20..)?).ok()?;
    let resend = *count;
    *count += 1;
    Some(Stage::LogonSent {
        version: version.to_owned(),
        auth_type,
        bytes: datagram.len(),
        resend,
    })
}

fn trace_stage(trace: &SessionTrace, last_error: &mut Option<NetError>) -> Option<Stage> {
    Some(match trace {
        SessionTrace::Connected { net_id } => Stage::Connected { net_id: *net_id },
        SessionTrace::LoggedOn => Stage::LoggedOn,
        SessionTrace::Interrogation { payload_len } => Stage::Interrogation {
            payload_len: *payload_len,
        },
        SessionTrace::InterrogationAnswered {
            language,
            dat_files,
        } => Stage::InterrogationAnswered {
            language: *language,
            dat_files: *dat_files,
        },
        SessionTrace::Undecoded {
            opcode,
            length,
            error,
        } => Stage::Undecoded {
            opcode: *opcode,
            length: *length,
            error: error.clone(),
        },
        SessionTrace::ConnectionError { world, error } => {
            // The reason the server gave outranks the `ServerDied` that follows it.
            *last_error = Some(error.clone());
            Stage::ConnectionError {
                world: *world,
                reason: error_text(error),
            }
        }
        SessionTrace::Disconnected(error) => {
            if !matches!(last_error, Some(NetError::Server { .. })) {
                *last_error = Some(error.clone());
            }
            Stage::Disconnected {
                reason: error_text(error),
            }
        }
    })
}
