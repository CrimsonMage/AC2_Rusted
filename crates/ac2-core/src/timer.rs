//! The client clock (`Timer`, `core/src/timer/timer.cpp`).
//!
//! Retail keeps three values: `local_time`, the seconds the process has been running;
//! `cur_time`, the server's clock as the client believes it; and `base_time`, their difference.
//! `Timer::update_time` runs once at the top of every frame; `Timer::set_time` moves the
//! server clock forward when a time sync arrives and never moves it back. The elapsed time is
//! passed in (sans-IO) so a frame is reproducible; callers read it from a monotonic clock.

/// `Timer::base_time`, `Timer::cur_time` and `Timer::local_time`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Timer {
    base_time: f64,
    cur_time: f64,
    local_time: f64,
}

impl Timer {
    /// `Timer::Init`: every clock at zero.
    pub const fn new() -> Self {
        Self {
            base_time: 0.0,
            cur_time: 0.0,
            local_time: 0.0,
        }
    }

    /// `Timer::update_time`: `cur_time = elapsed + base_time`, never decreasing (a regression
    /// advances by `1e-12` instead), and `local_time = cur_time - base_time`.
    pub fn update_time(&mut self, elapsed: f64) {
        let old = self.cur_time;
        self.cur_time = elapsed + self.base_time;
        if self.cur_time < old {
            self.cur_time = old + 1e-12;
        }
        self.local_time = self.cur_time - self.base_time;
    }

    /// `Timer::set_time`: adopt the server time `now` when it is ahead of
    /// `elapsed + base_time` by more than `1e-9`; an earlier time is ignored.
    pub fn set_time(&mut self, now: f64, elapsed: f64) {
        let delta = now - (elapsed + self.base_time);
        if 1e-9 < delta {
            self.base_time += delta;
            self.cur_time = now;
        }
    }

    /// `Timer::get_elapsed_time() + Timer::base_time`: the server clock at `elapsed`, as
    /// `CTimeSyncHeader::UpdateTimeSensitivePayload` reads it.
    pub fn server_time_at(&self, elapsed: f64) -> f64 {
        elapsed + self.base_time
    }

    pub fn cur_time(&self) -> f64 {
        self.cur_time
    }

    pub fn local_time(&self) -> f64 {
        self.local_time
    }

    pub fn base_time(&self) -> f64 {
        self.base_time
    }
}

#[cfg(test)]
mod tests {
    use super::Timer;

    #[test]
    fn update_tracks_elapsed_and_never_runs_backwards() {
        let mut timer = Timer::new();
        timer.update_time(2.5);
        assert_eq!((timer.cur_time(), timer.local_time()), (2.5, 2.5));
        timer.update_time(2.0);
        assert_eq!(timer.cur_time(), 2.5 + 1e-12);
    }

    #[test]
    fn set_time_only_moves_the_server_clock_forward() {
        let mut timer = Timer::new();
        timer.update_time(10.0);
        timer.set_time(1000.0, 10.0);
        assert_eq!(
            (timer.cur_time(), timer.base_time(), timer.local_time()),
            (1000.0, 990.0, 10.0)
        );
        timer.set_time(500.0, 10.5);
        assert_eq!(timer.base_time(), 990.0);
        timer.update_time(11.0);
        assert_eq!((timer.cur_time(), timer.local_time()), (1001.0, 11.0));
        assert_eq!(timer.server_time_at(11.25), 1001.25);
    }
}
