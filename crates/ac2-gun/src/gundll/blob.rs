//! GunDll's text codecs for byte blobs, from `decompiled-pdb/gundll/spec/0a_send_session.md` section 7
//! (verified in `verify_0a_send_session.md`): the 6-bit "blob" alphabet of `<Elem Val="..."/>`
//! (`FUN_10002da0` encode, `FUN_10002cf0` decode, `FUN_10002c10` character value), the standard Base64
//! decoder `SetTicket` uses (`FUN_10002e90`, bugs and all) and its tag extractor (`FUN_10002fd0`).

use crate::zt::*;
use crate::Hresult;

/// The 64-character alphabet at GunDll 0x10068080.
pub const ALPHABET: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!.";

/// `FUN_10002da0(out, outSize, in, inLen)` (0x10002da0): per group of up to 3 input bytes
/// `v = b0 | b1 << 8 | b2 << 16` it emits `L + 1` characters of `v & 63`, `(v >> 6) & 63`, ...
/// (3 bytes -> 4 characters, 2 -> 3, 1 -> 2), no padding, then a NUL. `out_size` is the buffer size
/// the caller gave (`FUN_100376d0` sizes it `2 * len + 4`, rounded to 4); a group that would leave no
/// room for the NUL, `offset + L + 1 >= out_size`, is `ZT_E_BUFFER_TOO_SMALL`. [confirmed]
pub fn encode(input: &[u8], out_size: usize) -> Result<String, Hresult> {
    let mut out = String::new();
    for group in input.chunks(3) {
        let l = group.len();
        if out.len() + l + 1 >= out_size {
            return Err(ZT_E_BUFFER_TOO_SMALL);
        }
        let mut v = 0u32;
        for (i, b) in group.iter().enumerate() {
            v |= (*b as u32) << (8 * i);
        }
        for i in 0..=l {
            out.push(ALPHABET[((v >> (6 * i)) & 0x3f) as usize] as char);
        }
    }
    Ok(out)
}

/// `FUN_10002da0` as `FUN_100376d0` calls it: the buffer is `2 * len + 4` bytes rounded up to a
/// multiple of 4. [confirmed: `0a_send_session.md` 7.2]
pub fn encode_for_attribute(input: &[u8]) -> Result<String, Hresult> {
    let size = (2 * input.len() + 4).div_ceil(4) * 4;
    encode(input, size)
}

/// `FUN_10002c10(char)` (0x10002c10): `a-z` -> 0-25, `A-Z` -> 26-51, `0-9` -> 52-61, `!` -> 62,
/// `.` -> 63; anything else `ZT_E_INVALID_CHARACTER`.
pub fn char_value(c: u16) -> Result<u32, Hresult> {
    match c {
        0x61..=0x7a => Ok((c - 0x61) as u32),
        0x41..=0x5a => Ok((c - 0x27) as u32),
        0x30..=0x39 => Ok((c + 4) as u32),
        0x21 => Ok(62),
        0x2e => Ok(63),
        _ => Err(ZT_E_INVALID_CHARACTER),
    }
}

/// `FUN_10002cf0(wstr, out, outCap, &outLen)` (0x10002cf0): wide characters up to the NUL; 6 bits
/// accumulate LSB first and a byte is emitted whenever 8 are available (`out_cap` full ->
/// `ZT_E_BUFFER_TOO_SMALL`); leftover bits that are not all zero -> `ZT_E_INVALID_CHARACTER`.
pub fn decode(text: &[u16], out_cap: usize) -> Result<Vec<u8>, Hresult> {
    let mut out = Vec::new();
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    for &c in text.iter().take_while(|&&c| c != 0) {
        let v = char_value(c)?;
        acc |= v << bits;
        bits += 6;
        if bits >= 8 {
            if out.len() >= out_cap {
                return Err(ZT_E_BUFFER_TOO_SMALL);
            }
            out.push((acc & 0xff) as u8);
            acc >>= 8;
            bits -= 8;
        }
    }
    if bits != 0 && acc != 0 {
        return Err(ZT_E_INVALID_CHARACTER);
    }
    Ok(out)
}

