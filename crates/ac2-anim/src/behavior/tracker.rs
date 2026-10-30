//! `BehaviorTracker`: the behaviours playing in one element and mode. Each behaviour is a
//! track keyed by its id; each layer of the behaviour table has at most one active track (the
//! one ramping in or playing) and one non-active track (ramping out).

use super::hash::RetailHash;
use super::track::{Ac2BehaviorParams, Ac2BehaviorTrack};
use super::{Ac2AnimEvent, Env};
use crate::behavior_table::{
    Ac2ComboInfo, AC2_BEHAVIOR_ACTION, AC2_BEHAVIOR_DYNAMIC_FP_CAMERA, AC2_BEHAVIOR_HOLD,
};
use crate::dbanimator::Ac2TrackerState;

/// The locomotion behaviour `Update` watches (`mIsPlayingMove`).
pub const AC2_BEHAVIOR_MOVE: u32 = 0x4000_0001;
/// The idle behaviours a tracker restarts once nothing a weenie asked for is playing.
pub const AC2_BEHAVIOR_IDLE: u32 = 0x4000_0002;
pub const AC2_BEHAVIOR_IDLE_LAYER: u32 = 0x4000_0003;

/// `BehaviorTracker::StopBehavior` results (`0` stopped).
pub const AC2_STOP_LAYER_BUSY: u32 = 0x55;
pub const AC2_STOP_NOT_ACTIVE: u32 = 0x57;
pub const AC2_STOP_NOT_PLAYING: u32 = 0x58;
/// Behaviour callback errors.
/// `ProcessRampingDownParamFlags`'s `DeleteSelf` delay for `mDestroyOnCompletion` (seconds).
const DESTROY_ON_COMPLETION_DELAY: f64 = 3.0;
const ERROR_ALREADY_PLAYING: u32 = 0x56;
const ERROR_INVALID_COMBO: u32 = 0x54;
const ERROR_UNKNOWN_BEHAVIOR: u32 = 0x59;
const ERROR_NO_ANIMATION: u32 = 0x5a;
pub(crate) const ERROR_INTERRUPTED: u32 = 0x5b;

#[derive(Debug, Clone, PartialEq)]
pub struct Ac2BehaviorTracker {
    element: u32,
    mode: u32,
    pub(crate) tracks: RetailHash<Ac2BehaviorTrack>,
    active_by_layer: RetailHash<u32>,
    non_active_by_layer: RetailHash<u32>,
    weenie_behaviors_playing: u32,
    pub(crate) start_idle: bool,
    pub(crate) stop_idle: bool,
    is_playing_move: bool,
}

impl Ac2BehaviorTracker {
    /// `BehaviorTracker::BehaviorTracker`.
    pub(crate) fn new() -> Self {
        Self {
            element: 0,
            mode: 0,
            tracks: RetailHash::default(),
            active_by_layer: RetailHash::default(),
            non_active_by_layer: RetailHash::default(),
            weenie_behaviors_playing: 0,
            start_idle: false,
            stop_idle: false,
            is_playing_move: false,
        }
    }

    pub fn element(&self) -> u32 {
        self.element
    }

    pub fn mode(&self) -> u32 {
        self.mode
    }

    /// `mIsPlayingMove`: the last `Update` saw the move behaviour.
    pub fn is_playing_move(&self) -> bool {
        self.is_playing_move
    }

    /// The tracks in retail iteration order (`behaviour id`, track).
    pub fn tracks(&self) -> impl Iterator<Item = (u32, &Ac2BehaviorTrack)> {
        self.tracks.iter()
    }

    /// `BehaviorTracker::GetTrack`.
    pub fn track(&self, behavior: u32) -> Option<&Ac2BehaviorTrack> {
        self.tracks.get(behavior)
    }

    pub(crate) fn track_mut(&mut self, behavior: u32) -> Option<&mut Ac2BehaviorTrack> {
        self.tracks.get_mut(behavior)
    }

    /// `BehaviorTracker::IsDoingBehavior`.
    pub fn is_doing_behavior(&self, behavior: u32) -> bool {
        self.tracks.contains(behavior)
    }

