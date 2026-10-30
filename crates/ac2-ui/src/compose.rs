//! Drawing the element tree the way retail does, in software: every root element owns a
//! `UISurface` (A8R8G8B8, starting clear) that `UIRegion::DrawHere` paints, element by element,
//! clipped to each ancestor, children from the last (highest z) to the first; the surfaces are
//! then drawn over the scene (`UISurfaceObject`, point sampled, blended by their alpha). A text
//! element's text goes over its image (`UIElement_Text::PostBlit`, in `text.rs`).
//!
//! `Graphic::Draw` puts an element's image 1:1 when the sizes match, stretched when the element
//! sizes its image to itself (`size_image_to_owner`) and tiled from the element's corner
//! otherwise. `SurfaceWindow::Blit` copies the pixels (alpha included) for draw mode 0; modes 1
//! and 2 blend by the source alpha into the colour while the surface keeps its alpha; mode 3
//! copies with the alpha scaled by the element's `alpha_value`. Scaling picks the nearest source
//! pixel, as retail's; the modern style filters it instead (`compose_filter`).

use std::collections::HashMap;
use std::sync::Arc;

use crate::element::{UiBox, UiImage, UiTree};

/// A decoded UI image (RGBA8, rows top-down).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiPixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

/// An element's image drawn in nine pieces with corners this many pixels square (`draw_nine`).
pub const NINE_SLICE: u32 = crate::token::tokenize("nine_slice");

/// One root's surface and where it sits on the screen.
#[derive(Debug, Clone, PartialEq)]
pub struct UiSurface {
    pub root: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    /// The surface object's rotation about its centre, degrees, and whether it is shown.
    pub rotation: i32,
    pub shown: bool,
    /// The surface object's material opacity.
    pub opacity: f32,
    pub rgba: Vec<u8>,
}

impl UiSurface {
    /// `full_src` of `image` drawn over `full_dst`, filtered (`compose_filter`), the part inside
    /// `clip` only; blended as `blit` blends `op`.
    fn blit_filtered(
        &mut self,
        image: &UiPixels,
        full_src: UiBox,
        full_dst: UiBox,
        clip: UiBox,
        op: BlitOp,
    ) {
        let Some(clip) = clip.intersection(&self.bounds()) else {
            return;
        };
        let (src_w, src_h) = (full_src.width(), full_src.height());
        let (dst_w, dst_h) = (full_dst.width(), full_dst.height());
        if src_w <= 0 || src_h <= 0 || dst_w <= 0 || dst_h <= 0 {
            return;
        }
        let (iw, ih) = (image.width as i32, image.height as i32);
        if full_src.x0 < 0 || full_src.y0 < 0 || full_src.x1 >= iw || full_src.y1 >= ih {
            return;
        }
        let xs = crate::compose_filter::taps(
            full_src.x0,
            src_w,
            dst_w,
            clip.x0 - full_dst.x0,
            clip.width(),
        );
        let ys = crate::compose_filter::taps(
            full_src.y0,
            src_h,
            dst_h,
            clip.y0 - full_dst.y0,
            clip.height(),
        );
        for (row, y_taps) in ys.iter().enumerate() {
            let ty = clip.y0 + row as i32;
            for (col, x_taps) in xs.iter().enumerate() {
                let tx = clip.x0 + col as i32;
                let source = crate::compose_filter::sample(image, x_taps, y_taps);
                let d = (ty as usize * self.width as usize + tx as usize) * 4;
                blend(&mut self.rgba[d..d + 4], &source, op);
            }
        }
    }

    fn bounds(&self) -> UiBox {
        UiBox::new(0, 0, self.width as i32 - 1, self.height as i32 - 1)
    }

