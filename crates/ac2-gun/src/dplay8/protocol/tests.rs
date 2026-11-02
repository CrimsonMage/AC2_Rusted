//! Protocol-engine tests against a scripted host, no sockets: the connect retry timer, the
//! CONNECTED rules, ACK timers, SACK handling, the retransmit formula, windows, fragmentation,
//! END_STREAM, HARD_DISCONNECT. Expected frames are literals from spec_client.md and
//! verify_spec_client.md; expected times are hand-derived from the timer wheel
//! (`timers.rs`): the pump ticks every 20 ms, a delay `d` under 0x2000 ms lands `((d + 2) >> 2)`
//! buckets ahead and the wheel advances 5 buckets per tick, so a timer armed from the ready
//! queue fires after `ceil(((d + 2) / 4 + 1) / 5)` ticks; a longer delay fires at the first tick
//! whose `now + 10` is past its expiry.

use crate::Hresult;

use super::frames::*;
use super::*;
use crate::dplay8::hresult::*;

const SID: u32 = 0xA1B2_C3D4;

/// ticks (of 20 ms) a wheel timer of `d` ms takes when armed right after a tick
fn wheel_ms(d: u32) -> u32 {
    if d >= 0x2000 {
        // first tick with now + 10 > armed + d
        let t = d.saturating_sub(9);
        return t.div_ceil(20) * 20;
    }
    let slot = (d + 2) >> 2;
    (slot + 1).div_ceil(5) * 20
}

struct P {
    p: Protocol,
    now: u32,
    out: Vec<(u32, Vec<u8>)>,
}

impl P {
    fn new() -> Self {
        Self::with_cfg(ProtocolConfig::default())
    }

    fn with_cfg(cfg: ProtocolConfig) -> Self {
        Self {
            p: Protocol::with_session_seed(cfg, 1000, SID),
            now: 1000,
            out: Vec::new(),
        }
    }

    fn tick(&mut self) {
        self.now += 20;
        self.p.do_work(self.now);
        let now = self.now;
        self.out
            .extend(self.p.take_outbox().into_iter().map(|d| (now, d)));
    }

    fn ticks(&mut self, n: u32) {
        for _ in 0..n {
            self.tick();
        }
    }

    fn feed(&mut self, d: &[u8]) {
        self.p.on_datagram(self.now, d);
    }

    fn drain(&mut self) -> Vec<(u32, Vec<u8>)> {
        std::mem::take(&mut self.out)
    }

    /// CONNECT at 1020, CONNECTED at 1040 (rtt 20), first KeepAlive acknowledged at 1060.
    fn established() -> Self {
        let mut p = Self::new();
        p.p.connect(1000);
        p.tick();
        p.drain();
        p.ticks(1);
        p.feed(&cframe(true, OP_CONNECTED, 7, 0, 0));
        p.tick();
        let o = p.drain();
        assert_eq!(o.len(), 2, "{o:?}");
        // v1.6 host: the KeepAlive carries bControl 0x02 and the session id (DX9 0x0042687f)
        let mut ka = vec![0x3F, 0x02, 0, 0];
        ka.extend_from_slice(&SID.to_le_bytes());
        assert_eq!(o[1].1, ka);
        p.feed(&sack(0, 1, 0));
        p.tick();
        p.p.take_events();
        p.drain();
        p
    }
}

fn cframe(poll: bool, op: u8, mid: u8, rsp: u8, tick: u32) -> Vec<u8> {
    let mut v = vec![if poll { 0x88 } else { 0x80 }, op, mid, rsp, 6, 0, 1, 0];
    v.extend_from_slice(&SID.to_le_bytes());
    v.extend_from_slice(&tick.to_le_bytes());
    v
}

