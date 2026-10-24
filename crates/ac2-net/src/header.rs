use crate::checksum::checksum32;

/// Size of `ProtoHeader`.
pub const HEADER_LEN: usize = 20;

/// Header flag bits: `NetPacket` state bits and the optional-header masks
/// (`COptionalHeaderFactory::RegisterOptionalHeaderType`).
pub mod flags {
    /// The checksum's body part is XORed with the sequence's ISAAC key.
    pub const ENCRYPTED_CHECKSUM: u32 = 0x0000_0002;
    /// Blob fragments follow the optional headers.
    pub const BLOB_FRAGMENTS: u32 = 0x0000_0004;
    /// `CSeqIDListHeader<4096>`: sequences the sender is missing.
    pub const NAK: u32 = 0x0000_1000;
    /// `CSeqIDListHeader<8192>`: sequences known to carry nothing.
    pub const EMPTY_ACK: u32 = 0x0000_2000;
    /// `COnePrimHeader<16384, u32>`: highest contiguous sequence received.
    pub const ACK: u32 = 0x0000_4000;
    /// `CEmptyHeader<32768>`: the sender is leaving.
    pub const LOGOFF: u32 = 0x0000_8000;
    /// `CLogonHeader`.
    pub const LOGON: u32 = 0x0001_0000;
    /// `CConnectHeader`.
    pub const CONNECT: u32 = 0x0004_0000;
    /// `COnePrimHeader<524288, u64>`: the connect cookie echoed back.
    pub const CONNECT_ACK: u32 = 0x0008_0000;
    /// `CPackObjHeader<NetError, 1048576>`.
    pub const NET_ERROR: u32 = 0x0010_0000;
    /// `CPackObjHeader<NetError, 2097152>`: the server is closing the connection.
    pub const NET_ERROR_DISCONNECT: u32 = 0x0020_0000;
    /// `CTimeSyncHeader`: f64 server time.
    pub const TIME_SYNC: u32 = 0x0100_0000;
    /// `CEchoRequestHeader`: f32 sender time.
    pub const ECHO_REQUEST: u32 = 0x0200_0000;
    /// `CEchoResponseHeader`: the echoed f32 and the responder's f32.
    pub const ECHO_RESPONSE: u32 = 0x0400_0000;
    /// `COnePrimHeader<134217728, CFlowStruct>`: u32 bytes and u16 interval.
    pub const FLOW: u32 = 0x0800_0000;
}

/// `ProtoHeader`: little-endian sequence, flags, checksum, recipient, interval, payload size and
/// iteration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PacketHeader {
    pub sequence: u32,
    pub flags: u32,
    pub checksum: u32,
    pub recipient: u16,
    pub interval: u16,
    pub size: u16,
    pub iteration: u16,
}

impl PacketHeader {
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let b = bytes.get(..HEADER_LEN)?;
        let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
        let u32_at = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        Some(Self {
            sequence: u32_at(0),
            flags: u32_at(4),
            checksum: u32_at(8),
            recipient: u16_at(12),
            interval: u16_at(14),
            size: u16_at(16),
            iteration: u16_at(18),
        })
    }

    pub fn encode(self) -> [u8; HEADER_LEN] {
        let mut b = [0; HEADER_LEN];
        b[0..4].copy_from_slice(&self.sequence.to_le_bytes());
        b[4..8].copy_from_slice(&self.flags.to_le_bytes());
        b[8..12].copy_from_slice(&self.checksum.to_le_bytes());
        b[12..14].copy_from_slice(&self.recipient.to_le_bytes());
        b[14..16].copy_from_slice(&self.interval.to_le_bytes());
        b[16..18].copy_from_slice(&self.size.to_le_bytes());
        b[18..20].copy_from_slice(&self.iteration.to_le_bytes());
        b
    }

    /// `SharedNet::ChecksumHeader`: the header summed with `0xBADD70DD` in the checksum field.
    pub fn header_checksum(self) -> u32 {
        checksum32(
            &Self {
                checksum: 0xBADD_70DD,
                ..self
            }
            .encode(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PacketHeader;

    #[test]
    fn header_round_trips_in_wire_order() {
        let header = PacketHeader {
            sequence: 0x0102_0304,
            flags: 0x0506_0708,
            checksum: 0x090A_0B0C,
            recipient: 0x0D0E,
            interval: 0x0F10,
            size: 0x1112,
            iteration: 0x1314,
        };
        let bytes = header.encode();
        assert_eq!(&bytes[..4], &[4, 3, 2, 1]);
        assert_eq!(&bytes[18..], &[0x14, 0x13]);
        assert_eq!(PacketHeader::decode(&bytes), Some(header));
        assert_eq!(PacketHeader::decode(&bytes[..19]), None);
    }

    #[test]
    fn header_checksum_ignores_the_stored_checksum() {
        let a = PacketHeader {
            sequence: 2,
            checksum: 1,
            ..Default::default()
        };
        let b = PacketHeader { checksum: 99, ..a };
        assert_eq!(a.header_checksum(), b.header_checksum());
    }
}
