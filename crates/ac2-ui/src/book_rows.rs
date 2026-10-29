//! Measures over a book-style text element that windows lay out as rows of text (the help window,
//! the Tracker): its left margin, a string's width and the line height in its font, and a slider
//! position as a scroll in pixels.

use crate::element::UiTree;
use crate::token::{TEXT_LEFT_MARGIN, TEXT_TOP_MARGIN};

/// The text element's left margin, in pixels.
pub(crate) fn margin_left(tree: &UiTree, text: usize) -> i32 {
    match tree.get(text).and_then(|e| e.attribute(TEXT_LEFT_MARGIN)) {
        Some(crate::layout::AttributeValue::Int(margin)) => *margin,
        _ => 0,
    }
}

/// The text element's top margin, in pixels.
pub(crate) fn margin_top(tree: &UiTree, text: usize) -> i32 {
    match tree.get(text).and_then(|e| e.attribute(TEXT_TOP_MARGIN)) {
        Some(crate::layout::AttributeValue::Int(margin)) => *margin,
        _ => 0,
    }
}

/// The text element's horizontal margins together (what its text box loses).
pub(crate) fn margins(tree: &UiTree, text: usize) -> i32 {
    tree.text_size_with_margins(text, "").map_or(0, |(w, _)| w)
}

/// The width of `s` in the text element's font, without its margins.
pub(crate) fn text_width(tree: &UiTree, text: usize, s: &str) -> i32 {
    let with = tree.text_size_with_margins(text, s).map_or(0, |(w, _)| w);
    with - margins(tree, text)
}

/// The text element's line height in pixels, without its margins.
pub(crate) fn line_height(tree: &UiTree, text: usize) -> i32 {
    let with = tree
        .text_size_with_margins(text, "Ag")
        .map_or(0, |(_, h)| h);
    let margin = tree.text_size_with_margins(text, "").map_or(0, |(_, h)| h);
    (with - margin).max(1)
}

/// The scroll position for a slider position in `[0, 1]` over `max` pixels.
pub(crate) fn position_to_pixels(position: f32, max: i32) -> i32 {
    (position.clamp(0.0, 1.0) * max as f32).round() as i32
}
