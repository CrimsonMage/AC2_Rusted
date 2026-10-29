//! The quest guide's window (a modern addition, `ac2_guide`): the AC2Hero site baked into the
//! client, browsed in the retail UI. The window is another `Book` of the gameplay layout made as
//! its own root, as the help window and the Tracker are, its frame in nine pieces so it can take
//! any size; a copy of the chat strip's corner grip resizes it, and its size is kept with its
//! place.
//!
//! - Tabs (the book's text button) pick Quests, NPCs, Monsters, Guides or Rewards.
//! - The left pane is a search field and the tab's list (`ac2_guide::browse`), quests under
//!   level bands, in a second copy of the book's text area with a copy of its slider.
//! - The right pane is the page (`guide_page`): its facts, rewards with their item icons, text,
//!   links, gold map coordinates and pictures, in the book's own text area and slider.
//! - Back returns to the page before; Share puts the page's chat token in the chat entry line
//!   (`ac2_guide::browse::share_text`), which the player sends.
//!
//! A press on a link opens its page or item; on a coordinate it points the floating arrow there.
//! The guide is decoded on a worker when the window first opens and dropped when it closes, with
//! the pictures it decoded.

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use ac2_guide::browse::{self, Entry, Tab};
use ac2_guide::{Guide, Kind, MapCoord};

use crate::book::BookUi;
use crate::book_rows::{
    line_height, margin_left, margin_top, margins, position_to_pixels, text_width,
};
use crate::compose::{UiPixels, NINE_SLICE};
use crate::element::UiTree;
use crate::guide_page::{lay_out_item, lay_out_page, run_at, Action, Laid, Metrics, Row, Tone};
use crate::layout::{ElementDesc, LayoutDesc, MediaDesc};
use crate::mouse::{UiMessage, GLOBAL_MOUSE_DOWN, UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::scrollbar::MSG_NEW_POSITION;
use crate::token::{
    tokenize, BORDER_BOTTOM, BORDER_RIGHT, MAX_HEIGHT, MAX_WIDTH, MIN_HEIGHT, MIN_WIDTH,
    MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP, SIZE_IMAGE_TO_OWNER, TEXT_MAX_CHARS,
};
use crate::window_parts::{
    copy_of, entry_field, grip_art, inside, max_scroll, owned_by, place_button, place_entry,
    plain_image, sync_scroll, text_button, vertical_scrollbar, GRIP_SIZE,
};

/// The window's first size, and the smallest and largest it takes (logical pixels; at 1080p the
/// first size is under half the screen).
const SIZE: (i32, i32) = (640, 440);
const MIN_SIZE: (i32, i32) = (520, 320);
const MAX_SIZE: (i32, i32) = (1000, 740);
const FRAME_BORDER: i32 = 32;
/// The gap between the frame and the panes.
const PAD: i32 = 22;
/// The panes' top and the gap left at the bottom for the frame.
const PANES_TOP: i32 = 90;
const PANES_BOTTOM: i32 = 38;
/// The grip: a copy of the chat strip's bottom grip (a `UIElement_Resizebar`, layout
/// `0x22000005`) dragging the right edge too, with the DATs' diagonal resize cursor, and its size.
/// The DAT's grips draw nothing, so a grip mark ([`GRIP_ART`], made here) sits behind it.
const GRIP_LAYOUT_ELEMENT: u32 = 0x7122_c1ba;
const GRIP_CURSOR: u32 = 0x4100_5999;
/// The buttons' widths.
const TAB_WIDTH: i32 = 80;
const BACK_WIDTH: i32 = 60;
const SHARE_WIDTH: i32 = 70;
pub const GRIP_ART: u32 = 0xF100_0020;
const GRIP: i32 = GRIP_SIZE;
/// The guide's pictures in the HUD's images: ids outside the DATs' image ranges.
const PICTURE_BASE: u32 = 0xF200_0000;

/// What the window shows on its right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Page(u32),
    Item(u32),
}

/// What a press in the guide asks the HUD for.
#[derive(Debug, Clone, PartialEq)]
pub enum GuideRequest {
    /// Put this text in the chat entry line.
    Share(String),
    /// Point the floating arrow at a map coordinate.
    PointAt(MapCoord),
}

