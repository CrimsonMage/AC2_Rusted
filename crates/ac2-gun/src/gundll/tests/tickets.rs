//! Tickets: `GetTicket`, `SetTicket`, `PresentTicket` and the registry values
//! (`0a_send_session.md` section 6, with the adjudication), and `SetAccountValue`.

use super::fakes::*;
use crate::gundll::blob;
use crate::gundll::xml_parse::XmlPacket;
use crate::interfaces::*;
use crate::zt::*;
use crate::Hresult;

fn blob_of(v: Vec<u8>) -> TagBlob {
    TagBlob {
        cb_size: v.len() as u32,
        data: Some(v),
    }
}

fn set(r: &Rig, name: &str, v: Vec<u8>) {
    r.reg.values.borrow_mut().insert(name.into(), v);
}

fn get(r: &Rig, name: &str) -> Option<Vec<u8>> {
    r.reg.values.borrow().get(name).cloned()
}

fn wide_bytes(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
}

/// Brings the session to the point where the session key exists (after the ConnectRes).
fn keyed() -> Rig {
    use crate::gundll::crypto::build_public_key_blob;
    use crate::gundll::dp8::Dp8Message;
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "XXXX-YYYY-ZZZZ", 0);
    r.gun.connect("h", 1);
    let sink = Sink::default();
    r.dp8.push(Dp8Message::ConnectComplete {
        hresult: Hresult::S_OK,
        reply: vec![],
    });
    r.gun.dispatch_queue(&sink);
    // a 512-bit modulus: any odd number with the top bit set does for the client's side
    let mut n = vec![0xC5u8; 64];
    n[63] = 0xB1;
    let key = build_public_key_blob(&n, 0x10001);
    let cx = r.gun.connect_cx();
    r.dp8.push_datagram(datagram(&format!(
        "<ConnectRes Cx=\"0x{cx:x}\" HR=\"0x0\" Random=\"5\" ><RSAKey Val=\"{}\"/></ConnectRes> ",
        blob::encode_for_attribute(&key).unwrap()
    )));
    assert_eq!(r.gun.dispatch_queue(&sink), Hresult::S_OK);
    assert_eq!(r.dp8.sent_count(), 2, "ConInfo went out");
    r
}

#[test]
fn get_ticket_reads_caches_deletes_and_needs_the_session_key() {
    let r = rig();
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    let mut b = TagBlob {
        cb_size: 64,
        data: Some(vec![0; 64]),
    };
    // nothing in the registry
    assert_eq!(r.gun.get_ticket(&mut b), ZT_E_AUTH_INVALID_TICKET);
    set(&r, "LaunchTicketSize", 4u32.to_le_bytes().to_vec());
    set(&r, "LaunchTicketData", vec![1, 2, 3, 4]);
    // no session key yet: the values are consumed and the call fails with NOT_CONNECTED
    assert_eq!(r.gun.get_ticket(&mut b), ZT_E_NOT_CONNECTED);
    assert_eq!(get(&r, "LaunchTicketSize"), None);
    assert_eq!(get(&r, "LaunchTicketData"), None);
}

#[test]
fn get_ticket_sizes_the_buffer_like_the_two_call_pattern() {
    let r = keyed();
    set(&r, "LaunchTicketSize", 5u32.to_le_bytes().to_vec());
    set(&r, "LaunchTicketData", vec![1, 2, 3, 4, 5]);
    let mut b = TagBlob::default();
    assert_eq!(r.gun.get_ticket(&mut b), ZT_E_BUFFER_TOO_SMALL);
    assert_eq!(b.cb_size, 5);
    let mut b = TagBlob {
        cb_size: 5,
        data: None,
    };
    assert_eq!(r.gun.get_ticket(&mut b), Hresult(0x8007_0057));
    let mut b = TagBlob {
        cb_size: 5,
        data: Some(vec![0; 5]),
    };
    assert_eq!(r.gun.get_ticket(&mut b), Hresult::S_OK);
    assert_ne!(b.data.as_ref().unwrap()[..5], [1, 2, 3, 4, 5], "encrypted");
}

