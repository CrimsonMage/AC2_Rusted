//! The connect handshake against the fake DP8 and a fake GUN server that checks every byte the client
//! sends (`0a_send_session.md` sections 2-5): DP8 setup, `<Connect>`, `<ConnectRes>`, `<ConInfo>` with
//! the RC4-40 CdKey and the SIMPLEBLOB, `<ConInfoRes>` and the local events.

use md5::{Digest, Md5};
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

use super::fakes::*;
use crate::gundll::blob;
use crate::gundll::crypto::{build_public_key_blob, rc4};
use crate::gundll::dp8::*;
use crate::gundll::xml_parse::XmlPacket;
use crate::interfaces::*;
use crate::zt::*;
use crate::Hresult;

/// A tiny xorshift generator for key generation (tests only).
struct TestRng(u64);

impl rsa::rand_core::RngCore for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for b in dest {
            *b = (self.next_u64() >> 24) as u8;
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rsa::rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rsa::rand_core::CryptoRng for TestRng {}

pub(super) fn server_key() -> RsaPrivateKey {
    RsaPrivateKey::new(&mut TestRng(0x1234_5678_9abc_def1), 512).expect("512-bit key")
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// The values of a `<ConInfo>` the client sent.
struct ConInfo {
    cx: u32,
    locale: String,
    app_guid: String,
    cd_key: Vec<u8>,
    random: Vec<u8>,
}

fn read_con_info(text: &str) -> ConInfo {
    assert!(text.starts_with("<ConInfo Cx=\"0x"), "{text}");
    assert!(
        text.ends_with("</ConInfo> "),
        "trailing space after the close tag: {text:?}"
    );
    let p = XmlPacket::parse(&wide(text)).unwrap();
    let blob_of = |name: &str| {
        let c = p.find_child(None, name).unwrap();
        let (_, v) = p.attr_by_index(Some(c), 0).unwrap();
        blob::decode(v, 4096).unwrap()
    };
    let s = |n: &str| String::from_utf16_lossy(p.attr_string(None, n).unwrap());
    // the attribute order of 4.4: Cx Locale AppGuid, then the children CdKey, Random
    assert_eq!(p.attr_by_index(None, 0).unwrap().0, "Cx");
    assert_eq!(p.attr_by_index(None, 1).unwrap().0, "Locale");
    assert_eq!(p.attr_by_index(None, 2).unwrap().0, "AppGuid");
    assert_eq!(p.name(Some(p.child(None, 0).unwrap())), "CdKey");
    assert_eq!(p.name(Some(p.child(None, 1).unwrap())), "Random");
    ConInfo {
        cx: p.attr_hex(None, "Cx").unwrap(),
        locale: s("Locale"),
        app_guid: s("AppGuid"),
        cd_key: blob_of("CdKey"),
        random: blob_of("Random"),
    }
}

/// Runs the handshake up to (and including) the `<ConInfo>` and returns the rig and the sent text.
fn up_to_con_info(r: &Rig, key: &RsaPrivateKey, server_random: u32) -> (Sink, String) {
    assert_eq!(
        r.gun.init(&APP_GUID, "1.0", 1033, "XXXX-YYYY-ZZZZ", 0),
        Hresult::S_OK
    );
    assert_eq!(r.gun.connect("gun.example.net", 8610), Hresult::E_PENDING);
    let sink = Sink::default();
    // DP8 CONNECT_COMPLETE (hr 0) -> <Connect>
    r.dp8.push(Dp8Message::ConnectComplete {
        hresult: Hresult::S_OK,
        reply: vec![],
    });
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(r.dp8.sent_count(), 1);
    let cx = r.gun.connect_cx();
    assert_eq!(
        r.dp8.sent_text(0),
        format!("<Connect Cx=\"0x{cx:x}\" ProtoVer=\"3.3\" />")
    );
    // <ConnectRes>: Random decimal, RSAKey a PUBLICKEYBLOB in the 6-bit text
    let pubkey = build_public_key_blob(&key.n().to_bytes_be(), 0x10001);
    let res = format!(
        "<ConnectRes Cx=\"0x{cx:x}\" HR=\"0x0\" Random=\"{server_random}\" ><RSAKey Val=\"{}\"/></ConnectRes> ",
        blob::encode_for_attribute(&pubkey).unwrap()
    );
    r.dp8.push_datagram(datagram(&res));
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(r.dp8.sent_count(), 2, "the client answered with ConInfo");
    (sink, r.dp8.sent_text(1))
}

#[test]
fn dp8_setup_and_connect_call() {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    // Init creates no thread pool (CGun::Init 0x1000fab0 has no call to FUN_10003810) ...
    assert!(r.dp8.0.borrow().thread_counts.is_empty());
    assert_eq!(r.gun.connect("gun.example.net", 8610), Hresult::E_PENDING);
    // ... Connect does, through the handler constructor FUN_10035ae0 -> FUN_10003810:
    // SetThreadCount(-1, 0, 0)
    assert_eq!(r.dp8.0.borrow().thread_counts, vec![(u32::MAX, 0, 0)]);
    let s = r.dp8.0.borrow();
    // Initialize with DISABLELINKTUNING (4)
    assert_eq!(s.init_flags, vec![4]);
    // keep-alive: GetCaps(dwSize 0x14), SetCaps(15000), GetCaps
    assert_eq!(
        s.caps_log,
        vec!["get size=0x14", "set keepalive=15000", "get size=0x14"]
    );
    let (app, host, device) = &s.connects[0];
    assert_eq!(app.size, 0x48);
    assert_eq!(app.flags, 1, "DPNSESSION_CLIENT_SERVER");
    assert_eq!(app.guid_application, APP_GUID);
    assert_eq!(
        app.guid_instance,
        Guid {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: [0; 8]
        }
    );
    // the TCP/IP SP {ebfe7ba0-628d-11d2-ae0f-006097b01411}
    assert_eq!(host.sp, DP8SP_TCPIP);
    assert_eq!(device.sp.data1, 0xebfe7ba0);
    assert_eq!(
        host.components,
        vec![
            (
                "hostname".to_string(),
                Dp8AddrData::String("gun.example.net".into())
            ),
            ("port".to_string(), Dp8AddrData::Dword(8610)),
        ]
    );
    // portLocal 0: the probe's port goes to the device address
    assert_eq!(
        device.components,
        vec![("port".to_string(), Dp8AddrData::Dword(40123))]
    );
}

#[test]
fn a_given_local_port_is_used_and_a_failed_probe_adds_none() {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 5555);
    r.gun.connect("h", 1);
    assert_eq!(
        r.dp8.0.borrow().connects[0].2.components,
        vec![("port".to_string(), Dp8AddrData::Dword(5555))]
    );
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.gun.env.borrow_mut().probe_port = Box::new(|| None);
    r.gun.connect("h", 1);
    assert!(r.dp8.0.borrow().connects[0].2.components.is_empty());
}

#[test]
fn init_connect_and_status_rules() {
    let r = rig();
    let mut st = ConnectStatus::Failed;
    assert_eq!(r.gun.get_status(&mut st), ZT_E_NOT_INITED);
    assert_eq!(r.gun.get_last_error(), ZT_E_NOT_INITED);
    assert_eq!(r.gun.connect("h", 1), ZT_E_NOT_INITED);
    assert_eq!(r.gun.disconnect(), ZT_E_NOT_INITED);
    assert_eq!(r.gun.dispatch_queue(&Sink::default()), ZT_E_NOT_INITED);
    assert_eq!(r.gun.init(&APP_GUID, "1.0", 1033, "k", 0), Hresult::S_OK);
    assert_eq!(
        r.gun.init(&APP_GUID, "1.0", 1033, "k", 0),
        ZT_E_ALREADY_INITED
    );
    assert_eq!(r.gun.get_status(&mut st), Hresult::S_OK);
    assert_eq!(st, ConnectStatus::Created);
    assert_eq!(r.gun.get_last_error(), Hresult::S_OK);
    assert_eq!(r.gun.disconnect(), ZT_E_CLOSED);
    assert_eq!(r.gun.connect("h", 1), Hresult::E_PENDING);
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Connecting);
    // Connect again: the old connection is closed first (FUN_10003ae0 -> FUN_1003d600 Close, then
    // the destructor FUN_1003dfa0 -> FUN_1003d600 Close and its final Close: three per Disconnect)
    assert_eq!(r.gun.connect("h2", 2), Hresult::E_PENDING);
    assert_eq!(r.dp8.0.borrow().closed, 3);
    assert_eq!(r.gun.disconnect(), Hresult::S_OK);
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Disconnected);
    assert_eq!(r.dp8.0.borrow().closed, 6);
}

