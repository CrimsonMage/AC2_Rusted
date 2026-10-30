//! The modern radar's art (`modern_radar`): the disc with its rings and the player's arrow, the
//! heading ring's letters, a button's round well, the coordinates' pill and the range chip, drawn
//! by `modern_art`'s painter under the client's own DIDs (`0x7E0100Bx`).

use std::collections::HashMap;

use crate::compose::UiPixels;
use crate::modern_art::{band, circle, inside, mix, rgb, round_rect, Canvas};
use crate::modern_art_drawer::{stroke_distance, Stroke};

pub const RADAR_DISC: u32 = 0x7E01_00B0;
pub const HEADING_RING: u32 = 0x7E01_00B1;
pub const RADAR_WELL: u32 = 0x7E01_00B2;
pub const RADAR_WELL_LIT: u32 = 0x7E01_00B3;
pub const COORD_PILL: u32 = 0x7E01_00B4;
pub const RANGE_CHIP: u32 = 0x7E01_00B5;
/// Nothing: the retail ring's image, gone.
pub const RADAR_CLEAR: u32 = 0x7E01_00B6;
/// The radar's geometry in the design: the disc's radius, its centre in the radar's box, the
/// box's size, the heading letters' radius.
pub const RADAR_RADIUS: i32 = 46;
pub const RADAR_CENTER: (i32, i32) = (66, 66);
pub const RADAR_BOX: (i32, i32) = (132, 150);
pub const LETTER_RADIUS: f32 = 59.0;
/// The disc's art: the outer ring's radius and a pixel of room.
pub const DISC_SIZE: i32 = 2 * (RADAR_RADIUS + 7);
pub const RADAR_WELL_SIZE: i32 = 20;
pub const PILL_BORDER: i32 = 9;
pub const RANGE_CHIP_BORDER: i32 = 6;

/// The disc: dark teal glass (`#02201d` at 72%) in a black line, the stone ring (`#a4a192`, 2
/// pixels), a dark outer ring, a faint inner ring, and the player's green arrow at the centre.
fn disc() -> UiPixels {
    let side = DISC_SIZE as usize;
    let mut c = Canvas::new(side, side);
    let m = DISC_SIZE as f32 / 2.0;
    let r = RADAR_RADIUS as f32;
    let disc = circle(m, m, r + 4.0);
    c.paint(&disc, |_, _, d| (rgb(0x02201D, 0.72), inside(d)));
    c.paint(&disc, |_, _, d| (rgb(0x0A0A08, 1.0), band(d, -1.0, 0.0)));
    let ring = |radius: f32, width: f32| {
        move |x: f32, y: f32| ((x - m).powi(2) + (y - m).powi(2)).sqrt() - radius - width / 2.0
    };
    c.paint(ring(r + 3.0, 2.0), |_, _, d| {
        (rgb(0xA4A192, 1.0), band(d, -2.0, 0.0))
    });
    c.paint(ring(r + 5.5, 1.2), |_, _, d| {
        (rgb(0x2A261C, 1.0), band(d, -1.2, 0.0))
    });
    c.paint(ring(r, 1.0), |_, _, d| {
        (rgb(0x8B846C, 0.25), band(d, -1.0, 0.0))
    });
    // The arrow (`M0 -5l4 7-4-2-4 2z`), green in a dark line.
    let arrow = |x: f32, y: f32| {
        let (x, y) = (x - m, y - m);
        // Inside the triangle (0,-5) (4,2) (-4,2), less the notch to (0,0).
        let edge = |ax: f32, ay: f32, bx: f32, by: f32| {
            ((bx - ax) * (y - ay) - (by - ay) * (x - ax)) / ((bx - ax).hypot(by - ay))
        };
        [
            edge(0.0, -5.0, 4.0, 2.0),
            edge(4.0, 2.0, 0.0, 0.0),
            edge(0.0, 0.0, -4.0, 2.0),
            edge(-4.0, 2.0, 0.0, -5.0),
        ]
        .into_iter()
        .fold(f32::MIN, f32::max)
    };
    c.paint(arrow, |_, _, d| (rgb(0x8FE08A, 1.0), inside(d)));
    c.paint(arrow, |_, _, d| (rgb(0x021412, 1.0), band(d, -0.35, 0.35)));
    c.finish()
}

