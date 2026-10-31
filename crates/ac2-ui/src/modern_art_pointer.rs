//! The modern style's pointing arrows (a modern addition, AGENTS.md: the HUD's two styles): the
//! chevron disc the selection's off-screen indicator becomes, and the delta the floating arrow
//! (the Tracker's, or a map coordinate's) becomes. Retail turned one small bitmap
//! (`OffScreenIndicator`'s art) by its surface object's rotation, sampled nearest, so it showed
//! jagged at most angles. Here each arrow is drawn by the client already turned, anti-aliased, at
//! every 5 degrees, under its own DIDs, and the element shows the drawing for its angle unturned.

use std::collections::HashMap;

use crate::compose::UiPixels;
use crate::element::UiTree;
use crate::modern_art::{band, circle, glow, inside, mix, rgb, Canvas};

/// The drawings per arrow: one every 5 degrees.
pub const POINTER_STEPS: u32 = 72;
/// The chevron disc at each step (`DISC_POINTER + step`).
pub const DISC_POINTER: u32 = 0x7E01_0200;
/// The delta at each step (`DELTA_POINTER + step`).
pub const DELTA_POINTER: u32 = 0x7E01_0300;
/// `OffScreenIndicator`'s size, which the drawings keep.
const SIZE: usize = 64;
const MID: f32 = SIZE as f32 / 2.0;

/// The step drawing `rotation` (degrees anticlockwise from pointing right, as the indicator turns).
pub fn pointer_step(rotation: i32) -> u32 {
    ((rotation.rem_euclid(360) as u32 + 2) / 5) % POINTER_STEPS
}

/// Shows `base`'s drawing for `rotation` on `index`, unturned; nothing changes when it already
/// shows it.
pub(crate) fn point(tree: &mut UiTree, index: usize, base: u32, rotation: i32) {
    tree.set_object_rotation(index, 0);
    let file = base + pointer_step(rotation);
    if tree.get(index).and_then(|e| e.image()).map(|i| i.file) != Some(file) {
        tree.set_image(index, file, crate::compose::DRAW_OVER);
    }
}

/// A point's place along the arrow (`along`, towards its tip) and across it (`side`) for an arrow
/// turned `degrees` anticlockwise from pointing right, about the drawing's middle.
fn frame(degrees: f32) -> impl Fn(f32, f32) -> (f32, f32) {
    let (s, c) = degrees.to_radians().sin_cos();
    move |x, y| {
        let (dx, dy) = (x - MID, y - MID);
        (dx * c - dy * s, dx * s + dy * c)
    }
}

/// The distance from `p` to the segment `a`..`b`.
fn segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let t = (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
    ((p.0 - a.0 - vx * t).powi(2) + (p.1 - a.1 - vy * t).powi(2)).sqrt()
}

/// The signed distance from `p` to a closed polygon (negative inside, by the even-odd rule).
fn polygon(p: (f32, f32), points: &[(f32, f32)]) -> f32 {
    let mut d = f32::MAX;
    let mut inside = false;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        d = d.min(segment(p, a, b));
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) / (b.1 - a.1) * (b.0 - a.0) {
            inside = !inside;
        }
    }
    if inside {
        -d
    } else {
        d
    }
}

/// The chevron disc turned `degrees`: dark glass in a gold ring, a double chevron pointing out.
pub fn disc(degrees: f32) -> UiPixels {
    let mut c = Canvas::new(SIZE, SIZE);
    let disc = circle(MID, MID, 15.0);
    c.paint(&disc, |_, _, d| (rgb(0x000000, 0.45), glow(d, 3.0)));
    c.paint(&disc, |_, _, d| (rgb(0x061613, 0.88), inside(d)));
    c.paint(&disc, |_, _, d| (rgb(0xD9C27A, 1.0), band(d, 0.0, 1.6)));
    c.paint(circle(MID, MID, 12.5), |_, _, d| {
        (rgb(0xD9C27A, 0.25), (1.0 - d.abs()).clamp(0.0, 1.0))
    });
    let local = frame(degrees);
    // Each chevron's tip along the arrow and its arms 6 back and 6 aside; the back one fainter.
    for (tip, alpha) in [(7.0, 1.0), (1.0, 0.6)] {
        c.paint(
            |x, y| {
                let p = local(x, y);
                let (a, b, e) = ((tip - 6.0, -6.0), (tip, 0.0), (tip - 6.0, 6.0));
                segment(p, a, b).min(segment(p, b, e)) - 1.2
            },
            |_, _, d| (rgb(0xF1DD96, alpha), inside(d)),
        );
    }
    c.finish()
}