/// A line of the list.
#[derive(Debug, Clone, PartialEq)]
enum ListRow {
    Text(String),
    Heading(String),
    Open(View, String),
}

#[derive(Debug)]
struct Loaded {
    guide: Arc<Guide>,
    item_quests: Vec<Vec<u32>>,
}

/// The guide window.
#[derive(Debug)]
pub struct GuideWindow {
    book: BookUi,
    gameplay: Arc<LayoutDesc>,
    size: (i32, i32),
    /// The frame's images: (element, whether it spans the window's height).
    frames: Vec<(usize, bool)>,
    tabs: Vec<(usize, Tab)>,
    search: Option<(usize, Option<usize>)>,
    query: String,
    list: Option<usize>,
    list_slider: Option<usize>,
    list_rows: Vec<ListRow>,
    list_scroll: i32,
    page: Option<usize>,
    page_slider: Option<usize>,
    laid: Laid,
    laid_width: i32,
    page_scroll: i32,
    pictures: Vec<usize>,
    back: Option<usize>,
    share: Option<usize>,
    grip: Option<usize>,
    grip_art: Option<usize>,
    tab: Tab,
    view: Option<View>,
    history: Vec<View>,
    loaded: Option<Loaded>,
    loading: Option<Receiver<Guide>>,
    /// A page to open once the guide is loaded (a chat link pressed before it was).
    pending: Option<String>,
    decoded_tx: Sender<(u32, UiPixels)>,
    decoded_rx: Receiver<(u32, UiPixels)>,
    /// The pictures in the HUD's images, and those asked of the worker.
    uploaded: Vec<u32>,
    requested: Vec<u32>,
}

impl Clone for GuideWindow {
    /// A copy shows the same elements; the guide, its loading and the pictures stay with the
    /// original (the copy loads its own when it opens).
    fn clone(&self) -> Self {
        let (decoded_tx, decoded_rx) = channel();
        Self {
            book: self.book.clone(),
            gameplay: Arc::clone(&self.gameplay),
            size: self.size,
            frames: self.frames.clone(),
            tabs: self.tabs.clone(),
            search: self.search,
            query: self.query.clone(),
            list: self.list,
            list_slider: self.list_slider,
            list_rows: self.list_rows.clone(),
            list_scroll: self.list_scroll,
            page: self.page,
            page_slider: self.page_slider,
            laid: self.laid.clone(),
            laid_width: self.laid_width,
            page_scroll: self.page_scroll,
            pictures: self.pictures.clone(),
            back: self.back,
            share: self.share,
            grip: self.grip,
            grip_art: self.grip_art,
            tab: self.tab,
            view: self.view,
            history: self.history.clone(),
            loaded: None,
            loading: None,
            pending: self.pending.clone(),
            decoded_tx,
            decoded_rx,
            uploaded: Vec::new(),
            requested: Vec::new(),
        }
    }
}

