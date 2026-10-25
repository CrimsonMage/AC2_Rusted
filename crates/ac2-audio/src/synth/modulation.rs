//! The connection blocks (`CONNECTION` records of the `art1` / `art2` chunks, read by `music::dls`)
//! of a region and its instrument, resolved for one note.
//!
//! A block is `(source, control, destination, transform, scale)`. The port follows the DLS
//! connection definitions as it reads them (the spec text was not available here; the constants
//! below are the values the corpus uses, and each is checked against the corpus counts in the
//! module docs of `synth`):
//!
//! - A `CONN_SRC_NONE` block with no control is a fixed value of its destination (the scale is the
//!   value). Blocks are read `art2` first, then `art1` (DLS Level 2 overrides Level 1 for the
//!   same destination), and within a level the region's come before the instrument's: the first
//!   block that sets a destination wins.
//! - A source block routes a modulator to a destination: `CONN_SRC_LFO` (1),
//!   `CONN_SRC_KEYONVELOCITY` (2), `CONN_SRC_KEYNUMBER` (3), `CONN_SRC_EG2` (5) and `CONN_SRC_VIBRATO`
//!   (9, the second LFO). A control scales the route per frame: `CONN_SRC_CC1` (`0x81`, the mod
//!   wheel) or `CONN_SRC_CHANNELPRESSURE` (`0x08`); a route with no control is always on. Other
//!   sources and controls are not applied.
//! - Units: pitch in cents, attenuation in dB (positive attenuates), pan in 0.1 percent of the
//!   range (±500 is hard left or right), LFO and vibrato frequency in absolute-pitch cents (see
//!   `lfo`), the filter cutoff in absolute-pitch cents (see `filter`), the filter resonance in
//!   centibels, times in timecents, sustain in 0.1 percent. All scales are 16.16 fixed-point: the
//!   raw `i32` over 65536. The attenuation unit and the key and velocity sources' ranges are the
//!   port's reading.
//! - `CONN_SRC_KEYNUMBER` reads the key minus middle C (60), and `CONN_SRC_KEYONVELOCITY` reads the
//!   velocity over 127 (for attenuation, `(127 - velocity) / 127`, so a louder note attenuates less).
//!
//! Key and velocity routes are static (fixed for the note). LFO, vibrato and EG2 routes move with
//! the voice, so they are kept as [`Route`]s for the voice to evaluate each frame.
//!
//! The DLS Level 2 identifications (names and numbers from the DirectX 8 SDK's `dls2.h`, in
//! `third_party/directx8/include`; the units from the corpus, not the spec text): `CONN_SRC_CHANNELPRESSURE`
//! 0x0008 and `CONN_SRC_VIBRATO` 0x0009 (source and control), `CONN_DST_VIB_FREQUENCY` 0x0114,
//! `CONN_DST_VIB_STARTDELAY` 0x0115, `CONN_DST_FILTER_CUTOFF` 0x0500 and `CONN_DST_FILTER_Q` 0x0501.
//! The corpus's values fit those units: the cutoffs are whole absolute cents (8302, 11170, 11602,
//! 11921, ...: 822 Hz to 8 kHz); the fixed `0x7FFFFFFF` is an open filter; a key-on-velocity route of
//! -2400 cents is the velocity to cutoff; the Q blocks are 30 and 128 centibels; the vibrato depth is
//! 50 cents (`3276800`, one block 25 cents); the vibrato delays are timecents (`-11960` is 1 ms); the
//! vibrato rates are absolute cents (`2790` is 8.18 Hz). The DLS2 EG delay and hold destinations
//! (0x020B, 0x020C, 0x030F, 0x0310) are in the header but not modelled (see the module docs of
//! `synth`).

use super::envelope::{EnvelopeParams, Shape};
use super::filter::resonance_q;
use super::lfo::hz_from_absolute_cents;
use crate::music::dls::Articulation;

