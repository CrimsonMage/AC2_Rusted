//! The modern HUD's drawer art (`modern_art`'s, split out): the inventory drawer's header, gold
//! row, doll frame, header button backings, item wells, search field, slot veil, filter chips, All
//! tab and section rule, and the thin scroll bar. Drawn by `modern_art`'s painter under the
//! client's own DIDs (`0x7E0100xx`).

use std::collections::HashMap;

use crate::compose::UiPixels;
use crate::modern_art::{band, circle, inside, mix, rgb, round_rect, Canvas, ITEM_PITCH};

/// The drawers' parts: the header, the gold row, the doll's frame, a header button's backing and
/// an item slot's well (the DAT slot at 36 pixels and the 4-pixel gap after it).
pub const DRAWER_HEADER: u32 = 0x7E01_0060;
pub const MONEY_ROW: u32 = 0x7E01_0061;
pub const DOLL_FRAME: u32 = 0x7E01_0062;
pub const HEADER_BUTTON: u32 = 0x7E01_0063;
pub const ITEM_WELL: u32 = 0x7E01_0064;
/// The drawer's search field: a dark pill in a fine bronze line.
pub const SEARCH_PILL: u32 = 0x7E01_0065;
/// The veil over an item slot the search leaves out.
pub const SLOT_VEIL: u32 = 0x7E01_0066;
/// The drawer's thin scroll bar: the track, the thumb's top, middle and bottom, and the arrows
/// (up, up lit, down, down lit), all `SCROLL_WIDTH` wide.
pub const SCROLL_TRACK: u32 = 0x7E01_0070;
pub const THUMB_TOP: u32 = 0x7E01_0071;
pub const THUMB_MID: u32 = 0x7E01_0072;
pub const THUMB_BOTTOM: u32 = 0x7E01_0073;
pub const SCROLL_UP: u32 = 0x7E01_0074;
pub const SCROLL_UP_LIT: u32 = 0x7E01_0075;
pub const SCROLL_DOWN: u32 = 0x7E01_0076;
pub const SCROLL_DOWN_LIT: u32 = 0x7E01_0077;
pub const SCROLL_WIDTH: i32 = 10;
/// The drawer's All tab, off and on, its corners, and the rule under a section's heading.
pub const ALL_TAB: u32 = 0x7E01_0078;
pub const ALL_TAB_ON: u32 = 0x7E01_0079;
pub const ALL_TAB_BORDER: i32 = 6;
pub const SECTION_RULE: u32 = 0x7E01_007A;
/// The drawer's filter chips, off and on.
pub const CHIP: u32 = 0x7E01_0067;
pub const CHIP_ON: u32 = 0x7E01_0068;
/// The chips' art keeps corners this size when stretched.
pub const CHIP_BORDER: i32 = 8;
/// The search pill's art keeps corners this size when stretched.
pub const SEARCH_PILL_BORDER: i32 = 10;

/// The item ring's buttons (`modern_item_ring`): one per action, each at rest, lit (under the
/// pointer) and dimmed (the action does not apply), `RING_ART` square.
pub const RING_ACTIONS: usize = 6;
pub const RING_BASE: u32 = 0x7E01_0080;
pub const RING_ART: i32 = 28;
/// The ring button image of action `action` (its index in `RING_ACTIONS`) in `look` (0 at rest,
/// 1 lit, 2 dimmed).
pub const fn ring_button(action: usize, look: usize) -> u32 {
    RING_BASE + (action * 3 + look) as u32
}

/// A drawer's header: a dark bronze fall with a light line along its top and a black one under it
/// (nine pieces).
fn drawer_header() -> UiPixels {
    let (w, h) = (32usize, 30usize);
    let mut c = Canvas::new(w, h);
    c.paint(
        |_, _| -1.0,
        |_, y, _| {
            let t = (y / (0.7 * h as f32)).min(1.0);
            (mix(rgb(0x3B382D, 1.0), rgb(0x16150F, 1.0), t), 1.0)
        },
    );
    c.paint(
        |_, y| if y < 1.0 { -1.0 } else { 1.0 },
        |_, _, d| (rgb(0xCFC8AC, 1.0), inside(d)),
    );
    c.paint(
        |_, y| if y > h as f32 - 1.0 { -1.0 } else { 1.0 },
        |_, _, d| (rgb(0x000000, 1.0), inside(d)),
    );
    c.finish()
}