impl GuideWindow {
    /// The hidden window over the gameplay layout's `Book`; `layouts` are searched for the entry
    /// field and the grip to copy. `None` without a `Book`.
    pub fn create(
        tree: &mut UiTree,
        gameplay: &Arc<LayoutDesc>,
        layouts: &[&Arc<LayoutDesc>],
    ) -> Option<Self> {
        let book = BookUi::create_window(tree, gameplay, "GuideBook")?;
        let root = book.field();
        let find = |tree: &UiTree, name: &str| tree.child_recursive(root, tokenize(name));
        let mut frames = Vec::new();
        for child in tree.children(root).to_vec() {
            if tree.get(child).is_some_and(|e| e.image().is_some()) {
                let tall = tree.get(child).is_some_and(|e| e.rect().height() > 100);
                tree.set_int(child, NINE_SLICE, FRAME_BORDER);
                frames.push((child, tall));
            }
        }
        for name in [
            "BookForeground",
            "BookClientAreaTextHeader",
            "BookHelpButton",
            "BookCopyButton",
        ] {
            if let Some(e) = find(tree, name) {
                tree.set_visible(e, false);
            }
        }
        if let Some(title) = find(tree, "Panel_Title_Book") {
            tree.set_text(title, "Quest Guide");
        }
        let page = find(tree, "BookClientAreaText");
        // The book's own slider runs across its bottom bar; the panes take the UI's vertical
        // scrollbar instead.
        if let Some(slider) = find(tree, "BookSliderBar") {
            tree.set_visible(slider, false);
        }
        let page_slider = vertical_scrollbar(tree, layouts, root, "GuidePageScroller");
        let mut tabs = Vec::new();
        for (i, tab) in Tab::ALL.into_iter().enumerate() {
            let id = format!("GuideChoice{i}");
            if let Some(e) = text_button(tree, gameplay, root, &id, tab.label(), (0, 0, 80)) {
                tabs.push((e, tab));
            }
        }
        let search = entry_field(
            tree,
            gameplay,
            layouts,
            root,
            ("GuideSearchInput", "GuideSearchBox"),
            (0, 0, 100, 22),
        );
        if let Some((field, _)) = search {
            tree.set_tooltip(
                field,
                Some("Search titles (and quests' rewards)".to_owned()),
            );
        }
        // The list: a second copy of the book's text area, with a scrollbar of its own.
        let list = page.and_then(|p| copy_of(tree, gameplay, root, p, "GuideListArea"));
        let list_slider = vertical_scrollbar(tree, layouts, root, "GuideListScroller");
        let back = text_button(tree, gameplay, root, "GuideGoBack", "Back", (0, 0, 60));
        let share = text_button(tree, gameplay, root, "GuideShareLink", "Share", (0, 0, 70));
        if let Some(share) = share {
            tree.set_tooltip(
                share,
                Some("Put a link to this page in the chat line".to_owned()),
            );
        }
        let grip = layouts
            .iter()
            .find_map(|layout| {
                Some((
                    Arc::clone(layout),
                    layout.find_element(GRIP_LAYOUT_ELEMENT)?,
                ))
            })
            .and_then(|(layout, template)| {
                let mut desc: ElementDesc = (*template).clone();
                desc.id = tokenize("GuideCornerGrip");
                for media in desc.states.iter_mut().flat_map(|s| s.media.iter_mut()) {
                    if let MediaDesc::Cursor { file, .. } = media {
                        *file = GRIP_CURSOR;
                    }
                }
                let grip = tree.create_element(layout, Arc::new(desc), false)?;
                tree.set_parent(grip, Some(root));
                tree.set_bool(grip, BORDER_RIGHT, true);
                tree.set_bool(grip, BORDER_BOTTOM, true);
                Some(grip)
            });
        let grip_art = grip.and_then(|grip| {
            let art = plain_image(
                tree,
                gameplay,
                root,
                "GuideGripArt",
                // Drawn once the window's frame has made the mark.
                0,
                (0, 0, GRIP, GRIP),
                0,
            )?;
            let z = tree.get(grip)?.z;
            tree.set_z_level(art, z + 1);
            Some(art)
        });
        for text in [page, list].into_iter().flatten() {
            tree.set_int(text, TEXT_MAX_CHARS, 1 << 20);
        }
        tree.set_int(root, MIN_WIDTH, MIN_SIZE.0);
        tree.set_int(root, MIN_HEIGHT, MIN_SIZE.1);
        tree.set_int(root, MAX_WIDTH, MAX_SIZE.0);
        tree.set_int(root, MAX_HEIGHT, MAX_SIZE.1);
        tree.set_bool(root, crate::token::ALLOW_RESIZE, true);
        let (decoded_tx, decoded_rx) = channel();
        let mut window = Self {
            book,
            gameplay: Arc::clone(gameplay),
            size: (0, 0),
            frames,
            tabs,
            search,
            query: String::new(),
            list,
            list_slider,
            list_rows: Vec::new(),
            list_scroll: 0,
            page,
            page_slider,
            laid: Laid::default(),
            laid_width: 0,
            page_scroll: 0,
            pictures: Vec::new(),
            back,
            share,
            grip,
            grip_art,
            tab: Tab::Pages(Kind::Quest),
            view: None,
            history: Vec::new(),
            loaded: None,
            loading: None,
            pending: None,
            decoded_tx,
            decoded_rx,
            uploaded: Vec::new(),
            requested: Vec::new(),
        };
        tree.resize_to(root, SIZE.0, SIZE.1);
        window.lay_out(tree);
        Some(window)
    }

    pub fn root(&self) -> usize {
        self.book.field()
    }

