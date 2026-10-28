//! Synthetic behaviour tables (no game data).

use std::collections::HashMap;
use std::sync::Arc;

use super::hash::RetailHash;
use super::*;
use crate::behavior_table::{Ac2BehaviorInfo, Ac2ModeTransition, AC2_BEHAVIOR_ACTION};

const GROUND: u32 = 0x4000_0001;
const AIR: u32 = 0x4000_0002;
const WATER: u32 = 0x4000_0003;
const PEACE: u32 = 0x4000_0001;
const MOVE: u32 = AC2_BEHAVIOR_MOVE;
const IDLE: u32 = AC2_BEHAVIOR_IDLE;
const WAVE: u32 = 0x4000_0050;

/// Every anim key plays at `cvel` cycles per second; key 99 has no animation.
struct Source {
    cvel: f32,
}

impl Ac2AnimationSource for Source {
    fn combo_cvel(&self, _tracker: crate::Ac2TrackerState, combo: Ac2ComboInfo) -> f32 {
        if combo.anim_key == 99 {
            0.0
        } else {
            self.cvel
        }
    }

    fn hold_loop(
        &self,
        _tracker: crate::Ac2TrackerState,
        _combo: Ac2ComboInfo,
    ) -> Option<(f32, f32)> {
        None
    }
}

const SOURCE: Source = Source { cvel: 1.0 };

fn key(src: (u32, u32), dst: (u32, u32)) -> Ac2ModeTransition {
    Ac2ModeTransition {
        src_element: src.0,
        src_mode: src.1,
        dst_element: dst.0,
        dst_mode: dst.1,
    }
}

/// Ground and air peace modes linked both ways (0.5 s ramp), water reachable from nowhere;
/// idle, move and wave combos on layers 0, 1 and 2 in both modes.
fn table() -> Arc<Ac2BehaviorTable> {
    let mut combos = HashMap::new();
    for element in [GROUND, AIR] {
        combos.insert(
            (element, PEACE, IDLE),
            Ac2ComboInfo {
                anim_key: 1,
                layer: 0,
            },
        );
        combos.insert(
            (element, PEACE, MOVE),
            Ac2ComboInfo {
                anim_key: 2,
                layer: 1,
            },
        );
        combos.insert(
            (element, PEACE, WAVE),
            Ac2ComboInfo {
                anim_key: 3,
                layer: 2,
            },
        );
    }
    let ground = (GROUND, PEACE);
    let air = (AIR, PEACE);
    let water = (WATER, PEACE);
    Arc::new(Ac2BehaviorTable {
        ramp_times: HashMap::from([(key(ground, air), 0.5), (key(air, ground), 0.0)]),
        combos,
        transitions: HashMap::new(),
        layers: vec![0, 1, 2],
        tweak_layers: Vec::new(),
        mode_transitions: HashMap::from([(ground, vec![air]), (air, vec![ground])]),
        modes: HashMap::from([(ground, 0), (air, 0), (water, 0)]),
        behavior_info: HashMap::from([
            (
                MOVE,
                Ac2BehaviorInfo {
                    ramp_time: 0.2,
                    effect: 0,
                },
            ),
            (
                WAVE,
                Ac2BehaviorInfo {
                    ramp_time: 0.25,
                    effect: 0,
                },
            ),
        ]),
        init_behaviors: vec![IDLE],
        init_mode: PEACE,
    })
}

fn template() -> Arc<Ac2CharTemplate> {
    Arc::new(Ac2CharTemplate {
        behaviors: HashMap::from([
            (IDLE, 0),
            (AC2_BEHAVIOR_IDLE_LAYER, 0),
            (MOVE, 0),
            (WAVE, AC2_BEHAVIOR_ACTION),
        ]),
        sliders: HashMap::new(),
        modes: HashMap::new(),
        default_element: GROUND,
        default_mode: PEACE,
        user_controlled_sliders: Vec::new(),
    })
}

