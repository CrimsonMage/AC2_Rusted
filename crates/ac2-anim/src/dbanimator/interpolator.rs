//! `AMRandom` (`DBAnimator` type 0x40000007) and the `AnimInterpolator` it wraps: an animator
//! that plays one of a list of animators and, while a slider interpolator ramps, crossfades
//! the one it leaves into the one it enters. `AMRandom` starts a ramp to a random animator at
//! random intervals.

use std::sync::Arc;

use super::asset::Ac2AnimTrigger;
use super::tree::Ac2DbAnimator;
use super::{Ac2AnimatorContext, Ac2BoneTransform, Ac2TrackTriggers, Ac2TrackerState};
use crate::behavior::hash::RetailHash;

/// `AnimInterpolator`: the slider whose interpolator picks the animators (`mSlider`) and the
/// animators to pick from (`dbAnimatorIDs`, with the animators they name).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2Interpolator {
    pub slider: u32,
    pub ids: Arc<[u32]>,
    pub children: Arc<[Option<Arc<Ac2DbAnimator>>]>,
}

/// `AMRandom`: an interpolator that starts ramping to a random one of its animators once the
/// slider interpolator has rested for `frequency` seconds plus a roll of `variation`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2Random {
    pub interp: Ac2Interpolator,
    pub variation: f32,
    pub frequency: f32,
    pub ramp_time: f32,
    /// `AMRandom::mSlider`: the slider whose interpolator `Animate` watches and starts.
    pub slider: u32,
}

/// What `AnimInterpolator::GetAnimatorWeights` picks: the animator being left (or the one
/// resting), the one being entered, and how far the ramp has gone.
struct Picked<'a> {
    a: Option<&'a Arc<Ac2DbAnimator>>,
    a_id: u32,
    b: Option<&'a Arc<Ac2DbAnimator>>,
    b_id: u32,
    interp: f32,
}

