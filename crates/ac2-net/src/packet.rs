//! One datagram's optional headers and blob fragments.
//!
//! `NetPacket::ComputeChecksum` sums each optional header's bytes and each fragment's 16-byte
//! header and data as separate `checksum32` segments. With [`flags::ENCRYPTED_CHECKSUM`] that body
//! sum is XORed with the sequence's ISAAC key; the header checksum is added last
//! (`SharedNet::SendPacket`).

use crate::checksum::checksum32;
use crate::header::{flags, PacketHeader, HEADER_LEN};
use crate::NetError;

/// Size of `BlobFragHeader_t`: u64 blob id, u16 fragment count, u16 size including this header,
/// u16 fragment index, u16 queue.
pub const FRAGMENT_HEADER_LEN: usize = 16;
/// Largest datagram accepted or produced.
pub const MAX_DATAGRAM: usize = 65_507;
/// `CSeqIDListHeader` lists hold fewer than 0x73 sequences.
const MAX_SEQUENCE_LIST: u32 = 0x73;

/// Optional headers decoded by a session packet, in the order they appear on the wire.
const SESSION_OPTIONAL: [u32; 10] = [
    flags::NAK,
    flags::EMPTY_ACK,
    flags::ACK,
    flags::LOGOFF,
    flags::NET_ERROR,
    flags::NET_ERROR_DISCONNECT,
    flags::TIME_SYNC,
    flags::ECHO_REQUEST,
    flags::ECHO_RESPONSE,
    flags::FLOW,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub blob_id: u64,
    pub count: u16,
    pub index: u16,
    pub queue: u16,
    pub data: Vec<u8>,
}

impl Fragment {
    fn decode(segment: &[u8]) -> Result<Self, NetError> {
        let u16_at = |i: usize| u16::from_le_bytes([segment[i], segment[i + 1]]);
        let mut id = [0; 8];
        id.copy_from_slice(&segment[..8]);
        let count = u16_at(8);
        let index = u16_at(12);
        if count == 0 || index >= count || segment.len() <= FRAGMENT_HEADER_LEN {
            return Err(NetError::Fragment);
        }
        Ok(Self {
            blob_id: u64::from_le_bytes(id),
            count,
            index,
            queue: u16_at(14),
            data: segment[FRAGMENT_HEADER_LEN..].to_vec(),
        })
    }

    pub(crate) fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(FRAGMENT_HEADER_LEN + self.data.len());
        out.extend(self.blob_id.to_le_bytes());
        out.extend(self.count.to_le_bytes());
        out.extend(((self.data.len() + FRAGMENT_HEADER_LEN) as u16).to_le_bytes());
        out.extend(self.index.to_le_bytes());
        out.extend(self.queue.to_le_bytes());
        out.extend(&self.data);
        out
    }
}

/// A validated session datagram.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InboundPacket {
    pub header: PacketHeader,
    pub naks: Vec<u32>,
    pub empty_acks: Vec<u32>,
    pub ack: Option<u32>,
    pub logoff: bool,
    /// `NetError` string and table ids.
    pub net_error: Option<(u32, u32)>,
    pub disconnect: Option<(u32, u32)>,
    pub time_sync: Option<f64>,
    pub echo_request: Option<f32>,
    pub echo_response: Option<(f32, f32)>,
    pub flow: Option<(u32, u16)>,
    pub fragments: Vec<Fragment>,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], NetError> {
        let end = self.pos.checked_add(n).ok_or(NetError::Length)?;
        let slice = self.bytes.get(self.pos..end).ok_or(NetError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }
    fn u16(&mut self) -> Result<u16, NetError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Result<u32, NetError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn f64(&mut self) -> Result<f64, NetError> {
        let b = self.take(8)?;
        let mut a = [0; 8];
        a.copy_from_slice(b);
        Ok(f64::from_le_bytes(a))
    }
}

