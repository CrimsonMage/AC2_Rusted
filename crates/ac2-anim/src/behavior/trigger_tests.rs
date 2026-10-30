//! `TriggerUpdate`: effects start once, follow their track and end with it; hooks fire once.

use ac2_core::Random;

use super::{
    Ac2AnimEvent, Ac2BehaviorParams, Ac2BehaviorTrack, Ac2TriggerAction, Ac2TriggerUpdate,
};
use crate::{Ac2AnimTrigger, Ac2WeenieHook};

fn effect(fx: u32, probability: f32, intensity_var: f32) -> Ac2AnimTrigger {
    Ac2AnimTrigger {
        fx,
        time: 0.0,
        stop: false,
        probability,
        intensity: 1.0,
        intensity_var,
        play: 1,
    }
}

fn events(actions: Vec<Ac2TriggerAction>) -> Vec<Ac2AnimEvent> {
    actions
        .into_iter()
        .filter_map(|action| match action {
            Ac2TriggerAction::Event(event) => Some(event),
            Ac2TriggerAction::Impulse(_) => None,
        })
        .collect()
}

#[test]
fn an_effect_starts_once_and_stops_when_its_track_loops() {
    let mut update = Ac2TriggerUpdate::default();
    let mut random = Random::default();
    let mut track = Ac2BehaviorTrack::new(Ac2BehaviorParams::new(1));
    track.triggers.effects.push(effect(9, 1.0, 0.0));
    update.add_track_triggers(&mut track);
    let start = Ac2AnimEvent::StartFx {
        fx: 9,
        intensity: 1.0,
        target: 0,
    };
    assert_eq!(events(update.fire(&mut random)), vec![start]);
    update.add_track_triggers(&mut track);
    assert!(
        events(update.fire(&mut random)).is_empty(),
        "already playing"
    );
    track.looped = true;
    update.add_track_triggers(&mut track);
    assert!(!track.looped, "OnTrackTerminated clears the loop");
    assert_eq!(
        events(update.fire(&mut random)),
        vec![Ac2AnimEvent::StopFx { fx: 9 }]
    );
    update.add_track_triggers(&mut track);
    assert_eq!(
        events(update.fire(&mut random)),
        vec![start],
        "the next cycle"
    );
}

#[test]
fn rolls_follow_the_shared_generator() {
    let mut update = Ac2TriggerUpdate::default();
    let mut random = Random::default();
    let mut track = Ac2BehaviorTrack::new(Ac2BehaviorParams::new(1));
    track.triggers.effects.push(effect(9, 0.5, 2.0));
    update.add_track_triggers(&mut track);
    let mut expected = Random::default();
    let played = expected.roll_dice_f32(0.0, 1.0) <= 0.5;
    let spread = (f64::from(expected.roll_dice_f32(0.0, 1.0)) - 0.5) * 2.0;
    let intensity = (spread + spread + 1.0) as f32;
    let fired = events(update.fire(&mut random));
    let want = if played {
        vec![Ac2AnimEvent::StartFx {
            fx: 9,
            intensity,
            target: 0,
        }]
    } else {
        Vec::new()
    };
    assert_eq!(fired, want);
    assert_eq!(random, expected, "two rolls");
}

#[test]
fn a_hook_fires_once_unless_packed_triggered() {
    let mut update = Ac2TriggerUpdate::default();
    let mut track = Ac2BehaviorTrack::new(Ac2BehaviorParams::new(1));
    let hook = Ac2WeenieHook {
        hook_type: 3,
        hook_num: 4,
        data: 5,
        triggered: false,
        time: 0.0,
    };
    track.triggers.hooks.push(hook);
    track.triggers.hooks.push(Ac2WeenieHook {
        data: 6,
        triggered: true,
        ..hook
    });
    let mut random = Random::default();
    update.add_track_triggers(&mut track);
    assert_eq!(
        events(update.fire(&mut random)),
        vec![Ac2AnimEvent::WeenieHook {
            hook_type: 3,
            hook_num: 4,
            data: 5
        }]
    );
    update.add_track_triggers(&mut track);
    assert!(events(update.fire(&mut random)).is_empty());
}

#[test]
fn fire_and_forget_effects_fire_at_the_next_update() {
    let mut update = Ac2TriggerUpdate::default();
    update.fire_and_forget(7, false);
    update.fire_and_forget(7, true);
    update.fire_and_forget(8, true);
    let fired = events(update.fire(&mut Random::default()));
    assert_eq!(fired.len(), 2, "one per id");
    assert!(fired.contains(&Ac2AnimEvent::StartFx {
        fx: 7,
        intensity: 1.0,
        target: 0
    }));
    assert!(fired.contains(&Ac2AnimEvent::StopFx { fx: 8 }));
    assert!(update.fire(&mut Random::default()).is_empty());
}