/// The delta turned `degrees`: a notched gold arrowhead in a dark line, a short fading trail
/// behind it.
pub fn delta(degrees: f32) -> UiPixels {
    let mut c = Canvas::new(SIZE, SIZE);
    let local = frame(degrees);
    c.paint(
        |x, y| segment(local(x, y), (-6.0, 0.0), (-28.0, 0.0)) - 2.0,
        |x, y, d| {
            let fall = ((local(x, y).0 + 28.0) / 22.0).clamp(0.0, 1.0);
            (rgb(0xD9C27A, 0.8 * fall), inside(d))
        },
    );
    let head = [(14.0, 0.0), (-9.0, 11.0), (-3.0, 0.0), (-9.0, -11.0)];
    let shape = |x: f32, y: f32| polygon(local(x, y), &head);
    c.paint(shape, |_, _, d| (rgb(0x000000, 0.7), inside(d - 2.2)));
    c.paint(shape, |x, y, d| {
        // Light at the tip to bronze at the notch.
        let t = ((14.0 - local(x, y).0) / 23.0).clamp(0.0, 1.0);
        let gold = if t < 0.45 {
            mix(rgb(0xFFF0BF, 1.0), rgb(0xD9C27A, 1.0), t / 0.45)
        } else {
            mix(rgb(0xD9C27A, 1.0), rgb(0x8A6A2E, 1.0), (t - 0.45) / 0.55)
        };
        (gold, inside(d))
    });
    c.paint(shape, |_, _, d| (rgb(0xFFF0BF, 0.7), band(d, 0.0, 0.8)));
    c.finish()
}

/// Every drawing of both arrows into `images`.
pub fn bake(images: &mut HashMap<u32, UiPixels>) {
    for step in 0..POINTER_STEPS {
        let degrees = (step * 5) as f32;
        images.insert(DISC_POINTER + step, disc(degrees));
        images.insert(DELTA_POINTER + step, delta(degrees));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(image: &UiPixels, x: usize, y: usize) -> u8 {
        image.rgba[(y * image.width as usize + x) * 4 + 3]
    }

    #[test]
    fn a_rotation_takes_its_nearest_step() {
        assert_eq!(pointer_step(0), 0);
        assert_eq!(pointer_step(2), 0);
        assert_eq!(pointer_step(3), 1);
        assert_eq!(pointer_step(90), 18);
        assert_eq!(pointer_step(359), 0);
        assert_eq!(pointer_step(-45), 63);
        assert_eq!(pointer_step(315), 63);
    }

    #[test]
    fn the_delta_points_the_way_it_is_turned() {
        // Right: its tip right of the middle, nothing above it.
        let right = delta(0.0);
        assert!(alpha(&right, 32 + 12, 32) > 200);
        assert_eq!(alpha(&right, 32, 32 - 13), 0);
        // Up (anticlockwise 90): the tip above, nothing at the right.
        let up = delta(90.0);
        assert!(alpha(&up, 32, 32 - 12) > 200);
        assert_eq!(alpha(&up, 32 + 13, 32), 0);
        // Its trail runs behind it, fading.
        assert!(alpha(&up, 32, 32 + 12) > alpha(&up, 32, 32 + 26));
    }

    #[test]
    fn the_disc_is_round_and_its_chevrons_turn() {
        let right = disc(0.0);
        let up = disc(90.0);
        // The ring is there whatever the turn; outside it, nothing but the shadow's fringe.
        for image in [&right, &up] {
            assert!(alpha(image, 32 + 14, 32) > 200);
            assert!(alpha(image, 2, 2) < 8);
        }
        // The front chevron's tip: right of the middle unturned (nothing as far to the left),
        // above it turned up.
        let lit =
            |image: &UiPixels, x: usize, y: usize| image.rgba[(y * image.width as usize + x) * 4];
        assert!(lit(&right, 32 + 7, 32) > lit(&right, 32 - 7, 32));
        assert!(lit(&up, 32, 32 - 7) > lit(&up, 32, 32 + 7));
    }
}
