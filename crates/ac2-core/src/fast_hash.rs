//! A fast, non-cryptographic hasher for per-frame tables keyed by internal identities.
//!
//! `std`'s default SipHash resists collision attacks on attacker-chosen keys. Render-side tables
//! keyed by pointers, IDs and float bits never see such keys, and rebuilding them every frame
//! with SipHash dominated the sun-shadow and doorway planners. This is the multiply-rotate hash
//! rustc uses for its own tables (FxHash). Iteration order differs from `RandomState`'s, which
//! is already randomized per process, so no caller can depend on it.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[derive(Debug, Clone, Copy, Default)]
pub struct FastHasher {
    hash: u64,
}

impl FastHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            self.add(u64::from_le_bytes(
                chunk.try_into().expect("eight-byte chunk"),
            ));
        }
        let rest = chunks.remainder();
        if !rest.is_empty() {
            let mut word = [0; 8];
            word[..rest.len()].copy_from_slice(rest);
            self.add(u64::from_le_bytes(word));
        }
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.add(value);
    }

    #[inline]
    fn write_usize(&mut self, value: usize) {
        self.add(value as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

pub type FastBuildHasher = BuildHasherDefault<FastHasher>;
pub type FastHashMap<K, V> = HashMap<K, V, FastBuildHasher>;
pub type FastHashSet<T> = HashSet<T, FastBuildHasher>;

/// An empty set sized for `capacity` entries.
pub fn fast_hash_set_with_capacity<T>(capacity: usize) -> FastHashSet<T> {
    FastHashSet::with_capacity_and_hasher(capacity, FastBuildHasher::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{BuildHasher, Hash};

    fn hash_of<T: Hash + ?Sized>(value: &T) -> u64 {
        FastBuildHasher::default().hash_one(value)
    }

    #[test]
    fn equal_keys_hash_equally_and_distinct_keys_spread() {
        let key = (
            0x7f00_1234_usize,
            0x7f00_5678_usize,
            [[1.0f32.to_bits(); 4]; 4],
        );
        assert_eq!(hash_of(&key), hash_of(&key.clone()));
        let hashes = (0..10_000_usize)
            .map(|index| hash_of(&(index * 64, index * 64 + 8)))
            .collect::<FastHashSet<_>>();
        assert_eq!(hashes.len(), 10_000);
    }

    #[test]
    fn byte_slices_include_their_tail() {
        assert_ne!(hash_of(&[1u8, 2, 3][..]), hash_of(&[1u8, 2, 4][..]));
        assert_ne!(hash_of(&[0u8; 9][..]), hash_of(&[0u8; 10][..]));
    }

    #[test]
    fn sets_and_maps_behave_as_std_collections() {
        let mut set = fast_hash_set_with_capacity(4);
        assert!(set.insert(3_u32));
        assert!(!set.insert(3_u32));
    }
}
