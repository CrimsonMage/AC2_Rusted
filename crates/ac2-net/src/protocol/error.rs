use std::fmt;

use crate::NetError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    Truncated,
    TrailingBytes,
    /// A field value outside what the client accepts.
    Invalid(&'static str),
    /// A structure part this decoder does not unpack yet.
    Unsupported(&'static str),
    String(NetError),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ProtocolError {}

impl From<NetError> for ProtocolError {
    fn from(error: NetError) -> Self {
        match error {
            NetError::Truncated => Self::Truncated,
            other => Self::String(other),
        }
    }
}
