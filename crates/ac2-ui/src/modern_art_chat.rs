//! The modern chat's art (`modern_chat`): a window's glass (and its active look), its header, a
//! tab, the log's backing, the entry's bar and box, the channel pill, the settings button and the
//! resize grip, drawn by `modern_art`'s painter under the client's own DIDs (`0x7E0100Cx`).

use std::collections::HashMap;

use crate::compose::UiPixels;
use crate::modern_art::{band, inside, mix, rgb, round_rect, Canvas};
use crate::modern_art_drawer::{stroke_distance, Stroke};

pub const CHAT_GLASS: u32 = 0x7E01_00C0;
pub const CHAT_GLASS_ACTIVE: u32 = 0x7E01_00C1;
pub const CHAT_HEADER: u32 = 0x7E01_00C2;
pub const CHAT_TAB: u32 = 0x7E01_00C3;
pub const CHAT_LOG: u32 = 0x7E01_00C4;
pub const CHAT_ENTRY_BAR: u32 = 0x7E01_00C5;
pub const CHAT_INPUT: u32 = 0x7E01_00C6;
pub const CHAT_PILL: u32 = 0x7E01_00C7;
pub const CHAT_MENU: u32 = 0x7E01_00C8;
pub const CHAT_MENU_LIT: u32 = 0x7E01_00C9;
pub const CHAT_GRIP: u32 = 0x7E01_00CA;
pub const CHAT_KEY: u32 = 0x7E01_00CB;
pub const CHAT_CLEAR: u32 = 0x7E01_00CC;
/// A tab not shown (`#00110f` in a fine line), the `+` button, the unread dot.
pub const CHAT_TAB_OFF: u32 = 0x7E01_00D1;
pub const CHAT_ADD: u32 = 0x7E01_00D2;
pub const CHAT_UNREAD: u32 = 0x7E01_00D3;
/// The "new below" pill: dark gold (`#3a2c08`) in a gold line, ends round.
pub const CHAT_NEW_PILL: u32 = 0x7E01_00D4;
/// The settings popover's glass.
pub const CHAT_POPOVER: u32 = 0x7E01_00CD;
/// The log's backing in each tint (`TINTS`' order: teal, black, bronze, blue).
pub const CHAT_TINTS: [u32; 4] = [0x7E01_00C4, 0x7E01_00CE, 0x7E01_00CF, 0x7E01_00D0];
/// The tints' names and colours (the design's `rgba(r,g,b,.8)`).
pub const TINT_NAMES: [&str; 4] = ["Teal", "Black", "Bronze", "Blue"];
const TINT_COLORS: [u32; 4] = [0x001412, 0x000000, 0x281C08, 0x06142C];
/// The corners the stretched parts keep.
pub const CHAT_BORDER: i32 = 8;
pub const TAB_BORDER: i32 = 5;
pub const PILL_BORDER_CHAT: i32 = 8;

/// A window's glass: teal at half (`rgba(0,26,23,.5)`) in a bronze line (gold and glowing when
/// active) and a black rim, corners 7.
fn glass(active: bool) -> UiPixels {
    let (w, h) = (24usize, 24usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(1.0, 1.0, w as f32 - 1.0, h as f32 - 1.0, 7.0);
    c.paint(&shape, |_, _, d| (rgb(0x001A17, 0.5), inside(d)));
    let line = if active {
        rgb(0xE1C56B, 1.0)
    } else {
        rgb(0x8B846C, 1.0)
    };
    c.paint(&shape, |_, _, d| (line, band(d, -1.0, 0.0)));
    c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// The header: bronze shaded down (`#3b382d` to `#16150f`), a light line on top and a dark one
/// under, top corners 6.
fn header() -> UiPixels {
    let (w, h) = (24usize, 26usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(1.0, 1.0, w as f32 - 1.0, h as f32 + 8.0, 6.0);
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x3B382D, 1.0), rgb(0x16150F, 1.0), y / h as f32),
            inside(d),
        )
    });
    c.paint(&shape, |_, y, d| {
        (
            rgb(0xCFC8AC, 1.0),
            if y < 2.5 { band(d, -1.0, 0.0) } else { 0.0 },
        )
    });
    c.paint(
        |_, y| if y > h as f32 - 1.0 { -1.0 } else { 1.0 },
        |_, _, d| (rgb(0x000000, 1.0), inside(d)),
    );
    c.finish()
}

