//! The logon request: `CLogonHeader` carrying the client data version and a `NetAuthenticator`.
//!
//! `ClientNet::SendLoginRequest` sends it alone at sequence 0 with recipient, interval and
//! iteration 0. The Legions client (7160) packs the version string `"7160"`; the admin 7148
//! build packs `"7148"`.

use crate::flow::standalone;
use crate::optional::OptionalHeader;
use crate::session_key::SessionCrypto;
use crate::string::{pack_string, unpack_string};
use crate::NetError;

/// The data version the Legions DVD client sends (`ClientNet::SendLoginRequest`, Legions
/// 0x004BC780).
pub const LEGIONS_VERSION: &str = "7160";

/// How the client authenticates (`gmClient::InitAuth`).
///
/// `Debug` never shows the password: the ticket is a secret and must not reach a log.
#[derive(Clone, PartialEq, Eq)]
pub enum Credential {
    /// Auth type 1, no extra data: no password and no GUN ticket (`Turbine::Client::InitAuth`).
    Account,
    /// Auth type `0x40000002`: the password, packed as a narrow string, is the extra data.
    Password(String),
    /// Auth type `0x41000001`: a GUN (Gaming Zone) ticket's bytes are the extra data.
    GunTicket(Vec<u8>),
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Account => f.write_str("Account"),
            Self::Password(_) => f.write_str("Password(..)"),
            Self::GunTicket(_) => f.write_str("GunTicket(..)"),
        }
    }
}

impl Credential {
    /// `gmClient::InitAuth` (Legions 0x00404BB0) for a ticket string given on the command line
    /// (`-glsticketdirect`): an empty ticket (`PStringBase::m_len == 1`) with no GUN ticket
    /// handler is `Turbine::Client::InitAuth` (auth type 1, [`Credential::Account`]); any other
    /// ticket is packed with `PStringBase::Pack` and sent as auth type `0x40000002`
    /// ([`Credential::Password`]).
    pub fn from_ticket(ticket: &str) -> Self {
        if ticket.is_empty() {
            Self::Account
        } else {
            Self::Password(ticket.to_owned())
        }
    }

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
    /// The client data version the server checks: [`LEGIONS_VERSION`] for the DVD client.
    pub version: String,
    pub account: String,
    pub credential: Credential,
    /// `m_dwConnectionSequenceNumber`: `Timer::get_real_time()` (Unix seconds) when
    /// `ClientNet::Init` ran.
    pub connection_sequence: u32,
    /// The session key `ClientNet::Init` exports into the crypto blob. Retail never logs on
    /// without one; `None` sends auth flags 0 and an empty blob, for servers without the
    /// RSA private key.
    pub crypto: Option<SessionCrypto>,
}

