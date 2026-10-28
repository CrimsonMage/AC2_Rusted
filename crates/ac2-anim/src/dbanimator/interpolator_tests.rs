//! `AMRandom` and `AnimInterpolator` over synthetic clips.

use std::collections::HashMap;
use std::sync::Arc;

use super::*;
use crate::behavior::Ac2SliderInterpolator;

const SLIDER: u32 = 0x4000_0030;
const STATE: Ac2TrackerState = Ac2TrackerState {
    element: 0x4000_0001,
    mode: 0x4000_0001,
};

#[derive(Default)]
struct Ctx {
    interpolators: HashMap<u32, Ac2SliderInterpolator>,
    activated: Vec<(u32, f32, f32, f32)>,
    float_roll: f32,
    long_roll: i32,
}

impl Ac2AnimatorContext for Ctx {
    fn slider(&self, _slider: u32) -> f32 {
        0.0
    }
    fn do_slider(&mut self, _slider: u32, _pos: f32, _time: f32) {}
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
    fn slider_interpolator(&self, slider: u32) -> Ac2SliderInterpolator {
        self.interpolators.get(&slider).copied().unwrap_or_default()
    }
    fn activate_slider_interpolator(&mut self, slider: u32, start: f32, end: f32, length: f32) {
        self.activated.push((slider, start, end, length));
        let mut interpolator = self.slider_interpolator(slider);
        interpolator.activate(start, end, length);
        self.interpolators.insert(slider, interpolator);
    }
    fn roll_dice(&mut self, _lower: i32, _upper: i32) -> i32 {
        self.long_roll
    }
    fn roll_dice_f32(&mut self, _lower: f32, _upper: f32) -> f32 {
        self.float_roll
    }
}

/// A one-bone clip holding bone 0 at x offset `x` for `period` seconds.
fn clip(period: f32, x: f32) -> Arc<Ac2DbAnimator> {
    Arc::new(Ac2DbAnimator::Asset(Arc::new(Ac2AnimAsset {
        period,
        bones: vec![Ac2AssetBone {
            scale: vec![([1.0; 3], 0.0)],
            offset: vec![([x, 0.0, 0.0], 0.0), ([x, 0.0, 0.0], period)],
            rotation: vec![([1.0, 0.0, 0.0, 0.0], 0.0)],
            weight: vec![(1.0, 0.0)],
        }],
        info: None,
    })))
}

fn random(variation: f32, frequency: f32) -> Ac2DbAnimator {
    Ac2DbAnimator::Random(Ac2Random {
        interp: Ac2Interpolator {
            slider: SLIDER,
            ids: Arc::from([10, 11]),
            children: Arc::from([Some(clip(2.0, 1.0)), Some(clip(4.0, 5.0))]),
        },
        variation,
        frequency,
        ramp_time: 0.5,
        slider: SLIDER,
    })
}

fn pose(animator: &Ac2DbAnimator, ctx: &mut Ctx) -> f32 {
    let mut out = vec![Ac2BoneTransform::ZERO; 1];
    let mut weights = vec![0.0; 1];
    animator.animate(ctx, STATE, 1.0, 0.0, &mut out, &mut weights, &[true]);
    out[0].offset[0]
}

#[test]
fn a_resting_interpolator_plays_the_animator_it_stands_on() {
    let animator = random(0.0, 5.0);
    let mut ctx = Ctx::default();
    assert_eq!(animator.period(&ctx, STATE), 2.0);
    assert_eq!(pose(&animator, &mut ctx), 1.0);
    ctx.interpolators.insert(
        SLIDER,
        Ac2SliderInterpolator {
            start: 1.0,
            ..Default::default()
        },
    );
    assert_eq!(animator.period(&ctx, STATE), 4.0);
    assert_eq!(pose(&animator, &mut ctx), 5.0);
}

#[test]
fn a_ramping_interpolator_crossfades_period_and_pose() {
    let animator = random(0.0, 5.0);
    let mut ctx = Ctx::default();
    ctx.interpolators.insert(
        SLIDER,
        Ac2SliderInterpolator {
            active: true,
            start: 0.0,
            end: 1.0,
            current: 0.25,
            ..Default::default()
        },
    );
    assert_eq!(animator.period(&ctx, STATE), 0.75 * 2.0 + 0.25 * 4.0);
    assert_eq!(pose(&animator, &mut ctx), 0.75 * 1.0 + 0.25 * 5.0);
    // A ramp to the animator it already plays is not a crossfade.
    ctx.interpolators.get_mut(&SLIDER).unwrap().end = 0.0;
    assert_eq!(animator.period(&ctx, STATE), 2.0);
}

#[test]
fn amrandom_starts_a_ramp_once_the_frequency_and_roll_have_passed() {
    let animator = random(1.0, 5.0);
    let mut ctx = Ctx {
        float_roll: -0.5,
        long_roll: 1,
        ..Ctx::default()
    };
    // Rested 4.0 s: 4.5 is not past it.
    ctx.interpolators.insert(
        SLIDER,
        Ac2SliderInterpolator {
            time: 4.4,
            ..Default::default()
        },
    );
    pose(&animator, &mut ctx);
    assert!(ctx.activated.is_empty());
    ctx.interpolators.get_mut(&SLIDER).unwrap().time = 4.6;
    pose(&animator, &mut ctx);
    assert_eq!(ctx.activated, vec![(SLIDER, 0.0, 1.0, 0.5)]);
    // Ramping now: no further roll starts another.
    pose(&animator, &mut ctx);
    assert_eq!(ctx.activated.len(), 1);
}

