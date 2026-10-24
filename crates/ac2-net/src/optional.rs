//! The optional headers between the `ProtoHeader` and the blob fragments.
//!
//! Each type has a mask (its `ProtoHeader` bit) and retail `m_Flags` (template argument 2),
//! which decide how `FlowQueue::CoalesceData` and `NetPacket::AddOptionalHeader` treat it:
//!
//! - `1` (`UNRELIABLE`): carries no sequence; stripped before a packet is stored for resending
//!   (`NetPacket::RemoveDisposableOptionalHeaders`). Every other header makes its packet
//!   reliable (sequenced, encrypted checksum).
//! - `2` (`EXCLUSIVE`): travels alone in its packet; such a packet takes nothing else.
//! - `4` (`CONNECTIONLESS`): accepted before a connection exists.
//! - `8` (`TIME_SENSITIVE`): payload rewritten at every transmission
//!   (`NetPacket::UpdateTimeSensitiveHeaders`).
//! - `0x20` (`SINGLE_TRY`): coalesced only into the first waiting packet, and ends the header
//!   pass of `CoalesceData`.
//! - `0x40`: touches the connection (`RecipientData::TouchConnection`).
//!
//! Decoding is `COptionalHeaderFactory::AddOptionalHeadersToPacket` with each type's
//! `CreateFromStream`: a header that does not fit fails the whole packet.

use crate::header::flags;
use crate::NetError;

/// `m_Flags` bit: no sequence needed.
pub(crate) const UNRELIABLE: u32 = 0x01;
/// `m_Flags` bit: alone in its packet.
pub(crate) const EXCLUSIVE: u32 = 0x02;
/// `m_Flags` bit: accepted without a connection.
pub(crate) const CONNECTIONLESS: u32 = 0x04;
/// `m_Flags` bit: payload rewritten when sent.
pub(crate) const TIME_SENSITIVE: u32 = 0x08;
/// `m_Flags` bit: only tried against the first waiting packet.
pub(crate) const SINGLE_TRY: u32 = 0x20;

/// `sockaddr_in` as it travels: family, big-endian port, address, eight zero bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SockAddrIn(pub [u8; 16]);

impl SockAddrIn {
    pub fn port(&self) -> u16 {
        u16::from_be_bytes([self.0[2], self.0[3]])
    }

    pub fn ip(&self) -> [u8; 4] {
        [self.0[4], self.0[5], self.0[6], self.0[7]]
    }
}

/// `NetError`: a string-table token and its table (`NetError::Pack`, 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NetErrorId {
    pub string_id: u32,
    pub table_id: u32,
}

impl NetErrorId {
    /// `ID_NetError_None`.
    pub const NONE: Self = Self {
        string_id: 0,
        table_id: 0,
    };
}

/// `CConnectHeader`'s 32-byte structure: server time, cookie, net id and the two ISAAC seeds,
/// with four bytes of tail padding.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Connect {
    pub server_time: f64,
    pub cookie: u64,
    /// The id this client puts in `ProtoHeader::recID_` of everything it sends
    /// (`ReceiverData::m_NetID`).
    pub net_id: u32,
    /// Seeds the client's incoming generator (`ReceiverData::InitCrypto`).
    pub outgoing_seed: u32,
    /// Seeds the client's outgoing generator.
    pub incoming_seed: u32,
    pub padding: u32,
}

/// `CReferralStruct`: cookie, address, server id and padding, 32 bytes with tail padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Referral {
    pub cookie: u64,
    pub addr: SockAddrIn,
    pub server_id: u16,
    pub padding: u16,
    pub tail: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptionalHeader {
    /// `CServerSwitchStruct`: the switch timestamp and `ServerSwitchType` (0 world, 1 logon).
    ServerSwitch {
        stamp: u32,
        kind: u32,
    },
    LogonServerAddr(SockAddrIn),
    Empty,
    Referral(Referral),
    Nak(Vec<u32>),
    EmptyAck(Vec<u32>),
    Ack(u32),
    Logoff,
    /// The packed `CLogonHeader` payload.
    Logon(Vec<u8>),
    WorldLoginRequest(u64),
    Connect(Connect),
    ConnectAck(u64),
    NetError(NetErrorId),
    NetErrorDisconnect(NetErrorId),
    /// `CICMDCommandStruct`: command and parameter.
    Cicmd {
        cmd: u32,
        param: u32,
    },
    TimeSync(f64),
    EchoRequest(f32),
    /// The echoed request payload and `HoldingTime`, the seconds the response waited.
    EchoResponse {
        echo: f32,
        holding: f32,
    },
    /// `CFlowStruct`: bytes received in the interval, and the interval.
    Flow {
        bytes: u32,
        interval: u16,
    },
}