#[test]
fn guid_text_is_stringfromguid2() {
    assert_eq!(
        crate::gundll::cgun::guid_text(&APP_GUID),
        "{2280767D-1689-4D6B-81DD-3AD9E80FB655}"
    );
}

/// The whole handshake with a server that verifies the CdKey and the key exchange.
#[test]
fn full_handshake_with_a_checking_server() {
    let r = rig();
    let key = server_key();
    let (sink, text) = up_to_con_info(&r, &key, 0x1234_5678);
    let ci = read_con_info(&text);
    // Cx is the one of <ConnectRes> (echoed), Locale signed decimal, AppGuid the escaped GUID text
    assert_eq!(ci.cx, r.gun.connect_cx());
    assert_eq!(ci.locale, "1033");
    assert_eq!(ci.app_guid, "{2280767D-1689-4D6B-81DD-3AD9E80FB655}");
    // Random is a SIMPLEBLOB: BLOBHEADER { 1, 2, 0, CALG_RC4 }, ALG_ID CALG_RSA_KEYX, 64 key bytes (LE)
    assert_eq!(ci.random.len(), 12 + 64, "12 + modulus bytes");
    assert_eq!(
        &ci.random[..12],
        &[1, 2, 0, 0, 0x01, 0x68, 0, 0, 0x00, 0xa4, 0, 0]
    );
    let mut ct = ci.random[12..].to_vec();
    ct.reverse();
    let key5 = key
        .decrypt(Pkcs1v15Encrypt, &ct)
        .expect("PKCS #1 v1.5 block");
    assert_eq!(key5.len(), 5, "a 40-bit key");
    // the key is the first 5 bytes of MD5(clientRandom LE || serverRandom LE); the counter RNG's first
    // four bytes are the client random
    let client_random: u32 = u32::from_le_bytes([1, 2, 3, 4]);
    let mut input = client_random.to_le_bytes().to_vec();
    input.extend_from_slice(&0x1234_5678u32.to_le_bytes());
    assert_eq!(&Md5::digest(&input)[..5], &key5[..]);
    // CdKey: RC4(UTF-8 + NUL) keyed with the 5 bytes and the 11 zero salt bytes
    let rc4_key = server_session_key(&key5);
    let plain = rc4_apply(&rc4_key, &ci.cd_key);
    assert_eq!(plain, b"XXXX-YYYY-ZZZZ\0");
    assert_eq!(ci.cd_key.len(), 15);
    // <ConInfoRes> -> ConnectEv, status Connected
    let cx = r.gun.connect_cx();
    r.dp8.push_datagram(datagram(&format!(
        "<ConInfoRes Cx=\"0x{cx:x}\" HR=\"0x0\" />"
    )));
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.ids(), vec![0x1300]);
    let m = &sink.got.borrow()[0];
    assert_eq!((m.flags, m.gmid, m.param, m.cb), (4, 1, 0, 0));
    assert!(m.message.inited);
    let mut st = ConnectStatus::Failed;
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Connected);
    // later encrypted fields use the same key and restart the key stream every call (CryptEncrypt with
    // Final = TRUE, msdocs/cryptencrypt.md)
    r.reg
        .values
        .borrow_mut()
        .insert("LaunchTicketSize".into(), 3u32.to_le_bytes().to_vec());
    r.reg
        .values
        .borrow_mut()
        .insert("LaunchTicketData".into(), vec![9, 8, 7]);
    let mut t1 = TagBlob {
        cb_size: 16,
        data: Some(vec![0; 16]),
    };
    assert_eq!(r.gun.get_ticket(&mut t1), Hresult::S_OK);
    assert_eq!(t1.cb_size, 3);
    assert_eq!(
        rc4_apply(&rc4_key, &t1.data.as_ref().unwrap()[..3]),
        vec![9, 8, 7]
    );
    let mut t2 = TagBlob {
        cb_size: 16,
        data: Some(vec![0; 16]),
    };
    assert_eq!(r.gun.get_ticket(&mut t2), Hresult::S_OK);
    assert_eq!(
        t1, t2,
        "the cached ticket encrypts to the same bytes every time"
    );
    // no other message was sent
    assert_eq!(r.dp8.sent_count(), 2);
}

