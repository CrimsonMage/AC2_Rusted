//! Core-layer tests: close, 0xDF, 0xC5, connection loss, 0xE0/0xE1, cancel. The handshake is
//! the one of `tests_session` (`connected()`), reproduced here from its literals.

use crate::Hresult;

use super::address::*;
use super::hresult::*;
use super::tests::*;

const SID: u32 = 0x1122_3344;

fn le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

fn dframe(cmd: u8, ctl: u8, seq: u8, nrcv: u8, payload: &[u8]) -> Vec<u8> {
    let mut v = vec![cmd, ctl, seq, nrcv];
    v.extend_from_slice(payload);
    v
}

fn start() -> H {
    start_with(4)
}

fn start_with(flags: u32) -> H {
    let mut h = H::with_keep(true, flags);
    h.gundll_caps();
    let mut dev = DpAddress::new();
    dev.set_sp(CLSID_DP8SP_TCPIP);
    dev.add_component("port", Component::Dword(2302));
    h.client
        .connect(
            &app_desc(),
            &DpAddress::build_address("10.0.0.5", 2300),
            &dev,
            false,
            false,
            None,
            0xC0FFEE,
            0,
        )
        .unwrap();
    h.tick();
    h.sent();
    h.tick();
    h.host(&cframe(true, 2, 5, 0, SID, 77));
    h.tick();
    h.sent();
    h
}

