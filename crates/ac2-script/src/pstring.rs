//! The scramble every `PStringBase` read from the DATs goes through
//! (`PStringBase<char>::UnPack` 0x00406CB0, `PStringBase<unsigned short>::UnPack` 0x00409BE0).

/// `ps_encryption_keys`, built on the stack by `PStringBase<char>::crypto_uint32_munge`
/// (0x00401FC0) and `PStringBase<unsigned short>::crypto_uint32_munge` (0x00409AB0).
const KEYS: [u32; 16] = [
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

/// Descrambles a string's character bytes (not the terminator) in place:
/// `crypto_uint32_munge` XORs every whole 32-bit word with `KEYS[i & 15]`, then
/// `crypto_nibble_munge` swaps the nibbles of every byte. Both munges are their own inverse
/// individually, so [`munge`] (the reverse order) scrambles.
pub fn unmunge(chars: &mut [u8]) {
    uint32_munge(chars);
    nibble_munge(chars);
}

/// Scrambles character bytes the way the DAT tools did (the inverse of [`unmunge`]).
pub fn munge(chars: &mut [u8]) {
    nibble_munge(chars);
    uint32_munge(chars);
}

fn uint32_munge(chars: &mut [u8]) {
    for (i, word) in chars.chunks_exact_mut(4).enumerate() {
        let value = u32::from_le_bytes([word[0], word[1], word[2], word[3]]) ^ KEYS[i & 15];
        word.copy_from_slice(&value.to_le_bytes());
    }
}

fn nibble_munge(chars: &mut [u8]) {
    for byte in chars {
        *byte = byte.rotate_left(4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn munge_round_trips_including_a_partial_last_word() {
        let text = b"Special characters!";
        let mut bytes = text.to_vec();
        munge(&mut bytes);
        assert_ne!(&bytes[..], &text[..]);
        unmunge(&mut bytes);
        assert_eq!(&bytes[..], &text[..]);
    }

    #[test]
    fn the_first_word_uses_the_first_key_and_bytes_are_nibble_swapped() {
        // One whole word: XOR with KEYS[0], then nibble swap.
        let mut bytes = [0u8; 4];
        unmunge(&mut bytes);
        let xored = KEYS[0].to_le_bytes();
        assert_eq!(bytes, xored.map(|b| b.rotate_left(4)));
        // A trailing partial word is only nibble-swapped.
        let mut tail = [0x12u8, 0x34];
        unmunge(&mut tail);
        assert_eq!(tail, [0x21, 0x43]);
    }
}
