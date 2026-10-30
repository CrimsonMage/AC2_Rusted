//! The modern HUD's own art (a modern addition, AGENTS.md: the HUD's two styles): the round wells,
//! rings, glows, the shortcut strip's fade, the dock's orb and pips and the bevelled backings of the
//! user's chosen design ("variant D"). They are drawn here, anti-aliased, into images under the
//! client's own DIDs (`0x7E01xxxx`, outside every DAT range), and the panel and attack buttons' DAT
//! icons are set into their wells. Colours are the design's (sRGB bytes, straight alpha).

use std::collections::HashMap;
use std::sync::Arc;

use crate::compose::UiPixels;
pub use crate::modern_art_chat::*;
pub use crate::modern_art_drawer::*;
pub use crate::modern_art_loot::*;
pub use crate::modern_art_radar::*;

/// A colour, straight alpha, each channel 0..1.
pub(crate) type Rgba = [f32; 4];

pub(crate) fn rgb(hex: u32, alpha: f32) -> Rgba {
    [
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
        alpha,
    ]
}

pub(crate) fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    [0, 1, 2, 3].map(|c| a[c] + (b[c] - a[c]) * t)
}

/// A drawing surface: straight-alpha pixels, painted source-over.
pub(crate) struct Canvas {
    pub width: usize,
    pub height: usize,
    px: Vec<Rgba>,
}

impl Canvas {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            px: vec![[0.0; 4]; width * height],
        }
    }

    /// `color` over the pixel at `(x, y)`, by `cover` (0..1).
    pub(crate) fn over(&mut self, x: usize, y: usize, color: Rgba, cover: f32) {
        let a = color[3] * cover.clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        let dst = &mut self.px[y * self.width + x];
        let out_a = a + dst[3] * (1.0 - a);
        for c in 0..3 {
            dst[c] = (color[c] * a + dst[c] * dst[3] * (1.0 - a)) / out_a.max(1e-6);
        }
        dst[3] = out_a;
    }

    /// Paints every pixel by `shade(x, y, distance)`: `distance` is the shape's signed distance at
    /// the pixel's centre (negative inside), and `shade` gives the colour and its cover there.
    pub(crate) fn paint(
        &mut self,
        sdf: impl Fn(f32, f32) -> f32,
        shade: impl Fn(f32, f32, f32) -> (Rgba, f32),
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let (color, cover) = shade(fx, fy, sdf(fx, fy));
                self.over(x, y, color, cover);
            }
        }
    }

    /// The pixels as an image.
    pub(crate) fn finish(self) -> UiPixels {
        let rgba: Vec<u8> = self
            .px
            .iter()
            .flat_map(|p| p.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8))
            .collect();
        UiPixels {
            width: self.width as u32,
            height: self.height as u32,
            rgba: Arc::from(rgba),
        }
    }

    /// `image` drawn into the box at `(x, y)` sized `(w, h)`, bilinear, times `alpha`.
    pub(crate) fn image(
        &mut self,
        image: &UiPixels,
        (x, y, w, h): (f32, f32, f32, f32),
        alpha: f32,
    ) {
        let (iw, ih) = (image.width as f32, image.height as f32);
        for py in 0..self.height {
            for px in 0..self.width {
                let (fx, fy) = (px as f32 + 0.5 - x, py as f32 + 0.5 - y);
                if fx < 0.0 || fy < 0.0 || fx >= w || fy >= h {
                    continue;
                }
                let (sx, sy) = (fx / w * iw - 0.5, fy / h * ih - 0.5);
                let color = bilinear(image, sx, sy);
                self.over(
                    px,
                    py,
                    [color[0], color[1], color[2], color[3] * alpha],
                    1.0,
                );
            }
        }
    }
}