fn sack(nseq: u8, nrcv: u8, mask1: u32) -> Vec<u8> {
    let mut v = vec![
        0x80,
        0x06,
        if mask1 != 0 { 0x03 } else { 0x01 },
        0,
        nseq,
        nrcv,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    if mask1 != 0 {
        v.extend_from_slice(&mask1.to_le_bytes());
    }
    v
}

fn dframe(cmd: u8, ctl: u8, seq: u8, nrcv: u8, payload: &[u8]) -> Vec<u8> {
    let mut v = vec![cmd, ctl, seq, nrcv];
    v.extend_from_slice(payload);
    v
}

fn connect_frames(o: &[(u32, Vec<u8>)]) -> Vec<(u32, Vec<u8>)> {
    o.iter()
        .filter(|(_, d)| d.len() == 16 && d[1] == OP_CONNECT)
        .cloned()
        .collect()
}

#[test]
fn connect_retries_double_from_200_ms_capped_at_5_s_fifteen_connects_then_noresponse() {
    let mut p = P::new();
    p.p.connect(1000);
    let mut ev = Vec::new();
    while p.now < 1000 + 70_000 {
        p.tick();
        ev.extend(p.p.take_events());
    }
    let c = connect_frames(&p.out);
    assert_eq!(c.len(), 15, "1 CONNECT + 14 retries");
    // bMsgID counts up from 0, POLL set, same session id, version 0x00010006.
    for (i, (_, d)) in c.iter().enumerate() {
        assert_eq!(d[..8], [0x88, 0x01, i as u8, 0, 6, 0, 1, 0]);
        assert_eq!(d[8..12], SID.to_le_bytes());
    }
    // The waits armed: 200, then the interval doubles before each resend and is cut to 5000
    // when over 4999 (DX9 0x00422356:29127-29132): 400, 800, 1600, 3200, 5000 x 10.
    let t: Vec<u32> = c.iter().map(|(t, _)| *t).collect();
    assert_eq!(t[0], 1020);
    let armed = [
        200u32, 400, 800, 1600, 3200, 5000, 5000, 5000, 5000, 5000, 5000, 5000, 5000, 5000,
    ];
    let mut want = 1020;
    for (i, d) in armed.iter().enumerate() {
        want += wheel_ms(*d);
        assert_eq!(t[i + 1], want, "resend {}", i + 1);
    }
    // Failure after the 15th expiry (nominal total 56.2 s), reported as 0x80158510.
    assert_eq!(ev, vec![Event::ConnectResult(Err(DPNERR_NORESPONSE))]);
    assert!(p.p.is_dead());
}

#[test]
fn connected_must_echo_an_outstanding_connect_msgid() {
    let mut p = P::new();
    p.p.connect(1000);
    p.tick();
    p.drain();
    // rsp 9 matches no CONNECT: the connector still answers the POLL (with CONNECTED) but
    // does not establish, and keeps retrying.
    p.feed(&cframe(true, OP_CONNECTED, 3, 9, 0));
    p.tick();
    let o = p.drain();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].1[..4], [0x80, 0x02, 0x01, 0x03]);
    assert!(!p.p.is_connected());
    assert!(p.p.take_events().is_empty());
    // A CONNECTED for another session id is ignored.
    let mut other = cframe(true, OP_CONNECTED, 3, 0, 0);
    other[8] ^= 0xff;
    p.feed(&other);
    p.tick();
    assert!(p.drain().is_empty());
    assert!(!p.p.is_connected());
    // The bogus rsp 9 dropped the older record (FUN_00423a70 frees records in front of the
    // match); a retry CONNECT (bMsgID 2: the CONNECTED reply above took 1) makes a new one.
    p.ticks(11);
    let retry = connect_frames(&p.drain());
    assert_eq!(retry.len(), 1);
    assert_eq!(retry[0].1[2], 2);
    // The right one (POLL clear: no reply) establishes.
    p.feed(&cframe(false, OP_CONNECTED, 4, 2, 0));
    p.tick();
    assert!(p.p.is_connected());
    assert_eq!(p.p.take_events(), vec![Event::ConnectResult(Ok(()))]);
}

#[test]
fn in_order_frame_without_poll_is_acked_after_100_ms() {
    let mut p = P::established();
    p.feed(&dframe(0x37, 0, 0, 1, b"x"));
    let t0 = p.now;
    p.ticks(5);
    assert!(p.drain().is_empty());
    p.tick();
    let o = p.drain();
    assert_eq!(o.len(), 1);
    assert_eq!(
        o[0].0,
        t0 + 120,
        "100 ms timer from a datagram handled before the tick"
    );
    assert_eq!(o[0].1[..6], [0x80, 0x06, 0x01, 0x00, 0x01, 0x01]);
}

