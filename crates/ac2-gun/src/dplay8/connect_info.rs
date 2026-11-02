//! The core session messages the client sends and reads (DPL8CS), from dpnet.dll.c.
//!
//! Every system message rides in a reliable, sequential DFRAME with USER_1 (0x40) set, and
//! starts with a 4-byte little-endian packet type (`DnCore_SendPacket`, dpnet 0x0040347e,
//! dpnet.dll.c:1462-1486: types with `(type & 0xC0) != 0` send the type as its own 4-byte
//! buffer and `flags |= 0x40`).

use crate::Hresult;

/// Packet types (`DnCore_DispatchPacket`, dpnet 0x0040489f, dpnet.dll.c:2463-2628).
pub const TYPE_PLAYER_CONNECT_INFO: u32 = 0xC1;
pub const TYPE_SEND_CONNECT_INFO: u32 = 0xC2;
pub const TYPE_ACK_CONNECT_INFO: u32 = 0xC3;
pub const TYPE_CONNECT_FAILED: u32 = 0xC5;
pub const TYPE_TERMINATE_SESSION: u32 = 0xDF;
pub const TYPE_REQ_PROCESS_COMPLETION: u32 = 0xE0;
pub const TYPE_PROCESS_COMPLETION: u32 = 0xE1;

/// The `dwDNETVersion` DX 9.0b writes into 0xC1: `local_48[1] = 7` (dpnet9.dll.c:10775); the
/// message is the `_EX` layout (header 0x58, MC-DPL8CS 2.2.1.2).
pub const DNET_VERSION: u32 = 7;

/// Core flag for a client (`IID_IDirectPlay8Client` -> `flags |= 2`, dpnet.dll.c:4568-4604); the
/// 0xC1 `dwFlags` is `core.flags & 7`.
pub const CORE_FLAG_CLIENT: u32 = 2;

/// Header length of 0xC1 in DX9: 22 dwords (0x58); the last two (alternate-address offset and
/// size, 0x50/0x54) are 0 for a client.
pub const CONNECT_INFO_HEADER: usize = 0x58;

/// The inputs of `DnCore_BuildConnectRequest` (dpnet 0x0040c6c7, dpnet.dll.c:9715).
pub struct ConnectRequest<'a> {
    /// `core.flags & 7`.
    pub flags: u32,
    /// Player name, UTF-16 with its terminator, size `wcslen * 2 + 2`
    /// (`SetClientInfo`, dpnet.dll.c:4920-4933). `None` when no client info was set.
    pub name: Option<&'a [u16]>,
    /// Player data, raw.
    pub data: Option<&'a [u8]>,
    /// Session password, UTF-16 with terminator.
    pub password: Option<&'a [u16]>,
    /// The opaque connect data of the Connect call.
    pub connect_data: Option<&'a [u8]>,
    pub guid_instance: [u8; 16],
    pub guid_application: [u8; 16],
}

fn wide_bytes(w: &[u16]) -> Vec<u8> {
    let mut b = Vec::with_capacity(w.len() * 2 + 2);
    for c in w {
        b.extend_from_slice(&c.to_le_bytes());
    }
    b.extend_from_slice(&[0, 0]);
    b
}

/// The 0xC1 body (after the 4-byte type), DX 9.0b `_EX` layout (0x0040eb96, dpnet9.dll.c:10769-10853):
/// the 0x58-byte header, then the blobs packed from the END of the buffer backwards in the order
/// name, data, password, connectData (a client has no URL and no alternate address: slots
/// 0x28/0x2c and 0x50/0x54 stay 0/0, 10733-10853). Offsets are from the start of the body and
/// an absent blob has offset 0 and size 0. GunDll's body is exactly the 0x58 header, so with the
/// type the DFRAME payload is 0x5c bytes.
pub fn build_connect_request(r: &ConnectRequest<'_>) -> Vec<u8> {
    let name = r.name.map(wide_bytes);
    let password = r.password.map(wide_bytes);
    let blobs: [Option<&[u8]>; 4] = [
        name.as_deref().filter(|b| !b.is_empty()),
        r.data.filter(|b| !b.is_empty()),
        password.as_deref().filter(|b| !b.is_empty()),
        r.connect_data.filter(|b| !b.is_empty()),
    ];
    let total = CONNECT_INFO_HEADER + blobs.iter().flatten().map(|b| b.len()).sum::<usize>();
    let mut buf = vec![0u8; total];
    let put =
        |buf: &mut Vec<u8>, at: usize, v: u32| buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
    put(&mut buf, 0, r.flags & 7);
    put(&mut buf, 4, DNET_VERSION);
    let mut end = total;
    for (i, blob) in blobs.iter().enumerate() {
        // Header slots: name 2/3, data 4/5, password 6/7, connect data 8/9.
        let slot = 8 + i * 8;
        if let Some(b) = blob {
            end -= b.len();
            buf[end..end + b.len()].copy_from_slice(b);
            put(&mut buf, slot, end as u32);
            put(&mut buf, slot + 4, b.len() as u32);
        }
    }
    buf[0x30..0x40].copy_from_slice(&r.guid_instance);
    buf[0x40..0x50].copy_from_slice(&r.guid_application);
    buf
}

