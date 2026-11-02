//! Connection establishment and teardown: CONNECT / CONNECTED / HARD_DISCONNECT, the connect
//! retry timer, the keep-alive timer, the SP's CFRAME list. All from dpnet.dll.c.

use crate::dplay8::hresult::*;
use crate::Hresult;

use super::frames::*;
use super::signing::{Mode, SigState};
use super::{flag, OpKind, PendingConnect, Protocol, Work};

impl Protocol {
    /// `Dpl8r_BuildCFrame` (0x004266b0, dpnet.dll.c:29730). Queues the 16 bytes on the SP's
    /// CFRAME list and, the first time, queues `FUN_00425790` to send the list.
    ///
    /// POLL (0x08) is set when the conn flag 0x1000 is set or the op is CONNECT, and then a
    /// `PendingConnect` records the `bMsgID` and the time. `bMsgID` is the conn's byte `[0xe]`,
    /// post-incremented.
    pub(crate) fn build_cframe(&mut self, op: u8, rsp_id: u8) {
        let tick = self.now;
        let msg_id = self.conn.msg_id;
        let mut poll = false;
        if self.conn.has(flag::POLL_NEXT) || op == OP_CONNECT {
            self.conn
                .pending
                .push_back(PendingConnect { msg_id, sent: tick });
            poll = true;
            self.conn.clear(flag::POLL_NEXT);
        }
        self.conn.msg_id = self.conn.msg_id.wrapping_add(1);
        let f = CFrame {
            poll,
            op,
            msg_id,
            rsp_id,
            version: PROTOCOL_VERSION,
            sess_id: self.conn.sess_id,
            tick,
        };
        let mut wire = f.encode().to_vec();
        if op == OP_HARD_DISCONNECT {
            if let Some(sig) = &self.conn.sig {
                // DX9 0x00427556:33825-33839: HARD_DISCONNECT gains 8 signature bytes at +0x10;
                // FULL also sets `bRspId` to the next send sequence (`conn+0x134`).
                if sig.mode == Mode::Full {
                    wire[3] = self.conn.next_send;
                }
                wire.extend_from_slice(&[0; 8]);
                let s8 = sig.sign_control(&wire);
                wire[16..].copy_from_slice(&s8);
            }
        }
        self.cframes.push_back(wire);
        self.schedule_cframe_flush();
    }

    /// The `if (!(spd[3] & 0x20)) { |= 0x20; FUN_004274c0(FUN_00425790, ...) }` tail of the
    /// CFRAME and SACK builders.
    pub(crate) fn schedule_cframe_flush(&mut self) {
        if !self.flush_scheduled {
            self.flush_scheduled = true;
            self.timers.queue_work(Work::FlushCframes);
        }
    }

    /// `FUN_004256e0` (0x004256e0): send everything on the SP's CFRAME list now.
    pub(crate) fn flush_cframes_now(&mut self) {
        while let Some(f) = self.cframes.pop_front() {
            self.outbox.push(f);
        }
    }

    /// `FUN_00425790` (0x00425790): the queued work item; the scheduled flag drops after the
    /// list is sent.
    pub(crate) fn flush_cframes_work(&mut self) {
        self.flush_cframes_now();
        self.flush_scheduled = false;
    }

    /// Start a connect: the core's Connect reached the protocol (`FUN_00421e10` 0x00421e10
    /// allocates the conn and links the op, `conn[6]`); the SP's connect then completes
    /// (`FUN_00424170` case 3) and `Dpl8r_SendConnect` runs from the ready queue.
    pub fn connect(&mut self, now: u32) {
        self.now = now;
        self.conn.connect_pending = true;
        self.timers.queue_work(Work::SpConnectComplete);
    }

    /// `FUN_0041cc10` case 3 (dpnet.dll.c:24762-24790): cancel a connect that has not finished.
    /// The retry timer stops and the connect op completes with `result`; the conn is released.
    pub fn cancel_connect(&mut self, now: u32, result: Hresult) -> Result<(), Hresult> {
        self.now = now;
        if !self.conn.connect_pending {
            return Err(Hresult(0x8015_8110));
        }
        let mut h = self.conn.t_connect;
        self.disarm(&mut h);
        self.conn.t_connect = 0;
        self.finish_connect(Err(result), true);
        Ok(())
    }

