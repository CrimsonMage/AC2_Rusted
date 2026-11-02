//! The adaptive window and pacing state of dpnet's sender, ported field for field:
//! * `FUN_00420ae0` (0x00420ae0) establishes the connection and seeds everything
//!   ([`Protocol::establish`]);
//! * `Dpl8r_UpdateRtt` (0x00420f80) takes an rtt sample;
//! * `FUN_00421340` (0x00421340) is the periodic evaluation the sample countdown triggers;
//! * `FUN_00421160` (0x00421160) grows the window or backs the pacing off;
//! * `FUN_00421740` / `FUN_004217b0` (0x00421740, 0x004217b0) react to a loss.
//!
//! Field names are descriptive; each carries its retail `conn` offset. The binary has no names
//! for any of them (stripped), so the meanings are from how the code uses them.

use super::{flag, Protocol, Work};

#[derive(Debug, Default, Clone)]
pub(crate) struct CongestionState {
    /// `[0x17]` +0x5c: loss events seen.
    pub loss_events: u32,
    /// `[0x18]` +0x60: tick of the last loss reaction.
    pub loss_time: u32,
    /// `[0x19]` +0x64: loss score (+10 per reaction, -1 per good sample).
    pub loss_score: u32,
    /// `[0x1a]` +0x68: samples until the next evaluation.
    pub countdown: u32,
    /// `[0x1c]` +0x70: pump runs that found the window closed.
    pub blocked_pumps: u32,
    /// `[0x1d]` +0x74, `[0x1e]` +0x78: accumulated bytes (scaled) and time.
    pub acc_bytes: u32,
    pub acc_time: u32,
    /// `[0x1f]` +0x7c, `[0x20]` +0x80: current and best rate.
    pub rate: u32,
    pub rate_max: u32,
    /// `[0x21]` +0x84: last time stamp; `[0x22]` +0x88: idle time accumulated.
    pub t_last: u32,
    pub t_gap: u32,
    /// `[0x23]` +0x8c: the previous rate.
    pub rate_prev: u32,
    /// `[0x24]` +0x90: acked-bytes counter at the last sample; `[0x25]` +0x94: its time.
    pub acked_mark: u32,
    pub t_sample: u32,
    /// `[0x26]` +0x98: probe wait; `[0x27]` +0x9c: the window key it was set for.
    pub probe_wait: u32,
    pub probe_key: u32,
    /// +0x108 +0x10c +0x110 +0x114: the settings before the last probe.
    pub save_fw: u32,
    pub save_steps: u32,
    pub save_factor: u32,
    pub save_ms: u32,
    /// +0x11c +0x120 +0x124 +0x128: the settings to return to after a loss back-off.
    pub loss_fw: u32,
    pub loss_steps: u32,
    pub loss_factor: u32,
    pub loss_ms: u32,
    /// +0x12c: time of the last window change.
    pub t_cc: u32,
    /// +0x208: credit that each loss raises by 0x1000000 and each paced frame decays by 1/256.
    pub credit: u32,
    /// `[0x6d]` +0x1b4, `[0x6e]` +0x1b8: frames and bytes acknowledged.
    pub acked_frames: u32,
    pub acked_bytes: u32,
    /// `[0xd]` +0x34: the DFRAME payload size (`spd[7]`).
    pub frame_payload: u32,
}

