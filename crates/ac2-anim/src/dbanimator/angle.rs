//! `AMAngle` (`DBAnimator` type 0x4000000A, `av_angle.amod`): tilts listed bones to the ground the
//! body stands on. Like `AMLook` it adds no pose of its own; its period is one second (the folded
//! `AMLook::GetPeriod`) and it adds no triggers.
//!
//! `AMAngle::Animate` (Legions 0x00632B10, read from the disassembly: both Ghidra exports alias
//! its stack):
//! - while the object has contact and is not sliding down (`PHYSICSOBJREP::HasContact`,
//!   `IsSlidingDown`), the contact plane's normal (`GetContactPlane`) in the object's frame
//!   (`Position::globaltolocalvec`) gives a tilt: its y clamped to `mMinY..mMaxY`, its x clamped
//!   to `mMinX..mMaxX`; otherwise both are 0;
//! - the current values of sliders 0x4000001B and 0x4000001C are read, and, while the object's
//!   position is valid, the sliders are sent to minus the y tilt and the x tilt
//!   (`DoSlider(0, slider, target, time)`), over 0.25 s when the current value is more than 0.15
//!   from the target, else at once;
//! - unless either current value is NaN, every bone whose type is a listed `BoneAngleMod`'s is
//!   turned (`Quaternion::rotate`) by `(0x1B value x weight x mod.x, 0x1C value x weight x mod.y,
//!   0)`, skipped when a component is NaN.

use super::look::rotate;
use super::{Ac2AnimatorContext, Ac2BoneTransform};

/// The tilt sliders.
const SLIDER_TILT_X: u32 = 0x4000_001B;
const SLIDER_TILT_Y: u32 = 0x4000_001C;
/// `0.15f` as the double `fcompl` compares against.
const RAMP_THRESHOLD: f64 = 0.150_000_005_960_464_48;
/// The ramp time sent when the slider is far from its target.
const RAMP_TIME: f32 = 0.25;

/// One `BoneAngleMod` (`BoneAngleMod::StreamPack`: bone type, x and y weights).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2BoneAngleMod {
    pub bone: u32,
    pub x: f32,
    pub y: f32,
}

/// `AMAngle`'s fields (`AMAngle::StreamPack`: the bone list, then `mMaxX`, `mMinX`, `mMaxY`,
/// `mMinY`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2AngleAnimator {
    pub bones: Vec<Ac2BoneAngleMod>,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

/// `if (max < v) v = max; if (v < min) v = min;` with x87's unordered branches.
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    let mut v = value;
    if max < v {
        v = max;
    }
    if v < min {
        v = min;
    }
    v
}