    /// `Dpl8r_SendConnect` (0x0041eaf0, dpnet.dll.c:25784) with the SP's result `0`.
    pub(crate) fn sp_connect_complete(&mut self) {
        if !self.conn.connect_pending {
            return;
        }
        // `(conn[4] & 0xc) == 0`: not already connecting or connected.
        if self.conn.flags & (flag::CONNECTING | flag::CONNECTED) == 0 {
            self.conn.clear(flag::NEW);
            self.conn.set(flag::CONNECTING | flag::INITIATOR);
            self.conn.sess_id = self.next_sess;
            self.next_sess = self.next_sess.wrapping_add(1);
            self.build_cframe(OP_CONNECT, 0);
            self.conn.retry_base = self.cfg.connect_timeout;
            self.conn.connect_retries = self.cfg.connect_retries;
            let h = self.arm(self.conn.retry_base, 100, Work::ConnectRetry);
            self.conn.t_connect = h;
        } else {
            // FUN_0041e920(op, spd, NULL, 0x80158150)
            self.finish_connect(Err(DPNERR_CONNECTING), false);
        }
    }

    /// `FUN_0041e920` (0x0041e920): the connect op completes with `result`. With an error the
    /// conn is released (`FUN_00420e50`); the core is told through callback `+0x2c`.
    pub(crate) fn finish_connect(&mut self, result: Result<(), Hresult>, release_conn: bool) {
        self.conn.connect_pending = false;
        if result.is_err() && release_conn {
            self.release_conn();
        }
        self.events.push_back(super::Event::ConnectResult(result));
    }

    /// `FUN_00420e50` (0x00420e50): drop the established bit, mark released, stop the
    /// keep-alive timer.
    pub(crate) fn release_conn(&mut self) {
        self.conn.clear(flag::CONNECTED);
        if !self.conn.has(flag::RELEASED) {
            self.conn.set(flag::RELEASED);
            let mut h = self.conn.t_keepalive;
            self.disarm(&mut h);
            self.conn.t_keepalive = 0;
        }
    }

    /// `Dpl8r_ConnectRetryTimer` (0x0041ecd0, dpnet.dll.c:25839).
    pub(crate) fn connect_retry_timer(&mut self, id: u32) {
        if self.conn.t_connect != id {
            return;
        }
        self.conn.t_connect = 0;
        if !self.conn.connect_pending {
            return;
        }
        if self.conn.has(flag::CONNECTING) {
            let left = self.conn.connect_retries;
            self.conn.connect_retries = left.wrapping_sub(1);
            if left != 0 {
                // DX9 0x00422356:29127-29132: interval * 2, cut to 5000 when over 4999.
                self.conn.retry_base <<= 1;
                if self.conn.retry_base > 4999 {
                    self.conn.retry_base = 5000;
                }
                // `*op == 3` (a connect op): another CONNECT, same session id.
                self.build_cframe(OP_CONNECT, 0);
                let h = self.arm(self.conn.retry_base, 100, Work::ConnectRetry);
                self.conn.t_connect = h;
                return;
            }
            // Retries used up (14 resends, 15 CONNECTs, ~56.2 s: 200, 400, 800, 1600, 3200 then
            // 5000 x 10): the result stays NORESPONSE 0x80158510 (DX9 0x00422356:29145).
            self.finish_connect(Err(DPNERR_NORESPONSE), true);
        } else {
            self.conn.connect_pending = false;
        }
    }