    fn combo(&self, env: &Env, behavior: u32) -> Option<Ac2ComboInfo> {
        env.table.combo_info(self.element, self.mode, behavior)
    }

    /// `AnimationObj::GetComboCVel` through the table: 0 for an invalid combo.
    fn combo_cvel(&self, env: &Env, combo: Option<Ac2ComboInfo>) -> f32 {
        combo.map_or(0.0, |combo| env.source.combo_cvel(self.state(), combo))
    }

    /// The element and mode animators switch on.
    pub(crate) fn state(&self) -> Ac2TrackerState {
        Ac2TrackerState {
            element: self.element,
            mode: self.mode,
        }
    }

    /// `BehaviorTracker::SendAndClearCallback`.
    fn send_and_clear_callback(env: &mut Env, track: &mut Ac2BehaviorTrack, error: u32) {
        let context = track.params.context;
        if context == 0 {
            return;
        }
        env.events.push(if track.callback_with_release {
            Ac2AnimEvent::ReleaseDone { context, error }
        } else {
            Ac2AnimEvent::BehaviorDone { context, error }
        });
        track.params.context = 0;
    }

    /// The camera period of a behaviour: `-1` while held, else `1 / ctime velocity`.
    fn camera_period(track: &Ac2BehaviorTrack) -> f32 {
        if track.params.camera_hold {
            -1.0
        } else {
            (1.0 / f64::from(track.ctime_vel)) as f32
        }
    }

    /// `BehaviorTracker::ProcessParamFlags`: the
    /// children fade out, the FX starts, and for the player the actions lock and the camera
    /// follows the behaviour.
    fn process_param_flags(env: &mut Env, track: &Ac2BehaviorTrack) {
        if track.params.fade_children {
            env.events.push(Ac2AnimEvent::ChildrenOpacity {
                opacity: 0.0,
                ramp_time: track.ramp_time,
            });
        }
        if track.params.fx != 0 {
            env.events.push(Ac2AnimEvent::StartFx {
                fx: track.params.fx,
                intensity: 1.0,
                target: 0,
            });
        }
        if !env.source.is_the_player() {
            return;
        }
        if track.params.lock_actions {
            env.events.push(Ac2AnimEvent::LockActions);
        }
        if track.params.camera_parent {
            env.events.push(Ac2AnimEvent::CameraAnimParent {
                period: Self::camera_period(track),
                ramp_time: track.ramp_time,
            });
        }
        if track.params.camera_target {
            env.events.push(Ac2AnimEvent::CameraAnimTarget {
                period: Self::camera_period(track),
                ramp_time: track.ramp_time,
            });
        }
        if track.params.camera_restore {
            env.events.push(Ac2AnimEvent::RestoreCamera);
        }
        if track.params.camera_target_id != 0 {
            env.events.push(Ac2AnimEvent::TargetedCamera {
                target: track.params.camera_target_id,
                behavior: track.params.camera_behavior,
                period: (1.0 / f64::from(track.ctime_vel)) as f32,
            });
        }
    }

    /// `BehaviorTracker::ProcessRampingDownParamFlags`: the children fade back in, an
    /// object that destroys itself on completion is deleted (`DeleteSelf(0, 3.0)`), the FX target and clone are cleared, the player's camera is restored, the behaviour's FX
    /// stops and weenie behaviours count down, restarting idle once none play. The count is
    /// unsigned and wraps as retail's does.
    fn process_ramping_down_param_flags(&mut self, env: &mut Env, track: &Ac2BehaviorTrack) {
        if track.params.fade_children {
            env.events.push(Ac2AnimEvent::ChildrenOpacity {
                opacity: 1.0,
                ramp_time: track.ramp_time,
            });
        }
        if track.params.destroy_on_completion {
            env.events.push(Ac2AnimEvent::DeleteSelf {
                fade: 0.0,
                delay: DESTROY_ON_COMPLETION_DELAY,
            });
        }
        if track.params.target != 0 || track.params.vdesc_to_clone != 0 {
            env.events.push(Ac2AnimEvent::ClearFxTarget);
        }
        if env.source.is_the_player() && (track.params.camera_parent || track.params.camera_target)
        {
            env.events.push(Ac2AnimEvent::RestoreCamera);
        }
        if track.params.fx != 0 {
            env.events.push(Ac2AnimEvent::StopFx {
                fx: track.params.fx,
            });
        }
        if (track.params.context != 0 || track.is_hold || track.was_hold)
            && !track.params.leave_idle_alone
        {
            self.weenie_behaviors_playing = self.weenie_behaviors_playing.wrapping_sub(1);
        }
        if self.weenie_behaviors_playing == 0 {
            self.start_idle = true;
        }
    }

