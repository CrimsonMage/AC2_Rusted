//! The DX9 thread pool's timers (`dpnet.dll` 5.3.0000000.900, "pool" object; the 8.0 20 ms
//! `timeSetEvent` wheel is gone):
//!
//! * `FUN_0042a742` (dpnet9.dll.c:36796-36829) sets up 1024 buckets of 8 bytes (0x2000 bytes),
//!   `wheelTime = timeGetTime() & ~3`, and a **4 ms periodic waitable timer** (due -10000 x 4
//!   ... period 4, `SetWaitableTimer(.., 4, ..)`); `timeBeginPeriod(1)` is called at
//!   dpnet9.dll.c:35312;
//! * `FUN_0042a88d` / `FUN_0042a903` (36899-36946) schedule: `expiry = timeGetTime() + delay`,
//!   bucket `((expiry + 3) >> 2) & 0x3ff`;
//! * `FUN_0042a96d` (36955-37010) is the tick: `now4 = timeGetTime() & ~3`; it visits
//!   `min((now4 - wheelTime) >> 2, 0x400)` buckets from `wheelTime >> 2`; an entry is due when
//!   `now4 != expiry && now4 - expiry >= 0` (strictly later), a not-yet-due entry (a delay of
//!   4096 ms or more wraps the wheel) stays; then `wheelTime = now4`.
//!
//! There are no granularity requests, no long list and no 30000 ms list in DX9. With zero worker
//! threads (GunDll) the due timers run inside `DoWork`, so the owner calls [`Timers::tick`] and
//! [`Timers::pop_ready`] from its pump; a pump every [`DEFAULT_PERIOD_MS`] (4 ms) reproduces the
//! waitable timer's cadence.
// open: DX9 hands the due list to the worker in reverse collection order (`*piVar6 = *param_2`
// pushes at the head, 0x0042a9e0) and compensates a timer that is already late when scheduled
// (`wheel skew`, 0x0042a8f0); both are not modelled: due entries run in collection order.

use std::collections::VecDeque;

/// Buckets in the wheel (`& 0x3ff`).
const WHEEL_SIZE: usize = 0x400;

/// The waitable timer's period (`SetWaitableTimer(.., 4, ..)`, 0x0042a742:36825).
pub const DEFAULT_PERIOD_MS: u32 = 4;

#[derive(Debug, Clone)]
struct Entry<T> {
    id: u32,
    expiry: u32,
    payload: T,
}

/// The wheel and the ready queue.
pub struct Timers<T> {
    wheel: Vec<Vec<Entry<T>>>,
    wheel_time: u32,
    /// Due timers and queued work, in the order they became ready.
    ready: VecDeque<Entry<T>>,
    serial: u32,
}

impl<T: Clone> Timers<T> {
    /// `FUN_0042a742`: the wheel starts at `now & ~3`.
    pub fn new(now: u32) -> Self {
        Self {
            wheel: (0..WHEEL_SIZE).map(|_| Vec::new()).collect(),
            wheel_time: now & !3,
            ready: VecDeque::new(),
            serial: 0,
        }
    }

    /// The tick period a real-time driver should call [`Timers::tick`] at: always 4 ms.
    pub fn period(&self) -> u32 {
        DEFAULT_PERIOD_MS
    }

    fn next_id(&mut self) -> u32 {
        self.serial = self.serial.wrapping_add(1);
        if self.serial == 0 {
            self.serial = 1;
        }
        self.serial
    }

    /// Retail's 8.0 housekeeping shortened and restored a tick period; DX9's period is fixed.
    pub fn housekeeping(&mut self) {}

    /// `FUN_0042a88d`: returns the handle `cancel` takes. `granularity` is ignored (DX9 has none).
    pub fn schedule(&mut self, now: u32, delay: u32, _granularity: u32, payload: T) -> u32 {
        let id = self.next_id();
        let expiry = now.wrapping_add(delay);
        let slot = (expiry.wrapping_add(3) >> 2) as usize & (WHEEL_SIZE - 1);
        self.wheel[slot].push(Entry {
            id,
            expiry,
            payload,
        });
        id
    }

