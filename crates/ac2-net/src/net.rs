//! `ClientNet`: the client side of the protocol, sans IO, one retail frame at a time.
//!
//! [`ClientNet::use_time`] is `Timer::update_time`, `ClientNet::UseTime` (the logon retry,
//! every datagram that arrived, `ProcessConnections`) and `PacketController::UseTime` (every
//! queued flush), in that order, as `Turbine::Client::UseTime` runs them each frame. Blobs the
//! game sends during a frame ([`ClientNet::send_buff`]) are queued and leave in the next
//! frame's flush.

use std::collections::BTreeMap;
use std::net::SocketAddr;

use ac2_core::Timer;

use crate::blob::{stamp_newer, Blob};
use crate::controller::{Clock, PacketController};
use crate::flow::{standalone, Destination};
use crate::handshake::Logon;
use crate::header::{flags, PacketHeader, HEADER_LEN};
use crate::optional::{Connect, NetErrorId, OptionalHeader};
use crate::packet::{split_packet, InboundPacket, MAX_DATAGRAM};
use crate::receiver::{ConnectionState, Receiver};
use crate::NetError;

/// Logon requests sent before giving up (`ClientNet::LogonUseTime`: more than `0x13` fails).
const MAX_LOGON_REQUESTS: u32 = 0x14;
/// Seconds of `get_real_time` between logon requests.
const LOGON_RETRY_SECONDS: f64 = 2.0;
/// Seconds between connect-ack repeats while waiting for the first packet.
const CONNECT_ACK_SECONDS: f64 = 0.333_333_333;
/// Seconds without data after which a connection is treated as gone.
const SILENCE_SECONDS: f64 = 140.0;

/// What the transport reports to the game.
#[derive(Debug, Clone, PartialEq)]
pub enum NetEvent {
    /// A server answered the logon (`ClientNet::HandleConnectionRequest`).
    Connected { net_id: u32, server_time: f64 },
    /// The first packet after the connect-ack arrived: logged on (`Net_LoginConnected`).
    LoggedOn,
    /// A completed blob for receive queue 1..8 (`PacketController::AddReceivedBlobToQueue`).
    Blob(Blob),
    /// The server clock was adopted (`ClientNet::HandleTimeSynch`).
    TimeSync(f64),
    /// The server moved the client's world (`ssWorldSwitch`) or logon (`ssLogonSwitch`)
    /// connection to the connection the header came on.
    ServerSwitch { world: bool, recipient: u16 },
    /// The session ended; no further events follow.
    Disconnected(NetError),
}

/// `ClientNet::CReferralQueueEntry`.
#[derive(Debug, Clone)]
struct ReferralEntry {
    server_id: u16,
    addr: SocketAddr,
    cookie: u64,
    next_send: f64,
    sent: u32,
}

/// The client transport state.
pub struct ClientNet {
    timer: Timer,
    elapsed: f64,
    real_time: i64,
    server: SocketAddr,
    logon: Vec<u8>,
    logging_on: bool,
    logon_requests: u32,
    last_logon: i64,
    /// `logonRecID_` and `currServerRecID_` (the low and high words of
    /// `m_dwCurrentConnectionSequenceNumber` in the admin PDB).
    logon_rec: u16,
    world_rec: u16,
    receivers: BTreeMap<u16, Receiver>,
    /// `connectionsHead_`: connected receivers in connection order.
    connections: Vec<u16>,
    /// Interval and byte count kept by a `ReceiverData` slot that has no connection.
    slots: BTreeMap<u16, (u16, u32)>,
    controller: PacketController,
    referrals: Vec<ReferralEntry>,
    /// `Proto_UI::orderedNumber_`, `unorderedNumber_` and `CCliDatEvent::s_unorderedStamp`.
    ordered: u16,
    unordered: u16,
    dat_stamp: u16,
    /// `PacketController::m_curNonEphemeralID`.
    non_ephemeral: u64,
    log_off_sent: bool,
    in_world: bool,
    world_switch: Option<u32>,
    logon_switch: Option<u32>,
    /// `m_LastDidUseTime`: when the server clock first lagged ours by a minute.
    lagging_since: f64,
    out: Vec<(SocketAddr, Vec<u8>)>,
    events: Vec<NetEvent>,
}

