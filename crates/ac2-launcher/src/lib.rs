//! The AC2 launcher (a modern addition, as retail's `ac2launcher.exe` stood beside the client):
//! the player's favourite servers and a public server list ([`servers`]), the sign-in each
//! server wants ([`account`] for a Pyrealium account service), and the start of the `ac2` program
//! with its ticket handed over in a file ([`launch`]). It sends nothing itself but the account
//! service's sign-in; the game connection is the client's.

pub mod account;
pub mod launch;
pub mod servers;