#[test]
fn in_order_poll_frame_takes_the_short_ack_and_end_stream_acks_now() {
    let mut p = P::established();
    p.feed(&dframe(0x3F, 0, 0, 1, b"x"));
    p.tick();
    // 4 ms timer: due within the tick that follows the datagram.
    let o = p.drain();
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(o[0].1[..6], [0x80, 0x06, 0x01, 0x00, 0x01, 0x01]);
    // END_STREAM (bControl 0x08) in order: a SACK as the frame is handled, before any tick
    // (DX9 0x004259cf:32429-32431). bControl 0x02 alone is not "ACK now" any more.
    let mut q = P::established();
    q.feed(&dframe(0x37, 0x08, 0, 1, &[]));
    q.p.do_work(q.now);
    let flushed = q.p.take_outbox();
    assert!(
        flushed
            .iter()
            .any(|f| f.len() >= 6 && f[..6] == [0x80, 0x06, 0x01, 0x00, 0x01, 0x01]),
        "{flushed:?}"
    );
}

#[test]
fn out_of_order_frame_sets_the_mask_and_acks_after_the_20_ms_sack_timer() {
    let mut p = P::established();
    // nextRecv is 0; seq 1 is one past the missing seq 0: diff 0 -> mask bit 0.
    p.feed(&dframe(0x37, 0, 1, 1, b"b"));
    p.tick();
    assert!(p.drain().is_empty());
    p.tick();
    let o = p.drain();
    assert_eq!(o.len(), 1, "{o:?}");
    // flags RESPONSE | MASK1, bNRcv 0, mask 0b1
    assert_eq!(o[0].1[..6], [0x80, 0x06, 0x03, 0x00, 0x01, 0x00]);
    assert_eq!(o[0].1[12..16], [1, 0, 0, 0]);
    // The missing frame arrives: in order again, both are delivered, mask clears.
    p.feed(&dframe(0x37, 0, 0, 1, b"a"));
    p.tick();
    let ev = p.p.take_events();
    let rx: Vec<_> = ev
        .iter()
        .filter_map(|e| match e {
            Event::Receive { data, .. } => Some(data.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(rx, vec![b"a".to_vec(), b"b".to_vec()]);
}

#[test]
fn out_of_window_frames_ack_now_with_poll_else_in_20_ms_and_duplicates_are_dropped() {
    let mut p = P::established();
    // diff 0x3f is outside the window of 0..62.
    p.feed(&dframe(0x3F, 0, 0x3f, 1, b"z"));
    p.p.do_work(p.now);
    let o = p.p.take_outbox();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0][..6], [0x80, 0x06, 0x01, 0x00, 0x01, 0x00]);
    p.feed(&dframe(0x37, 0, 0x3f, 1, b"z"));
    p.tick();
    assert!(p.drain().is_empty());
    p.tick();
    assert_eq!(p.drain().len(), 1);
    // duplicate of a held out-of-order frame: nothing new
    p.feed(&dframe(0x37, 0, 2, 1, b"q"));
    p.feed(&dframe(0x37, 0, 2, 1, b"q"));
    p.ticks(3);
    assert_eq!(p.p.conn.ooo.len(), 1);
}

#[test]
fn sack_gap_arms_the_10_ms_retry_and_cumulative_ack_completes_sends() {
    let mut p = P::established();
    p.p.queue_message(
        p.now,
        11,
        b"A",
        SendMode {
            reliable: true,
            nonsequential: false,
            priority: Priority::Normal,
            user_bits: 0,
        },
    )
    .unwrap();
    p.p.queue_message(
        p.now,
        12,
        b"B",
        SendMode {
            reliable: true,
            nonsequential: false,
            priority: Priority::Normal,
            user_bits: 0,
        },
    )
    .unwrap();
    p.tick();
    let o = p.drain();
    // "A" is not the last queued message: 0x37; "B" is the last: 0x37 | POLL = 0x3F.
    assert_eq!(
        o.iter().map(|x| x.1.clone()).collect::<Vec<_>>(),
        vec![dframe(0x37, 0, 1, 0, b"A"), dframe(0x3F, 0, 2, 0, b"B")]
    );
    // The host holds seq 2 but not seq 1: SACK nrcv 1, mask bit 0 = frame 2. DX9 has no
    // immediate resend: it replaces the retry timer with a 10 ms one for the oldest frame
    // (0x00427eeb:34368-34378), which resends only if that frame is older than the RTO.
    p.tick();
    p.feed(&sack(0, 1, 1));
    p.tick();
    assert!(p.drain().is_empty());
    assert!(p.p.conn.inflight[0].timer_owner);
    // Cumulative ack of both: SEND_COMPLETE x2 in order.
    p.feed(&sack(0, 3, 0));
    p.tick();
    let ev = p.p.take_events();
    assert_eq!(
        ev,
        vec![
            Event::SendComplete {
                op: 11,
                result: Hresult(0)
            },
            Event::SendComplete {
                op: 12,
                result: Hresult(0)
            }
        ]
    );
}

#[test]
fn retransmit_backoff_follows_the_formula_and_the_11th_expiry_drops_the_connection() {
    let mut p = P::established();
    let base = p.p.conn.retry_base;
    // First sample 20 ms: srtt = 20; the KeepAlive's ack came 0 ms (counted 1) after its send:
    // srtt += (1 - 20) / 8 -> rtt 17 and base = ((17 >> 2) + 17) * 2 + 100 = 142 (DX9 RTO 2.5 *
    // rtt + 100, 0x00423569:30167-30181).
    assert_eq!(base, 142);
    p.p.queue_message(
        p.now,
        1,
        b"R",
        SendMode {
            reliable: true,
            nonsequential: false,
            priority: Priority::Normal,
            user_bits: 0,
        },
    )
    .unwrap();
    p.tick();
    p.drain();
    let base = p.p.conn.retry_base;
    let mut ev = Vec::new();
    let mut resends = Vec::new();
    while resends.len() < 10 && p.now < 400_000 {
        p.tick();
        ev.extend(p.p.take_events());
        for (t, d) in p.drain() {
            if d[1] & 1 == 1 {
                resends.push((t, d));
            }
        }
    }
    assert_eq!(resends.len(), 10);
    assert!(resends
        .iter()
        .all(|(_, d)| d[..4] == [0x3F, 0x01, 0x01, 0x00] && d[4..] == *b"R"));
    // waits between expiries: k=1 base, 2..3 base*k, 4..7 base*(1<<k), then 5000.
    let waits: Vec<u32> = (1..=10u32)
        .map(|k| match k {
            1 => base,
            2..=3 => base * k,
            4..=7 => base * (1 << k),
            _ => 5000,
        })
        // any wait of 5000 or more is cut to 5000 (rtt is far below 5000 here)
        .map(|w| if w > 4999 { 5000 } else { w })
        .collect();
    for i in 1..10 {
        let gap = resends[i].0 - resends[i - 1].0;
        let want = waits[i - 1];
        assert!(gap >= want && gap <= want + 40, "gap {i}: {gap} vs {want}");
    }
    // the 11th expiry tears the connection down: the user send fails with CONNECTIONLOST
    while !p.p.is_dead() && p.now < 400_000 {
        p.tick();
        ev.extend(p.p.take_events());
    }
    assert!(p.p.is_dead());
    assert!(
        ev.contains(&Event::ConnectionLost(DPNERR_CONNECTIONLOST)),
        "{ev:?}"
    );
    assert!(ev.contains(&Event::SendComplete {
        op: 1,
        result: DPNERR_CONNECTIONLOST
    }));
}

#[test]
fn fragmentation_at_1356_with_window_of_two_frames() {
    let mut p = P::established();
    let data: Vec<u8> = (0..3000u32).map(|i| (i % 251) as u8).collect();
    p.p.queue_message(
        p.now,
        5,
        &data,
        SendMode {
            reliable: true,
            nonsequential: false,
            priority: Priority::Normal,
            user_bits: 0,
        },
    )
    .unwrap();
    p.tick();
    let o = p.drain();
    // Frames 1356, 1356, 288. cmd: first NEW_MSG 0x17; the second would be 0x07 but the window
    // (2 frames) fills, so it carries POLL: 0x0F. The third waits for an ack.
    assert_eq!(o.len(), 2);
    assert_eq!(o[0].1[..4], [0x17, 0, 1, 0]);
    assert_eq!(o[0].1.len(), 4 + 1356);
    assert_eq!(o[0].1[4..], data[..1356]);
    assert_eq!(o[1].1[..4], [0x0F, 0, 2, 0]);
    assert_eq!(o[1].1[4..], data[1356..2712]);
    p.feed(&sack(0, 3, 0));
    p.tick();
    let o = p.drain();
    // The last frame: END_MSG 0x27 and, last queued, POLL: 0x2F.
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].1[..4], [0x2F, 0, 3, 0]);
    assert_eq!(o[0].1[4..], data[2712..]);
}

