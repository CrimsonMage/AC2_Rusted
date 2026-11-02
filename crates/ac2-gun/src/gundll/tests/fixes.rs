//! Behaviours of the fix round that need the fake DP8: cancellation calls, the thread pool singleton,
//! the Q1 drain at disconnect, the session key after a failed export, and the 16-bit arithmetic of the
//! send and receive paths.

use std::rc::Rc;

use super::fakes::*;
use crate::gundll::blob;
use crate::gundll::crypto::build_public_key_blob;
use crate::gundll::dp8::*;
use crate::gundll::transport::NetEvent;
use crate::gundll::{CGun, GunEnv, SharedThreadPool};
use crate::interfaces::*;
use crate::zt::*;
use crate::Hresult;

fn connected() -> Rig {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.gun.connect("h", 1);
    r
}

fn transport(r: &Rig) -> Rc<crate::gundll::transport::Transport> {
    r.gun
        .st
        .borrow()
        .conn
        .as_ref()
        .unwrap()
        .transport
        .clone()
        .unwrap()
}

#[test]
fn every_connect_cancels_enum_operations_first() {
    let r = connected();
    // FUN_1003dbd0: S.vtable[13] = CancelAsyncOperation(0, DPNCANCEL_ENUM)
    assert_eq!(r.dp8.0.borrow().cancels, vec![(0, DPNCANCEL_ENUM)]);
    r.gun.connect("h2", 2);
    assert_eq!(r.dp8.0.borrow().cancels, vec![(0, 2), (0, 2)]);
}

#[test]
fn disconnect_cancels_a_pending_connect_handle_only() {
    // A+0x20 (the connect handle) is never set on this path: no cancel; the client is closed
    // (three Close calls, see disconnect_resets_the_client_before_it_drains_q1)
    let r = connected();
    r.gun.disconnect();
    assert_eq!(r.dp8.0.borrow().closed, 3);
    assert_eq!(
        r.dp8.0.borrow().cancels.len(),
        1,
        "only the connect-time ENUM cancel"
    );
}

#[test]
fn disconnect_resets_the_client_before_it_drains_q1() {
    // FUN_100038f0 -> FUN_10003ae0(A, 0): S.vtable[8] = FUN_1003d600 (Close, Release, CoCreate,
    // Initialize 4) first; FUN_10003570 drains Q1; S.vtable[15] = FUN_1003e160 -> FUN_1003dfa0 resets
    // again (FUN_1003d600) and then Close(0)s the newest client
    let r = connected();
    let before = r.dp8.0.borrow().log.len();
    assert_eq!(r.gun.disconnect(), Hresult::S_OK);
    let s = r.dp8.0.borrow();
    assert_eq!(
        s.log[before..].to_vec(),
        vec![
            "close",
            "create_client",
            "initialize",
            "close",
            "create_client",
            "initialize",
            "close"
        ]
    );
    assert_eq!(
        s.init_flags,
        vec![4, 4, 4],
        "every client is Initialize(.., 4)"
    );
}

#[test]
fn disconnect_raises_a_disconnect_ev_on_the_next_dispatch() {
    // FUN_1003b440 puts a TERMINATE_SESSION(DPNERR_USERCANCEL 0x80158650) into Q1; the drain runs it
    // through sink slot 7 (0x10003660, no shutting-down test) -> H.OnDisconnected(0xC112040D) ->
    // Q2 {2, H, hr}; FUN_10009e10 type 2 queues CMsg_DisconnectEv (0x1301) and sets the status 3
    let r = connected();
    assert_eq!(r.gun.disconnect(), Hresult::S_OK);
    let sink = Sink::default();
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.ids(), vec![0x1301]);
    let mut st = ConnectStatus::Created;
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Disconnected);
    // the event is delivered once
    let sink = Sink::default();
    r.gun.dispatch_queue(&sink);
    assert!(sink.ids().is_empty());
}

