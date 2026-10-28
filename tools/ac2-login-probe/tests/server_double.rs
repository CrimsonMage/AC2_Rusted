//! The probe against the in-process Legions server double, over real UDP on loopback: the
//! double (a scripted `Transport`) runs on a bridge thread behind two sockets, the logon port
//! and the connect-ack port after it.

use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use ac2_login_probe::{run_udp, FailureKind, ProbeConfig, Stage};
use ac2_net::{real_time, Credential, Logon, Transport, LEGIONS_VERSION};
use legions_server_double::{addr, double, Spawn, ACK_PORT, SERVER};

struct Bridge {
    server: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: JoinHandle<(Vec<Vec<u8>>, bool)>,
}

/// Two adjacent loopback sockets on an OS-chosen port, and the double behind them.
fn bridge() -> Bridge {
    let (logon_socket, ack_socket) = loop {
        let first = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = first.local_addr().unwrap().port();
        if let Ok(second) = UdpSocket::bind(("127.0.0.1", port.wrapping_add(1))) {
            break (first, second);
        }
    };
    let server = logon_socket.local_addr().unwrap();
    for socket in [&logon_socket, &ack_socket] {
        socket.set_nonblocking(true).unwrap();
    }
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let thread = std::thread::spawn(move || {
        let (mut transport, state) = double(Spawn::default());
        let start = Instant::now();
        let mut client = None;
        let mut buffer = vec![0u8; 0x1_0000];
        while !flag.load(Ordering::Relaxed) {
            state.borrow_mut().elapsed = start.elapsed().as_secs_f64();
            for (socket, to) in [(&logon_socket, addr(SERVER)), (&ack_socket, addr(ACK_PORT))] {
                while let Ok((len, from)) = socket.recv_from(&mut buffer) {
                    client = Some(from);
                    transport.send(to, &buffer[..len]).unwrap();
                }
            }
            state.borrow_mut().elapsed = start.elapsed().as_secs_f64();
            let outbound = transport.receive().unwrap();
            if let Some(client) = client {
                for (_, datagram) in outbound {
                    logon_socket.send_to(&datagram, client).unwrap();
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        let state = state.borrow();
        let sent = state.sent.iter().map(|(_, d)| d.clone()).collect();
        (sent, state.logged_off)
    });
    Bridge {
        server,
        stop,
        thread,
    }
}

fn config(server: SocketAddr, credential: Credential, version: &str) -> ProbeConfig {
    ProbeConfig {
        server,
        account: "local".into(),
        credential,
        version: version.into(),
        dat_iterations: Vec::new(),
        timeout: Duration::from_secs(20),
    }
}

#[test]
fn a_password_logon_reaches_the_character_set_and_logs_off() {
    let bridge = bridge();
    let before = real_time();
    let mut stages = Vec::new();
    let result = run_udp(
        config(
            bridge.server,
            Credential::Password("hunter2".into()),
            LEGIONS_VERSION,
        ),
        &mut |stage| stages.push(stage.clone()),
    );
    let after = real_time();
    let set = result.unwrap_or_else(|f| panic!("probe failed: {} {}", f.kind.label(), f.detail));
    assert!(!set.characters.is_empty());

    // Give the double a moment to read the logoff datagram, then stop it.
    std::thread::sleep(Duration::from_millis(300));
    bridge.stop.store(true, Ordering::Relaxed);
    let (sent, logged_off) = bridge.thread.join().unwrap();
    assert!(logged_off, "the double saw the logoff");

    let position = |wanted: &dyn Fn(&Stage) -> bool| stages.iter().position(wanted).unwrap();
    let logon = position(&|s| {
        matches!(
            s,
            Stage::LogonSent {
                auth_type: 0x4000_0002,
                resend: 0,
                ..
            }
        )
    });
    let connected = position(&|s| matches!(s, Stage::Connected { .. }));
    let characters = position(&|s| matches!(s, Stage::CharacterSet { .. }));
    let logoff = position(&|s| matches!(s, Stage::LogoffSent));
    assert!(logon < connected && connected < characters && characters < logoff);
    // The password never reaches the report.
    assert!(stages.iter().all(|s| !s.to_string().contains("hunter2")));

    // The logon datagram is `Logon::datagram()` for the same inputs (the connection sequence
    // is the Unix second the session started).
    let first = &sent[0];
    let identical = (before - 1..=after + 1).any(|connection_sequence| {
        let expected = Logon {
            version: LEGIONS_VERSION.into(),
            account: "local".into(),
            credential: Credential::Password("hunter2".into()),
            connection_sequence: connection_sequence as u32,
            crypto: None,
        };
        &expected.datagram() == first
    });
    assert!(
        identical,
        "the logon datagram differs from Logon::datagram()"
    );
}

#[test]
fn silence_times_out_with_its_own_failure_kind() {
    // A bound socket that never answers.
    let silent = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut probe = config(silent.local_addr().unwrap(), Credential::Account, "7160");
    probe.timeout = Duration::from_millis(300);
    let failure = run_udp(probe, &mut |_| {}).unwrap_err();
    assert_eq!(failure.kind, FailureKind::Timeout);
    assert_eq!(failure.kind.exit_code(), 3);
}

#[test]
fn a_version_mismatch_reason_is_a_refusal_with_its_decoded_name() {
    use ac2_net::{flags, tokenize, NetErrorId, OptionalHeader, PacketHeader};
    // A server that refuses the logon with `ID_ConnectionError_NetVersionMismatch`.
    let refuser = UdpSocket::bind("127.0.0.1:0").unwrap();
    refuser
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let server = refuser.local_addr().unwrap();
    let thread = std::thread::spawn(move || {
        let mut buffer = vec![0u8; 0x1_0000];
        let (_, client) = refuser.recv_from(&mut buffer).unwrap();
        let body = OptionalHeader::NetError(NetErrorId {
            string_id: tokenize(b"ID_ConnectionError_NetVersionMismatch"),
            table_id: 0x4000_0005,
        })
        .encode();
        let header = PacketHeader {
            flags: flags::NET_ERROR,
            ..Default::default()
        };
        let reply = legions_server_double::datagram(header, &[&body], None);
        refuser.send_to(&reply, client).unwrap();
    });
    let failure = run_udp(config(server, Credential::Account, "7148"), &mut |_| {}).unwrap_err();
    thread.join().unwrap();
    assert_eq!(failure.kind, FailureKind::Refused);
    assert_eq!(failure.kind.exit_code(), 4);
    assert!(
        failure.detail.contains("NetVersionMismatch"),
        "{}",
        failure.detail
    );
}