    pub fn is_shown(&self) -> bool {
        self.book.is_shown()
    }

    /// The tab shown and the page or item on the right.
    pub fn view(&self) -> (Tab, Option<View>) {
        (self.tab, self.view)
    }

    /// Opens the window (the guide starts decoding the first time).
    pub fn open(&mut self, tree: &mut UiTree) {
        if !self.is_shown() {
            self.book.show(tree, true);
        }
        if self.loaded.is_none() && self.loading.is_none() {
            let (tx, rx) = channel();
            self.loading = Some(rx);
            let _ = std::thread::Builder::new()
                .name("ac2 guide load".into())
                .spawn(move || {
                    let _ = tx.send(ac2_guide::baked());
                });
            self.show_list_message(tree, "Opening the guide...");
        }
    }

    /// Opens the window at the page named `slug` (a chat link).
    pub fn open_slug(&mut self, tree: &mut UiTree, slug: &str) {
        self.open(tree);
        self.pending = Some(slug.to_owned());
        self.apply_pending(tree);
    }

    /// Hides the window; the guide and its pictures are let go at the next frame.
    pub fn close(&mut self, tree: &mut UiTree) {
        self.book.show(tree, false);
    }

    /// The window's frame: the guide arrives from its worker, the search follows the field, a
    /// resize lays the window out again, decoded pictures go into the HUD's images; once
    /// closed, the guide and the pictures are dropped.
    pub fn frame(&mut self, tree: &mut UiTree, images: &mut HashMap<u32, UiPixels>) {
        if let std::collections::hash_map::Entry::Vacant(slot) = images.entry(GRIP_ART) {
            slot.insert(grip_art());
            if let Some(art) = self.grip_art {
                tree.set_image(art, GRIP_ART, 2);
            }
        }
        if !self.is_shown() {
            if self.loaded.is_some() || !self.uploaded.is_empty() {
                self.release(tree, images);
            }
            return;
        }
        if let Some(rx) = &self.loading {
            if let Ok(guide) = rx.try_recv() {
                self.loading = None;
                let item_quests = browse::item_quests(&guide);
                self.loaded = Some(Loaded {
                    guide: Arc::new(guide),
                    item_quests,
                });
                self.refresh_list(tree);
                if !self.apply_pending(tree) && self.view.is_none() {
                    self.show_welcome(tree);
                }
            }
        }
        let size = {
            let r = tree.screen_box(self.root());
            (r.width(), r.height())
        };
        if size != self.size {
            self.lay_out(tree);
        }
        if let Some((field, _)) = self.search {
            let typed = tree
                .get(field)
                .and_then(|e| e.text())
                .unwrap_or_default()
                .trim()
                .to_owned();
            if typed != self.query {
                self.query = typed;
                self.list_scroll = 0;
                self.refresh_list(tree);
            }
        }
        while let Ok((id, pixels)) = self.decoded_rx.try_recv() {
            images.insert(id, pixels);
            self.uploaded.push(id);
            // The pictures showing it draw it now.
            for &element in &self.pictures {
                if tree.get(element).and_then(|e| e.image()).map(|i| i.file) == Some(id) {
                    tree.set_image(element, id, 2);
                }
            }
        }
    }

    /// Drops the guide, the pictures and the page.
    fn release(&mut self, tree: &mut UiTree, images: &mut HashMap<u32, UiPixels>) {
        for id in self.uploaded.drain(..) {
            images.remove(&id);
        }
        self.requested.clear();
        self.loaded = None;
        self.loading = None;
        self.view = None;
        self.history.clear();
        self.laid = Laid::default();
        self.list_rows.clear();
        self.clear_pictures(tree);
        for text in [self.page, self.list].into_iter().flatten() {
            tree.set_text(text, "");
        }
        // Pictures still decoding are dropped as they arrive.
        let (tx, rx) = channel();
        self.decoded_tx = tx;
        self.decoded_rx = rx;
    }

    fn apply_pending(&mut self, tree: &mut UiTree) -> bool {
        let Some(loaded) = &self.loaded else {
            return false;
        };
        let Some(slug) = self.pending.take() else {
            return false;
        };
        match browse::page_of_slug(&loaded.guide, &slug) {
            Some(page) => self.show(tree, View::Page(page)),
            None => self.show_page_message(tree, &format!("This guide has no page {slug}.")),
        }
        true
    }

