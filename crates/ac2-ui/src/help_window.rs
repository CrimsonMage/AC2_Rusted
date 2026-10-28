//! The help window: the `KeyStone::OpenHelp` page drawn in the book's own frame.
//!
//! Retail showed help in `AC2HelpPlugin.dll`'s window. Here the window is a second `Book` of the
//! gameplay layout, made as its own HUD root ([`BookUi::create_window`]): the book's frame, title,
//! text area, scroll slider and close button. [`HelpView`] lays the page out (wrapped text, tabs,
//! frames), and this turns the laid-out blocks into the text area's lines. A frame's page is
//! inlined where the frame stands.
//!
//! The text element takes one font and colour, so bold, italic and underline are not drawn: a
//! styled run is drawn as plain text. The DAT has no face to use instead: the UI's font group
//! (`0x4000000B`) holds two Latin faces at 12 and 16 pixels, a third 12-pixel face for more glyphs,
//! and the two glyph fallbacks (`UIElement_Text::DetermineFont`), and an `Ac2Font` has no style
//! flag. Retail's text renderer draws no underline either, so none is drawn. Tabs are drawn as `(*) name` (checked) or `( ) name`, one
//! row of them. A press on a line follows the link or tab under the pointer.
//!
//! The text area scrolls by the slider (`BookSliderBar`, its `new_position` message), which is
//! also what the wheel reaches over the bar.

use crate::book::BookUi;
use crate::element::UiTree;
use crate::help::HelpPages;
use crate::help_view::{
    layout_blocks, HelpBlock, HelpLayout, HelpLink, HelpMetrics, HelpRequest, HelpView,
};
use crate::ksml::{KsmlDocument, KsmlFrame, KsmlNode};
use crate::layout::LayoutDesc;
use crate::mouse::{UiMessage, GLOBAL_MOUSE_DOWN, UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::scrollbar::MSG_NEW_POSITION;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP, TEXT_LEFT_MARGIN};
use std::sync::Arc;

/// The gap between two tabs in a row, in spaces.
const TAB_GAP: &str = "    ";
/// A frame inside a frame is inlined this many levels deep, then left as a blank block.
const MAX_FRAME_DEPTH: u32 = 3;

/// One piece of a text row: a run of text and the link it follows, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Piece {
    text: String,
    link: Option<HelpLink>,
}

/// One line of the text area. A tab row's pieces are its tabs, drawn with [`TAB_GAP`] between.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Row {
    pieces: Vec<Piece>,
    tabs: bool,
}

/// The help window's state: the book's element, the view over the page, and the lines shown.
#[derive(Debug, Clone)]
pub struct HelpWindow {
    book: BookUi,
    view: HelpView,
    rows: Vec<Row>,
}

impl HelpWindow {
    /// The window, made hidden as a HUD root over the gameplay layout's `Book`. `None` when the
    /// layout has no `Book`.
    pub fn create(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>) -> Option<Self> {
        let book = BookUi::create_window(tree, gameplay)?;
        // The page's image is the book's, and the window has none: its foreground is hidden, so a
        // press on the text reaches it.
        if let Some(foreground) = child(tree, &book, "BookForeground") {
            tree.set_visible(foreground, false);
        }
        Some(Self {
            book,
            view: HelpView::new(),
            rows: Vec::new(),
        })
    }

    /// Whether the window is shown.
    pub fn is_shown(&self) -> bool {
        self.book.is_shown()
    }

    /// The window's element named `name` (a child of its `Book`), for a message aimed at it.
    pub fn element(&self, tree: &UiTree, name: &str) -> Option<usize> {
        child(tree, &self.book, name)
    }

    /// The page the window shows (its `open` name), once one is open.
    pub fn page(&self) -> Option<&str> {
        self.view.page()
    }

    /// The number of text lines the window lays its page out as.
    pub fn line_count(&self) -> usize {
        self.rows.len()
    }

