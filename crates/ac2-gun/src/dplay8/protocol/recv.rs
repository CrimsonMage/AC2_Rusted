//! The receiver: the frame dispatcher, DFRAME acceptance, in-order and out-of-order reassembly,
//! delivery to the core. dpnet.dll.c, addresses per item.

use super::frames::*;
use super::{flag, Event, Protocol, RxFrame, RxMessage, Work};

/// The receive window: `((bSeq - nextRecv) & 0xff) < 0x3f` (`Dpl8r_HandleDFrame`
/// dpnet.dll.c:27712), so sequence offsets 0 through 62 are accepted.
pub const RECEIVE_WINDOW: u8 = 0x3f;

impl Protocol {
    /// `Dpl8r_DispatchFrame` (0x00421f50, dpnet.dll.c:27499): one datagram from the host.
    /// Bit 0 of the first byte selects DFRAME. The conn's last-receive tick (`conn[0x1b]`, what
    /// the keep-alive timer reads) is stamped first.
    pub fn on_datagram(&mut self, now: u32, data: &[u8]) {
        self.now = now;
        if data.is_empty() {
            return;
        }
        self.conn.last_rx = now;
        // FUN_00425e4d (0x00425e4d:32508-32525): a CFRAME is a datagram below 4 bytes or with bit
        // 0 of byte 0 clear; it needs 12 bytes and byte 0 of 0x80 or 0x88, else it is a bad
        // CFRAME. A DFRAME is dropped once the conn is released or hard-disconnecting
        // (`conn[4] & 0xc0000000`).
        if data.len() < 4 || data[0] & 1 == 0 {
            if data.len() < 12 || (data[0] != 0x80 && data[0] != 0x88) {
                self.bad_cframe();
            } else if self.conn.flags & flag::RELEASED == 0 {
                self.handle_cframe(data);
            }
        } else if self.conn.flags & (flag::RELEASED | flag::FORCED_DEAD) == 0 {
            self.handle_dframe(data);
        }
    }

    /// `Dpl8r_HandleCFrame` (0x004220e0, dpnet.dll.c:27583).
    fn handle_cframe(&mut self, data: &[u8]) {
        let Some(cmd) = parse_command(data) else {
            return;
        };
        // FUN_004257da (0x004257da:32155-32172): a HARD_DISCONNECT is handled before the
        // hard-disconnecting test, so it still ends our own hard disconnect at once; every
        // other opcode is a bad CFRAME while `conn[4] & 0x40000000`.
        if self.conn.has(flag::FORCED_DEAD) && !matches!(cmd, Command::HardDisconnect(_)) {
            self.bad_cframe();
            return;
        }
        match cmd {
            Command::Connect(f) => self.on_connect_cframe(&f),
            Command::Connected(f) => self.on_connected(&f),
            Command::ConnectedSigned(f) => self.on_connected_signed(&f),
            Command::HardDisconnect(f) => {
                // DX9 0x004257da:32155-32172: only an established connection accepts it, with
                // length >= 0x10 (>= 0x18 and a valid signature at +0x10 when signing; the
                // signature's sequence byte is `bRspId`); anything else is a bad CFRAME.
                // (This port clears CONNECTED when a hard disconnect starts; DX9's bit 8 stays
                // set, so FORCED_DEAD counts as established here.)
                let ok = (self.conn.has(flag::CONNECTED) || self.conn.has(flag::FORCED_DEAD))
                    && match &self.conn.sig {
                        None => data.len() >= 0x10,
                        Some(sig) => {
                            data.len() >= 0x18
                                && sig.verify(self.conn.next_recv, data, data[3], 0x10)
                        }
                    };
                if ok {
                    self.on_hard_disconnect(&f);
                } else {
                    self.bad_cframe();
                }
            }
            Command::Sack(s) => {
                if !self.conn.has(flag::CONNECTED) {
                    // not established: a bad CFRAME (ignored)
                    self.bad_cframe();
                    return;
                }
                // DX9 0x004257da:32209-32262: 12 bytes plus 4 per mask (flags 0x02, 0x04, 0x08,
                // 0x10), plus 8 when signing; a shorter frame, or a bad signature (at the end
                // of the header, `bNSeq` as the sequence byte), is dropped silently.
                let n = (data[2] & 0x1e).count_ones() as usize;
                let mut need = 12 + 4 * n;
                if self.conn.sig.is_some() {
                    need += 8;
                }
                if data.len() < need {
                    return;
                }
                if let Some(sig) = &self.conn.sig {
                    if !sig.verify(self.conn.next_recv, data, data[4], need - 8) {
                        return;
                    }
                }
                self.process_ack(s.nrcv, s.sack1, s.sack2);
                if s.send1 != 0 || s.send2 != 0 {
                    self.process_send_mask(s.nseq, s.send1, s.send2);
                    if !self.conn.delivery.is_empty() && !self.conn.has(flag::DELIVERING) {
                        self.conn.set(flag::DELIVERING);
                        self.deliver();
                    }
                }
                if self.conn.has(flag::COMPLETIONS_PENDING) {
                    self.process_completions();
                }
            }
            Command::Unknown(_) => self.bad_cframe(),
        }
    }