/// `CONN_SRC_NONE`.
const SRC_NONE: u16 = 0x0000;
/// `CONN_SRC_LFO`.
const SRC_LFO: u16 = 0x0001;
/// `CONN_SRC_KEYONVELOCITY`.
const SRC_KEY_ON_VELOCITY: u16 = 0x0002;
/// `CONN_SRC_KEYNUMBER`.
const SRC_KEY_NUMBER: u16 = 0x0003;
/// `CONN_SRC_EG2`.
const SRC_EG2: u16 = 0x0005;
/// `CONN_SRC_VIBRATO`: the second LFO (DLS Level 2).
const SRC_VIBRATO: u16 = 0x0009;
/// `CONN_SRC_CHANNELPRESSURE`: a control that scales a route by the channel's pressure.
const CONTROL_CHANNEL_PRESSURE: u16 = 0x0008;
/// `CONN_SRC_CC1`: the mod wheel, a control that scales a route.
const CONTROL_MOD_WHEEL: u16 = 0x0081;
/// No control.
const CONTROL_NONE: u16 = 0x0000;

/// `CONN_DST_GAIN` (the port's attenuation destination).
const DST_ATTENUATION: u16 = 0x0001;
/// `CONN_DST_PITCH`.
const DST_PITCH: u16 = 0x0003;
/// `CONN_DST_PAN`.
const DST_PAN: u16 = 0x0004;
/// `CONN_DST_LFO_FREQUENCY`.
const DST_LFO_FREQUENCY: u16 = 0x0104;
/// `CONN_DST_LFO_STARTDELAY`.
const DST_LFO_DELAY: u16 = 0x0105;
/// `CONN_DST_VIB_FREQUENCY` (DLS Level 2).
const DST_VIB_FREQUENCY: u16 = 0x0114;
/// `CONN_DST_VIB_STARTDELAY` (DLS Level 2).
const DST_VIB_DELAY: u16 = 0x0115;
const DST_EG1_ATTACK: u16 = 0x0206;
const DST_EG1_DECAY: u16 = 0x0207;
const DST_EG1_RELEASE: u16 = 0x0209;
const DST_EG1_SUSTAIN: u16 = 0x020A;
const DST_EG2_ATTACK: u16 = 0x030A;
const DST_EG2_DECAY: u16 = 0x030B;
const DST_EG2_RELEASE: u16 = 0x030D;
const DST_EG2_SUSTAIN: u16 = 0x030E;
/// `CONN_DST_FILTER_CUTOFF` (DLS Level 2, absolute-pitch cents).
const DST_FILTER_CUTOFF: u16 = 0x0500;
/// `CONN_DST_FILTER_Q` (DLS Level 2, centibels).
const DST_FILTER_Q: u16 = 0x0501;

/// The middle C the key-number source counts from.
const MIDDLE_C: f64 = 60.0;
/// The LFO's default delay (none).
const NO_DELAY: f64 = 0.0;

/// The modulator a per-frame route reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The LFO's output, -1 to 1.
    Lfo,
    /// The pitch envelope's level, 0 to 1.
    Eg2,
    /// The vibrato LFO's output, -1 to 1.
    Vibrato,
}

/// What a per-frame route moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dest {
    /// Cents of pitch.
    Pitch,
    /// dB of attenuation.
    Attenuation,
    /// Pan position (-1 left, +1 right).
    Pan,
    /// Cents of filter cutoff (relative to the note's cutoff).
    Cutoff,
}

/// The control that scales a per-frame route (its depth is 0 to 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    /// No control: the route is always on.
    None,
    /// The mod wheel (CC1).
    ModWheel,
    /// The channel pressure.
    ChannelPressure,
}

/// A route whose source moves during the note: `scale` times the source, into `dest`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Route {
    pub source: Source,
    pub control: Control,
    pub dest: Dest,
    /// In the destination's unit per unit of the source.
    pub scale: f64,
}

/// An LFO a note runs (the first LFO, or the vibrato), when a block routes it or sets its
/// frequency or delay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LfoParams {
    pub frequency: f64,
    pub delay: f64,
}

/// The note's resonant low-pass filter: the cutoff (absolute-pitch cents, its fixed and static
/// parts; the routes move it per block) and the resonance Q.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterParams {
    pub cutoff_cents: f64,
    pub q: f64,
}