impl ClientNet {
    /// `ClientNet::Init`: ready to send `logon` to `server` at the first frame.
    pub fn new(server: SocketAddr, logon: &Logon) -> Self {
        Self {
            timer: Timer::new(),
            elapsed: 0.0,
            real_time: 0,
            server,
            logon: logon.datagram(),
            logging_on: true,
            logon_requests: 0,
            last_logon: 0,
            logon_rec: 0,
            world_rec: 0,
            receivers: BTreeMap::new(),
            connections: Vec::new(),
            slots: BTreeMap::new(),
            controller: PacketController::default(),
            referrals: Vec::new(),
            ordered: 0,
            unordered: 0,
            dat_stamp: 0,
            non_ephemeral: 0,
            log_off_sent: false,
            in_world: false,
            world_switch: None,
            logon_switch: None,
            lagging_since: 0.0,
            out: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn logon_recipient(&self) -> u16 {
        self.logon_rec
    }

    pub fn world_recipient(&self) -> u16 {
        self.world_rec
    }

    pub fn is_logging_on(&self) -> bool {
        self.logging_on
    }

    /// The connection state of `recipient`, if connected.
    pub fn connection_state(&self, recipient: u16) -> Option<ConnectionState> {
        self.receivers.get(&recipient).map(|r| r.state)
    }

    /// One frame. `elapsed` is the monotonic seconds since start (`Timer::update_time`),
    /// `real_time` the Unix seconds (`Timer::get_real_time`), `inbound` the datagrams that
    /// arrived since the last frame. Returns the datagrams to send, in order.
    pub fn use_time(
        &mut self,
        elapsed: f64,
        real_time: i64,
        inbound: Vec<(SocketAddr, Vec<u8>)>,
    ) -> (Vec<(SocketAddr, Vec<u8>)>, Vec<NetEvent>) {
        self.timer.update_time(elapsed);
        self.elapsed = elapsed;
        self.real_time = real_time;
        if self.logging_on {
            self.logon_use_time();
        }
        for (from, bytes) in inbound {
            self.process_datagram(from, &bytes);
        }
        self.process_connections();
        self.flush();
        (
            std::mem::take(&mut self.out),
            std::mem::take(&mut self.events),
        )
    }

    fn clock(&self) -> Clock {
        Clock {
            local_time: self.timer.local_time(),
            cur_time: self.timer.cur_time(),
            server_time: self.timer.server_time_at(self.elapsed),
        }
    }

    /// `ClientNet::SendBuff`: nothing leaves once the logoff went out.
    fn send(&mut self, to: SocketAddr, datagram: Vec<u8>) {
        if !self.log_off_sent {
            self.out.push((to, datagram));
        }
    }

    /// `PacketController::UseTime`.
    fn flush(&mut self) {
        let clock = self.clock();
        let mut sent = Vec::new();
        self.controller
            .use_time(clock, &mut self.receivers, |id, to, datagram| {
                sent.push((id, to, datagram))
            });
        for (id, to, datagram) in sent {
            if let Some(addr) = self.receivers.get(&id).map(|r| r.addr) {
                self.send(destination(addr, to), datagram);
            }
        }
    }

    /// `ClientNet::LogonUseTime`.
    fn logon_use_time(&mut self) {
        if self.logon_rec == 0 {
            let due = self.logon_requests == 0
                || LOGON_RETRY_SECONDS <= (self.real_time - self.last_logon) as f64;
            if !due {
                return;
            }
            if self.logon_requests >= MAX_LOGON_REQUESTS {
                self.end_logon(Err(NetError::Timeout));
                return;
            }
            self.logon_requests += 1;
            self.last_logon = self.real_time;
            let datagram = self.logon.clone();
            self.send(self.server, datagram);
        } else if self
            .receivers
            .get(&self.logon_rec)
            .is_none_or(|r| r.state > ConnectionState::Connected)
        {
            self.end_logon(Err(NetError::Sequence));
        }
    }

    /// `ClientNet::EndState`.
    fn end_logon(&mut self, outcome: Result<(), NetError>) {
        self.logging_on = false;
        self.events.push(match outcome {
            Ok(()) => NetEvent::LoggedOn,
            Err(error) => NetEvent::Disconnected(error),
        });
    }

    /// `SharedNet::ReadNextPacket` and `ClientNet::ProcessPacket`.
    fn process_datagram(&mut self, from: SocketAddr, bytes: &[u8]) {
        if bytes.len() < HEADER_LEN || bytes.len() > MAX_DATAGRAM + HEADER_LEN {
            return;
        }
        let header = PacketHeader::decode(bytes).expect("20-byte header");
        if !self.verify_header(from, &header) {
            return;
        }
        let id = header.recipient;
        let encrypted = header.flags & flags::ENCRYPTED_CHECKSUM != 0;
        let mut key = None;
        if header.sequence == 0 {
            if encrypted {
                return;
            }
        } else {
            let Some(receiver) = self.receivers.get_mut(&id) else {
                return;
            };
            match receiver.process_sequence(&header) {
                Some(k) => key = k,
                None => return,
            }
        }
        let mut checksum = header.checksum.wrapping_sub(header.header_checksum());
        if let Some(key) = key {
            checksum ^= key;
        }
        let connected = self.receivers.contains_key(&id);
        let packet = match split_packet(bytes, checksum, connected) {
            Ok(packet) => packet,
            Err(_) => {
                // A damaged sequenced packet is NAKed again under the same key.
                if let (Some(receiver), Some(key)) = (self.receivers.get_mut(&id), key) {
                    receiver.add_nakked(header.sequence, Some(key));
                }
                return;
            }
        };
        self.process_packet(from, packet, bytes.len() as u32);
    }

    /// `ClientNet::VerifyHeader`.
    fn verify_header(&self, from: SocketAddr, header: &PacketHeader) -> bool {
        if header.recipient > 0xFF {
            return false;
        }
        let Some(receiver) = self.receivers.get(&header.recipient) else {
            return header.sequence == 0;
        };
        if receiver.addr.ip() != from.ip() {
            return false;
        }
        let order = overflow_compare(header.iteration, receiver.iteration);
        if order < 0 || (order > 0 && header.flags & flags::CONNECT == 0) {
            return false;
        }
        usize::from(header.size) < 0xFFE1
    }

    /// `SharedNet::ProcessPacket` after the split, then the `ClientNet::ProcessPacket` state step.
    fn process_packet(&mut self, from: SocketAddr, packet: InboundPacket, len: u32) {
        let id = packet.header.recipient;
        let interval = packet.header.interval;
        let local_time = self.timer.local_time();
        // `SharedNet::ProcessNewRemoteInterval`, then the byte count; a slot without a
        // connection keeps its own interval and count.
        let flow = match self.receivers.get_mut(&id) {
            Some(r) => {
                let flow = (r.remote_interval == 0 || stamp_newer(interval, r.remote_interval))
                    .then(|| r.remote_interval(interval))
                    .flatten();
                r.bytes_received = r.bytes_received.wrapping_add(len);
                flow
            }
            None => {
                let slot = self.slots.entry(id).or_insert((0, 0));
                if slot.0 == 0 || stamp_newer(interval, slot.0) {
                    *slot = (interval, 0);
                }
                slot.1 = slot.1.wrapping_add(len);
                None
            }
        };
        if let Some(flow) = flow {
            self.controller.enqueue_header(id, flow, local_time);
        }
        for header in &packet.optional {
            self.process_optional_header(from, &packet.header, header);
        }
        if let Some(r) = self.receivers.get_mut(&id) {
            r.last_got_data = local_time;
        }
        if !packet.fragments.is_empty() {
            let cur_time = self.timer.cur_time();
            for blob in self.controller.indicate(id, &packet.fragments, cur_time) {
                // `PacketController::AddReceivedBlobToQueue`: queues 1..8 only.
                if (1..9).contains(&blob.queue) {
                    self.events.push(NetEvent::Blob(blob));
                }
            }
        }
        let Some(r) = self.receivers.get_mut(&id) else {
            return;
        };
        match r.state {
            ConnectionState::ConnectionRequestAcked
                if packet.header.flags & flags::CONNECT == 0 =>
            {
                r.cookie = 0;
                self.set_connection_state(id, ConnectionState::Connected);
                if self.logging_on {
                    self.end_logon(Ok(()));
                }
            }
            ConnectionState::Connected => r.cookie = 0,
            _ => {}
        }
    }

    /// `SharedNet::SetConnectionState`: the receiver and its recipient.
    fn set_connection_state(&mut self, id: u16, state: ConnectionState) {
        let local_time = self.timer.local_time();
        if let Some(r) = self.receivers.get_mut(&id) {
            r.set_state(state, local_time);
        }
        if let Some(r) = self.controller.recipients.get_mut(&id) {
            r.state = state;
        }
    }

    /// `ClientNet::ProcessOptionalHeader` (with `SharedNet::ProcessOptionalHeader` first).
    fn process_optional_header(
        &mut self,
        from: SocketAddr,
        packet: &PacketHeader,
        header: &OptionalHeader,
    ) {
        let id = packet.recipient;
        let local_time = self.timer.local_time();
        match header {
            OptionalHeader::TimeSync(time) => self.handle_time_sync(id, *time),
            OptionalHeader::EchoRequest(echo) => {
                self.controller.enqueue_header(
                    id,
                    OptionalHeader::EchoResponse {
                        echo: *echo,
                        holding: 0.0,
                    },
                    local_time,
                );
            }
            OptionalHeader::EchoResponse { echo, holding } => {
                if let Some(r) = self.receivers.get_mut(&id) {
                    r.round_trip = ((f64::from(local_time as f32) - f64::from(*echo))
                        - f64::from(*holding)) as f32;
                }
            }
            OptionalHeader::NetErrorDisconnect(reason) => {
                self.set_connection_state(id, ConnectionState::DisconnectReceived);
                if *reason != NetErrorId::NONE {
                    self.events.push(NetEvent::Disconnected(NetError::Server {
                        string_id: reason.string_id,
                        table_id: reason.table_id,
                    }));
                }
            }
            OptionalHeader::Nak(ids) => self.controller.process_naks(id, ids),
            OptionalHeader::EmptyAck(ids) => {
                if let Some(r) = self.receivers.get_mut(&id) {
                    r.forget(ids);
                }
            }
            OptionalHeader::Ack(num) => self.controller.queue_flush(id, *num),
            OptionalHeader::NetError(reason) => {
                let error = NetError::Server {
                    string_id: reason.string_id,
                    table_id: reason.table_id,
                };
                if self.logging_on {
                    self.end_logon(Err(error));
                } else {
                    self.log_off_server();
                    self.events.push(NetEvent::Disconnected(error));
                }
            }
            OptionalHeader::Referral(referral) => self.handle_referral(referral),
            OptionalHeader::ServerSwitch { stamp, kind } => {
                self.handle_server_switch(id, *stamp, *kind)
            }
            OptionalHeader::Connect(connect) => {
                self.handle_connection_request(from, packet, connect)
            }
            _ => {}
        }
    }

    /// `ClientNet::HandleTimeSynch`: adopt the server clock; a clock running a minute ahead of
    /// the server's for a minute reports `ID_ConnectionError_RunningSpeedhack`.
    fn handle_time_sync(&mut self, id: u16, time: f64) {
        let ours = self.timer.cur_time();
        self.timer.set_time(time, self.elapsed);
        self.events.push(NetEvent::TimeSync(time));
        if ours <= time + 60.0 {
            self.lagging_since = 0.0;
        } else if self.lagging_since == 0.0 {
            self.lagging_since = ours;
        } else if self.lagging_since + 60.0 < ours {
            let speedhack = NetErrorId {
                string_id: tokenize(b"ID_ConnectionError_RunningSpeedhack"),
                table_id: 0x4000_0005,
            };
            let local_time = self.timer.local_time();
            self.controller
                .enqueue_header(id, OptionalHeader::NetError(speedhack), local_time);
        }
    }

    /// `ClientNet::HandleServerSwitch`: a newer switch stamp moves the world or logon
    /// connection.
    fn handle_server_switch(&mut self, id: u16, stamp: u32, kind: u32) {
        let history = match kind {
            0 => &mut self.world_switch,
            1 => &mut self.logon_switch,
            _ => return,
        };
        if let Some(last) = *history {
            if !crate::blob::sequence_newer(stamp, last) {
                return;
            }
        }
        *history = Some(stamp);
        if kind == 0 {
            self.world_rec = id;
        } else {
            if !self.in_world {
                self.world_rec = id;
            }
            self.logon_rec = id;
        }
        self.events.push(NetEvent::ServerSwitch {
            world: kind == 0,
            recipient: id,
        });
    }

    /// `ClientNet::HandleConnectionRequest`.
    fn handle_connection_request(
        &mut self,
        from: SocketAddr,
        packet: &PacketHeader,
        connect: &Connect,
    ) {
        let id = packet.recipient;
        if let Some(existing) = self.receivers.get(&id) {
            if overflow_compare(packet.iteration, existing.iteration) < 1 {
                return;
            }
            self.remove_connection(id);
        }
        let clock = self.clock();
        // `ReceiverData::Init` resets the interval but not the bytes counted on the slot.
        let (_, bytes) = self.slots.remove(&id).unwrap_or((0, 0));
        let mut receiver = Receiver::new(
            from,
            connect.net_id as u16,
            packet.iteration,
            packet.interval,
            connect.outgoing_seed,
            connect.incoming_seed,
            clock.cur_time,
            clock.local_time,
        );
        receiver.bytes_received = bytes;
        receiver.cookie = connect.cookie;
        let net_id = receiver.net_id;
        self.receivers.insert(id, receiver);
        self.controller
            .add_recipient(id, net_id, clock.cur_time, clock.local_time);
        self.connections.push(id);
        if self.logon_rec == 0 {
            self.logon_rec = id;
            self.world_rec = id;
        }
        self.referrals.retain(|r| r.server_id != id);
        self.set_connection_state(id, ConnectionState::ConnectionRequestAcked);
        self.send_connect_ack(id);
        self.events.push(NetEvent::Connected {
            net_id: connect.net_id,
            server_time: connect.server_time,
        });
    }

    /// `ClientNet::SendConnectAck`: the cookie, alone, to the server's port plus one.
    fn send_connect_ack(&mut self, id: u16) {
        let local_time = self.timer.local_time();
        let Some(r) = self.receivers.get_mut(&id) else {
            return;
        };
        r.last_sent_handshake = local_time;
        let datagram = standalone(OptionalHeader::ConnectAck(r.cookie), r.net_id, r.iteration);
        let to = destination(r.addr, Destination::PeerPortPlusOne);
        self.send(to, datagram);
    }

    /// `ClientNet::HandleReferral`: queue a world login request to a new server.
    fn handle_referral(&mut self, referral: &crate::optional::Referral) {
        let id = referral.server_id;
        if id > 0xFF {
            return;
        }
        if let Some(r) = self.receivers.get_mut(&id) {
            if r.state <= ConnectionState::Connected {
                r.cookie = referral.cookie;
                return;
            }
        }
        if self
            .receivers
            .get(&id)
            .is_some_and(|r| r.state > ConnectionState::Connected)
        {
            self.remove_connection(id);
        }
        if self.referrals.iter().any(|r| r.server_id == id) {
            return;
        }
        let ip = referral.addr.ip();
        let addr = SocketAddr::from((ip, referral.addr.port()));
        self.referrals.push(ReferralEntry {
            server_id: id,
            addr,
            cookie: referral.cookie,
            next_send: 0.0,
            sent: 0,
        });
        self.process_referral_queue();
    }

    /// `ClientNet::ProcessReferralQueue`: a world login request every third of a second, up to
    /// 840 per referral.
    fn process_referral_queue(&mut self) {
        let local_time = self.timer.local_time();
        let mut i = self.referrals.len();
        while i > 0 {
            i -= 1;
            if self.referrals[i].next_send >= local_time {
                continue;
            }
            if f64::from(self.referrals[i].sent) >= 840.000_000_84 {
                let last = self.referrals.len() - 1;
                self.referrals.swap(i, last);
                self.referrals.pop();
                self.events.push(NetEvent::Disconnected(NetError::Timeout));
                continue;
            }
            let entry = &mut self.referrals[i];
            entry.sent += 1;
            entry.next_send = local_time + CONNECT_ACK_SECONDS;
            let (to, cookie) = (entry.addr, entry.cookie);
            self.send(
                to,
                standalone(OptionalHeader::WorldLoginRequest(cookie), 0, 0),
            );
        }
    }

    /// `ClientNet::ProcessConnections` and `ClientNet::ProcessConnection`.
    fn process_connections(&mut self) {
        self.process_referral_queue();
        let clock = self.clock();
        for id in self.connections.clone() {
            let Some(r) = self.receivers.get(&id) else {
                continue;
            };
            let state = r.state;
            match state {
                ConnectionState::ConnectionRequestAcked => {
                    if CONNECT_ACK_SECONDS < clock.local_time - r.last_sent_handshake {
                        self.send_connect_ack(id);
                    }
                }
                ConnectionState::DisconnectReceived | ConnectionState::Connected => {
                    if state == ConnectionState::DisconnectReceived
                        && self
                            .controller
                            .recipients
                            .get(&id)
                            .is_none_or(|r| r.state == ConnectionState::DisconnectSent)
                    {
                        self.set_connection_state(id, ConnectionState::DisconnectSent);
                    }
                    let ack = self
                        .receivers
                        .get_mut(&id)
                        .and_then(|r| r.acknowledgement(clock.cur_time));
                    if let Some(ack) = ack {
                        self.controller.enqueue_header(id, ack, clock.local_time);
                    }
                }
                ConnectionState::DisconnectSent => {
                    self.remove_connection(id);
                    continue;
                }
                _ => {}
            }
            if let Some(r) = self.receivers.get(&id) {
                if SILENCE_SECONDS < clock.local_time - r.last_got_data
                    && r.state < ConnectionState::DisconnectReceived
                {
                    self.set_connection_state(id, ConnectionState::DisconnectReceived);
                    self.events.push(NetEvent::Disconnected(NetError::Timeout));
                }
            }
        }
    }

    /// `ClientNet::RemoveConnection`.
    fn remove_connection(&mut self, id: u16) {
        self.controller.delete_recipient(id);
        self.connections.retain(|&c| c != id);
        if let Some(r) = self.receivers.remove(&id) {
            self.slots.insert(id, (r.remote_interval, r.bytes_received));
        }
        if (id == self.logon_rec || id == self.world_rec) && !self.log_off_sent {
            self.log_off_server();
            self.events
                .push(NetEvent::Disconnected(NetError::Io("server died".into())));
        }
    }

    /// `ClientNet::LogOffServer`: a logoff header alone to every connection, then silence.
    pub fn log_off_server(&mut self) {
        for id in self.connections.clone() {
            if let Some(r) = self.receivers.get(&id) {
                let datagram = standalone(OptionalHeader::Logoff, r.net_id, r.iteration);
                let to = r.addr;
                self.send(to, datagram);
            }
        }
        self.log_off_sent = true;
        self.logon_rec = 0;
        // `ClientNet::ExitWorldDisconnect`.
        self.in_world = false;
        self.world_rec = self.logon_rec;
        for id in self.connections.clone() {
            if id != self.logon_rec {
                self.remove_connection(id);
            }
        }
    }

    /// Datagrams queued outside a frame (the logoff).
    pub fn take_datagrams(&mut self) -> Vec<(SocketAddr, Vec<u8>)> {
        std::mem::take(&mut self.out)
    }
}

/// `TimeStampUtils::OverflowCompare`.
fn overflow_compare(a: u16, b: u16) -> i32 {
    if a == b {
        0
    } else if stamp_newer(a, b) {
        1
    } else {
        -1
    }
}

fn destination(addr: SocketAddr, to: Destination) -> SocketAddr {
    match to {
        Destination::Peer => addr,
        Destination::PeerPortPlusOne => SocketAddr::new(addr.ip(), addr.port().wrapping_add(1)),
    }
}

/// `Tokenize(char *)`: string tokens of `NetError` ids.
pub fn tokenize(text: &[u8]) -> u32 {
    let mut hash = 0u32;
    let mut shift = 0u32;
    for &c in text {
        let v = (i32::from(c as i8) << 25) as u32;
        hash ^= v >> shift;
        if shift > 25 {
            hash ^= v << ((32 - shift) & 31);
        }
        shift = (shift + 7) & 31;
    }
    hash
}

#[path = "pui.rs"]
mod pui;

#[cfg(test)]
#[path = "net_tests.rs"]
mod tests;
