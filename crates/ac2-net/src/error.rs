use std::fmt;

/// Why a datagram, handshake or session step failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetError {
    /// The bytes ended before a field.
    Truncated,
    /// A length or count disagrees with the bytes present.
    Length,
    /// The header checksum does not match.
    Checksum,
    /// A flag this transport does not decode, or a combination it forbids.
    UnsupportedFlags(u32),
    /// A sequence outside the receive window or an unknown session identity.
    Sequence,
    /// A fragment that conflicts with earlier pieces of its blob.
    Fragment,
    /// A bound on pending, assembling or ordered data was reached.
    Capacity,
    /// A reliable packet was retransmitted too often without acknowledgement.
    RetryLimit,
    /// The server reported an error header (`NetError` string and table ids).
    Server { string_id: u32, table_id: u32 },
    /// The server did not answer the logon in time.
    Timeout,
    /// The session key could not be made or exported (`ID_NetError_CantCrypto`).
    Crypto(&'static str),
    /// A socket operation failed.
    Io(String),
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Server {
                string_id,
                table_id,
            } => write!(f, "server error {string_id:#010x} (table {table_id:#010x})"),
            Self::Io(message) => write!(f, "socket error: {message}"),
            other => write!(f, "{other:?}"),
        }
    }
}

impl std::error::Error for NetError {}

impl From<std::io::Error> for NetError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
