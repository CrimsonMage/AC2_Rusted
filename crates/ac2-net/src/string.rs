//! `PStringBase<char>::Pack` and `PStringBase<unsigned short>::Pack`: a u16 unit count (`0xFFFF`
//! then a u32 for long strings), the character bytes with each byte's nibbles swapped and every
//! complete 32-bit word XORed with a repeating 16-word table, then zero padding to a 4-byte
//! boundary. Alignment is measured from the start of the containing message (`base`).

use crate::NetError;

const MUNGE: [u32; 16] = [
    0xE584_9238,
    0xEB2D_767D,
    0xFC0F_DD03,
    0xCB45_A6A6,
    0x7F21_E6BB,
    0xF5A6_CDC8,
    0x22D9_AD12,
    0xB2A3_D4DE,
    0x3B83_4DB7,
    0xC406_C91C,
    0x8392_305E,
    0x0A9F_0099,
    0x0943_B90C,
    0xA66D_953C,
    0x0D92_9E1F,
    0x0E2E_9AD4,
];

fn xor_words(bytes: &mut [u8]) {
    for (index, word) in bytes.chunks_exact_mut(4).enumerate() {
        for (byte, key) in word.iter_mut().zip(MUNGE[index & 15].to_le_bytes()) {
            *byte ^= key;
        }
    }
}

fn pack_units(out: &mut Vec<u8>, base: usize, units: usize, data: &[u8]) {
    if units >= 0xFFFF {
        out.extend(0xFFFF_u16.to_le_bytes());
        out.extend((units as u32).to_le_bytes());
    } else {
        out.extend((units as u16).to_le_bytes());
    }
    let mut encoded: Vec<u8> = data.iter().map(|b| b.rotate_left(4)).collect();
    xor_words(&mut encoded);
    out.extend(encoded);
    while (out.len() - base) % 4 != 0 {
        out.push(0);
    }
}

/// Appends a packed narrow string; `base` is the offset of the message start in `out`.
pub fn pack_string(out: &mut Vec<u8>, base: usize, value: &[u8]) {
    pack_units(out, base, value.len(), value);
}

/// Appends a packed UTF-16 string; `base` is the offset of the message start in `out`.
pub fn pack_wide_string(out: &mut Vec<u8>, base: usize, value: &str) {
    let units: Vec<u16> = value.encode_utf16().collect();
    let data: Vec<u8> = units.iter().flat_map(|unit| unit.to_le_bytes()).collect();
    pack_units(out, base, units.len(), &data);
}

fn unpack_units(
    bytes: &[u8],
    pos: &mut usize,
    width: usize,
    limit: usize,
) -> Result<Vec<u8>, NetError> {
    let take = |pos: &mut usize, n: usize| -> Result<&[u8], NetError> {
        let end = pos.checked_add(n).ok_or(NetError::Length)?;
        let slice = bytes.get(*pos..end).ok_or(NetError::Truncated)?;
        *pos = end;
        Ok(slice)
    };
    let short = take(pos, 2)?;
    let mut units = usize::from(u16::from_le_bytes([short[0], short[1]]));
    if units == 0xFFFF {
        let long = take(pos, 4)?;
        units = u32::from_le_bytes([long[0], long[1], long[2], long[3]]) as usize;
    }
    let len = units.checked_mul(width).ok_or(NetError::Length)?;
    if len > limit {
        return Err(NetError::Capacity);
    }
    let mut data = take(pos, len)?.to_vec();
    xor_words(&mut data);
    data.iter_mut().for_each(|b| *b = b.rotate_left(4));
    take(pos, (4 - *pos % 4) % 4)?;
    Ok(data)
}

/// Reads a packed narrow string at `*pos`, measured from the message start, advancing past its
/// padding. `limit` bounds the byte length before anything is allocated.
pub fn unpack_string(bytes: &[u8], pos: &mut usize, limit: usize) -> Result<Vec<u8>, NetError> {
    unpack_units(bytes, pos, 1, limit)
}

/// Reads a packed UTF-16 string at `*pos`; `limit` bounds the byte length.
pub fn unpack_wide_string(bytes: &[u8], pos: &mut usize, limit: usize) -> Result<String, NetError> {
    let data = unpack_units(bytes, pos, 2, limit)?;
    let units: Vec<u16> = data
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&units).map_err(|_| NetError::Length)
}

#[cfg(test)]
mod tests {
    use super::{pack_string, pack_wide_string, unpack_string, unpack_wide_string};

    #[test]
    fn version_string_matches_the_client_encoding() {
        // "7148": nibble swapped 73 13 43 83, XORed with the first table word 38 92 84 E5.
        let mut out = Vec::new();
        pack_string(&mut out, 0, b"7148");
        assert_eq!(out, [4, 0, 0x4B, 0x81, 0xC7, 0x66, 0, 0]);
        let mut pos = 0;
        assert_eq!(unpack_string(&out, &mut pos, 64).unwrap(), b"7148");
        assert_eq!(pos, out.len());
    }

    #[test]
    fn a_partial_word_is_only_nibble_swapped_and_padding_follows_the_message_start() {
        let mut out = vec![0xAA; 4];
        pack_string(&mut out, 0, b"abc");
        assert_eq!(&out[4..], &[3, 0, 0x16, 0x26, 0x36, 0, 0, 0]);
        let mut pos = 4;
        assert_eq!(unpack_string(&out, &mut pos, 64).unwrap(), b"abc");
        assert_eq!(pos, 12);
    }

    #[test]
    fn wide_strings_round_trip_and_limits_apply_before_allocation() {
        let mut out = Vec::new();
        pack_wide_string(&mut out, 0, "Explorer");
        let mut pos = 0;
        assert_eq!(unpack_wide_string(&out, &mut pos, 64).unwrap(), "Explorer");
        let mut pos = 0;
        assert!(unpack_wide_string(&out, &mut pos, 4).is_err());
        let mut pos = 0;
        assert!(unpack_string(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F], &mut pos, 4096).is_err());
    }
}
