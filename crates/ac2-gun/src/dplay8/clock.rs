//! The clock behind the protocol: `timeGetTime()` (dpnet imports WINMM `timeGetTime`, used by
//! every `Dpl8r_*` function that stamps or compares a time). Millisecond ticks wrap at 2^32 and
//! every comparison in the port uses wrapping subtraction, as the retail code does
//! (`DVar - stamp`, `(int)(a - b) < 0`).

use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

/// A source of `timeGetTime()` milliseconds and a way to wait (the protocol layer waits only in
/// `Close`, where retail sleeps 10 ms between polls, dpnet 0x00420510).
pub trait Clock {
    /// `timeGetTime()`.
    fn now_ms(&self) -> u32;
    /// `Sleep(ms)`: a real clock sleeps, a fake clock advances itself.
    fn sleep_ms(&self, ms: u32);
}

/// The wall clock: milliseconds since construction plus a start offset, so the first reading
/// is not 0.
pub struct SystemClock {
    start: Instant,
    base: u32,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            base: 1000,
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u32 {
        self.base
            .wrapping_add(self.start.elapsed().as_millis() as u32)
    }

    fn sleep_ms(&self, ms: u32) {
        std::thread::sleep(std::time::Duration::from_millis(u64::from(ms)));
    }
}

/// A clock the test advances by hand. Clones share the same time.
#[derive(Clone, Default)]
pub struct FakeClock {
    now: Rc<Cell<u32>>,
}

impl FakeClock {
    pub fn new(start: u32) -> Self {
        Self {
            now: Rc::new(Cell::new(start)),
        }
    }

    pub fn advance(&self, ms: u32) {
        self.now.set(self.now.get().wrapping_add(ms));
    }

    pub fn set(&self, t: u32) {
        self.now.set(t);
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> u32 {
        self.now.get()
    }

    fn sleep_ms(&self, ms: u32) {
        self.advance(ms);
    }
}

/// The random source for session ids: DX9 `FUN_004242bb` (0x004242bb, dpnet9.dll.c:30884) is
/// `CryptGenRandom(DAT_00454c50, 4, &v)`, called again while the value is 0 (0x00424bef:31475-31481).
pub trait RandomSource {
    fn next_u32(&mut self) -> u32;
}

/// The operating system's random bytes (`/dev/urandom`; where that is missing, a clock-seeded
/// generator: not cryptographic, but only a connection key, not a secret).
// open: Windows `CryptGenRandom`; non-Unix targets use the fallback.
pub struct OsRandom {
    state: u32,
}

impl OsRandom {
    pub fn new() -> Self {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() ^ (d.as_secs() as u32))
            .unwrap_or(0x9e37_79b9);
        Self { state: t | 1 }
    }
}

impl Default for OsRandom {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomSource for OsRandom {
    fn next_u32(&mut self) -> u32 {
        use std::io::Read;
        let mut b = [0u8; 4];
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            if f.read_exact(&mut b).is_ok() {
                return u32::from_le_bytes(b);
            }
        }
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.state
    }
}

/// Returns scripted values in turn (tests); the last one repeats.
pub struct FixedRandom(pub Vec<u32>, pub usize);

impl RandomSource for FixedRandom {
    fn next_u32(&mut self) -> u32 {
        let v = self.0[self.1.min(self.0.len() - 1)];
        self.1 += 1;
        v
    }
}
