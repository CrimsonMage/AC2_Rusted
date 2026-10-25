//! Soak runs of the client transport over a seeded lossy network: datagrams in both directions
//! dropped, duplicated and held back (so they arrive out of order). Thousands of blobs each way
//! must arrive exactly once and intact, and every datagram the client sent is checked by the
//! oracle (`oracle/mod.rs`): sequences, keys, resends byte for byte, ACK/NAK cadence and
//! contents, time-sensitive payloads on the clock the server's time syncs set.

mod harness;
mod oracle;

use std::collections::BTreeMap;

use ac2_net::{NetEvent, OptionalHeader};
use harness::{lossy, payload, Session, STEP};
use legions_server_double::Conditions;

struct Run {
    session: Session,
    /// Blobs the client received, in order: id and bytes.
    received: Vec<(u64, Vec<u8>)>,
    server_sent: BTreeMap<u64, Vec<u8>>,
    client_sent: BTreeMap<u64, Vec<u8>>,
    /// Time syncs the client processed.
    time_syncs: u32,
}

/// Runs a session: `traffic` seconds of blobs each way in bursts of five seconds with three
/// idle (the server one every other frame, a multi-fragment one every 40th; the client one
/// every third frame, a multi-fragment one every 64th) and a server time sync every five
/// seconds, then up to `drain` seconds for everything to land.
fn run(seed: u64, conditions: Conditions, traffic: f64, drain: f64) -> Run {
    let mut session = Session::new(seed, conditions);
    session.log_on();
    let mut received = Vec::new();
    let mut server_sent = BTreeMap::new();
    let mut client_sent = BTreeMap::new();
    let mut time_syncs = 0;
    for n in 0u32.. {
        let at = f64::from(n) * STEP;
        let sending = at < traffic && at % 8.0 < 5.0;
        let state = &session.state;
        if n % 320 == 0 && at < traffic {
            // The server clock: 5000 at the connect.
            let time = 5000.0 + session.now();
            state
                .borrow_mut()
                .reliable_header(OptionalHeader::TimeSync(time));
        }
        if sending && n % 2 == 0 {
            let large = n % 40 == 0;
            let data = payload(n, if large { 2000 } else { 12 + (n as usize % 50) });
            let id = if large {
                state.borrow_mut().send_fragmented(&data)
            } else {
                state.borrow_mut().send_event(&data)
            };
            server_sent.insert(id, data);
        }
        if sending && n % 3 == 0 {
            let large = n % 64 == 0;
            let data = payload(n, if large { 1500 } else { 8 + (n as usize % 40) });
            let id = session.client.send(4, data.clone()).expect("queued");
            client_sent.insert(id, data);
        }
        for event in session.frame() {
            match event {
                // Not the login's `CharGenVerification` and `CharacterSet` (event type 0x23).
                NetEvent::Blob(blob) if blob.id >> 56 != 0x23 => {
                    received.push((blob.id, blob.data))
                }
                NetEvent::TimeSync(_) => time_syncs += 1,
                NetEvent::Disconnected(error) => panic!("{at}: disconnected: {error}"),
                _ => {}
            }
        }
        let settled = at >= traffic
            && received.len() == server_sent.len()
            && session.state.borrow().blobs.len() >= client_sent.len();
        if settled || at > traffic + drain {
            break;
        }
    }
    Run {
        session,
        received,
        server_sent,
        client_sent,
        time_syncs,
    }
}

/// Every blob arrived exactly once, intact, in both directions.
fn assert_delivered(run: &Run) {
    let mut seen = BTreeMap::new();
    for (id, data) in &run.received {
        assert!(
            seen.insert(*id, data).is_none(),
            "blob {id:#x} delivered twice"
        );
    }
    assert_eq!(seen.len(), run.server_sent.len(), "server blobs delivered");
    for (id, data) in &run.server_sent {
        assert_eq!(
            seen.get(id).map(|d| d.as_slice()),
            Some(data.as_slice()),
            "{id:#x}"
        );
    }
    let state = run.session.state.borrow();
    assert_eq!(state.assembler.duplicate_fragments, 0);
    let got: BTreeMap<u64, &Vec<u8>> = state
        .blobs
        .iter()
        .filter(|(_, queue, _)| *queue == 4)
        .map(|(id, _, data)| (*id, data))
        .collect();
    assert_eq!(got.len(), run.client_sent.len(), "client blobs delivered");
    for (id, data) in &run.client_sent {
        assert_eq!(got.get(id), Some(&data), "{id:#x}");
    }
}

#[test]
fn a_clean_network_needs_no_resends() {
    let none = Conditions {
        drop: 0.0,
        duplicate: 0.0,
        delay: 0.0,
        max_delay: 0.0,
    };
    let run = run(1, none, 10.0, 10.0);
    assert_delivered(&run);
    let counts = oracle::check(&run.session.frames());
    assert_eq!((counts.resends, counts.naks, counts.empty_acks), (0, 0, 0));
    assert!(counts.acks > 0 && counts.reliable > 100, "{counts:?}");
}

#[test]
fn a_lossy_reordering_duplicating_network_delivers_everything_exactly_once() {
    for seed in [7, 0x5EED, 0xAC2] {
        let run = run(seed, lossy(), 120.0, 60.0);
        assert_delivered(&run);
        let counts = oracle::check(&run.session.frames());
        let state = run.session.state.borrow();
        let stats = state.channel.as_ref().unwrap().stats;
        // About 4,700 datagrams through the channel, a tenth of them lost.
        assert!(stats.dropped > 400 && stats.duplicated > 200 && stats.delayed > 400);
        assert!(
            counts.resends > 150 && counts.naks > 100,
            "{seed}: {counts:?}"
        );
        // ACKs go in the idle stretches, when nothing is missing.
        assert!(counts.acks > 10, "{seed}: {counts:?}");
        assert!(state.resent.len() > 100, "the double resent too");
        // The server clock was adopted: about 5000 s ahead of the client's.
        assert!(run.time_syncs >= 20, "{}", run.time_syncs);
        let base = run.session.client.net().timer().base_time();
        assert!((4999.0..5001.0).contains(&base), "{base}");
    }
}

#[test]
fn heavy_loss_still_converges() {
    let heavy = Conditions {
        drop: 0.3,
        duplicate: 0.1,
        delay: 0.2,
        max_delay: 1.0,
    };
    let run = run(99, heavy, 30.0, 120.0);
    assert_delivered(&run);
    let counts = oracle::check(&run.session.frames());
    assert!(counts.resends > 100, "{counts:?}");
}

#[test]
fn stale_naks_after_long_holds_are_answered_with_empty_acks() {
    // Datagrams held up to 3 s, past the 0.6 s NAK period: NAKs of sequences the client has
    // since flushed arrive late, and duplicates arrive long after the original.
    let slow = Conditions {
        drop: 0.05,
        duplicate: 0.2,
        delay: 0.3,
        max_delay: 3.0,
    };
    let run = run(31, slow, 60.0, 120.0);
    assert_delivered(&run);
    let counts = oracle::check(&run.session.frames());
    assert!(counts.empty_acks > 0, "{counts:?}");
}
