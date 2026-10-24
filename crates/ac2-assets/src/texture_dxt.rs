use ac2_core::PixelFormatID;

/// Hardware block format compatible with a retained retail DXT payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockCompressedTextureFormat {
    Bc1Rgba,
    Bc2Rgba,
    Bc3Rgba,
}

/// Validated DXT blocks that can be uploaded directly to a BC-capable GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockCompressedTexture {
    format: BlockCompressedTextureFormat,
    source_format: PixelFormatID,
    blocks: std::sync::Arc<[u8]>,
}

/// Decodes a previously validated BC payload for a backend without native BC support.
pub fn decode_block_compressed_texture(
    format: BlockCompressedTextureFormat,
    width: usize,
    height: usize,
    blocks: &[u8],
) -> Option<Vec<u8>> {
    let source = match format {
        BlockCompressedTextureFormat::Bc1Rgba => PixelFormatID::PFID_DXT1,
        BlockCompressedTextureFormat::Bc2Rgba => PixelFormatID::PFID_DXT3,
        BlockCompressedTextureFormat::Bc3Rgba => PixelFormatID::PFID_DXT5,
    };
    decode(source, width, height, blocks)
}

impl BlockCompressedTexture {
    pub const fn format(&self) -> BlockCompressedTextureFormat {
        self.format
    }

    pub(crate) fn blocks_ref(&self) -> &std::sync::Arc<[u8]> {
        &self.blocks
    }

    pub fn blocks(&self) -> &[u8] {
        &self.blocks
    }

    /// Immutable backing retained by the render codec when an entire authored chain can stay
    /// native.  This keeps the resource layer responsible only for validating source blocks.
    pub fn blocks_arc(&self) -> std::sync::Arc<[u8]> {
        std::sync::Arc::clone(&self.blocks)
    }

    pub fn decode_rgba(&self, width: usize, height: usize) -> Option<Vec<u8>> {
        decode(self.source_format, width, height, self.blocks())
    }
}

pub(super) fn retain(
    source_format: PixelFormatID,
    width: usize,
    height: usize,
    source: &std::sync::Arc<[u8]>,
) -> Option<BlockCompressedTexture> {
    let format = format_for(source_format)?;
    let blocks_wide = width.checked_add(3)?.checked_div(4)?;
    let blocks_high = height.checked_add(3)?.checked_div(4)?;
    let block_count = blocks_wide.checked_mul(blocks_high)?;
    if source.len() != block_count.checked_mul(block_byte_len(format))? {
        return None;
    }
    Some(BlockCompressedTexture {
        format,
        source_format,
        blocks: std::sync::Arc::clone(source),
    })
}

pub(super) fn decode(
    format: PixelFormatID,
    width: usize,
    height: usize,
    source: &[u8],
) -> Option<Vec<u8>> {
    let block_bytes = block_byte_len(format_for(format)?);
    let blocks_wide = width.checked_add(3)? / 4;
    let blocks_high = height.checked_add(3)? / 4;
    let block_count = blocks_wide.checked_mul(blocks_high)?;
    if source.len() != block_count.checked_mul(block_bytes)? {
        return None;
    }

    let mut rgba = vec![0_u8; width.checked_mul(height)?.checked_mul(4)?];
    if format == PixelFormatID::PFID_DXT1 && width % 4 == 0 && height % 4 == 0 {
        for block_y in 0..blocks_high {
            for block_x in 0..blocks_wide {
                let offset = (block_y * blocks_wide + block_x) * 8;
                let block = &source[offset..offset + 8];
                let colors = decode_color_palette(block, false);
                let indices = u32::from_le_bytes(block[4..8].try_into().ok()?);
                for pixel_y in 0..4 {
                    let row_offset = ((block_y * 4 + pixel_y) * width + block_x * 4) * 4;
                    let row = &mut rgba[row_offset..row_offset + 16];
                    let row_indices = indices >> (pixel_y * 8);
                    for pixel_x in 0..4 {
                        let color = &colors[((row_indices >> (pixel_x * 2)) & 3) as usize];
                        row[pixel_x * 4..pixel_x * 4 + 4].copy_from_slice(color);
                    }
                }
            }
        }
        return Some(rgba);
    }
    for block_y in 0..blocks_high {
        for block_x in 0..blocks_wide {
            let offset = (block_y * blocks_wide + block_x) * block_bytes;
            let block = &source[offset..offset + block_bytes];
            let (alpha, color_block, force_four_colors) = match format {
                PixelFormatID::PFID_DXT1 => (None, block, false),
                PixelFormatID::PFID_DXT2 | PixelFormatID::PFID_DXT3 => {
                    (Some(decode_dxt3_alpha(&block[..8])), &block[8..], true)
                }
                PixelFormatID::PFID_DXT4 | PixelFormatID::PFID_DXT5 => {
                    (Some(decode_dxt5_alpha(&block[..8])), &block[8..], true)
                }
                _ => return None,
            };
            let colors = decode_color_palette(color_block, force_four_colors);
            let indices = u32::from_le_bytes(color_block[4..8].try_into().ok()?);
            for pixel in 0..16 {
                let x = block_x * 4 + pixel % 4;
                let y = block_y * 4 + pixel / 4;
                if x >= width || y >= height {
                    continue;
                }
                let color = colors[((indices >> (2 * pixel)) & 3) as usize];
                let output = (y * width + x) * 4;
                rgba[output..output + 4].copy_from_slice(&[
                    color[0],
                    color[1],
                    color[2],
                    alpha.map_or(color[3], |values| values[pixel]),
                ]);
            }
        }
    }
    Some(rgba)
}

