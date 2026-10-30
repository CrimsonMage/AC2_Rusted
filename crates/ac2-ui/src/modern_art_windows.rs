//! The modern windows' shared art (`modern_windows`, the design's "Other windows" set): the
//! window's header and glass, the help and close buttons, the tabs, the text buttons, the wells,
//! the check box, the chevrons and the rule, drawn by `modern_art`'s painter under the client's
//! own DIDs (`0x7E0100Fx`, `0x7E0101xx`). The colours are the design's tokens: glass `#002623`,
//! well `#00110f`, gold `#d9c27a` / `#a88f48`, cream `#ece3c4`, stone `#a4a192`, the line
//! `#6f6957`.

use std::collections::HashMap;

use crate::compose::UiPixels;
use crate::modern_art::{band, inside, mix, rgb, round_rect, Canvas};
use crate::modern_art_drawer::{stroke_distance, Stroke};

pub const WIN_GLASS: u32 = 0x7E01_00F0;
pub const WIN_CLOSE: u32 = 0x7E01_00F2;
pub const WIN_CLOSE_LIT: u32 = 0x7E01_00F3;
pub const WIN_HELP: u32 = 0x7E01_00F4;
pub const WIN_HELP_LIT: u32 = 0x7E01_00F5;
/// The tabs' pieces: left, middle, right; off then on.
pub const WIN_TAB: [u32; 6] = [
    0x7E01_00F6,
    0x7E01_00F7,
    0x7E01_00F8,
    0x7E01_00F9,
    0x7E01_00FA,
    0x7E01_00FB,
];
/// The text button: normal, lit, pressed, ghosted.
pub const WIN_BUTTON: [u32; 4] = [0x7E01_00FC, 0x7E01_00FD, 0x7E01_00FE, 0x7E01_00FF];
/// The well's nine pieces, row by row from the top left.
pub const WIN_WELL: [u32; 9] = [
    0x7E01_0100,
    0x7E01_0101,
    0x7E01_0102,
    0x7E01_0103,
    0x7E01_0104,
    0x7E01_0105,
    0x7E01_0106,
    0x7E01_0107,
    0x7E01_0108,
];
pub const WIN_CHECK: u32 = 0x7E01_0109;
pub const WIN_CHECK_ON: u32 = 0x7E01_010A;
pub const WIN_CHEVRON_RIGHT: u32 = 0x7E01_010B;
pub const WIN_CHEVRON_RIGHT_LIT: u32 = 0x7E01_010C;
pub const WIN_CHEVRON_DOWN: u32 = 0x7E01_010D;
pub const WIN_CHEVRON_DOWN_LIT: u32 = 0x7E01_010E;
pub const WIN_RULE: u32 = 0x7E01_010F;
/// The nine-slice borders of the button and the rule.
/// The window's header height (retail's frames' top) and its side and foot borders.
pub const WIN_HEADER_HEIGHT: u8 = 31;
pub const WIN_GLASS_BORDER: u8 = 8;
pub const WIN_BUTTON_BORDER: i32 = 9;
pub const WIN_RULE_BORDER: i32 = 12;

const LINE: u32 = 0x6F6957;
const GOLD: u32 = 0xD9C27A;
const GOLD2: u32 = 0xA88F48;
const CREAM: u32 = 0xECE3C4;

/// The window: its header (the top `WIN_HEADER_HEIGHT` rows, where retail's frames have
/// theirs) dark stone shaded down (`#38352b` to `#16150f`) under a light line, a black line under
/// it, then the glass (`#002623`); all in a black line and a dark stone one (`#545346`), corners
/// 6.
fn glass() -> UiPixels {
    let (w, h) = (32usize, 72usize); // The header within half its height (`draw_nine`).
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 6.0);
    let head = WIN_HEADER_HEIGHT as f32;
    c.paint(&shape, |_, y, d| {
        let colour = if y < head - 1.0 {
            mix(rgb(0x38352B, 1.0), rgb(0x16150F, 1.0), y / head)
        } else if y < head {
            rgb(0x000000, 1.0)
        } else {
            rgb(0x002623, 0.95)
        };
        (colour, inside(d))
    });
    c.paint(&shape, |_, y, d| {
        let top = if y < 4.0 { 0.7 } else { 0.0 };
        (rgb(0xCFC8AC, top), band(d, -2.0, -1.0))
    });
    c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, -1.0, 0.0)));
    c.paint(&shape, |_, y, d| {
        let below = if y >= 4.0 { 1.0 } else { 0.0 };
        (rgb(0x545346, below), band(d, -2.0, -1.0))
    });
    c.finish()
}

