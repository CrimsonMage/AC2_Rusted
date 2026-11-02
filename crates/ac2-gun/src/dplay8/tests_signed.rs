//! CONNECTED_SIGNED and signed frames against a scripted signing host (DX9 0x00421f01,
//! 0x00427720, 0x004248ec; `signing.rs` for the algorithm). The expected FAST frames are literal
//! bytes; the FULL signatures are SHA-1 values computed outside this crate (Python hashlib over
//! the datagram with the signature field zero followed by the 8 secret bytes, first 8 digest
//! bytes) and written in as constants.

use crate::Hresult;

use super::clock::FixedRandom;
use super::tests::*;

const SID: u32 = 0x1122_3344;
const T0: u32 = 10_000;
/// The client's drawn secrets: send `S` (the first two words), receive `R` (the next two).
const S: [u8; 8] = [0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x22, 0x22];
const R: [u8; 8] = [0x33, 0x33, 0x33, 0x33, 0x44, 0x44, 0x44, 0x44];

fn c1_payload() -> Vec<u8> {
    let mut p = vec![0xC1, 0, 0, 0, 2, 0, 0, 0, 7, 0, 0, 0];
    p.extend_from_slice(&[0; 40]);
    p.extend_from_slice(&[0; 16]);
    p.extend_from_slice(&[
        0x7d, 0x76, 0x80, 0x22, 0x89, 0x16, 0x6b, 0x4d, 0x81, 0xdd, 0x3a, 0xd9, 0xe8, 0x0f, 0xb6,
        0x55,
    ]);
    p.extend_from_slice(&[0; 8]);
    p
}

fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.iter().flat_map(|p| p.iter().copied()).collect()
}

/// The host's CONNECTED_SIGNED answering our CONNECT (bMsgID 0): POLL, bMsgID 5, 0x10006, our
/// session id, tick 77, cookie 01..08, no secrets, options `opts`, no echo.
fn host_connected_signed(opts: u8) -> Vec<u8> {
    cat(&[
        &[0x88, 0x03, 0x05, 0x00, 0x06, 0x00, 0x01, 0x00],
        &SID.to_le_bytes(),
        &77u32.to_le_bytes(),
        &[1, 2, 3, 4, 5, 6, 7, 8],
        &[0; 16],
        &[opts, 0, 0, 0, 0, 0, 0, 0],
    ])
}

fn start(h: &mut H) {
    h.client.set_secret_source(Box::new(FixedRandom(
        vec![0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444],
        0,
    )));
    h.gundll_caps();
    let host = super::address::DpAddress::build_address("10.0.0.5", 2300);
    let mut dev = super::address::DpAddress::new();
    dev.set_sp(super::address::CLSID_DP8SP_TCPIP);
    dev.add_component("port", super::address::Component::Dword(2302));
    assert!(h
        .client
        .connect(&app_desc(), &host, &dev, false, false, None, 0xC0FFEE, 0)
        .is_ok());
    h.tick();
    assert_eq!(h.sent(), vec![cframe(true, 1, 0, 0, SID, T0 + 20)]);
    h.tick();
}

/// The client's CONNECTED_SIGNED reply (0x30 bytes, `bRspId` 0, secrets S/R, options echoed,
/// the host's cookie and tick echoed), sent at tick `T0 + 60`.
fn expected_reply(opts: u8) -> Vec<u8> {
    cat(&[
        &[0x80, 0x03, 0x01, 0x00, 0x06, 0x00, 0x01, 0x00],
        &SID.to_le_bytes(),
        &(T0 + 60).to_le_bytes(),
        &[1, 2, 3, 4, 5, 6, 7, 8],
        &S,
        &R,
        &[opts, 0, 0, 0],
        &77u32.to_le_bytes(),
    ])
}

#[test]
fn fast_signing_handshake_data_and_bad_signature() {
    let mut h = H::new();
    start(&mut h);
    h.host(&host_connected_signed(1));
    h.tick();
    let out = h.sent();
    assert_eq!(out.len(), 3, "{out:?}");
    assert_eq!(out[0], expected_reply(1));
    // The initial KeepAlive and the lone 0xC1 carry the FAST signature (our send secret) after
    // the 4-byte header; the KeepAlive's session id follows the signature.
    assert_eq!(
        out[1],
        cat(&[&[0x3F, 0x02, 0x00, 0x00], &S, &SID.to_le_bytes()])
    );
    assert_eq!(out[2], cat(&[&[0x7F, 0x00, 0x01, 0x00], &S, &c1_payload()]));
    // The host's 0xC2 under its secret R (= our receive secret) completes the connect.
    let mut c2 = vec![0xC2, 0, 0, 0];
    c2.extend_from_slice(&[0; 12]);
    h.host(&cat(&[&[0x77, 0x00, 0x00, 0x02], &R, &c2]));
    h.tick();
    assert_eq!(
        h.sent(),
        vec![cat(&[&[0x7F, 0x00, 0x02, 0x01], &S, &[0xC3, 0, 0, 0]])]
    );
    assert_eq!(h.got(), vec![Got::Connect(Hresult(0), vec![], 0xC0FFEE)]);
    // A frame with the wrong signature (our own secret) is dropped; the right one is delivered.
    h.host(&cat(&[&[0x37, 0x00, 0x01, 0x03], &S, b"nope"]));
    h.tick();
    assert!(h.got().is_empty());
    h.host(&cat(&[&[0x37, 0x00, 0x01, 0x03], &R, b"yes"]));
    h.tick();
    let got = h.got();
    assert_eq!(got.len(), 1);
    assert!(matches!(&got[0], Got::Receive(d, _) if d == b"yes"));
}