/// The image's colour at `(x, y)` in its pixels, bilinear and premultiplied in the mix.
pub(crate) fn bilinear(image: &UiPixels, x: f32, y: f32) -> Rgba {
    let (w, h) = (image.width as i32, image.height as i32);
    let (x0, y0) = (x.floor() as i32, y.floor() as i32);
    let (tx, ty) = (x - x0 as f32, y - y0 as f32);
    let mut sum = [0.0f32; 4];
    for (dx, dy, wgt) in [
        (0, 0, (1.0 - tx) * (1.0 - ty)),
        (1, 0, tx * (1.0 - ty)),
        (0, 1, (1.0 - tx) * ty),
        (1, 1, tx * ty),
    ] {
        let (sx, sy) = ((x0 + dx).clamp(0, w - 1), (y0 + dy).clamp(0, h - 1));
        let i = ((sy * w + sx) * 4) as usize;
        let a = f32::from(image.rgba[i + 3]) / 255.0 * wgt;
        for (c, total) in sum.iter_mut().take(3).enumerate() {
            *total += f32::from(image.rgba[i + c]) / 255.0 * a;
        }
        sum[3] += a;
    }
    if sum[3] <= 0.0 {
        return [0.0; 4];
    }
    [sum[0] / sum[3], sum[1] / sum[3], sum[2] / sum[3], sum[3]]
}

/// The cover of a shape's edge at signed distance `d` (a pixel-wide ramp).
pub(crate) fn inside(d: f32) -> f32 {
    (0.5 - d).clamp(0.0, 1.0)
}

/// The cover of the band `from..to` pixels in from a shape's edge (an inset ring).
pub(crate) fn band(d: f32, from: f32, to: f32) -> f32 {
    (inside(d + from) - inside(d + to)).max(0.0)
}

/// A soft glow `blur` pixels out from a shape's edge (zero inside).
pub(crate) fn glow(d: f32, blur: f32) -> f32 {
    if d <= 0.0 {
        0.0
    } else {
        (-(d / blur).powi(2) * 2.0).exp()
    }
}

pub(crate) fn circle(cx: f32, cy: f32, r: f32) -> impl Fn(f32, f32) -> f32 {
    move |x, y| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r
}

pub(crate) fn round_rect(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> impl Fn(f32, f32) -> f32 {
    move |x, y| {
        let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let (hx, hy) = ((x1 - x0) / 2.0 - r, (y1 - y0) / 2.0 - r);
        let qx = (x - cx).abs() - hx;
        let qy = (y - cy).abs() - hy;
        (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r
    }
}

/// The client's own image DIDs.
pub const SLOT_WELL: u32 = 0x7E01_0001;
pub const SLOT_EMPTY: u32 = 0x7E01_0002;
pub const SLOT_EMPTY_SHOWN: u32 = 0x7E01_0003;
pub const SLOT_CLEAR: u32 = 0x7E01_0004;
pub const SLOT_PRESSED: u32 = 0x7E01_0005;
pub const PANEL_WELL: u32 = 0x7E01_0010;
pub const PANEL_WELL_ON: u32 = 0x7E01_0011;
pub const ORB: u32 = 0x7E01_0020;
pub const PIP: u32 = 0x7E01_0021;
pub const PIP_ON: u32 = 0x7E01_0022;
pub const BEVEL: u32 = 0x7E01_0030;
pub const SEPARATOR: u32 = 0x7E01_0031;
pub const ARROW_UP: u32 = 0x7E01_0040;
pub const ARROW_DOWN: u32 = 0x7E01_0041;
pub const BADGE: u32 = 0x7E01_0042;
/// The strip's fade (`fade(height)`, made for the strip's height when it is laid out).
pub const STRIP_FADE: u32 = 0x7E01_0050;
/// The DAT item slot the item well is made of.
pub const ITEM_SLOT: u32 = 0x4100_02BC;
/// An item slot's pitch (36 and the gap).
pub const ITEM_PITCH: i32 = 40;

/// The design's colours.
const GOLD: u32 = 0xD9C27A;
/// The gold as the text's colour channels.
pub const GOLD_RGB: [f32; 3] = [
    0xD9 as f32 / 255.0,
    0xC2 as f32 / 255.0,
    0x7A as f32 / 255.0,
];
const BRONZE_LINE: u32 = 0x8B846C;

/// The size of a panel button's art: the 38-pixel well and its glow's room on every side.
pub const PANEL_WELL_SIZE: i32 = 38;
pub const GLOW_ROOM: i32 = 6;
pub const PANEL_ART: i32 = PANEL_WELL_SIZE + 2 * GLOW_ROOM;
/// The attack buttons' well and art.
pub const ATTACK_WELL_SIZE: i32 = 30;
pub const ATTACK_ART: i32 = ATTACK_WELL_SIZE + 2 * GLOW_ROOM;
/// The orb's art (the 38-pixel orb and its pips' ring) and a pip's art.
pub const ORB_ART: i32 = 58;
pub const PIP_ART: i32 = 14;
/// A shortcut slot's well, and its pitch: the well and the 3-pixel gap after it, which the slot's
/// art keeps clear on its right and bottom.
pub const SLOT: i32 = 32;
pub const SLOT_PITCH: i32 = 35;

/// A round well `size` across, centred in a canvas `art` across: a radial fill from `inner` to
/// `outer`, an inset line, a black rim, and optionally a gold ring and glow.
fn well(
    art: usize,
    size: f32,
    inner: Rgba,
    outer: Rgba,
    line: (Rgba, f32),
    ring: Option<(f32, Rgba, f32)>,
) -> Canvas {
    let mut c = Canvas::new(art, art);
    let mid = art as f32 / 2.0;
    let r = size / 2.0;
    let shape = circle(mid, mid, r);
    if let Some((_, glow_color, blur)) = ring {
        c.paint(&shape, |_, _, d| (glow_color, glow(d, blur)));
    }
    // The black rim one pixel out.
    c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, -1.0, 0.0)));
    c.paint(&shape, |x, y, d| {
        let t = ((x - mid).powi(2) + (y - mid).powi(2)).sqrt() / r;
        (mix(inner, outer, t), inside(d))
    });
    c.paint(&shape, |_, _, d| (line.0, band(d, 0.0, line.1)));
    if let Some((width, color, _)) = ring {
        c.paint(&shape, |_, _, d| (color, band(d, 0.0, width)));
    }
    c
}

