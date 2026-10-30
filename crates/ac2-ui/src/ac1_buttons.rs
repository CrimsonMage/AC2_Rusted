//! The modern HUD's panel button icons (a modern addition, AGENTS.md: the HUD's two styles): one
//! matching set in Asheron's Call 1's look, the user's choice. Each of the seven panels takes AC1's
//! own toolbar symbol (Examination its magnifier, Navigation its compass, Option its checklist,
//! Inventory its backpack, Quest its leaf, Skill its flexed arm, Social its figures), drawn in
//! AC2's colours: the same bronze orb for all (AC2's Skill orb with its anvil filled out), the
//! symbol in the orb symbols' ivory with a dark rim, scaled to fit the orb's face. The rollover is
//! the same on a brighter orb.
//!
//! The icons ship as images with the client (`assets/hud/`, [`shipped_icons`]); this module is
//! the maths that makes them, run by the `bake_panel_icons` example over the player's AC2 and
//! AC1 DATs.

use std::collections::HashMap;
use std::sync::Arc;

use crate::compose::UiPixels;

/// The panel buttons' orbs (normal, rollover), in [`crate::ToolbarChild::ALL`]'s order: their DAT
/// images, which the shipped icons replace.
pub const PANEL_ORBS: [[u32; 2]; 7] = [
    [0x4100_09D0, 0x4100_09D1],
    [0x4100_09CE, 0x4100_09CF],
    [0x4100_2526, 0x4100_2527],
    [0x4100_09CC, 0x4100_09CD],
    [0x4100_1E54, 0x4100_1E55],
    [0x4100_09CA, 0x4100_09CB],
    [0x4100_1E84, 0x4100_1E85],
];
/// The orb every icon is drawn on: AC2's Skill orb.
pub const BASE_ORB: u32 = 0x4100_09CA;

/// How an AC1 symbol stands apart from its image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A green symbol inside AC1's dark ring.
    Ring,
    /// A bright symbol on dark ground, without a ring.
    Dark,
}

/// The part of an image (x0, y0, x1, y1, exclusive).
pub type Area = (u32, u32, u32, u32);

/// Each panel's AC1 symbol: its file name stem, its image, how it stands apart, and the part of the
/// image it is in (the rest is AC1's bar or tab art).
pub const PANEL_SYMBOLS: [(&str, u32, SymbolKind, Option<Area>); 7] = [
    (
        "examination",
        0x0600_1127,
        SymbolKind::Dark,
        Some((0, 0, 22, 26)),
    ),
    ("navigation", 0x0600_1116, SymbolKind::Ring, None),
    ("option", 0x0600_111C, SymbolKind::Ring, None),
    (
        "inventory",
        0x0600_4CF7,
        SymbolKind::Dark,
        Some((2, 3, 54, 56)),
    ),
    ("quest", 0x0600_69AE, SymbolKind::Ring, None),
    ("skill", 0x0600_1122, SymbolKind::Ring, None),
    ("social", 0x0600_111F, SymbolKind::Ring, None),
];

/// The face's radius the symbol is scaled to fill, the ivory, the rollover's light.
const FACE: f32 = 8.6;
const IVORY: [f32; 3] = [238.0, 226.0, 196.0];
pub const ROLLOVER_LIGHT: f32 = 1.3;

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// An RGBA image as plain pixels (rows top-down).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Rgba {
    fn at(&self, x: u32, y: u32) -> [f32; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [0, 1, 2, 3].map(|c| f32::from(self.rgba[i + c]))
    }

    /// The part `area` of the image.
    pub fn crop(&self, (x0, y0, x1, y1): Area) -> Self {
        let (x1, y1) = (x1.min(self.width), y1.min(self.height));
        let mut rgba = Vec::with_capacity(((x1 - x0) * (y1 - y0) * 4) as usize);
        for y in y0..y1 {
            let row = ((y * self.width + x0) * 4) as usize;
            rgba.extend_from_slice(&self.rgba[row..row + ((x1 - x0) * 4) as usize]);
        }
        Self {
            width: x1 - x0,
            height: y1 - y0,
            rgba,
        }
    }
}

/// How much of an orb pixel is its bronze, not its symbol's metal or gold.
fn bronze(p: [f32; 4]) -> f32 {
    clamp01((p[0] - p[2] - 30.0) / 60.0) * clamp01((p[0] - p[1] - 12.0) / 30.0)
}

/// How much of a symbol image's pixel is symbol.
fn cover(kind: SymbolKind, p: [f32; 4]) -> f32 {
    match kind {
        SymbolKind::Ring => {
            clamp01((p[1] - 120.0) / 70.0) * clamp01((p[1] - p[0].max(p[2]) - 20.0) / 40.0)
        }
        SymbolKind::Dark => {
            let lum = 0.3 * p[0] + 0.59 * p[1] + 0.11 * p[2];
            clamp01((lum - 45.0) / 45.0) * p[3] / 255.0
        }
    }
}

