/// The checksum key generator of `CryptoSystem`, a `QTIsaac<8, u32>`.
///
/// The constructor copies the seed into the `a`, `b` and `c` registers and starts from 256 zero
/// result words; `randinit` runs four golden-ratio mixes and two passes and generates the first
/// block. `CryptoSystem::GetNextCryptoKey` hands out results from index 255 down to 0 and then
/// refills.
#[derive(Clone)]
pub struct Isaac {
    memory: [u32; 256],
    results: [u32; 256],
    a: u32,
    b: u32,
    c: u32,
    remaining: usize,
}

impl std::fmt::Debug for Isaac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Isaac")
            .field("remaining", &self.remaining)
            .finish_non_exhaustive()
    }
}

impl Isaac {
    pub fn new(seed: u32) -> Self {
        let mut state = Self {
            memory: [0; 256],
            results: [0; 256],
            a: seed,
            b: seed,
            c: seed,
            remaining: 0,
        };
        let mut mix = [0x9E37_79B9_u32; 8];
        for _ in 0..4 {
            shuffle(&mut mix);
        }
        for pass in 0..2 {
            for base in (0..256).step_by(8) {
                let source = if pass == 0 {
                    &state.results
                } else {
                    &state.memory
                };
                for (lane, value) in mix.iter_mut().enumerate() {
                    *value = value.wrapping_add(source[base + lane]);
                }
                shuffle(&mut mix);
                state.memory[base..base + 8].copy_from_slice(&mix);
            }
        }
        state.generate();
        state
    }

    /// The next key; consume exactly one per newly sent (or newly received) encrypted sequence.
    pub fn next_key(&mut self) -> u32 {
        if self.remaining == 0 {
            self.generate();
        }
        self.remaining -= 1;
        self.results[self.remaining]
    }

    /// `QTIsaac::isaac`: one block of 256 results.
    fn generate(&mut self) {
        self.c = self.c.wrapping_add(1);
        self.b = self.b.wrapping_add(self.c);
        for i in 0..256 {
            let x = self.memory[i];
            self.a ^= match i & 3 {
                0 => self.a << 13,
                1 => self.a >> 6,
                2 => self.a << 2,
                _ => self.a >> 16,
            };
            self.a = self.a.wrapping_add(self.memory[(i + 128) & 255]);
            let y = self.memory[((x >> 2) & 255) as usize]
                .wrapping_add(self.a)
                .wrapping_add(self.b);
            self.memory[i] = y;
            self.b = self.memory[((y >> 10) & 255) as usize].wrapping_add(x);
            self.results[i] = self.b;
        }
        self.remaining = 256;
    }
}

/// `QTIsaac::shuffle`, the ISAAC mix with 32-bit wraparound.
fn shuffle(v: &mut [u32; 8]) {
    v[0] ^= v[1] << 11;
    v[3] = v[3].wrapping_add(v[0]);
    v[1] = v[1].wrapping_add(v[2]);
    v[1] ^= v[2] >> 2;
    v[4] = v[4].wrapping_add(v[1]);
    v[2] = v[2].wrapping_add(v[3]);
    v[2] ^= v[3] << 8;
    v[5] = v[5].wrapping_add(v[2]);
    v[3] = v[3].wrapping_add(v[4]);
    v[3] ^= v[4] >> 16;
    v[6] = v[6].wrapping_add(v[3]);
    v[4] = v[4].wrapping_add(v[5]);
    v[4] ^= v[5] << 10;
    v[7] = v[7].wrapping_add(v[4]);
    v[5] = v[5].wrapping_add(v[6]);
    v[5] ^= v[6] >> 4;
    v[0] = v[0].wrapping_add(v[5]);
    v[6] = v[6].wrapping_add(v[7]);
    v[6] ^= v[7] << 8;
    v[1] = v[1].wrapping_add(v[6]);
    v[7] = v[7].wrapping_add(v[0]);
    v[7] ^= v[0] >> 9;
    v[2] = v[2].wrapping_add(v[7]);
    v[0] = v[0].wrapping_add(v[1]);
}

#[cfg(test)]
mod tests {
    use super::Isaac;

    /// Keys produced by the original `randinit` and `isaac` routines executed as x86, at output
    /// indices 0, 1, 2, 3, 254, 255, 256, 257 and 511 (the refill boundary is at 256).
    #[test]
    fn matches_the_original_generator_across_a_refill() {
        let cases: [(u32, [u32; 9]); 4] = [
            (
                0,
                [
                    0x182600f3, 0x300b4a8d, 0x301b6622, 0xb08acd21, 0xd91aa738, 0xe76dd339,
                    0x7a68710f, 0x6554abda, 0xf650e4c8,
                ],
            ),
            (
                1,
                [
                    0x38168fdd, 0x173ff3d7, 0x832ddc13, 0xc4291d70, 0xbd249809, 0xfe83b016,
                    0x1ba8a003, 0x061c754f, 0xa7b65ae0,
                ],
            ),
            (
                0x1234_5678,
                [
                    0xfdd4afeb, 0x398311d2, 0xc71829a4, 0xb19df96a, 0xd92733af, 0xdf7ca7e9,
                    0xa608787a, 0x2271e865, 0x4b91d57d,
                ],
            ),
            (
                u32::MAX,
                [
                    0x4317c641, 0x1d124727, 0xe877449d, 0x5c326457, 0xbd249809, 0xfe83b016,
                    0xe28dc1f7, 0x7b3c165a, 0xcd945ebf,
                ],
            ),
        ];
        let indices = [0, 1, 2, 3, 254, 255, 256, 257, 511];
        for (seed, expected) in cases {
            let mut generator = Isaac::new(seed);
            let keys: Vec<u32> = (0..512).map(|_| generator.next_key()).collect();
            for (slot, index) in indices.iter().enumerate() {
                assert_eq!(keys[*index], expected[slot], "seed {seed:#x} index {index}");
            }
        }
    }
}