#[test]
fn the_interpolator_merges_the_effects_of_both_animators_by_weight() {
    let with_effect = |period: f32, time: f32, intensity: f32| {
        Arc::new(Ac2DbAnimator::Asset(Arc::new(Ac2AnimAsset {
            period,
            bones: Vec::new(),
            info: Some(Ac2AnimInfo {
                effects: vec![Ac2AnimTrigger {
                    fx: 9,
                    time,
                    stop: false,
                    probability: 1.0,
                    intensity,
                    intensity_var: 0.0,
                    play: 1,
                }],
                ..Ac2AnimInfo::default()
            }),
        })))
    };
    let animator = Ac2DbAnimator::Random(Ac2Random {
        interp: Ac2Interpolator {
            slider: SLIDER,
            ids: Arc::from([10, 11]),
            children: Arc::from([
                Some(with_effect(1.0, 0.2, 1.0)),
                Some(with_effect(1.0, 0.6, 3.0)),
            ]),
        },
        variation: 0.0,
        frequency: 5.0,
        ramp_time: 0.5,
        slider: SLIDER,
    });
    let mut ctx = Ctx::default();
    ctx.interpolators.insert(
        SLIDER,
        Ac2SliderInterpolator {
            active: true,
            start: 0.0,
            end: 1.0,
            current: 0.5,
            ..Default::default()
        },
    );
    let mut found = Ac2TrackTriggers::default();
    animator.triggers(&ctx, STATE, 0.9, false, &mut found);
    assert_eq!(found.effects.len(), 1);
    let effect = found.effects[0];
    assert!((effect.time - 0.4).abs() < 1e-6 && effect.intensity == 2.0);
    assert!(found.triggered_effects.contains(&9));
    // Before the merged time nothing has been reached.
    let mut early = Ac2TrackTriggers::default();
    animator.triggers(&ctx, STATE, 0.3, false, &mut early);
    assert!(early.effects.is_empty());
}

/// Every `DBAnimator` record of the portal DAT decodes, and the type words are the ones
/// `Animator::Create` builds: `AxisCube` (0x40000006) has no record, `AMRandom` loads with every
/// animator it names (only with `AC2_DAT_DIR`).
#[test]
fn retail_animator_records_decode_and_amrandom_loads_its_animators() {
    use ac2_dat::DatFileKind;
    use ac2_db::ResourceStore;

    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let mut randoms = 0;
    for did in dataset.ids_in_range(DatFileKind::Portal, 0x0500_0000, 0x0500_ffff) {
        let Ok(Some(bytes)) = dataset.read_packed(DatFileKind::Portal, did) else {
            continue;
        };
        let word = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        assert_ne!(word, 0x4000_0006, "{did:#x}: AxisCube");
        let animator = super::tree::load_animator_did(&store, ac2_core::DataId::new(did))
            .unwrap_or_else(|e| panic!("{did:#x}: {e:?}"))
            .expect("record present");
        if word == 0x4000_0007 {
            randoms += 1;
            let Ac2DbAnimator::Random(random) = &*animator else {
                panic!("{did:#x}: not AMRandom");
            };
            assert!(!random.interp.ids.is_empty());
            assert!(random.interp.children.iter().all(Option::is_some));
            let ctx = Ctx::default();
            assert!(animator.period(&ctx, STATE).is_finite());
        }
    }
    assert!(randoms >= 2, "{randoms}");
}

/// Bone 0 is a neck bone, bone 1 a head bone, bone 2 neither.
struct LookCtx {
    sliders: [f32; 3],
}

impl Ac2AnimatorContext for LookCtx {
    fn slider(&self, slider: u32) -> f32 {
        self.sliders
            .get((slider.wrapping_sub(0x4000_0007)) as usize)
            .copied()
            .unwrap_or(0.0)
    }
    fn do_slider(&mut self, _slider: u32, _pos: f32, _time: f32) {}
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
        matches!((bone, ty), (0, 0x4000_0003) | (1, 0x4000_0006))
    }
}

#[test]
fn amlook_turns_the_neck_and_head_bones_by_the_look_sliders() {
    let animator = Ac2DbAnimator::Look;
    // A quarter turn about z (the third slider) at the rest pose.
    let mut ctx = LookCtx {
        sliders: [0.0, 0.0, 0.5],
    };
    assert_eq!(animator.period(&ctx, STATE), 1.0);
    let rest = Ac2BoneTransform {
        offset: [0.0; 3],
        scale: [1.0; 3],
        rotation: IDENTITY_Q,
    };
    let mut out = vec![rest; 3];
    let mut weights = vec![0.0; 3];
    animator.animate(
        &mut ctx,
        STATE,
        1.0,
        0.0,
        &mut out,
        &mut weights,
        &[true; 3],
    );
    // Neck: z * 0.25 * 1.4 = 0.175 rad about z; head: 0.5 * 0.25 * 0.93 = 0.11625.
    let turn = |bone: usize| 2.0 * f64::from(out[bone].rotation[3]).asin();
    assert!((turn(0) - 0.175).abs() < 1e-6, "{}", turn(0));
    assert!((turn(1) - 0.11625).abs() < 1e-6, "{}", turn(1));
    assert_eq!(
        out[2].rotation, IDENTITY_Q,
        "other bones keep their rotation"
    );
    // The look is held to the unit sphere: 3 on each axis is as the unit vector.
    let mut big = LookCtx {
        sliders: [3.0, 0.0, 0.0],
    };
    let mut out = vec![rest; 3];
    animator.animate(
        &mut big,
        STATE,
        1.0,
        0.0,
        &mut out,
        &mut weights,
        &[true; 3],
    );
    let angle = 2.0 * f64::from(out[0].rotation[1]).asin();
    assert!((angle - 0.35).abs() < 1e-6, "{angle}");
}

const IDENTITY_Q: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
