//! The UI drawn at a window's own size (a deviation from retail, which drew its UI pixel for
//! pixel on a 1024 x 768 window): the layout stays in UI pixels, and a surface `scale` times the
//! root's size is drawn with every box scaled (`UiTree::set_render_scale`). An image is filtered
//! once from its source to its scaled box, and a glyph is grown from the DAT font's coverage:
//! interpolated, then its edge narrowed to a window pixel (the coverage read as a distance to the
//! glyph's edge), so lettering stays sharp at any size instead of a small surface stretched soft.
//! At scale 1 nothing here runs and the surfaces are retail's.

use std::collections::HashMap;

use ac2_assets::Ac2Font;

use crate::compose::{blit_op, UiPixels, UiSurface};
use crate::element::{UiBox, UiImage};

/// A glyph's coverage at a scale: its width, height and bytes (row by row).
pub(crate) type ScaledGlyph = (i32, i32, Vec<u8>);

/// Glyphs already grown while drawing one text, by font, character and colour-free use.
pub(crate) type GlyphCache = HashMap<(usize, u16), ScaledGlyph>;

impl UiSurface {
    /// A UI-pixel edge (a box's left or top, or one past its right or bottom) in surface pixels.
    pub(crate) fn scaled(&self, v: i32) -> i32 {
        if self.scale == 1.0 {
            v
        } else {
            (f64::from(v) * self.scale).round() as i32
        }
    }

    /// A box in UI pixels in surface pixels; neighbouring boxes stay neighbours.
    pub(crate) fn scaled_box(&self, b: UiBox) -> UiBox {
        if self.scale == 1.0 {
            return b;
        }
        UiBox::new(
            self.scaled(b.x0),
            self.scaled(b.y0),
            self.scaled(b.x1 + 1) - 1,
            self.scaled(b.y1 + 1) - 1,
        )
    }

