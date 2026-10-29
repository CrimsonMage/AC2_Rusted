//! The Tracker's interface (a modern addition, `ac2_macro::tracker`): an eyeball button on the
//! radar's edge that opens and closes the Tracker window, and the window itself.
//!
//! Everything is cut from the DAT layouts. The button is a copy of the radar's own coordinates
//! button (`CoordBoxButton`, its orb art and states) on the opposite side of the ring, with the
//! eyeball icon (`icon_it_blowgunsniper_eyeball`) over it. The window is another `Book` of the
//! gameplay layout made as its own root (as the help window is), smaller, its frame and title bar
//! drawn in nine pieces so their corners keep their shape; the pattern field is a copy of a
//! single-line entry field and the buttons copies of the book's own text button.
//!
//! The text area lists the patterns (a press on `[x]` removes one) and the matches (a press on a
//! name picks it, on `[x]` dismisses it), scrolled by the wheel and the book's slider. A press
//! becomes a [`TrackerRequest`] for the client to run against its `ac2_macro::tracker::Tracker`.

use std::sync::Arc;

use ac2_macro::tracker::Presentation;

use crate::book::BookUi;
use crate::book_rows::{line_height, margin_left, position_to_pixels, text_width};
use crate::compose::{UiPixels, NINE_SLICE};
use crate::element::UiTree;
use crate::layout::{ElementDesc, LayoutDesc};
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS, GLOBAL_MOUSE_DOWN, UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::scrollbar::MSG_NEW_POSITION;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP, SIZE_IMAGE_TO_OWNER};
use crate::window_parts::{entry_field, owned_by, plain_image, text_button};

/// The Enter key's action, as the chat reads it (`gmChatUI`'s `ENTER`).
const ENTER_KEY: u32 = 0x4000_0018;
/// The eyeball drawn over the button (`icon_it_blowgunsniper_eyeball.tga`).
pub const EYE_ICON: u32 = 0x4100_1AAB;
/// The eyeball as the button draws it, made from `EYE_ICON` by [`bake_eye`]: an id outside the
/// DATs' image ranges.
pub const EYE_ORB: u32 = 0xF100_0001;
/// The coordinates button's socket and bracket as the radar's top art paints them (x, y, width,
/// height on that image), copied mirrored behind the Tracker button by [`bake_eye`] under
/// [`EYE_BRACKET`], so the two buttons match.
const SOCKET: (i32, i32, i32, i32) = (0, 12, 25, 35);
pub const EYE_BRACKET: u32 = 0xF100_0002;
/// How much of the eye icon's frame is left out on each side (the icon's own border).
const EYE_ICON_BORDER: f32 = 5.0;
/// The ids of the elements made here are the client's own, named so they never read as a
/// layout's element names. The radar button's id, and where it sits on the radar's top image: the coordinates button's
/// place mirrored across the ring (it sits at (6, 25) on the 142-pixel-wide image).
const BUTTON_ID: u32 = tokenize("TrackerEyeOrb");
/// The eye drawn over the button, and the socket behind it.
const EYE_ART_ID: u32 = tokenize("TrackerEyeArt");
const EYE_SOCKET_NAME: &str = "TrackerEyeSocket";
const BUTTON_AT: (i32, i32) = (123, 25);
/// The window's size, and the corner size its frame and title bar keep.
const WINDOW: (i32, i32) = (380, 440);
const FRAME_BORDER: i32 = 32;

/// What a press in the Tracker's interface asks the client for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackerRequest {
    /// Add the pattern typed in the field (`exact`: the whole name).
    Add {
        pattern: String,
        exact: bool,
    },
    /// Add the selected object's name.
    AddSelected,
    /// Reload the pattern list from its file.
    Reload,
    RemovePattern(usize),
    /// Pick a match: select it and point the arrow at it.
    Select(u64),
    Dismiss(u64),
}

#[derive(Debug, Clone)]
struct Piece {
    text: String,
    action: Option<TrackerRequest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    Add,
    Exact,
    AddSelected,
    Reload,
}

impl Button {
    const ALL: [Button; 4] = [
        Button::Add,
        Button::Exact,
        Button::AddSelected,
        Button::Reload,
    ];

    fn label(self) -> &'static str {
        match self {
            Button::Add => "Add",
            Button::Exact => "Exact",
            Button::AddSelected => "Add selected",
            Button::Reload => "Reload",
        }
    }
}

/// The eyeball on the radar and the window it opens.
#[derive(Debug, Clone)]
pub struct TrackerUi {
    button: Option<usize>,
    book: Option<BookUi>,
    entry: Option<usize>,
    buttons: Vec<(usize, Button)>,
    rows: Vec<Vec<Piece>>,
    scroll: i32,
}

