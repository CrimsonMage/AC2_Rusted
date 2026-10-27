//! The OS clipboard the text elements copy, cut and paste through (`ac2_ui::TextClipboard`),
//! through `arboard`: the clipboard `Device::SendStringToClipboard` and
//! `Device::GetStringFromClipboard` reach on Windows, and the platform's own elsewhere (on Wayland
//! through the `wayland-data-control` protocol, see Cargo.toml).

use std::sync::{Arc, Mutex, OnceLock};

use ac2_ui::{MemoryClipboard, TextClipboard};

/// The OS clipboard. On X11 and Wayland the owner serves a copied text only while its
/// `Clipboard` lives, so [`shared`] keeps the one instance in a static for the whole process.
struct OsClipboard(Mutex<arboard::Clipboard>);

impl TextClipboard for OsClipboard {
    fn text(&self) -> Option<String> {
        let Ok(mut clipboard) = self.0.lock() else {
            eprintln!("clipboard: unavailable (lock poisoned), paste ignored");
            return None;
        };
        match clipboard.get_text() {
            Ok(text) => Some(text),
            // Empty, or holding something other than text: nothing to paste, not a fault.
            Err(arboard::Error::ContentNotAvailable | arboard::Error::ConversionFailure) => None,
            Err(error) => {
                eprintln!("clipboard: paste failed: {error}");
                None
            }
        }
    }

    fn set_text(&self, text: &str) -> bool {
        let Ok(mut clipboard) = self.0.lock() else {
            eprintln!("clipboard: unavailable (lock poisoned), copy ignored");
            return false;
        };
        match clipboard.set_text(text.to_owned()) {
            Ok(()) => true,
            Err(error) => {
                eprintln!("clipboard: copy refused: {error}");
                false
            }
        }
    }
}

/// The clipboard every text tree shares: the OS one, or an in-memory one where the platform has
/// none (no display), which is logged once.
pub(crate) fn shared() -> Arc<dyn TextClipboard> {
    static CLIPBOARD: OnceLock<Arc<dyn TextClipboard>> = OnceLock::new();
    CLIPBOARD
        .get_or_init(|| match arboard::Clipboard::new() {
            Ok(clipboard) => Arc::new(OsClipboard(Mutex::new(clipboard))),
            Err(error) => {
                eprintln!("clipboard: no OS clipboard ({error}); copies stay inside the client");
                Arc::new(MemoryClipboard::default())
            }
        })
        .clone()
}
