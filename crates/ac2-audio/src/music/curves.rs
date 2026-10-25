//! Curve items (`DMUS_IO_CURVE_ITEM`, the `curl` chunk of a sequence track) as controller events
//! over time.
//!
//! What the port reads, from the item's fields (`DMUS_CURVET_*`, `DMUS_CURVES_*`, `DMUS_CURVE_*`
//! as the segment's own track gives them; the corpus is the reference):
//!
//! - The curve starts at `start + offset` ticks and runs for `duration` ticks, from `start_value`
//!   to `end_value`. `kind` 3 is a pitch bend (14-bit, 8192 centred), 4 a control change on
//!   `cc`, and 5 channel pressure. Kinds 6 (poly aftertouch) and the RPN/NRPN kinds are not
//!   emitted: their note and parameter fields are not read here.
//! - `shape` 0 is linear, 2 exponential (`u^2`), 3 logarithmic (`sqrt(u)`), 4 sine (`(1 - cos(pi
//!   u)) / 2`), all as the port's own reading of the names; 1 (instant) sends the end value at
//!   the start. The shape curves are assumptions, not checked against retail.
//! - `reset_duration` with the reset flag (`DMUS_CURVE_RESET`, bit 0 of `flags`) ramps the value
//!   linearly from the end value to `reset_value` over the reset duration, after the curve. The
//!   corpus's pitch-bend curves reset to centre, which is where they already are.
//!
//! An event goes out at each tick where the rounded value changes, so a ramp is one event per
//! step of the controller's range, not per tick. A curve with no duration sends its end value at
//! its start.

use super::performance::MusicEvent;
use super::segment::CurveItem;

/// `DMUS_CURVET_PBCURVE`.
const KIND_PITCH_BEND: u8 = 3;
/// `DMUS_CURVET_CCCURVE`.
const KIND_CONTROL: u8 = 4;
/// `DMUS_CURVET_MATCURVE`.
const KIND_CHANNEL_PRESSURE: u8 = 5;
/// `DMUS_CURVES_INSTANT`.
const SHAPE_INSTANT: u8 = 1;
/// `DMUS_CURVE_RESET`.
const FLAG_RESET: u8 = 1;
/// The largest 14-bit pitch bend value (8192 is centre).
const BEND_MAX: i32 = 16_383;

/// The position `u` (0 to 1) of a curve has reached in its shape: 0 to 1.
fn shaped(shape: u8, u: f64) -> f64 {
    match shape {
        2 => u * u,
        3 => u.sqrt(),
        4 => (1.0 - (std::f64::consts::PI * u).cos()) / 2.0,
        _ => u,
    }
}

/// The event for `value` on the curve's controller, or `None` for a kind not emitted.
fn event_for(item: &CurveItem, value: i32) -> Option<MusicEvent> {
    let channel = item.pchannel;
    match item.kind {
        KIND_PITCH_BEND => Some(MusicEvent::PitchBend {
            channel,
            value: value.clamp(0, BEND_MAX) as u16,
        }),
        KIND_CONTROL => Some(MusicEvent::Control {
            channel,
            controller: item.cc & 0x7F,
            value: value.clamp(0, 127) as u8,
        }),
        KIND_CHANNEL_PRESSURE => Some(MusicEvent::ChannelPressure {
            channel,
            value: value.clamp(0, 127) as u8,
        }),
        _ => None,
    }
}

/// Samples a ramp from `from` to `to` over `duration` ticks at `start`, shaped by `shape`, and
/// returns an event at each tick where the rounded value changes (the first tick always).
fn ramp(
    item: &CurveItem,
    start: i32,
    duration: i32,
    from: f64,
    to: f64,
    shape: u8,
    out: &mut Vec<(i32, MusicEvent)>,
) {
    let mut last: Option<i32> = None;
    for k in 0..=duration {
        let u = if duration == 0 {
            1.0
        } else {
            f64::from(k) / f64::from(duration)
        };
        let value = (from + (to - from) * shaped(shape, u)).round() as i32;
        if last == Some(value) {
            continue;
        }
        last = Some(value);
        if let Some(event) = event_for(item, value) {
            out.push((start + k, event));
        }
    }
}