/// A system message: the 4-byte type, then the body.
pub fn with_type(ty: u32, body: &[u8]) -> Vec<u8> {
    let mut v = ty.to_le_bytes().to_vec();
    v.extend_from_slice(body);
    v
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// A blob located by `{offset, size}` at `at` in `body`; `offset` counts from the start of the
/// body (`FUN_00403de6` reads `*p + p`, dpnet.dll.c:1872-1885). Offset 0 is "none".
fn blob_at(body: &[u8], at: usize) -> Option<Vec<u8>> {
    let off = u32_at(body, at)? as usize;
    let size = u32_at(body, at + 4)? as usize;
    if off == 0 {
        return Some(Vec::new());
    }
    body.get(off..off.checked_add(size)?).map(|s| s.to_vec())
}

/// 0xC2 `DN_SEND_CONNECT_INFO`: the first two dwords are the application reply data's
/// `{offset, size}` (the host builds them first, dpnet.dll.c:9390-9395). The rest of the message
/// (session description, name table) is parsed by dpnet's generic unpacker (`FUN_00402624`,
/// descriptor 0xf), which is not ported.
// open: the 0xC2 name-table / membership entries are not parsed; only the reply data is read.
pub fn parse_send_connect_info(body: &[u8]) -> Option<Vec<u8>> {
    blob_at(body, 0)
}

/// 0xC5 `DN_CONNECT_FAILED`: `{ hResultCode, dwReplyOffset, dwReplySize, reply }`
/// (`DnCore_OnConnectFailed`, dpnet 0x0040bb24, dpnet.dll.c:9058).
pub fn parse_connect_failed(body: &[u8]) -> Option<(Hresult, Vec<u8>)> {
    let hr = u32_at(body, 0)?;
    let reply = blob_at(body, 4).unwrap_or_default();
    Some((Hresult(hr), reply))
}

/// 0xDF `DN_TERMINATE_SESSION`: `{ dwTerminateDataOffset, dwTerminateDataSize, data }`
/// (`DnCore_OnTerminateSession`, dpnet 0x00403de6).
pub fn parse_terminate_session(body: &[u8]) -> Vec<u8> {
    blob_at(body, 0).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gundll_connect_request_is_exactly_the_0x58_header() {
        // GunDll sends no name, data, password or connect data, and a client sends no URL
        // (verify_spec_client_delta 3.1d, 3.1g): the DFRAME payload is 0x5c bytes.
        let app = [
            0x7d, 0x76, 0x80, 0x22, 0x89, 0x16, 0x6b, 0x4d, 0x81, 0xdd, 0x3a, 0xd9, 0xe8, 0x0f,
            0xb6, 0x55,
        ];
        let r = ConnectRequest {
            flags: CORE_FLAG_CLIENT,
            name: None,
            data: None,
            password: None,
            connect_data: None,
            guid_instance: [0; 16],
            guid_application: app,
        };
        let b = build_connect_request(&r);
        assert_eq!(b.len(), 0x58);
        // dwFlags 2, dwDNETVersion 7
        assert_eq!(&b[0..8], &[2, 0, 0, 0, 7, 0, 0, 0]);
        // name, data, password, connectData, URL: offset 0 size 0
        assert!(b[8..0x30].iter().all(|x| *x == 0));
        assert_eq!(&b[0x30..0x40], &[0u8; 16]);
        assert_eq!(&b[0x40..0x50], &app);
        assert!(b[0x50..].iter().all(|x| *x == 0));
    }

    #[test]
    fn blobs_pack_from_the_end_backwards_name_last() {
        let name: Vec<u16> = "ab".encode_utf16().collect();
        let r = ConnectRequest {
            flags: 2,
            name: Some(&name),
            data: Some(&[1, 2, 3]),
            password: None,
            connect_data: Some(&[9, 9]),
            guid_instance: [0; 16],
            guid_application: [0; 16],
        };
        let b = build_connect_request(&r);
        assert_eq!(b.len(), 0x58 + 6 + 3 + 2);
        let off = |slot: usize| u32::from_le_bytes(b[slot..slot + 4].try_into().unwrap()) as usize;
        assert_eq!(off(0x20), 0x58); // connect data lowest
        assert_eq!(off(0x10), 0x5a); // data
        assert_eq!(off(0x08), 0x5d); // name last
        assert_eq!(&b[0x5d..0x63], &[b'a', 0, b'b', 0, 0, 0]);
        assert_eq!(&b[0x5a..0x5d], &[1, 2, 3]);
        assert_eq!(&b[0x58..0x5a], &[9, 9]);
        assert_eq!(&b[0x18..0x20], &[0u8; 8]);
    }

    #[test]
    fn failure_and_terminate_layouts() {
        // 0xC5: result, reply offset, reply size, reply at offset (from body start)
        let mut body = Vec::new();
        body.extend_from_slice(&0x8015_8410u32.to_le_bytes());
        body.extend_from_slice(&12u32.to_le_bytes());
        body.extend_from_slice(&2u32.to_le_bytes());
        body.extend_from_slice(&[7, 8]);
        assert_eq!(
            parse_connect_failed(&body),
            Some((Hresult(0x8015_8410), vec![7, 8]))
        );
        // 0xDF: offset 8, size 3
        let mut t = Vec::new();
        t.extend_from_slice(&8u32.to_le_bytes());
        t.extend_from_slice(&3u32.to_le_bytes());
        t.extend_from_slice(&[1, 2, 3]);
        assert_eq!(parse_terminate_session(&t), vec![1, 2, 3]);
        assert_eq!(parse_terminate_session(&[0; 8]), Vec::<u8>::new());
    }
}
