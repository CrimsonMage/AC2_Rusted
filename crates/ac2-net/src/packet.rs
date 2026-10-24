//! `NetPacket`: one datagram's optional headers and blob fragments, built for sending
//! (`NetPacket::AddOptionalHeader`, `AddFrag`, `ComputeChecksum`, `SharedNet::SendPacket`) and
//! parsed on receipt (`SharedNet::SplitPacketData`).
//!
//! The checksum of a datagram is the body sum plus the header sum
//! (`SharedNet::ChecksumHeader`). The body sum adds `checksum32` of each optional header's bytes
//! and of each fragment's 16-byte header and data separately; on a reliable packet it is XORed
//! with the sequence's ISAAC key first (`FlowQueue::EncryptChecksum`).

use crate::checksum::checksum32;
use crate::header::{flags, PacketHeader, HEADER_LEN};
use crate::optional::{
    OptionalHeader, CONNECTIONLESS, EXCLUSIVE, MASKS, SINGLE_TRY, TIME_SENSITIVE, UNRELIABLE,
};
use crate::NetError;

/// Size of `BlobFragHeader_t`: u64 blob id, u16 fragment count, u16 size including this header,
/// u16 fragment index, u16 queue.
pub const FRAGMENT_HEADER_LEN: usize = 16;
/// Largest `ProtoHeader::datalen_` the client builds: one full fragment (`0x1d0`).
pub const MAX_PACKET_BODY: usize = 0x1D0;
/// Largest datagram read (`SharedNet::ReadNextPacket` reads into `0xffe0` bytes).
pub const MAX_DATAGRAM: usize = 0xFFE0;

/// `NetPacket::flags_`: the checksum holds its encrypted form.
pub(crate) const PACKET_ENCRYPTED: u32 = 1;
/// `NetPacket::flags_`: a time-sensitive header is aboard.
pub(crate) const PACKET_TIME_SENSITIVE: u32 = 2;
/// `NetPacket::flags_`: the packet needs a sequence.
pub(crate) const PACKET_RELIABLE: u32 = 4;
/// `NetPacket::flags_`: a single-try header is aboard (flow control never holds it).
pub(crate) const PACKET_SINGLE_TRY: u32 = 8;

/// One blob fragment: `BlobFragHeader_t` and its data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub blob_id: u64,
    pub count: u16,
    pub index: u16,
    pub queue: u16,
    pub data: Vec<u8>,
}

impl Fragment {
    /// `blobFragSize`: the data plus the 16-byte header.
    pub fn size(&self) -> usize {
        self.data.len() + FRAGMENT_HEADER_LEN
    }

    pub(crate) fn header_bytes(&self) -> [u8; FRAGMENT_HEADER_LEN] {
        let mut out = [0; FRAGMENT_HEADER_LEN];
        out[0..8].copy_from_slice(&self.blob_id.to_le_bytes());
        out[8..10].copy_from_slice(&self.count.to_le_bytes());
        out[10..12].copy_from_slice(&(self.size() as u16).to_le_bytes());
        out[12..14].copy_from_slice(&self.index.to_le_bytes());
        out[14..16].copy_from_slice(&self.queue.to_le_bytes());
        out
    }

    /// The two `checksum32` segments `NetPacket::AddFrag` adds.
    fn checksum(&self) -> u32 {
        checksum32(&self.header_bytes()).wrapping_add(checksum32(&self.data))
    }

    /// `BlobFrag::BlobFrag` (receive): the header's size must lie in `16..=0x1d0` and fit.
    fn decode(rest: &[u8]) -> Option<Self> {
        let size = usize::from(u16::from_le_bytes([*rest.get(10)?, *rest.get(11)?]));
        if !(FRAGMENT_HEADER_LEN..=MAX_PACKET_BODY).contains(&size) || size > rest.len() {
            return None;
        }
        let u16_at = |i: usize| u16::from_le_bytes([rest[i], rest[i + 1]]);
        let mut id = [0; 8];
        id.copy_from_slice(&rest[..8]);
        Some(Self {
            blob_id: u64::from_le_bytes(id),
            count: u16_at(8),
            index: u16_at(12),
            queue: u16_at(14),
            data: rest[FRAGMENT_HEADER_LEN..size].to_vec(),
        })
    }
}

/// An optional header queued for sending, with the local time it was built at
/// (`CEchoResponseHeader::m_LocalTimeConstructed`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OutHeader {
    pub header: OptionalHeader,
    pub constructed: f64,
    pub bytes: Vec<u8>,
}

