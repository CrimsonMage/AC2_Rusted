//! Automatic tools (a modern addition): once switched on, each acts by itself, and everything it
//! does is one of the player's own gestures, carried out by the client through the path the
//! gesture takes; none sends a message of its own.
//!
//! - `butcher`: the butchering knife used on each new butcherable corpse in reach (the knife's
//!   use, then the targeting click on the body).
//! - `rebuff`: chosen effects kept up by pressing the shortcut bar slot of the skill that casts
//!   each.
//! - `settings`: both tools' choices, saved per character, and the settings window's changes.

pub mod butcher;
pub mod rebuff;
pub mod settings;

pub use butcher::{
    AutoButcher, ButcherAction, ButcherEvent, ButcherSettings, ButcherView, Corpse, KilledBy, Knife,
};
pub use rebuff::{
    ActiveEffect, AutoRebuff, RebuffEntry, RebuffSettings, RebuffView, SkillState, UseShortcut,
};
pub use settings::{Change, Flag, Settings, Value};

#[cfg(test)]
mod tests;