    /// `SurfaceWindow::Blit` of `src` (in `image`) onto `dst`; sizes that differ pick the
    /// nearest source pixel.
    fn blit(&mut self, image: &UiPixels, src: UiBox, dst: UiBox, op: BlitOp) {
        let (src_w, src_h) = (src.width(), src.height());
        let (dst_w, dst_h) = (dst.width(), dst.height());
        if src_w <= 0 || src_h <= 0 || dst_w <= 0 || dst_h <= 0 {
            return;
        }
        for y in 0..dst_h {
            let sy = src.y0 + if src_h == dst_h { y } else { y * src_h / dst_h };
            let ty = dst.y0 + y;
            if sy < 0 || sy >= image.height as i32 || ty < 0 || ty >= self.height as i32 {
                continue;
            }
            for x in 0..dst_w {
                let sx = src.x0 + if src_w == dst_w { x } else { x * src_w / dst_w };
                let tx = dst.x0 + x;
                if sx < 0 || sx >= image.width as i32 || tx < 0 || tx >= self.width as i32 {
                    continue;
                }
                let s = (sy as usize * image.width as usize + sx as usize) * 4;
                let d = (ty as usize * self.width as usize + tx as usize) * 4;
                blend(&mut self.rgba[d..d + 4], &image.rgba[s..s + 4], op);
            }
        }
    }

    /// `Graphic::Draw` with `UIElement::SetImage`'s flags: element box `me`, the dirty part
    /// `dirty` (both surface coordinates). Draw mode 1 is an overlay and mode 2 blends by the
    /// image's alpha channel (blit flag 0x10), mode 3 takes `alpha_value` as the alpha (0x20),
    /// mode 0 copies. The image goes 1:1 when the sizes match, stretched when `scale`
    /// (`size_image_to_owner`), and tiled from the element's corner otherwise (`SetImage` never
    /// sets `m_centerImage`).
    fn draw_graphic(
        &mut self,
        image: &UiPixels,
        mode: UiImage,
        scale: bool,
        smooth: bool,
        alpha_value: f32,
        me: UiBox,
        dirty: UiBox,
    ) {
        let (w, h) = (me.width(), me.height());
        if w <= 0 || h <= 0 {
            return;
        }
        let (iw, ih) = (image.width as i32, image.height as i32);
        let op = match mode.draw_mode {
            1 | 2 => BlitOp::Channel,
            3 => BlitOp::Value(alpha_value),
            DRAW_OVER => BlitOp::Over,
            _ => BlitOp::Copy,
        };
        let local = dirty.offset(-me.x0, -me.y0);
        if w == iw && h == ih {
            self.blit(image, local, dirty, op);
        } else if scale && smooth {
            self.blit_filtered(image, UiBox::new(0, 0, iw - 1, ih - 1), me, dirty, op);
        } else if scale {
            let sx = f64::from(iw) / f64::from(w);
            let sy = f64::from(ih) / f64::from(h);
            let src = UiBox::new(
                (f64::from(local.x0) * sx) as i32,
                (f64::from(local.y0) * sy) as i32,
                (f64::from(local.x1 + 1) * sx) as i32 - 1,
                (f64::from(local.y1 + 1) * sy) as i32 - 1,
            );
            self.blit(image, src, dirty, op);
        } else {
            if iw <= 0 || ih <= 0 {
                return;
            }
            let mut tx = 0;
            while tx < w {
                let mut ty = 0;
                while ty < h {
                    let tile = UiBox::new(
                        me.x0 + tx,
                        me.y0 + ty,
                        me.x0 + tx + iw - 1,
                        me.y0 + ty + ih - 1,
                    );
                    if let Some(clip) = dirty.intersection(&tile) {
                        let x0 = (clip.x0 - me.x0) % iw;
                        let y0 = (clip.y0 - me.y0) % ih;
                        let src = UiBox::new(x0, y0, x0 + clip.width() - 1, y0 + clip.height() - 1);
                        self.blit(image, src, clip, op);
                    }
                    ty += ih;
                }
                tx += iw;
            }
        }
    }
}