/// Decodes a session datagram. `key` is the ISAAC key of its sequence and is required when the
/// header sets [`flags::ENCRYPTED_CHECKSUM`].
pub fn decode_packet(bytes: &[u8], key: Option<u32>) -> Result<InboundPacket, NetError> {
    if bytes.len() > MAX_DATAGRAM {
        return Err(NetError::Length);
    }
    let header = PacketHeader::decode(bytes).ok_or(NetError::Truncated)?;
    if bytes.len() != HEADER_LEN + usize::from(header.size) {
        return Err(NetError::Length);
    }
    let known = SESSION_OPTIONAL.iter().fold(
        flags::ENCRYPTED_CHECKSUM | flags::BLOB_FRAGMENTS,
        |all, flag| all | flag,
    );
    if header.flags & !known != 0 {
        return Err(NetError::UnsupportedFlags(header.flags & !known));
    }
    let encrypted = header.flags & flags::ENCRYPTED_CHECKSUM != 0;
    if encrypted && (header.sequence == 0 || key.is_none()) {
        return Err(NetError::UnsupportedFlags(flags::ENCRYPTED_CHECKSUM));
    }
    let body = &bytes[HEADER_LEN..];
    let mut cursor = Cursor {
        bytes: body,
        pos: 0,
    };
    let mut packet = InboundPacket {
        header,
        ..Default::default()
    };
    let mut sum = 0u32;
    for flag in SESSION_OPTIONAL {
        if header.flags & flag == 0 {
            continue;
        }
        let start = cursor.pos;
        match flag {
            flags::NAK | flags::EMPTY_ACK => {
                let count = cursor.u32()?;
                if count >= MAX_SEQUENCE_LIST {
                    return Err(NetError::Length);
                }
                let list = (0..count)
                    .map(|_| cursor.u32())
                    .collect::<Result<Vec<_>, _>>()?;
                if flag == flags::NAK {
                    packet.naks = list;
                } else {
                    packet.empty_acks = list;
                }
            }
            flags::ACK => packet.ack = Some(cursor.u32()?),
            flags::LOGOFF => packet.logoff = true,
            flags::NET_ERROR => packet.net_error = Some((cursor.u32()?, cursor.u32()?)),
            flags::NET_ERROR_DISCONNECT => {
                packet.disconnect = Some((cursor.u32()?, cursor.u32()?));
            }
            flags::TIME_SYNC => packet.time_sync = Some(cursor.f64()?),
            flags::ECHO_REQUEST => packet.echo_request = Some(f32::from_bits(cursor.u32()?)),
            flags::ECHO_RESPONSE => {
                let echoed = f32::from_bits(cursor.u32()?);
                packet.echo_response = Some((echoed, f32::from_bits(cursor.u32()?)));
            }
            flags::FLOW => packet.flow = Some((cursor.u32()?, cursor.u16()?)),
            _ => unreachable!("only listed optional headers are decoded"),
        }
        sum = sum.wrapping_add(checksum32(&body[start..cursor.pos]));
    }
    if header.flags & flags::BLOB_FRAGMENTS != 0 {
        while cursor.pos < body.len() {
            let rest = &body[cursor.pos..];
            if rest.len() < FRAGMENT_HEADER_LEN {
                return Err(NetError::Truncated);
            }
            let size = usize::from(u16::from_le_bytes([rest[10], rest[11]]));
            let segment = cursor.take(size)?;
            if segment.len() < FRAGMENT_HEADER_LEN {
                return Err(NetError::Fragment);
            }
            packet.fragments.push(Fragment::decode(segment)?);
            sum = sum
                .wrapping_add(checksum32(&segment[..FRAGMENT_HEADER_LEN]))
                .wrapping_add(checksum32(&segment[FRAGMENT_HEADER_LEN..]));
        }
        if packet.fragments.is_empty() {
            return Err(NetError::Length);
        }
    }
    if cursor.pos != body.len() {
        return Err(NetError::Length);
    }
    if let (true, Some(key)) = (encrypted, key) {
        sum ^= key;
    }
    if header.checksum != sum.wrapping_add(header.header_checksum()) {
        return Err(NetError::Checksum);
    }
    Ok(packet)
}

