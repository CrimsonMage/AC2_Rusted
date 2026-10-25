use std::sync::Arc;

use crate::{
    decode_block_compressed_texture, RgbaTextureLevel, TextureAsset, TexturePayload,
    TextureWrapMode,
};

/// One composed terrain cell texture, RGBA8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedComposedTerrainTexture {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

struct DecodedAc2Surface {
    width: u32,
    height: u32,
    rgba: Arc<[u8]>,
}

/// Composes the four slope-selected AC2 vertex surfaces across one terrain polygon. AC2 computes
/// surface information per land vertex; interpolating those authored corner results removes the
/// hard material squares produced by treating each polygon as one direct texture.
pub fn compose_ac2_vertex_blended_terrain_texture(
    corners: [&TextureAsset; 4],
    texture_size: u32,
    texture_tiling: u32,
) -> Option<DecodedComposedTerrainTexture> {
    let decoded = corners
        .map(decoded_base_surface)
        .into_iter()
        .collect::<Option<Vec<_>>>()?;
    let size = usize::try_from(texture_size.max(2)).ok()?;
    let tiling = usize::try_from(texture_tiling.max(1)).ok()?;
    let mut rgba = vec![0_u8; size.checked_mul(size)?.checked_mul(4)?];
    for y in 0..size {
        let south = y as f32 / (size - 1) as f32;
        for x in 0..size {
            let east = x as f32 / (size - 1) as f32;
            let weights = [
                (1.0 - east) * south,
                east * south,
                (1.0 - east) * (1.0 - south),
                east * (1.0 - south),
            ];
            let mut output = [0.0_f32; 4];
            for (corner, weight) in decoded.iter().zip(weights) {
                let pixel = repeating_sample(corner, x, y, size, tiling);
                for channel in 0..4 {
                    output[channel] += f32::from(pixel[channel]) * weight;
                }
            }
            let offset = (y * size + x) * 4;
            for channel in 0..4 {
                rgba[offset + channel] = output[channel].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Some(DecodedComposedTerrainTexture {
        width: size as u32,
        height: size as u32,
        rgba: Arc::from(rgba),
    })
}

fn decoded_base_surface(asset: &TextureAsset) -> Option<DecodedAc2Surface> {
    match asset.payload() {
        TexturePayload::Rgba(levels) => {
            let level = levels.levels.first()?;
            Some(DecodedAc2Surface {
                width: level.width,
                height: level.height,
                rgba: Arc::clone(level.faces.first()?),
            })
        }
        TexturePayload::BlockCompressed(levels) => {
            let level = levels.levels.first()?;
            let rgba = decode_block_compressed_texture(
                levels.format,
                usize::try_from(level.width).ok()?,
                usize::try_from(level.height).ok()?,
                level.faces.first()?,
            )?;
            Some(DecodedAc2Surface {
                width: level.width,
                height: level.height,
                rgba: Arc::from(rgba),
            })
        }
    }
}

/// Box filter of the source texels covered by one output texel (at most 8 x 8 of them), so a
/// large source shrinks into the cell texture without baked-in aliasing.
fn repeating_sample(
    surface: &DecodedAc2Surface,
    x: usize,
    y: usize,
    output_size: usize,
    tiling: usize,
) -> [u8; 4] {
    let width = surface.width as usize;
    let height = surface.height as usize;
    let footprint_x = (tiling.saturating_mul(width) / output_size).clamp(1, MAX_FOOTPRINT);
    let footprint_y = (tiling.saturating_mul(height) / output_size).clamp(1, MAX_FOOTPRINT);
    let source_x = x.saturating_mul(tiling).saturating_mul(width) / output_size;
    let source_y = y.saturating_mul(tiling).saturating_mul(height) / output_size;
    let mut sum = [0_u32; 4];
    for dy in 0..footprint_y {
        let row = (source_y + dy) % height * width;
        for dx in 0..footprint_x {
            let offset = (row + (source_x + dx) % width) * 4;
            for (total, &texel) in sum.iter_mut().zip(&surface.rgba[offset..offset + 4]) {
                *total += u32::from(texel);
            }
        }
    }
    let count = (footprint_x * footprint_y) as u32;
    sum.map(|total| ((total + count / 2) / count) as u8)
}

const MAX_FOOTPRINT: usize = 8;

/// A clamped RGBA texture with its complete box-filtered mip chain down to 1 x 1, for composed
/// terrain textures that minify across the whole view (a single level shimmers in motion).
pub fn ac2_mipmapped_clamped_texture(
    width: u32,
    height: u32,
    rgba: Arc<[u8]>,
) -> Option<TextureAsset> {
    let mut levels = vec![RgbaTextureLevel {
        width,
        height,
        faces: Arc::from([Arc::clone(&rgba)]),
    }];
    let (mut w, mut h, mut current) = (width as usize, height as usize, rgba);
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![0_u8; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                let taps = [
                    (2 * x, 2 * y),
                    (2 * x + 1, 2 * y),
                    (2 * x, 2 * y + 1),
                    (2 * x + 1, 2 * y + 1),
                ];
                for channel in 0..4 {
                    let total: u32 = taps
                        .iter()
                        .map(|&(sx, sy)| {
                            u32::from(current[(sy.min(h - 1) * w + sx.min(w - 1)) * 4 + channel])
                        })
                        .sum();
                    next[(y * nw + x) * 4 + channel] = ((total + 2) / 4) as u8;
                }
            }
        }
        current = Arc::from(next);
        (w, h) = (nw, nh);
        levels.push(RgbaTextureLevel {
            width: w as u32,
            height: h as u32,
            faces: Arc::from([Arc::clone(&current)]),
        });
    }
    TextureAsset::rgba_2d(width, height, Arc::from(levels), TextureWrapMode::Clamp)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::compose_ac2_vertex_blended_terrain_texture;
    use crate::TextureAsset;

    #[test]
    fn preserves_authored_corner_colors_and_blends_the_center() {
        let assets = [
            solid([255, 0, 0, 255]),
            solid([0, 255, 0, 255]),
            solid([0, 0, 255, 255]),
            solid([255, 255, 255, 255]),
        ];
        let texture = compose_ac2_vertex_blended_terrain_texture(
            [&assets[0], &assets[1], &assets[2], &assets[3]],
            3,
            1,
        )
        .unwrap();
        assert_eq!(&texture.rgba[0..4], &[0, 0, 255, 255]);
        assert_eq!(&texture.rgba[16..20], &[128, 128, 128, 255]);
        assert_eq!(&texture.rgba[24..28], &[255, 0, 0, 255]);
    }

    fn solid(color: [u8; 4]) -> TextureAsset {
        TextureAsset::clamped(1, 1, Arc::from(color))
    }
}
