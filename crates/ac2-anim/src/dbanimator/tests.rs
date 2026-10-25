//! Synthetic animators (no game data).

use std::collections::HashMap;
use std::sync::Arc;

use super::*;
use crate::behavior::{Ac2AnimationSource, Ac2ModeTracker};
use crate::behavior_table::{Ac2BehaviorTable, Ac2CharTemplate, Ac2ComboInfo};

#[derive(Default)]
struct Ctx {
    sliders: HashMap<u32, f32>,
    set: Vec<(u32, f32, f32)>,
    velocity: [f32; 3],
    omega: [f32; 3],
}

impl Ac2AnimatorContext for Ctx {
    fn slider(&self, slider: u32) -> f32 {
        self.sliders.get(&slider).copied().unwrap_or(0.0)
    }
    fn do_slider(&mut self, slider: u32, pos: f32, time: f32) {
        self.set.push((slider, pos, time));
    }
    fn anim_velocity(&self) -> [f32; 3] {
        self.velocity
    }
    fn effects_scale(&self) -> [f32; 3] {
        [1.0; 3]
    }
    fn vdesc_scale(&self) -> [f32; 3] {
        [1.0; 3]
    }
    fn omega(&self) -> [f32; 3] {
        self.omega
    }
    fn terrain_type(&self) -> u32 {
        0
    }
}

const STATE: Ac2TrackerState = Ac2TrackerState {
    element: 0x4000_0001,
    mode: 0x4000_0001,
};
const IDENTITY: [f32; 4] = [1.0, 0.0, 0.0, 0.0];

/// A two-bone clip moving bone 0 from `from` to `to` (x offset) over `period` seconds; bone 1
/// carries `weight` as its blend weight.
fn clip(period: f32, from: f32, to: f32, weight: f32) -> Arc<Ac2DbAnimator> {
    let bone = |x0: f32, x1: f32, w: f32| Ac2AssetBone {
        scale: vec![([1.0; 3], 0.0)],
        offset: vec![([x0, 0.0, 0.0], 0.0), ([x1, 0.0, 0.0], period)],
        rotation: vec![(IDENTITY, 0.0)],
        weight: vec![(w, 0.0)],
    };
    Arc::new(Ac2DbAnimator::Asset(Arc::new(Ac2AnimAsset {
        period,
        bones: vec![bone(from, to, 1.0), bone(from, to, weight)],
        info: None,
    })))
}

#[test]
fn d3dx_slerp_halves_a_right_angle_and_takes_the_short_way() {
    let a = IDENTITY;
    let b = [0.0, 0.0, 0.0, 1.0];
    let mid = d3dx_slerp(a, b, 0.5);
    let h = std::f32::consts::FRAC_1_SQRT_2;
    assert!(
        (mid[0] - h).abs() < 1e-6 && (mid[3] - h).abs() < 1e-6,
        "{mid:?}"
    );
    let flipped = d3dx_slerp(a, [-1.0, 0.0, 0.0, 0.0], 0.5);
    assert!((flipped[0] - 1.0).abs() < 1e-6, "{flipped:?}");
}

#[test]
fn an_asset_interpolates_its_keys_at_the_cycle_phase_with_its_bone_weights() {
    let animator = clip(2.0, 0.0, 4.0, 0.25);
    let mut ctx = Ctx::default();
    let mut out = vec![Ac2BoneTransform::ZERO; 2];
    let mut weights = vec![0.0; 2];
    // 1.25 cycles of a 2 s clip is 0.5 s in: a quarter of the way.
    animator.animate(
        &mut ctx,
        STATE,
        0.5,
        1.25,
        &mut out,
        &mut weights,
        &[true, false],
    );
    assert_eq!(weights, vec![0.5, 0.125]);
    assert_eq!(out[0].offset[0], 0.5, "1.0 at half weight");
    assert_eq!(out[1].offset[0], 0.125);
    assert_eq!(animator.period(&ctx, STATE), 2.0);
}