#[test]
fn a_fragmented_message_from_the_host_is_reassembled_into_one_receive() {
    let mut p = P::established();
    let data: Vec<u8> = (0..3000u32).map(|i| (i % 253) as u8).collect();
    p.feed(&dframe(0x17, 0, 0, 1, &data[..1356]));
    p.feed(&dframe(0x07, 0, 1, 1, &data[1356..2712]));
    p.feed(&dframe(0x27, 0, 2, 1, &data[2712..]));
    p.tick();
    let ev = p.p.take_events();
    assert_eq!(ev, vec![Event::Receive { data, flags: 0 }]);
}

#[test]
fn graceful_disconnect_end_stream_then_peer_end_stream_ends_with_a_final_ack() {
    let mut p = P::established();
    p.p.graceful_disconnect(p.now, 99).unwrap();
    p.tick();
    let o = p.drain();
    // END_STREAM: 0x27 | NEW_MSG, last queued -> POLL: 0x3F; bControl 0x08; seq 1.
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].1, vec![0x3F, 0x08, 0x01, 0x00]);
    // Closing (flag 0x20) is not dead: the exchange is still under way.
    assert!(!p.p.is_dead());
    // New sends are refused while closing.
    assert!(p
        .p
        .queue_message(
            p.now,
            2,
            b"x",
            SendMode {
                reliable: true,
                nonsequential: false,
                priority: Priority::Normal,
                user_bits: 0
            }
        )
        .is_err());
    // Ack of ours, then the host's END_STREAM (zero-length, control 0x08).
    p.feed(&sack(0, 2, 0));
    p.tick();
    assert!(p.p.take_events().is_empty());
    p.feed(&dframe(0x37, 0x08, 0, 2, &[]));
    p.p.do_work(p.now);
    // The final SACK goes out at once (BuildSack with the flush flag) before the teardown.
    let flushed = p.p.take_outbox();
    // An END_STREAM frame carries CORRELATE (control 0x08), which asks for an immediate SACK
    // (`Dpl8r_HandleDFrame`, dpnet.dll.c:27779), so the same SACK is built twice: once for the
    // frame, once by `Dpl8r_OnEndStream`'s final `BuildSack(conn, 1)`.
    assert_eq!(flushed.len(), 2);
    for f in &flushed {
        assert_eq!(f[..6], [0x80, 0x06, 0x01, 0x00, 0x02, 0x01]);
    }
    let ev = p.p.take_events();
    assert_eq!(
        ev,
        vec![Event::DisconnectComplete {
            op: 99,
            result: Hresult(0)
        }]
    );
    assert!(p.p.is_dead());
}