    /// The text the window's text area shows, one line per row.
    pub fn text(&self) -> String {
        self.rows
            .iter()
            .map(row_text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// `KeyStone::OpenHelp(token)`: shows the topic's page (or the default page for token 0).
    /// Without pages, or for a token the table does not name, the window says so.
    pub fn open(&mut self, tree: &mut UiTree, pages: Option<&HelpPages>, token: u32) {
        self.book.show(tree, true);
        self.book.reset(tree);
        let Some(pages) = pages else {
            return self.show_message(tree, "Help is not installed: no help pages were found.");
        };
        let Some(name) = pages.page_for_token(token).map(str::to_owned) else {
            return self.show_message(tree, "No help page for this topic.");
        };
        let HelpRequest::Load { ticket, page, .. } = self.view.open(&name) else {
            return;
        };
        let Some(document) = pages.page(&page).cloned() else {
            return self.show_message(tree, &format!("The help page {page} is missing."));
        };
        self.view.loaded(ticket, None, document);
        self.render(tree, pages, true);
    }

    /// The window's messages: the close button hides it, the slider scrolls the text, and a press
    /// on the text follows the link or tab under the pointer. Returns whether the message was the
    /// window's.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        pages: Option<&HelpPages>,
        message: &UiMessage,
    ) -> bool {
        if !self.is_shown() {
            return false;
        }
        if self.wheel_message(tree, message) {
            return true;
        }
        if !self.owns(tree, message.element) {
            return false;
        }
        if message.element_id == tokenize("BookCloseButton")
            && message.message == MSG_BUTTON_RELEASE_ON
        {
            self.close(tree);
            return true;
        }
        if message.element_id == tokenize("BookSliderBar") && message.message == MSG_NEW_POSITION {
            if let Some(slider) = child(tree, &self.book, "BookSliderBar") {
                let position = tree.scrollbar_position(slider);
                let max = self.max_scroll(tree);
                self.view.scroll_to(position_to_pixels(position, max));
                self.sync_scroll(tree);
            }
            return true;
        }
        if message.element_id == tokenize("BookClientAreaText")
            && message.message == MSG_MOUSE_PRESS_TOP
        {
            if let (Some(link), Some(pages)) = (self.link_at(tree, tree.pointer()), pages) {
                if let Some(HelpRequest::Load {
                    ticket,
                    page,
                    target,
                }) = self.view.press_link(&link)
                {
                    match target {
                        None => {
                            if let Some(document) = pages.page(&page).cloned() {
                                self.view.loaded(ticket, None, document);
                                self.render(tree, pages, true);
                            }
                        }
                        Some(name) => {
                            if let Some(document) = pages.page(&page).cloned() {
                                self.view.loaded(ticket, Some(&name), document);
                                self.render(tree, pages, false);
                            }
                        }
                    }
                }
            }
            return true;
        }
        false
    }

    /// A wheel turn over the text scrolls the page a notch (`HelpView::wheel`). The text element
    /// does not scroll itself and the slider is not bound to it, so the turn is read here: the
    /// tree broadcasts a wheel turn as a global mouse-down carrying its action. Returns whether the
    /// turn was over the text.
    fn wheel_message(&mut self, tree: &mut UiTree, message: &UiMessage) -> bool {
        let action = message.data as u32;
        if message.element.is_some()
            || message.message != GLOBAL_MOUSE_DOWN
            || !matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT)
        {
            return false;
        }
        let Some(text) = child(tree, &self.book, "BookClientAreaText") else {
            return false;
        };
        let rect = tree.screen_box(text);
        let (x, y) = tree.pointer();
        if x < rect.x0 || x >= rect.x1 || y < rect.y0 || y >= rect.y1 {
            return false;
        }
        // Wheel out turns the text down: a notch back up is positive.
        let notches = if action == UI_WHEEL_IN { 1 } else { -1 };
        let line = line_height(tree, text);
        let content = self.rows.len() as i32 * line;
        let view = rect.height();
        self.view.wheel(notches, line, content, view);
        self.sync_scroll(tree);
        true
    }

    /// `BookCloseButton`: the window hides and forgets its page.
    pub fn close(&mut self, tree: &mut UiTree) {
        self.book.show(tree, false);
        self.view = HelpView::new();
        self.rows.clear();
    }

    /// Whether `element` is this window's or one of its children.
    fn owns(&self, tree: &UiTree, element: Option<usize>) -> bool {
        let mut current = element;
        while let Some(index) = current {
            if index == self.book.field() {
                return true;
            }
            current = tree.get(index).and_then(|e| e.parent_index());
        }
        false
    }

    /// The window shows one line of `text` and no page.
    fn show_message(&mut self, tree: &mut UiTree, text: &str) {
        self.rows = vec![Row {
            pieces: vec![Piece {
                text: text.to_owned(),
                link: None,
            }],
            tabs: false,
        }];
        if let Some(element) = child(tree, &self.book, "BookClientAreaText") {
            tree.set_text(element, text);
        }
        self.sync_scroll(tree);
    }