    fn count_weenie_behavior(&mut self, params: &Ac2BehaviorParams) {
        if params.context != 0 && !params.leave_idle_alone {
            self.weenie_behaviors_playing = self.weenie_behaviors_playing.wrapping_add(1);
            if self.weenie_behaviors_playing == 1 {
                self.stop_idle = true;
            }
        }
    }

    /// `BehaviorTracker::Init`: enter `element`/`mode`, carrying over the tracks of the tracker
    /// being left (those without a combo here ramp down and tell their callers), then start the
    /// table's initial behaviours at full weight.
    pub(crate) fn init(
        &mut self,
        env: &mut Env,
        element: u32,
        mode: u32,
        carried: Option<&mut RetailHash<Ac2BehaviorTrack>>,
    ) {
        self.element = element;
        self.mode = mode;
        if let Some(carried) = carried {
            for id in carried.keys() {
                let Some(old) = carried.get_mut(id) else {
                    continue;
                };
                let mut track = old.clone();
                let combo = env.table.combo_info(element, mode, track.params.behavior);
                match combo {
                    None => {
                        self.process_ramping_down_param_flags(env, old);
                        if old.params.lock_actions {
                            env.events.push(Ac2AnimEvent::UnlockActions);
                        }
                        Self::send_and_clear_callback(env, old, ERROR_INTERRUPTED);
                    }
                    Some(combo) => {
                        track.ctime_vel = env.source.combo_cvel(self.state(), combo);
                        if track.ctime_vel == 0.0 {
                            self.process_ramping_down_param_flags(env, old);
                            if old.params.lock_actions {
                                env.events.push(Ac2AnimEvent::UnlockActions);
                            }
                            Self::send_and_clear_callback(env, old, ERROR_INTERRUPTED);
                            continue;
                        }
                        old.params.context = 0;
                        track.anim_key = combo.anim_key;
                        track.layer = combo.layer;
                        let behavior = track.params.behavior;
                        let ramping_down = track.is_ramping_down;
                        let params = track.params.clone();
                        self.tracks.set(behavior, track);
                        if !ramping_down {
                            self.active_by_layer.set(combo.layer, behavior);
                            self.count_weenie_behavior(&params);
                        } else {
                            self.non_active_by_layer.set(combo.layer, behavior);
                        }
                    }
                }
            }
        }
        for &behavior in &env.table.init_behaviors {
            let info = env.table.behavior_info(behavior);
            let flags = env.template.behavior(behavior).unwrap_or(0);
            let Some(combo) = env.table.combo_info(element, mode, behavior) else {
                continue;
            };
            let cvel = env.source.combo_cvel(self.state(), combo);
            if cvel == 0.0 {
                continue;
            }
            let mut params = Ac2BehaviorParams::new(behavior);
            params.send_event = false;
            let mut track = Ac2BehaviorTrack::new(params);
            track.ctime_vel = cvel;
            track.weight = 1.0;
            track.is_action = flags & AC2_BEHAVIOR_ACTION != 0;
            track.is_hold = flags & AC2_BEHAVIOR_HOLD != 0;
            track.is_dynamic_fp_camera = flags & AC2_BEHAVIOR_DYNAMIC_FP_CAMERA != 0;
            track.layer = combo.layer;
            track.anim_key = combo.anim_key;
            track.ramp_time = info.ramp_time;
            self.active_by_layer.set(combo.layer, behavior);
            self.tracks.set(behavior, track);
        }
    }