#[test]
fn full_signing_signs_with_sha1_and_rejects_a_wrong_hash() {
    let mut h = H::new();
    start(&mut h);
    h.host(&host_connected_signed(2));
    h.tick();
    let out = h.sent();
    assert_eq!(out[0], expected_reply(2));
    let sig = |hex: &str| -> Vec<u8> {
        (0..8)
            .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap())
            .collect()
    };
    assert_eq!(
        out[1],
        cat(&[
            &[0x3F, 0x02, 0x00, 0x00],
            &sig("119185997a457da5"),
            &SID.to_le_bytes()
        ])
    );
    assert_eq!(
        out[2],
        cat(&[
            &[0x7F, 0x00, 0x01, 0x00],
            &sig("147fc4954a2dbd41"),
            &c1_payload()
        ])
    );
    let mut c2 = vec![0xC2, 0, 0, 0];
    c2.extend_from_slice(&[0; 12]);
    // wrong hash first (the FAST-style secret in the field): dropped
    h.host(&cat(&[&[0x77, 0x00, 0x00, 0x02], &R, &c2]));
    h.tick();
    assert!(h.sent().is_empty());
    assert!(h.got().is_empty());
    h.host(&cat(&[
        &[0x77, 0x00, 0x00, 0x02],
        &sig("dbf580b62d97fe7b"),
        &c2,
    ]));
    h.tick();
    assert_eq!(
        h.sent(),
        vec![cat(&[
            &[0x7F, 0x00, 0x02, 0x01],
            &sig("c033122088999b20"),
            &[0xC3, 0, 0, 0]
        ])]
    );
    assert_eq!(h.got(), vec![Got::Connect(Hresult(0), vec![], 0xC0FFEE)]);
}

#[test]
fn connected_signed_is_ignored_unless_it_is_well_formed() {
    // options 0 and 3, non-zero secrets, no POLL: ignored, no reply, no connection.
    for bad in [0u8, 3] {
        let mut h = H::new();
        start(&mut h);
        h.host(&host_connected_signed(bad));
        h.tick();
        assert!(h.sent().is_empty());
        assert_eq!(h.client.state(), super::core::CoreState::Connecting);
    }
    let mut h = H::new();
    start(&mut h);
    let mut f = host_connected_signed(1);
    f[0x18] = 9;
    h.host(&f);
    h.tick();
    assert!(h.sent().is_empty());
    let mut h = H::new();
    start(&mut h);
    let mut f = host_connected_signed(1);
    f[0] = 0x80;
    h.host(&f);
    h.tick();
    assert!(h.sent().is_empty());
}

#[test]
fn coalesced_dframe_is_decoded_into_its_messages() {
    let mut h = H::new();
    start(&mut h);
    h.host(&cframe(true, 2, 5, 0, SID, 77));
    h.tick();
    h.sent();
    let mut c2 = vec![0xC2, 0, 0, 0];
    c2.extend_from_slice(&[0; 12]);
    h.host(&cat(&[&[0x77, 0x00, 0x00, 0x02], &c2]));
    h.tick();
    h.sent();
    h.got();
    // seq 1, bControl 4, cmd 0x37 | NEW|END: table [3,0x42] [2,0x01|0x80], 2 headers (even, no
    // pad), "abc" padded to 4, then "de". Sub 1 RELIABLE (0x02), sub 2 last
    // (0x01); no USER bits, so both are application data.
    let frame = cat(&[
        &[0x37, 0x04, 0x01, 0x03],
        &[3, 0x02, 2, 0x01],
        b"abc\0",
        b"de",
    ]);
    h.host(&frame);
    h.tick();
    let got = h.got();
    let data: Vec<&[u8]> = got
        .iter()
        .map(|g| match g {
            Got::Receive(d, _) => d.as_slice(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(data, vec![b"abc".as_slice(), b"de".as_slice()]);
    // a zero size is invalid and dropped
    h.host(&cat(&[&[0x37, 0x04, 0x02, 0x03], &[0, 0x01], b"xx"]));
    h.tick();
    assert!(h.got().is_empty());
}
