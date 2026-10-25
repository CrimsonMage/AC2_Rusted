//! `Font` (`DB_TYPE_FONT` 0x4000003E, DIDs 0x42000000..=0x42000FFE in the portal DAT;
//! `FontLocal` 0x42001000.. in the local DAT): a strip of glyphs side by side, `totalWidth`
//! pixels wide and `maxCharHeight` tall, 8-bit coverage when anti-aliased, else 1 bit per pixel.
//!
//! `Font::StreamPack`: the `DBObj` DID, `maxCharHeight`, `maxCharWidth`, `numCharacters`,
//! `avgCharWidth`, `totalWidth`, `m_nIsAntialiased` (u32 each), the `FontCharDesc`s (12 bytes:
//! width u32, unicode u16, padding, offset u32 into the strip), then the strip.
//! `Font::GenerateCharacterMap` indexes the glyphs by code point between the lowest and highest.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    CachedResource, ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore,
    ResourceTypeId, TypedResourceRef,
};

/// `FontCharDesc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ac2FontChar {
    pub width: u32,
    pub unicode: u16,
    /// The glyph's first column in the strip.
    pub offset: u32,
}

/// A `Font`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ac2Font {
    pub max_char_height: u32,
    pub max_char_width: u32,
    pub avg_char_width: u32,
    pub total_width: u32,
    pub antialiased: bool,
    pub chars: Vec<Ac2FontChar>,
    /// The strip, row by row: `total_width` bytes a row when anti-aliased, else
    /// `total_width / 8` bytes of bits, most significant first.
    pub data: Vec<u8>,
    min_unicode: u16,
    /// `characterMap`: code point - `min_unicode` -> glyph index, -1 where none.
    character_map: Vec<i32>,
}

impl Ac2Font {
    /// `Font::GetCharDesc`: the glyph for `chr`, else for `?`.
    pub fn char_desc(&self, chr: u16) -> Option<&Ac2FontChar> {
        self.lookup(chr).or_else(|| self.lookup(u16::from(b'?')))
    }

    fn lookup(&self, chr: u16) -> Option<&Ac2FontChar> {
        let index = usize::from(chr.checked_sub(self.min_unicode)?);
        let glyph = *self.character_map.get(index)?;
        usize::try_from(glyph).ok().and_then(|i| self.chars.get(i))
    }

    /// `Font::GetStringWidthInternal`: the widest line, lines split at `\n`.
    pub fn string_width(&self, text: &[u16]) -> u32 {
        let (mut widest, mut line) = (0, 0);
        for &unit in text {
            if unit == u16::from(b'\n') {
                widest = widest.max(line);
                line = 0;
            } else {
                line += self.char_desc(unit).map_or(0, |c| c.width);
            }
        }
        widest.max(line)
    }

    /// The coverage (0..=255) of strip pixel (`column`, `row`).
    pub fn coverage(&self, column: u32, row: u32) -> u8 {
        if self.antialiased {
            let at = row as usize * self.total_width as usize + column as usize;
            self.data.get(at).copied().unwrap_or(0)
        } else {
            let at = row as usize * (self.total_width as usize >> 3) + (column as usize >> 3);
            let bit = self
                .data
                .get(at)
                .map_or(0, |byte| byte >> (7 - (column & 7)) & 1);
            if bit != 0 {
                255
            } else {
                0
            }
        }
    }
}

fn bad(qdid: QualifiedDataId, context: &'static str) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name: Ac2Font::TYPE_NAME,
        context,
    }
}

impl ResourcePackedDecode for Ac2Font {
    const TYPE_NAME: &'static str = "AC2 Font";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(bad(qdid, "embedded DID did not match DAT identity"));
        }
        let max_char_height = cursor.read_u32()?;
        let max_char_width = cursor.read_u32()?;
        let count = cursor.read_u32()? as usize;
        let avg_char_width = cursor.read_u32()?;
        let total_width = cursor.read_u32()?;
        let antialiased = cursor.read_u32()? != 0;
        if count * 12 > cursor.remaining() {
            return Err(bad(qdid, "character count"));
        }
        let mut chars = Vec::with_capacity(count);
        for _ in 0..count {
            let width = cursor.read_u32()?;
            let unicode = cursor.read_u16()?;
            let _padding = cursor.read_u16()?;
            chars.push(Ac2FontChar {
                width,
                unicode,
                offset: cursor.read_u32()?,
            });
        }
        let mut size = total_width as usize * max_char_height as usize;
        if !antialiased {
            size >>= 3;
        }
        let data = cursor.read_bytes(size)?.to_vec();
        if cursor.remaining() != 0 {
            return Err(bad(qdid, "trailing bytes after Font"));
        }
        // `Font::GenerateCharacterMap`.
        let min_unicode = chars.iter().map(|c| c.unicode).min().unwrap_or(0);
        let max_unicode = chars.iter().map(|c| c.unicode).max().unwrap_or(0);
        let mut character_map = vec![-1; usize::from(max_unicode - min_unicode) + 1];
        for (index, c) in chars.iter().enumerate() {
            character_map[usize::from(c.unicode - min_unicode)] = index as i32;
        }
        Ok(Self {
            max_char_height,
            max_char_width,
            avg_char_width,
            total_width,
            antialiased,
            chars,
            data,
            min_unicode,
            character_map,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.chars.len() * 12
            + self.data.len()
            + self.character_map.len() * 4
    }
}

/// The font `did`: portal `Font` below 0x42001000, local `FontLocal` above.
pub fn load_ac2_font(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<CachedResource<Ac2Font>>> {
    let kind = if did.raw() < 0x4200_1000 {
        DatFileKind::Portal
    } else {
        DatFileKind::Local
    };
    store.load_cached_packed::<Ac2Font>(TypedResourceRef::new(kind, did, 0))
}