#[test]
fn the_hosts_end_stream_is_answered_with_ours_and_ends_the_session() {
    let mut p = P::established();
    p.feed(&dframe(0x37, 0x08, 0, 1, &[]));
    p.tick();
    let ev = p.p.take_events();
    assert_eq!(ev, vec![Event::PeerEndStream]);
    let o = p.drain();
    // our END_STREAM reply (a retransmit-free first send): control 0x08, nrcv 1
    let es: Vec<_> = o.iter().filter(|(_, d)| d[0] & 1 == 1).collect();
    assert_eq!(es.len(), 1);
    assert_eq!(es[0].1, vec![0x3F, 0x08, 0x01, 0x01]);
    // The host acknowledges it: both streams are done.
    p.feed(&sack(0, 2, 0));
    p.tick();
    assert_eq!(p.p.take_events(), vec![Event::ConnectionLost(Hresult(0))]);
    assert!(p.p.is_dead());
}

#[test]
fn hard_disconnect_from_the_host_tears_down_and_ours_is_three_cframes() {
    let mut p = P::established();
    p.feed(&cframe(false, OP_HARD_DISCONNECT, 0, 0, 0));
    p.tick();
    assert_eq!(
        p.p.take_events(),
        vec![Event::ConnectionLost(DPNERR_CONNECTIONLOST)]
    );
    let mut q = P::established();
    q.p.hard_disconnect(q.now);
    // the first at once, the other two at clamp(rtt / 2, 10, 500) ms
    q.ticks(3);
    let o = q.drain();
    assert_eq!(o.len(), 3);
    // bMsgID follows the CONNECTED reply (mid 1): 2, 3, 4; op 4; no POLL; rsp 0.
    for (i, (_, d)) in o.iter().enumerate() {
        assert_eq!(d[..4], [0x80, 0x04, 2 + i as u8, 0]);
        assert_eq!(d[8..12], SID.to_le_bytes());
    }
}

