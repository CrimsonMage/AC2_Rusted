//! `DBAnimator` evaluated as retail evaluates it: `AnimAsset` keyframe clips with per-bone blend
//! weights, `AnimCube` slider lattices, `AMHuman` (which drives the locomotion sliders from the
//! body's velocity and turn rate) and `AMSwitchME` (an animator per mode, element or terrain),
//! blended by `AnimBlender` per layer and `AnimCompositor` across layers into the bone
//! `TransformBase`s `CPartArray::DoAnimation` poses a body with.
//!
//! Float arithmetic follows the retail x87 model: expressions in `f64`, rounded to `f32` where
//! retail stores a `float`.

// Negated float comparisons keep retail's unordered (NaN) branches.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

mod asset;
mod compose;
mod tree;
mod triggers;

pub(crate) use asset::{decode_effects, decode_impulses};
pub use asset::{
    Ac2AnimAsset, Ac2AnimInfo, Ac2AnimTrigger, Ac2AssetBone, Ac2ImpulseTrigger, Ac2WeenieHook,
};
pub use compose::{Ac2Compositor, Ac2PoseInput};
pub use tree::{load_ac2_db_animator, Ac2CubeSlider, Ac2DbAnimator};
pub use triggers::Ac2TrackTriggers;

/// What animators read from (and, for `AMHuman`, write to) the animated object.
pub trait Ac2AnimatorContext {
    /// `GetSliderCurrentValue`.
    fn slider(&self, slider: u32) -> f32;
    /// `DoSlider(0, slider, pos, time)`.
    fn do_slider(&mut self, slider: u32, pos: f32, time: f32);
    /// `GetAnimVelocity` (body-local, m/s).
    fn anim_velocity(&self) -> [f32; 3];
    /// `GetEffectsScale` and `GetVdescScale`.
    fn effects_scale(&self) -> [f32; 3];
    fn vdesc_scale(&self) -> [f32; 3];
    /// `GetOmegaVelocity` (degrees per second).
    fn omega(&self) -> [f32; 3];
    /// `GetTerrainType`.
    fn terrain_type(&self) -> u32;
}

/// The element and mode of the `BehaviorTracker` an animator plays for (`AMSwitchME`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ac2TrackerState {
    pub element: u32,
    pub mode: u32,
}

/// `TransformBase`: offset, per-axis scale and rotation `(w, x, y, z)`, accumulated as a weighted
/// sum while animating.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2BoneTransform {
    pub offset: [f32; 3],
    pub scale: [f32; 3],
    pub rotation: [f32; 4],
}

impl Ac2BoneTransform {
    pub const ZERO: Self = Self {
        offset: [0.0; 3],
        scale: [0.0; 3],
        rotation: [0.0; 4],
    };

    /// `TransformBase::AddWeightedTransform`.
    pub(crate) fn add_weighted(&mut self, other: &Self, weight: f32) {
        let w = f64::from(weight);
        let add = |dst: &mut f32, src: f32| *dst = (w * f64::from(src) + f64::from(*dst)) as f32;
        for i in 0..3 {
            add(&mut self.offset[i], other.offset[i]);
        }
        for i in 0..3 {
            add(&mut self.scale[i], other.scale[i]);
        }
        for i in 0..4 {
            add(&mut self.rotation[i], other.rotation[i]);
        }
    }

    /// `TransformBase::WeightTransform`.
    pub(crate) fn weight(&mut self, weight: f32) {
        let w = f64::from(weight);
        for value in self
            .offset
            .iter_mut()
            .chain(self.scale.iter_mut())
            .chain(self.rotation.iter_mut())
        {
            *value = (w * f64::from(*value)) as f32;
        }
    }

    /// As the renderer's `Ac2Transform`.
    pub fn to_transform(&self) -> crate::Ac2Transform {
        let [w, x, y, z] = self.rotation;
        crate::Ac2Transform {
            offset: ac2_math::Vec3::from(self.offset),
            scale: ac2_math::Vec3::from(self.scale),
            rotation: ac2_math::Quat::from_xyzw(x, y, z, w),
        }
    }

    pub fn from_transform(transform: &crate::Ac2Transform) -> Self {
        let q = transform.rotation;
        Self {
            offset: transform.offset.to_array(),
            scale: transform.scale.to_array(),
            rotation: [q.w, q.x, q.y, q.z],
        }
    }
}

/// `D3DXQuaternionSlerp` (the x87 `c_` path the statically linked D3DX takes without 3DNow!)
/// over `(w, x, y, z)` quaternions.
pub(crate) fn d3dx_slerp(q1: [f32; 4], q2: [f32; 4], t: f32) -> [f32; 4] {
    let ([w1, x1, y1, z1], [w2, x2, y2, z2]) = (q1.map(f64::from), q2.map(f64::from));
    let t = f64::from(t);
    let s = 1.0 - t;
    let dot = ((x1 * x2 + z2 * z1) + y2 * y1) + w2 * w1;
    let sign: f64 = if dot < 0.0 { -1.0 } else { 1.0 };
    let d = sign * dot;
    let d_stored = f64::from(d as f32);
    let (a, b) = if 1.0 - d > f64::from(1.192_092_9e-7_f32) {
        let sin_theta = (1.0 - d * d).sqrt();
        let theta = f64::from(sin_theta.atan2(d_stored) as f32);
        let inverse = f64::from((1.0 / sin_theta) as f32);
        ((theta * s).sin() * inverse, (theta * t).sin() * inverse)
    } else {
        (s, t)
    };
    let b = b * sign;
    [
        (b * w2 + a * w1) as f32,
        (a * x1 + b * x2) as f32,
        (b * y2 + a * y1) as f32,
        (b * z2 + a * z1) as f32,
    ]
}

#[cfg(test)]
mod tests;
