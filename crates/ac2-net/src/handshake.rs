//! The logon handshake.
//!
//! The client sends `CLogonHeader` at sequence 0 with recipient, interval and iteration 0. The
//! server answers with `CConnectHeader` naming the client's recipient id, iteration and the two
//! ISAAC seeds; the client echoes the cookie in a connect-ack sent to the server's port plus one
//! (`ClientNet::SendConnectAck`).

use crate::header::{flags, PacketHeader};
use crate::packet::encode_packet;
use crate::string::{pack_string, unpack_string};
use crate::NetError;

/// How the client authenticates (`gmClient::InitAuth`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential {
    /// Auth type 1: the account name alone (`Turbine::Client::InitAuth`).
    Account,
    /// Auth type `0x40000002`: the password, packed as a narrow string, in the extra data.
    Password(String),
    /// Auth type `0x41000001`: a GUN (Gaming Zone) ticket's bytes in the extra data.
    GunTicket(Vec<u8>),
}

impl Credential {
    fn auth_type(&self) -> u32 {
        match self {
            Self::Account => 1,
            Self::Password(_) => 0x4000_0002,
            Self::GunTicket(_) => 0x4100_0001,
        }
    }

    fn extra_data(&self) -> Vec<u8> {
        match self {
            Self::Account => Vec::new(),
            Self::Password(password) => {
                let mut out = Vec::new();
                pack_string(&mut out, 0, password.as_bytes());
                out
            }
            Self::GunTicket(ticket) => ticket.clone(),
        }
    }
}

/// `CLogonHeader`: the packed data version, then the `NetAuthenticator` it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logon {
    /// The client data version the server checks, e.g. `7148`.
    pub version: String,
    pub account: String,
    pub credential: Credential,
    /// `m_dwConnectionSequenceNumber`.
    pub connection_sequence: u32,
}

impl Logon {
    /// `NetAuthenticator::StreamPack`: auth type, auth flags, connection sequence, packed
    /// account, then the crypto blob and the extra-data blob, each a u32 length and bytes. No
    /// CryptoAPI session key is exported, so flag bit 0 stays clear and the crypto blob is empty.
    fn authenticator(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(self.credential.auth_type().to_le_bytes());
        out.extend(0u32.to_le_bytes());
        out.extend(self.connection_sequence.to_le_bytes());
        pack_string(&mut out, 0, self.account.as_bytes());
        out.extend(0u32.to_le_bytes());
        let extra = self.credential.extra_data();
        out.extend((extra.len() as u32).to_le_bytes());
        out.extend(extra);
        out
    }

    /// `CLogonHeader::CreateFromData`: packed version, u32 authenticator size, authenticator.
    pub fn payload(&self) -> Vec<u8> {
        let mut out = Vec::new();
        pack_string(&mut out, 0, self.version.as_bytes());
        let authenticator = self.authenticator();
        out.extend((authenticator.len() as u32).to_le_bytes());
        out.extend(authenticator);
        out
    }

    /// The logon datagram.
    pub fn datagram(&self) -> Vec<u8> {
        encode_packet(
            PacketHeader::default(),
            &[(flags::LOGON, self.payload())],
            &[],
            None,
        )
    }

    /// Decodes a logon payload (used by tests and local servers).
    pub fn decode_payload(bytes: &[u8]) -> Result<(String, u32, String, Vec<u8>), NetError> {
        let mut pos = 0;
        let version = unpack_string(bytes, &mut pos, 64)?;
        let size = read_u32(bytes, &mut pos)? as usize;
        if size != bytes.len() - pos {
            return Err(NetError::Length);
        }
        let auth_type = read_u32(bytes, &mut pos)?;
        let _flags = read_u32(bytes, &mut pos)?;
        let _sequence = read_u32(bytes, &mut pos)?;
        let account = unpack_string(bytes, &mut pos, 256)?;
        let crypto = read_u32(bytes, &mut pos)? as usize;
        pos = pos.checked_add(crypto).ok_or(NetError::Length)?;
        let extra = read_u32(bytes, &mut pos)? as usize;
        let extra = bytes
            .get(pos..pos + extra)
            .ok_or(NetError::Truncated)?
            .to_vec();
        Ok((
            String::from_utf8_lossy(&version).into_owned(),
            auth_type,
            String::from_utf8_lossy(&account).into_owned(),
            extra,
        ))
    }
}

