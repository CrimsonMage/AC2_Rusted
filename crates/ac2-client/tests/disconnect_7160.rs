//! The server closing the connection while the player stands in the world (Legions 7160
//! server double): the reason reaches the game, the disconnect handshake runs, the connection
//! is lost (`gmUIFlow::RecvNotice_ServerDied`) and the world is torn down.

mod common;

use ac2_client::{FrameInput, Notice};
use ac2_net::{NetErrorId, OptionalHeader};
use ac2_weenie::SessionEvent;
use common::*;
use legions_server_double::Spawn;

#[test]
fn a_server_disconnect_in_the_world_loses_the_connection_and_clears_the_world() {
    let spawn = Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (mut client, server) = entered(spawn);
    client.take_notices();
    server
        .borrow_mut()
        .reliable_header(OptionalHeader::NetErrorDisconnect(NetErrorId {
            string_id: 0x0100_0005,
            table_id: 0x0E00_0002,
        }));
    let idle = FrameInput::default();
    let mut seen = Vec::new();
    for _ in 0..640 {
        let events = step(&mut client, &server, &idle);
        seen.extend(events.into_iter().filter(|e| {
            matches!(
                e,
                SessionEvent::ConnectionError(_)
                    | SessionEvent::ServerDied
                    | SessionEvent::Disconnected(_)
            )
        }));
        if client.session().is_closed() {
            break;
        }
    }
    assert!(client.session().is_closed(), "{seen:?}");
    assert_eq!(seen.len(), 3, "{seen:?}");
    assert!(matches!(seen[0], SessionEvent::ConnectionError(_)));
    assert_eq!(seen[1], SessionEvent::ServerDied);
    assert!(matches!(seen[2], SessionEvent::Disconnected(_)));
    assert_eq!(client.take_notices(), [Notice::ConnectionLost]);
    // The client answered the server's disconnect.
    assert_eq!(server.borrow().disconnects_received, 1);
    let frame = client.frame();
    assert!(!frame.in_world, "in world");
    assert!(
        frame.objects.is_empty(),
        "{:?}",
        frame.objects.iter().map(|o| o.id).collect::<Vec<_>>()
    );
}