impl Logon {
    /// `NetAuthenticator::StreamPack`: auth type, auth flags, connection sequence, packed
    /// account, then the crypto blob and the extra-data blob (`CGrowBuffer::StreamPack`: u32
    /// length and bytes). `NetAuthenticator::SetCryptoKey` sets flag bit 0 with the exported
    /// session key; bit 1 (`SetAdminAccountOverride`, admin builds only) is never set.
    fn authenticator(&self) -> Vec<u8> {
        let blob = self.crypto.as_ref().map_or(&[][..], |c| &c.blob[..]);
        let mut out = Vec::new();
        out.extend(self.credential.auth_type().to_le_bytes());
        out.extend(u32::from(!blob.is_empty()).to_le_bytes());
        out.extend(self.connection_sequence.to_le_bytes());
        pack_string(&mut out, 0, self.account.as_bytes());
        out.extend((blob.len() as u32).to_le_bytes());
        out.extend(blob);
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

    /// The logon datagram (`ClientNet::SendLoginRequest`).
    pub fn datagram(&self) -> Vec<u8> {
        standalone(OptionalHeader::Logon(self.payload()), 0, 0)
    }

    /// Decodes a logon payload: version, auth type, account and extra data (for tests and
    /// local servers).
    pub fn decode_payload(bytes: &[u8]) -> Result<(String, u32, String, Vec<u8>), NetError> {
        let mut pos = 0;
        let version = unpack_string(bytes, &mut pos, 64)?;
        let size = read_u32(bytes, &mut pos)? as usize;
        if size != bytes.len() - pos {
            return Err(NetError::Length);
        }
        let auth_type = read_u32(bytes, &mut pos)?;
        let auth_flags = read_u32(bytes, &mut pos)?;
        let _sequence = read_u32(bytes, &mut pos)?;
        let account = unpack_string(bytes, &mut pos, 256)?;
        if auth_flags & 2 != 0 {
            unpack_string(bytes, &mut pos, 256)?;
        }
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

#[cfg(test)]
mod tests {
    use super::{Credential, Logon, LEGIONS_VERSION};
    use crate::checksum::checksum32;
    use crate::header::{flags, PacketHeader};
    use crate::session_key::{SessionCrypto, SessionKey};

    fn logon(credential: Credential) -> Logon {
        Logon {
            version: LEGIONS_VERSION.into(),
            account: "local".into(),
            credential,
            connection_sequence: 0x6543_2100,
            crypto: None,
        }
    }

    #[test]
    fn account_logon_matches_send_login_request_byte_for_byte() {
        let datagram = logon(Credential::Account).datagram();
        let mut expected = Vec::new();
        // "7160" nibble-swapped then XORed with 0xE5849238: 73 13 63 03 ^ 38 92 84 E5.
        expected.extend([4, 0, 0x4B, 0x81, 0xE7, 0xE6, 0, 0]);
        expected.extend(28u32.to_le_bytes());
        expected.extend(1u32.to_le_bytes());
        expected.extend(0u32.to_le_bytes());
        expected.extend(0x6543_2100u32.to_le_bytes());
        // "local": 6C 6F 63 61 | 6C -> C6 F6 36 16 ^ 38 92 84 E5, then C6 and one pad byte
        // (alignment counts from the 4-aligned `m_PackedData`).
        expected.extend([5, 0, 0xFE, 0x64, 0xB2, 0xF3, 0xC6, 0]);
        expected.extend(0u32.to_le_bytes());
        expected.extend(0u32.to_le_bytes());
        let header = PacketHeader::decode(&datagram).unwrap();
        assert_eq!(
            (
                header.sequence,
                header.flags,
                header.recipient,
                header.interval,
                header.iteration
            ),
            (0, flags::LOGON, 0, 0, 0)
        );
        assert_eq!(header.size as usize, expected.len());
        assert_eq!(&datagram[20..], &expected[..]);
        let body = checksum32(&expected);
        assert_eq!(header.checksum, body.wrapping_add(header.header_checksum()));
        let (version, auth, account, extra) = Logon::decode_payload(&expected).unwrap();
        assert_eq!(
            (version.as_str(), auth, account.as_str()),
            ("7160", 1, "local")
        );
        assert!(extra.is_empty());
    }

    #[test]
    fn the_session_key_blob_sets_auth_flag_one() {
        let mut logon = logon(Credential::Account);
        logon.crypto = Some(SessionCrypto {
            key: SessionKey::from_bytes([1, 2, 3, 4, 5]),
            blob: (0..76).collect(),
        });
        let payload = logon.payload();
        let mut expected = vec![4, 0, 0x4B, 0x81, 0xE7, 0xE6, 0, 0];
        expected.extend((28u32 + 76).to_le_bytes());
        expected.extend(1u32.to_le_bytes());
        expected.extend(1u32.to_le_bytes());
        expected.extend(0x6543_2100u32.to_le_bytes());
        expected.extend([5, 0, 0xFE, 0x64, 0xB2, 0xF3, 0xC6, 0]);
        expected.extend(76u32.to_le_bytes());
        expected.extend(0..76u8);
        expected.extend(0u32.to_le_bytes());
        assert_eq!(payload, expected);
        let (_, auth, account, extra) = Logon::decode_payload(&payload).unwrap();
        assert_eq!((auth, account.as_str(), extra.len()), (1, "local", 0));
    }

    #[test]
    fn from_ticket_follows_gmclient_init_auth() {
        assert_eq!(Credential::from_ticket(""), Credential::Account);
        assert_eq!(
            Credential::from_ticket("secret"),
            Credential::Password("secret".into())
        );
    }

    #[test]
    fn debug_never_shows_the_ticket() {
        let shown = format!("{:?}", Credential::from_ticket("hunter2"));
        assert!(!shown.contains("hunter2"), "{shown}");
        let shown = format!("{:?}", logon(Credential::from_ticket("hunter2")));
        assert!(!shown.contains("hunter2"), "{shown}");
    }

    #[test]
    fn ticket_logon_matches_send_login_request_byte_for_byte() {
        let mut logon = logon(Credential::from_ticket("secret"));
        logon.account = "myname".into();
        let mut expected = vec![4, 0, 0x4B, 0x81, 0xE7, 0xE6, 0, 0];
        // Authenticator: type, flags, sequence (12), account (2 + 6), crypto length (4),
        // extra length (4) and the packed ticket (2 + 6).
        expected.extend(36u32.to_le_bytes());
        expected.extend(0x4000_0002u32.to_le_bytes());
        expected.extend(0u32.to_le_bytes());
        expected.extend(0x6543_2100u32.to_le_bytes());
        // "myname": nibble-swapped, then the first word XORed with 0xE5849238; the last two
        // bytes are not a whole word and are only nibble-swapped.
        expected.extend([6, 0, 0xEE, 0x05, 0x62, 0xF3, 0xD6, 0x56]);
        expected.extend(0u32.to_le_bytes());
        // The ticket is packed into its own buffer: "secret" -> 0F C4 B2 C2 | 56 47.
        expected.extend(8u32.to_le_bytes());
        expected.extend([6, 0, 0x0F, 0xC4, 0xB2, 0xC2, 0x56, 0x47]);
        assert_eq!(logon.payload(), expected);
        let (_, auth, account, extra) = Logon::decode_payload(&expected).unwrap();
        assert_eq!((auth, account.as_str()), (0x4000_0002, "myname"));
        let mut pos = 0;
        assert_eq!(
            crate::unpack_string(&extra, &mut pos, 64).unwrap(),
            b"secret"
        );
    }

    #[test]
    fn password_and_ticket_credentials_use_their_auth_types() {
        let payload = logon(Credential::Password("secret".into())).payload();
        let (_, auth, _, extra) = Logon::decode_payload(&payload).unwrap();
        assert_eq!(auth, 0x4000_0002);
        let mut pos = 0;
        assert_eq!(
            crate::unpack_string(&extra, &mut pos, 64).unwrap(),
            b"secret"
        );
        let payload = logon(Credential::GunTicket(vec![1, 2, 3])).payload();
        let (_, auth, _, extra) = Logon::decode_payload(&payload).unwrap();
        assert_eq!((auth, extra), (0x4100_0001, vec![1, 2, 3]));
    }
}