#[test]
fn bad_cframes_are_ignored_by_a_client() {
    // FUN_00424ca3 tears down only a Listen-owned conn: unknown opcode, byte 0 not 0x80/0x88,
    // a short frame and a HARD_DISCONNECT with the wrong layout leave a client untouched.
    let mut p = P::established();
    p.feed(&cframe(false, 5, 0, 0, 0));
    let mut odd = cframe(false, 2, 0, 0, 0);
    odd[0] = 0x90;
    p.feed(&odd);
    p.feed(&[0x80, 0x02, 0, 0, 6, 0, 1, 0]);
    assert!(!p.p.is_dead());
    assert!(p.p.is_connected());
    assert!(p.p.take_events().is_empty());
}

#[test]
fn an_inbound_hard_disconnect_ends_our_own_hard_disconnect_at_once() {
    // FUN_004257da:32155 handles opcode 4 before the 0x40000000 test; FUN_004247fc tears down.
    let mut q = P::established();
    q.p.hard_disconnect(q.now);
    assert!(!q.p.is_dead());
    q.feed(&cframe(false, 4, 0, 0, 0));
    assert!(q.p.is_dead());
}

#[test]
fn dframes_before_the_handshake_ends_are_dropped() {
    let mut p = P::new();
    p.p.connect(1000);
    p.tick();
    p.feed(&dframe(0x37, 0, 0, 0, b"early"));
    p.tick();
    assert!(p.p.take_events().is_empty());
    assert!(p.p.conn.ooo.is_empty() && p.p.conn.delivery.is_empty());
}

#[test]
fn the_first_retransmission_is_a_loss_event_that_shrinks_the_window_to_one_frame() {
    let mut p = P::established();
    assert_eq!((p.p.conn.frame_window, p.p.conn.byte_window), (2, 2768));
    p.p.queue_message(
        p.now,
        1,
        b"L",
        SendMode {
            reliable: true,
            nonsequential: false,
            priority: Priority::Normal,
            user_bits: 0,
        },
    )
    .unwrap();
    p.tick();
    while p.p.conn.inflight[0].retries == 0 {
        p.tick();
    }
    // FUN_004217b0: frame window 2 / 2 = 1 (floor 1), byte steps stay 1, score +10, back-off flag
    // 0x4000000; the pacing only starts from a window of one frame (`frame_window == 1` is
    // tested before the halving).
    assert_eq!(p.p.conn.frame_window, 1);
    assert_eq!(p.p.conn.byte_window, 1384);
    assert_eq!(p.p.conn.cc.loss_score, 10);
    assert!(p.p.conn.has(flag::CC_BACKOFF));
    assert_eq!(p.p.conn.pace_ms, 0);
}

#[test]
fn an_unacknowledged_end_stream_is_resent_with_control_0x09() {
    let mut p = P::established();
    p.p.graceful_disconnect(p.now, 5).unwrap();
    p.tick();
    assert_eq!(p.drain()[0].1, vec![0x3F, 0x08, 0x01, 0x00]);
    let mut resend = None;
    while resend.is_none() {
        p.tick();
        resend = p.drain().into_iter().find(|(_, d)| d[0] & 1 == 1);
    }
    // CORRELATE (0x02) is dropped on a retry, END_STREAM kept, RETRY set: 0x09 (M5).
    assert_eq!(resend.unwrap().1, vec![0x3F, 0x09, 0x01, 0x00]);
}
