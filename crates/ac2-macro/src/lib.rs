//! Player automation tools (`ac2-macro`): modern additions with no retail counterpart, kept as
//! pure policy. The client hands them observations (objects, selection, positions) and carries out
//! the effects they return (saving a file, a chat line, a selection); they read no files, draw
//! nothing and send nothing to the server.
//!
//! - `tracker`: named-object detection (Virindi Sense's pattern list and behaviour).
//! - `automation`: the automatic tools (auto-butcher, auto-rebuff), which act only through the
//!   player's own gestures, carried out by the client.

pub mod automation;
pub mod tracker;