    /// `BehaviorTracker::DoAutonomousBehavior`: a behaviour with default parameters and no
    /// network event.
    pub(crate) fn do_autonomous_behavior(&mut self, env: &mut Env, behavior: u32) -> bool {
        let mut params = Ac2BehaviorParams::new(behavior);
        params.send_event = false;
        self.do_behavior(env, &params)
    }

    /// `BehaviorTracker::DoBehavior`: start a behaviour on its combo's layer, ramping in over
    /// its ramp time and stopping what played on that layer. Refused (with a callback error)
    /// while the behaviour plays and is not ramping down, for a behaviour the `CharTemplate`
    /// lacks, without a combo in this element and mode, or without an animation.
    pub(crate) fn do_behavior(&mut self, env: &mut Env, request: &Ac2BehaviorParams) -> bool {
        let mut params = request.clone();
        let behavior = params.behavior;
        let refuse = |env: &mut Env, context: u32, error: u32| {
            if context != 0 {
                env.events
                    .push(Ac2AnimEvent::BehaviorDone { context, error });
            }
            false
        };
        if self
            .tracks
            .get(behavior)
            .is_some_and(|track| !track.is_ramping_down)
        {
            return refuse(env, params.context, ERROR_ALREADY_PLAYING);
        }
        let Some(flags) = env.template.behavior(behavior) else {
            return refuse(env, params.context, ERROR_UNKNOWN_BEHAVIOR);
        };
        let is_action = flags & AC2_BEHAVIOR_ACTION != 0;
        let is_hold = flags & AC2_BEHAVIOR_HOLD != 0;
        if !is_action && !is_hold && params.context != 0 {
            env.events.push(Ac2AnimEvent::BehaviorDone {
                context: params.context,
                error: 0,
            });
            params.context = 0;
        }
        let info = env.table.behavior_info(behavior);
        let Some(combo) = self.combo(env, behavior) else {
            return refuse(env, params.context, ERROR_INVALID_COMBO);
        };
        let cvel = env.source.combo_cvel(self.state(), combo);
        if cvel == 0.0 {
            return refuse(env, params.context, ERROR_NO_ANIMATION);
        }
        if let Some(old) = self.non_active_by_layer.remove(combo.layer) {
            if let Some(mut track) = self.tracks.remove(old) {
                Self::send_and_clear_callback(env, &mut track, 0);
                if !track.terminated_fx {
                    env.triggers.on_track_terminated(&mut track);
                }
            }
        }
        if info.effect != 0 {
            env.triggers.fire_and_forget(info.effect, false);
        }
        let mut track = Ac2BehaviorTrack::new(params);
        if info.ramp_time == 0.0 {
            track.weight = 1.0;
        } else {
            track.weight_vel = (1.0 / f64::from(info.ramp_time)) as f32;
        }
        track.ctime_vel = cvel;
        track.is_action = is_action;
        track.ramp_time = info.ramp_time;
        track.is_hold = is_hold;
        track.layer = combo.layer;
        track.anim_key = combo.anim_key;
        track.is_dynamic_fp_camera = flags & AC2_BEHAVIOR_DYNAMIC_FP_CAMERA != 0;
        Self::process_param_flags(env, &track);
        if track.is_hold {
            match env.source.hold_loop(self.state(), combo) {
                Some((start, end)) if start != 0.0 || end != 0.0 => {
                    track.loop_start = start;
                    track.loop_end = end;
                }
                _ => {
                    track.is_hold = false;
                    track.is_action = true;
                }
            }
        }
        self.count_weenie_behavior(&track.params);
        if let Some(&playing) = self.active_by_layer.get(combo.layer) {
            if playing != behavior {
                self.stop_behavior(env, playing);
            }
        }
        self.active_by_layer.set(combo.layer, behavior);
        self.tracks.set(behavior, track);
        true
    }