/// What a note's connection blocks set, resolved for its key and velocity.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteModulation {
    /// The volume envelope (EG1, dB-shaped).
    pub eg1: EnvelopeParams,
    /// The pitch envelope (EG2, linear).
    pub eg2: EnvelopeParams,
    pub lfo: Option<LfoParams>,
    /// The vibrato LFO (`CONN_SRC_VIBRATO` routes and the vibrato destinations).
    pub vibrato: Option<LfoParams>,
    /// The low-pass filter, when a block sets a cutoff (or routes one).
    pub filter: Option<FilterParams>,
    /// The static pitch offset in cents (a fixed pitch, and key routes).
    pub pitch_cents: f64,
    /// The static attenuation in dB (velocity and fixed routes).
    pub attenuation_db: f64,
    /// The static pan position (-1 to 1).
    pub pan: f64,
    /// The routes that move during the note.
    pub routes: Vec<Route>,
}

impl Default for NoteModulation {
    /// A note with no connection blocks: no LFO, no filter, no offsets, immediate envelopes.
    fn default() -> Self {
        Self {
            eg1: EnvelopeParams::immediate(Shape::Db),
            eg2: EnvelopeParams::immediate(Shape::Linear),
            lfo: None,
            vibrato: None,
            filter: None,
            pitch_cents: 0.0,
            attenuation_db: 0.0,
            pan: 0.0,
            routes: Vec::new(),
        }
    }
}

/// A DLS 16.16 fixed-point value.
fn scaled(raw: i32) -> f64 {
    f64::from(raw) / 65_536.0
}

/// Seconds for a 16.16 timecent value (`2^(tc / 1200)`; the minimum 32-bit value is zero time).
pub fn seconds_from_timecents(tc: f64) -> f64 {
    if tc <= scaled(i32::MIN) {
        0.0
    } else {
        (tc / 1200.0).exp2()
    }
}

/// The values the blocks set for one note, before they are shaped into the note's parameters.
#[derive(Default)]
struct Setting {
    eg1_attack: Option<f64>,
    eg1_decay: Option<f64>,
    eg1_release: Option<f64>,
    eg1_sustain: Option<f64>,
    eg2_attack: Option<f64>,
    eg2_decay: Option<f64>,
    eg2_release: Option<f64>,
    eg2_sustain: Option<f64>,
    lfo_frequency: Option<f64>,
    lfo_delay: Option<f64>,
    vib_frequency: Option<f64>,
    vib_delay: Option<f64>,
    cutoff: Option<f64>,
    resonance: Option<f64>,
    pitch: Option<f64>,
    attenuation: Option<f64>,
    pan: Option<f64>,
    /// Whether a block routes the LFO (so it runs even with no fixed frequency).
    lfo_routed: bool,
    /// Whether a block routes the vibrato LFO.
    vib_routed: bool,
    /// Whether a block routes the filter cutoff (the cutoff then exists even with no fixed value).
    cutoff_routed: bool,
    /// The timecent deltas from key and velocity routes, per EG time.
    eg1_time_delta: [f64; 3],
    eg2_time_delta: [f64; 3],
    eg1_sustain_delta: f64,
    eg2_sustain_delta: f64,
    static_pitch: f64,
    static_attenuation: f64,
    static_pan: f64,
    static_cutoff: f64,
    static_resonance: f64,
    routes: Vec<Route>,
}

/// Records a fixed value: the first block that sets a destination keeps it.
fn set_first(slot: &mut Option<f64>, value: f64) {
    if slot.is_none() {
        *slot = Some(value);
    }
}

impl Setting {
    /// Applies one articulation's blocks for a note at `key` and `velocity`.
    fn read(&mut self, art: &Articulation, key: u8, velocity: u8) {
        for c in &art.connections {
            if c.source == SRC_NONE && c.control == CONTROL_NONE {
                self.fixed(c.destination, scaled(c.scale));
                continue;
            }
            self.routed(
                c.source,
                c.control,
                c.destination,
                scaled(c.scale),
                key,
                velocity,
            );
        }
    }

