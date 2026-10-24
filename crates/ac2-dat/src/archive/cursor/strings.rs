use encoding_rs::WINDOWS_1252;

use super::ArchiveCursor;
use crate::{ArchiveError, ArchiveResult};

impl<'a> ArchiveCursor<'a> {
    pub fn read_utf16_string(&mut self) -> ArchiveResult<String> {
        let len = self.read_len()?;
        self.read_utf16_string_units(len)
    }

    pub fn read_pstring(&mut self) -> ArchiveResult<String> {
        let len = self.read_len()?;
        self.read_utf8_string(len)
    }

    pub fn read_retail_utf16_string(&mut self) -> ArchiveResult<String> {
        let len = self.read_retail_len()?;
        self.read_utf16_string_units(len)
    }

    pub fn read_retail_pstring(&mut self) -> ArchiveResult<String> {
        let len = self.read_retail_len()?;
        self.read_utf8_string(len)
    }

    pub fn read_packed_string_len(&mut self) -> ArchiveResult<u32> {
        let short = self.read_u16()?;
        if short == u16::MAX {
            self.read_u32()
        } else {
            Ok(u32::from(short))
        }
    }

    pub fn read_packed_string(&mut self) -> ArchiveResult<String> {
        self.read_packed_string_with_alignment(Some(4))
    }

    pub fn read_packed_string_with_alignment(
        &mut self,
        align: Option<usize>,
    ) -> ArchiveResult<String> {
        let len = self.read_packed_string_len()?;
        let len = usize::try_from(len).map_err(|_| ArchiveError::ValueOutOfRange {
            context: "packed string length",
            value: u64::from(len),
            max: usize::MAX as u64,
        })?;
        let bytes = self.read_bytes(len)?;
        if let Some(align) = align {
            self.align_to(align)?;
        }

        if len == 1 && bytes[0] == 0 {
            return Ok(String::new());
        }

        String::from_utf8(bytes.to_vec()).map_err(|_| ArchiveError::InvalidUtf8String)
    }

    pub fn read_windows_1252_packed_string(&mut self) -> ArchiveResult<String> {
        self.read_windows_1252_packed_string_with_alignment(Some(4))
    }

    pub fn read_windows_1252_packed_string_with_alignment(
        &mut self,
        align: Option<usize>,
    ) -> ArchiveResult<String> {
        let len = self.read_packed_string_len()?;
        let len = usize::try_from(len).map_err(|_| ArchiveError::ValueOutOfRange {
            context: "packed string length",
            value: u64::from(len),
            max: usize::MAX as u64,
        })?;
        let bytes = self.read_bytes(len)?;
        if let Some(align) = align {
            self.align_to(align)?;
        }
        let (decoded, _, _) = WINDOWS_1252.decode(bytes);
        Ok(decoded.into_owned())
    }

    fn read_utf16_string_units(&mut self, len: usize) -> ArchiveResult<String> {
        let byte_len = len.checked_mul(2).ok_or(ArchiveError::ValueOutOfRange {
            context: "UTF-16 string byte length",
            value: len as u64,
            max: (usize::MAX / 2) as u64,
        })?;
        let bytes = self.read_bytes(byte_len)?;
        let mut units = Vec::with_capacity(len);
        for chunk in bytes.chunks_exact(2) {
            units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
        }
        String::from_utf16(&units).map_err(|_| ArchiveError::InvalidUtf16String)
    }

    fn read_utf8_string(&mut self, len: usize) -> ArchiveResult<String> {
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ArchiveError::InvalidUtf8String)
    }
}
