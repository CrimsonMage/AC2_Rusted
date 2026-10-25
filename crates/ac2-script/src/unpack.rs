//! The retail `PackObj` unpack conventions the script data uses: little-endian 32-bit words,
//! `PackObj::ALIGN_PTR` 4-byte alignment (relative to the record start, which retail keeps
//! 4-aligned), `PSmartArray` (u32 count, then items) and `PHashTable` headers.

use std::fmt;

use ac2_dat::{ArchiveCursor, ArchiveError};

use crate::pstring;

/// A script record that does not unpack the way retail unpacks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpackError {
    pub context: &'static str,
    pub position: usize,
}

impl fmt::Display for UnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.context, self.position)
    }
}

impl std::error::Error for UnpackError {}

pub type UnpackResult<T> = Result<T, UnpackError>;

/// Collections longer than this are treated as corrupt (the largest real one, the vtable
/// section's offset arrays, holds 3.7 million words in total but far fewer per array).
const MAX_ITEMS: usize = 1 << 24;

/// An [`ArchiveCursor`] with the retail script unpack helpers.
pub struct Reader<'a> {
    cursor: ArchiveCursor<'a>,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            cursor: ArchiveCursor::new(bytes),
        }
    }

    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    pub fn remaining(&self) -> usize {
        self.cursor.remaining()
    }

    pub fn bad(&self, context: &'static str) -> UnpackError {
        UnpackError {
            context,
            position: self.cursor.position(),
        }
    }

    fn map<T>(&self, result: Result<T, ArchiveError>, context: &'static str) -> UnpackResult<T> {
        result.map_err(|_| self.bad(context))
    }

    pub fn u8(&mut self) -> UnpackResult<u8> {
        let r = self.cursor.read_u8();
        self.map(r, "unexpected end of data")
    }

    pub fn u16(&mut self) -> UnpackResult<u16> {
        let r = self.cursor.read_u16();
        self.map(r, "unexpected end of data")
    }

    pub fn u32(&mut self) -> UnpackResult<u32> {
        let r = self.cursor.read_u32();
        self.map(r, "unexpected end of data")
    }

    pub fn i32(&mut self) -> UnpackResult<i32> {
        Ok(self.u32()? as i32)
    }

    pub fn u64(&mut self) -> UnpackResult<u64> {
        let r = self.cursor.read_u64();
        self.map(r, "unexpected end of data")
    }

    pub fn f32(&mut self) -> UnpackResult<f32> {
        Ok(f32::from_bits(self.u32()?))
    }

    pub fn f64(&mut self) -> UnpackResult<f64> {
        Ok(f64::from_bits(self.u64()?))
    }

    pub fn bytes(&mut self, len: usize) -> UnpackResult<&'a [u8]> {
        let r = self.cursor.read_bytes(len);
        self.map(r, "unexpected end of data")
    }

    /// `PackObj::ALIGN_PTR`.
    pub fn align(&mut self) -> UnpackResult<()> {
        let r = self.cursor.align_to(4);
        self.map(r, "alignment past the end")
    }

    /// A u32 element count, checked against what is left (`item` bytes per element at least).
    pub fn count(&mut self, item: usize, context: &'static str) -> UnpackResult<usize> {
        let n = self.u32()? as usize;
        self.check_count(n, item, context)
    }

    fn check_count(&self, n: usize, item: usize, context: &'static str) -> UnpackResult<usize> {
        if n > MAX_ITEMS || n.saturating_mul(item) > self.remaining() {
            return Err(self.bad(context));
        }
        Ok(n)
    }

    /// `PHashTable::StreamPack`'s header: the element count in the low 24 bits, the bucket-size
    /// index (`g_bucketSizes`) in the high 8.
    pub fn hash_count(&mut self, item: usize, context: &'static str) -> UnpackResult<usize> {
        Ok(self.hash_header(item, context)?.0)
    }

    /// [`Self::hash_count`] plus the bucket-size index.
    pub fn hash_header(&mut self, item: usize, context: &'static str) -> UnpackResult<(usize, u8)> {
        let header = self.u32()?;
        let n = self.check_count((header & 0x00FF_FFFF) as usize, item, context)?;
        Ok((n, (header >> 24) as u8))
    }

    /// `PSmartArray<unsigned long>::StreamPack`.
    pub fn u32_array(&mut self) -> UnpackResult<Vec<u32>> {
        let n = self.count(4, "u32 array count")?;
        (0..n).map(|_| self.u32()).collect()
    }

    /// A `PStringBase` length: u16, or 0xFFFF then u32.
    fn string_len(&mut self) -> UnpackResult<usize> {
        let short = self.u16()?;
        if short == u16::MAX {
            Ok(self.u32()? as usize)
        } else {
            Ok(usize::from(short))
        }
    }

    /// `PStringBase<char>::UnPack`: length, the scrambled characters, alignment.
    pub fn pstring(&mut self) -> UnpackResult<Vec<u8>> {
        let len = self.string_len()?;
        let mut chars = self.bytes(len)?.to_vec();
        pstring::unmunge(&mut chars);
        self.align()?;
        Ok(chars)
    }

    /// `PStringBase<unsigned short>::UnPack`: length in UTF-16 units, the scrambled units,
    /// alignment.
    pub fn wpstring(&mut self) -> UnpackResult<Vec<u16>> {
        let len = self.string_len()?;
        let byte_len = len
            .checked_mul(2)
            .ok_or_else(|| self.bad("wide string length"))?;
        let mut raw = self.bytes(byte_len)?.to_vec();
        pstring::unmunge(&mut raw);
        self.align()?;
        Ok(raw
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect())
    }

    /// Fails unless every byte was read.
    pub fn finish(&self, context: &'static str) -> UnpackResult<()> {
        if self.remaining() != 0 {
            return Err(self.bad(context));
        }
        Ok(())
    }
}
