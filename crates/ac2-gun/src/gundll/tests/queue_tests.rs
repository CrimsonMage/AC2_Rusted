//! `DispatchQueue` semantics, the datagram framing and re-entrancy (`0a_receive_messages.md` 1.2 and
//! 1.4, with the verifier's framing resolution).

use std::cell::RefCell;
use std::rc::Rc;

use super::fakes::*;
use crate::gundll::queue::{read, Queue, QUEUE_FRONT};
use crate::interfaces::*;
use crate::message::{GunQueueMessage, MessageBody};
use crate::zt::*;
use crate::Hresult;

fn ev(user: &str, mode: u32) -> String {
    format!("<FriendStat User=\"{user}\" Mode=\"{mode}\" />")
}

/// An initialised `CGun` with a (never completed) connection.
fn connected() -> Rig {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.gun.connect("h", 1);
    r
}

fn user_of(m: &GunQueueMessage) -> String {
    match &m.message.body {
        MessageBody::FriendStatus { user_name, .. } => user_name.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn queue_is_fifo_with_a_front_flag() {
    let q = RefCell::new(Queue::default());
    q.borrow_mut().write(0, 1);
    q.borrow_mut().write(0, 2);
    q.borrow_mut().write(QUEUE_FRONT, 3);
    let mut got = vec![];
    while read(&q, |v| {
        got.push(v);
        Hresult::S_OK
    }) == Hresult::S_OK
    {}
    assert_eq!(got, vec![3, 1, 2]);
    // empty: E_PENDING
    assert_eq!(read(&q, |_: i32| Hresult::S_OK), Hresult::E_PENDING);
}

#[test]
fn a_negative_sink_result_makes_read_answer_one() {
    let q = RefCell::new(Queue::default());
    q.borrow_mut().write(0, 1);
    assert_eq!(read(&q, |_| ZT_E_INTERNAL), Hresult(1));
    assert!(q.borrow().is_empty(), "the entry is gone either way");
}

#[test]
fn messages_arrive_in_order_and_the_empty_queue_is_s_ok() {
    let r = connected();
    let sink = Sink::default();
    r.dp8.push_datagram(datagram(&ev("a", 1)));
    r.dp8.push_datagram(datagram(&ev("b", 2)));
    r.dp8.push_datagram(datagram(&ev("c", 3)));
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    let users: Vec<_> = sink.got.borrow().iter().map(user_of).collect();
    assert_eq!(users, ["a", "b", "c"]);
    // every datagram's buffer went back to DP8 (ReturnBuffer)
    assert_eq!(r.dp8.0.borrow().returned, vec![99, 99, 99]);
    // DoWork(-1, 0) ran once
    assert_eq!(r.dp8.0.borrow().do_work, vec![(u32::MAX, 0)]);
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.got.borrow().len(), 3);
}

#[test]
fn a_failing_sink_stops_the_dispatch_with_one_and_keeps_the_rest() {
    let r = connected();
    let sink = Sink::default();
    sink.results
        .borrow_mut()
        .extend([Hresult::S_OK, ZT_E_INTERNAL]);
    for u in ["a", "b", "c"] {
        r.dp8.push_datagram(datagram(&ev(u, 1)));
    }
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult(1));
    assert_eq!(sink.got.borrow().len(), 2);
    // the third is still queued
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.got.borrow().len(), 3);
}

#[test]
fn without_a_connection_the_queue_is_still_read() {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    let sink = Sink::default();
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert!(sink.got.borrow().is_empty());
    assert_eq!(r.dp8.0.borrow().do_work.len(), 0, "no connection, no pump");
}

#[test]
fn the_sink_may_call_connect_from_inside_read() {
    // TurbineGun::RcvConnectEvent reconnects on 0x1301 / 0x1302 from inside Read
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.gun.connect("h", 1);
    let sink = Sink::default();
    let gun = r.gun.clone();
    let results = Rc::new(RefCell::new(vec![]));
    let res2 = results.clone();
    *sink.on_read.borrow_mut() = Some(Box::new(move |m| {
        if m.message.id.0 == 0x1302 {
            res2.borrow_mut().push(gun.connect("h", 1));
        }
        // and the other re-entrant calls of AuthenticateWithGUN
        let _ = gun.get_last_error();
        let mut st = ConnectStatus::Created;
        let _ = gun.get_status(&mut st);
    }));
    r.dp8.push(crate::gundll::dp8::Dp8Message::ConnectComplete {
        hresult: crate::gundll::dp8::DPNERR_HOSTREJECTEDCONNECTION,
        reply: vec![],
    });
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.ids(), vec![0x1302]);
    assert_eq!(*results.borrow(), vec![Hresult::E_PENDING]);
    let mut st = ConnectStatus::Created;
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Connecting);
    assert_eq!(r.dp8.0.borrow().connects.len(), 2);
}

