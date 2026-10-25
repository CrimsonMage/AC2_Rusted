//! The DLS volume envelope (`EG1`): attack, decay, sustain and release read from the connection
//! blocks of an articulation (`art1`), as linear segments computed in f64.
//!
//! Connection destinations used (DLS Level 1, `CONN_DST_EG1_*`): attack `0x0206`, decay
//! `0x0207`, release `0x0209`, sustain level `0x020A`. A connection counts when its source is
//! `CONN_SRC_NONE` with no control, so the value is fixed for the note. Times are 16.16
//! fixed-point timecents (2^(tc/1200) seconds). The sustain level is 16.16 fixed-point tenths of a percent (the corpus values 0 to 991 after the
//! division by 65536 fit that reading); the spec text was not available to confirm it (see the gaps
//! in `synth/mod.rs`).

use crate::music::dls::Articulation;

const DST_EG1_ATTACK: u16 = 0x0206;
const DST_EG1_DECAY: u16 = 0x0207;
const DST_EG1_RELEASE: u16 = 0x0209;
const DST_EG1_SUSTAIN: u16 = 0x020A;
/// `CONN_SRC_NONE`.
const SRC_NONE: u16 = 0x0000;

/// The four EG1 values, in seconds and a level from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvelopeParams {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
}

impl Default for EnvelopeParams {
    /// Used for any destination no articulation sets: an immediate attack, no decay, full
    /// sustain and an immediate release.
    fn default() -> Self {
        Self {
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.0,
        }
    }
}

/// Seconds for a DLS time connection's scale: 16.16 fixed-point timecents (1200 per octave from
/// one second; the scale's low 16 bits are the fraction, which the corpus needs: raw timecents
/// would give attacks of 2^47 seconds). The minimum 32-bit value is zero time.
pub fn seconds_from_timecents(scale: i32) -> f64 {
    if scale == i32::MIN {
        0.0
    } else {
        (f64::from(scale) / 65_536.0 / 1200.0).exp2()
    }
}

impl EnvelopeParams {
    /// The EG1 values the articulations set. The first articulation in `arts` that sets a
    /// destination wins, so a region's articulation is given before its instrument's.
    pub fn from_articulations<'a>(arts: impl IntoIterator<Item = &'a Articulation>) -> Self {
        let mut attack = None;
        let mut decay = None;
        let mut release = None;
        let mut sustain = None;
        for art in arts {
            for c in art
                .connections
                .iter()
                .filter(|c| c.source == SRC_NONE && c.control == 0)
            {
                match c.destination {
                    DST_EG1_ATTACK => attack = attack.or(Some(c.scale)),
                    DST_EG1_DECAY => decay = decay.or(Some(c.scale)),
                    DST_EG1_RELEASE => release = release.or(Some(c.scale)),
                    DST_EG1_SUSTAIN => sustain = sustain.or(Some(c.scale)),
                    _ => {}
                }
            }
        }
        let defaults = Self::default();
        Self {
            attack: attack.map_or(defaults.attack, seconds_from_timecents),
            decay: decay.map_or(defaults.decay, seconds_from_timecents),
            release: release.map_or(defaults.release, seconds_from_timecents),
            sustain: sustain.map_or(defaults.sustain, |v| {
                (f64::from(v) / 65_536.0 / 1000.0).clamp(0.0, 1.0)
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

/// The envelope of one voice: [`tick`](Self::tick) advances it and returns its level.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    params: EnvelopeParams,
    stage: Stage,
    level: f64,
    release_from: f64,
}

impl Envelope {
    pub fn new(params: EnvelopeParams) -> Self {
        Self {
            params,
            stage: Stage::Attack,
            level: 0.0,
            release_from: 0.0,
        }
    }

    /// Note-off: the release starts from the current level.
    pub fn release(&mut self) {
        if self.stage != Stage::Done {
            self.release_from = self.level;
            self.stage = Stage::Release;
        }
    }

    /// Whether the release has ended (the voice is silent for good).
    pub fn finished(&self) -> bool {
        self.stage == Stage::Done
    }

    /// Advances by `dt` seconds and returns the level, 0 to 1.
    pub fn tick(&mut self, dt: f64) -> f64 {
        let p = self.params;
        match self.stage {
            Stage::Attack => {
                if p.attack <= 0.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                } else {
                    self.level += dt / p.attack;
                    if self.level >= 1.0 {
                        self.level = 1.0;
                        self.stage = Stage::Decay;
                    }
                }
            }
            Stage::Decay => {
                if p.decay <= 0.0 {
                    self.level = p.sustain;
                    self.stage = Stage::Sustain;
                } else {
                    self.level -= (1.0 - p.sustain) * dt / p.decay;
                    if self.level <= p.sustain {
                        self.level = p.sustain;
                        self.stage = Stage::Sustain;
                    }
                }
            }
            Stage::Sustain => {}
            Stage::Release => {
                if p.release <= 0.0 {
                    self.level = 0.0;
                    self.stage = Stage::Done;
                } else {
                    self.level -= self.release_from * dt / p.release;
                    if self.level <= 0.0 {
                        self.level = 0.0;
                        self.stage = Stage::Done;
                    }
                }
            }
            Stage::Done => self.level = 0.0,
        }
        self.level
    }
}
