//! The in-process lossy proxy around a transport: a session through it, with the double behind
//! (no channel of its own), delivers everything once despite the proxy's loss.

use std::collections::BTreeSet;

use ac2_net::{Credential, Logon, NetClient, NetClientConfig, NetEvent};
use legions_server_double::{addr, double, Channel, Conditions, LossyTransport, Spawn, SERVER};

#[test]
fn a_session_through_the_lossy_proxy_delivers_everything_once() {
    let (inner, state) = double(Spawn::default());
    let conditions = Conditions {
        drop: 0.15,
        duplicate: 0.1,
        delay: 0.15,
        max_delay: 0.5,
    };
    let (proxy, channel) = LossyTransport::new(Box::new(inner), Channel::new(42, conditions));
    let config = NetClientConfig {
        server: addr(SERVER),
        logon: Logon {
            version: ac2_net::LEGIONS_VERSION.into(),
            account: "local".into(),
            credential: Credential::Account,
            connection_sequence: legions_server_double::REAL as u32,
            crypto: None,
        },
    };
    let mut client = NetClient::with_transport(config, Box::new(proxy));
    let mut logged_on = false;
    let mut sent = BTreeSet::new();
    let mut received = BTreeSet::new();
    let mut client_sent = BTreeSet::new();
    for frame in 0..64 * 120 {
        if logged_on && frame % 4 == 0 && sent.len() < 400 {
            sent.insert(
                state
                    .borrow_mut()
                    .send_event(&[1, 0, 0xF0, 0xF0, frame as u8]),
            );
            let blob = vec![2, 0, 0xF0, 0xF0, frame as u8];
            client_sent.insert(client.send(4, blob).expect("queued"));
        }
        for event in client.poll() {
            match event {
                NetEvent::LoggedOn => logged_on = true,
                NetEvent::Blob(blob) if blob.id >> 56 != 0x23 => {
                    assert!(received.insert(blob.id), "{:#x} twice", blob.id)
                }
                NetEvent::Disconnected(error) => panic!("{error}"),
                _ => {}
            }
        }
        state.borrow_mut().elapsed += 1.0 / 64.0;
        if sent.len() == 400 && received == sent {
            break;
        }
    }
    assert_eq!(received, sent);
    // The client's blobs all reached the double, once each.
    for _ in 0..64 * 10 {
        let got: BTreeSet<u64> = state
            .borrow()
            .blobs
            .iter()
            .filter(|b| b.1 == 4)
            .map(|b| b.0)
            .collect();
        if got == client_sent {
            break;
        }
        client.poll();
        state.borrow_mut().elapsed += 1.0 / 64.0;
    }
    let state = state.borrow();
    let got: Vec<u64> = state
        .blobs
        .iter()
        .filter(|b| b.1 == 4)
        .map(|b| b.0)
        .collect();
    assert_eq!(got.len(), client_sent.len());
    assert_eq!(got.into_iter().collect::<BTreeSet<_>>(), client_sent);
    let stats = channel.borrow().stats;
    assert!(
        stats.dropped > 50 && stats.duplicated > 30 && stats.delayed > 50,
        "{stats:?}"
    );
}