/// The square header button: dark (`#2b2a22` to `#141310`) in a stone line (`#a4a192`, gold
/// `#ffe49b` when `lit`), corners 3, with `glyph` in cream.
fn header_button(glyph: &[Stroke], lit: bool) -> UiPixels {
    let side = 17usize;
    let mut c = Canvas::new(side, side);
    let s = side as f32;
    let shape = round_rect(0.0, 0.0, s, s, 3.0);
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x2B2A22, 1.0), rgb(0x141310, 1.0), y / s),
            inside(d),
        )
    });
    let line = if lit { 0xFFE49B } else { 0xA4A192 };
    c.paint(&shape, |_, _, d| (rgb(line, 1.0), band(d, -1.0, 0.0)));
    c.paint(
        |x, y| stroke_distance(glyph, x, y) - 0.8,
        |_, _, d| (rgb(CREAM, 1.0), inside(d)),
    );
    c.finish()
}

const CROSS: [Stroke; 2] = [
    Stroke::Line(&[(5.0, 5.0), (12.0, 12.0)]),
    Stroke::Line(&[(12.0, 5.0), (5.0, 12.0)]),
];
const QUESTION: [Stroke; 2] = [
    Stroke::Line(&[
        (6.0, 6.5),
        (6.8, 4.6),
        (8.5, 4.0),
        (10.2, 4.6),
        (11.0, 6.2),
        (10.4, 7.8),
        (8.5, 9.0),
        (8.5, 10.6),
    ]),
    Stroke::Line(&[(8.5, 12.8), (8.5, 13.2)]),
];

/// A tab, 24 by 20 (its pieces the left 8, the middle 8 and the right 8): dark glass in the line
/// colour, or lighter glass in gold when `on`, corners 5.
fn tab(on: bool) -> UiPixels {
    let (w, h) = (24usize, 20usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.5, 0.5, w as f32 - 0.5, h as f32 - 0.5, 5.0);
    let (top, bottom, line) = if on {
        (0x1E3A33, 0x0C2420, GOLD)
    } else {
        (0x0F1A18, 0x08110F, LINE)
    };
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(top, 0.95), rgb(bottom, 0.95), y / h as f32),
            inside(d),
        )
    });
    c.paint(&shape, |_, _, d| (rgb(line, 1.0), band(d, -1.0, 0.0)));
    c.finish()
}

/// `image`'s columns `x0..x1`.
fn columns(image: &UiPixels, x0: u32, x1: u32) -> UiPixels {
    let mut rgba = Vec::with_capacity(((x1 - x0) * image.height * 4) as usize);
    for y in 0..image.height {
        let row = (y * image.width) as usize * 4;
        rgba.extend_from_slice(&image.rgba[row + x0 as usize * 4..row + x1 as usize * 4]);
    }
    UiPixels {
        width: x1 - x0,
        height: image.height,
        rgba: rgba.into(),
    }
}

/// `image`'s cell `(cx, cy)` of a grid of `cell`-pixel squares.
fn cell(image: &UiPixels, cx: u32, cy: u32, cell: u32) -> UiPixels {
    let mut rgba = Vec::with_capacity((cell * cell * 4) as usize);
    for y in cy * cell..(cy + 1) * cell {
        let row = (y * image.width) as usize * 4;
        rgba.extend_from_slice(
            &image.rgba[row + (cx * cell) as usize * 4..row + ((cx + 1) * cell) as usize * 4],
        );
    }
    UiPixels {
        width: cell,
        height: cell,
        rgba: rgba.into(),
    }
}

/// The text button, 40 by 24, ends round: bronze shaded down in a bronze line (normal), brighter
/// in gold (lit), darker in gold (pressed), or faint (ghosted).
fn button(state: usize) -> UiPixels {
    let (w, h) = (40usize, 24usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.5, 0.5, w as f32 - 0.5, h as f32 - 0.5, 9.0);
    let (top, bottom, line, alpha) = match state {
        0 => (0x4A3F23, 0x241D0D, GOLD2, 1.0),
        1 => (0x6F5E34, 0x362C14, GOLD, 1.0),
        2 => (0x241D0D, 0x4A3F23, GOLD, 1.0),
        _ => (0x1D211C, 0x101412, 0x3D3B31, 0.6),
    };
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(top, alpha), rgb(bottom, alpha), y / h as f32),
            inside(d),
        )
    });
    c.paint(&shape, |_, _, d| (rgb(line, alpha), band(d, -1.0, 0.0)));
    c.finish()
}

