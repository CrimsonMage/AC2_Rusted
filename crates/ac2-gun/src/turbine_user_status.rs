//! `TurbineUserStatus` (`ac2/shared/src/gun/helper/turbineuserstatus.cpp.c`): a friend's name,
//! location and presence mode, the element of the `PList<TurbineUserStatus>` the client hands
//! the scripts. Layout: `+0 vptr`, `+4 m_Name`, `+8 m_Location`, `+0xC m_Mode`.

use ac2_core::pstring::{munge, unmunge};

/// `STREAMTYPE` (PDB): what a `StreamPack` call does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
    GetPackSize = 0,
    Pack = 1,
    UnPack = 2,
}

/// `TurbineUserStatus`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TurbineUserStatus {
    /// `m_Name` (+4), the null string (empty) when unset.
    pub name: String,
    /// `m_Location` (+8).
    pub location: String,
    /// `m_Mode` (+0xC).
    pub mode: u32,
}

impl TurbineUserStatus {
    /// `TurbineUserStatus::TurbineUserStatus()` (admin 0x006047D0, Legions 0x005C56B0): both
    /// strings the null string, then `Clear()`. (The copy constructor, admin 0x00604830, Legions
    /// 0x005C5710, is the default fields plus `operator=`: `Clone`.)
    pub fn new() -> Self {
        let mut status = Self {
            name: String::new(),
            location: String::new(),
            mode: 0,
        };
        status.clear();
        status
    }

    /// `TurbineUserStatus::Clear` (admin 0x006045A0, Legions 0x005C5480): name and location the
    /// null string, mode 0.
    pub fn clear(&mut self) {
        self.name.clear();
        self.location.clear();
        self.mode = 0;
    }

    /// `TurbineUserStatus::InqName` (admin 0x006046C0, Legions 0x005C55A0): `None` (returns 0)
    /// when `m_Name.m_len == 1`, else a copy (returns 1).
    pub fn inq_name(&self) -> Option<String> {
        if self.name.is_empty() {
            None
        } else {
            Some(self.name.clone())
        }
    }

    /// `TurbineUserStatus::InqLocation` (admin 0x00604770, Legions 0x005C5650): the same for the
    /// location.
    pub fn inq_location(&self) -> Option<String> {
        if self.location.is_empty() {
            None
        } else {
            Some(self.location.clone())
        }
    }

    /// `TurbineUserStatus::SetLocation` (admin 0x00604720, Legions 0x005C5600): assigns the
    /// location string.
    pub fn set_location(&mut self, location: &str) {
        self.location = location.to_owned();
    }

    /// `TurbineUserStatus::InqMode` (admin 0x00604530, Legions 0x005C5410): `*out = m_Mode;
    /// return 1`.
    pub fn inq_mode(&self) -> u32 {
        self.mode
    }

