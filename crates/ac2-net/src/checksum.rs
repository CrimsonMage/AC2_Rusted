/// `PortalChecksum::CalcChecksum32`: the byte count shifted into the high half, plus every
/// complete little-endian word, plus each trailing byte shifted to bits 24, 16 and 8 in turn.
pub fn checksum32(bytes: &[u8]) -> u32 {
    let mut sum = (bytes.len() as u32).wrapping_shl(16);
    let mut words = bytes.chunks_exact(4);
    for word in &mut words {
        sum = sum.wrapping_add(u32::from_le_bytes([word[0], word[1], word[2], word[3]]));
    }
    for (index, byte) in words.remainder().iter().enumerate() {
        sum = sum.wrapping_add(u32::from(*byte) << (24 - index * 8));
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::checksum32;

    #[test]
    fn counts_length_words_and_the_trailing_bytes_high_first() {
        assert_eq!(checksum32(&[]), 0);
        assert_eq!(checksum32(&[1, 0, 0, 0]), 0x0004_0001);
        assert_eq!(checksum32(&[0xAA]), 0x0001_0000 + 0xAA00_0000);
        assert_eq!(
            checksum32(&[0, 0, 0, 0, 1, 2, 3]),
            0x0007_0000 + 0x0102_0300
        );
        assert_eq!(
            checksum32(&[0xFF; 4]),
            0x0004_0000u32.wrapping_add(0xFFFF_FFFF)
        );
    }
}