/// Every type the client registers, ascending (`COptionalHeaderFactory` order).
pub(crate) const MASKS: [u32; 19] = [
    flags::SERVER_SWITCH,
    flags::LOGON_SERVER_ADDR,
    flags::EMPTY_HEADER,
    flags::REFERRAL,
    flags::NAK,
    flags::EMPTY_ACK,
    flags::ACK,
    flags::LOGOFF,
    flags::LOGON,
    flags::WORLD_LOGIN_REQUEST,
    flags::CONNECT,
    flags::CONNECT_ACK,
    flags::NET_ERROR,
    flags::NET_ERROR_DISCONNECT,
    flags::CICMD,
    flags::TIME_SYNC,
    flags::ECHO_REQUEST,
    flags::ECHO_RESPONSE,
    flags::FLOW,
];

/// `CSeqIDListHeader` lists hold at most `0x72` ids.
pub(crate) const MAX_SEQUENCE_LIST: usize = 0x72;

impl OptionalHeader {
    /// The `ProtoHeader` bit.
    pub fn mask(&self) -> u32 {
        match self {
            Self::ServerSwitch { .. } => flags::SERVER_SWITCH,
            Self::LogonServerAddr(_) => flags::LOGON_SERVER_ADDR,
            Self::Empty => flags::EMPTY_HEADER,
            Self::Referral(_) => flags::REFERRAL,
            Self::Nak(_) => flags::NAK,
            Self::EmptyAck(_) => flags::EMPTY_ACK,
            Self::Ack(_) => flags::ACK,
            Self::Logoff => flags::LOGOFF,
            Self::Logon(_) => flags::LOGON,
            Self::WorldLoginRequest(_) => flags::WORLD_LOGIN_REQUEST,
            Self::Connect(_) => flags::CONNECT,
            Self::ConnectAck(_) => flags::CONNECT_ACK,
            Self::NetError(_) => flags::NET_ERROR,
            Self::NetErrorDisconnect(_) => flags::NET_ERROR_DISCONNECT,
            Self::Cicmd { .. } => flags::CICMD,
            Self::TimeSync(_) => flags::TIME_SYNC,
            Self::EchoRequest(_) => flags::ECHO_REQUEST,
            Self::EchoResponse { .. } => flags::ECHO_RESPONSE,
            Self::Flow { .. } => flags::FLOW,
        }
    }

    /// Retail `m_Flags` (the second template argument).
    pub(crate) fn retail_flags(&self) -> u32 {
        match self {
            Self::ServerSwitch { .. } => 0x60,
            Self::LogonServerAddr(_)
            | Self::Empty
            | Self::Logon(_)
            | Self::WorldLoginRequest(_)
            | Self::NetError(_)
            | Self::Cicmd { .. } => 7,
            Self::Referral(_) => 0x4000_0062,
            Self::Nak(_) | Self::EmptyAck(_) => 0x21,
            Self::Ack(_) => 1,
            Self::Logoff => 3,
            Self::Connect(_) | Self::ConnectAck(_) => 0x2000_0007,
            Self::NetErrorDisconnect(_) => 2,
            Self::TimeSync(_) | Self::EchoRequest(_) | Self::EchoResponse { .. } => 0x18,
            Self::Flow { .. } => 0x10,
        }
    }