/// A tab (on): green-dark shaded down (`#35402e` to `#16200f`) in a gold line, top corners 5.
fn tab() -> UiPixels {
    let (w, h) = (16usize, 21usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32 + 6.0, 5.0);
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x35402E, 1.0), rgb(0x16200F, 1.0), y / h as f32),
            inside(d),
        )
    });
    c.paint(&shape, |_, _, d| (rgb(0xD9C27A, 1.0), band(d, -1.0, 0.0)));
    c.finish()
}

/// A tab not shown: `#00110f` in a fine line (`#6f6957`), top corners 5.
fn tab_off() -> UiPixels {
    let (w, h) = (16usize, 21usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32 + 6.0, 5.0);
    c.paint(&shape, |_, _, d| (rgb(0x00110F, 1.0), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0x6F6957, 1.0), band(d, -1.0, 0.0)));
    c.finish()
}

/// The `+` button: bronze in a light line with a gold cross, corners 4.
fn add() -> UiPixels {
    let side = 19usize;
    let mut c = Canvas::new(side, side);
    let shape = round_rect(0.0, 0.0, side as f32, side as f32, 4.0);
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x58544A, 1.0), rgb(0x242219, 1.0), y / side as f32),
            inside(d),
        )
    });
    c.paint(&shape, |_, _, d| (rgb(0xB8B19B, 1.0), band(d, -1.0, 0.0)));
    const CROSS: [Stroke; 2] = [
        Stroke::Line(&[(9.5, 5.5), (9.5, 13.5)]),
        Stroke::Line(&[(5.5, 9.5), (13.5, 9.5)]),
    ];
    c.paint(
        |x, y| stroke_distance(&CROSS, x, y) - 0.8,
        |_, _, d| (rgb(0xD9C27A, 1.0), inside(d)),
    );
    c.finish()
}

/// The unread dot: the design's teal (`#7fe0c9`) with a glow.
fn unread() -> UiPixels {
    let side = 10usize;
    let mut c = Canvas::new(side, side);
    let dot = crate::modern_art::circle(5.0, 5.0, 3.0);
    c.paint(&dot, |_, _, d| {
        (
            rgb(0x7FE0C9, 1.0),
            if d > 0.0 {
                (0.6 - d / 3.0).max(0.0)
            } else {
                1.0
            },
        )
    });
    c.finish()
}

/// The "new below" pill.
fn new_pill() -> UiPixels {
    let (w, h) = (24usize, 18usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 9.0);
    c.paint(&shape, |_, _, d| (rgb(0x3A2C08, 1.0), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0xD9C27A, 1.0), band(d, -1.0, 0.0)));
    c.finish()
}

/// A plain fill: `color` at `alpha`, `w` by `h`.
fn fill(color: u32, alpha: f32) -> UiPixels {
    let mut c = Canvas::new(4, 4);
    c.paint(|_, _| -1.0, |_, _, _| (rgb(color, alpha), 1.0));
    c.finish()
}

/// The entry's bar: near-black glass (`#00110fd8`) with a bronze line on top, bottom corners 6.
fn entry_bar() -> UiPixels {
    let (w, h) = (24usize, 24usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(1.0, -8.0, w as f32 - 1.0, h as f32 - 1.0, 6.0);
    c.paint(&shape, |_, _, d| (rgb(0x00110F, 0.85), inside(d)));
    c.paint(
        |_, y| if y < 1.0 { -1.0 } else { 1.0 },
        |_, _, d| (rgb(0x6F6957, 1.0), inside(d)),
    );
    c.finish()
}

/// The entry's box: `#031416` in a dark line (`#3b3a2e`), corners 3.
fn input() -> UiPixels {
    let (w, h) = (16usize, 16usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 3.0);
    c.paint(&shape, |_, _, d| (rgb(0x031416, 1.0), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0x3B3A2E, 1.0), band(d, -1.0, 0.0)));
    c.finish()
}

