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
//! pixel.

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
                let source = &image.rgba[s..s + 4];
                let dest = &mut self.rgba[d..d + 4];
                let alpha = i32::from(source[3]);
                match op {
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
            _ => BlitOp::Copy,
        };
        let local = dirty.offset(-me.x0, -me.y0);
        if w == iw && h == ih {
            self.blit(image, local, dirty, op);
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

/// `SurfaceWindow::Blit`'s flags as `Graphic::Draw` sets them.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BlitOp {
    Copy,
    /// 0x10: blended by the source alpha; the surface keeps its alpha.
    Channel,
    /// 0x20: copied with the alpha scaled by the value.
    Value(f32),
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
            if let Some(image) = images.get(&mode.file) {
                let scale = matches!(
                    el.attributes.get(&crate::token::SIZE_IMAGE_TO_OWNER),
                    Some(crate::layout::AttributeValue::Bool(true))
                );
                surface.draw_graphic(image, mode, scale, el.alpha_value, me, clip);
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
            .map(|root| {
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
            })
            .collect()
    }
}