impl UiSurface {
    /// An image drawn in nine pieces (an improvement over retail, which stretched or tiled a
    /// window's frame whole): the corners `border` pixels square at their size, the edges stretched
    /// along their length, the middle stretched both ways, so a frame keeps its corners at any
    /// size. Blended as `draw_graphic` blends the mode.
    fn draw_nine(
        &mut self,
        image: &UiPixels,
        mode: UiImage,
        border: i32,
        smooth: bool,
        alpha_value: f32,
        me: UiBox,
        dirty: UiBox,
    ) {
        let (iw, ih) = (image.width as i32, image.height as i32);
        let b = border
            .min(iw / 2)
            .min(ih / 2)
            .min(me.width() / 2)
            .min(me.height() / 2)
            .max(0);
        if iw <= 0 || ih <= 0 || me.width() <= 0 || me.height() <= 0 {
            return;
        }
        let op = match mode.draw_mode {
            1 | 2 => BlitOp::Channel,
            3 => BlitOp::Value(alpha_value),
            DRAW_OVER => BlitOp::Over,
            _ => BlitOp::Copy,
        };
        // Source and destination cuts along each axis: [start, border, far border, end).
        let sx = [0, b, iw - b, iw];
        let sy = [0, b, ih - b, ih];
        let dx = [me.x0, me.x0 + b, me.x1 + 1 - b, me.x1 + 1];
        let dy = [me.y0, me.y0 + b, me.y1 + 1 - b, me.y1 + 1];
        for col in 0..3 {
            for row in 0..3 {
                let (s0x, s1x, s0y, s1y) = (sx[col], sx[col + 1], sy[row], sy[row + 1]);
                let (d0x, d1x, d0y, d1y) = (dx[col], dx[col + 1], dy[row], dy[row + 1]);
                if s1x <= s0x || s1y <= s0y || d1x <= d0x || d1y <= d0y {
                    continue;
                }
                let piece = UiBox::new(d0x, d0y, d1x - 1, d1y - 1);
                let Some(clip) = dirty.intersection(&piece) else {
                    continue;
                };
                // The clipped part's source, in proportion to where it sits in the piece.
                let map = |v: i32, d0: i32, d1: i32, s0: i32, s1: i32| {
                    s0 + ((i64::from(v - d0) * i64::from(s1 - s0)) / i64::from(d1 - d0)) as i32
                };
                if smooth && (s1x - s0x != d1x - d0x || s1y - s0y != d1y - d0y) {
                    let full = UiBox::new(s0x, s0y, s1x - 1, s1y - 1);
                    self.blit_filtered(image, full, piece, clip, op);
                    continue;
                }
                let src = UiBox::new(
                    map(clip.x0, d0x, d1x, s0x, s1x),
                    map(clip.y0, d0y, d1y, s0y, s1y),
                    (map(clip.x1 + 1, d0x, d1x, s0x, s1x) - 1)
                        .max(map(clip.x0, d0x, d1x, s0x, s1x)),
                    (map(clip.y1 + 1, d0y, d1y, s0y, s1y) - 1)
                        .max(map(clip.y0, d0y, d1y, s0y, s1y)),
                );
                self.blit(image, src, clip, op);
            }
        }
    }
}

/// The modern style's own draw mode (no retail mode): the image composited over the surface by its
/// alpha, the surface's alpha accumulating (source over), so the client's translucent art (glass,
/// fades, glows) shows on a clear surface. Retail's modes 1 and 2 keep the surface's alpha.
pub const DRAW_OVER: u32 = 4;

/// One pixel of `SurfaceWindow::Blit`: `source` onto `dest` by `op`.
fn blend(dest: &mut [u8], source: &[u8], op: BlitOp) {
    let alpha = i32::from(source[3]);
    match op {
        BlitOp::Over => {
            let sa = f32::from(source[3]) / 255.0;
            if sa <= 0.0 {
                return;
            }
            let da = f32::from(dest[3]) / 255.0;
            let out = sa + da * (1.0 - sa);
            for c in 0..3 {
                let v = (f32::from(source[c]) * sa + f32::from(dest[c]) * da * (1.0 - sa)) / out;
                dest[c] = v.round().clamp(0.0, 255.0) as u8;
            }
            dest[3] = (out * 255.0).round() as u8;
        }
        BlitOp::Copy => dest.copy_from_slice(source),
        BlitOp::Value(value) => {
            // Copied, then the alpha byte times the value (`_ftol2`: truncated).
            dest.copy_from_slice(source);
            dest[3] = (f64::from(dest[3]) * f64::from(value)) as u8;
        }
        BlitOp::Channel if alpha == 0 => {}
        BlitOp::Channel if alpha == 255 => dest.copy_from_slice(source),
        BlitOp::Channel => {
            for c in 0..3 {
                let d = i32::from(dest[c]);
                // `(x + (x >> 31 & 0xff)) >> 8`: x / 256 truncated toward zero.
                dest[c] = (d - (d - i32::from(source[c])) * alpha / 256) as u8;
            }
        }
    }
}