fn connected() -> H {
    let mut h = start();
    h.host(&dframe(
        0x77,
        0,
        0,
        2,
        &[0xC2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ));
    h.tick();
    h.sent();
    assert_eq!(h.got(), vec![Got::Connect(Hresult(0), vec![], 0xC0FFEE)]);
    h
}

#[test]
fn close_queues_end_stream_pumps_once_and_returns_without_draining() {
    let mut h = connected();
    // The host answers our END_STREAM (seq 3: keepalive 0, 0xC1 1, 0xC3 2) with an ack of
    // everything and its own END_STREAM, as a well-behaved peer does.
    h.net.borrow_mut().responder = Some(Box::new(|d: &[u8]| {
        if d.len() == 4 && d[1] == 0x08 {
            let mut sack = vec![0x80, 0x06, 0x01, 0x00, 0x01, d[2].wrapping_add(1), 0, 0];
            sack.extend_from_slice(&le(0));
            vec![sack, dframe(0x37, 0x08, 1, d[2].wrapping_add(1), &[])]
        } else {
            vec![]
        }
    }));
    assert_eq!(h.client.close(0), Hresult(0));
    let sent = h.sent();
    // END_STREAM: 0x27 + NEW_MSG, last queued -> POLL: 0x3F; control 0x08 (v1.6); seq 3; nrcv 1.
    assert_eq!(sent[0], vec![0x3F, 0x08, 0x03, 0x01]);
    // Then the SACKs for the host's END_STREAM frame (nseq 4, nrcv 2).
    assert!(
        sent[1..]
            .iter()
            .all(|d| d[..6] == [0x80, 0x06, 0x01, 0x00, 0x04, 0x02]),
        "{sent:?}"
    );
    assert!(sent.len() >= 2);
    // A local Close tells the handler nothing.
    assert!(h.got().is_empty());
    assert_eq!(h.client.state(), super::core::CoreState::Idle);
}

#[test]
fn terminate_session_0xdf_carries_the_hosts_data() {
    let mut h = connected();
    let mut body = vec![0xDF, 0, 0, 0];
    body.extend_from_slice(&le(8));
    body.extend_from_slice(&le(3));
    body.extend_from_slice(b"xyz");
    h.host(&dframe(0x77, 0, 1, 3, &body));
    h.tick();
    assert_eq!(
        h.got(),
        vec![Got::Terminate(
            DPNERR_HOSTTERMINATEDSESSION,
            b"xyz".to_vec()
        )]
    );
    assert_eq!(h.client.state(), super::core::CoreState::Idle);
}

#[test]
fn connect_failed_0xc5_completes_the_connect_with_the_hosts_code_and_reply() {
    let mut h = start();
    let mut body = vec![0xC5, 0, 0, 0];
    body.extend_from_slice(&le(0x8015_8410)); // INVALIDPASSWORD
    body.extend_from_slice(&le(12));
    body.extend_from_slice(&le(2));
    body.extend_from_slice(&[7, 8]);
    h.host(&dframe(0x77, 0, 0, 2, &body));
    h.tick();
    assert_eq!(
        h.got(),
        vec![Got::Connect(DPNERR_INVALIDPASSWORD, vec![7, 8], 0xC0FFEE)]
    );
    assert_eq!(h.client.state(), super::core::CoreState::Idle);
    // the failed connect leaves the object free to connect again
    assert!(h.client.send(&[b"x"], 0, 0, 0x9).is_err());
}

#[test]
fn a_hard_disconnect_from_the_host_ends_a_connected_session_as_connection_lost() {
    let mut h = connected();
    h.host(&cframe(false, 4, 0, 0, SID, 0));
    h.tick();
    assert_eq!(h.got(), vec![Got::Terminate(DPNERR_CONNECTIONLOST, vec![])]);
    // sends now fail
    assert_eq!(h.client.send(&[b"x"], 0, 0, 0x9), Err(DPNERR_NOCONNECTION));
}

#[test]
fn send_errors_before_connect_and_while_connecting() {
    let mut h = H::new();
    assert_eq!(h.client.send(&[b"x"], 0, 0, 0x9), Err(DPNERR_NOCONNECTION));
    let mut h = start();
    assert_eq!(h.client.send(&[b"x"], 0, 0, 0x9), Err(DPNERR_CONNECTING));
}

#[test]
fn complete_on_process_wraps_in_0xe0_and_finishes_on_0xe1() {
    let mut h = connected();
    let hd = h.client.send(&[b"PQ"], 0, 0x77, 0x9 | 0x4).unwrap();
    h.tick();
    let sent = h.sent();
    // 0xE0, the op handle as context, then the payload; reliable USER_1 (0x40): 0x7F.
    let mut want = vec![0xE0, 0, 0, 0];
    want.extend_from_slice(&le(hd.0));
    want.extend_from_slice(b"PQ");
    assert_eq!(sent, vec![dframe(0x7F, 0, 3, 1, &want)]);
    // Acked, but not complete until the host's 0xE1 names the handle.
    h.host(&[0x80, 0x06, 0x01, 0x00, 0x01, 0x04, 0, 0, 0, 0, 0, 0]);
    h.tick();
    assert!(h.got().is_empty());
    let mut e1 = vec![0xE1, 0, 0, 0];
    e1.extend_from_slice(&le(hd.0));
    h.host(&dframe(0x77, 0, 1, 4, &e1));
    h.tick();
    let got = h.got();
    assert!(
        matches!(got.as_slice(), [Got::Send(r, 0x77, _)] if *r == Hresult(0)),
        "{got:?}"
    );
}

#[test]
fn an_0xe0_request_is_delivered_and_return_buffer_answers_with_0xe1() {
    let mut h = connected();
    let mut e0 = vec![0xE0, 0, 0, 0];
    e0.extend_from_slice(&le(0x55AA));
    e0.extend_from_slice(b"data");
    h.host(&dframe(0x77, 0, 1, 3, &e0));
    h.tick();
    let got = h.got();
    let Got::Receive(d, buf) = got[0].clone() else {
        panic!("{got:?}")
    };
    assert_eq!(d, b"data");
    h.sent();
    assert_eq!(h.client.return_buffer(buf, 0), Hresult(0));
    h.tick();
    let sent = h.sent();
    let mut want = vec![0xE1, 0, 0, 0];
    want.extend_from_slice(&le(0x55AA));
    assert!(sent.iter().any(|d| d[4..] == want[..]), "{sent:?}");
}

#[test]
fn cancel_only_reaches_a_send_that_has_not_started() {
    // Initialize flags 0: link tuning on, a window of 2 frames (flag 4 would fix it at 63).
    let mut h = start_with(0);
    h.host(&dframe(
        0x77,
        0,
        0,
        2,
        &[0xC2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ));
    h.tick();
    h.sent();
    h.got();
    // the 0xC3 (seq 2) is still unacknowledged after the handshake: acknowledge it first
    h.host(&[0x80, 0x06, 0x01, 0x00, 0x01, 0x03, 0, 0, 0, 0, 0, 0]);
    h.tick();
    let a = h.client.send(&[b"1"], 0, 1, 0x9).unwrap();
    let b = h.client.send(&[b"2"], 0, 2, 0x9).unwrap();
    let c = h.client.send(&[b"3"], 0, 3, 0x9).unwrap();
    h.tick(); // window of 2: 1 and 2 go out, 3 waits
    assert_eq!(h.client.cancel_async_operation(b, 4), Hresult(0x8015_8110));
    assert_eq!(h.client.cancel_async_operation(c, 4), Hresult(0));
    h.tick();
    let got = h.got();
    assert_eq!(got.len(), 1);
    assert!(
        matches!(got[0], Got::Send(r, 3, _) if r == DPNERR_USERCANCEL),
        "{got:?}"
    );
    let _ = a;
}

#[test]
fn thread_pool_do_work_drives_the_client() {
    use super::client::ThreadPool;
    let mut h = start();
    let mut tp = ThreadPool::new();
    assert_eq!(tp.initialize(0), Hresult(0));
    assert_eq!(tp.initialize(0), DPNERR_ALREADYINITIALIZED);
    assert_eq!(tp.set_thread_count(0xFFFF_FFFF, 0, 0), Hresult(0));
    assert_eq!(tp.get_thread_count(), 0);
    h.host(&dframe(
        0x77,
        0,
        0,
        2,
        &[0xC2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ));
    h.clock.advance(20);
    assert_eq!(tp.do_work(&mut h.client, u32::MAX, 0), Hresult(0));
    assert_eq!(h.got(), vec![Got::Connect(Hresult(0), vec![], 0xC0FFEE)]);
}
