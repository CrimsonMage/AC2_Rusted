//! Pixel payloads of AC2 `RenderSurface` records. The Legions DATs use DXT1/DXT3 FourCCs, the
//! D3D formats R8G8B8 (20), A8R8G8B8 (21) and A8 (28), and Turbine's JFIF format (500, whose
//! record carries 0 x 0 and takes its size from the JPEG).

use std::io::Cursor;
use std::sync::Arc;

use jpeg_decoder::{Decoder as JpegDecoder, PixelFormat as JpegPixelFormat};

use crate::BlockCompressedTextureFormat;

const FORMAT_R8G8B8: u32 = 20;
const FORMAT_A8R8G8B8: u32 = 21;
const FORMAT_A8: u32 = 28;
const FORMAT_JFIF: u32 = 500;
const FOURCC_DXT1: u32 = u32::from_le_bytes(*b"DXT1");
const FOURCC_DXT2: u32 = u32::from_le_bytes(*b"DXT2");
const FOURCC_DXT3: u32 = u32::from_le_bytes(*b"DXT3");
const FOURCC_DXT4: u32 = u32::from_le_bytes(*b"DXT4");
const FOURCC_DXT5: u32 = u32::from_le_bytes(*b"DXT5");

/// GPU-ready pixels of one surface level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Ac2SurfacePixels {
    BlockCompressed(BlockCompressedTextureFormat, Arc<[u8]>),
    Rgba(Arc<[u8]>),
}

impl Ac2SurfacePixels {
    pub(crate) fn bytes(&self) -> &Arc<[u8]> {
        match self {
            Self::BlockCompressed(_, bytes) | Self::Rgba(bytes) => bytes,
        }
    }

    /// Whether two levels can share one mip chain.
    pub(crate) fn same_kind(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BlockCompressed(a, _), Self::BlockCompressed(b, _)) => a == b,
            (Self::Rgba(_), Self::Rgba(_)) => true,
            _ => false,
        }
    }
}

/// Converts a surface's image bytes to GPU-ready pixels, returning the real dimensions.
pub(crate) fn decode_surface_pixels(
    format: u32,
    width: u32,
    height: u32,
    bytes: &[u8],
) -> Result<(u32, u32, Ac2SurfacePixels), &'static str> {
    if format == FORMAT_JFIF {
        let (width, height, rgba) =
            decode_raw_jpeg_data(bytes).ok_or("AC2 JFIF surface did not decode")?;
        return Ok((width, height, Ac2SurfacePixels::Rgba(rgba)));
    }
    if width == 0 || height == 0 {
        return Err("AC2 surface has zero dimensions");
    }
    let block = |format, block_bytes: usize| {
        let blocks = (width.div_ceil(4) as usize).checked_mul(height.div_ceil(4) as usize);
        if blocks.and_then(|count| count.checked_mul(block_bytes)) != Some(bytes.len()) {
            return Err("AC2 DXT block count did not match dimensions");
        }
        Ok(Ac2SurfacePixels::BlockCompressed(format, Arc::from(bytes)))
    };
    let pixels = match format {
        FOURCC_DXT1 => block(BlockCompressedTextureFormat::Bc1Rgba, 8)?,
        FOURCC_DXT2 | FOURCC_DXT3 => block(BlockCompressedTextureFormat::Bc2Rgba, 16)?,
        FOURCC_DXT4 | FOURCC_DXT5 => block(BlockCompressedTextureFormat::Bc3Rgba, 16)?,
        FORMAT_R8G8B8 => expand(width, height, bytes, 3, |p| [p[2], p[1], p[0], 255])?,
        FORMAT_A8R8G8B8 => expand(width, height, bytes, 4, |p| [p[2], p[1], p[0], p[3]])?,
        // D3D samples A8 as black with alpha.
        FORMAT_A8 => expand(width, height, bytes, 1, |p| [0, 0, 0, p[0]])?,
        _ => return Err("unsupported AC2 surface pixel format"),
    };
    Ok((width, height, pixels))
}