    /// `Dpl8r_OnConnected` (0x0041f340, dpnet.dll.c:26029): the host's CONNECTED.
    pub(crate) fn on_connected(&mut self, f: &CFrame) {
        let now = self.now;
        if self.conn.sess_id == f.sess_id && !self.conn.has(flag::NEW) {
            if !self.conn.has(flag::CONNECTED) {
                if self.conn.has(flag::CONNECTING) && self.conn.connect_pending {
                    // Reply only when the host's CONNECTED had POLL; the reply echoes its bMsgID.
                    if f.poll {
                        self.build_cframe(OP_CONNECTED, f.msg_id);
                    }
                    // FUN_00423a70: the CONNECT this answers must still be outstanding.
                    let Some(rec) = self.take_pending(f.rsp_id) else {
                        return;
                    };
                    // DX9 0x00421d70:28862-28865 (`conn[5] |= 0x40` iff host minor >= 6) and
                    // 0x00421c4e:28787-28790 (`conn[5] |= 8` iff host minor < 5 or major != 1).
                    // No version is ever refused on the connector path.
                    let (major, minor) = (f.version >> 16, f.version & 0xffff);
                    self.conn.peer_v16 = major == 1 && minor >= 6;
                    self.establish_connector(now, rec, f.version);
                }
            } else if f.poll && self.conn.sig.is_none() {
                // Established already: a repeated CONNECTED+POLL is answered again, unless the
                // connection signs (`(conn[5] & 0x180) == 0`, FUN_00421d70:28852); a signing
                // one re-answers only a CONNECTED_SIGNED.
                self.build_cframe(OP_CONNECTED, f.msg_id);
            }
        } else {
            // A CONNECTED with another session id is a bad CFRAME: ignored for a client.
            self.bad_cframe();
        }
    }

    /// `FUN_00424ca3` (DX9 0x00424ca3, dpnet9.dll.c:31512; asm `test [esi+0x10],2` /
    /// `cmp dword [edi],5`): the "bad CFRAME" handler tears down only when `conn[4] & 2` and the
    /// owner object is a Listen (type 5). A client's connection is owned by a type-4 connect
    /// object, so an unknown opcode, a short frame, a first byte other than 0x80/0x88, a bad
    /// HARD_DISCONNECT, a session-id mismatch or a DFRAME before establishment is **ignored**.
    pub(crate) fn bad_cframe(&mut self) {}

    /// The tail both CONNECTED paths share (`FUN_00421c4e`, DX9 0x00421c4e:28787-28830): the
    /// connection is established.
    fn establish_connector(&mut self, now: u32, rec: PendingConnect, version: u32) {
        let (major, minor) = (version >> 16, version & 0xffff);
        self.conn.no_coalesce = major != 1 || minor < 5;
        let mut h = self.conn.t_connect;
        self.disarm(&mut h);
        self.conn.t_connect = 0;
        self.conn.set(flag::CONNECTED | 0x200);
        self.establish(now.wrapping_sub(rec.sent));
        // `*(byte*)(conn+0x130) = 0; *(byte*)(conn+0x131) = 0`
        self.conn.next_send = 0;
        self.conn.next_recv = 0;
        // FUN_00423b10: free the pending-CONNECT list.
        self.conn.pending.clear();
        // FUN_00426e30: nothing queued yet -> a KeepAlive goes out now.
        self.first_keepalive();
        // FUN_0041e920(op, spd, conn, 0): callback +0x2c, then `conn[4] &= ~0x14`.
        self.finish_connect(Ok(()), false);
        self.conn.clear(flag::CONNECTING | flag::INITIATOR);
        // `Dpl8r_DeliverReceivedFrame(conn)`: frames held since the allocator set
        // flag 0x40 (0x43) are delivered now and the flag drops.
        self.deliver();
    }