    /// `Dpl8r_OnConnect` (0x0041ef80, dpnet.dll.c:25921) as a connector meets it: with the
    /// connect op pending nothing happens; afterwards a CONNECT with our own session id is
    /// answered with CONNECTED (`rsp` = its `bMsgID`), any other session id tears the
    /// connection down. The listener-side acceptance is host code and not ported.
    fn on_connect_cframe(&mut self, f: &CFrame) {
        if self.conn.connect_pending {
            return;
        }
        if self.conn.sess_id == f.sess_id {
            self.build_cframe(OP_CONNECTED, f.msg_id);
        } else if !self.conn.has(flag::DYING) {
            self.teardown();
        }
    }

    /// `Dpl8r_HandleDFrame` (0x00422340, dpnet.dll.c:27674).
    fn handle_dframe(&mut self, data: &[u8]) {
        let now = self.now;
        let signed = self.conn.sig.is_some();
        let Some(mut d) = parse_dframe_ex(data, signed) else {
            return;
        };
        // Before the handshake finishes a DFRAME is a bad CFRAME (0x004259cf:32297), ignored.
        if !self.conn.has(flag::CONNECTED) {
            self.bad_cframe();
            return;
        }
        // DX9 0x004259cf:32314-32323: with signing the signature (after the masks, `bSeq` as the
        // sequence byte) must verify, else the frame is dropped.
        if let (Some(sig), Some(at)) = (&self.conn.sig, d.sig_at) {
            if !sig.verify(self.conn.next_recv, data, d.seq, at) {
                return;
            }
        }
        if self.conn.peer_v16 && d.ctl & CTL_CORRELATE != 0 {
            // DX9 0x004259cf:32355-32361: a keep-alive of a v1.6 peer carries our session id
            // (4 bytes after the header and signature) and no coalescing bit; anything else is
            // dropped, without teardown.
            if d.payload.len() < 4
                || d.ctl & CTL_COALESCE != 0
                || d.payload[..4] != self.conn.sess_id.to_le_bytes()
            {
                return;
            }
            d.payload.drain(..4);
        } else if d.ctl & CTL_COALESCE != 0 {
            match decode_coalesced(d) {
                Some(x) => d = x,
                None => return,
            }
        }
        if d.seq.wrapping_sub(self.conn.next_recv) >= RECEIVE_WINDOW {
            self.out_of_window(d.cmd);
            return;
        }
        // Acknowledgements first: SACK1/SACK2 with the cumulative `bNRcv`, then the send masks.
        self.process_ack(d.nrcv, d.sack1, d.sack2);
        if d.ctl & (CTL_SEND1 | CTL_SEND2) != 0 {
            self.process_send_mask(d.seq, d.send1, d.send2);
        }
        if let Some(sig) = self.conn.sig.as_mut() {
            // DX9 0x004259cf:32407-32412: the first reliable payload of the cycle seeds the key.
            let first_reliable = d
                .subs
                .as_ref()
                .and_then(|v| v.iter().find(|x| x.flags & 2 != 0))
                .map(|x| x.data.as_slice())
                .unwrap_or(&d.payload);
            sig.note_received(d.cmd, d.ctl, d.seq, first_reliable);
        }
        self.conn.set(flag::ACK_PENDING);
        let frame = RxFrame {
            seq: d.seq,
            cmd: d.cmd,
            ctl: d.ctl,
            payload: d.payload,
            time: now,
            stray: false,
            delivered: false,
            subs: d.subs,
        };
        if d.seq == self.conn.next_recv {
            self.append_in_order(frame);
            if d.ctl & CTL_END_STREAM != 0 {
                // DX9 0x004259cf:32429-32431: an in-order END_STREAM is acked at once. (bControl
                // 0x02 is no longer "ACK now" as in 8.0, 0x004259cf:32326-32362.)
                self.build_sack(false);
            } else if d.cmd & CMD_POLL == 0 {
                // 100 ms delayed ACK, one timer per connection (0x004259cf:32442-32472).
                if self.conn.t_ack == 0 {
                    let h = self.arm(100, 0, Work::DelayedAck);
                    self.conn.t_ack = h;
                }
            } else {
                // POLL: 4 ms (0x004259cf:32433-32440).
                let mut a = self.conn.t_ack;
                self.disarm(&mut a);
                self.conn.set(flag::ACK_PENDING);
                let h = self.arm(4, 4, Work::DelayedAck);
                self.conn.t_ack = h;
            }
        } else {
            let diff = d.seq.wrapping_sub(self.conn.next_recv).wrapping_sub(1);
            if self.mask_has(diff) {
                // Already held: dropped silently (dpnet.dll.c:27787).
                return;
            }
            self.add_out_of_order(frame, diff);
            if d.cmd & CMD_POLL != 0 {
                let mut a = self.conn.t_ack;
                self.disarm(&mut a);
                self.conn.set(flag::ACK_PENDING);
                let h = self.arm(5, 5, Work::DelayedAck);
                self.conn.t_ack = h;
            }
        }
        if !self.conn.delivery.is_empty() && !self.conn.has(flag::DELIVERING) {
            self.conn.set(flag::DELIVERING);
            self.deliver();
        }
        if self.conn.has(flag::COMPLETIONS_PENDING) {
            self.process_completions();
        }
    }

