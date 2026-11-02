//! The client's side of GUN (`ac2/client/src/gun`, `gmClient`'s GUN members and
//! `gmCWInterface::RequestGUN_*`), over `ac2-gun`'s `TurbineGun`. Admin 7148 and Legions 7160
//! run the same code in every function here except where an item says otherwise (the Legions
//! build has less: no `GunTicketGen` block, no `FATAL ERROR` reports); every item cites both
//! addresses (`decompiled-pdb/gundll/spec/0c_client.md` and its adjudication).
//!
//! * `options`: the command-line options and `gmClient`'s GUN fields.
//! * `shared`: the retail globals the classes reach (`S_pTurbineGun`,
//!   `S_fUseTurbineChatInsteadOfGUN`, `G_pGunTicketHandler`'s home, the player interface).
//! * `handlers`: the `CCli*` listener and result classes.
//! * `chat_extra`: `GunChatExtraInfo` and `gmCWInterface::HearTell` / `GUNEvent_HearFromRoom`.
//! * `interface`: `gmCWInterface::RequestGUN_*`.
//! * `client`: `gmClient::Init` / `StartupGUN` / `AuthenticateWithGUN` / `InitAuth` and the notices.
//! * `output`: what the classes raise (the `CM_UI` notices and the script IFuns) for the
//!   program to carry out.

pub mod chat_extra;
pub mod client;
pub mod handlers;
pub mod interface;
pub mod options;
pub mod output;
pub mod shared;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_client;

pub use client::{ConnectPoll, GunClient, GunError};
pub use options::{GunOptions, TurbineClientOptions};
pub use output::{GunNotice, GunOutput};
pub use shared::{shared, GunShared};
