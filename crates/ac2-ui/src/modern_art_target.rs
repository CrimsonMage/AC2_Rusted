//! The modern style's selection marks (a modern addition, AGENTS.md: the HUD's two styles; the
//! design's variant B): four fine gold L brackets for the reticle's corners, the dark glass card
//! behind the selection's name and meters, the slim meters themselves and the smaller glass pill
//! of every other object's name. Retail turned one diagonal gem (`0x410016EB`) for each corner
//! and drew the name and 5 pixel meters bare over the scene.

use std::collections::HashMap;

use crate::compose::UiPixels;
use crate::modern_art::{band, inside, mix, rgb, round_rect, Canvas};

/// The brackets, top-left, bottom-left, top-right and bottom-right (`BRACKETS + corner`, in the
/// reticle's corner order).
pub const BRACKETS: u32 = 0x7E01_0400;
/// The selection's card and its nine-slice border.
pub const SELECTION_CARD: u32 = 0x7E01_0410;
pub const SELECTION_CARD_BORDER: i32 = 7;
/// Another object's name pill and its nine-slice border.
pub const NAME_PILL: u32 = 0x7E01_0411;
pub const NAME_PILL_BORDER: i32 = 6;
/// The meters: health's well and fill, vigor's, and the PK losses over each.
pub const HEALTH_WELL: u32 = 0x7E01_0420;
pub const HEALTH_FILL: u32 = 0x7E01_0421;
pub const VIGOR_WELL: u32 = 0x7E01_0422;
pub const VIGOR_FILL: u32 = 0x7E01_0423;
pub const PK_DAMAGE_FILL: u32 = 0x7E01_0424;
pub const PK_VIGOR_FILL: u32 = 0x7E01_0425;

/// `TargettingReticle`'s size, which the brackets keep.
const CORNER: usize = 20;
/// The meters' width, health's height and vigor's.
pub const METER_WIDTH: i32 = 134;
pub const HEALTH_HEIGHT: i32 = 8;
pub const VIGOR_HEIGHT: i32 = 2;

/// The bracket for `corner` (0 top-left, 1 bottom-left, 2 top-right, 3 bottom-right): its bend
/// 10 pixels in from the corner's outer edges, its arms running to the inner edges (towards the
/// selection), 2 pixels of gold in a dark line.
fn bracket(corner: u32) -> UiPixels {
    let mut c = Canvas::new(CORNER, CORNER);
    let (left, top) = (corner < 2, corner.is_multiple_of(2));
    let end = CORNER as f32 - 1.0;
    let bend: f32 = 9.0;
    // The bend and the arms' ends, in the top-left bracket's terms, then mirrored.
    let flip = |x: f32, y: f32| {
        (
            if left { x } else { CORNER as f32 - x },
            if top { y } else { CORNER as f32 - y },
        )
    };
    let arms = move |x: f32, y: f32| {
        let (x, y) = flip(x, y);
        let along = |p: f32, q: f32, a: f32, b: f32| {
            // The distance to the segment from (bend, bend) to (a, b) (axis-aligned).
            let dx = (p - p.clamp(bend.min(a), bend.max(a))).abs();
            let dy = (q - q.clamp(bend.min(b), bend.max(b))).abs();
            dx.max(dy)
        };
        along(x, y, end, bend).min(along(x, y, bend, end)) - 1.0
    };
    c.paint(arms, |_, _, d| (rgb(0x000000, 0.7), inside(d - 1.3)));
    c.paint(arms, |_, _, d| (rgb(0xD9C27A, 1.0), inside(d)));
    c.finish()
}

/// A glass rounded box `w` by `h`, corners `r`: dark glass at `glass`, a gold line at `rim`.
fn glass(w: usize, h: usize, r: f32, glass: f32, rim: f32) -> UiPixels {
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.5, 0.5, w as f32 - 0.5, h as f32 - 0.5, r);
    c.paint(&shape, |_, _, d| (rgb(0x061613, glass), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0xD9C27A, rim), band(d, 0.0, 1.0)));
    c.finish()
}

/// A meter's well: a dark rounded trough.
fn well(h: i32) -> UiPixels {
    let (w, h) = (METER_WIDTH as usize, h as usize);
    let mut c = Canvas::new(w, h);
    let r = (h as f32 / 2.0).min(2.0);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, r);
    c.paint(&shape, |_, _, d| (rgb(0x000000, 0.6), inside(d)));
    c.finish()
}

/// A meter's fill: `top` at its top to `bottom` at its bottom, a light line along its top.
fn fill(h: i32, top: u32, bottom: u32) -> UiPixels {
    let (w, h) = (METER_WIDTH as usize, h as usize);
    let mut c = Canvas::new(w, h);
    let r = (h as f32 / 2.0).min(2.0);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, r);
    c.paint(&shape, |_, y, d| {
        let t = y / h as f32;
        (mix(rgb(top, 1.0), rgb(bottom, 1.0), t), inside(d))
    });
    if h > 4 {
        c.paint(&shape, |_, y, d| {
            let cover = if y < 1.5 { inside(d) } else { 0.0 };
            (rgb(0xFFFFFF, 0.25), cover)
        });
    }
    c.finish()
}

/// Every selection mark into `images`.
pub fn bake(images: &mut HashMap<u32, UiPixels>) {
    for corner in 0..4 {
        images.insert(BRACKETS + corner, bracket(corner));
    }
    images.insert(SELECTION_CARD, glass(24, 24, 5.0, 0.82, 0.6));
    images.insert(NAME_PILL, glass(20, 18, 4.0, 0.62, 0.35));
    images.insert(HEALTH_WELL, well(HEALTH_HEIGHT));
    images.insert(VIGOR_WELL, well(VIGOR_HEIGHT));
    images.insert(HEALTH_FILL, fill(HEALTH_HEIGHT, 0x7BE08A, 0x3FA855));
    images.insert(VIGOR_FILL, fill(VIGOR_HEIGHT, 0x4F8CFF, 0x4F8CFF));
    images.insert(PK_DAMAGE_FILL, fill(HEALTH_HEIGHT, 0xF0C060, 0xC98A2E));
    images.insert(PK_VIGOR_FILL, fill(VIGOR_HEIGHT, 0xB07CFF, 0xB07CFF));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(image: &UiPixels, x: usize, y: usize) -> u8 {
        image.rgba[(y * image.width as usize + x) * 4 + 3]
    }

    #[test]
    fn each_bracket_bends_away_from_the_selection() {
        // Top-left: the bend at (9, 9), the arms running right and down; nothing in the corner
        // nearest the selection.
        let tl = bracket(0);
        assert!(alpha(&tl, 9, 9) > 200);
        assert!(alpha(&tl, 18, 9) > 200);
        assert!(alpha(&tl, 9, 18) > 200);
        assert_eq!(alpha(&tl, 16, 16), 0);
        assert_eq!(alpha(&tl, 2, 2), 0);
        // Bottom-right is its mirror.
        let br = bracket(3);
        assert!(alpha(&br, 10, 10) > 200);
        assert!(alpha(&br, 1, 10) > 200);
        assert_eq!(alpha(&br, 3, 3), 0);
        // Bottom-left: the arms run right and up.
        let bl = bracket(1);
        assert!(alpha(&bl, 18, 10) > 200);
        assert!(alpha(&bl, 9, 1) > 200);
        assert_eq!(alpha(&bl, 16, 3), 0);
    }
}
