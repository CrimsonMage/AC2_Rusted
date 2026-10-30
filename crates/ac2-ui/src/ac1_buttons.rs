//! The modern HUD's Social and Skills buttons in AC1's icons (a modern addition, AGENTS.md: the
//! HUD's two styles). AC1's toolbar drew a green symbol (gold when highlighted) in a dark ring;
//! AC2's panel buttons are bronze orbs with a light metal symbol. Each AC2 orb is remade in AC2's
//! look around AC1's symbol: the orb's own bronze kept, its symbol filled in from the bronze at the
//! same distance from the centre (lit from above as the art is),
//! then AC1's symbol, scaled from inside its ring to the orb's face, painted in the orb symbols'
//! ivory with a dark rim. The symbol's shape is the green of AC1's normal image; the orb's
//! normal image takes it on the orb, its rollover image on a brighter orb. The images replace the orbs' own under their DIDs, so
//! the buttons' states, messages and tooltips are untouched.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_assets::ac1_icons::{Ac1Image, Ac1ToolbarIcons};

use crate::compose::UiPixels;

/// `SkillButton`'s orb (normal, rollover) and `SocialButton`'s.
pub const SKILL_ORB: [u32; 2] = [0x4100_09CA, 0x4100_09CB];
pub const SOCIAL_ORB: [u32; 2] = [0x4100_1E84, 0x4100_1E85];

/// The radius of the orb's face that the symbol fills, and of the AC1 ring's inside it is taken
/// from (both images' symbols sit in about this much of their disc).
const FACE_RADIUS: f32 = 9.5;
const AC1_RADIUS: f32 = 8.5;
/// The AC1 ring's centre row (its symbol is not centred on the image: a strip of the bar sits
/// above the ring).
const AC1_CENTRE_Y: f32 = 14.0;
/// The symbols' ivory, and the rollover orb's extra light.
const IVORY: [f32; 3] = [238.0, 226.0, 196.0];
const ROLLOVER_LIGHT: f32 = 1.18;

/// Both buttons' orbs remade from `icons` in `images` (the orbs' normal images must be loaded:
/// they are the art the remade ones are cut from). Returns how many orbs were remade.
pub fn bake(images: &mut HashMap<u32, UiPixels>, icons: &Ac1ToolbarIcons) -> usize {
    let mut baked = 0;
    for (orb, symbols) in [(SKILL_ORB, &icons.skills), (SOCIAL_ORB, &icons.social)] {
        let Some(base) = images.get(&orb[0]).cloned() else {
            continue;
        };
        // The symbol's shape is the normal image's green; both orbs take it.
        images.insert(orb[0], remade(&base, &symbols[0], 1.0));
        images.insert(orb[1], remade(&base, &symbols[0], ROLLOVER_LIGHT));
        baked += 1;
    }
    baked
}

/// How much of a pixel of AC1's normal image is symbol: the symbol is green, the ring bronze and
/// the background dark.
fn symbol_cover(p: [f32; 3]) -> f32 {
    // Only the bright green is the symbol; the darker green of its folds stays the orb's, so
    // the symbol keeps its inner lines.
    let bright = ((p[1] - 120.0) / 70.0).clamp(0.0, 1.0);
    let green = ((p[1] - p[0].max(p[2]) - 20.0) / 40.0).clamp(0.0, 1.0);
    bright * green
}

/// How much of an orb pixel is its bronze, not its symbol's metal or gold.
fn bronze(p: [f32; 3]) -> f32 {
    // Warm and orange: a symbol's yellow (green as high as red) or metal grey is not bronze.
    let warm = ((p[0] - p[2] - 30.0) / 60.0).clamp(0.0, 1.0);
    let orange = ((p[0] - p[1] - 12.0) / 30.0).clamp(0.0, 1.0);
    warm * orange
}

/// The AC1 image's colour at `(x, y)` (bilinear, clamped to the image).
fn sample(image: &Ac1Image, x: f32, y: f32) -> [f32; 3] {
    let (w, h) = (image.width as i32, image.height as i32);
    let at = |x: i32, y: i32| {
        let i = ((y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) * 4) as usize;
        [
            f32::from(image.rgba[i]),
            f32::from(image.rgba[i + 1]),
            f32::from(image.rgba[i + 2]),
        ]
    };
    let (x0, y0) = (x.floor() as i32, y.floor() as i32);
    let (tx, ty) = (x - x0 as f32, y - y0 as f32);
    let mut out = [0.0; 3];
    for (dx, dy, w) in [
        (0, 0, (1.0 - tx) * (1.0 - ty)),
        (1, 0, tx * (1.0 - ty)),
        (0, 1, (1.0 - tx) * ty),
        (1, 1, tx * ty),
    ] {
        let p = at(x0 + dx, y0 + dy);
        for c in 0..3 {
            out[c] += p[c] * w;
        }
    }
    out
}

