//! RIFF reading for the DirectMusic and DLS files the `DBMusicObj` records hold: chunks are a
//! four-character id, a little-endian size and the data (padded to even); `RIFF` and `LIST`
//! chunks start with a form type and hold more chunks.

/// A file the parser could not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicFormatError(pub &'static str);

impl std::fmt::Display for MusicFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for MusicFormatError {}

pub type MusicResult<T> = Result<T, MusicFormatError>;

/// One chunk: its id, and for `RIFF` / `LIST` its form type; `data` is the body after the form.
#[derive(Debug, Clone, Copy)]
pub struct Chunk<'a> {
    pub id: [u8; 4],
    pub form: Option<[u8; 4]>,
    pub data: &'a [u8],
}

impl<'a> Chunk<'a> {
    /// Whether it is a `RIFF` or `LIST` of form `form`.
    pub fn is_list(&self, form: &[u8; 4]) -> bool {
        self.form.as_ref() == Some(form)
    }

    /// Its sub-chunks (for a `RIFF` / `LIST`).
    pub fn children(&self) -> Chunks<'a> {
        Chunks { data: self.data }
    }

    /// The first sub-chunk with id `id`.
    pub fn child(&self, id: &[u8; 4]) -> Option<Chunk<'a>> {
        self.children().find(|c| &c.id == id && c.form.is_none())
    }

    /// The first sub-list of form `form`.
    pub fn list(&self, form: &[u8; 4]) -> Option<Chunk<'a>> {
        self.children().find(|c| c.is_list(form))
    }

    /// A little-endian reader over the data.
    pub fn reader(&self) -> Reader<'a> {
        Reader {
            data: self.data,
            at: 0,
        }
    }

    /// The data as a NUL-terminated UTF-16 string.
    pub fn text(&self) -> String {
        let units: Vec<u16> = self
            .data
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&u| u != 0)
            .collect();
        String::from_utf16_lossy(&units)
    }
}

/// The chunks of a block of data, in order.
#[derive(Debug, Clone)]
pub struct Chunks<'a> {
    data: &'a [u8],
}

impl<'a> Chunks<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Iterator for Chunks<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() < 8 {
            return None;
        }
        let id: [u8; 4] = self.data[0..4].try_into().ok()?;
        let size = u32::from_le_bytes(self.data[4..8].try_into().ok()?) as usize;
        let end = 8usize.checked_add(size)?;
        if end > self.data.len() {
            self.data = &[];
            return None;
        }
        let body = &self.data[8..end];
        let padded = (end + (size & 1)).min(self.data.len());
        self.data = &self.data[padded..];
        if (&id == b"RIFF" || &id == b"LIST") && body.len() >= 4 {
            Some(Chunk {
                id,
                form: Some(body[0..4].try_into().ok()?),
                data: &body[4..],
            })
        } else {
            Some(Chunk {
                id,
                form: None,
                data: body,
            })
        }
    }
}

/// The top `RIFF` chunk of a file.
pub fn riff(data: &[u8]) -> MusicResult<Chunk<'_>> {
    Chunks::new(data)
        .next()
        .filter(|c| &c.id == b"RIFF")
        .ok_or(MusicFormatError("not a RIFF file"))
}

/// A little-endian field reader; reads past the end give zero (DirectMusic structures grew over
/// versions, and a shorter one reads as the older layout with the new fields zero).
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.at)
    }

    pub fn position(&self) -> usize {
        self.at
    }

    pub fn seek(&mut self, at: usize) {
        self.at = at;
    }

    pub fn skip(&mut self, n: usize) {
        self.at = self.at.saturating_add(n);
    }

    fn bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut out = [0u8; N];
        if let Some(src) = self.data.get(self.at..self.at + N) {
            out.copy_from_slice(src);
        } else if let Some(src) = self.data.get(self.at..) {
            out[..src.len()].copy_from_slice(src);
        }
        self.at = self.at.saturating_add(N);
        out
    }

    pub fn u8(&mut self) -> u8 {
        self.bytes::<1>()[0]
    }

    pub fn i8(&mut self) -> i8 {
        self.u8() as i8
    }

    pub fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.bytes())
    }

    pub fn i16(&mut self) -> i16 {
        i16::from_le_bytes(self.bytes())
    }

    pub fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.bytes())
    }

    pub fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.bytes())
    }

    pub fn f64(&mut self) -> f64 {
        f64::from_le_bytes(self.bytes())
    }

    pub fn guid(&mut self) -> Guid {
        Guid(self.bytes())
    }

    /// `n` raw bytes (fewer at the end).
    pub fn take(&mut self, n: usize) -> &'a [u8] {
        let start = self.at.min(self.data.len());
        let end = self.at.saturating_add(n).min(self.data.len());
        self.at = self.at.saturating_add(n);
        &self.data[start..end]
    }

    /// A DirectMusic array chunk: a leading item size, then items of that size (each read by
    /// `item` from its own reader, so an item shorter or longer than the reader expects stays
    /// aligned).
    pub fn sized_items<T>(&mut self, mut item: impl FnMut(&mut Reader<'a>) -> T) -> Vec<T> {
        let size = self.u32() as usize;
        let mut out = Vec::new();
        if size == 0 {
            return out;
        }
        while self.remaining() >= size {
            let mut r = Reader::new(self.take(size));
            out.push(item(&mut r));
        }
        out
    }
}

/// A GUID as stored (little-endian fields).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Guid(pub [u8; 16]);

impl Guid {
    pub const NULL: Guid = Guid([0; 16]);

    /// From the registry form's first field (`{D2AC28xx-B39B-11D1-8704-00600893B1BD}`).
    pub const fn from_parts(a: u32, b: u16, c: u16, d: [u8; 8]) -> Self {
        let a = a.to_le_bytes();
        let b = b.to_le_bytes();
        let c = c.to_le_bytes();
        Guid([
            a[0], a[1], a[2], a[3], b[0], b[1], c[0], c[1], d[0], d[1], d[2], d[3], d[4], d[5],
            d[6], d[7],
        ])
    }
}