impl Protocol {
    /// `FUN_00420ae0` (0x00420ae0): the first rtt sample (`now - sentTime` of the matching
    /// CONNECT) establishes the connection's timing and seeds the window state.
    pub(crate) fn establish(&mut self, rtt: u32) {
        let now = self.now;
        // DX9 0x004233e3:30125-30141 / 0x00421c4e:28778-28786: the first sample is clamped to
        // [1, 2500] (`pd+0x28 >> 1`).
        let rtt = rtt.clamp(1, 2500);
        let max = self.cfg.sp_max_datagram;
        let c = &mut self.conn;
        c.rtt = rtt;
        c.srtt = rtt << 16;
        c.cc.loss_events = 0;
        c.cc.loss_score = 0;
        c.pace_ms = 0;
        c.pace_factor = 1;
        c.pace_bytes = max;
        c.cc.acked_frames = 0;
        c.cc.acked_bytes = 0;
        c.cc.save_factor = 1;
        c.cc.save_ms = 0;
        c.cc.save_fw = 1;
        c.cc.save_steps = 1;
        c.pace_carry = 0;
        c.cc.t_cc = now;
        c.cc.blocked_pumps = 0;
        c.cc.t_last = now;
        c.cc.t_gap = 0;
        c.cc.t_sample = now;
        c.cc.acked_mark = 0;
        c.cc.acc_bytes = 0;
        c.cc.acc_time = 0;
        c.cc.rate = 0;
        c.cc.rate_max = 0;
        c.cc.rate_prev = 0;
        c.sack1 = 0;
        c.sack2 = 0;
        c.sack_stamp = 0;
        c.send1 = 0;
        c.send2 = 0;
        c.cc.probe_key = 0;
        c.set(flag::FAST_ACK);
        c.cc.countdown = 4;
        // RTO = 2.5 * rtt + 100 (DX9 0x004233e3:30125; 8.0 had + 200)
        c.retry_base = (rtt + (rtt >> 2)) * 2 + 100;
        if c.retry_base < c.pace_ms {
            c.retry_base = c.pace_ms;
        }
        c.cc.frame_payload = self.cfg.frame_payload();
        if self.conn.t_keepalive == 0 {
            let period = self.keepalive_period();
            let h = self.arm(period, 1000, Work::KeepAlive);
            self.conn.t_keepalive = h;
        }
    }

    /// `FUN_00423569` (DX9 0x00423569, dpnet.dll.c:30160-30182). `rtt` is the sample in ms
    /// (`now - frame+0x4c`, the last transmission); `retried` is `frame+0x48 != 0`.
    pub(crate) fn update_rtt(&mut self, rtt: u32, retried: bool) {
        let now = self.now;
        let max = self.cfg.sp_max_datagram;
        let c = &mut self.conn;
        let rtt = if rtt == 0 {
            1
        } else if (c.rtt << 7) < rtt {
            return;
        } else {
            rtt
        };
        // 30173-30175: a retransmitted frame's sample below the current rtt (`conn+0x3c`, the
        // value before this update) becomes `2 * rtt - sample`.
        let rtt = if retried && rtt < c.rtt {
            c.rtt.wrapping_mul(2).wrapping_sub(rtt)
        } else {
            rtt
        };
        let d = (rtt.wrapping_mul(0x10000)).wrapping_sub(c.srtt) as i32;
        c.srtt = c
            .srtt
            .wrapping_add(((d.wrapping_add((d >> 31) & 7)) >> 3) as u32);
        c.rtt = c.srtt >> 16;
        // DX9 0x00423569:30167-30181: RTO = ((rtt >> 2) + rtt) * 2 + 100.
        // The floor is `conn+0x10c` = `pace_ms` (30182), applied just below.
        c.retry_base = ((c.rtt >> 2).wrapping_add(c.rtt))
            .wrapping_mul(2)
            .wrapping_add(100);
        if c.retry_base < c.pace_ms {
            c.retry_base = c.pace_ms;
        }
        if !c.has(flag::CC_BACKOFF) {
            if c.cc.loss_score != 0 {
                c.cc.loss_score -= 1;
            }
        } else if (c.rtt << 3) < now.wrapping_sub(c.cc.loss_time) {
            c.clear(flag::CC_BACKOFF);
            c.cc.loss_events = 0;
            c.pace_ms = c.cc.loss_ms;
            c.pace_factor = c.cc.loss_factor;
            c.pace_bytes = c.pace_factor * max;
            c.frame_window = c.cc.loss_fw;
            c.byte_steps = c.cc.loss_steps;
            c.byte_window = c.byte_steps * max;
            c.cc.t_cc = now;
        }
        c.cc.countdown = c.cc.countdown.wrapping_sub(1);
        // Link tuning is not run with Initialize flag 4 (DX9 0x00423569:30185, `conn[5] & 0x400`).
        if c.cc.countdown == 0 && !self.cfg.link_tuning_off {
            self.cc_evaluate();
        }
    }