    /// Places the window's parts for its size.
    fn lay_out(&mut self, tree: &mut UiTree) {
        let root = self.root();
        let r = tree.screen_box(root);
        let (w, h) = (r.width(), r.height());
        self.size = (w, h);
        for &(frame, tall) in &self.frames {
            let height = if tall {
                h
            } else {
                tree.get(frame).map_or(0, |e| e.rect().height())
            };
            tree.move_to(frame, 0, 0);
            tree.resize_to(frame, w, height);
        }
        let find = |tree: &UiTree, name: &str| tree.child_recursive(root, tokenize(name));
        if let Some(close) = find(tree, "BookCloseButton") {
            tree.move_to(close, w - 34, 7);
        }
        if let Some(title) = find(tree, "Panel_Title_Book") {
            tree.resize_to(title, w, 29);
        }
        if let Some(bottom) = find(tree, "BookBottom") {
            tree.move_to(bottom, 0, h - 32);
            tree.resize_to(bottom, w, 30);
            tree.set_int(bottom, NINE_SLICE, FRAME_BORDER);
            tree.set_visible(bottom, false);
        }
        // Every part is placed and sized here: the layout's edges stretch them with the window.
        let mut x = PAD;
        for &(tab, _) in &self.tabs {
            place_button(tree, tab, (x, 38, TAB_WIDTH));
            x += TAB_WIDTH + 4;
        }
        // The scrollbar's width as the layout draws it.
        let slider_w = self.page_slider.and_then(|s| tree.get(s)).map_or(15, |e| {
            i32::from(e.reference.rect.x1) - i32::from(e.reference.rect.x0) + 1
        });
        let left_w = ((w - 2 * PAD) * 36 / 100).clamp(180, 260);
        let panes_h = h - PANES_TOP - PANES_BOTTOM;
        if let Some((field, field_box)) = self.search {
            place_entry(tree, field, field_box, (PAD, 63, left_w, 22));
        }
        if let Some(list) = self.list {
            tree.move_to(list, PAD, PANES_TOP);
            tree.resize_to(list, left_w - slider_w - 2, panes_h);
        }
        if let Some(slider) = self.list_slider {
            tree.move_to(slider, PAD + left_w - slider_w, PANES_TOP);
            tree.resize_to(slider, slider_w, panes_h);
        }
        let right_x = PAD + left_w + 12;
        if let Some(back) = self.back {
            place_button(tree, back, (right_x, 64, BACK_WIDTH));
        }
        if let Some(share) = self.share {
            place_button(tree, share, (right_x + BACK_WIDTH + 6, 64, SHARE_WIDTH));
        }
        if let Some(page) = self.page {
            tree.move_to(page, right_x, PANES_TOP);
            tree.resize_to(page, w - PAD - right_x - slider_w - 2, panes_h);
        }
        if let Some(slider) = self.page_slider {
            tree.move_to(slider, w - PAD - slider_w, PANES_TOP);
            tree.resize_to(slider, slider_w, panes_h);
        }
        for grip in [self.grip, self.grip_art].into_iter().flatten() {
            tree.move_to(grip, w - GRIP - 6, h - GRIP - 6);
            tree.resize_to(grip, GRIP, GRIP);
        }
        self.sync_list(tree);
        if self.page_wrap(tree) != self.laid_width {
            self.relay_page(tree);
        } else {
            self.sync_page(tree);
        }
    }

    // ---- the list ----

