//! `DBAnimator::GetTriggers`: the effect triggers, weenie hooks and impulse triggers an animator
//! has reached, added to its track's lists (`AnimInfo::GetTriggers`), with `AnimCube`'s
//! weighted merge of its children's effects and hooks.

use std::collections::BTreeSet;

use super::asset::{Ac2AnimInfo, Ac2AnimTrigger, Ac2ImpulseTrigger, Ac2WeenieHook};
use super::tree::{cube_weights, Ac2CubeSlider, Ac2DbAnimator, MIN_WEIGHT};
use super::{Ac2AnimatorContext, Ac2TrackerState};
use crate::behavior::hash::RetailHash;

/// An `AnimTrack`'s trigger lists.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ac2TrackTriggers {
    /// `mEffect`: the effect triggers reached (a plain behaviour clears them each cycle).
    pub effects: Vec<Ac2AnimTrigger>,
    /// `mHooks`, each with its `mTriggered`.
    pub hooks: Vec<Ac2WeenieHook>,
    /// `mImpulses`, each with its `mTriggered`.
    pub impulses: Vec<(Ac2ImpulseTrigger, bool)>,
    /// `mTriggeredEffects`: the effects the track started (a `std::set`).
    pub triggered_effects: BTreeSet<u32>,
}

/// `Trigger::operator==`: name, time and probability.
fn same_trigger(a: &Ac2AnimTrigger, b: &Ac2AnimTrigger) -> bool {
    a.fx == b.fx && a.time == b.time && a.probability == b.probability
}

/// `WeenieHook::operator==`: type, data and time.
fn same_hook(a: &Ac2WeenieHook, b: &Ac2WeenieHook) -> bool {
    a.hook_type == b.hook_type && a.data == b.data && a.time == b.time
}

/// `ImpulseTrigger::operator==`: the jump flag, the impulse within 0.0002 per axis, the time.
fn same_impulse(a: &Ac2ImpulseTrigger, b: &Ac2ImpulseTrigger) -> bool {
    let near = |x: f32, y: f32| (f64::from(x) - f64::from(y)).abs() < f64::from(MIN_WEIGHT);
    a.jump == b.jump
        && near(a.impulse[0], b.impulse[0])
        && near(a.impulse[1], b.impulse[1])
        && near(a.impulse[2], b.impulse[2])
        && a.time == b.time
}

impl Ac2AnimInfo {
    /// `AnimInfo::GetTriggers`: every trigger at or before `ctime` the track does not hold yet.
    fn add_triggers(&self, ctime: f32, out: &mut Ac2TrackTriggers) {
        for effect in self.effects.iter().filter(|e| e.time <= ctime) {
            if !out.effects.iter().any(|known| same_trigger(known, effect)) {
                out.effects.push(*effect);
            }
        }
        for hook in self.hooks.iter().filter(|h| h.time <= ctime) {
            if !out.hooks.iter().any(|known| same_hook(known, hook)) {
                out.hooks.push(*hook);
            }
        }
        for impulse in self.impulses.iter().filter(|i| i.time <= ctime) {
            if !out
                .impulses
                .iter()
                .any(|(known, _)| same_impulse(known, impulse))
            {
                out.impulses.push((*impulse, false));
            }
        }
    }
}

impl Ac2DbAnimator {
    /// `DBAnimator::GetTriggers`: add what the animator has reached by clip time `ctime` (its
    /// fraction when `normalize` and past one cycle) to `out`. `AMHuman` and `AMSwitchME` ask
    /// their animator with the time normalised.
    pub fn triggers(
        &self,
        ctx: &dyn Ac2AnimatorContext,
        tracker: Ac2TrackerState,
        ctime: f32,
        normalize: bool,
        out: &mut Ac2TrackTriggers,
    ) {
        match self {
            Self::Asset(asset) => {
                let Some(info) = asset.info.as_ref() else {
                    return;
                };
                let mut time = ctime;
                if normalize && 1.0 < time {
                    time = f64::from(time).fract() as f32;
                }
                info.add_triggers(time, out);
            }
            Self::Cube { sliders, children } => {
                cube_triggers(sliders, children, ctx, tracker, ctime, out)
            }
            Self::Human(child) => {
                if let Some(child) = child {
                    child.triggers(ctx, tracker, ctime, true, out);
                }
            }
            Self::Switch { .. } => {
                if let Some(child) = self.switch_target(ctx, tracker) {
                    child.triggers(ctx, tracker, ctime, true, out);
                }
            }
            Self::Other => {}
        }
    }
}

/// `AnimCube::GetTriggers`: each weighted child adds everything in its cycle to the track; the
/// track's effects (keyed by name) and hooks (keyed by data) are merged into weighted sums of
/// their times, intensities and probabilities, and the lists are cleared after each child.
/// Impulses stay as added. The merged effects and hooks before `ctime` (within the cycle) are
/// then the track's, and the merged effects count as started by it.
fn cube_triggers(
    sliders: &[Ac2CubeSlider],
    children: &[Option<std::sync::Arc<Ac2DbAnimator>>],
    ctx: &dyn Ac2AnimatorContext,
    tracker: Ac2TrackerState,
    ctime: f32,
    out: &mut Ac2TrackTriggers,
) {
    let mut ctime = ctime;
    if 1.0 < ctime {
        ctime = f64::from(ctime).fract() as f32;
    }
    let mut effects: RetailHash<Ac2AnimTrigger> = RetailHash::default();
    let mut hooks: RetailHash<Ac2WeenieHook> = RetailHash::default();
    let sum = |w: f32, value: f32, total: f32| {
        (f64::from(w) * f64::from(value) + f64::from(total)) as f32
    };
    let scaled = |w: f32, value: f32| (f64::from(value) * f64::from(w)) as f32;
    for (w, child) in cube_weights(sliders, children.len(), ctx)
        .into_iter()
        .zip(children.iter())
    {
        let (true, Some(child)) = (w >= MIN_WEIGHT, child) else {
            continue;
        };
        child.triggers(ctx, tracker, 1.0, false, out);
        for effect in &out.effects {
            match effects.get_mut(effect.fx) {
                Some(total) => {
                    total.time = sum(w, effect.time, total.time);
                    total.intensity = sum(w, effect.intensity, total.intensity);
                    total.intensity_var = sum(w, effect.intensity_var, total.intensity_var);
                    total.probability = sum(w, effect.probability, total.probability);
                }
                None => {
                    effects.add(
                        effect.fx,
                        Ac2AnimTrigger {
                            fx: effect.fx,
                            time: scaled(w, effect.time),
                            stop: false,
                            probability: scaled(w, effect.probability),
                            intensity: scaled(w, effect.intensity),
                            intensity_var: scaled(w, effect.intensity_var),
                            play: 1,
                        },
                    );
                }
            }
        }
        for hook in &out.hooks {
            match hooks.get_mut(hook.data) {
                Some(total) => total.time = sum(w, hook.time, total.time),
                None => {
                    hooks.add(
                        hook.data,
                        Ac2WeenieHook {
                            time: scaled(w, hook.time),
                            ..*hook
                        },
                    );
                }
            }
        }
        out.effects.clear();
        out.hooks.clear();
    }
    for (_, effect) in effects.iter() {
        if effect.time < ctime {
            out.effects.push(*effect);
            out.triggered_effects.insert(effect.fx);
        }
    }
    for (_, hook) in hooks.iter() {
        if hook.time < ctime {
            out.hooks.push(*hook);
        }
    }
}