    /// `BehaviorTracker::CheckActiveBehaviorsForDynamicFPCamera`: some track is flagged
    /// `DYNAMIC_FP_CAMERA` (`ReleaseBehavior` clears the flag).
    pub fn check_active_behaviors_for_dynamic_fp_camera(&self) -> bool {
        self.tracks
            .iter()
            .any(|(_, track)| track.is_dynamic_fp_camera)
    }

    /// `BehaviorTracker::AddTriggers`: every track's reached triggers into the update.
    pub(crate) fn add_triggers(&mut self, triggers: &mut super::Ac2TriggerUpdate) {
        for behavior in self.tracks.keys() {
            if let Some(track) = self.tracks.get_mut(behavior) {
                triggers.add_track_triggers(track);
            }
        }
    }

    /// `BehaviorTracker::Destroy` (the tracker is being deleted): every track's weight drops to
    /// -1 and its triggers go to the update one last time; a track that faded the children
    /// brings them back (full opacity over a quarter second).
    pub(crate) fn destroy(
        &mut self,
        triggers: &mut super::Ac2TriggerUpdate,
        events: &mut Vec<Ac2AnimEvent>,
    ) {
        for behavior in self.tracks.keys() {
            if let Some(track) = self.tracks.get_mut(behavior) {
                track.weight = -1.0;
                triggers.add_track_triggers(track);
                if track.params.fade_children {
                    events.push(Ac2AnimEvent::ChildrenOpacity {
                        opacity: 1.0,
                        ramp_time: 0.25,
                    });
                }
            }
        }
    }

    /// `BehaviorTracker::StopBehavior`: ramp the track out over the behaviour's ramp time and
    /// move it to its layer's non-active slot.
    pub(crate) fn stop_behavior(&mut self, env: &mut Env, behavior: u32) -> u32 {
        if !self.tracks.contains(behavior) {
            return AC2_STOP_NOT_PLAYING;
        }
        let layer = self
            .combo(env, behavior)
            .map_or(u32::MAX, |combo| combo.layer);
        let ramp = env.table.behavior_info(behavior).ramp_time;
        let Some(mut track) = self.tracks.get(behavior).cloned() else {
            return AC2_STOP_NOT_PLAYING;
        };
        if ramp == 0.0 {
            track.weight = -1.0;
            track.weight_vel = 0.0;
        } else {
            track.weight_vel = (-1.0 / f64::from(ramp)) as f32;
        }
        self.process_ramping_down_param_flags(env, &track);
        if track.params.lock_actions {
            env.events.push(Ac2AnimEvent::UnlockActions);
        }
        track.stopped_action = true;
        track.is_ramping_down = true;
        Self::send_and_clear_callback(env, &mut track, ERROR_INTERRUPTED);
        self.tracks.set(behavior, track);
        match self.non_active_by_layer.get(layer).copied() {
            None => {
                if let Some(&active) = self.active_by_layer.get(layer) {
                    if active != behavior {
                        self.non_active_by_layer.set(layer, behavior);
                        return AC2_STOP_NOT_ACTIVE;
                    }
                }
                self.active_by_layer.remove(layer);
                self.non_active_by_layer.set(layer, behavior);
            }
            Some(ramping) if ramping != behavior => {
                self.non_active_by_layer.set(layer, behavior);
                return AC2_STOP_LAYER_BUSY;
            }
            Some(_) => {}
        }
        0
    }

    /// `BehaviorTracker::StopAllWeenieBehaviors`: stop every track a weenie context waits on.
    pub(crate) fn stop_all_weenie_behaviors(&mut self, env: &mut Env) {
        let stop: Vec<u32> = self
            .tracks
            .iter()
            .filter(|(_, track)| track.params.context != 0)
            .map(|(_, track)| track.params.behavior)
            .collect();
        for behavior in stop {
            self.stop_behavior(env, behavior);
        }
    }