/// A panel button's well: normal, or lit for a shown panel.
fn panel_well(on: bool) -> UiPixels {
    let art = PANEL_ART as usize;
    if on {
        well(
            art,
            PANEL_WELL_SIZE as f32,
            rgb(0x4B4220, 1.0),
            rgb(0x1D1A0C, 1.0),
            (rgb(GOLD, 1.0), 2.0),
            Some((2.0, rgb(GOLD, 1.0), 6.0)),
        )
        .finish()
    } else {
        well(
            art,
            PANEL_WELL_SIZE as f32,
            rgb(0x0E403A, 1.0),
            rgb(0x02201C, 1.0),
            (rgb(BRONZE_LINE, 1.0), 1.0),
            None,
        )
        .finish()
    }
}

/// A panel button's icon set in its art's middle at its own size; `hover` adds the gold line and
/// a soft glow the design gives a hovered button.
pub fn panel_icon(icon: &UiPixels, hover: bool) -> UiPixels {
    let art = PANEL_ART as usize;
    let mut c = Canvas::new(art, art);
    let mid = art as f32 / 2.0;
    if hover {
        let shape = circle(mid, mid, PANEL_WELL_SIZE as f32 / 2.0);
        c.paint(&shape, |_, _, d| (rgb(0xE8C050, 0.33), glow(d, 5.0)));
        c.paint(&shape, |_, _, d| (rgb(GOLD, 1.0), band(d, 0.0, 1.0)));
    }
    let (w, h) = (icon.width as f32, icon.height as f32);
    c.image(icon, (mid - w / 2.0, mid - h / 2.0, w, h), 1.0);
    c.finish()
}

