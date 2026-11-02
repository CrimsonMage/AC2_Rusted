//! The full client exchange against a scripted host: handshake, the first KeepAlive, 0xC1 /
//! 0xC2 / 0xC3, data both ways, a SACK, a KeepAlive after the silence, the graceful close.
//! The expected datagrams are literals from spec_client.md (sections 2.2-2.5, 3.2, the
//! adjudication's 0x3F/0x7F and `bRspId` rules) and verify_spec_client.md (M2, M5, 2.5).

use crate::Hresult;

use super::messages::*;
use super::tests::*;

const SID: u32 = 0x1122_3344;
const T0: u32 = 10_000;

fn le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

/// A host DFRAME: `cmd ctl seq nrcv` then the payload.
fn dframe(cmd: u8, ctl: u8, seq: u8, nrcv: u8, payload: &[u8]) -> Vec<u8> {
    let mut v = vec![cmd, ctl, seq, nrcv];
    v.extend_from_slice(payload);
    v
}

/// A host SACK: `80 06 01 00 nseq nrcv 00 00 tick`.
fn host_sack(nseq: u8, nrcv: u8) -> Vec<u8> {
    let mut v = vec![0x80, 0x06, 0x01, 0x00, nseq, nrcv, 0, 0];
    v.extend_from_slice(&le(0));
    v
}

fn c1_payload() -> Vec<u8> {
    // type 0xC1, then the DX9 `_EX` body (verify_spec_client_delta 3.1): dwFlags 2,
    // dwDNETVersion 7, name/data/password/connectData/URL all 0/0, guidInstance zero,
    // guidApplication, alternate-address slot 0/0: exactly 0x58 bytes, so 0x5c with the type.
    let mut p = vec![0xC1, 0, 0, 0, 2, 0, 0, 0, 7, 0, 0, 0];
    p.extend_from_slice(&[0; 40]);
    p.extend_from_slice(&[0; 16]);
    p.extend_from_slice(&[
        0x7d, 0x76, 0x80, 0x22, 0x89, 0x16, 0x6b, 0x4d, 0x81, 0xdd, 0x3a, 0xd9, 0xe8, 0x0f, 0xb6,
        0x55,
    ]);
    p.extend_from_slice(&[0; 8]);
    assert_eq!(p.len(), 0x5c);
    p
}