#[test]
fn a_stale_disconnect_event_does_not_free_a_newer_connection() {
    // FUN_10009e10 frees `conn` only when it is the H of the event
    let r = connected();
    r.gun.disconnect();
    assert_eq!(r.gun.connect("h2", 2), Hresult::E_PENDING);
    let sink = Sink::default();
    r.gun.dispatch_queue(&sink);
    assert_eq!(sink.ids(), vec![0x1301]);
    assert!(
        r.gun.has_conn(),
        "the new connection survives the old one's event"
    );
}

#[test]
fn disconnect_drains_q1_through_the_dispatcher() {
    // a datagram DP8 already delivered still becomes a message after Disconnect; the connect callback
    // is skipped (A+0xe, 0x100035fb) but the terminate callback is not (sink slot 7 has no test):
    // Q1 is [datagram, DP8's terminate, the injected USERCANCEL]; the first terminate unlinks H, the
    // second finds `A+4 == NULL`
    let r = connected();
    let ev = |u: &str| format!("<FriendStat User=\"{u}\" Mode=\"1\" />");
    r.dp8.push_datagram(datagram(&ev("late")));
    r.dp8.push(Dp8Message::TerminateSession {
        hresult: DPNERR_NOCONNECTION,
        data: vec![],
    });
    r.dp8.push(Dp8Message::ConnectComplete {
        hresult: Hresult::S_OK,
        reply: vec![],
    });
    r.dp8.do_work(u32::MAX, 0);
    assert_eq!(r.gun.disconnect(), Hresult::S_OK);
    let sink = Sink::default();
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.ids(), vec![0x430d, 0x1301]);
    assert_eq!(r.dp8.0.borrow().returned, vec![99], "the buffer went back");
}

#[test]
fn a_disconnect_made_while_q1_drains_is_deferred() {
    // FUN_10003b40 sets A+0xc while it reads Q1; FUN_10003ae0 then resets the client at once but only
    // sets A+0xd; the drain calls FUN_10003570 when the queue is empty
    let r = connected();
    let t = transport(&r);
    let events = Rc::new(std::cell::RefCell::new(Vec::new()));
    {
        let t = t.clone();
        let events = events.clone();
        *r.dp8.0.borrow_mut().on_return_buffer.borrow_mut() = Some(Rc::new(move || {
            events.borrow_mut().extend(t.close());
        }));
    }
    r.dp8
        .push_datagram(datagram("<FriendStat User=\"x\" Mode=\"1\" />"));
    r.dp8.do_work(u32::MAX, 0);
    let closed_before = r.dp8.0.borrow().closed;
    let drained = t.drain();
    // the close() made inside the callback had nothing to report itself
    assert!(events.borrow().is_empty());
    // message first, then the injected terminate (the loop kept reading), then the deferred
    // FUN_10003570 ran: reset (1 Close) + destructor (2 Close) = 3
    assert_eq!(drained.len(), 2);
    assert!(matches!(drained[0], NetEvent::Message(_)));
    assert_eq!(
        drained[1],
        NetEvent::Disconnected(ZT_E_CONNECTION_CLIENTDISCONNECT)
    );
    assert_eq!(r.dp8.0.borrow().closed - closed_before, 3);
    assert!(t.is_destroyed());
    // a second close does nothing more
    assert!(t.close().is_empty());
    assert_eq!(r.dp8.0.borrow().closed - closed_before, 3);
}

#[test]
fn a_terminate_destroys_the_sink_object_itself() {
    // sink slot 7 calls A.vtable[0](A, 1): no reset first, and (deferred out of the drain) FUN_10003570
    // + S's destructor: reset (Close, CoCreate, Initialize) and the final Close
    let r = connected();
    let t = transport(&r);
    let before = r.dp8.0.borrow().log.len();
    r.dp8.push(Dp8Message::TerminateSession {
        hresult: DPNERR_HOSTTERMINATEDSESSION,
        data: vec![],
    });
    // a datagram behind the terminate: `A+4` is NULL, sink slot 0 drops it
    r.dp8
        .push_datagram(datagram("<FriendStat User=\"z\" Mode=\"1\" />"));
    r.dp8.do_work(u32::MAX, 0);
    let events = t.drain();
    assert_eq!(
        events,
        vec![NetEvent::Disconnected(ZT_E_CONNECTION_SERVERBOOT)]
    );
    assert!(t.is_destroyed());
    assert_eq!(
        r.dp8.0.borrow().log[before..].to_vec(),
        vec!["close", "create_client", "initialize", "close"]
    );
    // H+8 was cleared: SendOnConn answers ZT_E_NOT_CONNECTED
    assert_eq!(t.send_text(b"x\0"), ZT_E_NOT_CONNECTED);
}

