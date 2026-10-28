//! Names for the `NetError` ids a server sends in `Net_LoginConnectionError` and the logon
//! `NetError` header. The ids are `Tokenize(name)` hashes (`ac2_net::tokenize`); the names are
//! the client's own `ID_ConnectionError_*` and `ID_NetError_*` strings.

use ac2_net::tokenize;

const NAMES: &[&str] = &[
    "ID_ConnectionError_AccountAuthenticationFailed",
    "ID_ConnectionError_BadCryptoKey",
    "ID_ConnectionError_ClientLogOnFailed",
    "ID_ConnectionError_ClientTimedOutServer",
    "ID_ConnectionError_Generic",
    "ID_ConnectionError_InsufficientPriveledge",
    "ID_ConnectionError_LogonServerMigrated",
    "ID_ConnectionError_NetVersionMismatch",
    "ID_ConnectionError_NoLogonServer",
    "ID_ConnectionError_PlayerAlreadyLoggedOn",
    "ID_ConnectionError_RunningSpeedhack",
    "ID_ConnectionError_SecondLogon",
    "ID_ConnectionError_ServerClosedConnection",
    "ID_ConnectionError_ServerFull",
    "ID_ConnectionError_ServerTimedOutClient",
    "ID_NetErr_ConnectionLost",
    "ID_NetErr_Error",
    "ID_NetErr_ResolveHostname",
    "ID_NetError_AbortedHandshake",
    "ID_NetError_BadServerAddress",
    "ID_NetError_CantBind",
    "ID_NetError_CantCrypto",
    "ID_NetError_CantSocket",
    "ID_NetError_None",
];

/// The client string name a `NetError` string id tokenizes from, if it is one the client knows.
pub fn name(string_id: u32) -> Option<&'static str> {
    NAMES
        .iter()
        .copied()
        .find(|name| tokenize(name.as_bytes()) == string_id)
}

/// A reason as printed: the name when known, always the raw ids.
pub fn describe(string_id: u32, table_id: u32) -> String {
    match name(string_id) {
        Some(name) => format!("{name} (string {string_id:#010x}, table {table_id:#010x})"),
        None => format!("unknown reason (string {string_id:#010x}, table {table_id:#010x})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_names_round_trip_through_the_token() {
        let id = tokenize(b"ID_ConnectionError_NetVersionMismatch");
        assert_eq!(name(id), Some("ID_ConnectionError_NetVersionMismatch"));
        assert!(describe(id, 0x4000_0005).contains("NetVersionMismatch"));
        assert!(describe(1, 2).starts_with("unknown reason"));
    }
}
