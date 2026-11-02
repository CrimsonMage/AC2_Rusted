//! Acknowledgements: SACK building, the delayed-ACK timers, cumulative and selective ACK
//! processing, send masks. dpnet.dll.c, addresses per item.

use super::frames::*;
use super::{flag, Protocol, RxFrame, Work};

impl Protocol {
    /// `Dpl8r_BuildSack` (0x00426880, dpnet.dll.c:29790). Both ack timers are cancelled first.
    /// The SACK carries the pending SACK masks (flag 0x4000) and send masks (flag 0x2000); with
    /// `now_flush` it is sent at once together with anything else on the SP's CFRAME list
    /// (`FUN_004256e0`), else the flush work item is queued.
    pub(crate) fn build_sack(&mut self, now_flush: bool) {
        let mut a = self.conn.t_ack;
        self.disarm(&mut a);
        self.conn.t_ack = 0;
        let mut s = self.conn.t_sack;
        self.disarm(&mut s);
        self.conn.t_sack = 0;
        let mut f = SackFrame {
            retry: self.conn.rx_retry,
            nseq: self.conn.next_send,
            nrcv: self.conn.next_recv,
            tick: self.conn.rx_tick,
            ..SackFrame::default()
        };
        if self.conn.has(flag::SACK_PENDING) {
            f.sack1 = self.conn.sack1;
            f.sack2 = self.conn.sack2;
            self.conn.clear(flag::SACK_PENDING);
        }
        if self.conn.has(flag::SENDMASK_PENDING) {
            f.send1 = self.conn.send1;
            f.send2 = self.conn.send2;
            self.conn.send1 = 0;
            self.conn.send2 = 0;
            self.conn.clear(flag::SENDMASK_PENDING);
        }
        self.conn.clear(flag::ACK_PENDING);
        let mut wire = f.encode();
        if let Some(sig) = &self.conn.sig {
            // DX9 0x00427839:33990-34017: 8 more bytes; FAST the secret, FULL the hash of the
            // frame with the field zero followed by the secret.
            wire.extend_from_slice(&[0; 8]);
            let s8 = sig.sign_control(&wire);
            let at = wire.len() - 8;
            wire[at..].copy_from_slice(&s8);
        }
        self.cframes.push_back(wire);
        if now_flush {
            self.flush_cframes_now();
        } else {
            self.schedule_cframe_flush();
        }
    }

    /// `FUN_00426be0` (0x00426be0): a delayed-ACK or SACK timer fired. If anything is still
    /// owed (flags `0xe000`) and the connection is up, a SACK goes out now.
    pub(crate) fn ack_timer(&mut self, id: u32, _delayed: bool) {
        if self.conn.t_ack == id {
            self.conn.t_ack = 0;
        } else if self.conn.t_sack == id {
            self.conn.t_sack = 0;
        } else {
            return;
        }
        if self.conn.has(flag::CONNECTED) && self.conn.flags & 0xe000 != 0 {
            self.build_sack(true);
        }
    }

