use crate::{ArchiveError, ArchiveResult};

mod collections;
mod primitives;
mod strings;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_collections;

#[derive(Debug, Clone, Copy)]
pub struct ArchiveCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ArchiveCursor<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub const fn position(&self) -> usize {
        self.position
    }

    /// Returns the bytes already consumed from this borrowed archive input.
    pub fn serialized_bytes(&self) -> &'a [u8] {
        &self.bytes[..self.position]
    }

    /// Returns the unread borrowed tail without advancing the cursor.
    pub fn remaining_bytes(&self) -> &'a [u8] {
        &self.bytes[self.position..]
    }

    pub const fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.position)
    }

    pub fn set_position(&mut self, position: usize) -> ArchiveResult<()> {
        if position > self.bytes.len() {
            return Err(ArchiveError::InvalidSeek {
                position,
                len: self.bytes.len(),
            });
        }
        self.position = position;
        Ok(())
    }

    pub fn align_to(&mut self, align: usize) -> ArchiveResult<()> {
        let next = aligned_position(self.position, align)?;
        if next > self.bytes.len() {
            return Err(ArchiveError::UnexpectedEof {
                position: self.position,
                requested: next - self.position,
                len: self.bytes.len(),
            });
        }
        self.position = next;
        Ok(())
    }

    pub fn read_bytes(&mut self, len: usize) -> ArchiveResult<&'a [u8]> {
        let start = self.position;
        let end = start.checked_add(len).ok_or(ArchiveError::UnexpectedEof {
            position: start,
            requested: len,
            len: self.bytes.len(),
        })?;
        if end > self.bytes.len() {
            return Err(ArchiveError::UnexpectedEof {
                position: start,
                requested: len,
                len: self.bytes.len(),
            });
        }
        self.position = end;
        Ok(&self.bytes[start..end])
    }

    pub fn peek_bytes_at(&self, position: usize, len: usize) -> ArchiveResult<&'a [u8]> {
        let end = position
            .checked_add(len)
            .ok_or(ArchiveError::UnexpectedEof {
                position,
                requested: len,
                len: self.bytes.len(),
            })?;
        if end > self.bytes.len() {
            return Err(ArchiveError::UnexpectedEof {
                position,
                requested: len,
                len: self.bytes.len(),
            });
        }
        Ok(&self.bytes[position..end])
    }
}

pub(crate) fn aligned_position(position: usize, align: usize) -> ArchiveResult<usize> {
    if align == 0 || !align.is_power_of_two() {
        return Err(ArchiveError::InvalidAlignment { align });
    }
    let mask = align - 1;
    position
        .checked_add(mask)
        .map(|value| value & !mask)
        .ok_or(ArchiveError::AlignmentOverflow { position, align })
}
