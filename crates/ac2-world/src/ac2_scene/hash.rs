//! The integer hashes AC2 uses to make procedural scenery deterministic per landscape vertex.
//! All arithmetic is 32-bit wrapping, as in the retail client.

const UNIT: f64 = 1.0 / 65536.0;

/// `SceneGroup` placement random (`prand2`): a 16-bit fraction in `[0, 1)` from the global
/// vertex `(i, j)` and a per-draw key `k`.
pub(super) fn prand2(i: u32, j: u32, k: u32) -> f64 {
    let inner = i
        .wrapping_mul(j)
        .wrapping_mul(j)
        .wrapping_mul(k)
        .wrapping_mul(k)
        .wrapping_mul(0x2BD3_8901)
        .wrapping_add(0x6B3C_5F11);
    let value = k
        .wrapping_mul(0x6243_3D7D)
        .wrapping_sub(i.wrapping_mul(inner))
        .wrapping_sub(j.wrapping_mul(0x4A65_06DB));
    f64::from(value >> 16) * UNIT
}

/// `SceneGroup::CheckFreq`: whether a group with frequency `freq` fires at vertex `(x, y)` for
/// object seed `seed`.
pub(super) fn check_freq(freq: f64, x: u32, y: u32, seed: u32) -> bool {
    let n = seed.wrapping_add(0x0119_1E67);
    let value = 0x6243_3D7D_u32
        .wrapping_sub(
            n.wrapping_mul(x)
                .wrapping_mul(x)
                .wrapping_mul(y)
                .wrapping_mul(y)
                .wrapping_mul(0x2BD3_8901),
        )
        .wrapping_mul(n)
        .wrapping_sub(y.wrapping_mul(0x4A65_06DB))
        .wrapping_sub(x.wrapping_mul(0x6B3C_5F11));
    f64::from(value >> 16) * UNIT < freq
}

/// `CLandBlockData::CacheNearValues` scene pick: the fraction that selects one of a scene type's
/// frequency-weighted scene files at vertex `(x, y)`.
pub(super) fn scene_rand(x: u32, y: u32) -> f64 {
    let value = y
        .wrapping_mul(x)
        .wrapping_mul(x)
        .wrapping_mul(0x019A_266F)
        .wrapping_sub(0x4A65_06DB)
        .wrapping_mul(y)
        .wrapping_sub(x.wrapping_mul(0x6B3C_5F11))
        .wrapping_sub(0x39B6_25FB);
    f64::from(value >> 16) * UNIT
}

#[cfg(test)]
mod tests {
    use super::{check_freq, prand2, scene_rand};

    // Reference values computed by the Python port of the Legions 7160 hashes
    // (`/tmp/ac2probe/scene/gen.py`, unbounded integers masked to 32 bits).

    #[test]
    fn prand2_matches_the_reference() {
        assert_eq!(prand2(3149, 1245, 0x28D0_642D), 0.745086669921875);
        assert_eq!(prand2(3260, 4000, 0x0DE8_76C7), 0.4124755859375);
        assert_eq!(prand2(4095, 4095, 0xFFFF_FFFF), 0.92742919921875);
        assert_eq!(prand2(0, 0, 0), 0.0);
    }

    #[test]
    fn scene_rand_matches_the_reference() {
        assert_eq!(scene_rand(3149, 1245), 0.5406951904296875);
        assert_eq!(scene_rand(3260, 4000), 0.9131622314453125);
        assert_eq!(scene_rand(17, 23), 0.759124755859375);
    }

    #[test]
    fn check_freq_compares_the_reference_fraction() {
        for (x, y, seed, fraction) in [
            (3149, 1245, 0x923B1, 0.8648223876953125),
            (3260, 4000, 12345, 0.5180816650390625),
        ] {
            assert!(!check_freq(fraction, x, y, seed));
            assert!(check_freq(fraction + 1.0 / 131072.0, x, y, seed));
        }
    }
}