    /// `Graphic::Draw` of an element's image (box `me`, dirty part `dirty`, UI pixels): retail's
    /// at scale 1, else [`UiSurface::draw_graphic_scaled`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_image(
        &mut self,
        image: &UiPixels,
        mode: UiImage,
        stretch: bool,
        smooth: bool,
        alpha_value: f32,
        me: UiBox,
        dirty: UiBox,
    ) {
        if self.scale == 1.0 {
            self.draw_graphic(image, mode, stretch, smooth, alpha_value, me, dirty);
        } else {
            self.draw_graphic_scaled(image, mode, stretch, alpha_value, me, dirty);
        }
    }

    /// `Graphic::Draw` at the surface's scale: an image the element's size (or stretched to it)
    /// filtered to the element's scaled box, a smaller one tiled in scaled tiles.
    fn draw_graphic_scaled(
        &mut self,
        image: &UiPixels,
        mode: UiImage,
        stretch: bool,
        alpha_value: f32,
        me: UiBox,
        dirty: UiBox,
    ) {
        let (w, h) = (me.width(), me.height());
        let (iw, ih) = (image.width as i32, image.height as i32);
        if w <= 0 || h <= 0 || iw <= 0 || ih <= 0 {
            return;
        }
        let op = blit_op(mode, alpha_value);
        let full = UiBox::new(0, 0, iw - 1, ih - 1);
        let dirty = self.scaled_box(dirty);
        let draw = |surface: &mut Self, tile: UiBox| {
            let tile = surface.scaled_box(tile);
            if let Some(clip) = dirty.intersection(&tile) {
                surface.blit_filtered(image, full, tile, clip, op);
            }
        };
        if (w == iw && h == ih) || stretch {
            draw(self, me);
            return;
        }
        let mut ty = 0;
        while ty < h {
            let mut tx = 0;
            while tx < w {
                // A tile cut at the element's edge keeps its source pixels' size.
                let tile = UiBox::new(
                    me.x0 + tx,
                    me.y0 + ty,
                    (me.x0 + tx + iw - 1).min(me.x1),
                    (me.y0 + ty + ih - 1).min(me.y1),
                );
                let cut = UiBox::new(0, 0, tile.width() - 1, tile.height() - 1);
                let dst = self.scaled_box(tile);
                if let Some(clip) = dirty.intersection(&dst) {
                    self.blit_filtered(image, cut, dst, clip, op);
                }
                tx += iw;
            }
            ty += ih;
        }
    }

    /// A box `width` x `height` at (`x`, `y`) in `window` (UI pixels) filled with `pixel`,
    /// clipped to `window`: retail's `fill_box` at scale 1.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fill(
        &mut self,
        window: UiBox,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        pixel: [u8; 4],
    ) {
        if self.scale == 1.0 {
            self.fill_box(window, x, y, width, height, pixel);
            return;
        }
        let (gx, gy) = (window.x0 + x, window.y0 + y);
        let at = UiBox::new(gx, gy, gx + width - 1, gy + height - 1);
        let Some(clip) = self
            .scaled_box(at)
            .intersection(&self.scaled_box(window))
            .and_then(|b| b.intersection(&self.bounds()))
        else {
            return;
        };
        for ty in clip.y0..=clip.y1 {
            for tx in clip.x0..=clip.x1 {
                let i = (ty as usize * self.width as usize + tx as usize) * 4;
                self.rgba[i..i + 4].copy_from_slice(&pixel);
            }
        }
    }

    /// `SurfaceWindow::DrawCharacter` at (`x`, `y`) in `window` (UI pixels): retail's at scale
    /// 1, else the glyph grown to the scale (`cache` keeps it for the rest of the text) and
    /// blended by its coverage. `offset` moves it whole surface pixels (an outline's copies).
    /// Returns the advance in UI pixels.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn glyph(
        &mut self,
        window: UiBox,
        x: i32,
        y: i32,
        offset: (i32, i32),
        font: &Ac2Font,
        ch: u16,
        color: [f32; 4],
        blend: bool,
        cache: &mut GlyphCache,
    ) -> i32 {
        if self.scale == 1.0 {
            return self.draw_character(window, x + offset.0, y + offset.1, font, ch, color, blend);
        }
        let Some(desc) = font.char_desc(ch) else {
            return 0;
        };
        let advance = desc.width as i32;
        if color[3] <= 0.0 {
            return advance;
        }
        let scale = self.scale;
        let key = (font as *const Ac2Font as usize, ch);
        let (gw, gh, coverage) = cache
            .entry(key)
            .or_insert_with(|| grow_glyph(font, desc.offset, advance, scale));
        let (gw, gh) = (*gw, *gh);
        let clip = self.scaled_box(window).intersection(&self.bounds());
        let Some(clip) = clip else {
            return advance;
        };
        let x0 = self.scaled(window.x0 + x) + offset.0;
        let y0 = self.scaled(window.y0 + y) + offset.1;
        let pixel = crate::text::color_bytes(color);
        for row in 0..gh {
            let ty = y0 + row;
            if ty < clip.y0 || ty > clip.y1 {
                continue;
            }
            for col in 0..gw {
                let tx = x0 + col;
                if tx < clip.x0 || tx > clip.x1 {
                    continue;
                }
                let c = coverage[(row * gw + col) as usize];
                if c == 0 {
                    continue;
                }
                let at = (ty as usize * self.width as usize + tx as usize) * 4;
                let dest = &mut self.rgba[at..at + 4];
                if c == 255 || (!blend && c >= 128) {
                    dest.copy_from_slice(&pixel);
                    continue;
                }
                crate::text::blend_coverage(dest, color, c);
            }
        }
        advance
    }
}

