use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitmapTextMetrics {
    pub width: f32,
    pub line_height: f32,
    pub line_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitmapFontGlyph {
    pub unicode: u16,
    pub atlas_x: u16,
    pub atlas_y: u16,
    pub width: u8,
    pub height: u8,
    pub horizontal_offset_before: i8,
    pub horizontal_offset_after: i8,
    pub vertical_offset_before: i8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapFontAsset {
    pub font_id: u32,
    pub max_char_height: u32,
    pub max_char_width: u32,
    pub baseline_offset: i32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub atlas_rgba: Arc<[u8]>,
    pub min_unicode_char: u16,
    pub max_unicode_char: u16,
    pub character_map: Vec<u16>,
    pub glyphs: Vec<BitmapFontGlyph>,
}

impl BitmapFontAsset {
    pub fn glyph(&self, chr: u16) -> Option<&BitmapFontGlyph> {
        let mut candidate = chr;
        loop {
            if !self.character_map.is_empty()
                && candidate >= self.min_unicode_char
                && candidate <= self.max_unicode_char
            {
                let index =
                    usize::from(self.character_map[usize::from(candidate - self.min_unicode_char)]);
                if index < self.glyphs.len() {
                    return self.glyphs.get(index);
                }
                return None;
            }

            if let Some(index) = self
                .glyphs
                .iter()
                .position(|glyph| glyph.unicode == candidate)
            {
                return self.glyphs.get(index);
            }

            if candidate == b'?' as u16 {
                return None;
            }
            candidate = b'?' as u16;
        }
    }

    pub fn glyph_advance(&self, chr: u16) -> u8 {
        let Some(glyph) = self.glyph(chr) else {
            return 0;
        };

        glyph
            .width
            .wrapping_add(glyph.horizontal_offset_before as u8)
            .wrapping_add(glyph.horizontal_offset_after as u8)
    }

    pub fn line_height(&self) -> f32 {
        let glyph_bottom = self
            .glyphs
            .iter()
            .map(|glyph| glyph.vertical_offset_before as f32 + glyph.height as f32)
            .fold(0.0_f32, f32::max);
        glyph_bottom
            .max(self.baseline_offset.max(1) as f32)
            .max(self.max_char_height.max(1) as f32)
    }

    pub fn fallback_advance(&self) -> f32 {
        self.max_char_width.max(1) as f32
    }

    pub fn glyph_advance_or_fallback(&self, chr: u16) -> f32 {
        let advance = self.glyph_advance(chr);
        if advance == 0 {
            self.fallback_advance()
        } else {
            advance as f32
        }
    }

    pub fn measure_text(&self, text: &str) -> BitmapTextMetrics {
        let mut width = 0.0_f32;
        let mut widest = 0.0_f32;
        let mut line_count = 1_u32;
        for ch in text.chars() {
            if ch == '\n' {
                widest = widest.max(width);
                width = 0.0;
                line_count += 1;
                continue;
            }
            let advance = u16::try_from(ch as u32)
                .ok()
                .map(|code| self.glyph_advance_or_fallback(code))
                .unwrap_or_else(|| self.fallback_advance());
            width += advance;
        }

        BitmapTextMetrics {
            width: widest.max(width),
            line_height: self.line_height(),
            line_count,
        }
    }

    pub fn fitted_text_scale(
        &self,
        metrics: BitmapTextMetrics,
        target_width: f32,
        target_height: f32,
    ) -> f32 {
        let total_line_height = metrics.line_height.max(1.0) * metrics.line_count.max(1) as f32;
        let height_scale = target_height.max(1.0) / total_line_height;
        let width_scale = if metrics.width > 0.0 {
            target_width.max(1.0) / metrics.width
        } else {
            height_scale
        };

        height_scale.min(width_scale).max(0.01)
    }
}