impl OutHeader {
    pub fn new(header: OptionalHeader, local_time: f64) -> Self {
        let bytes = header.encode();
        Self {
            header,
            constructed: local_time,
            bytes,
        }
    }

    pub fn mask(&self) -> u32 {
        self.header.mask()
    }

    pub fn retail_flags(&self) -> u32 {
        self.header.retail_flags()
    }

    /// `UpdateTimeSensitivePayload` of the three time-sensitive types.
    fn update(&mut self, local_time: f64, server_time: f64) {
        match &mut self.header {
            // `CTimeSyncHeader`: `Timer::get_elapsed_time() + Timer::base_time`.
            OptionalHeader::TimeSync(time) => *time = server_time,
            // `CEchoRequestHeader`: `(float)Timer::local_time`.
            OptionalHeader::EchoRequest(time) => *time = local_time as f32,
            // `CEchoResponseHeader`: how long the response waited.
            OptionalHeader::EchoResponse { holding, .. } => {
                *holding = (local_time - self.constructed) as f32;
            }
            _ => {}
        }
        self.bytes = self.header.encode();
    }
}

/// `NetPacket` on the send side.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OutPacket {
    pub headers: Vec<OutHeader>,
    pub fragments: Vec<Fragment>,
    pub size: usize,
    pub sequence: u32,
    pub crypto_key: u32,
    pub checksum: u32,
    pub flags: u32,
}

impl OutPacket {
    /// `NetPacket::CreateForSend`.
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            fragments: Vec::new(),
            size: 0,
            sequence: 0,
            crypto_key: 0,
            checksum: 0,
            flags: 0,
        }
    }

    /// `NetPacket::AddOptionalHeader`: inserted after every header of lower or equal mask.
    pub fn add_header(&mut self, header: OutHeader) {
        let at = self
            .headers
            .iter()
            .rposition(|h| h.mask() <= header.mask())
            .map_or(0, |i| i + 1);
        self.size += header.bytes.len();
        self.checksum = self.checksum.wrapping_add(checksum32(&header.bytes));
        let retail = header.retail_flags();
        if retail & TIME_SENSITIVE != 0 {
            self.flags |= PACKET_TIME_SENSITIVE;
        }
        if retail & UNRELIABLE == 0 {
            self.flags |= PACKET_RELIABLE;
        }
        if retail & SINGLE_TRY != 0 {
            self.flags |= PACKET_SINGLE_TRY;
        }
        self.headers.insert(at, header);
    }

    /// `NetPacket::AddFrag`.
    pub fn add_fragment(&mut self, fragment: Fragment) {
        self.size += fragment.size();
        self.checksum = self.checksum.wrapping_add(fragment.checksum());
        self.flags |= PACKET_RELIABLE;
        self.fragments.push(fragment);
    }

    /// `NetPacket::ApplySpecialFrags`: the OR of every header mask.
    pub fn header_masks(&self) -> u32 {
        self.headers.iter().fold(0, |all, h| all | h.mask())
    }

    /// Whether this packet's only header is exclusive (`CoalesceData` adds nothing to it).
    pub fn closed(&self) -> bool {
        self.headers.len() == 1 && self.headers[0].retail_flags() & EXCLUSIVE != 0
    }

    /// `NetPacket::ComputeChecksum`.
    pub fn compute_checksum(&mut self) {
        self.flags &= !PACKET_ENCRYPTED;
        self.checksum = self
            .headers
            .iter()
            .map(|h| checksum32(&h.bytes))
            .chain(self.fragments.iter().map(Fragment::checksum))
            .fold(0, u32::wrapping_add);
    }

    /// `NetPacket::UpdateTimeSensitiveHeaders`: rewrite time-sensitive payloads and drop the
    /// stale checksum.
    pub fn update_time_sensitive(&mut self, local_time: f64, server_time: f64) {
        if self.flags & PACKET_TIME_SENSITIVE == 0 {
            return;
        }
        let mut any = false;
        for header in &mut self.headers {
            if header.retail_flags() & TIME_SENSITIVE != 0 {
                header.update(local_time, server_time);
                any = true;
            }
        }
        if any {
            self.flags &= !PACKET_ENCRYPTED;
            self.checksum = 0;
        } else {
            self.flags &= !PACKET_TIME_SENSITIVE;
        }
    }

    /// `NetPacket::RemoveDisposableOptionalHeaders`: drop unreliable headers before storing the
    /// packet for resending.
    pub fn remove_disposable(&mut self) {
        let before = self.headers.len();
        let mut removed = 0;
        self.headers.retain(|h| {
            let keep = h.retail_flags() & UNRELIABLE == 0;
            if !keep {
                removed += h.bytes.len();
            }
            keep
        });
        self.size -= removed;
        if self.headers.len() != before {
            self.checksum = 0;
            self.flags &= !PACKET_ENCRYPTED;
        }
    }

    /// `SharedNet::SendPacket`: the `ProtoHeader` (its checksum field holding the body sum
    /// gains the header sum), the header bytes, then each fragment.
    pub fn datagram(&self, mut header: PacketHeader) -> Vec<u8> {
        header.checksum = header.checksum.wrapping_add(header.header_checksum());
        let mut out = Vec::with_capacity(HEADER_LEN + self.size);
        out.extend(header.encode());
        for h in &self.headers {
            out.extend(&h.bytes);
        }
        for fragment in &self.fragments {
            out.extend(fragment.header_bytes());
            out.extend(&fragment.data);
        }
        out
    }
}