    /// Queue a callback for the next worker pass (the pool's ready list).
    pub fn queue_work(&mut self, payload: T) -> u32 {
        let id = self.next_id();
        self.ready.push_back(Entry {
            id,
            expiry: 0,
            payload,
        });
        id
    }

    /// True when the timer or work item had not run yet and is now removed.
    pub fn cancel(&mut self, id: u32) -> bool {
        if id == 0 {
            return false;
        }
        if let Some(i) = self.ready.iter().position(|e| e.id == id) {
            self.ready.remove(i);
            return true;
        }
        for b in &mut self.wheel {
            if let Some(i) = b.iter().position(|e| e.id == id) {
                b.remove(i);
                return true;
            }
        }
        false
    }

    /// `FUN_0042a96d`.
    pub fn tick(&mut self, now: u32) {
        let now4 = now & !3;
        let diff = now4.wrapping_sub(self.wheel_time);
        if diff == 0 || (diff as i32) < 0 {
            return;
        }
        let steps = (diff >> 2).min(0x400);
        for at in ((self.wheel_time >> 2) as usize..).take(steps as usize) {
            let slot = at & (WHEEL_SIZE - 1);
            let bucket = std::mem::take(&mut self.wheel[slot]);
            for e in bucket {
                if now4 == e.expiry || (now4.wrapping_sub(e.expiry) as i32) < 0 {
                    self.wheel[slot].push(e);
                } else {
                    self.ready.push_back(e);
                }
            }
        }
        self.wheel_time = now4;
    }

    /// The next ready callback, if any: `(handle, payload)`.
    pub fn pop_ready(&mut self) -> Option<(u32, T)> {
        self.ready.pop_front().map(|e| (e.id, e.payload))
    }

    pub fn has_ready(&self) -> bool {
        !self.ready.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fire_time(delay: u32, step: u32, limit: u32) -> Option<u32> {
        let mut t = Timers::<u8>::new(1000);
        t.schedule(1000, delay, 0, 7);
        let mut now = 1000;
        while now <= 1000 + limit {
            now += step;
            t.tick(now);
            if t.has_ready() {
                return Some(now - 1000);
            }
        }
        None
    }

    #[test]
    fn a_timer_fires_on_the_first_4_ms_tick_strictly_after_its_expiry() {
        // expiry 1100: due when `now & ~3 > 1100`, i.e. at 1104.
        assert_eq!(fire_time(100, 4, 500), Some(104));
        assert_eq!(fire_time(100, 20, 500), Some(120));
        assert_eq!(fire_time(0, 4, 100), Some(4));
    }

    #[test]
    fn cancel_removes_a_pending_timer() {
        let mut t = Timers::<u8>::new(0);
        let id = t.schedule(0, 50, 0, 1);
        assert!(t.cancel(id));
        t.tick(100);
        assert!(!t.has_ready());
        assert!(!t.cancel(id));
    }

    #[test]
    fn queue_work_runs_before_later_timers() {
        let mut t = Timers::<u8>::new(0);
        t.schedule(0, 0, 0, 1);
        t.queue_work(2);
        t.tick(20);
        assert_eq!(t.pop_ready().map(|x| x.1), Some(2));
        assert_eq!(t.pop_ready().map(|x| x.1), Some(1));
    }

    #[test]
    fn a_delay_beyond_the_wheel_stays_until_due() {
        let mut t = Timers::<u8>::new(0);
        t.schedule(0, 10_000, 0, 1);
        let mut now = 0;
        while now < 9_900 {
            now += 4;
            t.tick(now);
            assert!(!t.has_ready(), "{now}");
        }
        while now < 10_100 && !t.has_ready() {
            now += 4;
            t.tick(now);
        }
        assert_eq!(now, 10_004);
    }
}
