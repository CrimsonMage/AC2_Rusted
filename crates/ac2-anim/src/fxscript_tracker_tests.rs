//! `FXScriptTracker` timelines on synthetic scripts: starts, once-only triggers, the rest at -4,
//! the impulse, and the drop of a track that cannot resolve.

use std::sync::Arc;

use ac2_core::Random;

use super::*;
use crate::behavior::{Ac2AnimEvent, Ac2TriggerAction};
use crate::{Ac2AnimTrigger, Ac2FxScript, Ac2ImpulseTrigger};

const FX: u32 = 0x3000_0007;

fn effect(fx: u32, time: f32) -> Ac2AnimTrigger {
    Ac2AnimTrigger {
        fx,
        time,
        stop: false,
        probability: 1.0,
        intensity: 1.0,
        intensity_var: 0.0,
        play: 1,
    }
}

/// A script of period 2 seconds: an effect at 0.5 periods and a jump at 1.0 periods.
fn script() -> Arc<Ac2FxScript> {
    Arc::new(Ac2FxScript {
        effects: vec![effect(FX, 0.5)],
        impulses: vec![Ac2ImpulseTrigger {
            jump: true,
            impulse: [0.0; 3],
            num: 3,
            time: 1.0,
        }],
        period: 2.0,
    })
}

/// One update's actions after `quantum` seconds of the timeline.
fn step(
    tracker: &mut Ac2FxScriptTracker,
    update: &mut Ac2TriggerUpdate,
    quantum: f32,
) -> Vec<Ac2TriggerAction> {
    tracker.update(quantum, update);
    update.fire(&mut Random::default())
}

fn starts(actions: &[Ac2TriggerAction]) -> Vec<u32> {
    actions
        .iter()
        .filter_map(|action| match action {
            Ac2TriggerAction::Event(Ac2AnimEvent::StartFx { fx, .. }) => Some(*fx),
            _ => None,
        })
        .collect()
}

#[test]
fn a_track_waits_for_its_script_and_is_dropped_when_none_resolves() {
    let mut tracker = Ac2FxScriptTracker::default();
    tracker.do_fx_script(FX);
    let mut update = Ac2TriggerUpdate::default();
    assert!(step(&mut tracker, &mut update, 0.5).is_empty());
    assert_eq!(tracker.len(), 1, "unresolved tracks are kept");
    tracker.resolve(|_| None);
    assert!(tracker.is_empty(), "a missing record drops the track");
}

#[test]
fn the_effect_starts_once_its_time_is_reached_and_not_again() {
    let mut tracker = Ac2FxScriptTracker::default();
    tracker.do_fx_script(0x5700_0001);
    tracker.resolve(|_| Some(script()));
    let mut update = Ac2TriggerUpdate::default();
    // 0.5 seconds is a quarter of the 2 second period: the effect (0.5 periods) is not yet due.
    assert!(starts(&step(&mut tracker, &mut update, 0.5)).is_empty());
    // Half a period: the effect starts.
    assert_eq!(starts(&step(&mut tracker, &mut update, 1.0)), vec![FX]);
    // The effect keeps playing; a later update does not start it again.
    assert!(starts(&step(&mut tracker, &mut update, 0.2)).is_empty());
}

#[test]
fn an_impulse_fires_once_at_its_time() {
    let mut tracker = Ac2FxScriptTracker::default();
    tracker.do_fx_script(0x5700_0001);
    tracker.resolve(|_| Some(script()));
    let mut update = Ac2TriggerUpdate::default();
    let mut jumps = 0;
    for _ in 0..8 {
        jumps += step(&mut tracker, &mut update, 0.5)
            .iter()
            .filter(|a| matches!(a, Ac2TriggerAction::Impulse(i) if i.jump))
            .count();
    }
    assert_eq!(jumps, 1);
}

#[test]
fn a_timeline_past_one_period_rests_at_minus_four_and_keeps_running() {
    let mut tracker = Ac2FxScriptTracker::default();
    tracker.do_fx_script(0x5700_0001);
    tracker.resolve(|_| Some(script()));
    let mut update = Ac2TriggerUpdate::default();
    // One whole period reaches 1, so the time rests at -4.
    step(&mut tracker, &mut update, 2.0);
    assert_eq!(tracker.len(), 1);
    assert_eq!(tracker.tracks[0].track.ctime, REST_TIME);
    // Four more periods of rest bring it back to 0 and the timeline runs again.
    for _ in 0..4 {
        step(&mut tracker, &mut update, 2.0);
    }
    assert_eq!(tracker.tracks[0].track.ctime, 0.0);
    assert_eq!(tracker.len(), 1);
}

#[test]
fn a_track_with_a_nan_time_ends() {
    let mut tracker = Ac2FxScriptTracker::default();
    tracker.do_fx_script(0x5700_0001);
    tracker.resolve(|_| Some(script()));
    let mut update = Ac2TriggerUpdate::default();
    step(&mut tracker, &mut update, f32::NAN);
    assert!(tracker.is_empty());
}

#[test]
fn the_script_ids_are_kept_in_start_order() {
    let mut tracker = Ac2FxScriptTracker::default();
    tracker.do_fx_script(0x5700_0002);
    tracker.do_fx_script(0x5700_0001);
    tracker.resolve(|_| Some(script()));
    assert_eq!(
        tracker.script_ids().collect::<Vec<_>>(),
        vec![0x5700_0002, 0x5700_0001]
    );
}