    /// The header's wire bytes (`m_pData`, `m_cbData`).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let ids = |out: &mut Vec<u8>, list: &[u32]| {
            out.extend((list.len() as u32).to_le_bytes());
            list.iter().for_each(|id| out.extend(id.to_le_bytes()));
        };
        match self {
            Self::ServerSwitch { stamp, kind } => {
                out.extend(stamp.to_le_bytes());
                out.extend(kind.to_le_bytes());
            }
            Self::LogonServerAddr(addr) => out.extend(addr.0),
            Self::Empty | Self::Logoff => {}
            Self::Referral(r) => {
                out.extend(r.cookie.to_le_bytes());
                out.extend(r.addr.0);
                out.extend(r.server_id.to_le_bytes());
                out.extend(r.padding.to_le_bytes());
                out.extend(r.tail.to_le_bytes());
            }
            Self::Nak(list) | Self::EmptyAck(list) => ids(&mut out, list),
            Self::Ack(seq) => out.extend(seq.to_le_bytes()),
            Self::Logon(payload) => out.extend(payload),
            Self::WorldLoginRequest(cookie) | Self::ConnectAck(cookie) => {
                out.extend(cookie.to_le_bytes())
            }
            Self::Connect(c) => {
                out.extend(c.server_time.to_le_bytes());
                out.extend(c.cookie.to_le_bytes());
                out.extend(c.net_id.to_le_bytes());
                out.extend(c.outgoing_seed.to_le_bytes());
                out.extend(c.incoming_seed.to_le_bytes());
                out.extend(c.padding.to_le_bytes());
            }
            Self::NetError(e) | Self::NetErrorDisconnect(e) => {
                out.extend(e.string_id.to_le_bytes());
                out.extend(e.table_id.to_le_bytes());
            }
            Self::Cicmd { cmd, param } => {
                out.extend(cmd.to_le_bytes());
                out.extend(param.to_le_bytes());
            }
            Self::TimeSync(time) => out.extend(time.to_le_bytes()),
            Self::EchoRequest(time) => out.extend(time.to_le_bytes()),
            Self::EchoResponse { echo, holding } => {
                out.extend(echo.to_le_bytes());
                out.extend(holding.to_le_bytes());
            }
            Self::Flow { bytes, interval } => {
                out.extend(bytes.to_le_bytes());
                out.extend(interval.to_le_bytes());
            }
        }
        out
    }

    /// `CreateFromStream` for `mask` at `body[*pos..]`, advancing `*pos`.
    pub(crate) fn decode(mask: u32, body: &[u8], pos: &mut usize) -> Result<Self, NetError> {
        let mut take = |n: usize| -> Result<&[u8], NetError> {
            let end = pos.checked_add(n).ok_or(NetError::Length)?;
            let slice = body.get(*pos..end).ok_or(NetError::Truncated)?;
            *pos = end;
            Ok(slice)
        };
        let u32_at = |b: &[u8], i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        let u64_at = |b: &[u8], i: usize| {
            let mut a = [0; 8];
            a.copy_from_slice(&b[i..i + 8]);
            u64::from_le_bytes(a)
        };
        Ok(match mask {
            flags::SERVER_SWITCH => {
                let b = take(8)?;
                Self::ServerSwitch {
                    stamp: u32_at(b, 0),
                    kind: u32_at(b, 4),
                }
            }
            flags::LOGON_SERVER_ADDR => {
                let mut addr = [0; 16];
                addr.copy_from_slice(take(16)?);
                Self::LogonServerAddr(SockAddrIn(addr))
            }
            flags::EMPTY_HEADER => Self::Empty,
            flags::REFERRAL => {
                let b = take(32)?;
                let mut addr = [0; 16];
                addr.copy_from_slice(&b[8..24]);
                Self::Referral(Referral {
                    cookie: u64_at(b, 0),
                    addr: SockAddrIn(addr),
                    server_id: u16::from_le_bytes([b[24], b[25]]),
                    padding: u16::from_le_bytes([b[26], b[27]]),
                    tail: u32_at(b, 28),
                })
            }
            flags::NAK | flags::EMPTY_ACK => {
                let count = u32_at(take(4)?, 0) as usize;
                if count > MAX_SEQUENCE_LIST {
                    return Err(NetError::Length);
                }
                let b = take(count * 4)?;
                let list = (0..count).map(|i| u32_at(b, i * 4)).collect();
                if mask == flags::NAK {
                    Self::Nak(list)
                } else {
                    Self::EmptyAck(list)
                }
            }
            flags::ACK => Self::Ack(u32_at(take(4)?, 0)),
            flags::LOGOFF => Self::Logoff,
            flags::WORLD_LOGIN_REQUEST => Self::WorldLoginRequest(u64_at(take(8)?, 0)),
            flags::CONNECT => {
                let b = take(32)?;
                Self::Connect(Connect {
                    server_time: f64::from_bits(u64_at(b, 0)),
                    cookie: u64_at(b, 8),
                    net_id: u32_at(b, 16),
                    outgoing_seed: u32_at(b, 20),
                    incoming_seed: u32_at(b, 24),
                    padding: u32_at(b, 28),
                })
            }
            flags::CONNECT_ACK => Self::ConnectAck(u64_at(take(8)?, 0)),
            flags::NET_ERROR | flags::NET_ERROR_DISCONNECT => {
                let b = take(8)?;
                let error = NetErrorId {
                    string_id: u32_at(b, 0),
                    table_id: u32_at(b, 4),
                };
                if mask == flags::NET_ERROR {
                    Self::NetError(error)
                } else {
                    Self::NetErrorDisconnect(error)
                }
            }
            flags::CICMD => {
                let b = take(8)?;
                Self::Cicmd {
                    cmd: u32_at(b, 0),
                    param: u32_at(b, 4),
                }
            }
            flags::TIME_SYNC => Self::TimeSync(f64::from_bits(u64_at(take(8)?, 0))),
            flags::ECHO_REQUEST => Self::EchoRequest(f32::from_bits(u32_at(take(4)?, 0))),
            flags::ECHO_RESPONSE => {
                let b = take(8)?;
                Self::EchoResponse {
                    echo: f32::from_bits(u32_at(b, 0)),
                    holding: f32::from_bits(u32_at(b, 4)),
                }
            }
            flags::FLOW => {
                let b = take(6)?;
                Self::Flow {
                    bytes: u32_at(b, 0),
                    interval: u16::from_le_bytes([b[4], b[5]]),
                }
            }
            // `CLogonHeader` is only ever sent by a client.
            _ => return Err(NetError::UnsupportedFlags(mask)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Connect, NetErrorId, OptionalHeader, Referral, SockAddrIn, MASKS};
    use crate::header::flags;

    fn round_trip(header: OptionalHeader, len: usize) {
        let bytes = header.encode();
        assert_eq!(bytes.len(), len, "{header:?}");
        let mut pos = 0;
        assert_eq!(
            OptionalHeader::decode(header.mask(), &bytes, &mut pos).unwrap(),
            header
        );
        assert_eq!(pos, len);
        if len > 0 {
            let mut pos = 0;
            assert!(OptionalHeader::decode(header.mask(), &bytes[..len - 1], &mut pos).is_err());
        }
    }

    #[test]
    fn every_registered_header_has_its_retail_size() {
        let addr = SockAddrIn([2, 0, 0x23, 0x28, 127, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!((addr.port(), addr.ip()), (9000, [127, 0, 0, 1]));
        round_trip(OptionalHeader::ServerSwitch { stamp: 5, kind: 1 }, 8);
        round_trip(OptionalHeader::LogonServerAddr(addr), 16);
        round_trip(OptionalHeader::Empty, 0);
        round_trip(
            OptionalHeader::Referral(Referral {
                cookie: 9,
                addr,
                server_id: 3,
                padding: 0,
                tail: 0,
            }),
            32,
        );
        round_trip(OptionalHeader::Nak(vec![4, 9]), 12);
        round_trip(OptionalHeader::EmptyAck(vec![]), 4);
        round_trip(OptionalHeader::Ack(7), 4);
        round_trip(OptionalHeader::Logoff, 0);
        round_trip(OptionalHeader::WorldLoginRequest(11), 8);
        round_trip(
            OptionalHeader::Connect(Connect {
                server_time: 2.5,
                cookie: 3,
                net_id: 4,
                outgoing_seed: 5,
                incoming_seed: 6,
                padding: 0,
            }),
            32,
        );
        round_trip(OptionalHeader::ConnectAck(3), 8);
        round_trip(OptionalHeader::NetError(NetErrorId::NONE), 8);
        round_trip(OptionalHeader::NetErrorDisconnect(NetErrorId::NONE), 8);
        round_trip(OptionalHeader::Cicmd { cmd: 1, param: 0 }, 8);
        round_trip(OptionalHeader::TimeSync(100.25), 8);
        round_trip(OptionalHeader::EchoRequest(1.5), 4);
        round_trip(
            OptionalHeader::EchoResponse {
                echo: 1.5,
                holding: 0.0,
            },
            8,
        );
        round_trip(
            OptionalHeader::Flow {
                bytes: 484,
                interval: 3,
            },
            6,
        );
        assert!(MASKS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn sequence_lists_hold_at_most_0x72_ids() {
        let mut bytes = 0x73u32.to_le_bytes().to_vec();
        bytes.extend(vec![0; 0x73 * 4]);
        let mut pos = 0;
        assert!(OptionalHeader::decode(flags::NAK, &bytes, &mut pos).is_err());
        let full = OptionalHeader::Nak((0..0x72).collect());
        let mut pos = 0;
        assert_eq!(
            OptionalHeader::decode(flags::NAK, &full.encode(), &mut pos).unwrap(),
            full
        );
    }
}
