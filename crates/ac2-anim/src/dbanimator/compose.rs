//! `CPartArray::DoAnimation` over `AnimCompositor` and its `AnimBlender` layers: every playing
//! track adds its animator to its layer with the track weight times its tracker's transition
//! weight; layers blend from the highest down, a lower layer only filling the bone weight the
//! higher ones left (tweak layers add on top); bones end normalised by their weight, or at the
//! Setup's placement pose where nothing animated them.

use super::{Ac2AnimatorContext, Ac2BoneTransform, Ac2DbAnimator, Ac2TrackerState};
use crate::behavior::{Ac2BehaviorTracker, Ac2ModeTracker};

/// `F_EPSILON`: a bone weight under this keeps the placement pose.
const F_EPSILON: f32 = 0.0002;

/// What a body's parts pose with: the behaviour table's layer count and tweak layers, which
/// bones are roots, and the Setup placement frame bones fall back to.
#[derive(Debug, Clone, Copy)]
pub struct Ac2PoseInput<'a> {
    pub layers: usize,
    pub tweak_layers: &'a [i32],
    pub roots: &'a [bool],
    pub rest: &'a [Ac2BoneTransform],
}

/// `AnimBlender::BlendArray` entry.
struct Blend<'a> {
    animator: &'a Ac2DbAnimator,
    weight: f32,
    tracker: Ac2TrackerState,
    playing_move: bool,
    time: f32,
}

/// `AnimCompositor` (one `AnimBlender` per layer; the layer weights stay 1: no slider drives a
/// layer).
pub struct Ac2Compositor;

impl Ac2Compositor {
    /// `CPartArray::DoAnimation` with `refreshBones`: the bone transforms of the mode tracker's
    /// current animation state.
    pub fn pose<'a>(
        modes: &Ac2ModeTracker,
        animator: impl Fn(u32) -> Option<&'a Ac2DbAnimator>,
        ctx: &mut dyn Ac2AnimatorContext,
        input: &Ac2PoseInput,
    ) -> Vec<Ac2BoneTransform> {
        let bones = input.rest.len();
        let mut layers: Vec<Vec<Blend>> = (0..input.layers).map(|_| Vec::new()).collect();
        let (from, to, transition) = modes.trackers();
        if let Some(from) = from {
            match to {
                None => add_animation(&mut layers, from, 1.0, &animator),
                Some(to) => {
                    let out_weight = (1.0 - f64::from(transition)) as f32;
                    add_animation(&mut layers, from, out_weight, &animator);
                    add_animation(&mut layers, to, transition, &animator);
                }
            }
        }
        // `AnimCompositor::Init` stores the table's tweak flags in reverse layer order.
        let tweak = |layer: usize| {
            input
                .tweak_layers
                .get(input.layers - 1 - layer)
                .copied()
                .unwrap_or(0)
        };
        let mut out = vec![Ac2BoneTransform::ZERO; bones];
        let mut weights = vec![0.0_f32; bones];
        for layer in (0..layers.len()).rev() {
            let blends = &layers[layer];
            if blends.is_empty() {
                continue;
            }
            if tweak(layer) != 0 {
                blend_layer(blends, true, 1.0, ctx, &mut out, &mut weights, input.roots);
                continue;
            }
            let mut temp = vec![Ac2BoneTransform::ZERO; bones];
            let mut temp_weights = vec![0.0_f32; bones];
            blend_layer(
                blends,
                false,
                1.0,
                ctx,
                &mut temp,
                &mut temp_weights,
                input.roots,
            );
            for bone in 0..bones {
                fill_remaining(
                    &mut out[bone],
                    &mut weights[bone],
                    &temp[bone],
                    temp_weights[bone],
                );
            }
        }
        for bone in 0..bones {
            normalize_bone(&mut out[bone], weights[bone], &input.rest[bone]);
        }
        out
    }
}

/// `CPartArray::AddAnimation`: the tracker's tracks with an animator and a positive weight, in
/// retail iteration order.
fn add_animation<'a>(
    layers: &mut [Vec<Blend<'a>>],
    tracker: &Ac2BehaviorTracker,
    tracker_weight: f32,
    animator: &impl Fn(u32) -> Option<&'a Ac2DbAnimator>,
) {
    let state = Ac2TrackerState {
        element: tracker.element(),
        mode: tracker.mode(),
    };
    for (_, track) in tracker.tracks() {
        let Some(found) = animator(track.anim_key) else {
            continue;
        };
        if !(0.0 < track.weight) {
            continue;
        }
        let Some(layer) = layers.get_mut(track.layer as usize) else {
            continue;
        };
        layer.push(Blend {
            animator: found,
            weight: (f64::from(tracker_weight) * f64::from(track.weight)) as f32,
            tracker: state,
            playing_move: tracker.is_playing_move(),
            time: track.ctime,
        });
    }
}