fn tracker() -> Ac2ModeTracker {
    let mut tracker = Ac2ModeTracker::new(template());
    assert!(tracker.init(Some(table()), &SOURCE));
    tracker
}

#[test]
fn the_hash_links_new_keys_at_the_chain_head_and_walks_buckets_in_order() {
    let mut hash = RetailHash::default();
    hash.add(5, 'a');
    hash.add(3, 'b');
    hash.add(5 + 47, 'c');
    assert!(!hash.add(5, 'z'));
    hash.set(3, 'd');
    assert_eq!(hash.keys(), vec![3, 52, 5]);
    assert_eq!(hash.get(3), Some(&'d'));
    hash.remove(52);
    assert_eq!(hash.keys(), vec![3, 5]);
}

#[test]
fn an_element_change_waits_for_the_mode_transition() {
    let mut tracker = tracker();
    assert_eq!(tracker.element(), GROUND);
    assert!(tracker.is_doing_behavior(IDLE));
    assert!(tracker.do_element(AIR, &SOURCE));
    assert_eq!(tracker.desired_element(), AIR);
    assert_eq!(
        tracker.element(),
        GROUND,
        "still ramping toward the air mode"
    );
    tracker.update(0.25, &SOURCE);
    assert_eq!(tracker.element(), GROUND);
    let (from, to, t) = tracker.trackers();
    assert!(from.is_some() && to.is_some());
    assert_eq!(t, 0.5);
    tracker.update(0.25, &SOURCE);
    assert_eq!(tracker.element(), AIR);
    assert!(tracker.trackers().1.is_none());
    assert!(tracker.is_doing_behavior(IDLE), "idle carried into the air");
    // A zero ramp completes on the next update.
    assert!(tracker.do_element(GROUND, &SOURCE));
    tracker.update(0.01, &SOURCE);
    assert_eq!(tracker.element(), GROUND);
}

#[test]
fn an_element_without_a_mode_path_is_refused() {
    let mut tracker = tracker();
    assert!(!tracker.do_element(WATER, &SOURCE));
    assert_eq!(tracker.desired_element(), GROUND);
    assert_eq!(tracker.element(), GROUND);
}

#[test]
fn without_a_table_nothing_animates_and_the_element_never_changes() {
    let mut tracker = Ac2ModeTracker::new(template());
    assert!(tracker.init(None, &SOURCE));
    assert!(tracker.do_element(AIR, &SOURCE));
    assert_eq!(tracker.element(), GROUND);
    assert!(!tracker.is_doing_behavior(IDLE));
}

#[test]
fn a_behaviour_ramps_in_and_out_over_its_ramp_time() {
    let mut tracker = tracker();
    assert!(tracker.do_autonomous_behavior(MOVE, &SOURCE));
    assert!(
        !tracker.do_autonomous_behavior(MOVE, &SOURCE),
        "already playing"
    );
    let weight = |t: &Ac2ModeTracker| t.trackers().0.unwrap().track(MOVE).map(|t| t.weight);
    assert_eq!(weight(&tracker), Some(0.0));
    tracker.update(0.1, &SOURCE);
    assert!((weight(&tracker).unwrap() - 0.5).abs() < 1e-6);
    tracker.update(0.15, &SOURCE);
    tracker.update(0.1, &SOURCE);
    assert_eq!(weight(&tracker), Some(1.0), "clamped at full weight");
    assert_eq!(tracker.stop_behavior(MOVE, &SOURCE), 0);
    tracker.update(0.1, &SOURCE);
    assert!((weight(&tracker).unwrap() - 0.5).abs() < 1e-6);
    tracker.update(0.1, &SOURCE);
    assert_eq!(weight(&tracker), None, "removed at zero weight");
    assert_eq!(tracker.stop_behavior(MOVE, &SOURCE), AC2_STOP_NOT_PLAYING);
}