/// The letters' strokes in a unit box (x and y 0..1, y down).
const LETTER_N: [Stroke; 1] = [Stroke::Line(&[
    (0.0, 1.0),
    (0.0, 0.0),
    (1.0, 1.0),
    (1.0, 0.0),
])];
const LETTER_E: [Stroke; 2] = [
    Stroke::Line(&[(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (1.0, 1.0)]),
    Stroke::Line(&[(0.0, 0.5), (0.8, 0.5)]),
];
const LETTER_S: [Stroke; 1] = [Stroke::Line(&[
    (1.0, 0.12),
    (0.75, 0.0),
    (0.25, 0.0),
    (0.0, 0.18),
    (0.05, 0.38),
    (0.95, 0.62),
    (1.0, 0.82),
    (0.75, 1.0),
    (0.25, 1.0),
    (0.0, 0.88),
])];
const LETTER_W: [Stroke; 1] = [Stroke::Line(&[
    (0.0, 0.0),
    (0.25, 1.0),
    (0.5, 0.35),
    (0.75, 1.0),
    (1.0, 0.0),
])];

/// The heading ring: N (gold, 12 pixels), E, S and W (cream, 9) at `LETTER_RADIUS` around the
/// centre of a box the radar's width square, N at the top; the ring turns with the player.
fn heading_ring() -> UiPixels {
    let side = RADAR_BOX.0 as usize;
    let mut c = Canvas::new(side, side);
    let m = RADAR_BOX.0 as f32 / 2.0;
    for (strokes, angle, size, color) in [
        (&LETTER_N[..], 0.0f32, 10.0, 0xF6E27A),
        (&LETTER_E[..], 90.0, 7.0, 0xCDBF92),
        (&LETTER_S[..], 180.0, 7.0, 0xCDBF92),
        (&LETTER_W[..], 270.0, 7.5, 0xCDBF92),
    ] {
        let a = angle.to_radians();
        let (cx, cy) = (m + LETTER_RADIUS * a.sin(), m - LETTER_RADIUS * a.cos());
        let w = size * if angle == 270.0 { 0.9 } else { 0.62 };
        let (x0, y0) = (cx - w / 2.0, cy - size / 2.0);
        let distance = move |x: f32, y: f32| {
            // In the unit box, scaled back to pixels by the smaller side.
            stroke_distance(strokes, (x - x0) / w, (y - y0) / size) * w.min(size)
        };
        // A dark outline, then the letter.
        c.paint(distance, |_, _, d| (rgb(0x000000, 0.8), inside(d - 1.4)));
        c.paint(distance, move |_, _, d| (rgb(color, 1.0), inside(d - 0.7)));
    }
    c.finish()
}

/// A radar button's round well: teal glass (`#124a42` to `#031a17`) in a bronze line, gold and
/// glowing when `lit`.
fn radar_well(lit: bool) -> UiPixels {
    let side = RADAR_WELL_SIZE as usize;
    let mut c = Canvas::new(side, side);
    let half = RADAR_WELL_SIZE as f32 / 2.0;
    let body = circle(half, half, half - 1.0);
    c.paint(&body, |x, y, d| {
        let t = (((x - half).powi(2) + (y - half).powi(2)).sqrt() / half).min(1.0);
        (mix(rgb(0x124A42, 1.0), rgb(0x031A17, 1.0), t), inside(d))
    });
    let line = if lit {
        rgb(0xD9C27A, 1.0)
    } else {
        rgb(0x8B846C, 1.0)
    };
    c.paint(&body, |_, _, d| (line, band(d, -1.0, 0.0)));
    c.paint(&body, |_, _, d| (rgb(0x000000, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// The coordinates' pill: teal glass fading down (`#001f1dd0` to `#001f1d90`) in a bronze line
/// and a black rim, its ends round.
fn coord_pill() -> UiPixels {
    let (w, h) = (24usize, 18usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(1.0, 1.0, w as f32 - 1.0, h as f32 - 1.0, 8.0);
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x001F1D, 0.82), rgb(0x001F1D, 0.56), y / h as f32),
            inside(d),
        )
    });
    c.paint(&shape, |_, _, d| (rgb(0x8B846C, 1.0), band(d, -1.0, 0.0)));
    c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// The range chip: dark glass (`#000a`), corners 6.
fn range_chip() -> UiPixels {
    let (w, h) = (14usize, 12usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 6.0);
    c.paint(&shape, |_, _, d| (rgb(0x000000, 0.67), inside(d)));
    c.finish()
}

/// The radar's images made into `images`.
pub(crate) fn bake(images: &mut HashMap<u32, UiPixels>) {
    for (id, image) in [
        (RADAR_DISC, disc()),
        (HEADING_RING, heading_ring()),
        (RADAR_WELL, radar_well(false)),
        (RADAR_WELL_LIT, radar_well(true)),
        (COORD_PILL, coord_pill()),
        (RANGE_CHIP, range_chip()),
        (RADAR_CLEAR, Canvas::new(1, 1).finish()),
    ] {
        images.insert(id, image);
    }
}