/// An attack button's art: its icon at 22 pixels in a 30-pixel well, ringed gold when `on`, at
/// opacity `alpha` (the design's 0.8 at rest, 1 under the pointer, 0.35 when it does nothing).
pub fn attack_button(icon: &UiPixels, on: bool, alpha: f32) -> UiPixels {
    let art = ATTACK_ART as usize;
    let mut c = well(
        art,
        ATTACK_WELL_SIZE as f32,
        rgb(0x0C3A35, 1.0),
        rgb(0x021815, 1.0),
        (rgb(BRONZE_LINE, 1.0), 1.0),
        on.then_some((2.0, rgb(GOLD, 1.0), 6.0)),
    );
    let mid = art as f32 / 2.0;
    c.image(icon, (mid - 11.0, mid - 11.0, 22.0, 22.0), 1.0);
    for p in &mut c.px {
        p[3] *= alpha;
    }
    c.finish()
}

/// A shortcut slot's well: the dark glass with its line, or an empty slot's faint line (shown at
/// the strip's hover).
fn slot(kind: u32) -> UiPixels {
    let n = SLOT as f32;
    // The template's art is the pitch square (the well in its corner); the frame's is the well.
    let side = match kind {
        SLOT_WELL | SLOT_EMPTY | SLOT_EMPTY_SHOWN => SLOT_PITCH,
        _ => SLOT,
    } as usize;
    let mut c = Canvas::new(side, side);
    let shape = round_rect(0.0, 0.0, n, n, 4.0);
    match kind {
        SLOT_WELL => {
            c.paint(&shape, |_, _, d| (rgb(0x001F1D, 0.8), inside(d)));
            c.paint(&shape, |_, _, d| (rgb(0x6F6957, 1.0), band(d, 0.0, 1.0)));
        }
        SLOT_EMPTY => c.paint(&shape, |_, _, d| {
            (rgb(BRONZE_LINE, 0.13 * 0.28), band(d, 0.0, 1.0))
        }),
        SLOT_EMPTY_SHOWN => c.paint(&shape, |_, _, d| {
            (rgb(BRONZE_LINE, 0.53 * 0.9), band(d, 0.0, 1.0))
        }),
        SLOT_PRESSED => c.paint(&shape, |_, _, d| (rgb(GOLD, 1.0), band(d, 0.0, 2.0))),
        _ => {}
    }
    c.finish()
}

/// The dock's orb: a lit teal disc in a bronze ring, the menu's three bars on it.
fn orb() -> UiPixels {
    let art = ORB_ART as usize;
    let mut c = Canvas::new(art, art);
    let (cx, cy, r) = (29.0, 29.0, 19.0);
    let shape = circle(cx, cy, r);
    // Its shadow, three pixels down.
    let shadow = circle(cx, cy + 3.0, r);
    c.paint(&shadow, |_, _, d| (rgb(0x000000, 0.67), glow(d, 6.0)));
    c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, -1.0, 0.0)));
    let (lx, ly) = (cx - r + 0.35 * 2.0 * r, cy - r + 0.30 * 2.0 * r);
    c.paint(&shape, |x, y, d| {
        let t = ((x - lx).powi(2) + (y - ly).powi(2)).sqrt() / (1.3 * r);
        let color = if t < 0.55 {
            mix(rgb(0x4A8C80, 1.0), rgb(0x0A3A34, 1.0), t / 0.55)
        } else {
            mix(rgb(0x0A3A34, 1.0), rgb(0x021412, 1.0), (t - 0.55) / 0.45)
        };
        (color, inside(d))
    });
    c.paint(&shape, |_, _, d| (rgb(0xB8B19B, 1.0), band(d, 0.0, 2.0)));
    c.paint(&shape, |_, _, d| (rgb(0x3B372B, 1.0), band(d, 2.0, 3.0)));
    for row in [-5.0, 0.0, 5.0] {
        let bar = round_rect(
            cx - 7.0 - 0.9,
            cy + row - 0.9,
            cx + 7.0 + 0.9,
            cy + row + 0.9,
            0.9,
        );
        c.paint(&bar, |_, _, d| (rgb(0xF1E9C8, 1.0), inside(d)));
    }
    c.finish()
}