    /// Lays the view's page out into the text area. `expand_frames` loads the frames the page
    /// embeds (on a new page); a link into a frame keeps the frames it already shows.
    fn render(&mut self, tree: &mut UiTree, pages: &HelpPages, expand_frames: bool) {
        let Some(document) = self.view.document().cloned() else {
            return;
        };
        if expand_frames {
            if let Some(title) = meta(&document, "title") {
                if let Some(element) = child(tree, &self.book, "Panel_Title_Book") {
                    tree.set_text(element, title);
                }
            }
        }
        let nodes = self.inline(pages, &document.nodes, 0, expand_frames);
        let Some(text) = child(tree, &self.book, "BookClientAreaText") else {
            return;
        };
        let line = line_height(tree, text);
        let width = tree.get(text).map_or(0, |e| e.rect().width()) - margins(tree, text);
        let measure = |s: &str, _: crate::ksml::KsmlStyle| text_width(tree, text, s);
        let metrics = HelpMetrics {
            width: &measure,
            line_height: line,
        };
        let layout = layout_blocks(&nodes, width.max(1), &metrics);
        self.rows = rows_of(&layout, line);
        let lines: Vec<String> = self.rows.iter().map(row_text).collect();
        tree.set_text(text, &lines.join("\n"));
        self.view.scroll_to(0);
        self.sync_scroll(tree);
    }

    /// The page's nodes with each frame that has a loaded page replaced by that page's nodes.
    fn inline(
        &mut self,
        pages: &HelpPages,
        nodes: &[KsmlNode],
        depth: u32,
        load: bool,
    ) -> Vec<KsmlNode> {
        let mut out = Vec::with_capacity(nodes.len());
        for node in nodes {
            match node {
                KsmlNode::Frame(frame) => {
                    if let Some(inlined) = self.inline_frame(pages, frame, depth, load) {
                        out.extend(inlined);
                    } else {
                        out.push(node.clone());
                    }
                }
                KsmlNode::Link(link) => {
                    let mut link = link.clone();
                    link.children = self.inline(pages, &link.children, depth, load);
                    out.push(KsmlNode::Link(link));
                }
                KsmlNode::Control(control) => {
                    let mut control = control.clone();
                    control.children = self.inline(pages, &control.children, depth, load);
                    out.push(KsmlNode::Control(control));
                }
                other => out.push(other.clone()),
            }
        }
        out
    }

    /// A frame's nodes, when its page is (or now becomes) loaded. `None` leaves the frame as a
    /// block of its height.
    fn inline_frame(
        &mut self,
        pages: &HelpPages,
        frame: &KsmlFrame,
        depth: u32,
        load: bool,
    ) -> Option<Vec<KsmlNode>> {
        if depth >= MAX_FRAME_DEPTH {
            return None;
        }
        let name = frame.name.clone()?;
        if load {
            for request in self.view.embed_frames(std::slice::from_ref(frame)) {
                if let HelpRequest::Load {
                    ticket,
                    page,
                    target,
                } = request
                {
                    if let Some(document) = pages.page(&page).cloned() {
                        self.view.loaded(ticket, target.as_deref(), document);
                    }
                }
            }
        }
        let document = self
            .view
            .frames()
            .iter()
            .find(|f| f.name == name)
            .and_then(|f| f.document.clone())?;
        Some(self.inline(pages, &document.nodes, depth + 1, load))
    }

    /// The largest scroll position in pixels: the text past the window's height.
    fn max_scroll(&self, tree: &UiTree) -> i32 {
        let Some(text) = child(tree, &self.book, "BookClientAreaText") else {
            return 0;
        };
        let content = self.rows.len() as i32 * line_height(tree, text);
        let view = tree.get(text).map_or(0, |e| e.rect().height());
        (content - view).max(0)
    }

    /// Applies the view's scroll to the text area, the slider and the controls: the slider and
    /// its buttons show only when the text overflows.
    fn sync_scroll(&mut self, tree: &mut UiTree) {
        let Some(text) = child(tree, &self.book, "BookClientAreaText") else {
            return;
        };
        let max = self.max_scroll(tree);
        let scroll = self.view.scroll().clamp(0, max);
        tree.set_text_view_y(text, scroll);
        if let Some(controls) = child(tree, &self.book, "BookBottom") {
            tree.set_visible(controls, max > 0);
        }
        if let Some(slider) = child(tree, &self.book, "BookSliderBar") {
            let position = if max > 0 {
                scroll as f32 / max as f32
            } else {
                0.0
            };
            tree.scrollbar_set_position(slider, position);
        }
    }

    /// The link under the pointer: the row at the pointer's height, the piece at its x.
    fn link_at(&self, tree: &UiTree, pointer: (i32, i32)) -> Option<HelpLink> {
        let text = child(tree, &self.book, "BookClientAreaText")?;
        let rect = tree.screen_box(text);
        let (x, y) = pointer;
        if x < rect.x0 || x >= rect.x1 || y < rect.y0 || y >= rect.y1 {
            return None;
        }
        let line = line_height(tree, text);
        let row = ((y - rect.y0 + self.view.scroll()) / line.max(1)) as usize;
        let row = self.rows.get(row)?;
        let mut left = x - rect.x0 - margin_left(tree, text);
        let gap = text_width(tree, text, TAB_GAP);
        for piece in &row.pieces {
            let width = text_width(tree, text, &piece.text);
            if left < width {
                return piece.link.clone();
            }
            left -= width + if row.tabs { gap } else { 0 };
        }
        None
    }
}

