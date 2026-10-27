//! `ClientNet`'s connections: referrals (`HandleReferral`, `ProcessReferralQueue`), each
//! frame's `ProcessConnections` (connect-ack repeats, ACK/NAK, the 140 s silence check),
//! `RemoveConnection` and `LogOffServer`.

use std::net::SocketAddr;

use super::{ClientNet, NetEvent, ReferralEntry, CONNECT_ACK_SECONDS, SILENCE_SECONDS};
use crate::flow::standalone;
use crate::optional::OptionalHeader;
use crate::receiver::ConnectionState;
use crate::NetError;

impl ClientNet {
    /// `ClientNet::HandleReferral`: queue a world login request to a new server.
    pub(super) fn handle_referral(&mut self, referral: &crate::optional::Referral) {
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
    pub(super) fn process_referral_queue(&mut self) {
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
    pub(super) fn process_connections(&mut self) {
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
                    self.link_heartbeat(id, clock.local_time);
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
            // 140 s without data (unless the client itself stalled that long since its last
            // frame): the connection is taken as gone; a connected one that came by referral is
            // referred to again. The disconnect then runs its course (`RemoveConnection` tells
            // the game the server died).
            let Some(r) = self.receivers.get(&id) else {
                continue;
            };
            if clock.cur_time - self.last_did_use_time < SILENCE_SECONDS
                && SILENCE_SECONDS < clock.local_time - r.last_got_data
                && r.state < ConnectionState::DisconnectReceived
            {
                let (was, cookie, addr) = (r.state, r.cookie, r.addr);
                self.set_connection_state(id, ConnectionState::DisconnectReceived);
                if was == ConnectionState::Connected && cookie != 0 && !self.log_off_sent {
                    let referral = crate::optional::Referral {
                        cookie,
                        addr: sockaddr_in(addr),
                        server_id: id,
                        padding: 0,
                        tail: 0,
                    };
                    self.handle_referral(&referral);
                }
            }
        }
    }

    /// The world's part of `ClientNet::ProcessConnection`: every two seconds the world receiver
    /// closes the link snapshot (its round trip, the time since it last got data, the time since
    /// the last snapshot), averages it, tells the link-status plugins (`NotifyPluginsOfHeartbeat`)
    /// and starts a new interval.
    fn link_heartbeat(&mut self, id: u16, local_time: f64) {
        if id != self.world_rec {
            return;
        }
        let Some(r) = self.receivers.get_mut(&id) else {
            return;
        };
        if local_time - r.last_heartbeat < 2.0 {
            return;
        }
        r.last_heartbeat = local_time;
        let mut snap = self.link_current;
        snap.round_trip = r.round_trip;
        snap.since_last_data = (local_time - r.last_got_data) as f32;
        snap.duration = (local_time - self.link.local_time_of_snapshot()) as f32;
        self.link.add_snapshot(&snap, local_time);
        self.link_current = Default::default();
        // `NotifyPluginsOfHeartbeat` runs once per frame at most, so one heartbeat is held.
        self.link_heartbeat = Some(Box::new(self.link.clone()));
    }

    /// `ClientNet::RemoveConnection`.
    pub(super) fn remove_connection(&mut self, id: u16) {
        self.controller.delete_recipient(id);
        self.connections.retain(|&c| c != id);
        if let Some(r) = self.receivers.remove(&id) {
            self.slots.insert(id, (r.remote_interval, r.bytes_received));
        }
        if (id == self.logon_rec || id == self.world_rec) && !self.log_off_sent {
            self.log_off_server();
            // `ECM_Login::SendNotice_ServerDied` (notice 0x1040D).
            self.events
                .push(NetEvent::Disconnected(NetError::ServerDied));
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
}

/// A `sockaddr_in` as the referral carries it: family, port and address in network order.
fn sockaddr_in(addr: SocketAddr) -> crate::optional::SockAddrIn {
    let mut bytes = [0u8; 16];
    bytes[..2].copy_from_slice(&2u16.to_le_bytes());
    bytes[2..4].copy_from_slice(&addr.port().to_be_bytes());
    if let SocketAddr::V4(v4) = addr {
        bytes[4..8].copy_from_slice(&v4.ip().octets());
    }
    crate::optional::SockAddrIn(bytes)
}