impl TrackerUi {
    /// The button over the radar (`radar_top`: the radar's top image, the coordinates button's
    /// grandparent) and the hidden window over the gameplay layout's `Book`; `layouts` are
    /// searched for the entry field to copy. Either part is left out when its template is missing.
    pub fn create(
        tree: &mut UiTree,
        gameplay: &Arc<LayoutDesc>,
        layouts: &[&Arc<LayoutDesc>],
        radar_top: Option<usize>,
    ) -> Self {
        let button = radar_top.and_then(|top| Self::create_button(tree, gameplay, top));
        let mut ui = Self {
            button,
            book: None,
            entry: None,
            buttons: Vec::new(),
            rows: Vec::new(),
            scroll: 0,
        };
        ui.create_window(tree, gameplay, layouts);
        ui
    }

    fn create_button(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>, top: usize) -> Option<usize> {
        let template = gameplay.find_element(tokenize("CoordBoxButton"))?;
        let mut desc: ElementDesc = (*template).clone();
        desc.id = BUTTON_ID;
        let button = tree.create_element(Arc::clone(gameplay), Arc::new(desc), false)?;
        tree.set_parent(button, Some(top));
        tree.move_to(button, BUTTON_AT.0, BUTTON_AT.1);
        // The template's own size: its edges would stretch it over the new parent.
        let size = (
            i32::from(template.rect.x1) - i32::from(template.rect.x0) + 1,
            i32::from(template.rect.y1) - i32::from(template.rect.y0) + 1,
        );
        tree.resize_to(button, size.0, size.1);
        // The eyeball over the orb, the orb's rim showing round it: a copy of the button with no
        // states of its own, sized to the orb less a pixel each side. Its image is the icon until
        // `bake_eye` makes the round eye from it.
        let mut icon_desc: ElementDesc = (*template).clone();
        icon_desc.id = EYE_ART_ID;
        icon_desc.states.clear();
        if let Some(icon) = tree.create_element(Arc::clone(gameplay), Arc::new(icon_desc), false) {
            tree.set_parent(icon, Some(button));
            tree.move_to(icon, 1, 1);
            tree.resize_to(icon, size.0 - 2, size.1 - 2);
            tree.set_image(icon, EYE_ICON, 2);
            tree.set_bool(icon, SIZE_IMAGE_TO_OWNER, true);
        }
        // The mirrored socket behind the button (in front of the radar's top art): the top art's
        // image until `bake_eye` cuts the socket out of it.
        let width = tree.get(top).map_or(0, |e| e.rect().width());
        let top_art = tree.get(top).and_then(|e| e.image()).map(|i| i.file);
        if let Some(art) = top_art {
            let rect = (width - SOCKET.0 - SOCKET.2, SOCKET.1, SOCKET.2, SOCKET.3);
            if let Some(socket) = plain_image(tree, gameplay, top, EYE_SOCKET_NAME, art, rect, 0) {
                if let Some(z) = tree.get(button).map(|e| e.z) {
                    tree.set_z_level(socket, z + 1);
                }
            }
        }
        tree.set_tooltip(button, Some("Tracker".to_owned()));
        Some(button)
    }

