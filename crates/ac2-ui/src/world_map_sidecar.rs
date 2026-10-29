//! The world map's layers list: a sidecar window docked beside the navigation panel (right of it,
//! or left when the right would leave the screen), cut from the DAT layouts as the Tracker window
//! is: another `Book` of the gameplay layout, small, its frame and title bar in nine pieces. Each
//! row is a layer with its icon (the note's own, or the guide's marker) and a check (`[x]` / `[ ]`);
//! a press on a row turns the layer on or off.

use std::collections::HashMap;
use std::sync::Arc;

use crate::book::BookUi;
use crate::book_rows::{line_height, margin_left};
use crate::compose::NINE_SLICE;
use crate::element::UiTree;
use crate::layout::{ElementDesc, LayoutDesc};
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP, SIZE_IMAGE_TO_OWNER};
use crate::world_map_layers::{GuideLayer, NoteKind, WorldNote};

/// One map layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Note(NoteKind),
    Guide(GuideLayer),
    ZoneLevels,
    Classic,
}

impl Layer {
    fn label(self) -> &'static str {
        match self {
            Layer::Note(kind) => kind.label(),
            Layer::Guide(layer) => layer.label(),
            Layer::ZoneLevels => "Zone levels",
            Layer::Classic => "Painted map",
        }
    }
}

/// A row of the list: a layer, or a heading.
#[derive(Debug, Clone, PartialEq)]
enum Row {
    Heading(&'static str),
    Layer(Layer),
}

/// The sidecar's size and the frame's corners.
const SIZE: (i32, i32) = (200, 370);
const FRAME_BORDER: i32 = 24;
/// The row icons' size and their gap from the window's left.
const ICON: i32 = 11;

#[derive(Debug, Clone)]
pub struct Sidecar {
    book: BookUi,
    rows: Vec<Row>,
    on: HashMap<Layer, bool>,
    /// The row icons, one per layer row with an icon.
    icons: Vec<usize>,
}

impl Sidecar {
    /// The hidden list, with a row for each note kind the region has, the guide's layers, the
    /// zone levels and the painted map.
    pub fn create(
        tree: &mut UiTree,
        gameplay: &Arc<LayoutDesc>,
        notes: &[WorldNote],
    ) -> Option<Self> {
        let book = BookUi::create_window(tree, gameplay, "MapLayersBook")?;
        let root = book.field();
        let find = |tree: &UiTree, name: &str| tree.child_recursive(root, tokenize(name));
        tree.resize_to(root, SIZE.0, SIZE.1);
        for child in tree.children(root).to_vec() {
            if tree.get(child).is_some_and(|e| e.image().is_some()) {
                let height = tree.get(child).map_or(0, |e| e.rect().height());
                let height = if height > 100 { SIZE.1 } else { height };
                tree.resize_to(child, SIZE.0, height);
                tree.set_int(child, NINE_SLICE, FRAME_BORDER);
            }
        }
        for name in [
            "BookForeground",
            "BookClientAreaTextHeader",
            "BookHelpButton",
            "BookCopyButton",
            "BookBottom",
        ] {
            if let Some(e) = find(tree, name) {
                tree.set_visible(e, false);
            }
        }
        if let Some(close) = find(tree, "BookCloseButton") {
            tree.move_to(close, SIZE.0 - 30, 7);
        }
        if let Some(title) = find(tree, "Panel_Title_Book") {
            tree.resize_to(title, SIZE.0, 29);
            tree.set_text(title, "Layers");
        }
        let mut rows = vec![Row::Heading("Map")];
        let mut on = HashMap::new();
        for kind in NoteKind::ALL {
            if notes.iter().any(|n| n.kind == kind) {
                rows.push(Row::Layer(Layer::Note(kind)));
                on.insert(Layer::Note(kind), true);
            }
        }
        rows.push(Row::Heading("AC2Hero guide"));
        for layer in GuideLayer::ALL {
            rows.push(Row::Layer(Layer::Guide(layer)));
            on.insert(Layer::Guide(layer), false);
        }
        rows.push(Row::Layer(Layer::ZoneLevels));
        on.insert(Layer::ZoneLevels, false);
        rows.push(Row::Heading("Background"));
        rows.push(Row::Layer(Layer::Classic));
        on.insert(Layer::Classic, false);
        let mut sidecar = Self {
            book,
            rows,
            on,
            icons: Vec::new(),
        };
        if let Some(text) = sidecar.text(tree) {
            tree.move_to(text, 14, 36);
            tree.resize_to(text, SIZE.0 - 24, SIZE.1 - 48);
        }
        sidecar.render(tree);
        sidecar.make_icons(tree, gameplay, notes);
        Some(sidecar)
    }

    fn text(&self, tree: &UiTree) -> Option<usize> {
        tree.child_recursive(self.book.field(), tokenize("BookClientAreaText"))
    }

    pub fn is_shown(&self) -> bool {
        self.book.is_shown()
    }

