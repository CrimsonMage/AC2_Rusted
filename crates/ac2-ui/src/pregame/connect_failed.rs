//! `gmDataPatchUI::OnConnectFailed` (admin 0x00411630): the text a failed connection queues the
//! disconnected screen with. `ID_DataPatch_ConnectFailed` carries the reason as its `error`
//! variable, picked by the connection's `NetError`:
//!
//! - `ID_NetError_BadServerAddress`: `ID_NetErr_ResolveHostname` with the host.
//! - `ID_ConnectionError_Generic`: `ID_NetErr_Error` with the host and the port.
//! - `ID_NetError_CantCrypto`: its own text with the last crypto error.
//! - any other: its own text (`NetError::operator StringInfo`).
//!
//! A reason whose string is not on disk gives no text (the screen stays), except a migrated logon,
//! which says the server closed the connection.
//!
//! The session's errors that carry no retail `NetError` take the one retail's connection raised
//! for the same cause: no answer to the logon is `ID_ConnectionError_ClientTimedOutServer`, a
//! closed connection `ID_ConnectionError_ServerClosedConnection`, a key that could not be made
//! `ID_NetError_CantCrypto`, and a socket or protocol failure `ID_ConnectionError_Generic`.

use ac2_db::{RenderInfo, RenderVar};
use ac2_net::NetError;

use crate::pregame::strings::{text_var, UiStrings};
use crate::token::tokenize;

/// The retail `NetError` (its string id) a session error stands for.
fn net_error_id(error: &NetError) -> Option<u32> {
    Some(match error {
        NetError::Server { string_id, .. } => *string_id,
        NetError::Timeout => tokenize("ID_ConnectionError_ClientTimedOutServer"),
        NetError::ServerDied => tokenize("ID_ConnectionError_ServerClosedConnection"),
        NetError::Crypto(_) => tokenize("ID_NetError_CantCrypto"),
        _ => tokenize("ID_ConnectionError_Generic"),
    })
}

/// `OnConnectFailed`'s text for `error` on `host`:`port`, or `None` when its string is missing.
pub(crate) fn connect_failed_text(
    strings: &UiStrings,
    error: &NetError,
    host: &str,
    port: u16,
) -> Option<String> {
    let id = net_error_id(error)?;
    let own = |string_id: u32| RenderInfo {
        string_id,
        table_id: strings.net_table(),
        ..RenderInfo::default()
    };
    let ui = |name: &str, variables: Vec<(u32, RenderVar)>| RenderInfo {
        string_id: tokenize(name),
        table_id: strings.table(),
        literal: None,
        variables,
    };
    let mut info = if id == tokenize("ID_NetError_BadServerAddress") {
        ui("ID_NetErr_ResolveHostname", vec![text_var("host", host)])
    } else if id == tokenize("ID_ConnectionError_Generic") {
        ui(
            "ID_NetErr_Error",
            vec![
                text_var("host", host),
                (tokenize("port"), RenderVar::Int(i32::from(port))),
            ],
        )
    } else if id == tokenize("ID_NetError_CantCrypto") {
        // `Turbine::Client::GetLastCryptoError`: the session keeps no code; 0.
        RenderInfo {
            variables: vec![(tokenize("error"), RenderVar::Int(0))],
            ..own(id)
        }
    } else {
        own(id)
    };
    if !strings.is_on_disk(&info) {
        if id != tokenize("ID_ConnectionError_LogonServerMigrated") {
            return None;
        }
        info = own(tokenize("ID_ConnectionError_ServerClosedConnection"));
        if !strings.is_on_disk(&info) {
            return None;
        }
    }
    let failed = ui(
        "ID_DataPatch_ConnectFailed",
        vec![(tokenize("error"), RenderVar::Info(Box::new(info)))],
    );
    Some(strings.render_info(&failed))
}