    fn create_window(
        &mut self,
        tree: &mut UiTree,
        gameplay: &Arc<LayoutDesc>,
        layouts: &[&Arc<LayoutDesc>],
    ) {
        let Some(book) = BookUi::create_window(tree, gameplay, "TrackerBook") else {
            return;
        };
        let root = book.field();
        let find = |tree: &UiTree, name: &str| tree.child_recursive(root, tokenize(name));
        tree.resize_to(root, WINDOW.0, WINDOW.1);
        // The frame and the title bar: every image under the root that is the window's width
        // keeps its corners.
        let children: Vec<usize> = tree.children(root).to_vec();
        for child in children {
            let Some(element) = tree.get(child) else {
                continue;
            };
            if element.image().is_some() {
                let height = element.rect().height();
                tree.resize_to(child, WINDOW.0, height.min(WINDOW.1));
                if height > 100 {
                    tree.resize_to(child, WINDOW.0, WINDOW.1);
                }
                tree.set_int(child, NINE_SLICE, FRAME_BORDER);
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
        if let Some(close) = find(tree, "BookCloseButton") {
            tree.move_to(close, WINDOW.0 - 34, 7);
        }
        if let Some(title) = find(tree, "Panel_Title_Book") {
            tree.resize_to(title, WINDOW.0, 29);
            tree.set_text(title, "Tracker");
        }
        // The pattern field and the buttons: two rows under the title.
        let (left, right) = (22, WINDOW.0 - 22);
        if let Some((field, _)) = entry_field(
            tree,
            gameplay,
            layouts,
            root,
            ("TrackerPatternInput", "TrackerFieldBox"),
            (left, 39, right - left - 126, 22),
        ) {
            tree.set_tooltip(field, Some("A name, or a regular expression".to_owned()));
            self.entry = Some(field);
        }
        let place = [
            (Button::Add, (right - 120, 40, 58)),
            (Button::Exact, (right - 58, 40, 58)),
            (Button::AddSelected, (left, 66, 120)),
            (Button::Reload, (left + 126, 66, 80)),
        ];
        for (button, at) in place {
            let id = format!("TrackerAction{}", button as u32);
            if let Some(element) = text_button(tree, gameplay, root, &id, button.label(), at) {
                self.buttons.push((element, button));
            }
        }
        if let Some(text) = find(tree, "BookClientAreaText") {
            tree.move_to(text, left, 96);
            tree.resize_to(text, right - left, WINDOW.1 - 96 - 40);
        }
        if let Some(bottom) = find(tree, "BookBottom") {
            tree.move_to(bottom, 0, WINDOW.1 - 32);
            tree.resize_to(bottom, WINDOW.0, 30);
            tree.set_int(bottom, NINE_SLICE, FRAME_BORDER);
        }
        debug_assert!(Button::ALL.len() == 4);
        self.book = Some(book);
    }

    /// The radar's eyeball button, when it was made.
    pub fn button(&self) -> Option<usize> {
        self.button
    }

    /// The window's root, when it was made.
    pub fn window(&self) -> Option<usize> {
        self.book.as_ref().map(BookUi::field)
    }

    pub fn is_shown(&self) -> bool {
        self.book.as_ref().is_some_and(BookUi::is_shown)
    }

    pub fn show(&mut self, tree: &mut UiTree, shown: bool) {
        if let Some(book) = &mut self.book {
            book.show(tree, shown);
        }
    }

    /// Lays the Tracker's list out in the text area: the patterns, then the matches with their
    /// distances and coordinates.
    pub fn present(&mut self, tree: &mut UiTree, presentation: &Presentation) {
        let mut rows: Vec<Vec<Piece>> = Vec::new();
        let plain = |text: &str| Piece {
            text: text.to_owned(),
            action: None,
        };
        rows.push(vec![plain("Tracking")]);
        if presentation.patterns.is_empty() {
            rows.push(vec![plain(
                "  Nothing yet: type a name above and press Add.",
            )]);
        }
        for (index, pattern) in presentation.patterns.iter().enumerate() {
            rows.push(vec![
                plain("  "),
                Piece {
                    text: "[x]".to_owned(),
                    action: Some(TrackerRequest::RemovePattern(index)),
                },
                plain(&format!(" {}", pattern.display)),
            ]);
        }
        rows.push(Vec::new());
        rows.push(vec![plain("Found")]);
        if presentation.matches.is_empty() {
            rows.push(vec![plain("  Nothing in range.")]);
        }
        for found in presentation.matches.iter() {
            let picked = presentation.pointer_target == Some(found.object);
            let metres = f64::from(found.distance_decimetres) / 10.0;
            let mut row = vec![
                plain(if picked { "> " } else { "  " }),
                Piece {
                    text: found.name.to_string(),
                    action: Some(TrackerRequest::Select(found.object)),
                },
                plain(&format!("  {metres:.1} m")),
            ];
            if let Some(at) = &found.coordinates {
                row.push(plain(&format!("  {at}")));
            }
            row.push(plain("  "));
            row.push(Piece {
                text: "[x]".to_owned(),
                action: Some(TrackerRequest::Dismiss(found.object)),
            });
            rows.push(row);
        }
        self.rows = rows;
        if let Some(text) = self.text(tree) {
            let lines: Vec<String> = self
                .rows
                .iter()
                .map(|row| row.iter().map(|p| p.text.as_str()).collect())
                .collect();
            tree.set_text(text, &lines.join("\n"));
        }
        self.sync_scroll(tree);
    }

    /// The interface's messages: the eyeball toggles the window, the close button hides it, the
    /// buttons and a press on the list make requests, the wheel and the slider scroll. Returns
    /// whether the message was the Tracker's, and what it asks for.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        message: &UiMessage,
    ) -> (bool, Option<TrackerRequest>) {
        if self.button.is_some() && message.element == self.button {
            if message.message == MSG_BUTTON_RELEASE_ON {
                let shown = !self.is_shown();
                self.show(tree, shown);
            }
            return (true, None);
        }
        if !self.is_shown() {
            return (false, None);
        }
        if self.wheel(tree, message) {
            return (true, None);
        }
        let Some(root) = self.window() else {
            return (false, None);
        };
        if !owned_by(tree, message.element, root) {
            return (false, None);
        }
        if message.element_id == tokenize("BookCloseButton")
            && message.message == MSG_BUTTON_RELEASE_ON
        {
            self.show(tree, false);
            return (true, None);
        }
        if message.element_id == tokenize("BookSliderBar") && message.message == MSG_NEW_POSITION {
            if let Some(slider) = tree.child_recursive(root, tokenize("BookSliderBar")) {
                let position = tree.scrollbar_position(slider);
                self.scroll = position_to_pixels(position, self.max_scroll(tree));
                self.sync_scroll(tree);
            }
            return (true, None);
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            let pressed = self
                .buttons
                .iter()
                .find(|(element, _)| owned_by(tree, message.element, *element))
                .map(|(_, button)| *button);
            if let Some(button) = pressed {
                return (true, self.button_request(tree, button));
            }
        }
        if message.element_id == tokenize("BookClientAreaText")
            && message.message == MSG_MOUSE_PRESS_TOP
        {
            return (true, self.action_at(tree, tree.pointer()));
        }
        (false, None)
    }

    /// Enter in the pattern field while it has the focus: the press is taken out of `messages`
    /// (so no other panel acts on it) and adds the typed pattern.
    pub fn take_enter(
        &mut self,
        tree: &mut UiTree,
        messages: &mut Vec<UiMessage>,
    ) -> Option<TrackerRequest> {
        let entry = self.entry?;
        if !self.is_shown() || tree.active() != Some(entry) {
            return None;
        }
        let at = messages.iter().position(|m| {
            m.element.is_none() && m.message == GLOBAL_KEY_PRESS && m.data as u32 == ENTER_KEY
        })?;
        messages.remove(at);
        self.button_request(tree, Button::Add)
    }

    fn button_request(&self, tree: &mut UiTree, button: Button) -> Option<TrackerRequest> {
        let typed = |tree: &mut UiTree| -> Option<String> {
            let entry = self.entry?;
            let text = tree.get(entry)?.text()?.trim().to_owned();
            tree.clear_text(entry);
            (!text.is_empty()).then_some(text)
        };
        match button {
            Button::Add => typed(tree).map(|pattern| TrackerRequest::Add {
                pattern,
                exact: false,
            }),
            Button::Exact => typed(tree).map(|pattern| TrackerRequest::Add {
                pattern,
                exact: true,
            }),
            Button::AddSelected => Some(TrackerRequest::AddSelected),
            Button::Reload => Some(TrackerRequest::Reload),
        }
    }

    fn text(&self, tree: &UiTree) -> Option<usize> {
        tree.child_recursive(self.window()?, tokenize("BookClientAreaText"))
    }

    /// The action under the pointer: the row at its height, the piece at its x.
    fn action_at(&self, tree: &UiTree, (x, y): (i32, i32)) -> Option<TrackerRequest> {
        let text = self.text(tree)?;
        let rect = tree.screen_box(text);
        if x < rect.x0 || x >= rect.x1 || y < rect.y0 || y >= rect.y1 {
            return None;
        }
        let row = ((y - rect.y0 + self.scroll) / line_height(tree, text)) as usize;
        let mut left = x - rect.x0 - margin_left(tree, text);
        for piece in self.rows.get(row)? {
            let width = text_width(tree, text, &piece.text);
            if left < width {
                return piece.action.clone();
            }
            left -= width;
        }
        None
    }

    /// A wheel turn over the list scrolls it a line (the tree broadcasts the turn as a global
    /// mouse-down carrying its action).
    fn wheel(&mut self, tree: &mut UiTree, message: &UiMessage) -> bool {
        let action = message.data as u32;
        if message.element.is_some()
            || message.message != GLOBAL_MOUSE_DOWN
            || !matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT)
        {
            return false;
        }
        let Some(text) = self.text(tree) else {
            return false;
        };
        let rect = tree.screen_box(text);
        let (x, y) = tree.pointer();
        if x < rect.x0 || x >= rect.x1 || y < rect.y0 || y >= rect.y1 {
            return false;
        }
        let line = line_height(tree, text);
        self.scroll += if action == UI_WHEEL_IN { -line } else { line };
        self.sync_scroll(tree);
        true
    }