/// The orb `base` remade around `symbol`'s symbol, its light scaled by `light`.
pub fn remade(base: &UiPixels, symbol: &Ac1Image, light: f32) -> UiPixels {
    let (w, h) = (base.width as usize, base.height as usize);
    let alpha = |x: usize, y: usize| base.rgba[(y * w + x) * 4 + 3];
    // The orb's centre: the middle of its opaque pixels.
    let (mut sx, mut sy, mut n) = (0.0f32, 0.0f32, 0.0f32);
    for y in 0..h {
        for x in 0..w {
            if alpha(x, y) > 127 {
                (sx, sy, n) = (sx + x as f32, sy + y as f32, n + 1.0);
            }
        }
    }
    if n == 0.0 {
        return base.clone();
    }
    let (cx, cy) = (sx / n, sy / n);
    let dist = |x: usize, y: usize| ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
    // The orb's mean colour at each whole distance from its centre, its symbol averaged away.
    let rings = (dist(0, 0).max(dist(w - 1, h - 1)) as usize) + 2;
    let mut sum = vec![[0.0f32; 4]; rings];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            let a = f32::from(base.rgba[i + 3]) / 255.0;
            let p = [0, 1, 2].map(|c| f32::from(base.rgba[i + c]));
            let weight = a * (bronze(p) + 1e-3);
            let ring = &mut sum[dist(x, y) as usize];
            for c in 0..3 {
                ring[c] += p[c] * weight;
            }
            ring[3] += weight;
        }
    }
    // The symbol's cover over the face, then its rim one pixel out.
    let ac1_cx = (symbol.width as f32 - 1.0) / 2.0;
    let scale = AC1_RADIUS / FACE_RADIUS;
    let mut cover = vec![0.0f32; w * h];
    let mut shade = vec![[0.0f32; 3]; w * h];
    for y in 0..h {
        for x in 0..w {
            if dist(x, y) > FACE_RADIUS {
                continue;
            }
            let p = sample(
                symbol,
                ac1_cx + (x as f32 - cx) * scale,
                AC1_CENTRE_Y + (y as f32 - cy) * scale,
            );
            cover[y * w + x] = symbol_cover(p);
            // The green's own shading, as the metal's.
            let tone = (0.55 + p[1] / 255.0 * 0.6).min(1.1);
            shade[y * w + x] = IVORY.map(|c| c * tone);
        }
    }
    let mut rgba = base.rgba.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            if rgba[i + 3] == 0 {
                continue;
            }
            // The orb's own bronze where it is bronze; where its symbol was, the bronze of its
            // ring, lit from above as the art is.
            let ring = sum[dist(x, y) as usize];
            let own_colour = [0, 1, 2].map(|c| f32::from(base.rgba[i + c]));
            let keep = bronze(own_colour);
            let lit = 1.12 - 0.24 * y as f32 / h as f32;
            let mut colour = [0.0f32; 3];
            for c in 0..3 {
                let filled = ring[c] / ring[3].max(1e-3) * lit;
                colour[c] = (filled + (own_colour[c] - filled) * keep) * light;
            }
            // The rim: the most a neighbour is covered beyond this pixel's own cover.
            let mut rim = 0.0f32;
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1), (1, 1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                    rim = rim.max(cover[ny as usize * w + nx as usize]);
                }
            }
            let own = cover[y * w + x];
            let rim = (rim - own).max(0.0) * 0.55;
            for c in 0..3 {
                let dark = colour[c] * (1.0 - rim);
                let value = dark + (shade[y * w + x][c] - dark) * own;
                rgba[i + c] = value.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    UiPixels {
        width: base.width,
        height: base.height,
        rgba: Arc::from(rgba),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain bronze disc and an AC1 image with a bright green square in the middle of its ring.
    fn fixtures() -> (UiPixels, Ac1Image) {
        let (w, h) = (27usize, 28usize);
        let mut base = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let d = ((x as f32 - 13.0).powi(2) + (y as f32 - 13.0).powi(2)).sqrt();
                if d <= 12.5 {
                    base[(y * w + x) * 4..][..4].copy_from_slice(&[150, 110, 50, 255]);
                }
            }
        }
        let (sw, sh) = (35usize, 27usize);
        let mut symbol = vec![0u8; sw * sh * 4];
        for y in 0..sh {
            for x in 0..sw {
                let green = (14..=20).contains(&x) && (11..=17).contains(&y);
                let p = if green {
                    [0, 200, 40, 255]
                } else {
                    [70, 60, 40, 255]
                };
                symbol[(y * sw + x) * 4..][..4].copy_from_slice(&p);
            }
        }
        (
            UiPixels {
                width: w as u32,
                height: h as u32,
                rgba: Arc::from(base),
            },
            Ac1Image {
                width: sw as u32,
                height: sh as u32,
                rgba: symbol,
            },
        )
    }

    #[test]
    fn the_symbol_takes_the_orbs_ivory_and_the_outside_stays_clear() {
        let (base, symbol) = fixtures();
        let orb = remade(&base, &symbol, 1.0);
        let at = |x: usize, y: usize| {
            let i = (y * 27 + x) * 4;
            [
                orb.rgba[i],
                orb.rgba[i + 1],
                orb.rgba[i + 2],
                orb.rgba[i + 3],
            ]
        };
        let centre = at(13, 13);
        assert!(
            centre[0] > 180 && centre[2] > 120,
            "ivory at the centre: {centre:?}"
        );
        assert_eq!(at(0, 0)[3], 0, "the corner stays clear");
        let edge = at(13, 2);
        assert!(edge[0] > edge[2], "bronze at the edge: {edge:?}");
    }

    #[test]
    fn the_rollover_orb_is_lighter_and_both_orbs_are_baked_in_place() {
        let (base, symbol) = fixtures();
        let normal = remade(&base, &symbol, 1.0);
        let rollover = remade(&base, &symbol, ROLLOVER_LIGHT);
        let i = (3 * 27 + 13) * 4;
        assert!(rollover.rgba[i] >= normal.rgba[i]);
        let mut images = HashMap::from([(SKILL_ORB[0], base.clone()), (SOCIAL_ORB[0], base)]);
        let icons = Ac1ToolbarIcons {
            social: [symbol.clone(), symbol.clone()],
            skills: [symbol.clone(), symbol],
        };
        assert_eq!(bake(&mut images, &icons), 2);
        assert!(images.contains_key(&SKILL_ORB[1]) && images.contains_key(&SOCIAL_ORB[1]));
    }
}