/// The orb with its symbol filled out: each pixel the orb's bronze at its distance from the
/// centre (the centre's rings, mostly symbol, from the first ring out that is mostly bronze), lit
/// from above as the art is. Returns the orb and its centre.
fn clean_orb(base: &Rgba) -> (Vec<[f32; 4]>, (f32, f32)) {
    let (w, h) = (base.width, base.height);
    let (mut sx, mut sy, mut n) = (0.0, 0.0, 0.0);
    for y in 0..h {
        for x in 0..w {
            if base.at(x, y)[3] > 127.0 {
                (sx, sy, n) = (sx + x as f32, sy + y as f32, n + 1.0);
            }
        }
    }
    let (cx, cy) = (sx / f32::max(n, 1.0), sy / f32::max(n, 1.0));
    let dist = |x: u32, y: u32| ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
    // Per whole distance: the bronze-weighted colour and weight, and the pixels' alpha.
    let mut rings = vec![[0.0f32; 5]; (w.max(h) as usize) + 2];
    for y in 0..h {
        for x in 0..w {
            let p = base.at(x, y);
            let a = p[3] / 255.0;
            let weight = a * (bronze(p) + 1e-3);
            let ring = &mut rings[dist(x, y) as usize];
            for c in 0..3 {
                ring[c] += p[c] * weight;
            }
            ring[3] += weight;
            ring[4] += a;
        }
    }
    let mut out = vec![[0.0f32; 4]; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let p = base.at(x, y);
            if p[3] == 0.0 {
                continue;
            }
            let mut d = dist(x, y) as usize;
            while d < 8 && rings[d][3] < 0.5 * rings[d][4] {
                d += 1;
            }
            let ring = rings[d];
            let from_glint =
                ((x as f32 - (cx - 4.0)).powi(2) + (y as f32 - (cy - 5.0)).powi(2)).sqrt();
            let glint = 1.0 + 0.18 * clamp01(1.0 - from_glint / 12.0);
            let lit = (1.12 - 0.24 * y as f32 / h as f32) * glint;
            let mut c = [0.0; 4];
            for (i, value) in c.iter_mut().take(3).enumerate() {
                *value = (ring[i] / ring[3].max(1e-3) * lit).min(255.0);
            }
            c[3] = p[3];
            out[(y * w + x) as usize] = c;
        }
    }
    (out, (cx, cy))
}

/// The symbol's cover at `(x, y)` in its image, bilinear and clamped to the image.
fn sample(symbol: &Rgba, kind: SymbolKind, x: f32, y: f32) -> f32 {
    let (w, h) = (symbol.width as i32, symbol.height as i32);
    let (x0, y0) = (x.floor() as i32, y.floor() as i32);
    let (tx, ty) = (x - x0 as f32, y - y0 as f32);
    let mut sum = 0.0;
    for (dx, dy, wt) in [
        (0, 0, (1.0 - tx) * (1.0 - ty)),
        (1, 0, tx * (1.0 - ty)),
        (0, 1, (1.0 - tx) * ty),
        (1, 1, tx * ty),
    ] {
        let (sx, sy) = ((x0 + dx).clamp(0, w - 1), (y0 + dy).clamp(0, h - 1));
        sum += cover(kind, symbol.at(sx as u32, sy as u32)) * wt;
    }
    sum
}

/// The symbol's bounds: the box of its pixels well covered.
fn bounds(symbol: &Rgba, kind: SymbolKind) -> Option<(f32, f32, f32, f32)> {
    let mut b: Option<Area> = None;
    for y in 0..symbol.height {
        for x in 0..symbol.width {
            if cover(kind, symbol.at(x, y)) > 0.35 {
                b = Some(match b {
                    None => (x, y, x, y),
                    Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                });
            }
        }
    }
    b.map(|(x0, y0, x1, y1)| (x0 as f32, y0 as f32, x1 as f32, y1 as f32))
}

/// One icon: `symbol` (an AC1 image, standing apart as `kind`) on the clean `base` orb, the orb's
/// light times `light`.
pub fn icon(base: &Rgba, symbol: &Rgba, kind: SymbolKind, light: f32) -> Rgba {
    let (w, h) = (base.width, base.height);
    let (orb, (cx, cy)) = clean_orb(base);
    let Some((x0, y0, x1, y1)) = bounds(symbol, kind) else {
        return base.clone();
    };
    // The symbol's larger half-extent fills the face, so the whole symbol fits inside it.
    let (scx, scy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let scale = ((x1 - x0).max(y1 - y0) / 2.0 + 0.5) / FACE;
    let mut covered = vec![0.0f32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            if d <= FACE + 1.5 {
                covered[(y * w + x) as usize] = sample(
                    symbol,
                    kind,
                    scx + (x as f32 - cx) * scale,
                    scy + (y as f32 - cy) * scale,
                );
            }
        }
    }
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let c = orb[i];
            if c[3] == 0.0 {
                continue;
            }
            let own = covered[i];
            let mut rim = 0.0f32;
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1), (1, 1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h {
                    rim = rim.max(covered[(ny as u32 * w + nx as u32) as usize]);
                }
            }
            let rim = (rim - own).max(0.0) * 0.6;
            let tone = 1.0 - 0.18 * y as f32 / h as f32;
            for k in 0..3 {
                let dark = c[k] * light * (1.0 - rim);
                let v = dark + (IVORY[k] * tone - dark) * own;
                rgba[i * 4 + k] = v.round().clamp(0.0, 255.0) as u8;
            }
            rgba[i * 4 + 3] = c[3] as u8;
        }
    }
    Rgba {
        width: w,
        height: h,
        rgba,
    }
}