#[test]
fn rc4_known_vectors() {
    // the classic test vectors
    let mut d = b"Plaintext".to_vec();
    rc4(b"Key", &mut d);
    assert_eq!(d, [0xBB, 0xF3, 0x16, 0xE8, 0xD9, 0x40, 0xAF, 0x0A, 0xD3]);
    let mut d = b"pedia".to_vec();
    rc4(b"Wiki", &mut d);
    assert_eq!(d, [0x10, 0x21, 0xBF, 0x04, 0x20]);
}

#[test]
fn md5_known_vector() {
    // RFC 1321: MD5("abc")
    let h = Md5::digest(b"abc");
    assert_eq!(
        h[..],
        [
            0x90, 0x01, 0x50, 0x98, 0x3c, 0xd2, 0x4f, 0xb0, 0xd6, 0x96, 0x3f, 0x7d, 0x28, 0xe1,
            0x7f, 0x72
        ]
    );
}

#[test]
fn connect_res_with_a_failing_hr_sets_the_error_and_stops_the_dispatch() {
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
    r.dp8.push_datagram(datagram(&format!(
        "<ConnectRes Cx=\"0x{cx:x}\" HR=\"0xc1120c05\" Random=\"1\" ><RSAKey Val=\"\"/></ConnectRes> "
    )));
    // the handler returns the negative HR: Read answers 1 and DispatchQueue returns 1
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult(1));
    let mut st = ConnectStatus::Created;
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Failed);
    assert_eq!(r.gun.get_last_error(), ZT_E_BAD_PROTOCOL_VER);
    assert!(
        sink.ids().is_empty(),
        "the handshake message never reaches the application"
    );
    assert_eq!(r.dp8.sent_count(), 1, "no ConInfo");
}

