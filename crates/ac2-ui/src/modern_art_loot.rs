//! The modern loot window's art (`modern_loot`): the glass inside retail's frame, the buttons' pill
//! and the close button, drawn by `modern_art`'s painter under the client's own DIDs
//! (`0x7E0100Ex`).

use std::collections::HashMap;

use crate::compose::UiPixels;
use crate::modern_art::{band, inside, mix, rgb, round_rect, Canvas};
use crate::modern_art_drawer::{stroke_distance, Stroke};

pub const LOOT_GLASS: u32 = 0x7E01_00E0;
pub const LOOT_PILL: u32 = 0x7E01_00E1;
pub const LOOT_PILL_LIT: u32 = 0x7E01_00E2;
pub const LOOT_CLOSE: u32 = 0x7E01_00E3;
pub const LOOT_CLOSE_LIT: u32 = 0x7E01_00E4;
pub const LOOT_PILL_BORDER: i32 = 12;
/// The targeting banner: gold shaded down (`#f1d57a` to `#b89a45`) in a light line (`#fff3b0`)
/// with a gold glow, corners 6.
pub const TARGET_BANNER: u32 = 0x7E01_00E5;
pub const TARGET_BANNER_BORDER: i32 = 10;
pub const LOOT_GLASS_BORDER: i32 = 9;

/// The glass inside the frame: deep teal (`rgba(0,30,27,.9)`) lit from its top (a radial
/// highlight, `rgba(20,80,74,.35)`), its lower corners round.
fn glass() -> UiPixels {
    let (w, h) = (48usize, 48usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, -10.0, w as f32, h as f32, 8.0);
    c.paint(&shape, |x, y, d| {
        let t = ((((x - 24.0) / 29.0).powi(2) + (y / 34.0).powi(2)).sqrt()).min(1.0);
        let base = rgb(0x001E1B, 0.9);
        (mix(rgb(0x14504A, 0.95), base, t), inside(d))
    });
    c.finish()
}

/// The buttons' pill: bronze shaded down (`#4a3f23` to `#241d0d`) in a bronze line (`#8d7a45`),
/// brighter when `lit`, ends round.
fn pill(lit: bool) -> UiPixels {
    let (w, h) = (32usize, 24usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 12.0);
    let (top, bottom) = if lit {
        (rgb(0x6F5E34, 1.0), rgb(0x362C14, 1.0))
    } else {
        (rgb(0x4A3F23, 1.0), rgb(0x241D0D, 1.0))
    };
    c.paint(&shape, |_, y, d| {
        (mix(top, bottom, y / h as f32), inside(d))
    });
    c.paint(&shape, |_, _, d| (rgb(0x8D7A45, 1.0), band(d, -1.0, 0.0)));
    c.finish()
}

/// The close button: bronze (`#5a4c2c` to `#2a2210`) in a bronze line (`#a58e55`, `#ffe49b`
/// when `lit`) with a cream cross, corners 3.
fn close(lit: bool) -> UiPixels {
    let side = 19usize;
    let mut c = Canvas::new(side, side);
    let shape = round_rect(0.0, 0.0, side as f32, side as f32, 3.0);
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x5A4C2C, 1.0), rgb(0x2A2210, 1.0), y / side as f32),
            inside(d),
        )
    });
    let line = if lit {
        rgb(0xFFE49B, 1.0)
    } else {
        rgb(0xA58E55, 1.0)
    };
    c.paint(&shape, |_, _, d| (line, band(d, -1.0, 0.0)));
    const CROSS: [Stroke; 2] = [
        Stroke::Line(&[(5.5, 5.5), (13.5, 13.5)]),
        Stroke::Line(&[(13.5, 5.5), (5.5, 13.5)]),
    ];
    c.paint(
        |x, y| stroke_distance(&CROSS, x, y) - 0.9,
        |_, _, d| (rgb(0xECE3C4, 1.0), inside(d)),
    );
    c.finish()
}

fn banner() -> UiPixels {
    let (w, h) = (40usize, 32usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(4.0, 4.0, w as f32 - 4.0, h as f32 - 4.0, 6.0);
    c.paint(&shape, |_, _, d| {
        (
            rgb(0xF1D57A, 0.5),
            if d > 0.0 {
                (1.0 - d / 4.0).max(0.0)
            } else {
                0.0
            },
        )
    });
    c.paint(&shape, |_, y, d| {
        (
            mix(
                rgb(0xF1D57A, 1.0),
                rgb(0xB89A45, 1.0),
                (y - 4.0) / (h as f32 - 8.0),
            ),
            inside(d),
        )
    });
    c.paint(&shape, |_, _, d| (rgb(0xFFF3B0, 1.0), band(d, -1.0, 0.0)));
    c.finish()
}

/// The loot window's images made into `images`.
pub(crate) fn bake(images: &mut HashMap<u32, UiPixels>) {
    for (id, image) in [
        (LOOT_GLASS, glass()),
        (LOOT_PILL, pill(false)),
        (LOOT_PILL_LIT, pill(true)),
        (LOOT_CLOSE, close(false)),
        (LOOT_CLOSE_LIT, close(true)),
        (TARGET_BANNER, banner()),
    ] {
        images.insert(id, image);
    }
}
