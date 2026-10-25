//! The controllers a synthesizer keeps for each performance channel: volume (CC7), expression
//! (CC11), pan (CC10), the mod wheel (CC1), the sustain pedal (CC64), the pitch bend and its range
//! (the RPN 0 pitch-bend sensitivity, CC101/CC100 select and CC6/CC38 set it), and the channel
//! pressure.
//!
//! Defaults follow the MIDI ones (volume 100, expression 127, pan 64, mod wheel 0, bend centred,
//! a bend range of 2 semitones). Gains are read relative to their defaults, so an untouched
//! channel plays at unity: volume is `CC7 / 100`, expression `CC11 / 127`, and pan runs from
//! -1 (CC10 0) to +1 (CC10 127) with 64 centred. These gain laws are the port's assumption (the
//! MIDI curves are logarithmic in the GM spec; the retail curve was not read).

/// The default MIDI volume (CC7), which plays at unity gain.
const DEFAULT_VOLUME: u8 = 100;
/// The default expression (CC11), which plays at unity gain.
const DEFAULT_EXPRESSION: u8 = 127;
/// The centred pan (CC10).
const CENTRE_PAN: u8 = 64;
/// The pitch bend's centre (14-bit).
pub const BEND_CENTRE: u16 = 8192;
/// The pitch-bend range until an RPN sets it (semitones).
const DEFAULT_BEND_RANGE: f64 = 2.0;
/// `CC64` at or above this is the sustain pedal down.
const PEDAL_DOWN: u8 = 64;
/// The RPN that selects the pitch-bend sensitivity (MSB and LSB both 0).
const RPN_BEND_RANGE: (u8, u8) = (0, 0);
/// No RPN selected (both bytes 127).
const RPN_NULL: u8 = 127;

/// One performance channel's controller state.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelState {
    pub volume: u8,
    pub expression: u8,
    pub pan: u8,
    pub mod_wheel: u8,
    pub sustain: bool,
    pub bend: u16,
    pub pressure: u8,
    /// The pitch-bend range in semitones (RPN 0, the MSB in semitones plus the LSB in cents).
    bend_range: f64,
    rpn: (u8, u8),
    /// The RPN data entry's MSB and LSB (CC6 and CC38) for the pitch-bend range.
    data: (u8, u8),
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            volume: DEFAULT_VOLUME,
            expression: DEFAULT_EXPRESSION,
            pan: CENTRE_PAN,
            mod_wheel: 0,
            sustain: false,
            bend: BEND_CENTRE,
            pressure: 0,
            bend_range: DEFAULT_BEND_RANGE,
            rpn: (RPN_NULL, RPN_NULL),
            data: (0, 0),
        }
    }
}

impl ChannelState {
    /// Applies a control change. Returns `true` when the sustain pedal goes up (the caller
    /// releases the notes it held).
    pub fn control(&mut self, controller: u8, value: u8) -> bool {
        let value = value.min(127);
        match controller {
            1 => self.mod_wheel = value,
            6 => {
                self.data.0 = value;
                self.apply_rpn_data();
            }
            38 => {
                self.data.1 = value;
                self.apply_rpn_data();
            }
            7 => self.volume = value,
            10 => self.pan = value,
            11 => self.expression = value,
            64 => {
                let was = self.sustain;
                self.sustain = value >= PEDAL_DOWN;
                return was && !self.sustain;
            }
            100 => self.rpn.1 = value,
            101 => self.rpn.0 = value,
            _ => {}
        }
        false
    }

    /// Sets the pitch-bend range from the RPN data entry (only while RPN 0 is selected).
    fn apply_rpn_data(&mut self) {
        if self.rpn == RPN_BEND_RANGE {
            self.bend_range = f64::from(self.data.0) + f64::from(self.data.1) / 100.0;
        }
    }

    /// Sets the pitch bend (14 bits).
    pub fn set_bend(&mut self, value: u16) {
        self.bend = value.min(16_383);
    }

    /// The pitch bend in semitones from centre (`(bend - 8192) / 8192 * range`).
    pub fn bend_semitones(&self) -> f64 {
        (f64::from(self.bend) - f64::from(BEND_CENTRE)) / f64::from(BEND_CENTRE) * self.bend_range
    }

    /// The volume gain (`CC7 / 100`, unity at the default).
    pub fn volume_gain(&self) -> f64 {
        f64::from(self.volume) / f64::from(DEFAULT_VOLUME)
    }

    /// The expression gain (`CC11 / 127`, unity at the default).
    pub fn expression_gain(&self) -> f64 {
        f64::from(self.expression) / f64::from(DEFAULT_EXPRESSION)
    }

    /// The channel's pan position from -1 (left) to +1 (right).
    pub fn pan_position(&self) -> f64 {
        ((f64::from(self.pan) - f64::from(CENTRE_PAN)) / f64::from(CENTRE_PAN - 1)).clamp(-1.0, 1.0)
    }

    /// The mod wheel as a fraction (0 to 1).
    pub fn mod_wheel_fraction(&self) -> f64 {
        f64::from(self.mod_wheel) / 127.0
    }

    /// The channel pressure as a fraction (0 to 1), the depth of a `CONN_SRC_CHANNELPRESSURE`
    /// route.
    pub fn pressure_fraction(&self) -> f64 {
        f64::from(self.pressure) / 127.0
    }
}