/// A pip: a dark dot in a fine line, or gold with its glow for a shown panel.
fn pip(on: bool) -> UiPixels {
    let art = PIP_ART as usize;
    let mut c = Canvas::new(art, art);
    let mid = art as f32 / 2.0;
    let shape = circle(mid, mid, 3.0);
    if on {
        c.paint(&shape, |_, _, d| (rgb(0xF0C850, 1.0), glow(d, 4.0)));
        c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, -1.0, 0.0)));
        c.paint(&shape, |_, _, d| (rgb(GOLD, 1.0), inside(d)));
    } else {
        c.paint(&shape, |_, _, d| (rgb(0x10241F, 1.0), inside(d)));
        c.paint(&shape, |_, _, d| (rgb(0x6F6957, 1.0), band(d, 0.0, 1.0)));
    }
    c.finish()
}

/// The design's bevelled backing (`.bev`): a teal fall in a black rim and three inset lines,
/// corners 24 pixels round, drawn in nine pieces.
fn bevel() -> UiPixels {
    let n = 64usize;
    let mut c = Canvas::new(n, n);
    let shape = round_rect(1.0, 1.0, n as f32 - 1.0, n as f32 - 1.0, 23.0);
    c.paint(&shape, |_, _, d| (rgb(0x000000, 1.0), band(d, -1.0, 0.0)));
    c.paint(&shape, |_, y, d| {
        (
            mix(rgb(0x003632, 1.0), rgb(0x002A27, 1.0), y / n as f32),
            inside(d),
        )
    });
    c.paint(&shape, |_, _, d| (rgb(0xC3BCA4, 1.0), band(d, 0.0, 1.0)));
    c.paint(&shape, |_, _, d| (rgb(0x4D493C, 1.0), band(d, 1.0, 2.0)));
    c.paint(&shape, |_, _, d| (rgb(0x0C0C08, 1.0), band(d, 2.0, 3.0)));
    c.finish()
}

/// The corner size [`BEVEL`] keeps in nine pieces.
pub const BEVEL_BORDER: i32 = 24;

/// The strip's backing `height` pixels tall: the dark teal fading in from the top, its top corners
/// 18 pixels round and its bottom ones 8, drawn in nine pieces.
pub fn fade(height: i32) -> UiPixels {
    let (w, h) = (64usize, height.max(20) as usize);
    let mut c = Canvas::new(w, h);
    let top = round_rect(0.0, 0.0, w as f32, h as f32 + 18.0, 18.0);
    let bottom = round_rect(0.0, -10.0, w as f32, h as f32, 8.0);
    c.paint(
        |x, y| top(x, y).max(bottom(x, y)),
        |_, y, d| (rgb(0x00201E, 0.69 * y / h as f32), inside(d)),
    );
    c.finish()
}

/// The corner size the fade keeps in nine pieces.
pub const FADE_BORDER: i32 = 18;

/// A small gold arrow (`up` or down) centred in a 34 x 10 image, as the set chip's.
fn arrow(up: bool) -> UiPixels {
    let (w, h) = (34usize, 10usize);
    let mut c = Canvas::new(w, h);
    let (cx, cy) = (17.0f32, 5.0f32);
    // Widest at the far end from its point: an up arrow's point is at the top.
    let sign = if up { 1.0 } else { -1.0 };
    let tri = move |x: f32, y: f32| {
        // A triangle 9 wide and 6 tall: the larger of its three edges' distances.
        let dy = (y - cy) * sign;
        let a = (dy - 3.0).max(-dy - 3.0 + (x - cx).abs() * 1.33);
        a / 1.2
    };
    c.paint(tri, |_, _, d| (rgb(GOLD, 0.7), inside(d)));
    c.finish()
}

/// A key badge's backing: black at 0.8 in a fine bronze line, corners 3 round (nine pieces).
fn badge() -> UiPixels {
    let (w, h) = (16usize, 12usize);
    let mut c = Canvas::new(w, h);
    let shape = round_rect(0.0, 0.0, w as f32, h as f32, 3.0);
    c.paint(&shape, |_, _, d| (rgb(0x000000, 0.8), inside(d)));
    c.paint(&shape, |_, _, d| (rgb(0x6F6957, 1.0), band(d, 0.0, 1.0)));
    c.finish()
}

/// The fan's separator: a fine line 26 pixels tall.
fn separator() -> UiPixels {
    let mut c = Canvas::new(1, 26);
    c.paint(|_, _| -1.0, |_, _, _| (rgb(BRONZE_LINE, 0.4), 1.0));
    c.finish()
}

