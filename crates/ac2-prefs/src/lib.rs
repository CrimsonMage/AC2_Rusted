//! The client's preferences (`portal/shared/src/prefs`): the INI file (`Preferences.ini`, read
//! and written with the Win32 INI calls), `PreferenceBank` (the preferences the subsystems
//! register, their sections and their owners), the four preference types (bool, float, enum,
//! string), and the `Preferences` cache for the values read by key.
//!
//! No file is read or written by the types: [`ini::IniFile`] and [`bank::PreferenceBank`] work on
//! bytes, and [`paths`] does the I/O at startup (before the frame loop) and on a save (off the
//! frame thread). Section, key and label names come from the game data through [`names::NameTable`].

pub mod bank;
pub mod cnum;
pub mod gls_ticket;
pub mod ini;
pub mod names;
pub mod paths;
pub mod pref;
pub mod store;

pub use bank::PreferenceBank;
pub use names::{NameTable, NoNames};
pub use pref::{Choice, Kind, Pref, Value};
pub use store::Preferences;

#[cfg(test)]
mod tests;
