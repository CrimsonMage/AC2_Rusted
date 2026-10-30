//! `TriggerUpdate`: the effects, weenie hooks and impulses the behaviour tracks reached this
//! update, and the effects playing, as `CPhysicsObj::UpdateAnimation` fires them.
//!
//! `TriggerUpdate::AddEffectsTrigger` rolls a new effect's probability and intensity variation
//! on the shared generator (`Random::RollDice(0.0, 1.0)`) when it adds it; this port records the
//! rolls in that order and makes them when the triggers fire (the same update for every add
//! made during `ModeTracker::Update`).

use ac2_core::Random;

use super::hash::RetailHash;
use super::Ac2AnimEvent;
use crate::dbanimator::Ac2TrackTriggers;
use crate::{Ac2AnimTrigger, Ac2ImpulseTrigger, Ac2WeenieHook};

/// What firing the triggers does, in retail order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ac2TriggerAction {
    /// `DoFX`, `StopFX` or the weenie hook.
    Event(Ac2AnimEvent),
    /// An impulse trigger (`Jump`, or leaving the ground at its velocity).
    Impulse(Ac2ImpulseTrigger),
}

/// An `initEffects` entry and the add that made it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct InitEffect {
    trigger: Ac2AnimTrigger,
    add: u64,
}

/// The rolls one `AddEffectsTrigger` add makes.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Roll {
    add: u64,
    probability: f32,
    intensity: f32,
    intensity_var: f32,
}

/// `TriggerUpdate`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ac2TriggerUpdate {
    init_effects: RetailHash<InitEffect>,
    active_effects: RetailHash<Ac2AnimTrigger>,
    kill_effects: RetailHash<Ac2AnimTrigger>,
    fire_and_forget: RetailHash<Ac2AnimTrigger>,
    hooks: Vec<Ac2WeenieHook>,
    impulses: Vec<Ac2ImpulseTrigger>,
    rolls: Vec<Roll>,
    adds: u64,
}

/// `Trigger::Trigger(name, time, intensity, play)` and the other constructors' defaults.
fn trigger(fx: u32, time: f32, intensity: f32, play: u32) -> Ac2AnimTrigger {
    Ac2AnimTrigger {
        fx,
        time,
        stop: false,
        probability: 1.0,
        intensity,
        intensity_var: 0.0,
        play,
    }
}

impl Ac2TriggerUpdate {
    /// `HashTable::add` into `fireAndForget`: an effect (`DoFX(fx, 1)`) or, with `stop`, its
    /// end (`StopFX`) fired at the next update. Refused while one with that id waits.
    pub(crate) fn fire_and_forget(&mut self, fx: u32, stop: bool) {
        let mut effect = trigger(fx, 0.0, 1.0, 1);
        effect.stop = stop;
        self.fire_and_forget.add(fx, effect);
    }

    /// `TriggerUpdate::AddTrackTriggers`.
    pub(crate) fn add_track_triggers(&mut self, track: &mut super::Ac2BehaviorTrack) {
        if !track.triggers.effects.is_empty() {
            self.add_effects_trigger(track);
        }
        // `AddWeenieHook` and `AddImpulseTrigger` queue the untriggered ones; they fire (and
        // are marked) in the same `UpdateAnimation`, so they are marked here.
        for hook in track.triggers.hooks.iter_mut().filter(|h| !h.triggered) {
            self.hooks.push(*hook);
            hook.triggered = true;
        }
        for (impulse, triggered) in track.triggers.impulses.iter_mut() {
            if !*triggered {
                self.impulses.push(*impulse);
                *triggered = true;
            }
        }
    }

    /// `TriggerUpdate::AddEffectsTrigger`: a track ramping down or past a cycle ends its effects
    /// (`OnTrackTerminated`); otherwise each effect already playing takes the track's time and
    /// intensity, and each other one is added to start (with its probability and intensity
    /// variation rolled) and counts as the track's.
    fn add_effects_trigger(&mut self, track: &mut super::Ac2BehaviorTrack) {
        if track.is_ramping_down || track.looped {
            self.on_track_terminated(track);
            return;
        }
        for effect in &track.triggers.effects {
            if let Some(playing) = self.active_effects.get_mut(effect.fx) {
                playing.time = effect.time;
                playing.intensity = effect.intensity;
                continue;
            }
            self.adds += 1;
            let add = self.adds;
            self.init_effects.add(
                effect.fx,
                InitEffect {
                    trigger: trigger(effect.fx, effect.time, effect.intensity, 1),
                    add,
                },
            );
            self.rolls.push(Roll {
                add,
                probability: effect.probability,
                intensity: effect.intensity,
                intensity_var: effect.intensity_var,
            });
            track.triggers.triggered_effects.insert(effect.fx);
        }
    }

