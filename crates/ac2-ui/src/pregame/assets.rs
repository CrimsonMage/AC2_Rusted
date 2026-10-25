//! What the pregame frameworks read from the DATs, read up front on a worker thread
//! (`UIMainFramework::CreateAndAddRootElement`'s layouts, `StringDownload`'s fonts and strings,
//! `Graphic`'s images) so the frame thread, which makes and switches the frameworks, never reads
//! a file: every pregame layout, the fonts and `entry` strings any of their states name, the
//! images of every state's media, and the UI string table.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::{ResourceResult, ResourceStore};

use crate::compose::UiPixels;
use crate::dialog::LAYOUT_DIALOGS;
use crate::element::UiTree;
use crate::hud::load_layout_by_enum;
use crate::layout::{AttributeValue, ElementDesc, LayoutDesc};
use crate::layout_media::MediaDesc;
use crate::pregame::strings::UiStrings;
use crate::pregame::{
    LAYOUT_AVATAR_MODIFICATION, LAYOUT_AVATAR_SELECTION, LAYOUT_CHARACTER_SELECTION,
    LAYOUT_CREDITS, LAYOUT_DATA_PATCH, LAYOUT_DISCONNECTED, LAYOUT_GENERATION_CONFIRMATION,
    LAYOUT_INTRO, LAYOUT_KEYBOARD,
};
use crate::text::UiTextData;
use crate::text_data::StoreTextData;
use crate::token::{TEXT_ENTRY, TEXT_FONT};

/// The layouts the pregame frameworks and their dialogs use.
pub const PREGAME_LAYOUTS: [u32; 10] = [
    LAYOUT_DISCONNECTED,
    LAYOUT_CHARACTER_SELECTION,
    LAYOUT_INTRO,
    LAYOUT_DATA_PATCH,
    LAYOUT_CREDITS,
    LAYOUT_KEYBOARD,
    LAYOUT_DIALOGS,
    LAYOUT_AVATAR_SELECTION,
    LAYOUT_AVATAR_MODIFICATION,
    LAYOUT_GENERATION_CONFIRMATION,
];

/// The pregame UI's data, loaded.
#[derive(Clone)]
pub struct PregameAssets {
    layouts: HashMap<u32, Arc<LayoutDesc>>,
    text: Option<Arc<dyn UiTextData>>,
    pub strings: UiStrings,
    pub images: HashMap<u32, UiPixels>,
}

impl std::fmt::Debug for PregameAssets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PregameAssets")
            .field("layouts", &self.layouts.keys().collect::<Vec<_>>())
            .field("images", &self.images.len())
            .finish_non_exhaustive()
    }
}

impl PregameAssets {
    /// The layout a layout enum names, if the DATs hold it.
    pub fn layout(&self, layout_enum: u32) -> Option<&Arc<LayoutDesc>> {
        self.layouts.get(&layout_enum)
    }

    /// A tree that draws its text from these assets.
    pub fn tree(&self, width: i32, height: i32) -> UiTree {
        let mut tree = UiTree::new(width, height);
        if let Some(text) = &self.text {
            tree.set_text_data(Arc::clone(text));
        }
        tree
    }

    /// Assets made of `layouts` (keyed by enum) without fonts, strings or images (tests).
    pub fn from_layouts(layouts: HashMap<u32, Arc<LayoutDesc>>) -> Self {
        Self {
            layouts,
            text: None,
            strings: UiStrings::default(),
            images: HashMap::new(),
        }
    }
}

/// Every font, string and image the description's states name, collected.
fn walk(
    desc: &ElementDesc,
    fonts: &mut BTreeSet<u32>,
    strings: &mut BTreeSet<u32>,
    images: &mut BTreeSet<u32>,
) {
    for state in &desc.states {
        for attribute in &state.attributes {
            match (attribute.id, &attribute.value) {
                (TEXT_FONT, AttributeValue::DataId(did)) => {
                    fonts.insert(*did);
                }
                (TEXT_ENTRY, AttributeValue::StringId(id)) => {
                    strings.insert(*id);
                }
                _ => {}
            }
        }
        for media in &state.media {
            match media {
                MediaDesc::Image { file, .. } | MediaDesc::Alpha { file, .. } => {
                    images.insert(*file);
                }
                MediaDesc::Anim { frames, .. } => images.extend(frames.iter().copied()),
                _ => {}
            }
        }
        for child in &state.children {
            walk(child, fonts, strings, images);
        }
    }
}

/// Reads the pregame UI's data (call it on a worker thread): the layouts, then every font,
/// `entry` string and image they name, then the UI string table. The text data is frozen.
pub fn load_pregame(store: &Arc<ResourceStore>) -> ResourceResult<PregameAssets> {
    let mut layouts = HashMap::new();
    for layout_enum in PREGAME_LAYOUTS {
        if let Some(layout) = load_layout_by_enum(store, layout_enum)? {
            layouts.insert(layout_enum, layout);
        }
    }
    let text = Arc::new(StoreTextData::new(Arc::clone(store)));
    text.default_font();
    let mut images = HashMap::new();
    for layout in layouts.values() {
        let (mut fonts, mut strings, mut files) = Default::default();
        for element in &layout.elements {
            walk(element, &mut fonts, &mut strings, &mut files);
        }
        for font in fonts {
            text.font(font);
        }
        for id in strings {
            text.string(layout.string_table, id);
        }
        for file in files {
            if images.contains_key(&file) {
                continue;
            }
            if let Some(surface) = ac2_assets::load_ac2_ui_surface(store, DataId::new(file))? {
                images.insert(
                    file,
                    UiPixels {
                        width: surface.width,
                        height: surface.height,
                        rgba: surface.rgba,
                    },
                );
            }
        }
    }
    if let Some(credits) = layouts.get(&LAYOUT_CREDITS) {
        crate::pregame::credits::preload_credits(store, credits, &text, &mut images)?;
    }
    text.freeze();
    Ok(PregameAssets {
        layouts,
        text: Some(text),
        strings: UiStrings::load(store)?,
        images,
    })
}