#[cfg(test)]
impl HelpWindow {
    /// A point on the first link (or, with `tabs`, the first unchecked tab) of the text that is in
    /// view, on screen, and the link there: where a press lands.
    pub(crate) fn link_point(&self, tree: &UiTree, tabs: bool) -> Option<((i32, i32), HelpLink)> {
        let text = child(tree, &self.book, "BookClientAreaText")?;
        let rect = tree.screen_box(text);
        let line = line_height(tree, text);
        let gap = text_width(tree, text, TAB_GAP);
        for (index, row) in self.rows.iter().enumerate() {
            if row.tabs != tabs {
                continue;
            }
            let mut left = margin_left(tree, text);
            for piece in &row.pieces {
                let width = text_width(tree, text, &piece.text);
                let y = rect.y0 + index as i32 * line + line / 2 - self.view.scroll();
                let shown = y >= rect.y0 && y < rect.y1;
                // A checked tab (`(*)`) is already the one shown; pressing it changes nothing.
                let checked = tabs && piece.text.starts_with("(*)");
                if let (Some(link), true, false) = (&piece.link, shown, checked) {
                    let x = rect.x0 + left + width / 2;
                    return Some(((x, y), link.clone()));
                }
                left += width + if row.tabs { gap } else { 0 };
            }
        }
        None
    }
}

/// The child of the window's `Book` named `name`.
fn child(tree: &UiTree, book: &BookUi, name: &str) -> Option<usize> {
    tree.child_recursive(book.field(), tokenize(name))
}

/// A `meta` entry of the page, when it is not empty.
fn meta<'a>(document: &'a KsmlDocument, key: &str) -> Option<&'a str> {
    document
        .meta
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v.as_str())
        .filter(|v| !v.is_empty())
}

/// The text element's left margin, in pixels.
fn margin_left(tree: &UiTree, text: usize) -> i32 {
    match tree.get(text).and_then(|e| e.attribute(TEXT_LEFT_MARGIN)) {
        Some(crate::layout::AttributeValue::Int(margin)) => *margin,
        _ => 0,
    }
}

/// The text element's horizontal margins together (what its text box loses).
fn margins(tree: &UiTree, text: usize) -> i32 {
    tree.text_size_with_margins(text, "").map_or(0, |(w, _)| w)
}

/// The width of `s` in the text element's font, without its margins.
fn text_width(tree: &UiTree, text: usize, s: &str) -> i32 {
    let with = tree.text_size_with_margins(text, s).map_or(0, |(w, _)| w);
    with - margins(tree, text)
}

/// The text element's line height in pixels, without its margins.
fn line_height(tree: &UiTree, text: usize) -> i32 {
    let with = tree
        .text_size_with_margins(text, "Ag")
        .map_or(0, |(_, h)| h);
    let margin = tree.text_size_with_margins(text, "").map_or(0, |(_, h)| h);
    (with - margin).max(1)
}

/// The window's rows from the laid-out blocks: a text line is a row, a tab row is a row of its
/// tabs, and a frame without a page is as many blank rows as its height takes.
fn rows_of(layout: &HelpLayout, line: i32) -> Vec<Row> {
    let mut rows = Vec::new();
    for (_, height, block) in &layout.blocks {
        match block {
            HelpBlock::Line(segments) => rows.push(Row {
                pieces: segments
                    .iter()
                    .map(|s| Piece {
                        text: s.text.clone(),
                        link: s.link.clone(),
                    })
                    .collect(),
                tabs: false,
            }),
            HelpBlock::Tabs(tabs) => rows.push(Row {
                pieces: tabs
                    .iter()
                    .map(|tab| Piece {
                        text: format!("({}) {}", if tab.checked { "*" } else { " " }, tab.text),
                        link: Some(HelpLink {
                            href: tab.href.clone(),
                            target: tab.target.clone(),
                        }),
                    })
                    .collect(),
                tabs: true,
            }),
            HelpBlock::Frame(_) => {
                for _ in 0..(*height / line.max(1)).max(1) {
                    rows.push(Row::default());
                }
            }
        }
    }
    rows
}

/// A row's text as the text element draws it.
fn row_text(row: &Row) -> String {
    let parts: Vec<&str> = row.pieces.iter().map(|p| p.text.as_str()).collect();
    parts.join(if row.tabs { TAB_GAP } else { "" })
}

/// The scroll position for a slider position in `[0, 1]` over `max` pixels.
fn position_to_pixels(position: f32, max: i32) -> i32 {
    (position.clamp(0.0, 1.0) * max as f32).round() as i32
}