    /// `TriggerUpdate::OnTrackTerminated`: every effect the track started leaves the start list,
    /// a playing one is stopped, and the track's loop flag clears.
    pub(crate) fn on_track_terminated(&mut self, track: &mut super::Ac2BehaviorTrack) {
        terminate(self, &track.triggers);
        track.looped = false;
    }

    /// The rolls of `AddEffectsTrigger`, in add order: the probability
    /// (`Random::RollDice(0, 1) <= probability`, no roll at exactly 1) and the intensity
    /// `(roll - 0.5) * var * 2 + intensity`.
    fn roll(&mut self, random: &mut Random) {
        for roll in std::mem::take(&mut self.rolls) {
            let play =
                if roll.probability == 1.0 || random.roll_dice_f32(0.0, 1.0) <= roll.probability {
                    1
                } else {
                    0
                };
            let spread = (f64::from(random.roll_dice_f32(0.0, 1.0)) - f64::from(0.5_f32))
                * f64::from(roll.intensity_var);
            let intensity = (spread + spread + f64::from(roll.intensity)) as f32;
            for key in self.init_effects.keys() {
                if let Some(entry) = self.init_effects.get_mut(key) {
                    if entry.add == roll.add {
                        entry.trigger.play = play;
                        entry.trigger.intensity = intensity;
                    }
                }
            }
        }
    }

    /// `TriggerUpdate::IsEmpty`.
    fn is_empty(&self) -> bool {
        self.init_effects.iter().next().is_none()
            && self.active_effects.iter().next().is_none()
            && self.kill_effects.iter().next().is_none()
            && self.fire_and_forget.iter().next().is_none()
            && self.hooks.is_empty()
            && self.impulses.is_empty()
    }

    /// The trigger half of `CPhysicsObj::UpdateAnimation`, then `TriggerUpdate::CleanUp`: the
    /// fire-and-forget effects, the effects starting (those whose probability played), the
    /// weenie hooks, the impulses, the effects ending; the started effects then count as
    /// playing.
    pub fn fire(&mut self, random: &mut Random) -> Vec<Ac2TriggerAction> {
        self.roll(random);
        if self.is_empty() {
            return Vec::new();
        }
        let fx = |effect: &Ac2AnimTrigger| {
            Ac2TriggerAction::Event(if effect.stop {
                Ac2AnimEvent::StopFx { fx: effect.fx }
            } else {
                Ac2AnimEvent::StartFx {
                    fx: effect.fx,
                    intensity: effect.intensity,
                    target: 0,
                }
            })
        };
        let mut actions: Vec<Ac2TriggerAction> =
            self.fire_and_forget.iter().map(|(_, e)| fx(e)).collect();
        self.fire_and_forget = RetailHash::default();
        actions.extend(
            self.init_effects
                .iter()
                .filter(|(_, entry)| entry.trigger.play != 0)
                .map(|(_, entry)| fx(&entry.trigger)),
        );
        actions.extend(self.hooks.drain(..).map(|hook| {
            Ac2TriggerAction::Event(Ac2AnimEvent::WeenieHook {
                hook_type: hook.hook_type,
                hook_num: hook.hook_num,
                data: hook.data,
            })
        }));
        actions.extend(self.impulses.drain(..).map(Ac2TriggerAction::Impulse));
        actions.extend(
            self.kill_effects
                .iter()
                .map(|(_, e)| Ac2TriggerAction::Event(Ac2AnimEvent::StopFx { fx: e.fx })),
        );
        let started: Vec<Ac2AnimTrigger> =
            self.init_effects.iter().map(|(_, e)| e.trigger).collect();
        for effect in started {
            self.active_effects.add(effect.fx, effect);
        }
        self.init_effects = RetailHash::default();
        self.kill_effects = RetailHash::default();
        actions
    }
}

/// The table work of `OnTrackTerminated`.
fn terminate(update: &mut Ac2TriggerUpdate, triggers: &Ac2TrackTriggers) {
    for &fx in &triggers.triggered_effects {
        update.init_effects.remove(fx);
        if update.active_effects.remove(fx).is_some() {
            update.kill_effects.add(fx, trigger(fx, 0.0, 0.0, 1));
        }
    }
}