#[test]
fn an_action_ramps_out_at_its_end_and_answers_its_context() {
    let mut tracker = tracker();
    let mut params = Ac2BehaviorParams::new(WAVE);
    params.context = 77;
    assert!(tracker.do_behavior(&params, &SOURCE));
    assert!(tracker.take_events().is_empty());
    let track = |t: &Ac2ModeTracker| t.trackers().0.unwrap().track(WAVE).cloned();
    tracker.update(0.125, &SOURCE);
    assert!((track(&tracker).unwrap().weight - 0.5).abs() < 1e-6);
    tracker.update(0.5, &SOURCE);
    assert_eq!(track(&tracker).unwrap().weight, 1.0);
    tracker.update(0.25, &SOURCE);
    let wave = track(&tracker).unwrap();
    assert!(wave.is_ramping_down && (wave.weight - 0.5).abs() < 1e-5);
    tracker.update(0.2, &SOURCE);
    assert!(track(&tracker).is_none());
    assert_eq!(
        tracker.take_events(),
        vec![Ac2AnimEvent::BehaviorDone {
            context: 77,
            error: 0
        }]
    );
}

#[test]
fn a_behaviour_without_animation_is_refused_with_its_error() {
    let mut table = (*table()).clone();
    table.combos.insert(
        (GROUND, PEACE, WAVE),
        Ac2ComboInfo {
            anim_key: 99,
            layer: 2,
        },
    );
    let mut tracker = Ac2ModeTracker::new(template());
    tracker.init(Some(Arc::new(table)), &SOURCE);
    let mut params = Ac2BehaviorParams::new(WAVE);
    params.context = 5;
    assert!(!tracker.do_behavior(&params, &SOURCE));
    assert_eq!(
        tracker.take_events(),
        vec![Ac2AnimEvent::BehaviorDone {
            context: 5,
            error: 0x5a
        }]
    );
}

#[test]
fn sliders_move_at_their_rate_and_stop_on_target() {
    let mut sliders = Ac2SliderTracker::new();
    assert_eq!(sliders.value(0x4000_0015), 1.0);
    assert!(sliders.do_slider(0, 0x4000_0004, 2.0, 0.5));
    assert_eq!(sliders.velocity(0x4000_0004), 4.0);
    sliders.update(0.25);
    assert_eq!(sliders.value(0x4000_0004), 1.0);
    sliders.update(0.3);
    assert_eq!(sliders.value(0x4000_0004), 2.0);
    assert_eq!(sliders.velocity(0x4000_0004), 0.0);
    sliders.do_slider(0, 0x4000_0004, -1.0, 0.0);
    assert_eq!(sliders.value(0x4000_0004), -1.0);
}

#[test]
fn a_slider_interpolator_ramps_for_its_length_then_rests_at_its_end() {
    let mut sliders = Ac2SliderTracker::new();
    let slider = 0x4000_0030;
    assert_eq!(
        sliders.interpolator(slider),
        Ac2SliderInterpolator::default()
    );
    sliders.activate_interpolator(slider, 0.0, 2.0, 1.0);
    // The tracker must be active (a moving slider) for the interpolators to advance.
    sliders.do_slider(0, 0x4000_0004, 1.0, 10.0);
    sliders.update(0.25);
    let ramp = sliders.interpolator(slider);
    assert!(ramp.active && ramp.time == 0.25 && ramp.current == 0.25);
    sliders.update(1.0);
    let done = sliders.interpolator(slider);
    assert!(!done.active);
    assert_eq!(
        (done.time, done.start, done.end, done.current),
        (0.0, 2.0, 0.0, 0.0)
    );
}

#[test]
fn an_inactive_interpolator_keeps_counting_time_while_the_tracker_runs() {
    let mut sliders = Ac2SliderTracker::new();
    sliders.activate_interpolator(0x4000_0031, 1.0, 1.0, 1.0);
    sliders.do_slider(0, 0x4000_0004, 1.0, 10.0);
    sliders.update(0.5);
    sliders.update(0.5);
    // Active: 1.0 is not past its length yet.
    assert!(sliders.interpolator(0x4000_0031).active);
    sliders.update(0.5);
    assert!(!sliders.interpolator(0x4000_0031).active);
    sliders.update(0.5);
    assert_eq!(sliders.interpolator(0x4000_0031).time, 0.5);
}

