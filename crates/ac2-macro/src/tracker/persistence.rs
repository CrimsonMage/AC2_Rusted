//! Where the pattern list lives and which source wins: the Tracker's own file in the client's
//! config directory; when it is missing, the first existing older list (Vitaeum's, then Virindi
//! Sense's under `Decal Plugins`) is loaded and copied over unchanged.

use std::sync::Arc;

/// The Tracker's directory under the client's config directory, and its file.
pub const CANONICAL_DIRECTORY: &str = "tracker";
pub const PATTERN_FILE: &str = "patterns.txt";
/// Older lists imported when the Tracker has none, as path pieces under the config directory's
/// parent: Vitaeum's Sense list, then Virindi Sense's.
pub const LEGACY_SOURCES: [&[&str]; 2] = [&["vitaeum-sense"], &["Decal Plugins", "Virindi Sense"]];

/// The result of reading one pattern file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternRead {
    Missing,
    Bytes(Arc<[u8]>),
    Failed(Arc<str>),
}

impl PatternRead {
    pub fn from_result(result: Result<Option<Vec<u8>>, impl std::fmt::Display>) -> Self {
        match result {
            Ok(Some(bytes)) => Self::Bytes(bytes.into()),
            Ok(None) => Self::Missing,
            Err(error) => Self::Failed(Arc::from(error.to_string())),
        }
    }
}

/// What the first load uses, what it copies to the Tracker's own file, and what it warns about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InitialPlan {
    pub bytes: Option<Arc<[u8]>>,
    pub migrate: Option<Arc<[u8]>>,
    pub warning: Option<Arc<str>>,
}

pub(super) fn initial_plan(canonical: PatternRead, legacy: PatternRead) -> InitialPlan {
    let none = |warning| InitialPlan {
        bytes: None,
        migrate: None,
        warning,
    };
    match canonical {
        PatternRead::Bytes(bytes) => InitialPlan {
            bytes: Some(bytes),
            migrate: None,
            warning: None,
        },
        PatternRead::Failed(error) => none(Some(Arc::from(format!(
            "[Tracker] Could not load the pattern list: {error}"
        )))),
        PatternRead::Missing => match legacy {
            PatternRead::Bytes(bytes) => InitialPlan {
                bytes: Some(Arc::clone(&bytes)),
                migrate: Some(bytes),
                warning: None,
            },
            PatternRead::Missing => none(None),
            PatternRead::Failed(error) => none(Some(Arc::from(format!(
                "[Tracker] Could not import the older pattern list: {error}"
            )))),
        },
    }
}
