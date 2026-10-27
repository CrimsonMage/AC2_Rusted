//! The clipboard the text elements copy, cut and paste through (`Device::SendStringToClipboard`
//! and `Device::GetStringFromClipboard`, which the admin text code's `Copy`, `Cut` and `Paste`
//! call). The client supplies the OS clipboard; a tree without one keeps its own in memory, so
//! the text code runs with no display (tests, the offline viewer).

use std::sync::Mutex;

/// The text clipboard. Implementations use interior mutability: a tree and its clones share one.
pub trait TextClipboard: Send + Sync {
    /// The clipboard's text, `None` when it holds none or cannot be read.
    fn text(&self) -> Option<String>;
    /// Replaces the clipboard's text; `false` when it refused.
    fn set_text(&self, text: &str) -> bool;
}

impl std::fmt::Debug for dyn TextClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TextClipboard")
    }
}

/// A clipboard held in memory: the default of a tree, and the test double.
#[derive(Debug, Default)]
pub struct MemoryClipboard {
    text: Mutex<Option<String>>,
}

impl MemoryClipboard {
    /// A clipboard that already holds `text`.
    pub fn with_text(text: &str) -> Self {
        Self {
            text: Mutex::new(Some(text.to_owned())),
        }
    }
}

impl TextClipboard for MemoryClipboard {
    fn text(&self) -> Option<String> {
        self.text.lock().ok()?.clone()
    }

    fn set_text(&self, text: &str) -> bool {
        match self.text.lock() {
            Ok(mut slot) => {
                *slot = Some(text.to_owned());
                true
            }
            Err(_) => false,
        }
    }
}
