//! Parts the client's own windows (the Tracker, the guide) are cut from the DAT layouts with: a
//! plain image, a text button, a single-line entry field in its box, a copy of a layout part, the
//! layouts' vertical scrollbar driven by a window's own scroll, a grip mark, and the ownership and
//! hit tests their message handlers use.

use std::sync::Arc;

use crate::book_rows::{line_height, margin_top};
use crate::compose::{UiPixels, NINE_SLICE};
use crate::element::UiTree;
use crate::layout::{ElementDesc, LayoutDesc};
use crate::token::tokenize;

/// The box drawn behind an entry field: the book's bottom bar (its stone frame and dark wood
/// inset), drawn in nine pieces with corners this size.
pub(crate) const FIELD_BOX: u32 = 0x4100_7F71;
const FIELD_BORDER: i32 = 8;
/// The text buttons' art keeps corners this size when stretched.
const BUTTON_BORDER: i32 = 8;
/// The single-line entry fields an entry field may copy, by preference.
const ENTRY_TEMPLATES: [&str; 3] = [
    "AbuserNameEntryText",
    "NoFellowshipNameEntryText",
    "ChatEntry",
];

/// A plain image element under `parent` at `rect` (x, y, width, height), drawn in nine pieces with
/// corners `border` pixels square: a copy of the radar's coordinates button without its states.
pub(crate) fn plain_image(
    tree: &mut UiTree,
    gameplay: &Arc<LayoutDesc>,
    parent: usize,
    id: &str,
    image: u32,
    rect: (i32, i32, i32, i32),
    border: i32,
) -> Option<usize> {
    let template = gameplay.find_element(tokenize("CoordBoxButton"))?;
    let mut desc: ElementDesc = (*template).clone();
    desc.id = tokenize(id);
    desc.states.clear();
    let element = tree.create_element(Arc::clone(gameplay), Arc::new(desc), false)?;
    tree.set_parent(element, Some(parent));
    tree.move_to(element, rect.0, rect.1);
    tree.resize_to(element, rect.2, rect.3);
    tree.set_image(element, image, 2);
    tree.set_int(element, NINE_SLICE, border);
    Some(element)
}

/// A text button under `parent`: a copy of the book's own text button (`BookCopyButton`) named
/// `id`, at `(x, y)` and `width` wide, its art in nine pieces and its label `label`.
pub(crate) fn text_button(
    tree: &mut UiTree,
    gameplay: &Arc<LayoutDesc>,
    parent: usize,
    id: &str,
    label: &str,
    (x, y, width): (i32, i32, i32),
) -> Option<usize> {
    let template = gameplay.find_element(tokenize("BookCopyButton"))?;
    let mut desc: ElementDesc = (*template).clone();
    desc.id = tokenize(id);
    let element = tree.create_element(Arc::clone(gameplay), Arc::new(desc), false)?;
    tree.set_parent(element, Some(parent));
    tree.move_to(element, x, y);
    tree.resize_to(element, width, 20);
    tree.set_int(element, NINE_SLICE, BUTTON_BORDER);
    set_button_label(tree, element, label);
    Some(element)
}

/// The button's label (its text child) set to `label`, sized to the button.
pub(crate) fn set_button_label(tree: &mut UiTree, button: usize, label: &str) {
    let width = tree.get(button).map_or(0, |e| e.rect().width());
    let text = tree
        .children(button)
        .iter()
        .copied()
        .find(|&c| tree.get(c).is_some_and(|e| e.text().is_some()));
    if let Some(text) = text {
        tree.resize_to(text, width, 19);
        tree.set_text(text, label);
    }
}

/// A single-line entry field under `parent` in its box: the field a copy of the first of the
/// layouts' entry fields found (named `id`), the box (`box_id`) behind it. `rect` is the box's;
/// the field sits inside its stone end caps. Returns the field and the box.
pub(crate) fn entry_field(
    tree: &mut UiTree,
    gameplay: &Arc<LayoutDesc>,
    layouts: &[&Arc<LayoutDesc>],
    parent: usize,
    (id, box_id): (&str, &str),
    rect: (i32, i32, i32, i32),
) -> Option<(usize, Option<usize>)> {
    let (layout, template) = ENTRY_TEMPLATES.iter().find_map(|name| {
        layouts
            .iter()
            .find_map(|layout| Some((Arc::clone(layout), layout.find_element(tokenize(name))?)))
    })?;
    let field_box = plain_image(
        tree,
        gameplay,
        parent,
        box_id,
        FIELD_BOX,
        rect,
        FIELD_BORDER,
    );
    let mut desc: ElementDesc = (*template).clone();
    desc.id = tokenize(id);
    let field = tree.create_element(layout, Arc::new(desc), false)?;
    tree.set_parent(field, Some(parent));
    place_entry(tree, field, field_box, rect);
    // The box right behind the field (a higher z draws further back), in front of the frame.
    if let (Some(field_box), Some(z)) = (field_box, tree.get(field).map(|e| e.z)) {
        tree.set_z_level(field_box, z + 1);
    }
    Some((field, field_box))
}