impl Ac2AngleAnimator {
    /// `AMAngle::Animate` at `weight` over the bones in `out`.
    pub(super) fn animate(
        &self,
        ctx: &mut dyn Ac2AnimatorContext,
        weight: f32,
        out: &mut [Ac2BoneTransform],
    ) {
        let (mut along_y, mut along_x) = (0.0_f32, 0.0_f32);
        if let Some([nx, ny, nz]) = ctx.local_contact_normal() {
            let [x, y, z] = [nx, ny, nz].map(f64::from);
            along_y = clamp((x * 0.0 + z * 0.0 + y) as f32, self.min_y, self.max_y);
            along_x = clamp((y * 0.0 + z * 0.0 + x) as f32, self.min_x, self.max_x);
        }
        let targets = [(SLIDER_TILT_X, -along_y), (SLIDER_TILT_Y, along_x)];
        let mut current = [0.0_f32; 2];
        for (slot, (slider, target)) in current.iter_mut().zip(targets) {
            *slot = ctx.slider(slider);
            let time = if (f64::from(*slot) - f64::from(target)).abs() > RAMP_THRESHOLD {
                RAMP_TIME
            } else {
                0.0
            };
            if ctx.position_valid() {
                ctx.do_slider(slider, target, time);
            }
        }
        let [tilt_x, tilt_y] = current;
        if tilt_x.is_nan() || tilt_y.is_nan() {
            return;
        }
        let w = f64::from(weight);
        for (bone, transform) in out.iter_mut().enumerate() {
            for angle in self.bones.iter().filter(|m| ctx.is_bone_type(bone, m.bone)) {
                let turn = [
                    ((f64::from(tilt_x) * w) as f32 as f64 * f64::from(angle.x)) as f32,
                    ((f64::from(tilt_y) * w) as f32 as f64 * f64::from(angle.y)) as f32,
                    (w * 0.0) as f32,
                ];
                if turn.iter().any(|v| v.is_nan()) {
                    continue;
                }
                if let Some(turned) = rotate(transform.rotation, turn) {
                    transform.rotation = turned;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Ctx {
        sliders: [f32; 2],
        sent: Vec<(u32, f32, f32)>,
        normal: Option<[f32; 3]>,
        bones: Vec<u32>,
    }

    impl Ac2AnimatorContext for Ctx {
        fn slider(&self, slider: u32) -> f32 {
            self.sliders[usize::from(slider == SLIDER_TILT_Y)]
        }
        fn do_slider(&mut self, slider: u32, pos: f32, time: f32) {
            self.sent.push((slider, pos, time));
        }
        fn anim_velocity(&self) -> [f32; 3] {
            [0.0; 3]
        }
        fn effects_scale(&self) -> [f32; 3] {
            [1.0; 3]
        }
        fn vdesc_scale(&self) -> [f32; 3] {
            [1.0; 3]
        }
        fn omega(&self) -> [f32; 3] {
            [0.0; 3]
        }
        fn terrain_type(&self) -> u32 {
            0
        }
        fn is_bone_type(&self, bone: usize, ty: u32) -> bool {
            self.bones.get(bone) == Some(&ty)
        }
        fn local_contact_normal(&self) -> Option<[f32; 3]> {
            self.normal
        }
    }

    fn angle() -> Ac2AngleAnimator {
        Ac2AngleAnimator {
            bones: vec![Ac2BoneAngleMod {
                bone: 0x8100_000E,
                x: -0.43,
                y: 0.0,
            }],
            min_x: -0.35,
            max_x: 0.35,
            min_y: -0.35,
            max_y: 0.35,
        }
    }

    /// On a slope the sliders head for minus the clamped y and the clamped x of the normal in
    /// the body's frame, ramping over 0.25 s when far; off the ground they head for zero.
    #[test]
    fn the_ground_normal_drives_the_tilt_sliders() {
        let mut ctx = Ctx {
            sliders: [0.0, 0.0],
            sent: Vec::new(),
            normal: Some([0.1, 0.6, 0.79]),
            bones: vec![0x8100_000E],
        };
        let mut out = vec![Ac2BoneTransform {
            offset: [0.0; 3],
            scale: [1.0; 3],
            rotation: [1.0, 0.0, 0.0, 0.0],
        }];
        angle().animate(&mut ctx, 1.0, &mut out);
        assert_eq!(
            ctx.sent,
            [(SLIDER_TILT_X, -0.35, 0.25), (SLIDER_TILT_Y, 0.1, 0.0)]
        );
        // The bone turns by the sliders' current values (zero), so it stays put.
        assert_eq!(out[0].rotation, [1.0, 0.0, 0.0, 0.0]);
        ctx.normal = None;
        ctx.sent.clear();
        ctx.sliders = [-0.35, 0.1];
        angle().animate(&mut ctx, 1.0, &mut out);
        assert_eq!(
            ctx.sent,
            [(SLIDER_TILT_X, 0.0, 0.25), (SLIDER_TILT_Y, 0.0, 0.0)]
        );
    }

    /// A listed bone turns about x by the 0x1B slider times the weight times its x weight.
    #[test]
    fn a_listed_bone_turns_by_the_slider_and_its_weight() {
        let mut ctx = Ctx {
            sliders: [-0.35, 0.0],
            sent: Vec::new(),
            normal: None,
            bones: vec![0x8100_0001, 0x8100_000E],
        };
        let identity = Ac2BoneTransform {
            offset: [0.0; 3],
            scale: [1.0; 3],
            rotation: [1.0, 0.0, 0.0, 0.0],
        };
        let mut out = vec![identity; 2];
        angle().animate(&mut ctx, 0.5, &mut out);
        assert_eq!(out[0].rotation, [1.0, 0.0, 0.0, 0.0]);
        let angle_x = -0.35_f32 * 0.5 * -0.43;
        let [w, x, y, z] = out[1].rotation;
        assert!((w - (angle_x / 2.0).cos()).abs() < 1e-6);
        assert!((x - (angle_x / 2.0).sin()).abs() < 1e-6);
        assert_eq!((y, z), (0.0, 0.0));
    }
}