#[test]
fn a_bad_server_key_fails_the_handshake() {
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
    r.dp8.push_datagram(datagram(&format!(
        "<ConnectRes Cx=\"0x{cx:x}\" HR=\"0x0\" Random=\"1\" ><RSAKey Val=\"ba\"/></ConnectRes> "
    )));
    let hr = r.gun.dispatch_queue(&sink);
    assert_eq!(hr, Hresult(1));
    assert_eq!(r.dp8.sent_count(), 1);
}

#[test]
fn con_info_res_with_a_failing_hr_raises_no_connect_event() {
    let r = rig();
    let key = server_key();
    let (sink, _) = up_to_con_info(&r, &key, 7);
    let cx = r.gun.connect_cx();
    r.dp8.push_datagram(datagram(&format!(
        "<ConInfoRes Cx=\"0x{cx:x}\" HR=\"0xc1121007\" />"
    )));
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult(1));
    assert!(sink.ids().is_empty());
    let mut st = ConnectStatus::Created;
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Failed);
    assert_eq!(r.gun.get_last_error(), ZT_E_AUTHENTICATION_FAILURE);
}

#[test]
fn a_refused_connect_raises_connect_failed_ev() {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.gun.connect("h", 1);
    let sink = Sink::default();
    r.dp8.push(Dp8Message::ConnectComplete {
        hresult: DPNERR_HOSTREJECTEDCONNECTION,
        reply: vec![],
    });
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    // 0x1302 ConnectFailedEv: status 4, last error E_ACCESSDENIED, the connection freed
    assert_eq!(sink.ids(), vec![0x1302]);
    let mut st = ConnectStatus::Created;
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Failed);
    assert_eq!(r.gun.get_last_error(), Hresult(0x8007_0005));
    assert_eq!(r.gun.friend_add("x", Context(1)), ZT_E_NOT_CONNECTED);
    assert_eq!(r.dp8.sent_count(), 0, "no <Connect> was sent");
    // any other failure is ZT_E_FAILED_TO_CONNECT
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r.gun.connect("h", 1);
    r.dp8.push(Dp8Message::ConnectComplete {
        hresult: Hresult(0x8015_8030),
        reply: vec![],
    });
    r.gun.dispatch_queue(&sink);
    assert_eq!(r.gun.get_last_error(), ZT_E_FAILED_TO_CONNECT);
}