#[test]
fn the_sink_may_send_and_disconnect_from_inside_read() {
    let r = connected();
    let sink = Sink::default();
    let gun = r.gun.clone();
    let results = Rc::new(RefCell::new(vec![]));
    let res2 = results.clone();
    *sink.on_read.borrow_mut() = Some(Box::new(move |_| {
        res2.borrow_mut()
            .push((gun.friend_add("x", Context(1)), gun.disconnect()));
    }));
    r.dp8.push_datagram(datagram(&ev("a", 1)));
    r.dp8.push_datagram(datagram(&ev("b", 1)));
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    // both were already queued before the sink ran; the Disconnect made by the first read queued a
    // DisconnectEv (0x1301, the injected USERCANCEL terminate) behind them, which the same loop reads
    assert_eq!(sink.ids(), vec![0x430d, 0x430d, 0x1301]);
    assert_eq!(
        *results.borrow(),
        vec![
            (Hresult::S_OK, Hresult::S_OK),
            (ZT_E_NOT_CONNECTED, ZT_E_CLOSED),
            (ZT_E_NOT_CONNECTED, ZT_E_CLOSED)
        ]
    );
    assert_eq!(r.dp8.sent_count(), 1, "the second send found no connection");
}

#[test]
fn a_nested_dispatch_zeroes_the_sink_and_the_outer_loop_drops_the_rest() {
    // CGun+0x44 is zeroed when DispatchQueue returns: after an inner call the outer loop has no sink and
    // FUN_1000a300 silently deletes the entries it reads
    let r = connected();
    let sink = Sink::default();
    let gun = r.gun.clone();
    let inner = Rc::new(Sink::default());
    inner.results.borrow_mut().push_back(ZT_E_INTERNAL); // stops the inner loop after one entry
    let inner2 = inner.clone();
    let inner_hr = Rc::new(RefCell::new(None));
    let ih = inner_hr.clone();
    *sink.on_read.borrow_mut() = Some(Box::new(move |_| {
        *ih.borrow_mut() = Some(gun.dispatch_queue(inner2.as_ref()));
    }));
    for u in ["a", "b", "c"] {
        r.dp8.push_datagram(datagram(&ev(u, 1)));
    }
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(
        sink.got.borrow().len(),
        1,
        "only the first reached the outer sink"
    );
    assert_eq!(
        inner.got.borrow().len(),
        1,
        "the inner loop read one and stopped"
    );
    assert_eq!(*inner_hr.borrow(), Some(Hresult(1)));
    // the third was read by the outer loop after the inner call and deleted: nothing is left
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.got.borrow().len(), 1);
}

// ---- framing (0a_receive_messages.md, framing resolution) ----

#[test]
fn an_application_frame_consumes_the_rest_of_the_datagram() {
    let r = connected();
    let sink = Sink::default();
    // one message per datagram: text up to the first NUL; anything after it is ignored
    let mut d = datagram(&ev("a", 1));
    d.extend_from_slice(&datagram(&ev("b", 2)));
    r.dp8.push_datagram(d);
    r.gun.dispatch_queue(&sink);
    let users: Vec<_> = sink.got.borrow().iter().map(user_of).collect();
    assert_eq!(users, ["a"]);
}