    /// `BehaviorTracker::StopMoveToCancelsBehaviors`: stop the tracks a move-to cancels (the
    /// `List::insert` at the head plays them back in reverse iteration order).
    pub(crate) fn stop_move_to_cancels_behaviors(&mut self, env: &mut Env) {
        let mut stop: Vec<u32> = self
            .tracks
            .iter()
            .filter(|(_, track)| track.params.move_to_cancels)
            .map(|(_, track)| track.params.behavior)
            .collect();
        stop.reverse();
        for behavior in stop {
            self.stop_behavior(env, behavior);
        }
    }

    /// `BehaviorTracker::StopLayer`.
    pub(crate) fn stop_layer(&mut self, env: &mut Env, layer: u32) -> u32 {
        if self.non_active_by_layer.contains(layer) {
            return AC2_STOP_LAYER_BUSY;
        }
        match self.active_by_layer.get(layer).copied() {
            Some(active) => self.stop_behavior(env, active),
            None => AC2_STOP_NOT_ACTIVE,
        }
    }

    /// `BehaviorTracker::ReleaseBehavior`: a hold becomes an action that plays out from its
    /// loop.
    pub(crate) fn release_behavior(&mut self, behavior: u32) -> bool {
        let Some(track) = self.tracks.get_mut(behavior) else {
            return false;
        };
        if !track.is_hold {
            return false;
        }
        track.is_action = true;
        track.is_hold = false;
        track.is_dynamic_fp_camera = false;
        track.was_hold = true;
        let (start, end) = (f64::from(track.loop_start), f64::from(track.loop_end));
        let mut ctime = f64::from(track.ctime);
        if track.loop_end != 0.0 && ctime > end {
            loop {
                ctime = ctime - end + start;
                if !(ctime > end) {
                    break;
                }
            }
        }
        track.ctime = ctime as f32;
        true
    }

    /// `BehaviorTracker::Update`: advance every track, finish holds and early callbacks, move
    /// tracks that began ramping down to their layer's non-active slot, drop tracks whose
    /// weight reached zero, and ask for idle when nothing else plays.
    pub(crate) fn update(&mut self, env: &mut Env, quantum: f32) {
        self.is_playing_move = false;
        let mut killed = Vec::new();
        for behavior in self.tracks.keys() {
            if behavior == AC2_BEHAVIOR_MOVE {
                self.is_playing_move = true;
            }
            let combo = self.combo(env, behavior);
            let Some(mut track) = self.tracks.get(behavior).cloned() else {
                continue;
            };
            track.ctime_vel = self.combo_cvel(env, combo);
            track.update(quantum);
            if track.is_hold && track.params.hold_cycles <= track.repeats {
                Self::send_and_clear_callback(env, &mut track, 0);
            }
            if track.is_ramping_down {
                if track.params.early_callback {
                    Self::send_and_clear_callback(env, &mut track, 0);
                }
                if track.started_ramping_down {
                    self.process_ramping_down_param_flags(env, &track);
                    track.started_ramping_down = false;
                    match combo {
                        None => track.weight = -1.0,
                        Some(combo) => {
                            if self.active_by_layer.get(combo.layer) == Some(&behavior) {
                                self.active_by_layer.remove(combo.layer);
                                self.non_active_by_layer.set(combo.layer, behavior);
                            }
                        }
                    }
                }
            }
            if f64::from(track.weight) <= 0.0 {
                if track.params.lock_actions {
                    env.events.push(Ac2AnimEvent::UnlockActions);
                }
                killed.push(track.params.behavior);
                track.weight = -1.0;
                track.weight_vel = 0.0;
                env.triggers.add_track_triggers(&mut track);
                env.triggers.on_track_terminated(&mut track);
                track.terminated_fx = true;
                if let Some(combo) = combo {
                    self.non_active_by_layer.remove(combo.layer);
                    Self::send_and_clear_callback(env, &mut track, 0);
                }
            }
            self.tracks.set(behavior, track);
        }
        for behavior in killed {
            self.tracks.remove(behavior);
        }
        if self.weenie_behaviors_playing == 0 && !self.tracks.contains(AC2_BEHAVIOR_IDLE) {
            self.start_idle = true;
        }
    }
}