#[test]
fn terminate_session_raises_disconnect_ev() {
    let r = rig();
    let key = server_key();
    let (sink, _) = up_to_con_info(&r, &key, 7);
    let cx = r.gun.connect_cx();
    r.dp8.push_datagram(datagram(&format!(
        "<ConInfoRes Cx=\"0x{cx:x}\" HR=\"0x0\" />"
    )));
    r.gun.dispatch_queue(&sink);
    r.dp8.push(Dp8Message::TerminateSession {
        hresult: DPNERR_HOSTTERMINATEDSESSION,
        data: vec![],
    });
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(sink.ids(), vec![0x1300, 0x1301]);
    let mut st = ConnectStatus::Created;
    r.gun.get_status(&mut st);
    assert_eq!(st, ConnectStatus::Disconnected);
    assert_eq!(
        r.gun.get_last_error(),
        Hresult::S_OK,
        "type 2 leaves +0x54 alone"
    );
    assert_eq!(r.gun.friend_add("x", Context(1)), ZT_E_NOT_CONNECTED);
}

#[test]
fn the_dp8_result_codes_map_as_the_dispatcher_does() {
    use crate::gundll::transport::{NetEvent, E_ACCESSDENIED};
    let t = |m: Dp8Message| {
        let r = rig();
        r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
        r.gun.connect("h", 1);
        // drive the transport of the connection directly
        let tr = r
            .gun
            .st
            .borrow()
            .conn
            .as_ref()
            .unwrap()
            .transport
            .clone()
            .unwrap();
        r.dp8.push(m);
        r.dp8.do_work(u32::MAX, 0);
        tr.drain()
    };
    let ev = |hr| {
        t(Dp8Message::TerminateSession {
            hresult: hr,
            data: vec![],
        })
    };
    assert_eq!(
        ev(DPNERR_NOCONNECTION),
        vec![NetEvent::Disconnected(ZT_E_CONNECTION_DROPPED_GONE)]
    );
    assert_eq!(
        ev(DPNERR_HOSTTERMINATEDSESSION),
        vec![NetEvent::Disconnected(ZT_E_CONNECTION_SERVERBOOT)]
    );
    assert_eq!(
        ev(DPNERR_USERCANCEL),
        vec![NetEvent::Disconnected(ZT_E_CONNECTION_CLIENTDISCONNECT)]
    );
    assert_eq!(
        ev(Hresult(0x8015_8999)),
        vec![NetEvent::Disconnected(ZT_E_CONNECTION_DROPPED_LOST)]
    );
    let cc = |hr| {
        t(Dp8Message::ConnectComplete {
            hresult: hr,
            reply: vec![],
        })
    };
    assert_eq!(
        cc(Hresult::S_OK),
        vec![NetEvent::ConnectComplete(Hresult::S_OK)]
    );
    assert_eq!(
        cc(DPNERR_HOSTREJECTEDCONNECTION),
        vec![NetEvent::ConnectComplete(E_ACCESSDENIED)]
    );
    assert_eq!(
        cc(Hresult(0x8015_8030)),
        vec![NetEvent::ConnectComplete(ZT_E_FAILED_TO_CONNECT)]
    );
    // SEND_COMPLETE, ASYNC_OP_COMPLETE, CREATE_PLAYER and unknown ids raise nothing
    assert!(t(Dp8Message::SendComplete {
        context: 1,
        hresult: DPNERR_TIMEDOUT
    })
    .is_empty());
    assert!(t(Dp8Message::AsyncOpComplete).is_empty());
    assert!(t(Dp8Message::CreatePlayer { dpnid: 3 }).is_empty());
    assert!(t(Dp8Message::Other { id: 0xFFFF_0001 }).is_empty());
}