/// The well, 30 square (its nine pieces 10 square): `#00110f` in the line colour, corners 5.
fn well() -> UiPixels {
    let side = 30usize;
    let mut c = Canvas::new(side, side);
    let s = side as f32;
    let shape = round_rect(0.5, 0.5, s - 0.5, s - 0.5, 5.0);
    c.paint(&shape, |_, _, d| (rgb(0x00110F, 0.85), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(LINE, 1.0), band(d, -1.0, 0.0)));
    c.finish()
}

/// The check box, 15 square: a well, with a gold tick when `on`.
fn check(on: bool) -> UiPixels {
    let side = 15usize;
    let mut c = Canvas::new(side, side);
    let s = side as f32;
    let shape = round_rect(1.0, 1.0, s - 1.0, s - 1.0, 3.0);
    c.paint(&shape, |_, _, d| (rgb(0x00110F, 1.0), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(LINE, 1.0), band(d, -1.0, 0.0)));
    if on {
        const TICK: [Stroke; 1] = [Stroke::Line(&[(4.0, 7.5), (6.5, 10.0), (11.0, 4.5)])];
        c.paint(
            |x, y| stroke_distance(&TICK, x, y) - 1.0,
            |_, _, d| (rgb(GOLD, 1.0), inside(d)),
        );
    }
    c.finish()
}

const RIGHT: [Stroke; 1] = [Stroke::Line(&[(5.5, 5.5), (10.5, 11.0), (5.5, 16.5)])];
const DOWN: [Stroke; 1] = [Stroke::Line(&[(3.0, 5.5), (8.0, 10.5), (13.0, 5.5)])];

/// A chevron, `w` by `h` (`RIGHT` 16 by 22, `DOWN` 16 square), gold (brighter when `lit`).
fn chevron(w: usize, h: usize, strokes: &[Stroke], lit: bool) -> UiPixels {
    let mut c = Canvas::new(w, h);
    let colour = if lit { GOLD } else { GOLD2 };
    c.paint(
        |x, y| stroke_distance(strokes, x, y) - 0.9,
        |_, _, d| (rgb(colour, 1.0), inside(d)),
    );
    c.finish()
}

/// The rule: a line in the line colour along the middle of three rows, fading at its ends.
fn rule() -> UiPixels {
    let (w, h) = (32usize, 3usize);
    let mut c = Canvas::new(w, h);
    c.paint(
        |_, y| if (1.0..2.0).contains(&y) { -1.0 } else { 1.0 },
        |x, _, d| {
            let edge = (x.min(w as f32 - x) / 12.0).min(1.0);
            (rgb(LINE, edge), inside(d))
        },
    );
    c.finish()
}

/// The windows' images made into `images`.
pub(crate) fn bake(images: &mut HashMap<u32, UiPixels>) {
    let tabs = [tab(false), tab(true)];
    for (i, image) in tabs.iter().enumerate() {
        for (j, (x0, x1)) in [(0, 8), (8, 16), (16, 24)].into_iter().enumerate() {
            images.insert(WIN_TAB[i * 3 + j], columns(image, x0, x1));
        }
    }
    let w = well();
    for (i, &id) in WIN_WELL.iter().enumerate() {
        images.insert(id, cell(&w, i as u32 % 3, i as u32 / 3, 10));
    }
    for (state, &id) in WIN_BUTTON.iter().enumerate() {
        images.insert(id, button(state));
    }
    for (id, image) in [
        (WIN_GLASS, glass()),
        (WIN_CLOSE, header_button(&CROSS, false)),
        (WIN_CLOSE_LIT, header_button(&CROSS, true)),
        (WIN_HELP, header_button(&QUESTION, false)),
        (WIN_HELP_LIT, header_button(&QUESTION, true)),
        (WIN_CHECK, check(false)),
        (WIN_CHECK_ON, check(true)),
        (WIN_CHEVRON_RIGHT, chevron(16, 22, &RIGHT, false)),
        (WIN_CHEVRON_RIGHT_LIT, chevron(16, 22, &RIGHT, true)),
        (WIN_CHEVRON_DOWN, chevron(16, 16, &DOWN, false)),
        (WIN_CHEVRON_DOWN_LIT, chevron(16, 16, &DOWN, true)),
        (WIN_RULE, rule()),
    ] {
        images.insert(id, image);
    }
}