/// The glyph whose strip columns start at `offset` and which draws `advance - 1` of them (as
/// `DrawCharacter` does), grown to `scale`: each window pixel's coverage interpolated from the
/// strip, then (grown) its edge narrowed to about a window pixel around half coverage.
fn grow_glyph(font: &Ac2Font, offset: u32, advance: i32, scale: f64) -> ScaledGlyph {
    let (cols, rows) = ((advance - 1).max(0), font.max_char_height as i32);
    let gw = (f64::from(cols) * scale).round() as i32;
    let gh = (f64::from(rows) * scale).round() as i32;
    if cols <= 0 || rows <= 0 || gw <= 0 || gh <= 0 {
        return (0, 0, Vec::new());
    }
    let source = |c: i32, r: i32| -> f64 {
        if c < 0 || r < 0 || c >= cols || r >= rows {
            0.0
        } else {
            f64::from(font.coverage(offset + c as u32, r as u32)) / 255.0
        }
    };
    let mut out = Vec::with_capacity((gw * gh) as usize);
    for j in 0..gh {
        for i in 0..gw {
            let value = if scale >= 1.0 {
                let u = (f64::from(i) + 0.5) / scale - 0.5;
                let v = (f64::from(j) + 0.5) / scale - 0.5;
                let (c0, r0) = (u.floor() as i32, v.floor() as i32);
                let (tu, tv) = (u - f64::from(c0), v - f64::from(r0));
                let top = source(c0, r0) * (1.0 - tu) + source(c0 + 1, r0) * tu;
                let bottom = source(c0, r0 + 1) * (1.0 - tu) + source(c0 + 1, r0 + 1) * tu;
                let smooth = top * (1.0 - tv) + bottom * tv;
                // The coverage changes by about one over a strip pixel, `scale` window pixels:
                // steepened by the scale it changes over one, centred a little under half so thin
                // strokes keep their weight.
                ((smooth - 0.42) * scale + 0.5).clamp(0.0, 1.0)
            } else {
                // Shrunk: the mean of the strip pixels the window pixel covers.
                let (a, b) = (f64::from(i) / scale, f64::from(i + 1) / scale);
                let (c, d) = (f64::from(j) / scale, f64::from(j + 1) / scale);
                let mut sum = 0.0;
                for r in c.floor() as i32..d.ceil() as i32 {
                    let wr = (f64::from(r + 1).min(d) - f64::from(r).max(c)).max(0.0);
                    for k in a.floor() as i32..b.ceil() as i32 {
                        let wk = (f64::from(k + 1).min(b) - f64::from(k).max(a)).max(0.0);
                        sum += source(k, r) * wr * wk;
                    }
                }
                (sum * scale * scale).clamp(0.0, 1.0)
            };
            out.push((value * 255.0).round() as u8);
        }
    }
    (gw, gh, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 5-wide, 6-high anti-aliased font with one glyph, 'I': a 2-pixel stem with soft edges.
    fn font() -> Ac2Font {
        let mut data = vec![0u8; 5 * 6];
        for row in 0..6 {
            data[row * 5 + 1] = 128;
            data[row * 5 + 2] = 255;
            data[row * 5 + 3] = 255;
        }
        Ac2Font::new(
            [6, 5, 5, 5],
            true,
            vec![ac2_assets::Ac2FontChar {
                width: 5,
                unicode: u16::from(b'I'),
                offset: 0,
            }],
            data,
        )
    }

    #[test]
    fn a_grown_glyph_has_solid_middles_and_one_pixel_edges() {
        let font = font();
        let (w, h, cov) = grow_glyph(&font, 0, 5, 3.0);
        assert_eq!((w, h), (12, 18));
        let row = &cov[(9 * w) as usize..(10 * w) as usize];
        // Solid inside the stem, clear outside it, and few partial pixels between.
        assert_eq!(row[7], 255, "{row:?}");
        assert_eq!(row[0], 0, "{row:?}");
        let partial = row.iter().filter(|&&c| c > 0 && c < 255).count();
        assert!(partial <= 4, "{row:?}");
    }

    #[test]
    fn scaled_boxes_tile_without_gaps() {
        let surface = UiSurface {
            root: 0,
            x: 0,
            y: 0,
            width: 117,
            height: 117,
            rotation: 0,
            shown: true,
            opacity: 1.0,
            scale: 1.171875,
            rgba: Vec::new(),
        };
        let a = surface.scaled_box(UiBox::new(0, 0, 9, 9));
        let b = surface.scaled_box(UiBox::new(10, 0, 19, 9));
        assert_eq!(a.x1 + 1, b.x0);
        assert_eq!(surface.scaled_box(UiBox::new(0, 0, 99, 99)).width(), 117);
    }
}
