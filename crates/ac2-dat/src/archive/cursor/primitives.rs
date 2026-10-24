use super::ArchiveCursor;
use crate::{ArchiveError, ArchiveResult};
use ac2_core::ColorArgbByteReader;

impl ColorArgbByteReader for ArchiveCursor<'_> {
    type Error = ArchiveError;

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        ArchiveCursor::read_u8(self)
    }
}

impl<'a> ArchiveCursor<'a> {
    pub fn read_u8(&mut self) -> ArchiveResult<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_bool_flag(&mut self) -> ArchiveResult<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(ArchiveError::InvalidBoolFlag { value }),
        }
    }

    pub fn read_bool32_flag(&mut self) -> ArchiveResult<bool> {
        match self.read_u32()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(ArchiveError::InvalidBool32Flag { value }),
        }
    }

    pub fn read_u16(&mut self) -> ArchiveResult<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_u8_count(&mut self) -> ArchiveResult<usize> {
        Ok(usize::from(self.read_u8()?))
    }

    pub fn read_u16_count(&mut self) -> ArchiveResult<usize> {
        Ok(usize::from(self.read_u16()?))
    }

    pub fn read_u32(&mut self) -> ArchiveResult<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_u32_count(&mut self) -> ArchiveResult<usize> {
        let count = self.read_u32()?;
        usize::try_from(count).map_err(|_| ArchiveError::ValueOutOfRange {
            context: "u32 item count",
            value: u64::from(count),
            max: usize::MAX as u64,
        })
    }

    pub fn read_i32(&mut self) -> ArchiveResult<i32> {
        Ok(self.read_u32()? as i32)
    }

    pub fn read_i32_non_negative_count(&mut self) -> ArchiveResult<usize> {
        let count = self.read_i32()?;
        usize::try_from(count).map_err(|_| ArchiveError::NegativeCount {
            context: "i32 item count",
            value: count,
        })
    }

    pub fn read_u64(&mut self) -> ArchiveResult<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_i64(&mut self) -> ArchiveResult<i64> {
        Ok(self.read_u64()? as i64)
    }

    pub fn read_f32(&mut self) -> ArchiveResult<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_f64(&mut self) -> ArchiveResult<f64> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    pub fn read_compressed_u32(&mut self) -> ArchiveResult<u32> {
        let first = self.read_u16()?;
        if first & 0x8000 == 0 {
            Ok(u32::from(first))
        } else {
            let second = self.read_u16()?;
            Ok((u32::from(first & 0x7FFF) << 16) | u32::from(second))
        }
    }

    pub fn read_len(&mut self) -> ArchiveResult<usize> {
        let value = self.read_compressed_u32()?;
        usize::try_from(value).map_err(|_| ArchiveError::ValueOutOfRange {
            context: "compressed archive length",
            value: u64::from(value),
            max: usize::MAX as u64,
        })
    }

    pub fn read_retail_compressed_u32(&mut self) -> ArchiveResult<u32> {
        let b0 = self.read_u8()?;
        if (b0 & 0x80) == 0 {
            return Ok(u32::from(b0));
        }

        let b1 = self.read_u8()?;
        if (b0 & 0x40) == 0 {
            return Ok((u32::from(b0 & 0x7F) << 8) | u32::from(b1));
        }

        let lower = self.read_u16()?;
        Ok((((u32::from(b0 & 0x3F) << 8) | u32::from(b1)) << 16) | u32::from(lower))
    }

    pub fn read_retail_compressed_u32_count(&mut self) -> ArchiveResult<usize> {
        let count = self.read_retail_compressed_u32()?;
        usize::try_from(count).map_err(|_| ArchiveError::ValueOutOfRange {
            context: "retail compressed u32 item count",
            value: u64::from(count),
            max: usize::MAX as u64,
        })
    }

    pub fn read_retail_len(&mut self) -> ArchiveResult<usize> {
        let value = self.read_retail_compressed_u32()?;
        usize::try_from(value).map_err(|_| ArchiveError::ValueOutOfRange {
            context: "retail compressed archive length",
            value: u64::from(value),
            max: usize::MAX as u64,
        })
    }
}
