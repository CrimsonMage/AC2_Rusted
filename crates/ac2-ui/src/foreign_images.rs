//! UI images from outside the AC2 DATs (the `ac1-monsters` branch's AC1 item icons): a
//! process-wide provider asked first for every image the UI loads, for the image DIDs it owns.

use std::sync::{Arc, OnceLock};

use crate::UiPixels;

/// A source of UI images by image DID; `None` for a DID it does not own.
pub trait ForeignUiImages: Send + Sync {
    fn image(&self, file: u32) -> Option<UiPixels>;
}

static FOREIGN: OnceLock<Arc<dyn ForeignUiImages>> = OnceLock::new();

/// Installs `images` for the process (once; `false` when a source was installed already).
pub fn set_foreign_ui_images(images: Arc<dyn ForeignUiImages>) -> bool {
    FOREIGN.set(images).is_ok()
}

/// The installed source's image `file`, if it owns it.
pub(crate) fn foreign_image(file: u32) -> Option<UiPixels> {
    FOREIGN.get()?.image(file)
}