/// `AnimBlender::Animate`. Retail's normalisation of an over-weighted layer reloads 1.0 after
/// the first bone, so only bone 0 is scaled; that is kept.
fn blend_layer(
    blends: &[Blend],
    tweak: bool,
    layer_weight: f32,
    ctx: &mut dyn Ac2AnimatorContext,
    out: &mut [Ac2BoneTransform],
    out_weights: &mut [f32],
    roots: &[bool],
) {
    let bones = out.len();
    for blend in blends {
        let w = (f64::from(layer_weight) * f64::from(blend.weight)) as f32;
        if tweak {
            blend
                .animator
                .animate(ctx, blend.tracker, w, blend.time, out, out_weights, roots);
            continue;
        }
        let mut temp = vec![Ac2BoneTransform::ZERO; bones];
        let mut temp_weights = vec![0.0_f32; bones];
        blend.animator.animate(
            ctx,
            blend.tracker,
            1.0,
            blend.time,
            &mut temp,
            &mut temp_weights,
            roots,
        );
        for bone in 0..bones {
            let added = if blend.playing_move {
                out[bone].add_weighted(&temp[bone], w);
                f64::from(w) * f64::from(temp_weights[bone])
            } else if temp_weights[bone] == 0.0 {
                // Retail divides by the zero weight here and poisons the bone with NaN; a bone
                // the animation does not weigh is left out instead.
                continue;
            } else {
                let share = (f64::from(w) / f64::from(temp_weights[bone])) as f32;
                out[bone].add_weighted(&temp[bone], share);
                f64::from(w)
            };
            out_weights[bone] = (added + f64::from(out_weights[bone])) as f32;
        }
    }
    let total: f64 = blends
        .iter()
        .map(|blend| f64::from(layer_weight) * f64::from(blend.weight))
        .sum();
    if total > 1.0 && bones > 0 {
        let inverse = (1.0 / total) as f32;
        out[0].weight(inverse);
        out_weights[0] = (f64::from(inverse) * f64::from(out_weights[0])) as f32;
    }
}

/// The non-tweak layer merge of `AnimCompositor::Animate`: a lower layer only fills the weight
/// the bone still lacks.
fn fill_remaining(
    out: &mut Ac2BoneTransform,
    weight: &mut f32,
    layer: &Ac2BoneTransform,
    layer_weight: f32,
) {
    if !(f64::from(*weight) < 1.0) {
        return;
    }
    if *weight == 0.0 {
        *weight = layer_weight;
        *out = *layer;
        return;
    }
    let remaining = 1.0 - f64::from(*weight);
    let (taken, share) = if remaining <= f64::from(layer_weight) {
        (
            remaining as f32,
            (remaining / f64::from(layer_weight)) as f32,
        )
    } else {
        (layer_weight, 1.0)
    };
    out.add_weighted(layer, share);
    *weight = (f64::from(taken) + f64::from(*weight)) as f32;
}

/// The per-bone normalisation of `CPartArray::DoAnimation`.
fn normalize_bone(bone: &mut Ac2BoneTransform, weight: f32, rest: &Ac2BoneTransform) {
    if !(f64::from(weight) >= f64::from(F_EPSILON)) {
        *bone = *rest;
        return;
    }
    let mut q = bone.rotation;
    if weight != 0.0 {
        let inverse = 1.0 / f64::from(weight);
        q = q.map(|v| (f64::from(v) * inverse) as f32);
    }
    let [w, x, y, z] = q.map(f64::from);
    let length2 = ((z * z + y * y) + x * x) + w * w;
    let threshold = f64::from(F_EPSILON) * f64::from(F_EPSILON);
    if length2 > threshold {
        let inverse = 1.0 / length2.sqrt();
        q = q.map(|v| (f64::from(v) * inverse) as f32);
    }
    bone.rotation = q;
    let inverse = 1.0 / f64::from(weight);
    bone.offset = bone.offset.map(|v| (inverse * f64::from(v)) as f32);
    let inverse = 1.0 / f64::from(weight);
    bone.scale = bone.scale.map(|v| (inverse * f64::from(v)) as f32);
}