/// Encodes a datagram: the given optional headers (flag and bytes, any order) and fragments.
/// `key` masks the body checksum and sets [`flags::ENCRYPTED_CHECKSUM`].
pub(crate) fn encode_packet(
    mut header: PacketHeader,
    optional: &[(u32, Vec<u8>)],
    fragments: &[Fragment],
    key: Option<u32>,
) -> Vec<u8> {
    let mut optional: Vec<&(u32, Vec<u8>)> = optional.iter().collect();
    optional.sort_by_key(|(flag, _)| *flag);
    let mut body = Vec::new();
    let mut sum = 0u32;
    header.flags = 0;
    for (flag, bytes) in optional {
        header.flags |= flag;
        sum = sum.wrapping_add(checksum32(bytes));
        body.extend(bytes);
    }
    if !fragments.is_empty() {
        header.flags |= flags::BLOB_FRAGMENTS;
        for fragment in fragments {
            let encoded = fragment.encode();
            sum = sum
                .wrapping_add(checksum32(&encoded[..FRAGMENT_HEADER_LEN]))
                .wrapping_add(checksum32(&encoded[FRAGMENT_HEADER_LEN..]));
            body.extend(encoded);
        }
    }
    if let Some(key) = key {
        header.flags |= flags::ENCRYPTED_CHECKSUM;
        sum ^= key;
    }
    header.size = body.len() as u16;
    header.checksum = header.header_checksum().wrapping_add(sum);
    let mut out = header.encode().to_vec();
    out.extend(body);
    out
}

/// Decodes a handshake datagram (one plain logon, connect or connect-ack header at sequence 0).
pub(crate) fn decode_handshake(bytes: &[u8]) -> Result<(PacketHeader, &[u8]), NetError> {
    let header = PacketHeader::decode(bytes).ok_or(NetError::Truncated)?;
    if !matches!(
        header.flags,
        flags::LOGON | flags::CONNECT | flags::CONNECT_ACK
    ) || header.sequence != 0
    {
        return Err(NetError::UnsupportedFlags(header.flags));
    }
    if bytes.len() != HEADER_LEN + usize::from(header.size) {
        return Err(NetError::Length);
    }
    let payload = &bytes[HEADER_LEN..];
    if header.checksum != header.header_checksum().wrapping_add(checksum32(payload)) {
        return Err(NetError::Checksum);
    }
    Ok((header, payload))
}

#[cfg(test)]
mod tests {
    use super::{decode_packet, encode_packet, Fragment};
    use crate::header::{flags, PacketHeader};
    use crate::NetError;

    fn header(sequence: u32) -> PacketHeader {
        PacketHeader {
            sequence,
            recipient: 3,
            iteration: 1,
            ..Default::default()
        }
    }

    #[test]
    fn optional_headers_and_fragments_round_trip_with_a_masked_checksum() {
        let fragment = Fragment {
            blob_id: 0x0100_0001_0000_0001,
            count: 1,
            index: 0,
            queue: 3,
            data: vec![1, 2, 3, 4, 5],
        };
        let bytes = encode_packet(
            header(2),
            &[(flags::ACK, 7u32.to_le_bytes().to_vec())],
            &[fragment.clone()],
            Some(0xDEAD_BEEF),
        );
        let packet = decode_packet(&bytes, Some(0xDEAD_BEEF)).unwrap();
        assert_eq!(packet.ack, Some(7));
        assert_eq!(packet.fragments, vec![fragment]);
        assert_eq!(
            packet.header.flags,
            flags::ACK | flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM
        );
        assert_eq!(decode_packet(&bytes, Some(1)), Err(NetError::Checksum));
        for end in 0..bytes.len() {
            assert!(decode_packet(&bytes[..end], Some(0xDEAD_BEEF)).is_err());
        }
    }

    #[test]
    fn control_headers_decode_in_ascending_flag_order() {
        let mut echo = 1.5f32.to_le_bytes().to_vec();
        echo.extend(2.5f32.to_le_bytes());
        let bytes = encode_packet(
            header(0),
            &[
                (flags::ECHO_RESPONSE, echo),
                (flags::TIME_SYNC, 12.25f64.to_le_bytes().to_vec()),
                (
                    flags::NAK,
                    [1u32, 9].iter().flat_map(|v| v.to_le_bytes()).collect(),
                ),
            ],
            &[],
            None,
        );
        let packet = decode_packet(&bytes, None).unwrap();
        assert_eq!(packet.naks, vec![9]);
        assert_eq!(packet.time_sync, Some(12.25));
        assert_eq!(packet.echo_response, Some((1.5, 2.5)));
    }

    #[test]
    fn unknown_flags_and_unkeyed_encryption_are_rejected() {
        let mut bytes = encode_packet(header(0), &[], &[], None);
        bytes[4] = 0x01;
        assert!(matches!(
            decode_packet(&bytes, None),
            Err(NetError::UnsupportedFlags(_))
        ));
        let encrypted = encode_packet(header(5), &[(flags::ACK, vec![0; 4])], &[], Some(9));
        assert!(decode_packet(&encrypted, None).is_err());
    }
}