    /// `FUN_00421340` (0x00421340).
    fn cc_evaluate(&mut self) {
        let now = self.now;
        let max = self.cfg.sp_max_datagram;
        let c = &mut self.conn;
        if (c.flags & 0xc00) == 0 {
            c.cc.t_gap = c.cc.t_gap.wrapping_add(now.wrapping_sub(c.cc.t_last));
            c.cc.t_last = now;
        }
        let span = (now.wrapping_sub(c.cc.t_sample) as i32).wrapping_sub(c.cc.t_gap as i32);
        if span < 1 {
            c.cc.countdown = 4;
            return;
        }
        let delta = c.cc.acked_bytes.wrapping_sub(c.cc.acked_mark);
        let sum = delta.wrapping_mul(0x100).wrapping_add(c.cc.acc_bytes);
        if sum < c.cc.acc_bytes {
            c.cc.acc_bytes >>= 1;
            c.cc.acc_time >>= 1;
            c.cc.acc_bytes = delta.wrapping_mul(0x100).wrapping_add(c.cc.acc_bytes);
        } else {
            c.cc.acc_bytes = sum;
        }
        c.cc.acc_time = c.cc.acc_time.wrapping_add(span as u32);
        c.cc.t_gap = 0;
        c.cc.t_sample = now;
        c.cc.acked_mark = c.cc.acked_bytes;
        c.cc.rate = c.cc.acc_bytes / c.cc.acc_time;
        if c.cc.rate_max < c.cc.rate {
            c.cc.rate_max = c.cc.rate;
        }
        if !c.has(flag::CC_IDLE) {
            c.cc.countdown = 8;
            if c.cc.blocked_pumps > 0xc && c.cc.loss_score == 0 {
                c.cc.blocked_pumps = 0;
                if c.cc.rate_prev < c.cc.rate {
                    cc_grow(c, max, now);
                } else if !c.has(flag::CC_SECOND) {
                    c.set(flag::CC_SECOND);
                    c.cc.acc_bytes = 0;
                    c.cc.acc_time = 0;
                } else {
                    c.pace_ms = c.cc.save_ms;
                    c.pace_factor = c.cc.save_factor;
                    c.pace_bytes = c.pace_factor * max;
                    c.frame_window = c.cc.save_fw;
                    c.byte_steps = c.cc.save_steps;
                    c.byte_window = c.byte_steps * max;
                    c.set(flag::CC_IDLE);
                    c.clear(flag::CC_SECOND);
                    let key = (c.frame_window << 16) | c.byte_steps;
                    if c.cc.probe_key == key {
                        c.cc.probe_wait <<= 1;
                    } else {
                        c.cc.probe_wait = 60000;
                        c.cc.probe_key = key;
                    }
                    c.cc.acc_bytes = 0;
                    c.cc.acc_time = 0;
                }
            }
        } else {
            c.cc.countdown = 0x20;
            if !c.has(flag::CC_TOGGLE) && c.cc.probe_wait < now.wrapping_sub(c.cc.t_cc) {
                if (c.flags & 0x600000) == 0 {
                    c.cc.probe_wait = (c.cc.probe_wait << 1).min(480_000);
                } else {
                    c.clear(flag::CC_IDLE);
                    c.cc.rate_prev = 0;
                    c.cc.rate = 0;
                    c.cc.acc_bytes = 0;
                    c.cc.acc_time = 0;
                    c.cc.blocked_pumps = 0;
                    c.flags &= 0xff9f_ffff;
                    c.cc.countdown = 0xc;
                }
            }
        }
        c.clear(0x0100_0000);
    }