#[test]
fn a_failed_connect_complete_destroys_the_sink_object_too() {
    let r = connected();
    let before = r.dp8.0.borrow().log.len();
    r.dp8.push(Dp8Message::ConnectComplete {
        hresult: Hresult(0x8015_8030),
        reply: vec![],
    });
    let sink = Sink::default();
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.ids(), vec![0x1302]);
    assert_eq!(
        r.dp8.0.borrow().log[before..].to_vec(),
        vec!["close", "create_client", "initialize", "close"]
    );
    assert!(!r.gun.has_conn(), "FUN_10009e10 type 1 freed H");
}

#[test]
fn the_thread_pool_is_created_by_connect_and_shared_through_its_slot() {
    // DAT_10069870: created and set to zero threads by the first Connect (FUN_10035ae0 ->
    // FUN_10003810), once, for every CGun that shares it; Init creates nothing
    let dp8 = Dp8::default();
    let slot = SharedThreadPool::new();
    for i in 0..2 {
        let mut env = GunEnv::new(Rc::new(dp8.clone()), Rc::new(Reg::default()));
        env.pool = slot.clone();
        let g = CGun::new(env);
        assert_eq!(g.init(&APP_GUID, "1.0", 1, "k", 0), Hresult::S_OK);
        assert_eq!(
            dp8.0.borrow().pool_created,
            i,
            "Init never creates the pool"
        );
        let _ = g.connect("h", 1);
    }
    assert_eq!(dp8.0.borrow().pool_created, 1);
    assert_eq!(dp8.0.borrow().thread_counts, vec![(u32::MAX, 0, 0)]);
    // a private slot makes its own
    let mut env = GunEnv::new(Rc::new(dp8.clone()), Rc::new(Reg::default()));
    env.pool = SharedThreadPool::new();
    let g = CGun::new(env);
    g.init(&APP_GUID, "1.0", 1, "k", 0);
    let _ = g.connect("h", 1);
    assert_eq!(dp8.0.borrow().pool_created, 2);
}

#[test]
fn a_failed_pool_creation_is_retried_by_the_next_connect() {
    // CGun::Init succeeds and so does Connect; FUN_10003810 tries again while DAT_10069870 is NULL
    let r = rig();
    r.dp8.0.borrow_mut().pool_fails = true;
    assert_eq!(r.gun.init(&APP_GUID, "1.0", 1033, "k", 0), Hresult::S_OK);
    assert_eq!(r.gun.connect("h", 1), Hresult::E_PENDING);
    assert_eq!(r.dp8.0.borrow().pool_created, 0);
    let sink = Sink::default();
    r.gun.dispatch_queue(&sink);
    assert!(r.dp8.0.borrow().do_work.is_empty(), "no pool, no DoWork");
    r.dp8.0.borrow_mut().pool_fails = false;
    assert_eq!(r.gun.connect("h", 1), Hresult::E_PENDING);
    assert_eq!(r.dp8.0.borrow().pool_created, 1);
    r.gun.dispatch_queue(&sink);
    assert_eq!(r.dp8.0.borrow().do_work, vec![(u32::MAX, 0)]);
}

#[test]
fn connect_complete_reply_data_does_not_change_the_mapping() {
    let r = connected();
    r.dp8.push(Dp8Message::ConnectComplete {
        hresult: Hresult::S_OK,
        reply: vec![1, 2, 3],
    });
    let sink = Sink::default();
    r.gun.dispatch_queue(&sink);
    assert_eq!(r.dp8.sent_count(), 1, "<Connect> went out");
}