    pub fn show(&mut self, tree: &mut UiTree, shown: bool) {
        self.book.show(tree, shown);
    }

    /// Whether `layer` is on.
    pub fn on(&self, layer: Layer) -> bool {
        self.on.get(&layer).copied().unwrap_or(false)
    }

    /// Whether a guide layer or the zone levels are on (the guide is decoded only then).
    pub fn wants_guide(&self) -> bool {
        self.on
            .iter()
            .any(|(layer, on)| *on && matches!(layer, Layer::Guide(_) | Layer::ZoneLevels))
    }

    /// The layers' states as bits, so the map is drawn again when one changes.
    pub fn bits(&self) -> u64 {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| matches!(row, Row::Layer(l) if self.on(*l)))
            .fold(0u64, |bits, (i, _)| bits | (1 << (i % 64)))
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
    }

    /// The rows as the text area shows them.
    fn render(&self, tree: &mut UiTree) {
        let lines: Vec<String> = self
            .rows
            .iter()
            .map(|row| match row {
                Row::Heading(text) => (*text).to_owned(),
                Row::Layer(layer) => {
                    let check = if self.on(*layer) { "[x]" } else { "[ ]" };
                    format!("  {check}     {}", layer.label())
                }
            })
            .collect();
        if let Some(text) = self.text(tree) {
            tree.set_text(text, &lines.join("\n"));
        }
    }

    /// Each layer row's icon after its check: the note kind's own icon, the guide's marker.
    fn make_icons(&mut self, tree: &mut UiTree, gameplay: &Arc<LayoutDesc>, notes: &[WorldNote]) {
        let (Some(text), Some(template)) = (
            self.text(tree),
            gameplay.find_element(tokenize("CoordBoxButton")),
        ) else {
            return;
        };
        let line = line_height(tree, text);
        let x = 14 + margin_left(tree, text) + crate::book_rows::text_width(tree, text, "  [x] ");
        for (i, row) in self.rows.iter().enumerate() {
            let icon = match row {
                Row::Layer(Layer::Note(kind)) => {
                    notes.iter().find(|n| n.kind == *kind).map(|n| n.icon)
                }
                Row::Layer(Layer::Guide(layer)) => Some(layer.icon()),
                _ => None,
            };
            let Some(icon) = icon else { continue };
            let mut desc: ElementDesc = (*template).clone();
            desc.id = tokenize("WorldMapLayerArt");
            desc.states.clear();
            let Some(element) = tree.create_element(Arc::clone(gameplay), Arc::new(desc), false)
            else {
                continue;
            };
            tree.set_parent(element, Some(self.book.field()));
            tree.move_to(element, x, 36 + i as i32 * line + (line - ICON) / 2);
            tree.resize_to(element, ICON, ICON);
            tree.set_image(element, icon, 2);
            tree.set_bool(element, SIZE_IMAGE_TO_OWNER, true);
            self.icons.push(element);
        }
    }

    /// The close button hides the list; a press on a layer's row turns it on or off. Returns
    /// whether the message was the list's.
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage) -> bool {
        if !self.is_shown() {
            return false;
        }
        let root = self.book.field();
        let mut current = message.element;
        let mut ours = false;
        while let Some(index) = current {
            if index == root {
                ours = true;
                break;
            }
            current = tree.get(index).and_then(|e| e.parent_index());
        }
        if !ours {
            return false;
        }
        if message.element_id == tokenize("BookCloseButton")
            && message.message == MSG_BUTTON_RELEASE_ON
        {
            self.show(tree, false);
            return true;
        }
        if message.message == MSG_MOUSE_PRESS_TOP {
            let Some(text) = self.text(tree) else {
                return true;
            };
            let rect = tree.screen_box(text);
            let (_, y) = tree.pointer();
            let row = ((y - rect.y0) / line_height(tree, text)) as usize;
            if let Some(Row::Layer(layer)) = self.rows.get(row) {
                let on = !self.on(*layer);
                self.on.insert(*layer, on);
                self.render(tree);
            }
            return true;
        }
        true
    }

    /// Keeps the list beside the navigation panel `field`: right of it, or left when the right
    /// would leave the screen.
    pub fn dock(&self, tree: &mut UiTree, field: usize) {
        if !self.is_shown() {
            return;
        }
        let panel = tree.screen_box(field);
        let (display_w, _) = tree.display;
        let x = if panel.x1 + 1 + SIZE.0 <= display_w {
            panel.x1 + 1
        } else {
            panel.x0 - SIZE.0
        };
        let y = panel.y0;
        let root = self.book.field();
        if tree.screen_box(root).x0 != x || tree.screen_box(root).y0 != y {
            tree.move_to(root, x, y);
        }
    }

    /// The list's window root.
    pub fn root(&self) -> usize {
        self.book.field()
    }

    /// The row of `layer` (for tests).
    pub fn row_of(&self, layer: Layer) -> Option<usize> {
        self.rows.iter().position(|r| *r == Row::Layer(layer))
    }
}