/// `SurfaceWindow::Blit`'s flags as `Graphic::Draw` sets them.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BlitOp {
    Copy,
    /// 0x10: blended by the source alpha; the surface keeps its alpha.
    Channel,
    /// 0x20: copied with the alpha scaled by the value.
    Value(f32),
    /// [`DRAW_OVER`]: composited over by the source's alpha, the alpha accumulating.
    Over,
}

impl UiTree {
    /// `UIRegion::DrawHere` for element `index` whose parent's box starts at `origin` and whose
    /// ancestors clip to `clip` (surface coordinates).
    fn draw_here(
        &self,
        index: usize,
        origin: (i32, i32),
        clip: UiBox,
        images: &HashMap<u32, UiPixels>,
        surface: &mut UiSurface,
    ) {
        let el = self.el(index);
        if !el.visible {
            return;
        }
        let me = el.rect.offset(origin.0, origin.1);
        let Some(clip) = clip.intersection(&me) else {
            return;
        };
        if let Some(mode) = el.image {
            let smooth = self.smooth_scaling;
            let scale = matches!(
                el.attributes.get(&crate::token::SIZE_IMAGE_TO_OWNER),
                Some(crate::layout::AttributeValue::Bool(true))
            );
            if let Some(layers) = &el.icon_layers {
                if let Some(icon) = crate::icon_desc::composite(layers, images) {
                    surface.draw_graphic(&icon, mode, scale, smooth, el.alpha_value, me, clip);
                }
            } else if let Some(image) = images.get(&mode.file) {
                match el.attributes.get(&NINE_SLICE) {
                    Some(crate::layout::AttributeValue::Int(border)) => {
                        surface.draw_nine(image, mode, *border, smooth, el.alpha_value, me, clip)
                    }
                    _ => surface.draw_graphic(image, mode, scale, smooth, el.alpha_value, me, clip),
                }
            }
        }
        if let Some(text) = &el.text {
            surface.draw_text(text, me, clip, self.active == Some(index));
        }
        for &child in el.children.iter().rev() {
            self.draw_here(child, (me.x0, me.y0), clip, images, surface);
        }
        // `UIElement_Tree::DrawDone`: the connections over the items.
        if el.tree.is_some() {
            let (segments, color) = self.tree_segments(index);
            for (from, to) in segments {
                let at = |p: (i32, i32)| (p.0 + me.x0, p.1 + me.y0);
                let width = surface.width as usize;
                crate::tree_view::draw_line(
                    &mut surface.rgba,
                    width,
                    clip,
                    at(from),
                    at(to),
                    color,
                );
            }
        }
    }

    /// Whether scaled images are filtered (the modern style) or take the nearest pixel (retail).
    pub fn set_smooth_scaling(&mut self, smooth: bool) {
        self.smooth_scaling = smooth;
    }

    /// Every visible root drawn into its own surface, in the order they go over the scene
    /// (highest z first).
    pub fn compose(&mut self, images: &HashMap<u32, UiPixels>) -> Vec<UiSurface> {
        // `UIElement_Text::DrawStart`: lay out text that changed.
        for el in self.slots.iter_mut().flatten() {
            let (width, height) = (el.rect.width(), el.rect.height());
            if let Some(text) = el.text.as_mut() {
                text.recalculate(width, height);
            }
        }
        let roots: Vec<usize> = self.composed_roots().collect();
        roots
            .into_iter()
            .map(|root| self.compose_root(root, images))
            .collect()
    }

    /// One root drawn into its surface (`UIRegion::ForceDraw` of a root).
    pub fn compose_root(&mut self, root: usize, images: &HashMap<u32, UiPixels>) -> UiSurface {
        let el = self.el(root);
        let (width, height) = (el.rect.width(), el.rect.height());
        let (x, y) = el.object.unwrap_or((0, 0));
        let mut surface = UiSurface {
            root,
            x,
            y,
            width: width as u32,
            height: height as u32,
            rotation: el.rotation,
            shown: el.shown,
            opacity: el.opacity,
            rgba: vec![0; width as usize * height as usize * 4],
        };
        let bounds = surface.bounds();
        self.draw_here(root, (0, 0), bounds, images, &mut surface);
        surface
    }
}