    /// `FUN_00421f01` (DX9 0x00421f01, dpnet9.dll.c:28895-29005): the host answers CONNECT with
    /// CONNECTED_SIGNED. In order: a frame with `sessID == 0`, or a different one once ours is
    /// known, is a bad CFRAME; an established connection re-answers a POLL one when it signs and
    /// ignores it otherwise; an unestablished connector accepts it only when the signing options
    /// are exactly FAST (1) or FULL (2), both secrets in the frame are zero, POLL is set and the
    /// CONNECT it answers (`bRspId`) is still pending. The connector then draws two non-zero
    /// 8-byte secrets (`CryptGenRandom`), answers with its own CONNECTED_SIGNED (`FUN_00427720`),
    /// marks the host v1.6 (`conn[5] |= 0x40`), enables FAST (`0x80`) or FULL (`0x100`) and
    /// establishes as for a plain CONNECTED. Nothing in this path asks whether the connect request
    /// wanted signing: the host decides (`verify_spec_client_delta.md` 2.3.6).
    // `conn[4] & 0x10` (0x00421f01:28900) is "the connection is being torn down" in DX9: a frame
    // arriving then is ignored; the port's RELEASED/DYING state already drops it.
    // open: a NEW connection (flag 0x2, no CONNECT sent yet) in the listener branch
    // (0x00421f01:28920-28942) is not ported; it is dropped.
    pub(crate) fn on_connected_signed(&mut self, f: &SignedFrame) {
        let now = self.now;
        let b = &f.base;
        if b.sess_id == 0 || (self.conn.sess_id != 0 && self.conn.sess_id != b.sess_id) {
            self.bad_cframe();
            return;
        }
        if self.conn.has(flag::CONNECTED) {
            if b.poll && self.conn.sig.is_some() {
                self.answer_signed(f);
            }
            return;
        }
        if !self.conn.has(flag::CONNECTING)
            || self.conn.has(flag::NEW)
            || !self.conn.connect_pending
        {
            return;
        }
        let ok = matches!(f.options & 3, 1 | 2)
            && f.sender_secret == [0; 8]
            && f.receiver_secret == [0; 8]
            && b.poll;
        if !ok {
            return;
        }
        let Some(rec) = self.take_pending(b.rsp_id) else {
            return;
        };
        let send = self.draw_secret();
        let recv = self.draw_secret();
        let mode = if f.options & 1 != 0 {
            Mode::Fast
        } else {
            Mode::Full
        };
        self.conn.sig = Some(SigState::new(mode, send, recv));
        self.answer_signed(f);
        self.conn.peer_v16 = true;
        self.establish_connector(now, rec, b.version);
    }

    /// A non-zero 8-byte secret (`CryptGenRandom(8)` repeated until non-zero,
    /// 0x00421f01:28975-28982).
    fn draw_secret(&mut self) -> [u8; 8] {
        loop {
            let mut s = [0u8; 8];
            s[..4].copy_from_slice(&self.secret_rng.next_u32().to_le_bytes());
            s[4..].copy_from_slice(&self.secret_rng.next_u32().to_le_bytes());
            if s != [0; 8] {
                return s;
            }
        }
    }

    /// `FUN_00427720`: queue our CONNECTED_SIGNED.
    fn answer_signed(&mut self, f: &SignedFrame) {
        let Some(sig) = &self.conn.sig else {
            return;
        };
        let msg_id = self.conn.msg_id;
        self.conn.msg_id = msg_id.wrapping_add(1);
        let b = SignedFrame::answer(msg_id, f.base.sess_id, self.now, f, sig.send, sig.recv);
        self.cframes.push_back(b.to_vec());
        self.schedule_cframe_flush();
    }

    /// `FUN_00423a70` (0x00423a70): find the pending CONNECT with `bMsgID == rsp`. Older
    /// records in front of it are dropped; a record newer than `rsp` (`(rsp - id) & 0x80`) ends
    /// the search with no match.
    fn take_pending(&mut self, rsp: u8) -> Option<PendingConnect> {
        while let Some(front) = self.conn.pending.front().copied() {
            if front.msg_id == rsp {
                self.conn.pending.pop_front();
                return Some(front);
            }
            if rsp.wrapping_sub(front.msg_id) & 0x80 != 0 {
                return None;
            }
            self.conn.pending.pop_front();
        }
        None
    }