#[test]
fn a_cube_splits_its_weight_between_the_bracketing_ticks() {
    let cube = Ac2DbAnimator::Cube {
        sliders: Arc::from(vec![Ac2CubeSlider {
            id: 7,
            ticks: vec![0.0, 2.0, 4.0],
        }]),
        children: Arc::from(vec![
            Some(clip(1.0, 1.0, 1.0, 1.0)),
            Some(clip(3.0, 2.0, 2.0, 1.0)),
            Some(clip(5.0, 3.0, 3.0, 1.0)),
        ]),
    };
    let mut ctx = Ctx::default();
    ctx.sliders.insert(7, 1.0);
    assert_eq!(cube.period(&ctx, STATE), 2.0, "half of 1 s and 3 s");
    let mut out = vec![Ac2BoneTransform::ZERO; 2];
    let mut weights = vec![0.0; 2];
    cube.animate(
        &mut ctx,
        STATE,
        1.0,
        0.0,
        &mut out,
        &mut weights,
        &[true, false],
    );
    assert_eq!(out[0].offset[0], 1.5);
    // On a tick both corners land on it.
    ctx.sliders.insert(7, 4.0);
    assert_eq!(cube.period(&ctx, STATE), 5.0);
    // Past the last tick the value clamps to it.
    ctx.sliders.insert(7, 9.0);
    assert_eq!(cube.period(&ctx, STATE), 5.0);
}

#[test]
fn a_human_animator_drives_the_locomotion_sliders_from_the_velocity() {
    let human = Ac2DbAnimator::Human(Some(clip(1.0, 0.0, 0.0, 1.0)));
    let mut ctx = Ctx {
        velocity: [1.0, 3.0, -12.0],
        omega: [0.0, 0.0, 45.0],
        ..Ctx::default()
    };
    let mut out = vec![Ac2BoneTransform::ZERO; 2];
    let mut weights = vec![0.0; 2];
    human.animate(
        &mut ctx,
        STATE,
        1.0,
        0.0,
        &mut out,
        &mut weights,
        &[true, false],
    );
    let side = ctx.set[0];
    assert_eq!(side.0, 0x4000_0002);
    // Turning at 45 deg/s keeps 1 - 0.785 of the sidestep.
    let kept = (1.0 - (45.0_f64 * f64::from(0.017_455_053_f32)).abs()) as f32;
    assert_eq!(side.1, kept);
    assert_eq!(
        ctx.set[1],
        (0x4000_0001, 3.0, 0.3),
        "a big jump moves over 0.3 s"
    );
    assert_eq!(
        ctx.set[2],
        (0x4000_0003, -10.0, 0.01),
        "fall speed capped at 10"
    );
    let speed = (3.0_f64 * 3.0 + f64::from(kept) * f64::from(kept)).sqrt() * 2.0;
    assert_eq!(ctx.set[3].1, (45.0 / speed) as f32);
    assert_eq!(weights[0], 1.0, "the child animated");
}

struct Source;

impl Ac2AnimationSource for Source {
    fn combo_cvel(&self, _tracker: Ac2TrackerState, _combo: Ac2ComboInfo) -> f32 {
        1.0
    }
    fn hold_loop(&self, _tracker: Ac2TrackerState, _combo: Ac2ComboInfo) -> Option<(f32, f32)> {
        None
    }
}