/// A received datagram that passed `SharedNet::SplitPacketData`.
#[derive(Debug, Clone, PartialEq)]
pub struct InboundPacket {
    pub header: PacketHeader,
    pub optional: Vec<OptionalHeader>,
    pub fragments: Vec<Fragment>,
}

/// `SharedNet::SplitPacketData`. `checksum` is the header's checksum less the header sum, and
/// decrypted (`ReceiverData::Decrypt`) when the packet is encrypted; `connected` is whether the
/// sender has a `ReceiverData`.
pub fn split_packet(
    bytes: &[u8],
    checksum: u32,
    connected: bool,
) -> Result<InboundPacket, NetError> {
    let header = PacketHeader::decode(bytes).ok_or(NetError::Truncated)?;
    let body = bytes.get(HEADER_LEN..).ok_or(NetError::Truncated)?;
    let mut pos = 0;
    let mut optional = Vec::new();
    let mut sum = 0u32;
    let mut reliable = false;
    let mut remaining = header.flags;
    // `COptionalHeaderFactory::AddOptionalHeadersToPacket`: ascending masks; any bit above the
    // packet-state byte that no type claims fails the packet.
    for mask in MASKS {
        if remaining & 0xFFFF_FF00 == 0 {
            break;
        }
        if header.flags & mask == 0 {
            continue;
        }
        remaining &= !mask;
        let start = pos;
        let decoded = OptionalHeader::decode(mask, body, &mut pos)?;
        sum = sum.wrapping_add(checksum32(&body[start..pos]));
        reliable |= decoded.retail_flags() & UNRELIABLE == 0;
        optional.push(decoded);
    }
    if remaining & 0xFFFF_FF00 != 0 {
        return Err(NetError::UnsupportedFlags(remaining & 0xFFFF_FF00));
    }
    for decoded in &optional {
        let retail = decoded.retail_flags();
        if retail & EXCLUSIVE != 0 && optional.len() > 1 {
            return Err(NetError::UnsupportedFlags(decoded.mask()));
        }
        if !connected && retail & CONNECTIONLESS == 0 {
            return Err(NetError::Sequence);
        }
    }
    let mut fragments = Vec::new();
    if header.flags & flags::BLOB_FRAGMENTS != 0 {
        if !connected {
            return Err(NetError::Sequence);
        }
        // Fragments are read while more than a bare fragment header remains.
        while body.len() > pos && body.len() - pos > FRAGMENT_HEADER_LEN {
            let fragment = Fragment::decode(&body[pos..]).ok_or(NetError::Fragment)?;
            pos += fragment.size();
            sum = sum.wrapping_add(fragment.checksum());
            reliable = true;
            fragments.push(fragment);
        }
    }
    if pos != body.len() {
        return Err(NetError::Length);
    }
    let encrypted = header.flags & flags::ENCRYPTED_CHECKSUM != 0;
    if encrypted != reliable || (encrypted && header.sequence == 0) {
        return Err(NetError::UnsupportedFlags(flags::ENCRYPTED_CHECKSUM));
    }
    if sum != checksum {
        return Err(NetError::Checksum);
    }
    Ok(InboundPacket {
        header,
        optional,
        fragments,
    })
}

#[cfg(test)]
mod tests {
    use super::{split_packet, Fragment, OutHeader, OutPacket, PACKET_RELIABLE};
    use crate::checksum::checksum32;
    use crate::header::{flags, PacketHeader};
    use crate::optional::OptionalHeader;
    use crate::NetError;

    fn fragment(data: &[u8]) -> Fragment {
        Fragment {
            blob_id: 0x0300_0001_0000_0000,
            count: 1,
            index: 0,
            queue: 4,
            data: data.to_vec(),
        }
    }