const fn format_for(format: PixelFormatID) -> Option<BlockCompressedTextureFormat> {
    match format {
        PixelFormatID::PFID_DXT1 => Some(BlockCompressedTextureFormat::Bc1Rgba),
        PixelFormatID::PFID_DXT2 | PixelFormatID::PFID_DXT3 => {
            Some(BlockCompressedTextureFormat::Bc2Rgba)
        }
        PixelFormatID::PFID_DXT4 | PixelFormatID::PFID_DXT5 => {
            Some(BlockCompressedTextureFormat::Bc3Rgba)
        }
        _ => None,
    }
}

const fn block_byte_len(format: BlockCompressedTextureFormat) -> usize {
    match format {
        BlockCompressedTextureFormat::Bc1Rgba => 8,
        BlockCompressedTextureFormat::Bc2Rgba | BlockCompressedTextureFormat::Bc3Rgba => 16,
    }
}

fn decode_color_palette(block: &[u8], force_four_colors: bool) -> [[u8; 4]; 4] {
    let first_value = u16::from_le_bytes([block[0], block[1]]);
    let second_value = u16::from_le_bytes([block[2], block[3]]);
    let first = rgb565(first_value);
    let second = rgb565(second_value);
    let mut colors = [first, second, [0; 4], [0; 4]];
    if force_four_colors || first_value > second_value {
        colors[2] = blend(first, second, 2, 1, 3);
        colors[3] = blend(first, second, 1, 2, 3);
    } else {
        colors[2] = blend(first, second, 1, 1, 2);
        colors[3] = [0, 0, 0, 0];
    }
    colors
}

fn rgb565(value: u16) -> [u8; 4] {
    let red = ((value >> 11) & 0x1f) as u8;
    let green = ((value >> 5) & 0x3f) as u8;
    let blue = (value & 0x1f) as u8;
    [
        (red << 3) | (red >> 2),
        (green << 2) | (green >> 4),
        (blue << 3) | (blue >> 2),
        255,
    ]
}

fn blend(
    first: [u8; 4],
    second: [u8; 4],
    first_weight: u16,
    second_weight: u16,
    divisor: u16,
) -> [u8; 4] {
    [
        ((u16::from(first[0]) * first_weight + u16::from(second[0]) * second_weight) / divisor)
            as u8,
        ((u16::from(first[1]) * first_weight + u16::from(second[1]) * second_weight) / divisor)
            as u8,
        ((u16::from(first[2]) * first_weight + u16::from(second[2]) * second_weight) / divisor)
            as u8,
        255,
    ]
}

fn decode_dxt3_alpha(block: &[u8]) -> [u8; 16] {
    let alpha_bits = u64::from_le_bytes(block.try_into().expect("DXT3 alpha block is eight bytes"));
    std::array::from_fn(|pixel| (((alpha_bits >> (4 * pixel)) & 0x0f) as u8) * 17)
}

fn decode_dxt5_alpha(block: &[u8]) -> [u8; 16] {
    let mut palette = [0_u8; 8];
    palette[0] = block[0];
    palette[1] = block[1];
    if palette[0] > palette[1] {
        for index in 2..8 {
            palette[index] = (((8 - index) as u16 * u16::from(palette[0])
                + (index - 1) as u16 * u16::from(palette[1]))
                / 7) as u8;
        }
    } else {
        for index in 2..6 {
            palette[index] = (((6 - index) as u16 * u16::from(palette[0])
                + (index - 1) as u16 * u16::from(palette[1]))
                / 5) as u8;
        }
        palette[6] = 0;
        palette[7] = 255;
    }
    let bits = block[2..8]
        .iter()
        .enumerate()
        .fold(0_u64, |bits, (index, byte)| {
            bits | (u64::from(*byte) << (8 * index))
        });
    std::array::from_fn(|pixel| palette[((bits >> (3 * pixel)) & 7) as usize])
}

#[cfg(test)]
mod tests {
    use super::decode;
    use ac2_core::PixelFormatID;

    #[test]
    fn dxt1_decodes_rgb565_blocks() {
        let rgba = decode(
            PixelFormatID::PFID_DXT1,
            4,
            4,
            &[0x00, 0xf8, 0xe0, 0x07, 0, 0, 0, 0],
        )
        .expect("DXT1 block should decode");

        assert_eq!(rgba.len(), 4 * 4 * 4);
        assert!(rgba.chunks_exact(4).all(|pixel| pixel == [255, 0, 0, 255]));
    }

    #[test]
    fn dxt5_decodes_explicit_alpha_indices() {
        let rgba = decode(
            PixelFormatID::PFID_DXT5,
            4,
            4,
            &[255, 0, 1, 0, 0, 0, 0, 0, 0x00, 0xf8, 0xe0, 0x07, 0, 0, 0, 0],
        )
        .expect("DXT5 block should decode");

        assert_eq!(&rgba[..4], &[255, 0, 0, 0]);
        assert_eq!(&rgba[4..8], &[255, 0, 0, 255]);
    }

    #[test]
    fn retains_valid_dxt_blocks_without_expanding_them() {
        let blocks: std::sync::Arc<[u8]> = [0x00, 0xf8, 0xe0, 0x07, 0, 0, 0, 0].into();
        let retained = super::retain(PixelFormatID::PFID_DXT1, 4, 4, &blocks)
            .expect("DXT1 payload should retain");

        assert_eq!(
            retained.format(),
            super::BlockCompressedTextureFormat::Bc1Rgba
        );
        assert_eq!(retained.blocks(), blocks.as_ref());
        assert!(std::sync::Arc::ptr_eq(&retained.blocks_arc(), &blocks));
        assert_eq!(
            retained
                .decode_rgba(4, 4)
                .expect("retained blocks should decode"),
            decode(PixelFormatID::PFID_DXT1, 4, 4, &blocks).expect("DXT1 should decode")
        );
    }
}