#[test]
fn a_higher_layer_overrides_a_lower_one_and_unanimated_bones_rest() {
    let (idle, wave) = (0x4000_0002, 0x4000_0050);
    let (g, p) = (STATE.element, STATE.mode);
    let table = Ac2BehaviorTable {
        ramp_times: HashMap::new(),
        combos: HashMap::from([
            (
                (g, p, idle),
                Ac2ComboInfo {
                    anim_key: 1,
                    layer: 0,
                },
            ),
            (
                (g, p, wave),
                Ac2ComboInfo {
                    anim_key: 2,
                    layer: 1,
                },
            ),
        ]),
        transitions: HashMap::new(),
        layers: vec![0, 1],
        tweak_layers: vec![0, 0],
        mode_transitions: HashMap::new(),
        modes: HashMap::from([((g, p), 0)]),
        behavior_info: HashMap::from([(
            wave,
            crate::Ac2BehaviorInfo {
                ramp_time: 0.0,
                effect: 0,
            },
        )]),
        init_behaviors: vec![idle],
        init_mode: p,
    };
    let template = Ac2CharTemplate {
        behaviors: HashMap::from([(idle, 0), (wave, 0)]),
        sliders: HashMap::new(),
        default_element: g,
        default_mode: p,
    };
    let mut modes = Ac2ModeTracker::new(Arc::new(template));
    modes.init(Some(Arc::new(table)), &Source);
    let animators = HashMap::from([(1, clip(1.0, 1.0, 1.0, 1.0)), (2, clip(1.0, 5.0, 5.0, 0.0))]);
    let rest = vec![
        Ac2BoneTransform {
            offset: [9.0, 0.0, 0.0],
            scale: [1.0; 3],
            rotation: IDENTITY,
        };
        3
    ];
    let input = Ac2PoseInput {
        layers: 2,
        tweak_layers: &[0, 0],
        roots: &[true, false, false],
        rest: &rest,
    };
    let mut ctx = Ctx::default();
    let pose = |modes: &Ac2ModeTracker, ctx: &mut Ctx| {
        Ac2Compositor::pose(modes, |key| animators.get(&key).map(|a| &**a), ctx, &input)
    };
    let idle_pose = pose(&modes, &mut ctx);
    assert_eq!(idle_pose[0].offset[0], 1.0);
    assert_eq!(idle_pose[2].offset[0], 9.0, "the clip has no third bone");
    let mut params = crate::Ac2BehaviorParams::new(wave);
    params.context = 0;
    assert!(modes.do_behavior(&params, &Source));
    let waving = pose(&modes, &mut ctx);
    assert_eq!(waving[0].offset[0], 5.0, "the wave layer wins bone 0");
    assert_eq!(
        waving[1].offset[0], 1.0,
        "bone 1 has no wave weight: idle shows"
    );
}

/// Every animator of the human `AnimMap` loads and evaluates to a finite pose (only with
/// `AC2_DAT_DIR`).
#[test]
fn retail_human_animators_load_and_pose_finitely() {
    use ac2_db::ResourceStore;

    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let map_did = ac2_core::DataId::new(0x0300_0014);
    let map = crate::animation::load_portal::<crate::Ac2AnimMap>(&store, map_did)
        .expect("anim map decodes")
        .expect("anim map present");
    let mut loaded = 0;
    for &(key, _) in map.entries.iter() {
        let Some(animator) = load_ac2_db_animator(&store, map_did, key).expect("animator decodes")
        else {
            continue;
        };
        loaded += 1;
        let mut ctx = Ctx::default();
        let period = animator.period(&ctx, STATE);
        assert!(period.is_finite() && period >= 0.0, "{key:#x}: {period}");
        let mut out = vec![Ac2BoneTransform::ZERO; 64];
        let mut weights = vec![0.0; 64];
        let roots = vec![false; 64];
        animator.animate(&mut ctx, STATE, 1.0, 0.3, &mut out, &mut weights, &roots);
        assert!(
            out.iter().all(|bone| bone
                .offset
                .iter()
                .chain(&bone.rotation)
                .all(|v| v.is_finite())),
            "{key:#x}"
        );
    }
    assert!(loaded > 10, "{loaded}");
}