fn pubkey_blob(modulus_be: &[u8]) -> String {
    blob::encode_for_attribute(&build_public_key_blob(modulus_be, 0x10001)).unwrap()
}

#[test]
fn a_failed_export_clears_the_previous_session_key() {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.gun.connect("h", 1);
    let sink = Sink::default();
    r.dp8.push(Dp8Message::ConnectComplete {
        hresult: Hresult::S_OK,
        reply: vec![],
    });
    r.gun.dispatch_queue(&sink);
    let cx = r.gun.connect_cx();
    let res = |modulus: &[u8]| {
        format!(
            "<ConnectRes Cx=\"0x{cx:x}\" HR=\"0x0\" Random=\"5\" ><RSAKey Val=\"{}\"/></ConnectRes> ",
            pubkey_blob(modulus)
        )
    };
    let mut n = vec![0xC5u8; 64];
    n[63] = 0xB1;
    r.dp8.push_datagram(datagram(&res(&n)));
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert!(r.gun.st.borrow().session_key.is_some());
    // a CALG_RSA_SIGN key (aiKeyAlg 0x2400): the import works, it is no key exchange key, the
    // export fails
    let sign = blob::encode_for_attribute(&crate::gundll::crypto::build_public_key_blob_alg(
        &n, 0x10001, 0x2400,
    ))
    .unwrap();
    r.dp8.push_datagram(datagram(&format!(
        "<ConnectRes Cx=\"0x{cx:x}\" HR=\"0x0\" Random=\"5\" ><RSAKey Val=\"{sign}\"/></ConnectRes> "
    )));
    let hr = r.gun.dispatch_queue(&sink);
    assert_eq!(hr, Hresult(1), "the handler's failure stops the dispatch");
    assert!(
        r.gun.st.borrow().session_key.is_none(),
        "CGun+0x80 is cleared when only the export failed"
    );
}

#[test]
fn the_send_total_wraps_like_retail() {
    let r = connected();
    let t = transport(&r);
    // FUN_10042af0: max(2 + len, 0x578) + 12 >= 0x10000 is E_INVALIDARG
    assert_eq!(t.send_text(&vec![b'x'; 0xFFF1]), Hresult::S_OK);
    assert_eq!(t.send_text(&vec![b'x'; 0xFFF2]), Hresult(0x8007_0057));
    // 0xFFFE and 0xFFFF bytes wrap the 16-bit total to 0 and 1: retail overflows its pool block
    // there (open); the port rejects
    assert_eq!(t.send_text(&vec![b'x'; 0xFFFE]), Hresult(0x8007_0057));
    assert_eq!(t.send_text(&vec![b'x'; 0xFFFF]), Hresult(0x8007_0057));
    assert_eq!(r.dp8.sent_count(), 1);
    // 0x10000 truncates to 0 in the u16 length: only the type word is sent
    assert_eq!(t.send_text(&vec![b'x'; 0x1_0000]), Hresult::S_OK);
    assert_eq!(r.dp8.0.borrow().sent[1].0, vec![0x50, 0xC3]);
}

#[test]
fn control_frame_sizes_are_sixteen_bit() {
    let r = connected();
    let sink = Sink::default();
    // a type 1 frame whose 8 + a + b is 0x10000 consumes 0 (u16): retail spins, the port stops
    let mut d = vec![1, 0, 0, 0, 0xF8, 0xFF, 0, 0];
    d.extend_from_slice(&datagram("<FriendStat User=\"x\" Mode=\"1\" />"));
    r.dp8.push_datagram(d);
    // a datagram longer than 64 KiB is cut to its low 16 bits of length
    let mut big = datagram("<FriendStat User=\"y\" Mode=\"1\" />");
    big.resize(0x1_0000 + 4, b' ');
    r.dp8.push_datagram(big);
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    // the first was dropped; the second's length is 4 bytes: type + 2 text bytes, no NUL: dropped
    assert!(sink.ids().is_empty());
    assert_eq!(r.dp8.0.borrow().returned.len(), 2);
}

