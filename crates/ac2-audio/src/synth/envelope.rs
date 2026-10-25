//! The DLS envelopes: delay, attack, hold, decay, sustain and release, computed in f64 per frame.
//!
//! Two shapes. The volume envelope (`EG1`) is dB-shaped: the attack rises linearly in amplitude,
//! then decay and release fall linearly in dB, at 100 dB per their time (the dB attenuation
//! grows by 100 over the time, full scale to silence). The sustain level is a fraction of full
//! scale and is held as its dB attenuation. The pitch envelope (`EG2`) is linear in its level,
//! decaying and releasing straight to the sustain level and to zero.
//!
//! The delay (`CONN_DST_EG1_DELAYTIME`, `CONN_DST_EG2_DELAYTIME`) is silence before the attack; the
//! hold (`CONN_DST_EG1_HOLDTIME`, `CONN_DST_EG2_HOLDTIME`) keeps the level at its peak (full scale,
//! 0 dB) after the attack, before the decay. Both are zero when no block sets them.
//!
//! The 100 dB-per-time rate is the port's reading (the DLS Level 1 text was not read); the times
//! come from the connection blocks (timecents, see `modulation`). A release runs from the level
//! the note has reached, so a release during the attack, the delay or the hold starts from that
//! level.

/// Full scale to silence in dB; a dB attenuation at or past it is silence.
const SILENCE_DB: f64 = 100.0;

/// The shape an envelope's decay and release take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Linear in level (the pitch envelope).
    Linear,
    /// Linear in dB attenuation (the volume envelope).
    Db,
}

/// The envelope values: times in seconds, a sustain level from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvelopeParams {
    pub delay: f64,
    pub attack: f64,
    pub hold: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
    pub shape: Shape,
}

impl EnvelopeParams {
    /// The values for a destination nothing sets: an immediate attack, no decay, full sustain
    /// and an immediate release.
    pub fn immediate(shape: Shape) -> Self {
        Self {
            delay: 0.0,
            attack: 0.0,
            hold: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.0,
            shape,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// Silent for `delay` seconds.
    Delay,
    Attack,
    /// At full scale for `hold` seconds.
    Hold,
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
    /// The attack's level (linear, 0 to 1).
    level: f64,
    /// The dB shape: the attenuation below full scale past the attack (0 at the peak). The linear
    /// shape keeps the level its release starts from here.
    attenuation: f64,
    /// Seconds left in the delay or the hold.
    timer: f64,
}

/// The dB attenuation of a linear level (silence beyond the floor).
fn db_of(level: f64) -> f64 {
    if level <= 0.0 {
        SILENCE_DB
    } else {
        (-20.0 * level.log10()).min(SILENCE_DB)
    }
}

/// The linear level of a dB attenuation.
fn level_of(attenuation: f64) -> f64 {
    if attenuation >= SILENCE_DB {
        0.0
    } else {
        10f64.powf(-attenuation / 20.0)
    }
}

impl Envelope {
    pub fn new(params: EnvelopeParams) -> Self {
        let (stage, timer) = if params.delay > 0.0 {
            (Stage::Delay, params.delay)
        } else {
            (Stage::Attack, 0.0)
        };
        Self {
            params,
            stage,
            level: 0.0,
            attenuation: 0.0,
            timer,
        }
    }

    /// Note-off: the release starts from the current level.
    pub fn release(&mut self) {
        if self.stage == Stage::Done {
            return;
        }
        match (self.params.shape, self.stage) {
            (Shape::Db, Stage::Delay | Stage::Attack | Stage::Hold) => {
                self.attenuation = db_of(self.level);
            }
            (Shape::Linear, _) => self.attenuation = self.level,
            _ => {}
        }
        self.stage = Stage::Release;
    }

    /// Whether the release has ended (the voice is silent for good).
    pub fn finished(&self) -> bool {
        self.stage == Stage::Done
    }

    /// Advances by `dt` seconds and returns the level, 0 to 1 (linear amplitude).
    pub fn tick(&mut self, dt: f64) -> f64 {
        if self.stage == Stage::Delay {
            if dt < self.timer {
                self.timer -= dt;
                return 0.0;
            }
            self.timer = 0.0;
            self.stage = Stage::Attack;
        }
        match self.params.shape {
            Shape::Linear => self.tick_linear(dt),
            Shape::Db => self.tick_db(dt),
        }
    }

    /// The attack, shared by both shapes: linear in amplitude. `true` once it has reached full;
    /// then the hold (when it has one) or the decay comes next.
    fn attack(&mut self, dt: f64) -> bool {
        if self.params.attack <= 0.0 {
            self.level = 1.0;
        } else {
            self.level = (self.level + dt / self.params.attack).min(1.0);
        }
        if self.level >= 1.0 {
            self.enter_peak();
            true
        } else {
            false
        }
    }

    /// The level reached full scale: the hold starts (its time is the hold), else the decay.
    fn enter_peak(&mut self) {
        self.level = 1.0;
        self.attenuation = 0.0;
        if self.params.hold > 0.0 {
            self.timer = self.params.hold;
            self.stage = Stage::Hold;
        } else {
            self.stage = Stage::Decay;
        }
    }

    /// One step of the hold: the level stays at full scale until its time runs out.
    fn hold(&mut self, dt: f64) {
        if dt < self.timer {
            self.timer -= dt;
        } else {
            self.timer = 0.0;
            self.stage = Stage::Decay;
        }
    }

    /// One step of the dB decay: the attenuation grows by 100 dB per decay time, to the sustain.
    fn decay_db(&mut self, dt: f64) {
        let p = self.params;
        let sustain = db_of(p.sustain);
        if p.decay <= 0.0 {
            self.attenuation = sustain;
            self.stage = Stage::Sustain;
        } else {
            self.attenuation += SILENCE_DB * dt / p.decay;
            if self.attenuation >= sustain {
                self.attenuation = sustain;
                self.stage = Stage::Sustain;
            }
        }
    }

    fn tick_linear(&mut self, dt: f64) -> f64 {
        let p = self.params;
        match self.stage {
            Stage::Delay => {}
            Stage::Attack => {
                self.attack(dt);
            }
            Stage::Hold => self.hold(dt),
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
                    self.level -= self.attenuation * dt / p.release;
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

    fn tick_db(&mut self, dt: f64) -> f64 {
        match self.stage {
            Stage::Delay => {}
            // An attack that completes this tick starts its decay in the same tick (none when
            // it holds first).
            Stage::Attack => {
                if self.attack(dt) && self.stage == Stage::Decay {
                    self.decay_db(dt);
                }
            }
            Stage::Hold => self.hold(dt),
            Stage::Decay => self.decay_db(dt),
            Stage::Sustain => {}
            Stage::Release => {
                let p = self.params;
                if p.release <= 0.0 {
                    self.stage = Stage::Done;
                } else {
                    self.attenuation += SILENCE_DB * dt / p.release;
                    if self.attenuation >= SILENCE_DB {
                        self.stage = Stage::Done;
                    }
                }
            }
            Stage::Done => {}
        }
        match self.stage {
            Stage::Delay => 0.0,
            Stage::Attack | Stage::Hold => self.level,
            Stage::Done => 0.0,
            _ => level_of(self.attenuation),
        }
    }
}