/// Expands little-endian D3D texels (B, G, R[, A] byte order) to RGBA8.
fn expand(
    width: u32,
    height: u32,
    bytes: &[u8],
    texel_bytes: usize,
    convert: impl Fn(&[u8]) -> [u8; 4],
) -> Result<Ac2SurfacePixels, &'static str> {
    let texels = (width as usize).checked_mul(height as usize);
    if texels.and_then(|count| count.checked_mul(texel_bytes)) != Some(bytes.len()) {
        return Err("AC2 surface byte count did not match dimensions");
    }
    let mut rgba = Vec::with_capacity(bytes.len() / texel_bytes * 4);
    for texel in bytes.chunks_exact(texel_bytes) {
        rgba.extend_from_slice(&convert(texel));
    }
    Ok(Ac2SurfacePixels::Rgba(Arc::from(rgba)))
}

/// Largest decoded JFIF surface accepted, in RGBA bytes.
const MAX_DECODED_JPEG_BYTES: usize = 16 * 1024 * 1024;

/// Turbine's JFIF surface payload (format 500) as RGBA8.
fn decode_raw_jpeg_data(source: &[u8]) -> Option<(u32, u32, Arc<[u8]>)> {
    let mut decoder = JpegDecoder::new(Cursor::new(source));
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    let width = u32::from(info.width);
    let height = u32::from(info.height);
    (width > 0 && height > 0).then_some(())?;
    let (pixel_count, rgba_bytes) = decoded_raster_layout(width, height)?;
    let decoded_bytes = match info.pixel_format {
        JpegPixelFormat::L8 => pixel_count,
        JpegPixelFormat::RGB24 => pixel_count.checked_mul(3)?,
        JpegPixelFormat::CMYK32 => rgba_bytes,
        _ => return None,
    };
    decoder.set_max_decoding_buffer_size(decoded_bytes);
    let pixels = decoder.decode().ok()?;
    (pixels.len() == decoded_bytes).then_some(())?;
    let mut rgba = rgba_buffer(rgba_bytes)?;
    match info.pixel_format {
        JpegPixelFormat::L8 => {
            for value in pixels {
                rgba.extend_from_slice(&[value, value, value, 255]);
            }
        }
        JpegPixelFormat::RGB24 => {
            for pixel in pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
        }
        JpegPixelFormat::CMYK32 => {
            for pixel in pixels.chunks_exact(4) {
                let c = u16::from(pixel[0]);
                let m = u16::from(pixel[1]);
                let y = u16::from(pixel[2]);
                let k = u16::from(pixel[3]);
                rgba.extend_from_slice(&[
                    255u16.saturating_sub((c + k).min(255)) as u8,
                    255u16.saturating_sub((m + k).min(255)) as u8,
                    255u16.saturating_sub((y + k).min(255)) as u8,
                    255,
                ]);
            }
        }
        _ => return None,
    }
    (rgba.len() == rgba_bytes).then_some(())?;
    Some((width, height, Arc::from(rgba)))
}

fn decoded_raster_layout(width: u32, height: u32) -> Option<(usize, usize)> {
    let pixels = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?;
    let rgba_bytes = pixels.checked_mul(4)?;
    (rgba_bytes <= MAX_DECODED_JPEG_BYTES).then_some((pixels, rgba_bytes))
}

fn rgba_buffer(rgba_bytes: usize) -> Option<Vec<u8>> {
    let mut rgba = Vec::new();
    rgba.try_reserve_exact(rgba_bytes).ok()?;
    Some(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d3d_texels_expand_to_rgba() {
        let (_, _, rgb) = decode_surface_pixels(FORMAT_R8G8B8, 1, 1, &[1, 2, 3]).unwrap();
        assert_eq!(&**rgb.bytes(), &[3, 2, 1, 255]);
        let (_, _, argb) = decode_surface_pixels(FORMAT_A8R8G8B8, 1, 1, &[1, 2, 3, 4]).unwrap();
        assert_eq!(&**argb.bytes(), &[3, 2, 1, 4]);
        let (_, _, alpha) = decode_surface_pixels(FORMAT_A8, 1, 1, &[9]).unwrap();
        assert_eq!(&**alpha.bytes(), &[0, 0, 0, 9]);
        assert!(decode_surface_pixels(FORMAT_A8R8G8B8, 2, 2, &[0; 4]).is_err());
    }
}
