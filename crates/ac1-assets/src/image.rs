//! AC1 `RenderSurface`s as RGBA8 mip chains: the pixel formats a creature's textures use, indexed
//! ones through the object's composed palette (`ImgTex::CopyIntoData`, which also clears the
//! indices below eight of a clip-map surface).

use std::sync::Arc;

use ac2_assets::{decode_block_compressed_texture, BlockCompressedTextureFormat};
use ac2_render::{Image, ImageLevel};

use crate::formats::{pixel_format as pf, RenderSurface};

/// `surface` as RGBA8 (`None` for a format this does not decode), indexed formats looked up in
/// `palette` (ARGB) and, for a clip map, indices below eight cleared.
pub(crate) fn decode_rgba(
    surface: &RenderSurface,
    palette: &[u32],
    clip_map: bool,
) -> Option<Vec<u8>> {
    let pixels = (surface.width * surface.height) as usize;
    let data = &surface.data;
    let mut rgba = Vec::with_capacity(pixels * 4);
    let argb = |color: u32| {
        [
            (color >> 16) as u8,
            (color >> 8) as u8,
            color as u8,
            (color >> 24) as u8,
        ]
    };
    match surface.format {
        pf::INDEX16 | pf::P8 => {
            let wide = surface.format == pf::INDEX16;
            let step = if wide { 2 } else { 1 };
            if data.len() < pixels * step {
                return None;
            }
            for i in 0..pixels {
                let index = if wide {
                    u16::from_le_bytes([data[2 * i], data[2 * i + 1]]) as usize
                } else {
                    data[i] as usize
                };
                let color = if clip_map && index < 8 {
                    [0; 4]
                } else {
                    argb(palette.get(index).copied().unwrap_or(0xFFFF_00FF))
                };
                rgba.extend_from_slice(&color);
            }
        }
        pf::R8G8B8 => {
            for px in data.chunks_exact(3).take(pixels) {
                rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
            }
        }
        pf::A8R8G8B8 | pf::X8R8G8B8 => {
            let opaque = surface.format == pf::X8R8G8B8;
            for px in data.chunks_exact(4).take(pixels) {
                rgba.extend_from_slice(&[px[2], px[1], px[0], if opaque { 255 } else { px[3] }]);
            }
        }
        pf::R5G6B5 => {
            for px in data.chunks_exact(2).take(pixels) {
                let v = u16::from_le_bytes([px[0], px[1]]);
                let (r, g, b) = ((v >> 11) & 31, (v >> 5) & 63, v & 31);
                rgba.extend_from_slice(&[
                    (r * 255 / 31) as u8,
                    (g * 255 / 63) as u8,
                    (b * 255 / 31) as u8,
                    255,
                ]);
            }
        }
        pf::A4R4G4B4 => {
            for px in data.chunks_exact(2).take(pixels) {
                let v = u16::from_le_bytes([px[0], px[1]]);
                let n = |shift: u16| (((v >> shift) & 15) * 17) as u8;
                rgba.extend_from_slice(&[n(8), n(4), n(0), n(12)]);
            }
        }
        pf::A8 => {
            for a in data.iter().take(pixels) {
                rgba.extend_from_slice(&[255, 255, 255, *a]);
            }
        }
        pf::DXT1 | pf::DXT3 | pf::DXT5 => {
            let format = match surface.format {
                pf::DXT1 => BlockCompressedTextureFormat::Bc1Rgba,
                pf::DXT3 => BlockCompressedTextureFormat::Bc2Rgba,
                _ => BlockCompressedTextureFormat::Bc3Rgba,
            };
            return decode_block_compressed_texture(
                format,
                surface.width as usize,
                surface.height as usize,
                data,
            );
        }
        _ => return None,
    }
    (rgba.len() == pixels * 4).then_some(rgba)
}

/// An RGBA8 image with its mip chain down to 1 x 1 (each level a 2 x 2 box of the one above).
pub(crate) fn mip_chain(width: u32, height: u32, rgba: Vec<u8>) -> Image {
    let mut levels = vec![ImageLevel {
        width,
        height,
        rgba: Arc::from(rgba),
    }];
    while let Some(last) = levels.last() {
        if last.width == 1 && last.height == 1 {
            break;
        }
        let (w, h) = ((last.width / 2).max(1), (last.height / 2).max(1));
        let mut next = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                for c in 0..4 {
                    let mut sum = 0u32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let sx = (x * 2 + dx).min(last.width - 1);
                        let sy = (y * 2 + dy).min(last.height - 1);
                        sum += u32::from(last.rgba[((sy * last.width + sx) * 4 + c) as usize]);
                    }
                    next[((y * w + x) * 4 + c) as usize] = (sum / 4) as u8;
                }
            }
        }
        levels.push(ImageLevel {
            width: w,
            height: h,
            rgba: Arc::from(next),
        });
    }
    Image::rgba(levels)
}

/// A 1 x 1 image of an ARGB colour (a `Base1Solid` surface).
pub(crate) fn solid(color: u32) -> Image {
    let rgba = vec![(color >> 16) as u8, (color >> 8) as u8, color as u8, 255];
    mip_chain(1, 1, rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface(format: u32, width: u32, height: u32, data: Vec<u8>) -> RenderSurface {
        RenderSurface {
            width,
            height,
            format,
            data,
            palette: 0,
        }
    }

    #[test]
    fn indexed_pixels_read_the_palette_and_a_clip_map_clears_the_first_eight() {
        let palette: Vec<u32> = (0..16).map(|i| 0xFF00_0000 | (i * 0x10_1010)).collect();
        let image = surface(pf::INDEX16, 2, 1, vec![3, 0, 9, 0]);
        let rgba = decode_rgba(&image, &palette, false).unwrap();
        assert_eq!(rgba, vec![0x30, 0x30, 0x30, 255, 0x90, 0x90, 0x90, 255]);
        let clipped = decode_rgba(&image, &palette, true).unwrap();
        assert_eq!(&clipped[..4], &[0; 4]);
        assert_eq!(&clipped[4..], &[0x90, 0x90, 0x90, 255]);
    }

    #[test]
    fn the_mip_chain_halves_to_one_pixel() {
        let image = mip_chain(4, 2, vec![200; 4 * 2 * 4]);
        let sizes: Vec<_> = image.levels.iter().map(|l| (l.width, l.height)).collect();
        assert_eq!(sizes, vec![(4, 2), (2, 1), (1, 1)]);
        assert_eq!(image.levels[2].rgba[0], 200);
    }
}