    fn max_scroll(&self, tree: &UiTree) -> i32 {
        let Some(text) = self.text(tree) else {
            return 0;
        };
        let content = self.rows.len() as i32 * line_height(tree, text);
        let view = tree.get(text).map_or(0, |e| e.rect().height());
        (content - view).max(0)
    }

    /// The scroll applied to the text, the slider and the book's bottom bar (shown only when the
    /// list overflows).
    fn sync_scroll(&mut self, tree: &mut UiTree) {
        let Some(root) = self.window() else {
            return;
        };
        let max = self.max_scroll(tree);
        self.scroll = self.scroll.clamp(0, max);
        if let Some(text) = self.text(tree) {
            tree.set_text_view_y(text, self.scroll);
        }
        if let Some(bottom) = tree.child_recursive(root, tokenize("BookBottom")) {
            tree.set_visible(bottom, max > 0);
        }
        if let Some(slider) = tree.child_recursive(root, tokenize("BookSliderBar")) {
            let position = if max > 0 {
                self.scroll as f32 / max as f32
            } else {
                0.0
            };
            tree.scrollbar_set_position(slider, position);
        }
    }
}

/// The round eyeball the radar's Tracker button shows: the eye icon's inside (without its square
/// frame) drawn in a circle the size of the button's eye element, under [`EYE_ORB`], and the
/// element pointed at it. Runs once the HUD's images are loaded (the icon among them).
pub fn bake_eye(tree: &mut UiTree, images: &mut std::collections::HashMap<u32, UiPixels>) {
    let Some(icon) = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, EYE_ART_ID))
    else {
        return;
    };
    let Some(source) = images.get(&EYE_ICON) else {
        return;
    };
    let (w, h) = tree
        .get(icon)
        .map_or((0, 0), |e| (e.rect().width(), e.rect().height()));
    if w <= 0 || h <= 0 || source.width == 0 || source.height == 0 {
        return;
    }
    let (sw, sh) = (source.width as f32, source.height as f32);
    let inner = (sw - 2.0 * EYE_ICON_BORDER, sh - 2.0 * EYE_ICON_BORDER);
    let radius = w.min(h) as f32 / 2.0;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = (
                x as f32 + 0.5 - w as f32 / 2.0,
                y as f32 + 0.5 - h as f32 / 2.0,
            );
            // A soft edge one pixel wide.
            let cover = (radius - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0);
            if cover <= 0.0 {
                continue;
            }
            let sx = (EYE_ICON_BORDER + (x as f32 + 0.5) / w as f32 * inner.0) as u32;
            let sy = (EYE_ICON_BORDER + (y as f32 + 0.5) / h as f32 * inner.1) as u32;
            let s = ((sy.min(source.height - 1) * source.width + sx.min(source.width - 1)) * 4)
                as usize;
            let o = ((y * w + x) * 4) as usize;
            rgba[o..o + 3].copy_from_slice(&source.rgba[s..s + 3]);
            rgba[o + 3] = (f32::from(source.rgba[s + 3]) * cover) as u8;
        }
    }
    images.insert(
        EYE_ORB,
        UiPixels {
            width: w as u32,
            height: h as u32,
            rgba: rgba.into(),
        },
    );
    tree.set_image(icon, EYE_ORB, 2);
    bake_socket(tree, images);
}

/// The coordinates button's socket cut from the radar's top art, mirrored, for the Tracker
/// button's own socket element.
fn bake_socket(tree: &mut UiTree, images: &mut std::collections::HashMap<u32, UiPixels>) {
    let Some(socket) = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize(EYE_SOCKET_NAME)))
    else {
        return;
    };
    let Some(art) = tree.get(socket).and_then(|e| e.image()).map(|i| i.file) else {
        return;
    };
    let Some(source) = images.get(&art) else {
        return;
    };
    let (x0, y0, w, h) = SOCKET;
    if source.width < (x0 + w) as u32 || source.height < (y0 + h) as u32 {
        return;
    }
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let s = (((y0 + y) as u32 * source.width + (x0 + w - 1 - x) as u32) * 4) as usize;
            let o = ((y * w + x) * 4) as usize;
            rgba[o..o + 4].copy_from_slice(&source.rgba[s..s + 4]);
        }
    }
    images.insert(
        EYE_BRACKET,
        UiPixels {
            width: w as u32,
            height: h as u32,
            rgba: rgba.into(),
        },
    );
    tree.set_image(socket, EYE_BRACKET, 2);
}

#[cfg(test)]
#[path = "tracker_ui_tests.rs"]
mod tests;
