//! Physical residency is a suffix of the authored mip chain, not just a texture view.
use super::*;

impl TextureAsset {
    /// Authored mip corresponding to the first physically resident level.
    pub const fn source_base_mip(&self) -> u8 {
        self.source_base_mip
    }

    /// Retains the requested authored mip and its tail. Discarded levels and decoded-source
    /// owners are not retained. Raising quality requires reloading the source, not this method.
    pub fn mip_tail(&self, requested_base: u8) -> Self {
        let mut drop = usize::from(requested_base.saturating_sub(self.source_base_mip))
            .min(self.level_count().saturating_sub(1));
        // BC top-level allocations must be block aligned. Smaller authored mips remain
        // selectable views within the tail, even when they cannot become its allocation root.
        if let TexturePayload::BlockCompressed(levels) = &self.payload {
            while drop > 0
                && (levels.levels[drop].width % 4 != 0 || levels.levels[drop].height % 4 != 0)
            {
                drop -= 1;
            }
        }
        if drop == 0 {
            return self.clone();
        }
        let mut result = self.clone();
        result.payload = match &self.payload {
            TexturePayload::Rgba(levels) => TexturePayload::Rgba(RgbaTextureLevels {
                levels: Arc::from(&levels.levels[drop..]),
            }),
            TexturePayload::BlockCompressed(levels) => {
                TexturePayload::BlockCompressed(CompressedTextureLevels {
                    format: levels.format,
                    levels: Arc::from(&levels.levels[drop..]),
                })
            }
        };
        let (width, height, faces) = match &result.payload {
            TexturePayload::Rgba(levels) => (
                levels.levels[0].width,
                levels.levels[0].height,
                levels.levels[0].faces.len(),
            ),
            TexturePayload::BlockCompressed(levels) => (
                levels.levels[0].width,
                levels.levels[0].height,
                levels.levels[0].faces.len(),
            ),
        };
        result.width = width;
        result.height = height;
        result.source_base_mip = self.source_base_mip.saturating_add(drop as u8);
        result.content_fingerprints = Arc::from(&self.content_fingerprints[drop * faces..]);
        result.decoded_levels = None;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trimming_releases_discarded_pixels_and_clamps_to_one_level() {
        let high: Arc<[u8]> = Arc::from([1; 64]);
        let weak = Arc::downgrade(&high);
        let source = TextureAsset::rgba_2d(
            4,
            4,
            Arc::from([
                RgbaTextureLevel {
                    width: 4,
                    height: 4,
                    faces: Arc::from([high]),
                },
                RgbaTextureLevel {
                    width: 2,
                    height: 2,
                    faces: Arc::from([Arc::from([2; 16])]),
                },
                RgbaTextureLevel {
                    width: 1,
                    height: 1,
                    faces: Arc::from([Arc::from([3; 4])]),
                },
            ]),
            TextureWrapMode::Repeat,
        )
        .unwrap();
        let tail = source.mip_tail(1);
        assert_eq!(
            (tail.width, tail.level_count(), tail.source_base_mip()),
            (2, 2, 1)
        );
        assert_eq!(tail.payload_byte_len(), 20);
        drop(source);
        assert!(weak.upgrade().is_none());
        let last = tail.mip_tail(255);
        assert_eq!(
            (last.width, last.level_count(), last.source_base_mip()),
            (1, 1, 2)
        );
        assert_eq!(last.mip_tail(0), last);
    }
    #[test]
    fn bc_tail_keeps_a_block_aligned_allocation_root() {
        let levels = [8_u32, 4, 2, 1].map(|size| CompressedTextureLevel {
            width: size,
            height: size,
            faces: Arc::from([Arc::<[u8]>::from(vec![
                0;
                (size.div_ceil(4).pow(2) * 8)
                    as usize
            ])]),
        });
        let source = TextureAsset::bc_2d(
            8,
            8,
            BlockCompressedTextureFormat::Bc1Rgba,
            Arc::from(levels),
            TextureWrapMode::Repeat,
        )
        .unwrap();
        let tail = source.mip_tail(3);
        assert_eq!(
            (tail.width, tail.source_base_mip(), tail.level_count()),
            (4, 1, 3)
        );
        assert_eq!(tail.payload_byte_len(), 24);
        assert_eq!(tail.mip_tail(3), tail);
    }

    #[test]
    fn cube_tail_preserves_every_face_and_its_fingerprint() {
        let levels = [4_u32, 2, 1].map(|size| RgbaTextureLevel {
            width: size,
            height: size,
            faces: Arc::from(
                (0..6)
                    .map(|face| Arc::<[u8]>::from(vec![face; (size * size * 4) as usize]))
                    .collect::<Vec<_>>(),
            ),
        });
        let source =
            TextureAsset::rgba_cube(4, 4, Arc::from(levels), TextureWrapMode::Clamp).unwrap();
        let tail = source.mip_tail(1);
        assert_eq!(tail.shape, TextureShape::Cube);
        assert_eq!(tail.payload_byte_len(), 120);
        assert_eq!(
            &*tail.content_fingerprints,
            &source.content_fingerprints[6..]
        );
        assert_eq!(tail.wrap_mode, TextureWrapMode::Clamp);
    }
}