    /// `Dpl8r_OnHardDisconnect` (DX9 0x004247fc, dpnet9.dll.c:31277-31290; handler
    /// precondition 0x004257da:32162-32173: established and length >= 0x10): the sessID is not
    /// compared. An established connection answers with 2 plain HARD_DISCONNECTs and a third
    /// carrying 0x200000, unless it is itself hard-disconnecting (0x40000000) or already
    /// answered (0x80000000); then it is torn down. A HARD_DISCONNECT while not established
    /// is a bad CFRAME, ignored by a client (`bad_cframe`).
    pub(crate) fn on_hard_disconnect(&mut self, _f: &CFrame) {
        if !self.conn.has(flag::CONNECTED) && !self.conn.has(flag::FORCED_DEAD) {
            self.bad_cframe();
            return;
        }
        if !self.conn.has(flag::FORCED_DEAD) && !self.conn.hd_answered {
            self.conn.hd_answered = true;
            for _ in 0..3 {
                self.build_cframe(OP_HARD_DISCONNECT, 0);
            }
        }
        self.teardown();
    }

    /// `FUN_004214b8` (DX9 0x004214b8:28313-28341) + `FUN_00427ad2`: abort. The first
    /// HARD_DISCONNECT goes out at once, the other two follow at `clamp(rtt / 2, 10, 500)` ms
    /// (granularity 10); the connection then drops (0x40000000: nothing more is accepted).
    pub fn hard_disconnect(&mut self, now: u32) {
        self.now = now;
        self.conn.set(flag::DYING | flag::FORCED_DEAD);
        self.conn.clear(flag::CONNECTED);
        self.build_cframe(OP_HARD_DISCONNECT, 0);
        self.conn.hd_left = 2;
        let d = (self.conn.rtt / 2).clamp(10, 500);
        let h = self.arm(d, 10, Work::HardDisconnect);
        self.conn.t_hd = h;
    }

    pub(crate) fn hard_disconnect_timer(&mut self, id: u32) {
        if self.conn.t_hd != id {
            return;
        }
        self.conn.t_hd = 0;
        self.build_cframe(OP_HARD_DISCONNECT, 0);
        self.conn.hd_left = self.conn.hd_left.saturating_sub(1);
        if self.conn.hd_left > 0 {
            let d = (self.conn.rtt / 2).clamp(10, 500);
            let h = self.arm(d, 10, Work::HardDisconnect);
            self.conn.t_hd = h;
        } else {
            self.teardown();
        }
    }

    /// `Dpl8r_TearDownConnection` (0x0041f690, dpnet.dll.c:26128).
    pub(crate) fn teardown(&mut self) {
        // flags &= 0xfffff9f7; flags |= 0x20
        self.conn.flags &= !(0x8 | 0x200 | 0x400);
        self.conn.set(flag::DYING);
        for slot in 0..6 {
            let mut h = match slot {
                0 => self.conn.t_retry,
                1 => self.conn.t_connect,
                2 => self.conn.t_ack,
                3 => self.conn.t_sack,
                4 => self.conn.t_pump,
                _ => self.conn.t_keepalive,
            };
            self.disarm(&mut h);
            match slot {
                0 => self.conn.t_retry = 0,
                1 => self.conn.t_connect = 0,
                2 => self.conn.t_ack = 0,
                3 => self.conn.t_sack = 0,
                4 => self.conn.t_pump = 0,
                _ => self.conn.t_keepalive = 0,
            }
        }
        self.drain_send_queues();
        // `(flags & 6) != 0 && conn[6] != 0`: a connect still waiting -> ABORTED.
        if self.conn.flags & (flag::NEW | flag::CONNECTING) != 0 && self.conn.connect_pending {
            self.finish_connect(Err(DPNERR_ABORTED), true);
        }
        self.conn.inflight.clear();
        self.conn.resend.clear();
        self.conn.ooo.clear();
        self.conn.rx_open.clear();
        self.conn.delivery.clear();
        self.release_conn();
        // `(flags & 0x50000004) == 0`: tell the core once, unless a connect was under way.
        if self.conn.flags & (flag::NOTIFIED | flag::FORCED_DEAD | flag::CONNECTING) == 0 {
            self.conn.set(flag::NOTIFIED);
            self.events
                .push_back(super::Event::ConnectionLost(DPNERR_CONNECTIONLOST));
        }
    }