#[test]
fn set_ticket_stores_a_short_ticket_as_given() {
    let r = rig();
    let mut t = blob_of(vec![7u8; 100]);
    assert_eq!(r.gun.set_ticket(&mut t), Hresult::S_OK);
    assert_eq!(get(&r, "LaunchTicketData"), Some(vec![7u8; 100]));
    assert_eq!(
        get(&r, "LaunchTicketSize"),
        Some(100u32.to_le_bytes().to_vec())
    );
    // 300 bytes still counts as short
    let mut t = blob_of(vec![1u8; 300]);
    assert_eq!(r.gun.set_ticket(&mut t), Hresult::S_OK);
    assert_eq!(
        get(&r, "LaunchTicketSize"),
        Some(300u32.to_le_bytes().to_vec())
    );
    // a registry failure is E_FAIL
    *r.reg.fail_writes.borrow_mut() = true;
    assert_eq!(
        r.gun.set_ticket(&mut blob_of(vec![1u8; 10])),
        Hresult(0x8000_FFFF)
    );
}

#[test]
fn set_ticket_repacks_an_xml_ticket() {
    let r = rig();
    // 16 bytes 00..0f as the IV, "hello" padded to 8 bytes as the text (standard Base64)
    let iv = "AAECAwQFBgcICQoLDA0ODw==";
    let text = "aGVsbG8hIQ==";
    let mut xml =
        format!("<ticket><iv>{iv}</iv><encryptedText>{text}</encryptedText>").into_bytes();
    xml.resize(310, b' ');
    let mut xml = blob_of(xml);
    assert_eq!(r.gun.set_ticket(&mut xml), Hresult::S_OK);
    let stored = get(&r, "LaunchTicketData").unwrap();
    assert_eq!(&stored[..16], &(0u8..16).collect::<Vec<_>>()[..]);
    // "hello!!" is 8 bytes; the decoder reports `3 * 3 + 2 - 1 = 7` for the 12-character text and
    // two padding characters (7.4: right for `==`)
    assert_eq!(&stored[16..], b"hello!!");
    assert_eq!(
        get(&r, "LaunchTicketSize"),
        Some((stored.len() as u32).to_le_bytes().to_vec())
    );
    // the caller's blob was rewritten in place and its cbSize is the new size (decoded + 0x10)
    assert_eq!(xml.cb_size as usize, stored.len());
    assert_eq!(xml.bytes(), &stored[..]);
    // a missing tag
    let mut bad = blob_of(b"<x>".repeat(120));
    assert_eq!(r.gun.set_ticket(&mut bad), ZT_E_AUTH_INVALID_TICKET);
}

fn sent_ticket_cmd(r: &Rig, n: usize) -> (String, Vec<u8>) {
    let text = r.dp8.sent_text(n);
    assert!(text.starts_with("<PresentTicketCmd Cx=\"0x"), "{text}");
    assert!(text.ends_with("</PresentTicketCmd> "));
    let w: Vec<u16> = text.encode_utf16().collect();
    let p = XmlPacket::parse(&w).unwrap();
    let email = String::from_utf16_lossy(p.attr_string(None, "Email").unwrap());
    let c = p.find_child(None, "Ticket").unwrap();
    let (_, v) = p.attr_by_index(Some(c), 0).unwrap();
    (email, blob::decode(v, 4096).unwrap())
}

#[test]
fn present_ticket_with_the_plaintext_marker_encrypts_and_sends_a_null_email() {
    let r = keyed();
    // markers at 100 and 0xCA are zero
    let ticket = vec![0x41u8; 0xE0];
    let mut t = ticket.clone();
    t[100] = 0;
    t[101] = 0;
    t[0xca] = 0;
    t[0xcb] = 0;
    set(&r, "Email", wide_bytes("me@example.com"));
    assert_eq!(r.gun.present_ticket(&t, Context(0x77)), Hresult::S_OK);
    let (email, sent) = sent_ticket_cmd(&r, 2);
    assert_eq!(
        email, "(null)",
        "[inferred] the CRT prints a NULL %ls as (null)"
    );
    assert_eq!(sent.len(), t.len());
    assert_ne!(sent, t, "encrypted with the session key");
    // the Email value was read and deleted either way
    assert_eq!(get(&r, "Email"), None);
}

#[test]
fn present_ticket_otherwise_sends_it_as_is_with_the_registry_email() {
    let r = keyed();
    let mut t = vec![0u8; 0xE0];
    t[100] = 1; // a non-zero marker: already in its sent form
    set(&r, "Email", wide_bytes("me@example.com"));
    assert_eq!(r.gun.present_ticket(&t, Context(0x77)), Hresult::S_OK);
    let (email, sent) = sent_ticket_cmd(&r, 2);
    assert_eq!(email, "me@example.com");
    assert_eq!(sent, t);
    assert_eq!(get(&r, "Email"), None);
    // without an Email value the buffer stays empty
    assert_eq!(r.gun.present_ticket(&t, Context(0x78)), Hresult::S_OK);
    let (email, _) = sent_ticket_cmd(&r, 3);
    assert_eq!(email, "");
    // an Email longer than the 100-byte buffer fails to read: empty
    set(&r, "Email", vec![b'a'; 120]);
    assert_eq!(r.gun.present_ticket(&t, Context(0x79)), Hresult::S_OK);
    assert_eq!(sent_ticket_cmd(&r, 4).0, "");
}