fn friend_stat(u: &str) -> Vec<u8> {
    datagram(&format!("<FriendStat User=\"{u}\" Mode=\"1\" />"))
}

#[test]
fn a_type_0_control_frame_reads_the_connection_info() {
    // FUN_1003b820 type 0 -> FUN_1003b4a0 -> FUN_1003cfc0: GetConnectionInfo(dwSize 0x5c); S+0x28 =
    // (S+0x28 + dwRoundTripLatencyMS) >> 1; the frame is 8 bytes and the datagram goes on
    let r = connected();
    let t = transport(&r);
    r.dp8.0.borrow_mut().rtt = Some(100);
    let mut d = vec![0, 0, 0, 0, 1, 2, 3, 4];
    d.extend_from_slice(&friend_stat("a"));
    r.dp8.push_datagram(d.clone());
    r.dp8.0.borrow_mut().rtt = Some(100);
    let sink = Sink::default();
    r.gun.dispatch_queue(&sink);
    assert_eq!(
        sink.ids(),
        vec![0x430d],
        "the frame consumed exactly 8 bytes"
    );
    assert_eq!(r.dp8.0.borrow().conn_info_sizes, vec![0x5c]);
    assert_eq!(t.latency_average(), 50, "(0 + 100) >> 1");
    assert!(t.latency_sample().is_some());
    r.dp8.0.borrow_mut().rtt = Some(30);
    r.dp8.push_datagram(d.clone());
    r.gun.dispatch_queue(&sink);
    assert_eq!(t.latency_average(), 40, "(50 + 30) >> 1");
    // a failing GetConnectionInfo changes nothing
    r.dp8.0.borrow_mut().rtt = None;
    r.dp8.push_datagram(d);
    r.gun.dispatch_queue(&sink);
    assert_eq!(t.latency_average(), 40);
    assert_eq!(r.dp8.0.borrow().conn_info_sizes.len(), 3);
}