#[test]
fn impulses_count_once_their_time_is_reached_within_the_cycle() {
    let impulse = Ac2ImpulseTrigger {
        jump: true,
        impulse: [0.0; 3],
        num: 0,
        time: 0.5,
    };
    let asset = Ac2DbAnimator::Asset(Arc::new(Ac2AnimAsset {
        period: 1.0,
        bones: Vec::new(),
        info: Some(Ac2AnimInfo {
            impulses: vec![impulse],
            ..Ac2AnimInfo::default()
        }),
    }));
    let ctx = Ctx::default();
    let mut found = Ac2TrackTriggers::default();
    asset.triggers(&ctx, STATE, 0.4, false, &mut found);
    assert!(found.impulses.is_empty());
    asset.triggers(&ctx, STATE, 0.5, false, &mut found);
    assert_eq!(found.impulses, vec![(impulse, false)]);
    asset.triggers(&ctx, STATE, 0.75, false, &mut found);
    assert_eq!(found.impulses.len(), 1, "added once");
    // A looping track wraps its time into the cycle: 1.25 is before the impulse again.
    let mut found = Ac2TrackTriggers::default();
    asset.triggers(&ctx, STATE, 1.25, true, &mut found);
    assert!(found.impulses.is_empty());
}

fn effect(fx: u32, time: f32, intensity: f32) -> Ac2AnimTrigger {
    Ac2AnimTrigger {
        fx,
        time,
        stop: false,
        probability: 1.0,
        intensity,
        intensity_var: 0.0,
        play: 1,
    }
}

fn info_clip(info: Ac2AnimInfo) -> Arc<Ac2DbAnimator> {
    Arc::new(Ac2DbAnimator::Asset(Arc::new(Ac2AnimAsset {
        period: 1.0,
        bones: Vec::new(),
        info: Some(info),
    })))
}

#[test]
fn a_cube_merges_its_childrens_effects_and_hooks_by_weight() {
    let hook = Ac2WeenieHook {
        hook_type: 1,
        hook_num: 2,
        data: 5,
        triggered: false,
        time: 0.4,
    };
    let cube = Ac2DbAnimator::Cube {
        sliders: Arc::from(vec![Ac2CubeSlider {
            id: 7,
            ticks: vec![0.0, 2.0, 4.0],
        }]),
        children: Arc::from(vec![
            Some(info_clip(Ac2AnimInfo {
                effects: vec![effect(9, 0.2, 1.0)],
                hooks: vec![hook],
                ..Ac2AnimInfo::default()
            })),
            Some(info_clip(Ac2AnimInfo {
                effects: vec![effect(9, 0.6, 3.0)],
                ..Ac2AnimInfo::default()
            })),
            None,
        ]),
    };
    let mut ctx = Ctx::default();
    ctx.sliders.insert(7, 1.0);
    let mut early = Ac2TrackTriggers::default();
    cube.triggers(&ctx, STATE, 0.3, false, &mut early);
    assert!(early.effects.is_empty(), "the merged effect is at 0.4");
    assert_eq!(early.hooks.len(), 1);
    assert_eq!(early.hooks[0].time, 0.2, "half of 0.4");
    let mut out = Ac2TrackTriggers::default();
    cube.triggers(&ctx, STATE, 1.5, false, &mut out);
    assert_eq!(out.effects.len(), 1, "within the cycle: 0.5");
    let merged = out.effects[0];
    assert_eq!(
        (merged.time, merged.intensity, merged.probability),
        (0.4, 2.0, 1.0)
    );
    assert!(out.triggered_effects.contains(&9));
}

#[test]
fn an_asset_adds_each_reached_trigger_once() {
    let hook = Ac2WeenieHook {
        hook_type: 1,
        hook_num: 2,
        data: 5,
        triggered: false,
        time: 0.5,
    };
    let asset = info_clip(Ac2AnimInfo {
        effects: vec![effect(9, 0.0, 1.0), effect(10, 0.75, 1.0)],
        hooks: vec![hook],
        ..Ac2AnimInfo::default()
    });
    let ctx = Ctx::default();
    let mut out = Ac2TrackTriggers::default();
    asset.triggers(&ctx, STATE, 0.5, false, &mut out);
    asset.triggers(&ctx, STATE, 0.6, false, &mut out);
    assert_eq!(out.effects, vec![effect(9, 0.0, 1.0)]);
    assert_eq!(out.hooks, vec![hook]);
}
