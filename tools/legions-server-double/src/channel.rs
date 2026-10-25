//! A lossy network between the client and the double (test tooling): each datagram, in either
//! direction, may be dropped, duplicated or held back a while (so it arrives after later ones),
//! decided by a seeded generator so every run is the same.

use std::net::SocketAddr;

/// `splitmix64`: a small seeded generator for the channel's decisions.
#[derive(Debug, Clone)]
pub struct SplitMix(u64);

impl SplitMix {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Which way a datagram travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ToServer,
    ToClient,
}

/// One datagram in flight.
#[derive(Debug, Clone)]
pub struct InFlight {
    pub release: f64,
    pub direction: Direction,
    /// The client address (towards the server) or the server's (towards the client).
    pub addr: SocketAddr,
    pub bytes: Vec<u8>,
}

/// The channel's odds, per datagram.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Conditions {
    pub drop: f64,
    pub duplicate: f64,
    /// The chance a datagram is held back, and the longest hold (seconds).
    pub delay: f64,
    pub max_delay: f64,
}

/// What the channel did, for the tests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChannelStats {
    pub passed: u32,
    pub dropped: u32,
    pub duplicated: u32,
    pub delayed: u32,
}

/// The lossy channel.
#[derive(Debug)]
pub struct Channel {
    rng: SplitMix,
    pub conditions: Conditions,
    held: Vec<InFlight>,
    pub stats: ChannelStats,
}

impl Channel {
    pub fn new(seed: u64, conditions: Conditions) -> Self {
        Self {
            rng: SplitMix::new(seed),
            conditions,
            held: Vec::new(),
            stats: ChannelStats::default(),
        }
    }

    /// Sends a datagram at `now`: dropped, delivered now (once or twice), or held.
    pub fn send(
        &mut self,
        now: f64,
        direction: Direction,
        addr: SocketAddr,
        bytes: Vec<u8>,
    ) -> Vec<InFlight> {
        let c = self.conditions;
        if self.rng.unit() < c.drop {
            self.stats.dropped += 1;
            return Vec::new();
        }
        let copies = if self.rng.unit() < c.duplicate {
            self.stats.duplicated += 1;
            2
        } else {
            1
        };
        let mut now_out = Vec::new();
        for _ in 0..copies {
            let flight = InFlight {
                release: now,
                direction,
                addr,
                bytes: bytes.clone(),
            };
            if self.rng.unit() < c.delay {
                self.stats.delayed += 1;
                self.held.push(InFlight {
                    release: now + self.rng.unit() * c.max_delay,
                    ..flight
                });
            } else {
                self.stats.passed += 1;
                now_out.push(flight);
            }
        }
        now_out
    }

    /// The held datagrams due by `now`, oldest release first.
    pub fn release(&mut self, now: f64) -> Vec<InFlight> {
        let mut due: Vec<InFlight> = Vec::new();
        let mut kept = Vec::new();
        for flight in self.held.drain(..) {
            if flight.release <= now {
                due.push(flight);
            } else {
                kept.push(flight);
            }
        }
        self.held = kept;
        due.sort_by(|a, b| a.release.total_cmp(&b.release));
        due
    }

    /// Whether anything is still in flight.
    pub fn idle(&self) -> bool {
        self.held.is_empty()
    }
}
