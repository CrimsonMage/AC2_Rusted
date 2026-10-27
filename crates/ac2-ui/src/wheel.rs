//! Wheel input from the window: turns the platform's scroll deltas into whole notches.
//!
//! Retail took the wheel as DirectInput's Z axis (`CInputManager_WIN32::ProcessDeviceData`,
//! `dwData` in multiples of `WHEEL_DELTA`, one action per sample), so a notch is one action. A
//! window reports two kinds of delta. A line delta (a detented wheel, or a line-based smooth
//! scroll) is already in notches. A pixel delta (a macOS trackpad, a Wayland smooth scroll, a
//! high-resolution wheel) is a distance, so its fractions are kept here until they add up to a
//! whole notch: a trackpad drag of 15 pixels turns no notch yet, and the next 25 complete one.
//! The sign is the window's: positive scrolls up, away from the user. macOS natural scrolling
//! reaches the window already applied, so nothing is inverted here.

/// `WHEEL_DELTA`: the units of one notch in `InputEvent::Wheel`.
pub const WHEEL_DELTA: i32 = 120;

/// The logical pixels one notch of a pixel-reporting device covers. A notch scrolls one line or
/// one list item, which is about this many pixels on the client's 1024 x 768 layout.
pub const PIXELS_PER_NOTCH: f64 = 40.0;

/// The fractional notches a window has not yet turned into whole ones.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WheelAccumulator {
    remainder: f64,
}

impl WheelAccumulator {
    /// A line delta (`MouseScrollDelta::LineDelta`), in notches. Whole values pass through.
    pub fn line_delta(&mut self, lines: f32) -> i32 {
        self.add(f64::from(lines))
    }

    /// A pixel delta (`MouseScrollDelta::PixelDelta`), in logical pixels. The fraction of a notch
    /// it leaves is kept for the next delta.
    pub fn pixel_delta(&mut self, pixels: f64) -> i32 {
        self.add(pixels / PIXELS_PER_NOTCH)
    }

    /// A new gesture begins (`TouchPhase::Started`): the fraction left by the last one is
    /// dropped, so a small flick after a pause does not turn a notch on its own.
    pub fn start_gesture(&mut self) {
        self.remainder = 0.0;
    }

    /// The whole notches in `notches` plus what was kept, truncated toward zero, so the kept
    /// fraction has the same sign as the motion.
    fn add(&mut self, notches: f64) -> i32 {
        if !notches.is_finite() {
            return 0;
        }
        self.remainder += notches;
        let whole = self.remainder.trunc();
        self.remainder -= whole;
        whole as i32
    }
}

/// The `InputEvent::Wheel` delta for `notches` whole notches (positive away from the user).
pub fn wheel_event_delta(notches: i32) -> i32 {
    notches.saturating_mul(WHEEL_DELTA)
}
