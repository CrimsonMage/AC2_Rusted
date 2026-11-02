//! The sender: queueing messages, framing and fragmentation, the send pump, the retransmit
//! timer, KeepAlive and END_STREAM. dpnet.dll.c, addresses per item.

use std::rc::Rc;

use crate::dplay8::hresult::*;
use crate::Hresult;

use super::frames::*;
use super::{
    flag, InFlight, OpId, OpKind, Protocol, SendFrame, SendMsg, Work, ITEM_END_STREAM,
    ITEM_KEEPALIVE, ITEM_LAST, ITEM_POLL,
};

/// The three send queues (`conn+0xa4`, `+0xac`, `+0xb4`; `FUN_0041e0b0` 0x0041e0b0 picks one by
/// the message's internal flags 4 = high, 8 = low, else normal) and which one `FUN_00425930`
/// serves first (high, normal, low: it tests lists `0x29`, `0x2b`, `0x2d` in that order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High,
    Normal,
    Low,
}

impl Priority {
    fn index(self) -> usize {
        match self {
            Priority::High => 0,
            Priority::Normal => 1,
            Priority::Low => 2,
        }
    }
}

/// How a user message goes out (the internal flags `FUN_0040a0c8` builds from the public
/// DPNSEND bits, dpnet.dll.c:7785-7794: GUARANTEED -> 1, NONSEQUENTIAL -> 2, PRIORITY_HIGH -> 4,
/// PRIORITY_LOW -> 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendMode {
    pub reliable: bool,
    pub nonsequential: bool,
    pub priority: Priority,
    /// `bCommand & 0xC0` passed through to the receiver (USER_1 0x40, USER_2 0x80).
    pub user_bits: u8,
}