    /// A block with no source: a fixed value of its destination (`scale` in the destination's
    /// unit, see the module docs).
    fn fixed(&mut self, destination: u16, value: f64) {
        match destination {
            DST_EG1_ATTACK => set_first(&mut self.eg1_attack, value),
            DST_EG1_DECAY => set_first(&mut self.eg1_decay, value),
            DST_EG1_RELEASE => set_first(&mut self.eg1_release, value),
            DST_EG1_SUSTAIN => set_first(&mut self.eg1_sustain, value),
            DST_EG2_ATTACK => set_first(&mut self.eg2_attack, value),
            DST_EG2_DECAY => set_first(&mut self.eg2_decay, value),
            DST_EG2_RELEASE => set_first(&mut self.eg2_release, value),
            DST_EG2_SUSTAIN => set_first(&mut self.eg2_sustain, value),
            DST_LFO_FREQUENCY => set_first(&mut self.lfo_frequency, value),
            DST_LFO_DELAY => set_first(&mut self.lfo_delay, value),
            DST_VIB_FREQUENCY => set_first(&mut self.vib_frequency, value),
            DST_VIB_DELAY => set_first(&mut self.vib_delay, value),
            DST_FILTER_CUTOFF => set_first(&mut self.cutoff, value),
            DST_FILTER_Q => set_first(&mut self.resonance, value),
            DST_PITCH => set_first(&mut self.pitch, value),
            DST_ATTENUATION => set_first(&mut self.attenuation, value),
            DST_PAN => set_first(&mut self.pan, value),
            _ => {}
        }
    }

    /// A block with a source: a static modulator (key number, velocity) is folded in now; an LFO,
    /// vibrato or EG2 route is kept for the voice. A static modulator takes no control.
    fn routed(
        &mut self,
        source: u16,
        control: u16,
        destination: u16,
        scale: f64,
        key: u8,
        velocity: u8,
    ) {
        let control = match control {
            CONTROL_NONE => Control::None,
            CONTROL_MOD_WHEEL => Control::ModWheel,
            CONTROL_CHANNEL_PRESSURE => Control::ChannelPressure,
            _ => return,
        };
        let key_value = f64::from(key) - MIDDLE_C;
        let velocity_value = f64::from(velocity.min(127)) / 127.0;
        let fixed = control == Control::None;
        match source {
            SRC_KEY_NUMBER if fixed => self.static_route(destination, scale * key_value),
            SRC_KEY_ON_VELOCITY if fixed => {
                let value = if destination == DST_ATTENUATION {
                    1.0 - velocity_value
                } else {
                    velocity_value
                };
                self.static_route(destination, scale * value);
            }
            SRC_LFO => {
                self.lfo_routed = true;
                self.dynamic(Source::Lfo, control, destination, scale);
            }
            SRC_VIBRATO => {
                self.vib_routed = true;
                self.dynamic(Source::Vibrato, control, destination, scale);
            }
            SRC_EG2 => self.dynamic(Source::Eg2, control, destination, scale),
            _ => {}
        }
    }

    /// A static route's value, in its destination's unit.
    fn static_route(&mut self, destination: u16, value: f64) {
        match destination {
            DST_PITCH => self.static_pitch += value,
            DST_ATTENUATION => self.static_attenuation += value,
            DST_PAN => self.static_pan += value,
            DST_FILTER_CUTOFF => self.static_cutoff += value,
            DST_FILTER_Q => self.static_resonance += value,
            DST_EG1_ATTACK => self.eg1_time_delta[0] += value,
            DST_EG1_DECAY => self.eg1_time_delta[1] += value,
            DST_EG1_RELEASE => self.eg1_time_delta[2] += value,
            DST_EG1_SUSTAIN => self.eg1_sustain_delta += value,
            DST_EG2_ATTACK => self.eg2_time_delta[0] += value,
            DST_EG2_DECAY => self.eg2_time_delta[1] += value,
            DST_EG2_RELEASE => self.eg2_time_delta[2] += value,
            DST_EG2_SUSTAIN => self.eg2_sustain_delta += value,
            _ => {}
        }
    }

    fn dynamic(&mut self, source: Source, control: Control, destination: u16, scale: f64) {
        let dest = match destination {
            DST_PITCH => Dest::Pitch,
            DST_ATTENUATION => Dest::Attenuation,
            DST_PAN => Dest::Pan,
            DST_FILTER_CUTOFF => {
                self.cutoff_routed = true;
                Dest::Cutoff
            }
            _ => return,
        };
        self.routes.push(Route {
            source,
            control,
            dest,
            scale,
        });
    }

    /// The envelope for one stage: the fixed time (timecents) plus the key and velocity deltas.
    fn time(base: Option<f64>, delta: f64) -> f64 {
        match base {
            None if delta == 0.0 => 0.0,
            base => seconds_from_timecents(base.unwrap_or(0.0) + delta),
        }
    }

