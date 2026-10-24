use crate::{pack_string, unpack_string, unpack_wide_string};

use crate::protocol::ProtocolError;

/// Longest string accepted from the server, in bytes.
pub(crate) const MAX_STRING: usize = 4096;

/// Little-endian reader whose alignment is measured from the start of the event.
pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub(crate) fn rest(&mut self) -> &'a [u8] {
        let rest = &self.bytes[self.pos..];
        self.pos = self.bytes.len();
        rest
    }

    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8], ProtocolError> {
        let end = self.pos.checked_add(n).ok_or(ProtocolError::Truncated)?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(ProtocolError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    pub(crate) fn u16(&mut self) -> Result<u16, ProtocolError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, ProtocolError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, ProtocolError> {
        Ok(u64::from(self.u32()?) | (u64::from(self.u32()?) << 32))
    }

    pub(crate) fn f32(&mut self) -> Result<f32, ProtocolError> {
        Ok(f32::from_bits(self.u32()?))
    }

    pub(crate) fn f64(&mut self) -> Result<f64, ProtocolError> {
        Ok(f64::from_bits(self.u64()?))
    }

    pub(crate) fn vec3(&mut self) -> Result<[f32; 3], ProtocolError> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    /// `STREAMPACK_ALIGN`.
    pub(crate) fn align(&mut self) -> Result<(), ProtocolError> {
        self.take((4 - self.pos % 4) % 4).map(|_| ())
    }

    pub(crate) fn string(&mut self) -> Result<Vec<u8>, ProtocolError> {
        Ok(unpack_string(self.bytes, &mut self.pos, MAX_STRING)?)
    }

    pub(crate) fn wide_string(&mut self) -> Result<String, ProtocolError> {
        Ok(unpack_wide_string(self.bytes, &mut self.pos, MAX_STRING)?)
    }

    pub(crate) fn finish(&self) -> Result<(), ProtocolError> {
        if self.pos == self.bytes.len() {
            Ok(())
        } else {
            Err(ProtocolError::TrailingBytes)
        }
    }
}

/// Little-endian event writer.
pub(crate) struct Writer {
    pub(crate) bytes: Vec<u8>,
}

impl Writer {
    pub(crate) fn event(opcode: u32) -> Self {
        Self {
            bytes: opcode.to_le_bytes().to_vec(),
        }
    }
    pub(crate) fn u16(&mut self, value: u16) -> &mut Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }
    pub(crate) fn u32(&mut self, value: u32) -> &mut Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }
    pub(crate) fn u64(&mut self, value: u64) -> &mut Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }
    pub(crate) fn f32(&mut self, value: f32) -> &mut Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }
    pub(crate) fn f64(&mut self, value: f64) -> &mut Self {
        self.bytes.extend(value.to_le_bytes());
        self
    }
    pub(crate) fn string(&mut self, value: &[u8]) -> &mut Self {
        pack_string(&mut self.bytes, 0, value);
        self
    }
    pub(crate) fn align(&mut self) -> &mut Self {
        while self.bytes.len() % 4 != 0 {
            self.bytes.push(0);
        }
        self
    }
}