#[test]
fn control_frames_carry_their_own_sizes() {
    let r = connected();
    let sink = Sink::default();
    // type 0: 8 bytes; type 1: 8 + u16[4] + u16[6] bytes; then the application frame
    let mut d = vec![0, 0, 0, 0, 0, 0, 0, 0];
    d.extend_from_slice(&[1, 0, 1, 0, 3, 0, 2, 0, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
    d.extend_from_slice(&datagram(&ev("a", 1)));
    r.dp8.push_datagram(d);
    r.gun.dispatch_queue(&sink);
    assert_eq!(sink.got.borrow().len(), 1);
    // any type >= 1000 is an application frame (the sender uses 0xC350)
    let mut d = 1000u16.to_le_bytes().to_vec();
    d.extend_from_slice(ev("b", 1).as_bytes());
    d.push(0);
    r.dp8.push_datagram(d);
    r.gun.dispatch_queue(&sink);
    assert_eq!(sink.got.borrow().len(), 2);
}

#[test]
fn malformed_datagrams_are_dropped_without_a_trace() {
    let r = connected();
    let sink = Sink::default();
    // no NUL: E_INVALIDARG from OnReceive, ignored
    let mut d = vec![0x50, 0xC3];
    d.extend_from_slice(ev("a", 1).as_bytes());
    r.dp8.push_datagram(d);
    // unknown element
    r.dp8.push_datagram(datagram("<Bogus />"));
    // a parse failure (missing HR)
    r.dp8.push_datagram(datagram("<FriendAddRes Cx=\"0x1\" />"));
    // an unsupported control type (2..998): retail spins on it; the port drops the datagram
    r.dp8.push_datagram(vec![2, 0, 1, 2, 3]);
    // a one-byte datagram
    r.dp8.push_datagram(vec![0x50]);
    r.dp8.push_datagram(datagram(&ev("ok", 1)));
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    let users: Vec<_> = sink.got.borrow().iter().map(user_of).collect();
    assert_eq!(users, ["ok"]);
    assert_eq!(
        r.dp8.0.borrow().returned.len(),
        6,
        "every buffer is returned"
    );
}

#[test]
fn receive_returns_pending_and_other_ids_queue_nothing() {
    use crate::gundll::dp8::*;
    use crate::gundll::transport::Dp8Handler;
    use std::collections::VecDeque;
    let q = Rc::new(RefCell::new(VecDeque::new()));
    let notify = Rc::new(RefCell::new(std::collections::HashSet::new()));
    let h = Dp8Handler::new(q.clone(), notify.clone());
    assert_eq!(
        h.message(&Dp8Message::Receive {
            data: vec![1],
            buffer_handle: 1
        }),
        DPNSUCCESS_PENDING
    );
    assert_eq!(
        h.message(&Dp8Message::ReturnBuffer { context: 1 }),
        Hresult::S_OK
    );
    assert_eq!(
        h.message(&Dp8Message::Other { id: 0xFFFF_0001 }),
        Hresult::S_OK
    );
    // ASYNC_OP_COMPLETE is not queued (jump-table index 0 lands after the Q1 Write)
    assert_eq!(h.message(&Dp8Message::AsyncOpComplete), Hresult::S_OK);
    // SEND_COMPLETE only when the send context's notify byte is set (ctx->byte[8] != 0)
    let done = |ctx| Dp8Message::SendComplete {
        context: ctx,
        hresult: Hresult::S_OK,
    };
    assert_eq!(h.message(&done(1)), Hresult::S_OK);
    assert_eq!(q.borrow().len(), 1, "only the RECEIVE so far");
    notify.borrow_mut().insert(2);
    assert_eq!(h.message(&done(2)), Hresult::S_OK);
    assert_eq!(q.borrow().len(), 2);
    assert!(notify.borrow().is_empty(), "the context is freed");
}

#[test]
fn the_wire_body_is_c350_text_and_nul_in_one_guaranteed_send() {
    let r = connected();
    r.gun.friend_add("Bob", Context(0x10));
    r.gun.friend_add("Al", Context(0x11));
    let s = r.dp8.0.borrow();
    assert_eq!(s.sent.len(), 2, "one DP8 send per message");
    for (buf, timeout, flags) in &s.sent {
        assert_eq!(&buf[..2], &50000u16.to_le_bytes());
        assert_eq!(*buf.last().unwrap(), 0);
        assert_eq!((*timeout, *flags), (0, 0x8 | 0x1));
    }
}

#[test]
fn an_oversized_message_is_rejected_before_the_send() {
    let r = connected();
    // FUN_10042af0: max(total, 0x578) + 12 >= 0x10000 is E_INVALIDARG
    let big = "x".repeat(0xFFF2 - 40);
    assert_eq!(
        r.gun.send_to_room(1, &big, Context(1)),
        Hresult(0x8007_0057)
    );
    assert_eq!(r.dp8.sent_count(), 0);
    // a message over 0xFFFF characters cannot be rendered at all
    let huge = "x".repeat(0x1_0000);
    assert_eq!(r.gun.send_to_room(1, &huge, Context(1)), ZT_E_STR_TOO_LONG);
}
