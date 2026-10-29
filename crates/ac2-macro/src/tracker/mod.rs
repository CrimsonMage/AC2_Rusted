//! The Tracker: a list of name patterns (case-insensitive regular expressions, Virindi Sense's
//! `patterns.txt` format) matched against the objects the client sees. A match is announced in
//! chat with its coordinates, kept in a list with its live distance, and can be picked to point
//! the floating arrow at it. It only reads what the client already knows; nothing is sent.

mod codec;
mod persistence;
mod presentation;
mod runtime;

pub use persistence::{PatternRead, CANONICAL_DIRECTORY, LEGACY_SOURCES, PATTERN_FILE};
pub use presentation::{
    coordinates_text, PositionQuery, Presentation, PresentedMatch, PresentedPattern,
    TrackerPosition,
};
pub use runtime::{
    Effect, MatchView, ObservedObject, PatternView, Tracker, TrackerUpdate, TrackerView,
};

#[cfg(test)]
mod tests;