/// `ModeTracker::CheckForDynamicFPCamera`: a flagged active behaviour, until it is released.
#[test]
fn a_dynamic_first_person_camera_behaviour_is_found_while_it_plays() {
    let template = Ac2CharTemplate {
        behaviors: HashMap::from([
            (IDLE, 0),
            (AC2_BEHAVIOR_IDLE_LAYER, 0),
            (
                WAVE,
                AC2_BEHAVIOR_ACTION | crate::behavior_table::AC2_BEHAVIOR_DYNAMIC_FP_CAMERA,
            ),
        ]),
        sliders: HashMap::new(),
        modes: HashMap::new(),
        default_element: GROUND,
        default_mode: PEACE,
        user_controlled_sliders: Vec::new(),
    };
    let mut tracker = Ac2ModeTracker::new(Arc::new(template));
    assert!(tracker.init(Some(table()), &SOURCE));
    assert!(!tracker.check_for_dynamic_fp_camera());
    assert!(tracker.do_behavior(&Ac2BehaviorParams::new(WAVE), &SOURCE));
    assert!(tracker.check_for_dynamic_fp_camera());
}

/// `Source` that is the player's object (the camera and action lock apply to it only).
struct PlayerSource;

impl Ac2AnimationSource for PlayerSource {
    fn combo_cvel(&self, tracker: crate::Ac2TrackerState, combo: Ac2ComboInfo) -> f32 {
        SOURCE.combo_cvel(tracker, combo)
    }
    fn hold_loop(
        &self,
        tracker: crate::Ac2TrackerState,
        combo: Ac2ComboInfo,
    ) -> Option<(f32, f32)> {
        SOURCE.hold_loop(tracker, combo)
    }
    fn is_the_player(&self) -> bool {
        true
    }
}

/// `BehaviorTracker::ProcessParamFlags` and `ProcessRampingDownParamFlags`: the children fade
/// out and back, and the player's camera follows the behaviour and is restored.
#[test]
fn a_behaviour_fades_the_children_and_drives_the_players_camera() {
    let mut params = Ac2BehaviorParams::new(WAVE);
    params.fade_children = true;
    params.camera_parent = true;
    params.camera_hold = true;
    params.camera_restore = true;
    params.camera_target_id = 0x77;
    params.camera_behavior = 0x4000_0099;
    let mut player = tracker();
    assert!(player.do_behavior(&params, &PlayerSource));
    assert_eq!(
        player.take_events(),
        vec![
            Ac2AnimEvent::ChildrenOpacity {
                opacity: 0.0,
                ramp_time: 0.25
            },
            Ac2AnimEvent::CameraAnimParent {
                period: -1.0,
                ramp_time: 0.25
            },
            Ac2AnimEvent::RestoreCamera,
            Ac2AnimEvent::TargetedCamera {
                target: 0x77,
                behavior: 0x4000_0099,
                period: 1.0
            },
        ]
    );
    assert_eq!(player.stop_behavior(WAVE, &PlayerSource), 0);
    let events = player.take_events();
    assert!(events.contains(&Ac2AnimEvent::ChildrenOpacity {
        opacity: 1.0,
        ramp_time: 0.25
    }));
    assert!(events.contains(&Ac2AnimEvent::RestoreCamera));
    // Another object's behaviour fades its children but never touches the camera.
    let mut other = tracker();
    assert!(other.do_behavior(&params, &SOURCE));
    assert_eq!(
        other.take_events(),
        vec![Ac2AnimEvent::ChildrenOpacity {
            opacity: 0.0,
            ramp_time: 0.25
        }]
    );
}