    /// `FUN_00423ef0` (0x00423ef0): a frame outside the window. POLL -> an immediate SACK, else
    /// a 20 ms delayed ACK.
    fn out_of_window(&mut self, cmd: u8) {
        self.conn.set(flag::ACK_PENDING);
        if cmd & CMD_POLL == 0 {
            if self.conn.t_ack == 0 {
                let h = self.arm(0x14, 0, Work::DelayedAck);
                self.conn.t_ack = h;
            }
        } else {
            self.build_sack(true);
        }
    }

    /// Bit `diff` of the 64-bit out-of-order mask (`conn[0x4f]` low, `conn[0x50]` high).
    pub(crate) fn mask_has(&self, diff: u8) -> bool {
        if diff < 0x20 {
            self.conn.sack1 & (1u32 << diff) != 0
        } else if diff < 0x40 {
            self.conn.sack2 & (1u32 << (diff - 0x20)) != 0
        } else {
            false
        }
    }

    /// `FUN_00422840` (0x00422840, dpnet.dll.c:27831): the frame is the next one expected. It is
    /// appended to the message being assembled (a first frame without NEW_MSG is a stray and
    /// ends its own message), `nextRecv` advances, and frames held out of order that are now
    /// contiguous are consumed the same way.
    pub(crate) fn append_in_order(&mut self, first: RxFrame) {
        let mut f = first;
        loop {
            self.conn.rx_tick = f.time;
            self.conn.rx_retry = f.ctl & 1;
            if self.conn.rx_open.is_empty() && f.cmd & CMD_NEW_MSG == 0 {
                f.stray = true;
                f.cmd |= CMD_END_MSG;
            } else if !self.conn.rx_open.is_empty() && f.cmd & CMD_NEW_MSG != 0 {
                // FUN_00424d1c:31560-31567: a NEW_MSG frame while a message is open is marked
                // stray and ends it, so the whole message is dropped.
                f.stray = true;
                f.cmd |= CMD_END_MSG;
            }
            let coalesced = f.subs.is_some();
            let end = f.cmd & CMD_END_MSG != 0 && !coalesced;
            if let Some(subs) = f.subs.take() {
                // A coalesced DFRAME: each sub-message is a whole message of its own, in order,
                // with its own USER_1/USER_2 bits.
                // open: RELIABLE/SEQUENTIAL of the sub-headers are not used to reorder.
                for sub in subs {
                    let frame = RxFrame {
                        seq: f.seq,
                        // FUN_0042521b:31887-31915 delivers each sub with `0x100 | 0x01 | flags &
                        // 0xc0` and counts it reliable by the sub-header bit 0x02 (the 0x100/0x01
                        // bits have no reader here: `Event::Receive` carries `flags & 0xc0`).
                        cmd: CMD_NEW_MSG
                            | CMD_END_MSG
                            | (sub.flags & 0xc0)
                            | (sub.flags & CMD_RELIABLE),
                        ctl: 0,
                        payload: sub.data,
                        time: f.time,
                        stray: false,
                        delivered: false,
                        subs: None,
                    };
                    self.conn.delivery.push_back(RxMessage {
                        frames: vec![frame],
                    });
                }
            } else {
                self.conn.rx_open.push(f);
            }
            if end {
                let frames = std::mem::take(&mut self.conn.rx_open);
                if frames.iter().all(|x| !x.stray) {
                    self.conn.delivery.push_back(RxMessage { frames });
                }
            }
            // Advance `nextRecv`, shifting the 64-bit out-of-order mask right by one.
            let next;
            loop {
                let lo = self.conn.sack1 & 1;
                self.conn.next_recv = self.conn.next_recv.wrapping_add(1);
                if let Some(sig) = self.conn.sig.as_mut() {
                    sig.on_next_recv(self.conn.next_recv);
                }
                self.conn.sack1 >>= 1;
                if self.conn.sack2 & 1 != 0 {
                    self.conn.sack1 |= 0x8000_0000;
                }
                self.conn.sack2 >>= 1;
                if lo == 0 {
                    next = None;
                    break;
                }
                let held = self.conn.ooo.remove(0);
                if !held.delivered {
                    next = Some(held);
                    break;
                }
                self.conn.rx_tick = held.time;
                self.conn.rx_retry = held.ctl & 1;
            }
            match next {
                Some(h) => f = h,
                None => break,
            }
        }
        if self.conn.sack1 == 0 && self.conn.sack2 == 0 {
            self.conn.clear(flag::SACK_PENDING);
            if !self.conn.has(flag::SENDMASK_PENDING) && self.conn.t_sack != 0 {
                let mut t = self.conn.t_sack;
                self.disarm(&mut t);
                self.conn.t_sack = 0;
            }
        }
    }

