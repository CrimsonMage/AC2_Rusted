//! Fonts and strings for text elements, read through the resource store while a tree is built
//! on a worker thread (`StringDownload`'s asynchronous fetch, done up front). Once frozen it
//! answers only from what it already read, so a tree used on the frame thread never reads files.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ac2_assets::Ac2Font;
use ac2_core::DataId;
use ac2_db::ResourceStore;

use crate::text::UiTextData;

/// `FONT_GROUP_ENUM` and `Default_FontID` (`UIElement_Text::UIElement_Text`).
pub const FONT_GROUP: u32 = 0x4000_000B;
pub const DEFAULT_FONT: u32 = 0x4000_0001;

type FontCache = HashMap<u32, Option<Arc<Ac2Font>>>;
type StringCache = HashMap<(u32, u32), Option<Vec<u16>>>;

/// [`UiTextData`] over a resource store.
pub struct StoreTextData {
    store: Arc<ResourceStore>,
    fonts: Mutex<FontCache>,
    strings: Mutex<StringCache>,
    /// The default font's DID, once looked up.
    default_did: Mutex<Option<Option<u32>>>,
    frozen: AtomicBool,
}

impl StoreTextData {
    pub fn new(store: Arc<ResourceStore>) -> Self {
        Self {
            store,
            fonts: Mutex::new(HashMap::new()),
            strings: Mutex::new(HashMap::new()),
            default_did: Mutex::new(None),
            frozen: AtomicBool::new(false),
        }
    }

    /// From now on answers only from the cache.
    pub fn freeze(&self) {
        self.frozen.store(true, Ordering::Release);
    }

    fn frozen(&self) -> bool {
        self.frozen.load(Ordering::Acquire)
    }
}

impl UiTextData for StoreTextData {
    fn font(&self, did: u32) -> Option<Arc<Ac2Font>> {
        let mut fonts = self.fonts.lock().ok()?;
        if let Some(font) = fonts.get(&did) {
            return font.clone();
        }
        if self.frozen() {
            return None;
        }
        let font = ac2_assets::load_ac2_font(&self.store, DataId::new(did))
            .ok()
            .flatten()
            .map(|font| Arc::new((*font).clone()));
        fonts.insert(did, font.clone());
        font
    }

    fn default_font(&self) -> Option<Arc<Ac2Font>> {
        let did = {
            let mut cached = self.default_did.lock().ok()?;
            match *cached {
                Some(did) => did,
                None if self.frozen() => None,
                None => {
                    let did = ac2_assets::ac2_enum_did(&self.store, FONT_GROUP, DEFAULT_FONT)
                        .ok()
                        .flatten()
                        .map(|did| did.raw());
                    *cached = Some(did);
                    did
                }
            }
        }?;
        self.font(did)
    }

    fn string(&self, table: u32, id: u32) -> Option<Vec<u16>> {
        let mut strings = self.strings.lock().ok()?;
        if let Some(string) = strings.get(&(table, id)) {
            return string.clone();
        }
        if self.frozen() {
            return None;
        }
        let string = ac2_db::load_string_table(&self.store, DataId::new(table))
            .ok()
            .flatten()
            .and_then(|t| t.get(id).map(|s| s.text().encode_utf16().collect()));
        strings.insert((table, id), string.clone());
        string
    }
}