    fn refresh_list(&mut self, tree: &mut UiTree) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let guide = &loaded.guide;
        let entries = browse::entries(guide, self.tab, &self.query);
        let count = entries
            .iter()
            .filter(|e| !matches!(e, Entry::Heading(_)))
            .count();
        let mut rows = vec![ListRow::Text(format!("{count} {}", self.tab.noun()))];
        for entry in entries {
            rows.push(match entry {
                Entry::Heading(text) => ListRow::Heading(text),
                Entry::Page(i) => {
                    let page = &guide.pages[i as usize];
                    let text = match page.level() {
                        Some(level) if self.tab == Tab::Pages(Kind::Quest) => {
                            format!("{level:>3}  {}", page.title)
                        }
                        _ => format!("  {}", page.title),
                    };
                    ListRow::Open(View::Page(i), text)
                }
                Entry::Item(i) => {
                    ListRow::Open(View::Item(i), format!("  {}", guide.items[i as usize].name))
                }
            });
        }
        self.list_rows = rows;
        self.render_list(tree);
    }

    fn show_list_message(&mut self, tree: &mut UiTree, text: &str) {
        self.list_rows = vec![ListRow::Text(text.to_owned())];
        self.render_list(tree);
    }

    fn render_list(&mut self, tree: &mut UiTree) {
        let Some(list) = self.list else {
            return;
        };
        tree.set_text(list, "");
        let picked = self.view;
        for (i, row) in self.list_rows.iter().enumerate() {
            if i > 0 {
                tree.append_text_rgb(list, "\n", None);
            }
            let (text, tone) = match row {
                ListRow::Text(text) => (text.as_str(), Tone::Label),
                ListRow::Heading(text) => (text.as_str(), Tone::Heading),
                ListRow::Open(view, text) if Some(*view) == picked => (text.as_str(), Tone::Link),
                ListRow::Open(_, text) => (text.as_str(), Tone::Body),
            };
            tree.append_text_rgb(list, text, tone.rgb());
        }
        self.sync_list(tree);
    }

    fn sync_list(&mut self, tree: &mut UiTree) {
        let (Some(list), rows) = (self.list, self.list_rows.len()) else {
            return;
        };
        self.list_scroll = sync_scroll(tree, list, self.list_slider, rows, self.list_scroll);
    }

    // ---- the page ----

    /// The page text's usable width.
    fn page_wrap(&self, tree: &UiTree) -> i32 {
        self.page.map_or(0, |page| {
            tree.get(page).map_or(0, |e| e.rect().width()) - margins(tree, page) - 4
        })
    }

    /// Shows `view` on the right, the one before kept for Back.
    fn show(&mut self, tree: &mut UiTree, view: View) {
        if let Some(current) = self.view.filter(|&c| c != view) {
            self.history.push(current);
            if self.history.len() > 64 {
                self.history.remove(0);
            }
        }
        self.view = Some(view);
        self.page_scroll = 0;
        self.relay_page(tree);
        self.render_list(tree);
    }

    fn show_welcome(&mut self, tree: &mut UiTree) {
        let pages = self.loaded.as_ref().map_or(0, |l| l.guide.pages.len());
        if pages == 0 {
            return self.show_page_message(
                tree,
                "This client was built without the guide (AC2HERO_DIR was not set).",
            );
        }
        self.show_page_message(
            tree,
            &format!(
                "{pages} pages from AC2Hero. Pick a quest, or search titles and rewards. \
                 Gold coordinates point the arrow; Share links a page in chat."
            ),
        );
    }

    fn show_page_message(&mut self, tree: &mut UiTree, text: &str) {
        self.laid = Laid {
            rows: vec![Row {
                runs: vec![crate::guide_page::Run {
                    text: text.to_owned(),
                    tone: Tone::Body,
                    action: None,
                }],
            }],
            pictures: Vec::new(),
        };
        self.laid_width = 0;
        self.render_page(tree);
    }

    /// Lays the view's page out again at the text's width.
    fn relay_page(&mut self, tree: &mut UiTree) {
        let (Some(page_text), Some(loaded), Some(view)) = (self.page, &self.loaded, self.view)
        else {
            return;
        };
        let guide = Arc::clone(&loaded.guide);
        let wrap = self.page_wrap(tree);
        let measure = |s: &str| text_width(tree, page_text, s);
        let size = |image: u32| {
            guide
                .images
                .get(image as usize)
                .and_then(|i| ac2_guide::picture::size(&i.bytes))
        };
        let metrics = Metrics {
            width: &measure,
            line: line_height(tree, page_text),
            wrap,
            image_size: &size,
        };
        self.laid = match view {
            View::Page(i) => lay_out_page(&guide, i, &metrics),
            View::Item(i) => {
                let quests = loaded
                    .item_quests
                    .get(i as usize)
                    .map_or(&[][..], Vec::as_slice);
                lay_out_item(&guide, i, quests, &metrics)
            }
        };
        self.laid_width = wrap;
        if let Some(share) = self.share {
            tree.set_visible(share, matches!(view, View::Page(_)));
        }
        self.render_page(tree);
        self.request_pictures();
    }

    fn render_page(&mut self, tree: &mut UiTree) {
        let Some(page) = self.page else {
            return;
        };
        tree.set_text(page, "");
        for (i, row) in self.laid.rows.iter().enumerate() {
            if i > 0 {
                tree.append_text_rgb(page, "\n", None);
            }
            for run in &row.runs {
                tree.append_text_rgb(page, &run.text, run.tone.rgb());
            }
        }
        self.clear_pictures(tree);
        let gameplay = Arc::clone(&self.gameplay);
        for picture in self.laid.pictures.clone() {
            let id = PICTURE_BASE + picture.image;
            if let Some(e) = plain_image(
                tree,
                &gameplay,
                page,
                "GuidePictureArt",
                id,
                (0, 0, 1, 1),
                0,
            ) {
                tree.set_int(e, NINE_SLICE, 0);
                tree.set_bool(e, SIZE_IMAGE_TO_OWNER, true);
                tree.resize_to(e, picture.width, picture.height);
                self.pictures.push(e);
            }
        }
        if let Some(back) = self.back {
            tree.set_visible(back, !self.history.is_empty());
        }
        self.sync_page(tree);
    }

    fn clear_pictures(&mut self, tree: &mut UiTree) {
        for element in self.pictures.drain(..) {
            tree.set_parent(element, None);
            tree.delete(element);
        }
    }

    /// The page's scroll applied to its text, its slider and its pictures.
    fn sync_page(&mut self, tree: &mut UiTree) {
        let Some(page) = self.page else {
            return;
        };
        self.page_scroll = sync_scroll(
            tree,
            page,
            self.page_slider,
            self.laid.rows.len(),
            self.page_scroll,
        );
        let line = line_height(tree, page);
        let left = margin_left(tree, page);
        let top = margin_top(tree, page);
        for (&element, picture) in self.pictures.iter().zip(&self.laid.pictures) {
            let y = top + picture.row as i32 * line - self.page_scroll;
            tree.move_to(element, left + picture.x, y);
        }
    }

    /// Asks a worker for the page's pictures not yet decoded.
    fn request_pictures(&mut self) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let wanted: Vec<u32> = self
            .laid
            .pictures
            .iter()
            .map(|p| p.image)
            .filter(|&i| {
                let id = PICTURE_BASE + i;
                !self.uploaded.contains(&id) && !self.requested.contains(&id)
            })
            .collect();
        if wanted.is_empty() {
            return;
        }
        self.requested
            .extend(wanted.iter().map(|&i| PICTURE_BASE + i));
        let guide = Arc::clone(&loaded.guide);
        let tx = self.decoded_tx.clone();
        let _ = std::thread::Builder::new()
            .name("ac2 guide pictures".into())
            .spawn(move || {
                for image in wanted {
                    let Some(bytes) = guide.images.get(image as usize).map(|i| &i.bytes) else {
                        continue;
                    };
                    if let Some((width, height, rgba)) = ac2_guide::picture::decode(bytes) {
                        let pixels = UiPixels {
                            width,
                            height,
                            rgba: rgba.into(),
                        };
                        if tx.send((PICTURE_BASE + image, pixels)).is_err() {
                            return;
                        }
                    }
                }
            });
    }

    // ---- messages ----

    /// The window's messages: the tabs, Back and Share, the sliders and the wheel, and presses
    /// on the list and the page. Returns whether the message was the window's, and what it asks.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        message: &UiMessage,
    ) -> (bool, Option<GuideRequest>) {
        if !self.is_shown() {
            return (false, None);
        }
        if self.wheel(tree, message) {
            return (true, None);
        }
        let root = self.root();
        if !owned_by(tree, message.element, root) {
            return (false, None);
        }
        let element = message.element;
        if message.message == MSG_BUTTON_RELEASE_ON {
            if message.element_id == tokenize("BookCloseButton") {
                self.close(tree);
                return (true, None);
            }
            if let Some(&(_, tab)) = self.tabs.iter().find(|(e, _)| owned_by(tree, element, *e)) {
                if tab != self.tab {
                    self.tab = tab;
                    self.list_scroll = 0;
                    self.refresh_list(tree);
                }
                return (true, None);
            }
            if self.back.is_some_and(|b| owned_by(tree, element, b)) {
                if let Some(view) = self.history.pop() {
                    self.view = Some(view);
                    self.page_scroll = 0;
                    self.relay_page(tree);
                    self.render_list(tree);
                }
                return (true, None);
            }
            if self.share.is_some_and(|s| owned_by(tree, element, s)) {
                let text = match (self.view, &self.loaded) {
                    (Some(View::Page(i)), Some(loaded)) => {
                        loaded.guide.pages.get(i as usize).map(browse::share_text)
                    }
                    _ => None,
                };
                return (true, text.map(GuideRequest::Share));
            }
        }
        if message.message == MSG_NEW_POSITION {
            if element.is_some() && element == self.list_slider {
                let max = max_scroll(tree, self.list, self.list_rows.len());
                let position = tree.scrollbar_position(self.list_slider.unwrap_or_default());
                self.list_scroll = position_to_pixels(position, max);
                self.sync_list(tree);
                return (true, None);
            }
            if element.is_some() && element == self.page_slider {
                let max = max_scroll(tree, self.page, self.laid.rows.len());
                let position = tree.scrollbar_position(self.page_slider.unwrap_or_default());
                self.page_scroll = position_to_pixels(position, max);
                self.sync_page(tree);
                return (true, None);
            }
        }
        if message.message == MSG_MOUSE_PRESS_TOP {
            let pointer = tree.pointer();
            if let Some(view) = self.list_view_at(tree, pointer) {
                self.show(tree, view);
                return (true, None);
            }
            if let Some(action) = self.page_action_at(tree, pointer) {
                return match action {
                    Action::Page(i) => {
                        self.show(tree, View::Page(i));
                        (true, None)
                    }
                    Action::Item(i) => {
                        self.show(tree, View::Item(i));
                        (true, None)
                    }
                    Action::Coord(at) => (true, Some(GuideRequest::PointAt(at))),
                };
            }
        }
        (true, None)
    }

    /// A wheel turn over the list or the page scrolls it three lines (the tree broadcasts the
    /// turn as a global mouse-down carrying its action).
    fn wheel(&mut self, tree: &mut UiTree, message: &UiMessage) -> bool {
        let action = message.data as u32;
        if message.element.is_some()
            || message.message != GLOBAL_MOUSE_DOWN
            || !matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT)
        {
            return false;
        }
        let pointer = tree.pointer();
        let step = |tree: &UiTree, text: usize| {
            let line = line_height(tree, text) * 3;
            if action == UI_WHEEL_IN {
                -line
            } else {
                line
            }
        };
        if let Some(list) = self.list.filter(|&l| inside(tree, l, pointer)) {
            self.list_scroll += step(tree, list);
            self.sync_list(tree);
            return true;
        }
        if let Some(page) = self.page.filter(|&p| inside(tree, p, pointer)) {
            self.page_scroll += step(tree, page);
            self.sync_page(tree);
            return true;
        }
        false
    }

    fn list_view_at(&self, tree: &UiTree, (x, y): (i32, i32)) -> Option<View> {
        let list = self.list.filter(|&l| inside(tree, l, (x, y)))?;
        let rect = tree.screen_box(list);
        let top = margin_top(tree, list);
        let row = (y - rect.y0 - top + self.list_scroll) / line_height(tree, list);
        match self.list_rows.get(usize::try_from(row).ok()?)? {
            ListRow::Open(view, _) => Some(*view),
            _ => None,
        }
    }

    fn page_action_at(&self, tree: &UiTree, (x, y): (i32, i32)) -> Option<Action> {
        let page = self.page.filter(|&p| inside(tree, p, (x, y)))?;
        let rect = tree.screen_box(page);
        let top = margin_top(tree, page);
        let row = (y - rect.y0 - top + self.page_scroll) / line_height(tree, page);
        let row = self.laid.rows.get(usize::try_from(row).ok()?)?;
        let measure = |s: &str| text_width(tree, page, s);
        run_at(row, x - rect.x0 - margin_left(tree, page), &measure)?.action
    }
}

#[cfg(test)]
#[path = "guide_window_tests.rs"]
mod tests;