/// The panel buttons' DAT icons and the attack buttons' (their normal images), in the order the
/// toolbar's buttons hold them.
pub const PANEL_ICONS: [u32; 14] = [
    0x4100_09D0,
    0x4100_09D1, // Examination
    0x4100_09CE,
    0x4100_09CF, // Navigation
    0x4100_2526,
    0x4100_2527, // Option
    0x4100_09CC,
    0x4100_09CD, // Inventory
    0x4100_1E54,
    0x4100_1E55, // Quest (its field's button)
    0x4100_09CA,
    0x4100_09CB, // Skill
    0x4100_1E84,
    0x4100_1E85, // Social
];
/// The attack buttons' images: start (normal, rollover, ghosted, its `92957A1C` state) and stop
/// (normal, rollover, ghosted).
pub const START_ATTACK: [u32; 4] = [0x4100_0CEB, 0x4100_531D, 0x4100_1CD7, 0x4100_7092];
pub const STOP_ATTACK: [u32; 3] = [0x4100_0CEE, 0x4100_531E, 0x4100_531F];
/// The slot's DAT frame (normal and pressed), which the wells replace.
pub const SLOT_FRAME: u32 = 0x4100_0B8B;
pub const SLOT_FRAME_PRESSED: u32 = 0x4100_0A64;

/// Every image of the design made into `images`, and the DAT icons the buttons show set into their
/// wells under their own DIDs (each loaded in `images`, or given in `icons`: a missing one is left
/// as the DAT has it).
pub fn bake(images: &mut HashMap<u32, UiPixels>, icons: &HashMap<u32, UiPixels>) {
    for (id, image) in [
        (SLOT_WELL, slot(SLOT_WELL)),
        (SLOT_EMPTY, slot(SLOT_EMPTY)),
        (SLOT_EMPTY_SHOWN, slot(SLOT_EMPTY_SHOWN)),
        (SLOT_CLEAR, slot(SLOT_CLEAR)),
        (SLOT_PRESSED, slot(SLOT_PRESSED)),
        (PANEL_WELL, panel_well(false)),
        (PANEL_WELL_ON, panel_well(true)),
        (ORB, orb()),
        (PIP, pip(false)),
        (PIP_ON, pip(true)),
        (BEVEL, bevel()),
        (SEPARATOR, separator()),
        (ARROW_UP, arrow(true)),
        (ARROW_DOWN, arrow(false)),
        (BADGE, badge()),
        (STRIP_FADE, fade(48)),
    ] {
        images.insert(id, image);
    }
    // The drawers' parts (`modern_art_drawer`), the item well from the DAT slot.
    let item_slot = icons
        .get(&ITEM_SLOT)
        .or_else(|| images.get(&ITEM_SLOT))
        .cloned();
    crate::modern_art_drawer::bake(images, item_slot.as_ref());
    crate::modern_art_radar::bake(images);
    crate::modern_art_chat::bake(images);
    crate::modern_art_loot::bake(images);
    // The slots' DAT frame gives way to the wells' own line; pressed, a gold ring.
    images.insert(SLOT_FRAME, slot(SLOT_CLEAR));
    images.insert(SLOT_FRAME_PRESSED, slot(SLOT_PRESSED));
    for pair in PANEL_ICONS.chunks(2) {
        // The normal icon in the middle of its art; the rollover with the gold line and glow.
        if let Some(icon) = images.get(&pair[0]).cloned() {
            images.insert(pair[0], panel_icon(&icon, false));
            images.insert(pair[1], panel_icon(&icon, true));
        }
    }
    // Each attack button's states from its normal icon (`icons` gives it when the button showed
    // another state at the build and its normal image was not loaded): at rest, under the
    // pointer, then ghosted. Retail ghosts the start button while the player attacks, which the
    // design shows lit; and the stop button while the player does not, which it dims.
    for (states, looks) in [
        (
            &START_ATTACK[..],
            &[(false, 0.8), (false, 1.0), (true, 1.0), (true, 1.0)][..],
        ),
        (
            &STOP_ATTACK[..],
            &[(false, 0.8), (false, 1.0), (false, 0.35)][..],
        ),
    ] {
        let Some(icon) = icons
            .get(&states[0])
            .or_else(|| images.get(&states[0]))
            .cloned()
        else {
            continue;
        };
        for (&id, &(on, alpha)) in states.iter().zip(looks) {
            images.insert(id, attack_button(&icon, on, alpha));
        }
    }
    // The set chip's halves: an arrow over the set's number, and the number over an arrow.
    for id in SET_UP {
        images.insert(id, arrow(true));
    }
    for id in SET_DOWN {
        images.insert(id, chip_down());
    }
}