impl Protocol {
    /// `Dpl8r_QueueUserMessage` (0x0041dcc0, dpnet.dll.c:25241): split `data` into frames of the
    /// SP payload size (1436), give each the command byte
    /// `DATA | RELIABLE? | SEQUENTIAL? | user bits`, the last also END_MSG, and queue the message.
    /// Needs the connection established and not closing (`(flags & 9) == 9 && !(flags & 0x20)`).
    pub fn queue_message(
        &mut self,
        now: u32,
        op: OpId,
        data: &[u8],
        mode: SendMode,
    ) -> Result<(), Hresult> {
        self.now = now;
        if self.conn.flags & 9 != 9 || self.conn.has(flag::DYING) {
            return Err(Hresult(0x8015_8330));
        }
        if data.is_empty() {
            // open: `ceil(0 / 1436)` is 0 frames and the binary then writes through an
            // uninitialised frame pointer; the core never sends an empty buffer.
            return Err(DPNERR_INVALIDPARAM);
        }
        let per = self.cfg.frame_payload() as usize;
        let mut base: u8 = if mode.reliable {
            CMD_DATA | CMD_RELIABLE
        } else {
            CMD_DATA
        };
        if !mode.nonsequential {
            base |= CMD_SEQUENTIAL;
        }
        let chunks: Vec<&[u8]> = data.chunks(per).collect();
        let n = chunks.len();
        let frames: Vec<SendFrame> = chunks
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let last = i + 1 == n;
                SendFrame {
                    cmd: base | (mode.user_bits & 0xc0) | if last { CMD_END_MSG } else { 0 },
                    item_flags: if last { ITEM_LAST } else { 0 },
                    payload: Rc::new(c.to_vec()),
                }
            })
            .collect();
        self.enqueue(
            SendMsg {
                op,
                op_kind: OpKind::User,
                reliable: mode.reliable,
                frames_unsent: n,
                frames,
                complete: false,
            },
            mode.priority,
        );
        Ok(())
    }

    /// `FUN_0041e0b0` (0x0041e0b0): put the message on its queue, mark data queued, and start the
    /// pump when the window is open and it is not already scheduled.
    fn enqueue(&mut self, msg: SendMsg, prio: Priority) {
        let id = self.conn.next_msg;
        self.conn.next_msg += 1;
        self.conn.msgs.insert(id, msg);
        self.conn.queues[prio.index()].push_back(id);
        self.conn.queued += 1;
        self.conn.set(flag::DATA_QUEUED);
        if !self.conn.has(flag::PUMP_SCHEDULED) && self.conn.has(flag::WINDOW_OPEN) {
            let now = self.now;
            self.conn.cc.t_gap = self
                .conn
                .cc
                .t_gap
                .wrapping_add(now.wrapping_sub(self.conn.cc.t_last));
            self.conn.set(flag::PUMP_SCHEDULED);
            let h = self.timers.queue_work(Work::Pump);
            self.conn.t_pump = h;
        }
    }

    /// `Dpl8r_QueueEndStream` (0x0041d560) + `FUN_0041d200` (0x0041d200): the graceful
    /// disconnect. A zero-length frame, command 0x27, item flag 8 (control `0x0A` when built),
    /// reliable, on the LOW queue (`puVar2[0xe] = 9`). The conn is flagged `0x20020` so nothing
    /// more is accepted for sending. `Ok` is `DPNSUCCESS_PENDING`.
    pub fn graceful_disconnect(&mut self, now: u32, op: OpId) -> Result<(), Hresult> {
        self.now = now;
        if self.conn.flags & 9 != 9 {
            return Err(Hresult(0x8015_8330));
        }
        if self.conn.flags & (flag::END_STREAM_QUEUED | flag::DYING) != 0 {
            // `-0x7fea7f90`
            return Err(Hresult(0x8015_8070));
        }
        self.conn.set(flag::END_STREAM_QUEUED | flag::DYING);
        self.queue_end_stream(op, OpKind::Disconnect);
        Ok(())
    }

    fn queue_end_stream(&mut self, op: OpId, kind: OpKind) {
        let frame = SendFrame {
            cmd: CMD_DATA | CMD_RELIABLE | CMD_SEQUENTIAL | CMD_END_MSG,
            item_flags: ITEM_END_STREAM,
            payload: Rc::new(Vec::new()),
        };
        self.enqueue(
            SendMsg {
                op,
                op_kind: kind,
                reliable: true,
                frames_unsent: 1,
                frames: vec![frame],
                complete: false,
            },
            Priority::Low,
        );
    }

    /// `Dpl8r_SendKeepAlive` (0x00426cf0, dpnet.dll.c:29926): one KeepAlive at a time
    /// (`flags & 0x10000`); a zero-length reliable frame, command 0x27, item flags `0x304`
    /// (`0x100` POLL, `0x200`, `0x4` last), low queue.
    pub(crate) fn send_keepalive(&mut self) {
        if self.conn.has(flag::KEEPALIVE_PENDING) {
            return;
        }
        self.conn.set(flag::KEEPALIVE_PENDING);
        let payload = if self.conn.peer_v16 {
            self.conn.sess_id.to_le_bytes().to_vec()
        } else {
            Vec::new()
        };
        let frame = SendFrame {
            cmd: CMD_DATA | CMD_RELIABLE | CMD_SEQUENTIAL | CMD_END_MSG,
            item_flags: 0x304 | ITEM_KEEPALIVE,
            payload: Rc::new(payload),
        };
        self.enqueue(
            SendMsg {
                op: 0,
                op_kind: OpKind::KeepAlive,
                reliable: true,
                frames_unsent: 1,
                frames: vec![frame],
                complete: false,
            },
            Priority::Low,
        );
    }

    /// `FUN_00426e30` (0x00426e30), called right after establishment: with nothing queued a
    /// KeepAlive goes out; otherwise the next frame to send just gets POLL.
    pub(crate) fn first_keepalive(&mut self) {
        if !self.conn.has(flag::DATA_QUEUED) {
            self.send_keepalive();
        } else if let Some(m) = self.conn.cur_msg {
            let i = self.conn.cur_frame;
            if let Some(f) = self.conn.msgs.get_mut(&m).and_then(|m| m.frames.get_mut(i)) {
                f.item_flags |= ITEM_POLL;
            }
        }
    }

    /// `FUN_0041cc10` (0x0041cc10) for a user message: cancel a message none of whose frames
    /// has been sent. `Err(0x80158110)` (`DPNERR_CANNOTCANCEL`) once it has started.
    pub fn cancel_message(&mut self, now: u32, op: OpId, result: Hresult) -> Result<(), Hresult> {
        self.now = now;
        let Some((&id, _)) = self.conn.msgs.iter().find(|(id, m)| {
            m.op == op
                && m.op_kind == OpKind::User
                && self.conn.cur_msg != Some(**id)
                && !self.conn.started.contains(id)
        }) else {
            return Err(Hresult(0x8015_8110));
        };
        for q in &mut self.conn.queues {
            q.retain(|x| *x != id);
        }
        self.conn.msgs.remove(&id);
        self.conn.queued = self.conn.queued.saturating_sub(1);
        if self.conn.queued == 0 && self.conn.cur_msg.is_none() {
            self.conn.clear(flag::DATA_QUEUED);
        }
        self.events
            .push_back(super::Event::SendComplete { op, result });
        Ok(())
    }

    /// `FUN_00425870` (0x00425870): the pump's entry (work item or timer).
    pub(crate) fn pump_entry(&mut self, id: u32) {
        if self.conn.t_pump == id {
            self.conn.t_pump = 0;
        }
        let c = &self.conn;
        if !c.has(flag::CONNECTED) || (c.flags & 0x600 != 0x600 && !c.has(flag::RESEND_PENDING)) {
            self.conn.clear(flag::PUMP_SCHEDULED);
            self.conn.set(flag::PUMP_IDLE);
            self.conn.cc.t_last = self.now;
            return;
        }
        self.send_pump();
    }

    /// `Dpl8r_BuildDFrame` (0x00425230, dpnet.dll.c:29162): the header for a new frame.
    /// Every frame takes the next sequence number; SACK masks ride along only when the last
    /// SACK-timer arming was more than 4 ms ago; send masks always do.
    fn build_dframe(&mut self, f: &SendFrame) -> DHeader {
        let now = self.now;
        let c = &mut self.conn;
        let mut cmd = f.cmd;
        let mut ctl = 0u8;
        if f.item_flags & ITEM_END_STREAM != 0 {
            // DX9 0x0042606c:32650-32653: ctrl = 8, and 0x0a when the host is not v1.6.
            ctl |= if c.peer_v16 { 0x08 } else { 0x0a };
        }
        if f.item_flags & ITEM_KEEPALIVE != 0 && c.peer_v16 {
            // v1.6 KeepAlive: bControl 0x02 with the 4-byte session id in the payload
            // (0x0042606c:32757-32761, 0x0042687f).
            ctl |= 0x02;
        }
        if f.item_flags & ITEM_POLL != 0 {
            cmd |= CMD_POLL;
        }
        let seq = c.next_send;
        c.next_send = c.next_send.wrapping_add(1);
        let mut h = DHeader {
            cmd,
            ctl,
            seq,
            nrcv: c.next_recv,
            sack1: 0,
            sack2: 0,
            send1: 0,
            send2: 0,
        };
        if c.has(flag::SACK_PENDING) && now.wrapping_sub(c.sack_stamp) > 4 {
            if c.sack1 != 0 {
                h.sack1 = c.sack1;
                h.ctl |= CTL_SACK1;
            }
            if c.sack2 != 0 {
                h.sack2 = c.sack2;
                h.ctl |= CTL_SACK2;
            }
            c.clear(flag::SACK_PENDING);
        }
        if c.has(flag::SENDMASK_PENDING) {
            if c.send1 != 0 {
                h.send1 = c.send1;
                h.ctl |= CTL_SEND1;
                c.send1 = 0;
            }
            if c.send2 != 0 {
                h.send2 = c.send2;
                h.ctl |= CTL_SEND2;
                c.send2 = 0;
            }
            c.clear(flag::SENDMASK_PENDING);
        }
        c.clear(flag::ACK_PENDING);
        let mut t = c.t_sack;
        if t != 0 && !c.has(flag::SACK_PENDING) {
            // `(conn[0x65] != 0) && !(flags & 0x4000)` (dpnet.dll.c:29221)
            self.disarm(&mut t);
            self.conn.t_sack = 0;
        }
        let mut a = self.conn.t_ack;
        if a != 0 {
            self.disarm(&mut a);
            self.conn.t_ack = 0;
        }
        h
    }

    /// `Dpl8r_RebuildRetransmit` (0x00425520, dpnet.dll.c:29244): same command byte and
    /// sequence; `bControl = (old & 0x0e) | 1` (DX9 FUN_0042638d: RETRY set, CORRELATE and END_STREAM kept), a fresh
    /// `bNRcv`, only SACK1/SACK2 attached (no send masks, no 4 ms rule).
    fn rebuild_retransmit(&mut self, i: usize) -> DHeader {
        let c = &mut self.conn;
        let f = &mut c.inflight[i];
        f.ctl = (f.ctl & 0x0e) | CTL_RETRY;
        let mut h = DHeader {
            cmd: f.cmd,
            ctl: f.ctl,
            seq: f.seq,
            nrcv: c.next_recv,
            sack1: 0,
            sack2: 0,
            send1: 0,
            send2: 0,
        };
        if c.has(flag::SACK_PENDING) {
            if c.sack1 != 0 {
                h.sack1 = c.sack1;
                h.ctl |= CTL_SACK1;
            }
            if c.sack2 != 0 {
                h.sack2 = c.sack2;
                h.ctl |= CTL_SACK2;
            }
            c.clear(flag::SACK_PENDING);
        }
        c.clear(flag::ACK_PENDING);
        let mut a = self.conn.t_ack;
        if a != 0 {
            self.disarm(&mut a);
            self.conn.t_ack = 0;
        }
        let mut t = self.conn.t_sack;
        if !self.conn.has(flag::SENDMASK_PENDING) && t != 0 {
            self.disarm(&mut t);
            self.conn.t_sack = 0;
        }
        h
    }

    /// The datagram for a DFRAME: header, then (when signing) the 8-byte signature after the
    /// masks (`FUN_0042606c` 0x0042606c:32718-32727: FAST the secret, FULL zero until the hash is
    /// taken), then the payload. A new reliable frame also seeds the rotation key
    /// (0x0042606c:32773-32785).
    fn frame_wire(&mut self, hdr: &DHeader, payload: &[u8]) -> Vec<u8> {
        let mut wire = hdr.encode();
        if self.conn.sig.is_none() {
            wire.extend_from_slice(payload);
            return wire;
        }
        let retry = hdr.ctl & CTL_RETRY != 0;
        let next_send = self.conn.next_send;
        let at = wire.len();
        wire.extend_from_slice(&[0; 8]);
        wire.extend_from_slice(payload);
        if let Some(sig) = self.conn.sig.as_mut() {
            if !retry {
                sig.note_sent(hdr.cmd, hdr.ctl, hdr.seq, payload);
            }
            let s8 = sig.sign_dframe(&wire, hdr.seq, retry, next_send);
            wire[at..at + 8].copy_from_slice(&s8);
        }
        wire
    }

    /// `Dpl8r_SendPump` (0x00425930, dpnet.dll.c:29369).
    pub(crate) fn send_pump(&mut self) {
        let now = self.now;
        if !self.conn.has(flag::CONNECTED) {
            self.conn.clear(flag::PUMP_SCHEDULED);
            return;
        }
        let (mut budget, mut delay): (i32, u32) = if self.conn.pace_ms == 0 {
            (i32::MAX, 0)
        } else {
            (
                self.conn
                    .pace_bytes
                    .wrapping_add(self.conn.pace_carry as u32) as i32,
                self.conn.pace_ms,
            )
        };
        let mut sent_bytes: i32 = 0;
        let mut sent_count: u32 = 0;
        loop {
            let f = self.conn.flags;
            let blocked = ((f & flag::WINDOW_OPEN == 0) || (f & flag::DATA_QUEUED == 0))
                && f & flag::RESEND_PENDING == 0;
            if blocked || budget <= sent_bytes || (self.conn.pace_ms > 10 && sent_count != 0) {
                break;
            }
            let (hdr, payload, len, reliable, msg_id) =
                if self.conn.flags & flag::RESEND_PENDING == 0 {
                    match self.next_new_frame() {
                        Some(x) => x,
                        None => break,
                    }
                } else {
                    // Resend: head of `conn[0x36]`, rebuilt by `Dpl8r_RebuildRetransmit`.
                    let seq = self.conn.resend.pop_front().unwrap();
                    if self.conn.resend.is_empty() {
                        self.conn.clear(flag::RESEND_PENDING);
                    }
                    let Some(i) = self.conn.inflight.iter().position(|x| x.seq == seq) else {
                        continue;
                    };
                    self.conn.inflight[i].in_resend = false;
                    let h = self.rebuild_retransmit(i);
                    let fr = &self.conn.inflight[i];
                    (h, fr.payload.clone(), fr.len, fr.reliable, fr.msg)
                };
            sent_count += 1;
            sent_bytes = sent_bytes.wrapping_add(len as i32);
            let wire = self.frame_wire(&hdr, &payload);
            // dpnet9.dll.c:33705-33710: payload bytes and a packet per frame the pump sends.
            let k = if reliable { 0 } else { 2 };
            self.conn.stats[k] = self.conn.stats[k].wrapping_add(payload.len() as u32);
            self.conn.stats[k + 1] = self.conn.stats[k + 1].wrapping_add(1);
            self.conn.cc.credit -= self.conn.cc.credit >> 8;
            self.outbox.push(wire);
            if !reliable {
                // The SP send completes at once: `FUN_00424170` case 2 counts the message's
                // frames down and completes it on the last.
                if let Some(m) = self.conn.msgs.get_mut(&msg_id) {
                    m.frames_unsent = m.frames_unsent.saturating_sub(1);
                    if m.frames_unsent == 0 {
                        let op = m.op;
                        self.conn.msgs.remove(&msg_id);
                        self.events.push_back(super::Event::SendComplete {
                            op,
                            result: Hresult(0),
                        });
                    }
                }
            }
        }
        // LAB_00425f1d
        if !self.conn.has(flag::WINDOW_OPEN) {
            self.conn.cc.blocked_pumps += 1;
        }
        if self.conn.queued == 0 && self.conn.cur_msg.is_none() {
            self.conn.clear(flag::DATA_QUEUED);
        }
        let left = budget.wrapping_sub(sent_bytes);
        if self.conn.pace_ms < 0xb {
            self.conn.pace_carry = if left < 0 { left } else { 0 };
        } else if left < 1 {
            if left < 0 {
                self.conn.pace_carry = left;
            }
        } else {
            let half = budget >> 1;
            let quarter = budget >> 2;
            let sixteenth = budget >> 4;
            let used = sent_bytes;
            let d = delay;
            if half < used {
                if quarter + half < used {
                    if used <= budget - sixteenth {
                        delay = d - (d >> 3);
                    }
                } else if half + sixteenth < used {
                    delay = d - (d >> 2);
                } else {
                    delay = d - ((d >> 2) + (d >> 3));
                }
            } else if quarter < used {
                if quarter + sixteenth < used {
                    delay = d >> 1;
                } else {
                    delay = d - ((d >> 1) + (d >> 3));
                }
            } else if sixteenth < used {
                delay = d >> 2;
            } else {
                delay = d >> 3;
            }
            self.conn.pace_carry = 0;
        }
        let _ = &mut budget;
        self.conn.pace_next = now.wrapping_add(self.conn.pace_ms);
        if self.conn.pace_ms != 0 {
            let h = self.arm(delay, 4, Work::Pump);
            self.conn.t_pump = h;
            return;
        }
        self.conn.clear(flag::PUMP_SCHEDULED);
        self.conn.set(flag::PUMP_IDLE);
        self.conn.cc.t_last = now;
    }

    /// The new-data half of the pump loop (dpnet.dll.c:29405-29552): take the next frame of the
    /// current message (starting the next queued message, NEW_MSG on its first frame), build its
    /// header, put it on the in-flight list, account the windows, arm the retransmit timer for
    /// the first frame in flight, and set POLL when a window fills or the message was the last
    /// queued.
    #[allow(clippy::type_complexity)]
    fn next_new_frame(&mut self) -> Option<(DHeader, Rc<Vec<u8>>, u32, bool, u32)> {
        let msg_id = match self.conn.cur_msg {
            Some(m) => m,
            None => {
                let q = self.conn.queues.iter_mut().find(|q| !q.is_empty())?;
                let m = q.pop_front().unwrap();
                self.conn.queued -= 1;
                self.conn.cur_msg = Some(m);
                self.conn.cur_frame = 0;
                self.conn.started.push_back(m);
                if let Some(msg) = self.conn.msgs.get_mut(&m) {
                    msg.frames[0].cmd |= CMD_NEW_MSG;
                }
                m
            }
        };
        let idx = self.conn.cur_frame;
        let (frame, reliable, nframes, op, kind) = {
            let msg = self.conn.msgs.get(&msg_id)?;
            (
                msg.frames[idx].clone(),
                msg.reliable,
                msg.frames.len(),
                msg.op,
                msg.op_kind,
            )
        };
        let mut hdr = self.build_dframe(&frame);
        let len = frame.payload.len() as u32;
        self.conn.inflight_bytes += len;
        let mut window_full = false;
        if self.conn.byte_window <= self.conn.inflight_bytes {
            self.conn.clear(flag::WINDOW_OPEN);
            self.conn.set(flag::BYTE_WINDOW_FULL);
            window_full = true;
        }
        self.conn.inflight_count += 1;
        if self.conn.frame_window <= self.conn.inflight_count {
            self.conn.clear(flag::WINDOW_OPEN);
            self.conn.set(flag::FRAME_WINDOW_FULL);
            window_full = true;
        }
        if window_full {
            hdr.cmd |= CMD_POLL;
        }
        let first_in_flight = self.conn.inflight_count == 1;
        // Advance within the message; the last frame of the last queued message gets POLL.
        if idx + 1 == nframes {
            self.conn.cur_msg = None;
            self.conn.cur_frame = 0;
            if self.conn.queued == 0 {
                hdr.cmd |= CMD_POLL;
            }
        } else {
            self.conn.cur_frame = idx + 1;
        }
        // `completes`: item flags 4 (last frame) or 8 (END_STREAM) finish the op when acked.
        let completes = if reliable && frame.item_flags & (ITEM_LAST | ITEM_END_STREAM) != 0 {
            Some((op, kind))
        } else {
            None
        };
        let mut e = InFlight {
            msg: msg_id,
            reliable,
            cmd: hdr.cmd,
            ctl: hdr.ctl,
            seq: hdr.seq,
            payload: frame.payload.clone(),
            len,
            retries: 0,
            sent: [0; 11],
            in_resend: false,
            timer_owner: false,
            completes,
        };
        e.sent[0] = self.now;
        if first_in_flight {
            e.timer_owner = true;
            let h = self.arm(self.conn.retry_base, 0, Work::Retransmit);
            self.conn.t_retry = h;
        }
        self.conn.inflight.push_back(e);
        Some((hdr, frame.payload, len, reliable, msg_id))
    }

    /// `FUN_00426160` (0x00426160, dpnet.dll.c:29572): the retransmit timer. After the 10th
    /// retransmission of the oldest frame the 11th expiry tears the connection down.
    pub(crate) fn retransmit_timer(&mut self, id: u32) {
        let now = self.now;
        if !self.conn.has(flag::CONNECTED) {
            if self.conn.t_retry == id {
                self.conn.t_retry = 0;
            }
            return;
        }
        if self.conn.t_retry != id {
            return;
        }
        self.conn.t_retry = 0;
        if self.conn.inflight_count < 1 {
            return;
        }
        let base = self.conn.retry_base;
        let head = &mut self.conn.inflight[0];
        let elapsed = now.wrapping_sub(head.sent[head.retries as usize]);
        let wait;
        if base < elapsed {
            head.retries += 1;
            if head.retries > 10 {
                self.teardown();
                return;
            }
            let k = head.retries;
            let mut w = if k == 1 {
                base
            } else if k < 4 {
                base.wrapping_mul(k)
            } else if k < 8 {
                base.wrapping_mul(1 << k)
            } else {
                5000
            };
            if w > 4999 {
                w = 5000;
                if self.conn.rtt > 5000 {
                    w = self.conn.rtt;
                }
            }
            wait = w;
            let reliable = head.reliable;
            // DX9 FUN_00427bfa:34205-34215 stamps `frame+0x4c` only in the reliable branch; an
            // unreliable frame keeps its first stamp (verify_phase4a_dx9 missed item 9).
            head.sent[k as usize] = if reliable {
                now
            } else {
                head.sent[k as usize - 1]
            };
            if !reliable {
                let len = head.len;
                let seq = head.seq;
                head.len = 0;
                self.conn.inflight_bytes = self.conn.inflight_bytes.saturating_sub(len);
                if k == 1 {
                    self.cc_loss_event(now);
                }
                let d = self.conn.next_send.wrapping_sub(seq);
                if d < 0x21 {
                    self.conn.send1 |= 1u32 << (d.wrapping_sub(1) & 0x1f);
                } else {
                    self.conn.send2 |= 1u32 << (d.wrapping_sub(0x21) & 0x1f);
                }
                self.conn.set(flag::SENDMASK_PENDING);
                if self.conn.t_sack == 0 {
                    let h = self.arm(0x28, 0, Work::SackTimer);
                    self.conn.t_sack = h;
                }
            } else {
                let seq = head.seq;
                let queued = head.in_resend;
                self.conn.clear(flag::ACK_PENDING);
                let mut a = self.conn.t_ack;
                self.disarm(&mut a);
                self.conn.t_ack = 0;
                self.cc_loss_event(now);
                if !queued {
                    self.conn.inflight[0].in_resend = true;
                    self.conn.set(flag::RESEND_PENDING);
                    self.conn.resend.push_back(seq);
                    if !self.conn.has(flag::PUMP_SCHEDULED) {
                        self.conn.set(flag::PUMP_SCHEDULED);
                        let h = self.timers.queue_work(Work::Pump);
                        self.conn.t_pump = h;
                    }
                }
            }
        } else {
            wait = base - elapsed;
        }
        let h = self.arm(wait, 0x14, Work::Retransmit);
        self.conn.t_retry = h;
    }

    /// `FUN_004239b0` (0x004239b0): finish every message at the front of the started list whose
    /// last frame was acknowledged (`FUN_00424f80`, `FUN_00424e60`).
    pub(crate) fn process_completions(&mut self) {
        while let Some(&front) = self.conn.started.front() {
            let done = self
                .conn
                .msgs
                .get(&front)
                .map(|m| m.complete)
                .unwrap_or(true);
            if !done {
                break;
            }
            self.conn.started.pop_front();
            if let Some(m) = self.conn.msgs.remove(&front) {
                self.complete_op(m.op, m.op_kind, Hresult(0));
            }
        }
        self.conn.clear(flag::COMPLETIONS_PENDING);
    }

    /// `FUN_00424f80` (0x00424f80, dpnet.dll.c:29078): an operation's frames were acknowledged.
    pub(crate) fn complete_op(&mut self, op: OpId, kind: OpKind, hr: Hresult) {
        match kind {
            OpKind::User => self
                .events
                .push_back(super::Event::SendComplete { op, result: hr }),
            OpKind::KeepAlive => {
                // `piVar1[4] &= 0xfffeffff`
                self.conn.clear(flag::KEEPALIVE_PENDING);
            }
            OpKind::Disconnect | OpKind::DisconnectReply => {
                self.conn.set(flag::END_STREAM_ACKED);
                if !self.conn.has(flag::PEER_END_STREAM) {
                    // `piVar1[6] = op`: wait for the peer's END_STREAM.
                    self.disconnect_op = Some((op, kind));
                } else {
                    let first = !self.conn.has(flag::NOTIFIED);
                    self.conn.set(flag::NOTIFIED);
                    self.teardown();
                    if kind == OpKind::Disconnect {
                        let result = if first { hr } else { DPNERR_CONNECTIONLOST };
                        self.events
                            .push_back(super::Event::DisconnectComplete { op, result });
                    } else if first {
                        self.events.push_back(super::Event::ConnectionLost(hr));
                    }
                }
            }
        }
    }

    /// `Dpl8r_OnEndStream` (0x00423b60, dpnet.dll.c:28445): the peer's END_STREAM was delivered.
    pub(crate) fn on_peer_end_stream(&mut self) {
        if !self.conn.has(flag::DYING) {
            self.events.push_back(super::Event::PeerEndStream);
            self.conn.set(flag::PEER_END_STREAM | flag::DYING);
            if !self.conn.has(flag::END_STREAM_QUEUED) {
                self.conn.set(flag::END_STREAM_QUEUED);
                self.queue_end_stream(0, OpKind::DisconnectReply);
            }
        }
        self.conn.set(flag::PEER_END_STREAM);
        if self.conn.has(flag::END_STREAM_ACKED) {
            self.conn.set(flag::ACK_PENDING);
            let op = self.disconnect_op.take();
            self.build_sack(true);
            if let Some((op, kind)) = op {
                self.complete_op(op, kind, Hresult(0));
            }
        }
    }
}