    fn sustain(base: Option<f64>, delta: f64) -> f64 {
        (base.unwrap_or(0.0) / 1000.0 + delta / 1000.0).clamp(0.0, 1.0)
    }

    fn eg1(&self) -> EnvelopeParams {
        EnvelopeParams {
            attack: Self::time(self.eg1_attack, self.eg1_time_delta[0]),
            decay: Self::time(self.eg1_decay, self.eg1_time_delta[1]),
            sustain: match self.eg1_sustain {
                None if self.eg1_sustain_delta == 0.0 => 1.0,
                base => Self::sustain(base, self.eg1_sustain_delta),
            },
            release: Self::time(self.eg1_release, self.eg1_time_delta[2]),
            shape: Shape::Db,
        }
    }

    fn eg2(&self) -> EnvelopeParams {
        EnvelopeParams {
            attack: Self::time(self.eg2_attack, self.eg2_time_delta[0]),
            decay: Self::time(self.eg2_decay, self.eg2_time_delta[1]),
            sustain: match self.eg2_sustain {
                None if self.eg2_sustain_delta == 0.0 => 1.0,
                base => Self::sustain(base, self.eg2_sustain_delta),
            },
            release: Self::time(self.eg2_release, self.eg2_time_delta[2]),
            shape: Shape::Linear,
        }
    }

    fn lfo(&self) -> Option<LfoParams> {
        if !self.lfo_routed && self.lfo_frequency.is_none() && self.lfo_delay.is_none() {
            return None;
        }
        Some(LfoParams {
            frequency: self
                .lfo_frequency
                .map_or(super::lfo::DEFAULT_FREQUENCY_HZ, hz_from_absolute_cents),
            delay: self.lfo_delay.map_or(NO_DELAY, seconds_from_timecents),
        })
    }

    fn vibrato(&self) -> Option<LfoParams> {
        if !self.vib_routed && self.vib_frequency.is_none() && self.vib_delay.is_none() {
            return None;
        }
        Some(LfoParams {
            frequency: self
                .vib_frequency
                .map_or(super::lfo::DEFAULT_FREQUENCY_HZ, hz_from_absolute_cents),
            delay: self.vib_delay.map_or(NO_DELAY, seconds_from_timecents),
        })
    }

    /// The filter: the cutoff's fixed and static parts (an open maximum when none is set, so a
    /// route alone moves an open filter), and the resonance. `None` when no block names a cutoff.
    fn filter(&self) -> Option<FilterParams> {
        if self.cutoff.is_none() && self.static_cutoff == 0.0 && !self.cutoff_routed {
            return None;
        }
        Some(FilterParams {
            cutoff_cents: self.cutoff.unwrap_or(scaled(i32::MAX)) + self.static_cutoff,
            q: resonance_q(self.resonance.unwrap_or(0.0) + self.static_resonance),
        })
    }
}

/// The pan from a 0.1-percent value: ±500 is the full range, so the position is ±1 at ±500.
fn pan_position(tenths_of_percent: f64) -> f64 {
    (tenths_of_percent / 500.0).clamp(-1.0, 1.0)
}

/// The modulation of a note on the region's and its instrument's connection blocks, for a note
/// at `key` and `velocity` (1 to 127). `articulations` lists the region's first, then the
/// instrument's. The level-2 (`art2`) blocks are read before the level-1 ones, so they override
/// them for a fixed destination; routes from both levels apply.
pub fn resolve<'a>(
    articulations: impl IntoIterator<Item = &'a Articulation>,
    key: u8,
    velocity: u8,
) -> NoteModulation {
    let arts: Vec<&Articulation> = articulations.into_iter().collect();
    let mut setting = Setting::default();
    for level2 in [true, false] {
        for art in arts.iter().filter(|a| a.level2 == level2) {
            setting.read(art, key, velocity);
        }
    }
    NoteModulation {
        eg1: setting.eg1(),
        eg2: setting.eg2(),
        lfo: setting.lfo(),
        vibrato: setting.vibrato(),
        filter: setting.filter(),
        pitch_cents: setting.pitch.unwrap_or(0.0) + setting.static_pitch,
        attenuation_db: setting.attenuation.unwrap_or(0.0) + setting.static_attenuation,
        pan: pan_position(setting.pan.unwrap_or(0.0) + setting.static_pan),
        routes: setting.routes,
    }
}
