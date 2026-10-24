//! The binary unpack cursor for packed records.

mod cursor;
mod error;

pub use cursor::ArchiveCursor;
pub use error::{ArchiveError, ArchiveResult};