/// The shipped icons' file names, per panel: normal and rollover.
pub fn icon_file_names(panel: &str) -> [String; 2] {
    [
        format!("panel_{panel}.png"),
        format!("panel_{panel}_hover.png"),
    ]
}

/// The icons shipped with the client (`assets/hud/`), per panel in [`crate::ToolbarChild::ALL`]'s
/// order: normal and rollover, PNG.
const SHIPPED: [[&[u8]; 2]; 7] = [
    [
        include_bytes!("../assets/hud/panel_examination.png"),
        include_bytes!("../assets/hud/panel_examination_hover.png"),
    ],
    [
        include_bytes!("../assets/hud/panel_navigation.png"),
        include_bytes!("../assets/hud/panel_navigation_hover.png"),
    ],
    [
        include_bytes!("../assets/hud/panel_option.png"),
        include_bytes!("../assets/hud/panel_option_hover.png"),
    ],
    [
        include_bytes!("../assets/hud/panel_inventory.png"),
        include_bytes!("../assets/hud/panel_inventory_hover.png"),
    ],
    [
        include_bytes!("../assets/hud/panel_quest.png"),
        include_bytes!("../assets/hud/panel_quest_hover.png"),
    ],
    [
        include_bytes!("../assets/hud/panel_skill.png"),
        include_bytes!("../assets/hud/panel_skill_hover.png"),
    ],
    [
        include_bytes!("../assets/hud/panel_social.png"),
        include_bytes!("../assets/hud/panel_social_hover.png"),
    ],
];

/// A PNG's pixels as RGBA.
fn decode_png(bytes: &[u8]) -> Option<UiPixels> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba {
        return None;
    }
    let rgba = buf[..(info.width * info.height * 4) as usize].to_vec();
    Some(UiPixels {
        width: info.width,
        height: info.height,
        rgba: Arc::from(rgba),
    })
}

/// The shipped icons under the panel buttons' orb DIDs (normal and rollover).
pub fn shipped_icons() -> HashMap<u32, UiPixels> {
    let mut icons = HashMap::new();
    for (orb, files) in PANEL_ORBS.iter().zip(SHIPPED) {
        for (&id, bytes) in orb.iter().zip(files) {
            if let Some(image) = decode_png(bytes) {
                icons.insert(id, image);
            }
        }
    }
    icons
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain bronze disc and an AC1-style image with a green square in its ring.
    fn fixtures() -> (Rgba, Rgba) {
        let (w, h) = (27u32, 28u32);
        let mut base = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let d = ((x as f32 - 13.0).powi(2) + (y as f32 - 13.0).powi(2)).sqrt();
                if d <= 12.5 {
                    base[((y * w + x) * 4) as usize..][..4].copy_from_slice(&[170, 120, 50, 255]);
                }
            }
        }
        let (sw, sh) = (35u32, 27u32);
        let mut symbol = vec![0u8; (sw * sh * 4) as usize];
        for y in 0..sh {
            for x in 0..sw {
                let green = (12..=22).contains(&x) && (9..=19).contains(&y);
                let p = if green {
                    [0, 220, 40, 255]
                } else {
                    [40, 40, 30, 255]
                };
                symbol[((y * sw + x) * 4) as usize..][..4].copy_from_slice(&p);
            }
        }
        let image = |width, height, rgba| Rgba {
            width,
            height,
            rgba,
        };
        (image(w, h, base), image(sw, sh, symbol))
    }

    #[test]
    fn the_symbol_fills_the_face_in_ivory_inside_the_orb() {
        let (base, symbol) = fixtures();
        let out = icon(&base, &symbol, SymbolKind::Ring, 1.0);
        let centre = out.at(13, 13);
        assert!(centre[0] > 200.0 && centre[2] > 150.0, "ivory: {centre:?}");
        let edge = out.at(13, 2);
        assert!(edge[0] > edge[2] + 40.0, "bronze: {edge:?}");
        assert_eq!(out.at(0, 0)[3], 0.0);
    }

    #[test]
    fn every_shipped_icon_decodes_at_the_orbs_size() {
        let shipped = shipped_icons();
        assert_eq!(shipped.len(), 14);
        assert!(shipped.values().all(|i| (i.width, i.height) == (27, 28)));
    }

    #[test]
    fn the_rollover_is_lighter() {
        let (base, symbol) = fixtures();
        let normal = icon(&base, &symbol, SymbolKind::Ring, 1.0);
        let lit = icon(&base, &symbol, SymbolKind::Ring, ROLLOVER_LIGHT);
        assert!(lit.at(13, 2)[0] >= normal.at(13, 2)[0]);
    }
}
