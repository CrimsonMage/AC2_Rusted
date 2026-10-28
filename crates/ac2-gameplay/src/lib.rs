//! Gameplay (`ac2/shared/src/quest`, `character`, `craft`, `examination`, `conversation`): the
//! rules the client applies to the quests, characters, crafting and conversations it receives.
//! Plain data in, plain data out; the UI (`ac2-ui`) and the script glue (`ac2-script`) turn the
//! results into strings and windows.

pub mod character;
pub mod conversation;
pub mod quest;