/// The channel pill: a light line round a clear middle (the text gives the channel's colour).
fn pill() -> UiPixels {
    let (w, h) = (20usize, 16usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.5, 0.5, w as f32 - 0.5, h as f32 - 0.5, 7.5);
    c.paint(&shape, |_, _, d| (rgb(0x000000, 0.4), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0xEFE8CC, 1.0), band(d, -1.2, 0.0)));
    c.finish()
}

/// The settings button: bronze in a light line with the design's `=`, gold when `lit`.
fn menu(lit: bool) -> UiPixels {
    let (w, h) = (20usize, 19usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 4.0);
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x58544A, 1.0), rgb(0x242219, 1.0), y / h as f32),
            inside(d),
        )
    });
    let line = if lit {
        rgb(0xF6E27A, 1.0)
    } else {
        rgb(0xB8B19B, 1.0)
    };
    c.paint(&shape, |_, _, d| (line, band(d, -1.0, 0.0)));
    const BARS: [Stroke; 2] = [
        Stroke::Line(&[(6.0, 7.5), (14.0, 7.5)]),
        Stroke::Line(&[(6.0, 11.5), (14.0, 11.5)]),
    ];
    c.paint(
        |x, y| stroke_distance(&BARS, x, y) - 0.7,
        |_, _, d| (rgb(0xD9C27A, 1.0), inside(d)),
    );
    c.finish()
}

/// The resize grip: the design's two diagonal light lines in the lower right corner.
fn grip() -> UiPixels {
    let side = 12usize;
    let mut c = Canvas::new(side, side);
    const LINES: [Stroke; 2] = [
        Stroke::Line(&[(12.0, 5.5), (5.5, 12.0)]),
        Stroke::Line(&[(12.0, 9.0), (9.0, 12.0)]),
    ];
    c.paint(
        |x, y| stroke_distance(&LINES, x, y) - 0.5,
        |_, _, d| (rgb(0xB8B19B, 1.0), inside(d)),
    );
    c.finish()
}

/// A tab's key chip (`F1`): dark (`#000a`), corners 2.
fn key() -> UiPixels {
    let (w, h) = (8usize, 8usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 2.0);
    c.paint(&shape, |_, _, d| (rgb(0x000000, 0.67), inside(d)));
    c.finish()
}

/// The popover's glass: deep teal (`rgba(0,36,32,.97)`) in a light bronze line (`#c3bca4`) and a
/// black rim, corners 8.
fn popover() -> UiPixels {
    let (w, h) = (24usize, 24usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(1.0, 1.0, w as f32 - 1.0, h as f32 - 1.0, 8.0);
    c.paint(&shape, |_, _, d| (rgb(0x002420, 0.97), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0xC3BCA4, 1.0), band(d, -1.0, 0.0)));
    c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// The chat's images made into `images`.
pub(crate) fn bake(images: &mut HashMap<u32, UiPixels>) {
    for (id, image) in [
        (CHAT_GLASS, glass(false)),
        (CHAT_GLASS_ACTIVE, glass(true)),
        (CHAT_HEADER, header()),
        (CHAT_TAB, tab()),
        (CHAT_POPOVER, popover()),
        (CHAT_TAB_OFF, tab_off()),
        (CHAT_ADD, add()),
        (CHAT_UNREAD, unread()),
        (CHAT_NEW_PILL, new_pill()),
        (CHAT_ENTRY_BAR, entry_bar()),
        (CHAT_INPUT, input()),
        (CHAT_PILL, pill()),
        (CHAT_MENU, menu(false)),
        (CHAT_MENU_LIT, menu(true)),
        (CHAT_GRIP, grip()),
        (CHAT_KEY, key()),
        (CHAT_CLEAR, Canvas::new(1, 1).finish()),
    ] {
        images.insert(id, image);
    }
    for (id, color) in CHAT_TINTS.into_iter().zip(TINT_COLORS) {
        images.insert(id, fill(color, 0.8));
    }
}