/// The set chip's lower half: the set's number's room, then the down arrow.
fn chip_down() -> UiPixels {
    let up = arrow(false);
    let (w, h) = (34usize, 26usize);
    let mut rgba = vec![0u8; w * h * 4];
    let top = h - up.height as usize;
    for y in 0..up.height as usize {
        let row = &up.rgba[y * w * 4..(y + 1) * w * 4];
        rgba[(top + y) * w * 4..(top + y + 1) * w * 4].copy_from_slice(row);
    }
    UiPixels {
        width: w as u32,
        height: h as u32,
        rgba: Arc::from(rgba),
    }
}

/// The DAT icons [`bake`] sets in wells from their normal images: load them when the HUD does not
/// show them yet.
pub const SOURCE_ICONS: [u32; 3] = [START_ATTACK[0], STOP_ATTACK[0], ITEM_SLOT];

/// The set chip's halves' images (normal and pressed): the upper and the lower.
pub const SET_UP: [u32; 2] = [0x4100_1D50, 0x4100_1D51];
pub const SET_DOWN: [u32; 2] = [0x4100_1D4E, 0x4100_1D4F];

#[cfg(test)]
mod tests {
    use super::*;

    fn at(image: &UiPixels, x: usize, y: usize) -> [u8; 4] {
        let i = (y * image.width as usize + x) * 4;
        [
            image.rgba[i],
            image.rgba[i + 1],
            image.rgba[i + 2],
            image.rgba[i + 3],
        ]
    }

    #[test]
    fn a_panel_well_is_a_dark_teal_disc_with_a_clear_outside() {
        let w = panel_well(false);
        assert_eq!((w.width as i32, w.height as i32), (PANEL_ART, PANEL_ART));
        let mid = PANEL_ART as usize / 2;
        let centre = at(&w, mid, mid);
        assert!(
            centre[3] == 255 && centre[1] > centre[0],
            "teal: {centre:?}"
        );
        assert_eq!(at(&w, 0, 0)[3], 0);
        let lit = panel_well(true);
        let gold = at(&lit, mid, GLOW_ROOM as usize);
        assert!(gold[0] > 180 && gold[2] < 160, "a gold ring: {gold:?}");
    }

    #[test]
    fn an_icon_sits_in_the_middle_of_its_art_and_the_hover_rings_it() {
        let icon = UiPixels {
            width: 2,
            height: 2,
            rgba: Arc::from(vec![255u8; 16]),
        };
        let plain = panel_icon(&icon, false);
        let mid = PANEL_ART as usize / 2;
        assert_eq!(at(&plain, mid, mid)[3], 255);
        assert_eq!(at(&plain, mid, GLOW_ROOM as usize)[3], 0);
        let hover = panel_icon(&icon, true);
        assert!(at(&hover, mid, GLOW_ROOM as usize)[3] > 128);
    }

    #[test]
    fn the_fade_darkens_downwards_and_every_image_is_baked() {
        let f = fade(40);
        assert!(at(&f, 32, 35)[3] > at(&f, 32, 12)[3]);
        let mut images = HashMap::new();
        bake(&mut images, &HashMap::new());
        for id in [
            SLOT_WELL, ORB, PIP_ON, BEVEL, ARROW_UP, BADGE, STRIP_FADE, SLOT_FRAME,
        ] {
            assert!(images.contains_key(&id), "{id:08X}");
        }
    }
}