    /// `FUN_00422b90` (0x00422b90, dpnet.dll.c:27913): a frame ahead of `nextRecv`, `diff` frames
    /// past the first missing one. It is set in the mask and kept in the sorted list; when it is
    /// the highest and opens a new gap the 20 ms SACK timer starts; a non-sequential frame
    /// completing a whole NEW_MSG..END_MSG run is delivered at once.
    pub(crate) fn add_out_of_order(&mut self, frame: RxFrame, diff: u8) {
        let before_lo = self.conn.sack1;
        let before_hi = self.conn.sack2;
        let lo_bit;
        let hi_bit;
        let mut new_gap = false;
        if diff < 0x20 {
            lo_bit = 1u32 << diff;
            hi_bit = 0;
            self.conn.sack1 |= lo_bit;
            new_gap = before_lo & (lo_bit >> 1) == 0;
        } else {
            hi_bit = 1u32 << (diff - 0x20);
            lo_bit = 0;
            self.conn.sack2 |= hi_bit;
            if diff - 0x20 == 0 {
                if before_lo & 0x8000_0000 == 0 {
                    new_gap = true;
                }
            } else {
                new_gap = before_hi & (hi_bit >> 1) == 0;
            }
        }
        let nr = self.conn.next_recv;
        let dist = |f: &RxFrame| f.seq.wrapping_sub(nr);
        let nonseq = frame.cmd & CMD_SEQUENTIAL == 0;
        let fseq = frame.seq;
        let highest = before_hi < hi_bit || (before_hi == 0 && before_lo < lo_bit);
        // Sorted by distance from `nextRecv`; the highest goes last.
        let pos = if highest {
            self.conn.ooo.len()
        } else {
            self.conn
                .ooo
                .iter()
                .position(|x| dist(x) > dist(&frame))
                .unwrap_or(self.conn.ooo.len())
        };
        self.conn.ooo.insert(pos, frame);
        if highest && ((before_lo == 0 && before_hi == 0) || new_gap) {
            self.conn.set(flag::SACK_PENDING);
            if self.conn.t_sack == 0 {
                let h = self.arm(0x14, 5, Work::SackTimer);
                self.conn.t_sack = h;
                self.conn.sack_stamp = self.now;
            }
        }
        if nonseq {
            self.deliver_nonsequential(fseq);
        }
    }