/// The entry field and its box moved to the box's `rect`.
pub(crate) fn place_entry(
    tree: &mut UiTree,
    field: usize,
    field_box: Option<usize>,
    (x, y, w, h): (i32, i32, i32, i32),
) {
    if let Some(field_box) = field_box {
        tree.move_to(field_box, x, y);
        tree.resize_to(field_box, w, h);
    }
    tree.move_to(field, x + 12, y + 2);
    tree.resize_to(field, w - 24, h - 4);
}

/// Whether `element` is `owner` or inside it.
pub(crate) fn owned_by(tree: &UiTree, element: Option<usize>, owner: usize) -> bool {
    let mut current = element;
    while let Some(index) = current {
        if index == owner {
            return true;
        }
        current = tree.get(index).and_then(|e| e.parent_index());
    }
    false
}

/// The vertical scrollbar template the UI layouts share.
const VERTICAL_SCROLLBAR: u32 = 0x413D_E718;
/// The grip mark's size.
pub(crate) const GRIP_SIZE: i32 = 16;

/// A grip mark [`GRIP_SIZE`] pixels square: three diagonal ridges in the corner, gold over a dark shadow.
pub(crate) fn grip_art() -> UiPixels {
    let n = GRIP_SIZE as usize;
    let mut rgba = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            // Ridges where x + y is 13, 17 and 21 (the lower-right half), a shadow below each.
            let d = x + y;
            let colour: Option<[u8; 4]> = match d {
                13 | 17 | 21 if x >= 4 && y >= 4 => Some([214, 186, 112, 235]),
                14 | 18 | 22 if x >= 4 && y >= 4 => Some([20, 14, 8, 200]),
                _ => None,
            };
            if let Some(c) = colour {
                rgba[(y * n + x) * 4..][..4].copy_from_slice(&c);
            }
        }
    }
    UiPixels {
        width: GRIP_SIZE as u32,
        height: GRIP_SIZE as u32,
        rgba: rgba.into(),
    }
}

/// A text button moved to `(x, y)` and sized `width` x 20, its label with it.
pub(crate) fn place_button(tree: &mut UiTree, button: usize, (x, y, width): (i32, i32, i32)) {
    tree.move_to(button, x, y);
    tree.resize_to(button, width, 20);
    let label = tree
        .children(button)
        .iter()
        .copied()
        .find(|&c| tree.get(c).is_some_and(|e| e.text().is_some()));
    if let Some(label) = label {
        tree.move_to(label, 0, 0);
        tree.resize_to(label, width, 19);
    }
}

/// The UI layouts' vertical scrollbar (its up and down buttons, track and widget; a template
/// the layouts share) copied under `parent` as `id`.
pub(crate) fn vertical_scrollbar(
    tree: &mut UiTree,
    layouts: &[&Arc<LayoutDesc>],
    parent: usize,
    id: &str,
) -> Option<usize> {
    let (layout, template) = layouts
        .iter()
        .find_map(|layout| Some((Arc::clone(layout), layout.find_element(VERTICAL_SCROLLBAR)?)))?;
    let mut desc: ElementDesc = (*template).clone();
    desc.id = tokenize(id);
    let element = tree.create_element(layout, Arc::new(desc), false)?;
    tree.set_parent(element, Some(parent));
    Some(element)
}

/// A copy of `source` (a text area or a slider of the book) under `parent`, named `id`.
pub(crate) fn copy_of(
    tree: &mut UiTree,
    gameplay: &Arc<LayoutDesc>,
    parent: usize,
    source: usize,
    id: &str,
) -> Option<usize> {
    let desc = tree.get(source).map(|e| Arc::clone(&e.reference))?;
    let mut desc: ElementDesc = (*desc).clone();
    desc.id = tokenize(id);
    let element = tree.create_element(Arc::clone(gameplay), Arc::new(desc), false)?;
    tree.set_parent(element, Some(parent));
    Some(element)
}

pub(crate) fn inside(tree: &UiTree, element: usize, (x, y): (i32, i32)) -> bool {
    let r = tree.screen_box(element);
    x >= r.x0 && x < r.x1 && y >= r.y0 && y < r.y1
}

/// The largest scroll of a text of `rows` lines, in pixels.
pub(crate) fn max_scroll(tree: &UiTree, text: Option<usize>, rows: usize) -> i32 {
    let Some(text) = text else {
        return 0;
    };
    let content = rows as i32 * line_height(tree, text);
    let view = tree.get(text).map_or(0, |e| e.rect().height()) - margin_top(tree, text);
    (content - view).max(0)
}

/// `scroll` kept in bounds and applied to the text and its slider (hidden when nothing
/// overflows). Returns the kept scroll.
pub(crate) fn sync_scroll(
    tree: &mut UiTree,
    text: usize,
    slider: Option<usize>,
    rows: usize,
    scroll: i32,
) -> i32 {
    let max = max_scroll(tree, Some(text), rows);
    let scroll = scroll.clamp(0, max);
    tree.set_text_view_y(text, scroll);
    if let Some(slider) = slider {
        tree.set_visible(slider, max > 0);
        let position = if max > 0 {
            scroll as f32 / max as f32
        } else {
            0.0
        };
        let view = tree.get(text).map_or(0, |e| e.rect().height());
        let content = (rows as i32 * line_height(tree, text)).max(1);
        tree.scrollbar_set_widget_share(slider, (view as f32 / content as f32).min(1.0));
        tree.scrollbar_set_position(slider, position);
    }
    scroll
}
