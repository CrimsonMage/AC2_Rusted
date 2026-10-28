//! `UIElement::SetIconDesc` (admin 0x005E9AB0): an item's icon drawn from its icon description,
//! every layer bottom first. Each layer is its image times its shift colour
//! (`Graphic::InitFromIconLayerDesc`: a copy, then `SurfaceWindow::Multiply`); the first is the
//! icon, and each later one is blended over it by its alpha (`Graphic::Incorporate`, blit flags
//! 0x14: the alpha blend, and a format conversion that does nothing between RGBA images).
//!
//! `Multiply` takes each channel as a float (`RGBAColor::SetColor32`: the byte times 1/255), times
//! the shift colour's (packed ARGB, the same conversion), and stores `trunc(product * 255)`. A
//! layer whose image is not loaded yet is left out until it is.

use std::collections::HashMap;
use std::sync::Arc;

use crate::compose::UiPixels;
use crate::element::UiTree;

/// One `IconLayerDesc`: its image and its shift colour (packed ARGB, as stored).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconLayer {
    pub image: u32,
    pub shift_color: u32,
}

/// `RGBAColor::SetColor32`'s scale.
const INV_255: f32 = 0.003_921_569;

fn channel(byte: u8) -> f32 {
    (f64::from(byte) * f64::from(INV_255)) as f32
}

/// `SurfaceWindow::Multiply` of the RGBA pixels by the ARGB colour.
fn multiply(rgba: &mut [u8], shift: u32) {
    let s = [
        channel((shift >> 16) as u8),
        channel((shift >> 8) as u8),
        channel(shift as u8),
        channel((shift >> 24) as u8),
    ];
    for pixel in rgba.chunks_exact_mut(4) {
        for (c, k) in pixel.iter_mut().zip(s) {
            let product = (f64::from(channel(*c)) * f64::from(k)) as f32;
            *c = (f64::from(product) * 255.0) as u8;
        }
    }
}

/// The icon the layers make, or `None` while the first layer's image is not loaded.
pub(crate) fn composite(layers: &[IconLayer], images: &HashMap<u32, UiPixels>) -> Option<UiPixels> {
    let (first, rest) = layers.split_first()?;
    let base = images.get(&first.image)?;
    let (width, height) = (base.width as usize, base.height as usize);
    let mut rgba = base.rgba.to_vec();
    multiply(&mut rgba, first.shift_color);
    for layer in rest {
        let Some(image) = images.get(&layer.image) else {
            continue;
        };
        let mut over = image.rgba.to_vec();
        multiply(&mut over, layer.shift_color);
        let (ow, oh) = (image.width as usize, image.height as usize);
        if ow == 0 || oh == 0 {
            continue;
        }
        for y in 0..height {
            for x in 0..width {
                // Sizes that differ pick the nearest source pixel, as the surface blit does.
                let (sx, sy) = (x * ow / width, y * oh / height);
                let s = (sy * ow + sx) * 4;
                let d = (y * width + x) * 4;
                let alpha = i32::from(over[s + 3]);
                if alpha == 0 {
                    continue;
                }
                if alpha == 255 {
                    rgba[d..d + 4].copy_from_slice(&over[s..s + 4]);
                    continue;
                }
                for c in 0..3 {
                    let dest = i32::from(rgba[d + c]);
                    rgba[d + c] = (dest - (dest - i32::from(over[s + c])) * alpha / 256) as u8;
                }
            }
        }
    }
    Some(UiPixels {
        width: base.width,
        height: base.height,
        rgba: Arc::from(rgba),
    })
}

impl UiTree {
    /// `UIElement::SetIconDesc`: the element shows the layers' icon (draw mode 0). No layers, or
    /// a first layer with no image (`InitFromIconLayerDesc` fails on it), leave the element as it
    /// was; a later layer with no image is left out.
    pub fn set_icon_desc(&mut self, index: usize, layers: &[IconLayer]) {
        let Some(first) = layers.first().filter(|l| l.image != 0) else {
            return;
        };
        self.set_image(index, first.image, 0);
        self.el_mut(index).icon_layers = Some(Arc::from(layers));
    }
}

#[cfg(test)]
#[path = "icon_desc_tests.rs"]
mod tests;