impl Ac2Interpolator {
    /// `AnimInterpolator::GetAnimatorWeights`: `_ftol2` truncates the interpolator's start and
    /// end values to indexes; an end that names the same animator is dropped.
    fn pick(&self, ctx: &dyn Ac2AnimatorContext) -> Picked<'_> {
        let interpolator = ctx.slider_interpolator(self.slider);
        let at = |value: f32| {
            let index = f64::from(value).trunc();
            if index < 0.0 || index >= self.ids.len() as f64 {
                return (0, None);
            }
            let index = index as usize;
            (self.ids[index], self.children[index].as_ref())
        };
        let (a_id, a) = at(interpolator.start);
        if !interpolator.active {
            return Picked {
                a,
                a_id,
                b: None,
                b_id: 0,
                interp: 0.0,
            };
        }
        let (b_id, b) = at(interpolator.end);
        if a_id == b_id {
            return Picked {
                a,
                a_id,
                b: None,
                b_id: 0,
                interp: 0.0,
            };
        }
        Picked {
            a,
            a_id,
            b,
            b_id,
            interp: interpolator.current,
        }
    }

    /// `AnimInterpolator::GetPeriod`: the animators' periods weighted `1 - t` and `t`.
    pub(super) fn period(&self, ctx: &dyn Ac2AnimatorContext, tracker: Ac2TrackerState) -> f32 {
        let picked = self.pick(ctx);
        if picked.a_id == 0 {
            return 0.0;
        }
        let mut period = 0.0_f32;
        if let Some(a) = picked.a {
            period = ((1.0 - f64::from(picked.interp)) * f64::from(a.period(ctx, tracker))) as f32;
        }
        if picked.b_id != 0 {
            if let Some(b) = picked.b {
                period = (f64::from(b.period(ctx, tracker)) * f64::from(picked.interp)
                    + f64::from(period)) as f32;
            }
        }
        period
    }

    /// `AnimInterpolator::Animate`: the left animator at `(1 - t) * weight`, the entered one at
    /// `t * weight`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn animate(
        &self,
        ctx: &mut dyn Ac2AnimatorContext,
        tracker: Ac2TrackerState,
        weight: f32,
        ctime: f32,
        out: &mut [Ac2BoneTransform],
        out_weights: &mut [f32],
        roots: &[bool],
    ) {
        let picked = self.pick(&*ctx);
        if picked.a_id == 0 {
            return;
        }
        let (a, b, interp) = (picked.a.cloned(), picked.b.cloned(), picked.interp);
        let b_id = picked.b_id;
        if let Some(a) = a {
            let w = ((1.0 - f64::from(interp)) * f64::from(weight)) as f32;
            a.animate(ctx, tracker, w, ctime, out, out_weights, roots);
        }
        if b_id != 0 {
            if let Some(b) = b {
                let w = (f64::from(interp) * f64::from(weight)) as f32;
                b.animate(ctx, tracker, w, ctime, out, out_weights, roots);
            }
        }
    }

    /// `AnimInterpolator::GetTriggers`: like `AnimCube::GetTriggers` over the two animators, with
    /// weights `1 - t` and `t`: each one's effects (keyed by name) merge into weighted sums and
    /// the merged effects before `ctime` (within the cycle) are the track's. Hooks and impulses
    /// the animators added stay as added.
    pub(super) fn triggers(
        &self,
        ctx: &dyn Ac2AnimatorContext,
        tracker: Ac2TrackerState,
        ctime: f32,
        out: &mut Ac2TrackTriggers,
    ) {
        let mut ctime = ctime;
        if 1.0 < ctime {
            ctime = f64::from(ctime).fract() as f32;
        }
        let picked = self.pick(ctx);
        if picked.a_id == 0 {
            return;
        }
        let mut merged: RetailHash<Ac2AnimTrigger> = RetailHash::default();
        let first = (1.0 - f64::from(picked.interp)) as f32;
        let sum = |w: f32, value: f32, total: f32| {
            (f64::from(w) * f64::from(value) + f64::from(total)) as f32
        };
        let scaled = |w: f32, value: f32| (f64::from(w) * f64::from(value)) as f32;
        let mut merge = |child: &Arc<Ac2DbAnimator>, w: f32, out: &mut Ac2TrackTriggers| {
            child.triggers(ctx, tracker, 1.0, false, out);
            for effect in &out.effects {
                match merged.get_mut(effect.fx) {
                    Some(total) => {
                        total.time = sum(w, effect.time, total.time);
                        total.intensity = sum(w, effect.intensity, total.intensity);
                        total.intensity_var = sum(w, effect.intensity_var, total.intensity_var);
                        total.probability = sum(w, effect.probability, total.probability);
                    }
                    None => {
                        merged.add(
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
            out.effects.clear();
        };
        if let Some(a) = picked.a {
            merge(a, first, out);
        }
        if picked.b_id != 0 {
            if let Some(b) = picked.b {
                merge(b, picked.interp, out);
            }
        }
        for (_, effect) in merged.iter() {
            if effect.time < ctime {
                out.effects.push(*effect);
                out.triggered_effects.insert(effect.fx);
            }
        }
    }
}

impl Ac2Random {
    /// `AMRandom::Animate`: while the slider interpolator rests and `frequency` plus a roll of
    /// `variation` has passed, ramp it from where it stands to a random animator over `ramp_time`
    /// (nothing animates when there is none); then the interpolator animates.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn animate(
        &self,
        ctx: &mut dyn Ac2AnimatorContext,
        tracker: Ac2TrackerState,
        weight: f32,
        ctime: f32,
        out: &mut [Ac2BoneTransform],
        out_weights: &mut [f32],
        roots: &[bool],
    ) {
        let interpolator = ctx.slider_interpolator(self.slider);
        if !interpolator.active {
            let roll = ctx.roll_dice_f32(-self.variation, self.variation);
            if f64::from(roll) + f64::from(self.frequency) < f64::from(interpolator.time) {
                let Some(last) = i32::try_from(self.interp.ids.len())
                    .ok()
                    .and_then(|len| len.checked_sub(1))
                    .filter(|&last| last >= 0)
                else {
                    return;
                };
                let pick = ctx.roll_dice(0, last);
                ctx.activate_slider_interpolator(
                    self.slider,
                    interpolator.start,
                    pick as u32 as f32,
                    self.ramp_time,
                );
            }
        }
        self.interp
            .animate(ctx, tracker, weight, ctime, out, out_weights, roots);
    }
}

/// `AnimInterpolator::StreamPack` after the type word: `mSlider`, then `dbAnimatorIDs`
/// (a count and that many ids).
pub(super) fn read_interpolator_ids(
    cursor: &mut ac2_dat::ArchiveCursor<'_>,
) -> Result<(u32, Vec<u32>), &'static str> {
    let slider = cursor.read_u32().map_err(|_| "interpolator slider")?;
    let count = cursor.read_u32().map_err(|_| "interpolator count")? as usize;
    if count > 1 << 12 || count * 4 > cursor.remaining() {
        return Err("interpolator animator count");
    }
    let ids = (0..count)
        .map(|_| cursor.read_u32().map_err(|_| "interpolator ids"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((slider, ids))
}