    /// `TurbineUserStatus::StreamPack` (admin 0x00604880, Legions 0x005C5760):
    /// `int StreamPack(STREAMTYPE op, void** addr, uint* size)` runs `STREAMPACK(op, ...)` on
    /// `m_Name`, `m_Location` and `m_Mode`, in that order, ignores what each returns, and always
    /// returns 1 (here `true`). `buf` and `pos` are `*addr` (the buffer is taken to start
    /// 4-aligned, as retail aligns by the absolute pointer); `size` is `*size`, the bytes left,
    /// kept in `u32` arithmetic as retail does (it wraps below zero).
    ///
    /// * `STREAM_GETPACKSIZE`: each member takes its packed size off `*size`.
    /// * `STREAM_PACK`: a string is `PStringBase<ushort>::Pack` (writes only when its packed size
    ///   is `<= *size`, but returns that size, which is taken off `*size` either way); the mode
    ///   is written when `*size > 3`, and 4 is taken off either way.
    /// * `STREAM_UNPACK`: a string is `PStringBase<ushort>::UnPack` (fails, leaving the string,
    ///   when the length or the characters do not fit, the cursor already past what it read);
    ///   `*size` falls by the bytes the cursor moved; the mode needs `*size >= 4`, else nothing
    ///   moves.
    pub fn stream_pack(
        &mut self,
        op: StreamType,
        buf: &mut [u8],
        pos: &mut usize,
        size: &mut u32,
    ) -> bool {
        streampack_pstring(op, &mut self.name, buf, pos, size);
        streampack_pstring(op, &mut self.location, buf, pos, size);
        streampack_ulong(op, &mut self.mode, buf, pos, size);
        true
    }
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// `PStringBase<unsigned_short>::GetPackSize` (admin 0x00409960, Legions 0x004096D0): 2 bytes of
/// length (+4 when 0xFFFF or more characters), 2 per character, rounded up to 4.
fn pstring_pack_size(s: &str) -> u32 {
    let n = utf16_len(s);
    let size = (if n > 0xfffe { 4 } else { 0 }) + 2 + n * 2;
    ((size + 3) & !3) as u32
}

/// Writes `bytes` at `pos` when the Rust buffer has room (retail would write past its end).
fn put(buf: &mut [u8], pos: usize, bytes: &[u8]) {
    if let Some(dst) = buf.get_mut(pos..pos + bytes.len()) {
        dst.copy_from_slice(bytes);
    }
}

/// `STREAMPACK(STREAMTYPE, PStringBase<unsigned_short>*, void**, uint*)` (admin 0x004098E0,
/// Legions 0x00409650). Returns what it returns: 1 for size and pack, `UnPack`'s result for
/// unpack.
fn streampack_pstring(
    op: StreamType,
    s: &mut String,
    buf: &mut [u8],
    pos: &mut usize,
    size: &mut u32,
) -> bool {
    match op {
        StreamType::GetPackSize => {
            *size = size.wrapping_sub(pstring_pack_size(s));
            true
        }
        StreamType::Pack => {
            let packed = pstring_pack(s, buf, pos, *size);
            *size = size.wrapping_sub(packed);
            true
        }
        StreamType::UnPack => {
            let before = *pos;
            let ok = pstring_unpack(s, buf, pos, *size);
            *size = size.wrapping_sub((*pos - before) as u32);
            ok
        }
    }
}

/// `PStringBase<unsigned_short>::Pack` (admin 0x00409990, Legions 0x00409700): returns the
/// packed size; only when it is `<= size` the length (a `u16`, 0xFFFF then a `u32` for 0xFFFF
/// characters or more), the characters nibble-munged then uint32-munged are written and the
/// cursor moves past them, aligned up to 4 (the padding bytes are not written).
fn pstring_pack(s: &str, buf: &mut [u8], pos: &mut usize, size: u32) -> u32 {
    let packed = pstring_pack_size(s);
    if packed <= size {
        let n = utf16_len(s);
        if n > 0xfffe {
            put(buf, *pos, &0xffffu16.to_le_bytes());
            put(buf, *pos + 2, &(n as u32).to_le_bytes());
            *pos += 6;
        } else {
            put(buf, *pos, &(n as u16).to_le_bytes());
            *pos += 2;
        }
        let mut chars: Vec<u8> = s.encode_utf16().flat_map(u16::to_le_bytes).collect();
        munge(&mut chars);
        put(buf, *pos, &chars);
        *pos += chars.len();
        *pos = (*pos + 3) & !3;
    }
    packed
}

/// `PStringBase<unsigned_short>::UnPack` (admin 0x00409BE0, Legions 0x00409950): `size < 2`
/// fails; reads the `u16` length (0xFFFF: a `u32` follows, `size < 6` fails); the characters
/// (`2 * count` bytes) must fit in what is left of `size`, else it fails with the cursor where
/// it stopped; a count of 0 clears the string, otherwise the characters are uint32-unmunged
/// then nibble-unmunged; the cursor ends aligned up to 4.
fn pstring_unpack(s: &mut String, buf: &[u8], pos: &mut usize, size: u32) -> bool {
    if size < 2 {
        return false;
    }
    let Some(head) = buf.get(*pos..*pos + 2) else {
        return false;
    };
    let len = u16::from_le_bytes([head[0], head[1]]);
    *pos += 2;
    let mut left = size - 2;
    let mut count = u32::from(len);
    if len == 0xffff {
        if left < 4 {
            return false;
        }
        let Some(w) = buf.get(*pos..*pos + 4) else {
            return false;
        };
        count = u32::from_le_bytes([w[0], w[1], w[2], w[3]]);
        *pos += 4;
        left = size - 6;
    }
    let bytes = count.wrapping_mul(2);
    if left < bytes {
        return false;
    }
    if count == 0 {
        s.clear();
    } else {
        let Some(raw) = buf.get(*pos..*pos + bytes as usize) else {
            return false;
        };
        let mut chars = raw.to_vec();
        *pos += bytes as usize;
        unmunge(&mut chars);
        let units: Vec<u16> = chars
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        *s = String::from_utf16_lossy(&units);
    }
    *pos = (*pos + 3) & !3;
    true
}

/// `STREAMPACK(STREAMTYPE, ulong*, void**, uint*)` (admin 0x004AFAF0, Legions 0x00409E80).
fn streampack_ulong(
    op: StreamType,
    value: &mut u32,
    buf: &mut [u8],
    pos: &mut usize,
    size: &mut u32,
) -> bool {
    match op {
        StreamType::GetPackSize => {
            *size = size.wrapping_sub(4);
            true
        }
        StreamType::Pack => {
            if *size > 3 {
                put(buf, *pos, &value.to_le_bytes());
                *pos += 4;
            }
            *size = size.wrapping_sub(4);
            true
        }
        StreamType::UnPack => {
            if *size < 4 {
                return false;
            }
            let Some(w) = buf.get(*pos..*pos + 4) else {
                return false;
            };
            *value = u32::from_le_bytes([w[0], w[1], w[2], w[3]]);
            *pos += 4;
            *size -= 4;
            true
        }
    }
}
