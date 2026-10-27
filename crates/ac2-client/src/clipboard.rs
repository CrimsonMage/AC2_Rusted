//! The OS clipboard the text elements copy, cut and paste through (`ac2_ui::TextClipboard`),
//! through `arboard`: the clipboard `Device::SendStringToClipboard` and
//! `Device::GetStringFromClipboard` reach on Windows, and the platform's own elsewhere.

use std::sync::{Arc, Mutex, OnceLock};

use ac2_ui::{MemoryClipboard, TextClipboard};

/// The OS clipboard. Its owner serves a copied text only while it lives, so the one instance
/// lasts as long as the client ([`shared`]).
struct OsClipboard(Mutex<arboard::Clipboard>);

impl TextClipboard for OsClipboard {
    fn text(&self) -> Option<String> {
        self.0.lock().ok()?.get_text().ok()
    }

    fn set_text(&self, text: &str) -> bool {
        self.0
            .lock()
            .is_ok_and(|mut clipboard| clipboard.set_text(text.to_owned()).is_ok())
    }
}

/// The clipboard every text tree shares: the OS one, or an in-memory one where the platform has
/// none (no display).
pub(crate) fn shared() -> Arc<dyn TextClipboard> {
    static CLIPBOARD: OnceLock<Arc<dyn TextClipboard>> = OnceLock::new();
    CLIPBOARD
        .get_or_init(|| match arboard::Clipboard::new() {
            Ok(clipboard) => Arc::new(OsClipboard(Mutex::new(clipboard))),
            Err(_) => Arc::new(MemoryClipboard::default()),
        })
        .clone()
}