fn read_u32(bytes: &[u8], pos: &mut usize) -> Result<u32, NetError> {
    let b = bytes.get(*pos..*pos + 4).ok_or(NetError::Truncated)?;
    *pos += 4;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// `CConnectHeader`: a 32-byte structure of server time, cookie, net id and the two seeds, with
/// four bytes of tail padding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Connect {
    pub server_time: f64,
    pub cookie: u64,
    pub net_id: u32,
    /// Seeds the client's incoming generator (`ClientNet::InitCrypto`).
    pub outgoing_seed: u32,
    /// Seeds the client's outgoing generator.
    pub incoming_seed: u32,
}

impl Connect {
    pub fn decode(bytes: &[u8]) -> Result<Self, NetError> {
        if bytes.len() != 32 {
            return Err(NetError::Length);
        }
        let u32_at =
            |i: usize| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        let mut time = [0; 8];
        time.copy_from_slice(&bytes[0..8]);
        let mut cookie = [0; 8];
        cookie.copy_from_slice(&bytes[8..16]);
        Ok(Self {
            server_time: f64::from_le_bytes(time),
            cookie: u64::from_le_bytes(cookie),
            net_id: u32_at(16),
            outgoing_seed: u32_at(20),
            incoming_seed: u32_at(24),
        })
    }

    pub fn encode(&self) -> [u8; 32] {
        let mut out = [0; 32];
        out[0..8].copy_from_slice(&self.server_time.to_le_bytes());
        out[8..16].copy_from_slice(&self.cookie.to_le_bytes());
        out[16..20].copy_from_slice(&self.net_id.to_le_bytes());
        out[20..24].copy_from_slice(&self.outgoing_seed.to_le_bytes());
        out[24..28].copy_from_slice(&self.incoming_seed.to_le_bytes());
        out
    }

    /// The connect-ack datagram: the cookie under the recipient and iteration the connect named.
    pub fn ack_datagram(&self, recipient: u16, iteration: u16) -> Vec<u8> {
        encode_packet(
            PacketHeader {
                recipient,
                iteration,
                ..Default::default()
            },
            &[(flags::CONNECT_ACK, self.cookie.to_le_bytes().to_vec())],
            &[],
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Connect, Credential, Logon};
    use crate::header::flags;
    use crate::packet::decode_handshake;

    fn logon(credential: Credential) -> Logon {
        Logon {
            version: "7148".into(),
            account: "local".into(),
            credential,
            connection_sequence: 0,
        }
    }

    #[test]
    fn account_logon_matches_the_client_layout() {
        let datagram = logon(Credential::Account).datagram();
        let (header, payload) = decode_handshake(&datagram).unwrap();
        assert_eq!(header.flags, flags::LOGON);
        assert_eq!(
            (header.sequence, header.recipient, header.iteration),
            (0, 0, 0)
        );
        // version (8 bytes), authenticator size, type 1, flags 0, sequence 0, "local" (8 bytes),
        // empty crypto blob, empty extra blob.
        assert_eq!(payload.len(), 8 + 4 + 12 + 8 + 4 + 4);
        assert_eq!(&payload[12..16], &1u32.to_le_bytes());
        let (version, auth, account, extra) = Logon::decode_payload(payload).unwrap();
        assert_eq!(
            (version.as_str(), auth, account.as_str()),
            ("7148", 1, "local")
        );
        assert!(extra.is_empty());
    }

    #[test]
    fn password_and_ticket_credentials_use_their_auth_types() {
        let datagram = logon(Credential::Password("secret".into())).datagram();
        let (_, payload) = decode_handshake(&datagram).unwrap();
        let (_, auth, _, extra) = Logon::decode_payload(payload).unwrap();
        assert_eq!(auth, 0x4000_0002);
        let mut pos = 0;
        assert_eq!(
            crate::unpack_string(&extra, &mut pos, 64).unwrap(),
            b"secret"
        );
        let datagram = logon(Credential::GunTicket(vec![1, 2, 3])).datagram();
        let (_, payload) = decode_handshake(&datagram).unwrap();
        let (_, auth, _, extra) = Logon::decode_payload(payload).unwrap();
        assert_eq!((auth, extra), (0x4100_0001, vec![1, 2, 3]));
    }

    #[test]
    fn connect_round_trips_and_the_ack_echoes_the_cookie() {
        let connect = Connect {
            server_time: 1.5,
            cookie: 0x0102_0304_0506_0708,
            net_id: 9,
            outgoing_seed: 11,
            incoming_seed: 12,
        };
        assert_eq!(Connect::decode(&connect.encode()).unwrap(), connect);
        let ack = connect.ack_datagram(9, 4);
        let (header, payload) = decode_handshake(&ack).unwrap();
        assert_eq!(
            (header.flags, header.recipient, header.iteration),
            (flags::CONNECT_ACK, 9, 4)
        );
        assert_eq!(payload, &connect.cookie.to_le_bytes());
    }
}