    /// The tail of `FUN_00422b90` (dpnet.dll.c:27996-28041): look back from the new frame over
    /// consecutive sequence numbers to a NEW_MSG frame, forward to an END_MSG frame; a complete
    /// run is queued for delivery (or discarded when a frame in it is a stray) and its frames
    /// are marked delivered so the in-order walk skips them later.
    fn deliver_nonsequential(&mut self, fseq: u8) {
        let Some(mut i) = self.conn.ooo.iter().position(|x| x.seq == fseq) else {
            return;
        };
        let mut expect = fseq;
        let start;
        loop {
            let f = &self.conn.ooo[i];
            if f.cmd & CMD_NEW_MSG != 0 || f.seq != expect {
                start = i;
                break;
            }
            expect = expect.wrapping_sub(1);
            if i == 0 {
                start = 0;
                break;
            }
            i -= 1;
        }
        let s = &self.conn.ooo[start];
        if s.cmd & CMD_NEW_MSG == 0 || s.seq != expect {
            return;
        }
        let mut end = self.conn.ooo.iter().position(|x| x.seq == fseq).unwrap();
        let mut exp = fseq;
        while self.conn.ooo[end].cmd & CMD_END_MSG == 0
            && self.conn.ooo[end].seq == exp
            && end + 1 < self.conn.ooo.len()
        {
            end += 1;
            exp = exp.wrapping_add(1);
        }
        if self.conn.ooo[end].cmd & CMD_END_MSG == 0 || self.conn.ooo[end].seq != exp {
            return;
        }
        let mut frames = Vec::new();
        for f in &mut self.conn.ooo[start..=end] {
            f.delivered = true;
            frames.push(f.clone());
        }
        if frames.iter().all(|f| !f.stray) {
            self.conn.delivery.push_back(RxMessage { frames });
        }
    }

    /// `Dpl8r_DeliverReceivedFrame` (0x00423110, dpnet.dll.c:28079). A one-frame message with no
    /// payload is a KeepAlive or an END_STREAM (`bControl & 8` -> `Dpl8r_OnEndStream`); anything
    /// else goes to the core with `bCommand & 0xC0` of its last frame.
    pub(crate) fn deliver(&mut self) {
        while let Some(m) = self.conn.delivery.pop_front() {
            if m.frames.len() == 1 && m.frames[0].payload.is_empty() {
                if m.frames[0].ctl & CTL_END_STREAM != 0 {
                    self.on_peer_end_stream();
                }
                continue;
            }
            let flags = m.frames.last().map(|f| f.cmd & 0xc0).unwrap_or(0);
            // dpnet9.dll.c:31884-31923: messages and bytes received, guaranteed or not.
            let n: usize = m.frames.iter().map(|f| f.payload.len()).sum();
            let k = if m.frames[0].cmd & CMD_RELIABLE != 0 {
                14
            } else {
                16
            };
            self.conn.stats[k] = self.conn.stats[k].wrapping_add(n as u32);
            self.conn.stats[k + 1] = self.conn.stats[k + 1].wrapping_add(1);
            self.conn.stats[18] = self.conn.stats[18].wrapping_add(1);
            let mut data = Vec::new();
            for f in &m.frames {
                data.extend_from_slice(&f.payload);
            }
            self.events.push_back(Event::Receive { data, flags });
        }
        self.conn.clear(flag::DELIVERING);
    }
}