/// The table at GunDll 0x100597e0 (256 bytes): `A-Z` 0-25, `a-z` 26-51, `0-9` 52-61, `+` 62, `/` 63,
/// everything else 0x82 (a signed char, so -126 once sign-extended).
fn b64_table(c: u8) -> i32 {
    (match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => 0x82,
    }) as i8 as i32
}

/// `FUN_10002e90(in, out, &len)` (0x10002e90, `Base64_Decode`): `in_len` is `*len` on entry (the
/// extractor gives `strlen + 1`, so the NUL is processed as one more, invalid, character). Output
/// bytes are written per fourth character; an invalid character (0x82) is sign-extended and OR-ed in,
/// not rejected. The returned length is `(3 * quads + (4 - r)) - 1` for a tail of `r` characters,
/// `3 * quads - 1` for none, which is right only for `r == 2` (`0a_send_session.md` 7.4,
/// adjudicated). Returns `(bytes_written_buffer, reported_len)`; `out` is sized for the 3-byte
/// groups it writes, as the caller's buffer is.
pub fn base64_decode(input: &[u8], in_len: u32) -> (Vec<u8>, u32) {
    let at = |i: usize| -> u8 { input.get(i).copied().unwrap_or(0) };
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut acc: i32 = 0;
    let mut written: u32 = 0; // uVar6
    let mut n: u32 = 1; // uVar7
    let end = in_len as usize;
    let mut c = at(0);
    while c != b'=' && written <= in_len && pos < end {
        pos += 1;
        acc = (acc | b64_table(c)).wrapping_shl(6);
        if n.is_multiple_of(4) {
            let v = acc >> 6;
            out.push((v >> 16) as u8);
            out.push((v >> 8) as u8);
            out.push(v as u8);
            acc = 0;
            written += 3;
        }
        c = at(pos);
        n += 1;
    }
    let r = (n - 1) % 4;
    if r != 0 {
        let mut k = 4 - r;
        written += k;
        while k > 0 {
            acc = acc.wrapping_shl(6);
            k -= 1;
        }
        let v = acc >> 6;
        out.push((v >> 16) as u8);
        out.push((v >> 8) as u8);
        out.push(v as u8);
    }
    (out, written.wrapping_sub(1))
}

/// `FUN_10002fd0(needle, haystack, out, &len)` (0x10002fd0): finds `needle` case-insensitively
/// (`__strnicmp`) in `haystack`, then copies the text after it up to the next `<` or NUL. Returns the
/// text and its `len` (text length + 1), or `0x8000FFFF` when the needle is absent. The haystack is
/// read as a NUL-terminated string.
pub fn extract_tag(needle: &str, haystack: &[u8]) -> Result<(Vec<u8>, u32), Hresult> {
    let hay: Vec<u8> = haystack.iter().copied().take_while(|&b| b != 0).collect();
    let nl = needle.len();
    let nb = needle.as_bytes();
    let mut p = 0usize;
    let matches = |p: usize| {
        hay.len() >= p + nl
            && hay[p..p + nl]
                .iter()
                .zip(nb)
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
    };
    // while (strnicmp != 0 && haystack[p + nl] != 0) p++;
    while !matches(p) && p + nl < hay.len() {
        p += 1;
    }
    if p + nl >= hay.len() {
        // haystack[p + nl] == 0: the needle was not found followed by any text
        return Err(Hresult(0x8000_FFFF));
    }
    let start = p + nl;
    let mut q = start;
    while q < hay.len() && hay[q] != b'<' {
        q += 1;
    }
    let text = hay[start..q].to_vec();
    let len = text.len() as u32 + 1;
    Ok((text, len))
}