#[test]
fn present_ticket_checks() {
    let r = keyed();
    assert_eq!(r.gun.present_ticket(&[], Context(1)), Hresult(0x8007_0057));
    // no session key: the plaintext form cannot be encrypted
    let r2 = rig();
    r2.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    r2.gun.connect("h", 1);
    assert_eq!(
        r2.gun.present_ticket(&vec![0u8; 0xE0], Context(1)),
        ZT_E_NOT_CONNECTED
    );
}

#[test]
fn set_account_value_stores_the_email() {
    let r = rig();
    let v = AccountValueType(0);
    assert_eq!(
        r.gun.set_account_value(v, &wide_bytes("a@b.c\0")),
        Hresult::S_OK
    );
    assert_eq!(get(&r, "Email"), Some(wide_bytes("a@b.c")));
    assert_eq!(
        r.gun
            .set_account_value(AccountValueType(1), &wide_bytes("x\0")),
        Hresult(0x8007_0057)
    );
    assert_eq!(
        r.gun
            .set_account_value(v, &wide_bytes(&format!("{}\0", "x".repeat(51)))),
        ZT_E_STR_TOO_LONG
    );
    *r.reg.fail_writes.borrow_mut() = true;
    assert_eq!(
        r.gun.set_account_value(v, &wide_bytes("a\0")),
        Hresult(0x8000_FFFF)
    );
}

#[test]
fn simulate_authentication_checks_and_output() {
    let r = rig();
    let mut b = blob_of(vec![0; 0xcc]);
    assert_eq!(
        r.gun.simulate_authentication("a", "b", &mut b),
        ZT_E_NOT_INITED
    );
    assert_eq!(b.cb_size, 0xcc, "cbSize is written on every path");
    r.gun.init(&APP_GUID, "1.0", 1033, "k", 0);
    let mut b = blob_of(vec![0; 0xcb]);
    assert_eq!(
        r.gun.simulate_authentication("a", "b", &mut b),
        ZT_E_BUFFER_TOO_SMALL
    );
    assert_eq!(b.cb_size, 0xcc);
    let mut b = blob_of(vec![0; 0xcc]);
    assert_eq!(
        r.gun.simulate_authentication(&"a".repeat(51), "b", &mut b),
        Hresult(0x8007_0057)
    );
    assert_eq!(b.cb_size, 0xcc);
    // the output: the name at wide characters 0.., the password at 0x33.., each NUL padded to 0x32
    // characters with the terminators at wide characters 0x32 and 0x65 (bytes 100 and 0xca)
    let mut b = blob_of(vec![0xEE; 0xd0]);
    assert_eq!(
        r.gun.simulate_authentication("Ab", "xyz", &mut b),
        Hresult::S_OK
    );
    assert_eq!(b.cb_size, 0xcc);
    let d = b.data.as_ref().unwrap();
    let w = |i: usize| u16::from_le_bytes([d[2 * i], d[2 * i + 1]]);
    assert_eq!((w(0), w(1), w(2)), (b'A' as u16, b'b' as u16, 0));
    assert_eq!(
        (w(0x33), w(0x34), w(0x35), w(0x36)),
        (b'x' as u16, b'y' as u16, b'z' as u16, 0)
    );
    assert_eq!((w(0x31), w(0x32), w(0x64), w(0x65)), (0, 0, 0, 0));
    assert_eq!(d[0xcc], 0xEE, "only 0xcc bytes are written");
    // those terminators are the zero markers PresentTicket tests
    assert_eq!((d[100], d[101], d[0xca], d[0xcb]), (0, 0, 0, 0));
    // NULL strings are L""
    let mut b = blob_of(vec![1; 0xcc]);
    assert_eq!(r.gun.simulate_authentication("", "", &mut b), Hresult::S_OK);
    assert!(b.data.unwrap().iter().all(|&x| x == 0));
}

#[test]
fn create_account_encrypts_the_password() {
    let r = keyed();
    assert_eq!(
        r.gun.create_account("user", "secret", "", "", Context(5)),
        Hresult::S_OK
    );
    let text = r.dp8.sent_text(2);
    assert!(
        text.starts_with("<CreateAccount Cx=\"0x5\" User=\"user\" ><Password Val=\""),
        "{text}"
    );
}