#[test]
fn a_type_1_control_frame_sets_the_keep_alive_and_is_8_plus_a_plus_b() {
    // FUN_1003b820 type 1 -> FUN_1003b4f0 -> FUN_1003da50: subtype byte [+2] 1 -> keep-alive
    // 0xFFFFFFFF, 2 -> 0x3A98 (15000), else none; the frame is ((u16[+4] + u16[+6]) & 0xFFFF) + 8
    let r = connected();
    let sink = Sink::default();
    let frame = |subtype: u8, a: u16, b: u16, len: usize| {
        let mut d = vec![1, 0, subtype, 0];
        d.extend_from_slice(&a.to_le_bytes());
        d.extend_from_slice(&b.to_le_bytes());
        d.resize(len, 0xEE);
        d.extend_from_slice(&friend_stat("k"));
        d
    };
    let keepalives = |r: &Rig| {
        r.dp8
            .0
            .borrow()
            .caps_log
            .iter()
            .filter(|l| l.starts_with("set "))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(keepalives(&r), vec!["set keepalive=15000"], "from Connect");
    r.dp8.push_datagram(frame(1, 2, 3, 13));
    r.gun.dispatch_queue(&sink);
    assert_eq!(sink.ids(), vec![0x430d]);
    assert_eq!(keepalives(&r).last().unwrap(), "set keepalive=4294967295");
    r.dp8.push_datagram(frame(2, 0, 0, 8));
    r.gun.dispatch_queue(&sink);
    assert_eq!(keepalives(&r).last().unwrap(), "set keepalive=15000");
    let n = keepalives(&r).len();
    r.dp8.push_datagram(frame(3, 4, 0, 12));
    r.gun.dispatch_queue(&sink);
    assert_eq!(keepalives(&r).len(), n, "subtype 3 changes nothing");
    assert_eq!(sink.ids(), vec![0x430d; 3], "every frame had its size");
    // (0xFFFC + 8) & 0xFFFF = 4, + 8 = 12 bytes
    r.dp8.push_datagram(frame(3, 0xFFFC, 8, 12));
    r.gun.dispatch_queue(&sink);
    assert_eq!(sink.ids(), vec![0x430d; 4]);
}

#[test]
fn public_key_blob_import_follows_the_base_provider() {
    use crate::gundll::crypto::{build_public_key_blob_alg, CryptContext};
    // [ms-docs: base-provider-algorithms.md] exchange keys 384..1024 bits, signature keys
    // 384..16384; [cryptimportkey-errors.md] NTE_BAD_TYPE 0x8009000A, NTE_BAD_VER 0x80090007,
    // NTE_BAD_DATA 0x80090005
    let modulus = |bits: usize| {
        let mut n = vec![0xC5u8; bits / 8];
        n[bits / 8 - 1] |= 1;
        n
    };
    let import = |blob: &[u8]| CryptContext::new().import_public_key(blob);
    let keyx = |bits| build_public_key_blob_alg(&modulus(bits), 0x10001, 0xa400);
    assert!(import(&keyx(384)).succeeded());
    assert!(import(&keyx(512)).succeeded());
    assert!(import(&keyx(1024)).succeeded());
    assert_eq!(import(&keyx(376)), Hresult(0x8009_0005));
    assert_eq!(import(&keyx(1032)), Hresult(0x8009_0005));
    let sign = |bits| build_public_key_blob_alg(&modulus(bits), 0x10001, 0x2400);
    assert!(import(&sign(2048)).succeeded());
    assert_eq!(import(&sign(16392)), Hresult(0x8009_0005));
    assert_eq!(
        import(&build_public_key_blob_alg(&modulus(512), 0x10001, 0x6801)),
        Hresult(0x8009_0005),
        "an algorithm the CSP does not support"
    );
    let mut b = keyx(512);
    b[0] = 7;
    assert_eq!(import(&b), Hresult(0x8009_000A));
    let mut b = keyx(512);
    b[1] = 3;
    assert_eq!(import(&b), Hresult(0x8009_0007));
}

#[test]
fn a_synchronous_connect_failure_runs_the_destroy_sequence() {
    // FUN_1003d6d0 0x1003d9f9: S.vtable[8] (Close #1, CoCreate, Initialize); FUN_10003b90 frees A raw
    // and runs S's destructor (0x10003ce9: reset = Close #2, CoCreate, Initialize; final Close #3)
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.dp8.0.borrow_mut().connect_result = Some(Hresult(0x8015_8030));
    let before = r.dp8.0.borrow().log.len();
    assert_eq!(r.gun.connect("h", 1), ZT_E_FAILED_TO_CONNECT);
    let s = r.dp8.0.borrow();
    assert_eq!(
        s.log[before..].to_vec(),
        vec![
            "create_client",
            "initialize",
            "cancel",
            "connect",
            "close",
            "create_client",
            "initialize",
            "close",
            "create_client",
            "initialize",
            "close"
        ]
    );
    assert_eq!(s.closed, 3);
}

#[test]
fn the_type_0_latency_argument_is_stamp_minus_arrival_plus_now() {
    // FUN_1003b4a0: u32[+4] - arrival + timeGetTime(); arrival is stamped by the DP8 handler when the
    // datagram comes in, so with no delay the argument is the frame's stamp (give or take the ms
    // between the handler and the dispatcher)
    let r = connected();
    let t = transport(&r);
    r.dp8.0.borrow_mut().rtt = Some(10);
    let mut d = vec![0, 0, 0, 0];
    d.extend_from_slice(&5000u32.to_le_bytes());
    d.extend_from_slice(&friend_stat("a"));
    r.dp8.push_datagram(d);
    r.gun.dispatch_queue(&Sink::default());
    let avg = t.latency_average();
    let sample = t.latency_sample().unwrap();
    assert_eq!(avg, 5);
    let arg = sample - (avg >> 1);
    assert!((5000..5100).contains(&arg), "{arg}");
}