/// The controller events of one curve item, as `(tick, event)` in time order.
pub fn curve_events(item: &CurveItem) -> Vec<(i32, MusicEvent)> {
    let mut out = Vec::new();
    let start = item.start + i32::from(item.offset);
    let (from, to) = (f64::from(item.start_value), f64::from(item.end_value));
    let duration = item.duration.max(0);
    if item.shape == SHAPE_INSTANT || duration == 0 {
        if let Some(event) = event_for(item, item.end_value.into()) {
            out.push((start, event));
        }
    } else {
        ramp(item, start, duration, from, to, item.shape, &mut out);
    }
    if item.flags & FLAG_RESET != 0 && item.reset_duration > 0 {
        let after = start + duration;
        ramp(
            item,
            after,
            item.reset_duration,
            to,
            f64::from(item.reset_value),
            0,
            &mut out,
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(kind: u8, shape: u8, start: i16, end: i16, duration: i32) -> CurveItem {
        CurveItem {
            start: 100,
            duration,
            reset_duration: 0,
            pchannel: 3,
            offset: 0,
            start_value: start,
            end_value: end,
            reset_value: 0,
            kind,
            shape,
            cc: 7,
            flags: 0,
        }
    }

    #[test]
    fn a_linear_cc_ramp_sends_one_event_per_value_step() {
        let events = curve_events(&curve(KIND_CONTROL, 0, 0, 4, 40));
        let got: Vec<(i32, u8)> = events
            .iter()
            .map(|(t, e)| match e {
                MusicEvent::Control { value, .. } => (*t, *value),
                other => panic!("not a control: {other:?}"),
            })
            .collect();
        // Each step of the value is reached half a step (5 ticks) after its own tick.
        assert_eq!(got, vec![(100, 0), (105, 1), (115, 2), (125, 3), (135, 4)]);
    }

    #[test]
    fn an_instant_curve_sends_its_end_value_at_its_start() {
        let events = curve_events(&curve(KIND_CONTROL, SHAPE_INSTANT, 12, 64, 1));
        assert_eq!(
            events,
            vec![(
                100,
                MusicEvent::Control {
                    channel: 3,
                    controller: 7,
                    value: 64,
                }
            )]
        );
    }

    #[test]
    fn a_pitch_bend_curve_is_14_bit_and_centred_on_8192() {
        let events = curve_events(&curve(KIND_PITCH_BEND, 0, 8192, 16383, 2));
        assert_eq!(
            events.first().map(|e| e.1),
            Some(MusicEvent::PitchBend {
                channel: 3,
                value: 8192,
            })
        );
        assert_eq!(
            events.last().map(|e| e.1),
            Some(MusicEvent::PitchBend {
                channel: 3,
                value: 16383,
            })
        );
    }

    #[test]
    fn a_reset_ramps_back_after_the_curve() {
        let mut item = curve(KIND_CONTROL, 0, 100, 100, 1);
        item.flags = FLAG_RESET;
        item.reset_duration = 4;
        item.reset_value = 0;
        let events = curve_events(&item);
        let ticks: Vec<i32> = events.iter().map(|e| e.0).collect();
        // The curve's own event at 100, then the reset from tick 101 to 105.
        assert_eq!(ticks.first(), Some(&100));
        assert_eq!(ticks.last(), Some(&105));
        assert_eq!(
            events.last().map(|e| e.1),
            Some(MusicEvent::Control {
                channel: 3,
                controller: 7,
                value: 0,
            })
        );
    }

    #[test]
    fn kinds_not_emitted_send_nothing() {
        assert!(curve_events(&curve(6, 0, 0, 127, 10)).is_empty());
    }

    #[test]
    fn the_sine_shape_passes_the_middle_at_half_way() {
        assert!((shaped(4, 0.5) - 0.5).abs() < 1e-12);
        assert!((shaped(2, 0.5) - 0.25).abs() < 1e-12);
    }
}
