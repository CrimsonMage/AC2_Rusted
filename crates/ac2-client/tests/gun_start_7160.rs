//! `gmClient::Connect` (Legions 0x00404D20): `StartupGUN` runs before the game connection, and a
//! failed one returns its `NetError` without ever calling `Turbine::Client::Connect`. The
//! logon is held until the GUN start answers; a failure goes to the connecting screen once and
//! the logon never starts.

mod common;

use std::sync::Arc;

use ac2_gun::message::{GunQueueMessage, Message, MessageBody};
use ac2_gun::test_support::{TestGunHandle, TestLoader};
use ac2_weenie::gun::{GunClient, GunOptions};
use ac2_weenie::SessionEvent;
use common::*;

fn event(id: ac2_gun::MessageId) -> GunQueueMessage {
    GunQueueMessage {
        flags: 4,
        gmid: 1,
        param: 0,
        cb: 0,
        message: Message {
            inited: true,
            id,
            result: None,
            body: MessageBody::None,
        },
    }
}

fn held_client(server_name: &str) -> (ac2_client::Client, TestGunHandle) {
    let spawn = legions_server_double::Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (transport, _server) = legions_server_double::double(spawn);
    let session =
        ac2_weenie::OnlineSession::with_transport_held(config(), Box::new(transport)).unwrap();
    let mut client = ac2_client::Client::new(session, Arc::new(Synthetic));
    let handle = TestGunHandle::new();
    let options = GunOptions {
        gun_server: server_name.into(),
        ..GunOptions::default()
    };
    let loader = TestLoader::new(handle.gun());
    client.attach_gun(GunClient::new(options, false, "", 0x0409, Box::new(loader)));
    (client, handle)
}

#[test]
fn a_failed_gun_start_never_starts_the_logon() {
    let (mut client, handle) = held_client("gun.example");
    let idle = ac2_client::FrameInput::default();
    assert!(client.use_time(&idle).unwrap().is_empty());
    assert!(
        client.session().is_logon_held(),
        "held while StartupGUN waits"
    );
    handle.queue_message(event(ac2_gun::message_id::CONNECT_FAILED_EVENT));
    let mut failures = Vec::new();
    for _ in 0..200 {
        for e in client.use_time(&idle).unwrap() {
            if let SessionEvent::Disconnected(error) = e {
                failures.push(error);
            }
        }
    }
    // The failure reaches the connecting screen once, as ID_DataPatch_GUNFailed of table
    // 0x41000018.
    assert_eq!(
        failures,
        [ac2_net::NetError::Server {
            string_id: ac2_net::tokenize(b"ID_DataPatch_GUNFailed"),
            table_id: 0x4100_0018,
        }]
    );
    assert!(client.session().is_logon_held(), "no game connection");
    assert!(!client.frame().in_world);
}

#[test]
fn a_successful_gun_start_starts_the_logon() {
    let (mut client, handle) = held_client("gun.example");
    let idle = ac2_client::FrameInput::default();
    client.use_time(&idle).unwrap();
    handle.queue_message(event(ac2_gun::message_id::CONNECT_EVENT));
    client.use_time(&idle).unwrap();
    assert!(!client.session().is_logon_held());
}

#[test]
fn without_a_gunserver_the_logon_starts_at_the_first_frame() {
    let (mut client, _handle) = held_client("");
    client.use_time(&ac2_client::FrameInput::default()).unwrap();
    assert!(!client.session().is_logon_held());
}