/// The gold row's backing: dark glass in a fine bronze line, corners 5 round (nine pieces).
fn money_row() -> UiPixels {
    let (w, h) = (32usize, 32usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 5.0);
    c.paint(&shape, |_, _, d| (rgb(0x00110F, 0.63), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0x6F6957, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// The search field's pill: dark glass, a fine bronze line, ends fully round.
fn search_pill() -> UiPixels {
    let (w, h) = (40usize, 20usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 10.0);
    c.paint(&shape, |_, _, d| (rgb(0x00110F, 1.0), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0x6F6957, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// The scroll bar's track: the well's dark glass, tiled.
fn scroll_track() -> UiPixels {
    let w = SCROLL_WIDTH as usize;
    let mut c = Canvas::new(w, 4);
    let shape = round_rect(0.0, -4.0, w as f32, 12.0, 0.0);
    c.paint(&shape, |_, _, d| (rgb(0x00110F, 1.0), inside(d)));
    c.finish()
}

/// A piece of the thumb (`part`: 0 top cap, 1 middle, 2 bottom cap): bronze shaded left to
/// right (`#a8a28a` to `#4d493c`), its ends round.
fn thumb(part: u8) -> UiPixels {
    let w = SCROLL_WIDTH as usize;
    let h = if part == 1 { 4 } else { 7 };
    let mut c = Canvas::new(w, h);
    let r = w as f32 / 2.0 - 1.0;
    let shape = match part {
        0 => round_rect(1.0, 0.0, w as f32 - 1.0, h as f32 + 2.0 * r, r),
        2 => round_rect(1.0, -2.0 * r, w as f32 - 1.0, h as f32, r),
        _ => round_rect(1.0, -4.0, w as f32 - 1.0, h as f32 + 4.0, 0.0),
    };
    c.paint(&shape, |x, _, d| {
        let t = (x / w as f32).clamp(0.0, 1.0);
        (mix(rgb(0xA8A28A, 1.0), rgb(0x4D493C, 1.0), t), inside(d))
    });
    c.finish()
}

/// A scroll arrow on the track's glass: a small chevron, bronze, or gold when `lit`.
fn scroll_arrow(up: bool, lit: bool) -> UiPixels {
    let w = SCROLL_WIDTH as usize;
    let mut c = Canvas::new(w, w);
    let back = round_rect(0.0, 0.0, w as f32, w as f32, 0.0);
    c.paint(&back, |_, _, d| (rgb(0x00110F, 1.0), inside(d)));
    let color = if lit {
        rgb(0xD9C27A, 1.0)
    } else {
        rgb(0x8B846C, 1.0)
    };
    let (cx, cy) = (w as f32 / 2.0, w as f32 / 2.0);
    c.paint(&back, |x, y, _| {
        // A chevron 6 wide and 3 tall, 1.4 pixels thick.
        let dy = if up { cy + 1.5 - y } else { y - (cy - 1.5) };
        let edge = (dy - (3.0 - (x - cx).abs())).abs();
        let within = (x - cx).abs() <= 3.2 && (-0.5..=3.5).contains(&dy);
        (
            color,
            if within {
                (1.0 - (edge - 0.7).max(0.0)).clamp(0.0, 1.0)
            } else {
                0.0
            },
        )
    });
    c.finish()
}

/// The All tab: bronze shaded top to bottom in a fine line, its top corners round; brighter on.
fn all_tab(on: bool) -> UiPixels {
    let (w, h) = (16usize, 26usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32 + 6.0, 5.0);
    let (top, bottom) = if on {
        (rgb(0x6B5C2E, 1.0), rgb(0x2C2511, 1.0))
    } else {
        (rgb(0x3B382D, 1.0), rgb(0x16150F, 1.0))
    };
    c.paint(&shape, |_, y, d| {
        (mix(top, bottom, y / h as f32), inside(d))
    });
    c.paint(&shape, |_, _, d| (rgb(0x8B846C, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// The rule under a section's heading: the design's bronze at a quarter (`#8b846c44`).
fn section_rule() -> UiPixels {
    let mut c = Canvas::new(4, 1);
    let shape = round_rect(-1.0, -1.0, 5.0, 2.0, 0.0);
    c.paint(&shape, |_, _, d| (rgb(0x8B846C, 0.27), inside(d)));
    c.finish()
}

/// A filter chip: a dark pill in a fine bronze line, or (on) filled bronze.
fn chip(on: bool) -> UiPixels {
    let (w, h) = (24usize, 16usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 8.0);
    if on {
        c.paint(&shape, |_, _, d| (rgb(0x6B5C2E, 1.0), inside(d)));
    } else {
        c.paint(&shape, |_, _, d| (rgb(0x00110F, 0.63), inside(d)));
        c.paint(&shape, |_, _, d| (rgb(0x6F6957, 1.0), band(d, 0.0, 1.0)));
    }
    c.finish()
}

/// The veil over a slot the search leaves out: the drawer's glass at 82%, so the slot shows at
/// about the design's 18%.
fn slot_veil() -> UiPixels {
    let side = ITEM_PITCH as usize;
    let mut c = Canvas::new(side, side);
    let shape = round_rect(0.0, 0.0, side as f32, side as f32, 3.0);
    c.paint(&shape, |_, _, d| (rgb(0x001F1D, 0.82), inside(d)));
    c.finish()
}

/// The paper doll's frame: a teal glow behind the figure in a fine bronze line, corners 8 round.
fn doll_frame() -> UiPixels {
    let (w, h) = (132usize, 196usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 8.0);
    let (cx, cy) = (w as f32 / 2.0, 0.6 * h as f32);
    c.paint(&shape, |x, y, d| {
        let t =
            (((x - cx) / (w as f32 / 2.0)).powi(2) + ((y - cy) / (h as f32 / 2.0)).powi(2)).sqrt();
        let color = if t < 0.62 {
            mix(rgb(0x0F4D45, 1.0), rgb(0x032622, 1.0), t / 0.62)
        } else {
            mix(rgb(0x032622, 1.0), rgb(0x011513, 1.0), (t - 0.62) / 0.38)
        };
        (color, inside(d))
    });
    c.paint(&shape, |_, _, d| (rgb(0x6F6957, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// A header button's backing: a dark bronze square in a light line and a black rim, corners 3.
fn header_button() -> UiPixels {
    let n = 20usize;
    let mut c = Canvas::new(n, n);
    let shape = round_rect(1.0, 1.0, n as f32 - 1.0, n as f32 - 1.0, 3.0);
    c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, -1.0, 0.0)));
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x58544A, 1.0), rgb(0x242219, 1.0), y / n as f32),
            inside(d),
        )
    });
    c.paint(&shape, |_, _, d| (rgb(0xB8B19B, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// An item slot's well: the DAT slot at its 36 pixels in the corner of the pitch square.
fn item_well(slot: &UiPixels) -> UiPixels {
    let n = ITEM_PITCH as usize;
    let mut c = Canvas::new(n, n);
    c.image(slot, (0.0, 0.0, 36.0, 36.0), 1.0);
    c.finish()
}

/// The ring icons' strokes in the design's 24-unit box (its SVG paths as polylines and circles).
enum Stroke {
    Line(&'static [(f32, f32)]),
    Ring(f32, f32, f32),
}

const HAND: [Stroke; 3] = [
    Stroke::Line(&[
        (8.0, 12.0),
        (8.0, 5.5),
        (9.5, 4.0),
        (11.0, 5.5),
        (11.0, 10.0),
    ]),
    Stroke::Line(&[(11.0, 7.5), (12.5, 6.0), (14.0, 7.5), (14.0, 11.0)]),
    Stroke::Line(&[
        (14.0, 9.0),
        (15.5, 7.5),
        (17.0, 9.0),
        (17.0, 13.5),
        (15.5, 17.5),
        (12.0, 20.5),
        (10.5, 20.5),
        (7.5, 18.5),
        (5.0, 15.5),
        (4.0, 13.0),
        (6.3, 11.6),
        (8.0, 14.0),
    ]),
];
const LENS: [Stroke; 2] = [
    Stroke::Ring(11.0, 11.0, 6.0),
    Stroke::Line(&[(15.5, 15.5), (21.0, 21.0)]),
];
const FORK: [Stroke; 2] = [
    Stroke::Line(&[(12.0, 3.0), (12.0, 11.0), (5.0, 20.0)]),
    Stroke::Line(&[(12.0, 11.0), (19.0, 20.0)]),
];
const COIN: [Stroke; 3] = [
    Stroke::Ring(12.0, 12.0, 8.0),
    Stroke::Line(&[(12.0, 7.0), (12.0, 17.0)]),
    Stroke::Line(&[
        (13.5, 9.5),
        (10.6, 9.5),
        (9.7, 11.2),
        (10.6, 12.5),
        (13.4, 12.5),
        (14.3, 14.2),
        (13.4, 16.0),
        (10.5, 16.0),
    ]),
];
const SHIRT_PATH: [(f32, f32); 11] = [
    (9.0, 4.0),
    (12.0, 6.0),
    (15.0, 4.0),
    (20.0, 8.0),
    (17.0, 11.0),
    (15.0, 10.0),
    (15.0, 20.0),
    (9.0, 20.0),
    (9.0, 10.0),
    (7.0, 11.0),
    (4.0, 8.0),
];
const SHIRT: [Stroke; 2] = [
    Stroke::Line(&SHIRT_PATH),
    Stroke::Line(&[(4.0, 8.0), (9.0, 4.0)]),
];
const NO_SHIRT: [Stroke; 3] = [
    Stroke::Line(&SHIRT_PATH),
    Stroke::Line(&[(4.0, 8.0), (9.0, 4.0)]),
    Stroke::Line(&[(3.0, 3.0), (21.0, 21.0)]),
];

/// The ring's icons, in `modern_item_ring::Action`'s order: use, examine, split, melt, equip,
/// unequip.
const RING_ICONS: [&[Stroke]; RING_ACTIONS] = [&HAND, &LENS, &FORK, &COIN, &SHIRT, &NO_SHIRT];

/// The distance from `(x, y)` to the strokes.
fn stroke_distance(strokes: &[Stroke], x: f32, y: f32) -> f32 {
    let segment = |(ax, ay): (f32, f32), (bx, by): (f32, f32)| {
        let (dx, dy) = (bx - ax, by - ay);
        let t = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy).max(1e-6)).clamp(0.0, 1.0);
        ((x - ax - t * dx).powi(2) + (y - ay - t * dy).powi(2)).sqrt()
    };
    strokes
        .iter()
        .map(|s| match s {
            Stroke::Line(points) => points
                .windows(2)
                .map(|w| segment(w[0], w[1]))
                .fold(f32::MAX, f32::min),
            Stroke::Ring(cx, cy, r) => (((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r).abs(),
        })
        .fold(f32::MAX, f32::min)
}

/// A ring button: dark teal glass in a bronze line (gold and glowing when `lit`), its icon's
/// strokes in ivory, 16 of the button's 28 pixels across; all at 30% when `dim`.
fn ring_button_art(icon: &[Stroke], lit: bool, dim: bool) -> UiPixels {
    let side = RING_ART as usize;
    let mut c = Canvas::new(side, side);
    let half = RING_ART as f32 / 2.0;
    let body = circle(half, half, half - 1.0);
    c.paint(&body, |x, y, d| {
        let t = (((x - half).powi(2) + (y - half).powi(2)).sqrt() / half).min(1.0);
        (mix(rgb(0x124A42, 1.0), rgb(0x031A17, 1.0), t), inside(d))
    });
    let (line, width) = if lit {
        (rgb(0xFFF0B0, 1.0), 2.0)
    } else {
        (rgb(0xA88F48, 1.0), 1.0)
    };
    c.paint(&body, |_, _, d| (line, band(d, -width, 0.0)));
    c.paint(&body, |_, _, d| (rgb(0x000000, 1.0), band(d, 0.0, 1.0)));
    // The icon: 16 pixels for the design's 24 units, strokes 1.6 units wide.
    let scale = 24.0 / 16.0;
    let offset = (RING_ART as f32 - 16.0) / 2.0;
    c.paint(
        |x, y| stroke_distance(icon, (x - offset) * scale, (y - offset) * scale) / scale - 0.55,
        |_, _, d| (rgb(0xF1E9C8, 1.0), inside(d)),
    );
    let mut image = c.finish();
    if dim {
        let rgba: Vec<u8> = image
            .rgba
            .chunks(4)
            .flat_map(|p| [p[0], p[1], p[2], (f32::from(p[3]) * 0.3).round() as u8])
            .collect();
        image.rgba = rgba.into();
    }
    image
}

/// The drawer's images made into `images`; the item well from the DAT slot `item_slot` when it
/// is loaded.
pub(crate) fn bake(images: &mut HashMap<u32, UiPixels>, item_slot: Option<&UiPixels>) {
    for (id, image) in [
        (DRAWER_HEADER, drawer_header()),
        (MONEY_ROW, money_row()),
        (SEARCH_PILL, search_pill()),
        (SLOT_VEIL, slot_veil()),
        (CHIP, chip(false)),
        (ALL_TAB, all_tab(false)),
        (ALL_TAB_ON, all_tab(true)),
        (SECTION_RULE, section_rule()),
        (SCROLL_TRACK, scroll_track()),
        (THUMB_TOP, thumb(0)),
        (THUMB_MID, thumb(1)),
        (THUMB_BOTTOM, thumb(2)),
        (SCROLL_UP, scroll_arrow(true, false)),
        (SCROLL_UP_LIT, scroll_arrow(true, true)),
        (SCROLL_DOWN, scroll_arrow(false, false)),
        (SCROLL_DOWN_LIT, scroll_arrow(false, true)),
        (CHIP_ON, chip(true)),
        (DOLL_FRAME, doll_frame()),
        (HEADER_BUTTON, header_button()),
    ] {
        images.insert(id, image);
    }
    if let Some(slot) = item_slot {
        images.insert(ITEM_WELL, item_well(slot));
    }
    for (action, icon) in RING_ICONS.iter().enumerate() {
        for (look, (lit, dim)) in [(false, false), (true, false), (false, true)]
            .into_iter()
            .enumerate()
        {
            images.insert(ring_button(action, look), ring_button_art(icon, lit, dim));
        }
    }
}