    /// `FUN_00421740` (0x00421740): a frame needed its first retransmission.
    pub(crate) fn cc_loss_event(&mut self, now: u32) {
        // With link tuning off (Initialize flag 4, `conn+0x14 |= 0x400`, FUN_00423298:30050) the
        // loss hook FUN_00423665 returns at its `(conn+0x14 & 0x400) == 0` gate (30215): neither
        // the window, the pace nor the loss counters are touched. The retransmit timer
        // FUN_00427bfa (34135) is otherwise unchanged by the flag.
        if self.cfg.link_tuning_off {
            return;
        }
        let c = &mut self.conn;
        c.cc.credit = c.cc.credit.wrapping_add(0x0100_0000);
        let prior = c.cc.loss_events;
        c.cc.loss_events += 1;
        if prior == 0 || c.cc.loss_events & 3 == 0 {
            self.cc_loss(now);
        }
    }

    /// `FUN_004217b0` (0x004217b0).
    fn cc_loss(&mut self, now: u32) {
        let max = self.cfg.sp_max_datagram;
        let c = &mut self.conn;
        c.set(flag::CC_BACKOFF);
        c.cc.loss_score += 10;
        c.cc.loss_time = now;
        c.cc.loss_factor = c.pace_factor;
        c.cc.loss_ms = c.pace_ms;
        c.cc.loss_fw = c.frame_window;
        c.cc.loss_steps = c.byte_steps;
        if c.frame_window == 1 && c.cc.credit < 0x0600_0000 {
            if c.pace_ms == 0 {
                c.pace_ms = if c.rtt >> 1 == 0 { 1 } else { c.rtt >> 1 };
            } else {
                c.pace_ms <<= 1;
            }
            c.pace_ms = c.pace_ms.min(0x9c4);
        }
        c.frame_window = if c.frame_window / 2 < 2 {
            1
        } else {
            c.frame_window / 2
        };
        c.byte_steps = if c.byte_steps >> 1 < 2 {
            1
        } else {
            c.byte_steps >> 1
        };
        c.byte_window = c.byte_steps * max;
        if c.cc.loss_score > 0x14 {
            c.cc.loss_score = 0;
            c.cc.loss_fw = if c.cc.loss_fw.wrapping_sub(1) < 2 {
                1
            } else {
                c.cc.loss_fw - 1
            };
            c.cc.loss_steps = if c.cc.loss_steps.wrapping_sub(1) < 2 {
                1
            } else {
                c.cc.loss_steps - 1
            };
            if c.cc.loss_factor < 2 {
                if c.cc.loss_ms != 0 {
                    c.cc.loss_ms = c.cc.loss_ms + 1 + (c.cc.loss_ms >> 2);
                }
            } else {
                c.cc.loss_factor -= 1;
                c.cc.loss_fw = c.cc.loss_factor;
            }
            c.set(flag::CC_IDLE);
        }
    }
}

/// `FUN_00421160` (0x00421160).
fn cc_grow(c: &mut super::Conn, max: u32, now: u32) {
    if c.pace_ms == 0 {
        c.cc.save_ms = c.pace_ms;
        c.cc.save_factor = c.pace_factor;
        c.clear(flag::CC_SECOND);
        c.cc.t_cc = now;
        c.cc.save_fw = c.frame_window;
        c.cc.save_steps = c.byte_steps;
        let mut grown = false;
        if c.has(flag::FRAME_WINDOW_FULL) && c.frame_window < 0x40 {
            c.frame_window += 1;
            grown = true;
        }
        if c.has(flag::BYTE_WINDOW_FULL) && c.byte_steps < 0x40 {
            c.byte_steps += 1;
            c.byte_window += max;
            grown = true;
        }
        c.flags &= 0xff9f_ffff;
        c.cc.blocked_pumps = 0;
        if grown {
            c.cc.rate_prev = c.cc.rate;
            c.cc.rate = 0;
            c.cc.acc_bytes = 0;
            c.cc.acc_time = 0;
        } else {
            c.set(flag::CC_IDLE);
        }
    } else {
        c.cc.save_ms = c.pace_ms;
        c.cc.save_factor = c.pace_factor;
        if c.pace_ms < 4 {
            c.pace_ms = 0;
        } else {
            c.pace_ms -= c.pace_ms >> 2;
        }
        c.cc.t_cc = now;
    }
}