    /// `Dpl8r_ProcessAck` (0x00424640, dpnet.dll.c:28867): `nrcv` is the peer's cumulative
    /// acknowledgement (the next sequence it wants), `sack1/2` its out-of-order mask (bit i =
    /// frame `nrcv + 1 + i` held).
    pub(crate) fn process_ack(&mut self, nrcv: u8, sack1: u32, sack2: u32) {
        if !self.conn.has(flag::CONNECTED) {
            return;
        }
        let now = self.now;
        // Cumulative part.
        while let Some(head) = self.conn.inflight.front() {
            // DX9 FUN_00427eeb (0x00427eeb:34296-34301, asm 0x427f24-0x427f49): a sample only
            // with no SACK mask, for `seq == (int)nrcv - 1` (so none at nrcv == 0) and
            // `(byte)(seq - conn[0x138]) < 0x41`; there is no `retries == 0` condition, a
            // retransmitted frame samples with the `2 * rtt - sample` correction.
            if sack1 == 0
                && sack2 == 0
                && i32::from(head.seq) == i32::from(nrcv) - 1
                && head.seq.wrapping_sub(self.conn.rtt_seq) < 0x41
            {
                let (seq, retried) = (head.seq, head.retries != 0);
                let mut rtt = now.wrapping_sub(head.sent[head.retries as usize]);
                if (rtt as i32) < 1 {
                    rtt = 1;
                }
                self.conn.rtt_seq = seq;
                self.update_rtt(rtt, retried);
            }
            let Some(head) = self.conn.inflight.front() else {
                break;
            };
            // `(byte)conn[0x3b] <= (byte)(nrcv - (seq + 1))`
            if (self.conn.inflight_count as u8) <= nrcv.wrapping_sub(head.seq.wrapping_add(1)) {
                break;
            }
            let f = self.conn.inflight.pop_front().unwrap();
            if f.timer_owner {
                let mut h = self.conn.t_retry;
                self.disarm(&mut h);
                self.conn.t_retry = 0;
            }
            self.conn.inflight_count = self.conn.inflight_count.saturating_sub(1);
            self.conn.inflight_bytes = self.conn.inflight_bytes.saturating_sub(f.len);
            if f.len != 0 {
                self.conn.cc.acked_frames += 1;
                self.conn.cc.acked_bytes += f.len;
            }
            if f.reliable {
                if let Some((op, _)) = f.completes {
                    let _ = op;
                    if let Some(m) = self.conn.msgs.get_mut(&f.msg) {
                        m.complete = true;
                    }
                    self.conn.set(flag::COMPLETIONS_PENDING);
                }
            }
        }
        // Selective part (DX9 0x00427eeb:34368-34378): with SACK masks present, more than one
        // frame in flight, `bNRcv` equal to the oldest frame's sequence number and that frame
        // never retransmitted, the retry timer is replaced by one of 10 ms (granularity 5) and the
        // frame is marked as the timer's owner. There is no immediate SACK-driven resend as in 8.0.
        // `conn[5] & 4` (0x004280eb) has no setter anywhere in the protocol code (every OR into
        // `conn+0x14` is 0x8..0x400), so the test is always "clear" and is not modelled.
        if (sack1 | sack2) != 0 && self.conn.inflight_count > 1 {
            let head = &self.conn.inflight[0];
            if head.seq == nrcv {
                // 34380-34395: walk the in-flight list shifting the 64-bit mask right; at the
                // step where the mask equals 1 the node reached is sampled (at most one sample
                // per ACK), guarded by `(byte)(seq - conn[0x138]) < 0x41`.
                // Retail-literal (verify_phase4a_dx9): this samples the frame just below the
                // highest SACKed one (bit i is frame nrcv + 1 + i).
                let m = u64::from(sack1) | (u64::from(sack2) << 32);
                let k = 63 - m.leading_zeros() as usize;
                if let Some(n) = self.conn.inflight.get(k) {
                    if n.seq.wrapping_sub(self.conn.rtt_seq) < 0x41 {
                        let (seq, retried) = (n.seq, n.retries != 0);
                        let mut rtt = now.wrapping_sub(n.sent[n.retries as usize]);
                        if (rtt as i32) < 1 {
                            rtt = 1;
                        }
                        self.conn.rtt_seq = seq;
                        self.update_rtt(rtt, retried);
                    }
                }
            }
            let head = &self.conn.inflight[0];
            if head.seq == nrcv && head.retries == 0 {
                let mut r = self.conn.t_retry;
                self.disarm(&mut r);
                let h = self.arm(10, 5, Work::Retransmit);
                self.conn.t_retry = h;
                self.conn.inflight[0].timer_owner = true;
            }
        }
        // The retry timer follows the oldest frame still in flight.
        if self.conn.inflight_count > 0 && self.conn.t_retry == 0 {
            let head = &self.conn.inflight[0];
            let waited = now.wrapping_sub(head.sent[head.retries as usize]);
            let wait = self.conn.retry_base.saturating_sub(waited);
            self.conn.inflight[0].timer_owner = true;
            let h = self.arm(wait, 0, Work::Retransmit);
            self.conn.t_retry = h;
        }
        if self.conn.inflight_count < self.conn.frame_window
            && self.conn.inflight_bytes < self.conn.byte_window
        {
            self.conn.set(flag::WINDOW_OPEN);
            if self.conn.has(flag::DATA_QUEUED) && !self.conn.has(flag::PUMP_SCHEDULED) {
                self.conn.set(flag::PUMP_SCHEDULED);
                let h = self.timers.queue_work(Work::Pump);
                self.conn.t_pump = h;
            }
        }
    }

    /// `Dpl8r_ProcessSendMask` (0x00423600, dpnet.dll.c:28241): the peer says which of the
    /// frames before `nseq` it will never send again (unreliable frames it gave up on). Each
    /// named frame is filled with an empty placeholder so `nextRecv` can move past it.
    pub(crate) fn process_send_mask(&mut self, nseq: u8, mut m1: u32, mut m2: u32) {
        let mut filled = false;
        'outer: loop {
            if nseq == self.conn.next_recv {
                break;
            }
            let mut diff = i32::from(nseq.wrapping_sub(self.conn.next_recv)) - 1;
            let mut seq = self.conn.next_recv;
            if diff > 0x3f {
                break;
            }
            while diff > 0x1f && m2 != 0 {
                let bit = 1u32 << ((diff - 0x20) & 0x1f);
                if bit & m2 != 0 {
                    filled |= self.fill_hole(seq);
                    m2 &= !bit;
                    continue 'outer;
                }
                seq = seq.wrapping_add(1);
                diff -= 1;
            }
            if diff > 0x1f {
                seq = seq.wrapping_add((diff - 0x1f) as u8);
                diff = 0x1f;
            }
            loop {
                if diff < 0 || m1 == 0 {
                    break 'outer;
                }
                let bit = 1u32 << (diff & 0x1f);
                if m1 & bit != 0 {
                    filled |= self.fill_hole(seq);
                    m1 &= !bit;
                    continue 'outer;
                }
                seq = seq.wrapping_add(1);
                diff -= 1;
            }
        }
        // DX9 0x004256d5:32132-32137 arms the 100 ms delayed ACK unconditionally.
        let _ = filled;
        if self.conn.t_ack == 0 {
            self.conn.set(flag::ACK_PENDING);
            let h = self.arm(100, 0, Work::DelayedAck);
            self.conn.t_ack = h;
        }
    }

    /// `FUN_00423830` (0x00423830) with `FUN_004237b0` (0x004237b0): a placeholder frame
    /// (command 0x30 = NEW_MSG | END_MSG, empty, stray flag) for sequence `seq`. The binary
    /// 8.0 computed `seq - nextRecv - 1` in an `int`; DX9 does it in a byte (FUN_0042564a asm
    /// 0x42567d-0x425683: `sub dl,cl; dec dl; movzx`), as here. No deviation remains.
    fn fill_hole(&mut self, seq: u8) -> bool {
        let f = RxFrame {
            seq,
            cmd: 0x30,
            ctl: 0,
            payload: Vec::new(),
            time: self.now,
            stray: true,
            delivered: false,
            subs: None,
        };
        if self.conn.next_recv == seq {
            self.append_in_order(f);
        } else {
            let diff = seq.wrapping_sub(self.conn.next_recv).wrapping_sub(1);
            if self.mask_has(diff) {
                return false;
            }
            self.add_out_of_order(f, diff);
        }
        true
    }
}