/// Run the handshake; returns the harness at the moment CONNECT_COMPLETE was delivered.
fn connected() -> H {
    let mut h = H::new();
    h.gundll_caps();
    let handle = h.client.connect(
        &app_desc(),
        &super::address::DpAddress::build_address("10.0.0.5", 2300),
        &{
            let mut d = super::address::DpAddress::new();
            d.set_sp(super::address::CLSID_DP8SP_TCPIP);
            d.add_component("port", super::address::Component::Dword(2302));
            d
        },
        false,
        false,
        None,
        0xC0FFEE,
        0,
    );
    assert_eq!(handle, Ok(DpnHandle(1)));
    assert_eq!(h.client.local_port(), Some(2302));
    // Nothing goes out in the Connect call: the SP completion runs in DoWork.
    assert!(h.sent().is_empty());
    h.tick(); // 10020
    assert_eq!(h.sent(), vec![cframe(true, 1, 0, 0, SID, T0 + 20)]);
    h.tick(); // 10040
    assert!(h.sent().is_empty());
    // The host's CONNECTED answers the CONNECT it saw (bRspId = its bMsgID 0) and asks to be
    // confirmed (POLL).
    h.host(&cframe(true, 2, 5, 0, SID, 77));
    h.tick(); // 10060
    let out = h.sent();
    assert_eq!(
        out,
        vec![
            // the connector's CONNECTED: bMsgID 1, bRspId = the host's bMsgID, no POLL
            cframe(false, 2, 1, 5, SID, T0 + 60),
            // the KeepAlive `OnConnected` queues at once (DX9 0x00421c4e:28803): 0x3F, bControl 0x02 (v1.6
            // host), seq 0, nrcv 0, then the 4-byte session id
            vec![0x3F, 0x02, 0x00, 0x00, 0x44, 0x33, 0x22, 0x11],
            // 0xC1 as a lone reliable USER_1 frame: 0x77 | POLL = 0x7F, seq 1
            dframe(0x7F, 0x00, 0x01, 0x00, &c1_payload()),
        ]
    );
    assert!(h.got().is_empty());
    // The host's DFRAME carries 0xC2 and acknowledges both frames (nrcv = 2).
    let mut c2 = vec![0xC2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    c2.extend_from_slice(&[0; 4]);
    h.host(&dframe(0x77, 0x00, 0x00, 0x02, &c2));
    h.tick(); // 10080
              // 0xC3: the body is empty, so the payload is just the type; seq 2, nrcv 1.
    assert_eq!(
        h.sent(),
        vec![dframe(0x7F, 0x00, 0x02, 0x01, &[0xC3, 0, 0, 0])]
    );
    assert_eq!(h.got(), vec![Got::Connect(Hresult(0), vec![], 0xC0FFEE)]);
    h
}

#[test]
fn handshake_matches_the_spec_byte_for_byte() {
    let _ = connected();
}

#[test]
fn data_receive_pending_buffer_delayed_ack_and_send() {
    let mut h = connected();
    // Host data: 0x37 (no POLL), seq 1, nrcv 3 acknowledges the 0xC3.
    h.host(&dframe(0x37, 0x00, 0x01, 0x03, b"hello"));
    h.tick(); // 10100
    let got = h.got();
    assert_eq!(got.len(), 1);
    let Got::Receive(d, buf) = got[0].clone() else {
        panic!("{got:?}")
    };
    assert_eq!(d, b"hello");
    // The handler returned PENDING: the buffer is held until ReturnBuffer.
    assert_eq!(h.client.held_buffer(buf).unwrap(), b"hello");
    assert_eq!(h.client.return_buffer(buf, 0), Hresult(0));
    assert!(h.client.held_buffer(buf).is_none());
    assert_eq!(
        h.client.return_buffer(buf, 0),
        super::hresult::DPNERR_INVALIDHANDLE
    );
    // No POLL: the 100 ms delayed ACK. The datagram arrived at 10100, so the timer expires at
    // 10200 and DX9's pool runs it on the first tick whose `time & ~3` is strictly later
    // (`FUN_0042a96d`): with this pump's 20 ms ticks that is 10220, not 10200.
    h.ticks(5);
    assert!(h.sent().is_empty());
    h.tick(); // 10220
              // SACK: bRetry 0, bNSeq 3, bNRcv 2, tick = receive time of the last DFRAME (10100).
    let mut sack = vec![0x80, 0x06, 0x01, 0x00, 0x03, 0x02, 0x00, 0x00];
    sack.extend_from_slice(&le(T0 + 100));
    assert_eq!(h.sent(), vec![sack]);
    // Send "AB" with GunDll's flags 0x9 (NOCOPY | GUARANTEED): one lone frame, 0x37 | POLL.
    let t_send = h.now();
    let hd = h.client.send(&[b"AB"], 0, 0xABCD, 0x9).unwrap();
    assert!(h.sent().is_empty());
    h.tick();
    assert_eq!(h.sent(), vec![dframe(0x3F, 0x00, 0x03, 0x02, b"AB")]);
    // The host acknowledges with a SACK (nrcv 4): SEND_COMPLETE with the send time.
    h.host(&host_sack(2, 4));
    h.tick();
    let sent_at = h.now();
    assert_eq!(
        h.got(),
        vec![Got::Send(Hresult(0), 0xABCD, sent_at - t_send)]
    );
    let _ = hd;
}

#[test]
fn keepalive_goes_out_after_15_seconds_of_silence_checked_every_3750_ms() {
    let mut h = connected();
    // The host acknowledges the 0xC3 (nrcv 3) at 10100 and then goes silent.
    h.host(&host_sack(1, 3));
    h.tick();
    assert!(h.sent().is_empty());
    // GunDll's 15000: the check period is min(15000 >> 2, 4000) = 3750 ms (DX9 0x004233e3:30143),
    // a KeepAlive goes out at the first check with more than 15000 ms of silence (> 25100).
    let mut at = None;
    while h.now() < 40_000 && at.is_none() {
        h.tick();
        for d in h.sent() {
            if d[0] == 0x3F {
                at = Some((h.now(), d));
            }
        }
    }
    let (t, d) = at.expect("a KeepAlive");
    assert!(t > 25_100 && t <= 25_100 + 3_750 + 60, "{t}");
    // seq 3 (keepalive 0, 0xC1 1, 0xC3 2), nrcv 1, v1.6 form with the session id
    assert_eq!(d, vec![0x3F, 0x02, 0x03, 0x01, 0x44, 0x33, 0x22, 0x11]);
}

#[test]
fn a_lost_keepalive_is_retransmitted_with_control_0x03() {
    // FUN_0042638d (DX9 0x0042638d, asm; verify_phase4a_dx9 finding B) keeps `bControl & 0x0e`
    // and sets RETRY: the v1.6 keep-alive 3F 02 goes out again as 3F 03 with its session id.
    let mut h = connected();
    h.host(&host_sack(1, 3));
    h.tick();
    let mut first = None;
    let mut again = None;
    while h.now() < 60_000 && again.is_none() {
        h.tick();
        for d in h.sent() {
            if d[0] == 0x3F && d.len() == 8 {
                if first.is_none() {
                    first = Some(d);
                } else {
                    again = Some(d);
                }
            }
        }
    }
    assert_eq!(
        first.unwrap(),
        vec![0x3F, 0x02, 0x03, 0x01, 0x44, 0x33, 0x22, 0x11]
    );
    assert_eq!(
        again.expect("a retransmission"),
        vec![0x3F, 0x03, 0x03, 0x01, 0x44, 0x33, 0x22, 0x11]
    );
}