    /// `FUN_0041d6b0` (0x0041d6b0): every queued message ends. A reliable or unreliable user
    /// send completes with `DPNERR_CONNECTIONLOST` (`FUN_00424f80` / `FUN_0041da70`, `0x80158160`);
    /// a pending disconnect op completes through callback `+0x30`.
    fn drain_send_queues(&mut self) {
        let mut ids: Vec<u32> = Vec::new();
        for q in &mut self.conn.queues {
            ids.extend(q.drain(..));
        }
        self.conn.queued = 0;
        if let Some(cur) = self.conn.cur_msg.take() {
            ids.insert(0, cur);
        }
        self.conn.cur_frame = 0;
        self.conn.flags &= !flag::RESEND_PENDING;
        let _ = ids;
        // Every message not yet complete, in the order it was queued: the four lists
        // (`conn+0xa4/+0xac/+0xb4` queued, `+0xbc` sent and waiting) all end here.
        let done: Vec<(super::OpId, OpKind)> =
            self.conn.msgs.values().map(|m| (m.op, m.op_kind)).collect();
        self.conn.msgs.clear();
        for (op, kind) in done {
            match kind {
                OpKind::User => self.events.push_back(super::Event::SendComplete {
                    op,
                    result: DPNERR_CONNECTIONLOST,
                }),
                OpKind::Disconnect => self.events.push_back(super::Event::DisconnectComplete {
                    op,
                    result: DPNERR_CONNECTIONLOST,
                }),
                OpKind::DisconnectReply | OpKind::KeepAlive => {}
            }
        }
        self.conn.clear(flag::KEEPALIVE_PENDING);
        self.disconnect_op = None;
    }

    /// `Dpl8r_KeepAliveTimer` (0x00426e70, dpnet.dll.c:29985).
    pub(crate) fn keepalive_timer(&mut self, id: u32) {
        if self.conn.t_keepalive != id {
            return;
        }
        let now = self.now;
        // `(conn[4] & 0xc0000000) == 0` else the timer stops.
        if self.conn.flags & (flag::FORCED_DEAD | flag::RELEASED) != 0 {
            self.conn.t_keepalive = 0;
            return;
        }
        // Pending-CONNECT records older than `rtt * 8 + 500` ms are dropped.
        let limit = now.wrapping_sub(self.conn.rtt.wrapping_mul(8).wrapping_add(500));
        while let Some(r) = self.conn.pending.front() {
            if (r.sent.wrapping_sub(limit) as i32) < 0 {
                self.conn.pending.pop_front();
            } else {
                break;
            }
        }
        let timeout = self.cfg.keepalive_timeout;
        // DX9 FUN_00426c14 (0x00426c14:33345-33352): only when nothing is queued or pending
        // (`conn[4] & 0x10400 == 0`) and the silence exceeds `ka`: a connection whose own
        // END_STREAM completed (`0x80000`) is torn down; otherwise a KeepAlive goes out unless
        // closing or hard-disconnecting (`& 0xc0020000`).
        if self.conn.flags & (flag::KEEPALIVE_PENDING | flag::DATA_QUEUED) == 0
            && timeout < now.wrapping_sub(self.conn.last_rx)
        {
            if self.conn.flags & (flag::END_STREAM_QUEUED | flag::END_STREAM_ACKED)
                == (flag::END_STREAM_QUEUED | flag::END_STREAM_ACKED)
            {
                self.conn.t_keepalive = 0;
                self.teardown();
                return;
            }
            if self.conn.flags & (flag::FORCED_DEAD | flag::RELEASED | flag::END_STREAM_QUEUED) == 0
            {
                self.send_keepalive();
            }
        }
        if !self.conn.has(flag::CONNECTED) {
            self.conn.t_keepalive = 0;
        } else {
            let period = self.keepalive_period();
            let h = self.arm(period, 1000, Work::KeepAlive);
            self.conn.t_keepalive = h;
        }
    }
}

impl Protocol {
    /// The keep-alive check period: `min(ka >> 2, 4000)`, not below the rtt, granularity 1000
    /// (DX9 0x004233e3:30143-30152, re-armed 0x00426c14:33356-33361). GunDll's 15000 gives 3750.
    pub(crate) fn keepalive_period(&self) -> u32 {
        (self.cfg.keepalive_timeout >> 2)
            .min(4000)
            .max(self.conn.rtt)
    }
}