    #[test]
    fn headers_insert_in_ascending_mask_order_and_set_packet_flags() {
        let mut packet = OutPacket::new();
        packet.add_header(OutHeader::new(OptionalHeader::TimeSync(1.0), 0.0));
        packet.add_header(OutHeader::new(OptionalHeader::Ack(5), 0.0));
        packet.add_header(OutHeader::new(OptionalHeader::EchoRequest(0.0), 0.0));
        let masks: Vec<u32> = packet.headers.iter().map(OutHeader::mask).collect();
        assert_eq!(masks, [flags::ACK, flags::TIME_SYNC, flags::ECHO_REQUEST]);
        assert_eq!(packet.size, 4 + 8 + 4);
        assert_eq!(packet.flags, 2 | PACKET_RELIABLE);
        packet.remove_disposable();
        assert_eq!(packet.headers.len(), 2);
        assert_eq!((packet.size, packet.checksum), (12, 0));
    }

    #[test]
    fn an_ack_alone_is_unreliable_and_parses_back() {
        let mut packet = OutPacket::new();
        packet.add_header(OutHeader::new(OptionalHeader::Ack(9), 0.0));
        assert_eq!(packet.flags, 0);
        let body_sum = packet.checksum;
        assert_eq!(body_sum, checksum32(&9u32.to_le_bytes()));
        let header = PacketHeader {
            sequence: 3,
            flags: flags::ACK,
            checksum: body_sum,
            recipient: 5,
            interval: 2,
            size: packet.size as u16,
            iteration: 1,
        };
        let bytes = packet.datagram(header);
        assert_eq!(bytes.len(), 24);
        let wire = PacketHeader::decode(&bytes).unwrap();
        assert_eq!(
            wire.checksum,
            body_sum.wrapping_add(header.header_checksum())
        );
        let parsed = split_packet(&bytes, body_sum, true).unwrap();
        assert_eq!(parsed.optional, vec![OptionalHeader::Ack(9)]);
        assert_eq!(
            split_packet(&bytes, body_sum + 1, true),
            Err(NetError::Checksum)
        );
    }

    #[test]
    fn reliability_must_match_the_encrypted_bit() {
        let mut packet = OutPacket::new();
        packet.add_fragment(fragment(b"abcd"));
        let header = PacketHeader {
            sequence: 2,
            flags: flags::BLOB_FRAGMENTS,
            checksum: packet.checksum,
            size: packet.size as u16,
            ..Default::default()
        };
        let plain = packet.datagram(header);
        assert!(split_packet(&plain, packet.checksum, true).is_err());
        let encrypted = packet.datagram(PacketHeader {
            flags: flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM,
            ..header
        });
        let parsed = split_packet(&encrypted, packet.checksum, true).unwrap();
        assert_eq!(parsed.fragments, vec![fragment(b"abcd")]);
        // Fragments need a connection.
        assert!(split_packet(&encrypted, packet.checksum, false).is_err());
    }

    #[test]
    fn unknown_masks_exclusive_headers_and_trailing_bytes_fail() {
        let header = PacketHeader {
            flags: 0x0000_0080 | 0x1000_0000,
            ..Default::default()
        };
        let bytes = OutPacket::new().datagram(header);
        assert!(matches!(
            split_packet(&bytes, 0, true),
            Err(NetError::UnsupportedFlags(0x1000_0000))
        ));
        let mut packet = OutPacket::new();
        packet.add_header(OutHeader::new(OptionalHeader::Logoff, 0.0));
        packet.add_header(OutHeader::new(OptionalHeader::Ack(1), 0.0));
        let header = PacketHeader {
            flags: flags::ACK | flags::LOGOFF,
            size: 4,
            ..Default::default()
        };
        let bytes = packet.datagram(header);
        assert!(split_packet(&bytes, packet.checksum, true).is_err());
        // Encryption without reliable content fails; so does a bare 16-byte fragment tail.
        let header = PacketHeader {
            flags: flags::BLOB_FRAGMENTS | flags::ENCRYPTED_CHECKSUM,
            sequence: 2,
            ..Default::default()
        };
        let empty = OutPacket::new().datagram(header);
        assert!(split_packet(&empty, 0, true).is_err());
        let mut tail = OutPacket::new();
        tail.add_fragment(fragment(b""));
        let bytes = tail.datagram(PacketHeader { size: 16, ..header });
        assert_eq!(
            split_packet(&bytes, tail.checksum, true),
            Err(NetError::Length)
        );
    }
}
