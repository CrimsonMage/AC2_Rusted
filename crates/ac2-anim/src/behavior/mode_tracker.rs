//! `ModeTracker`: the element and mode an object animates in. A change of element or mode is a
//! transition: a second `BehaviorTracker` for the next mode along the table's mode path ramps
//! in over the transition's ramp time (or its transition behaviour's period) while the current
//! one ramps out, and only when it completes does the object's element or mode change.

use std::sync::Arc;

use super::track::Ac2BehaviorParams;
use super::tracker::{Ac2BehaviorTracker, AC2_BEHAVIOR_IDLE, AC2_BEHAVIOR_IDLE_LAYER};
use super::{Ac2AnimEvent, Ac2AnimationSource, Env};
use crate::behavior_table::{Ac2BehaviorTable, Ac2CharTemplate, Ac2ModeTransition};

/// The default mode `DoElement` falls back to when the desired mode has no path.
const MODE_DEFAULT: u32 = 0x4000_0001;
/// `SetMode` interrupts a pending `DoMode` with this error.
const ERROR_MODE_RESET: u32 = 0x36;
const ERROR_NO_PATH: u32 = 0x5a;
const ERROR_INTERRUPTED: u32 = super::tracker::ERROR_INTERRUPTED;

#[derive(Debug, Clone)]
pub struct Ac2ModeTracker {
    template: Arc<Ac2CharTemplate>,
    table: Option<Arc<Ac2BehaviorTable>>,
    from: Option<Ac2BehaviorTracker>,
    to: Option<Ac2BehaviorTracker>,
    transition_ctime: f32,
    transition_cvel: f32,
    desired_element: u32,
    desired_mode: u32,
    can_animate: bool,
    context: u32,
    events: Vec<Ac2AnimEvent>,
}

/// Splits the tracker into its environment and its two behaviour trackers.
macro_rules! env {
    ($self:ident, $source:expr) => {
        match $self.table.as_deref() {
            Some(table) => Some(Env {
                table,
                template: &$self.template,
                source: $source,
                events: &mut $self.events,
            }),
            None => None,
        }
    };
}

impl Ac2ModeTracker {
    /// `ModeTracker::ModeTracker`: the `CharTemplate` default element and mode, no table.
    pub fn new(template: Arc<Ac2CharTemplate>) -> Self {
        Self {
            desired_element: template.default_element,
            desired_mode: template.default_mode,
            template,
            table: None,
            from: None,
            to: None,
            transition_ctime: 0.0,
            transition_cvel: 0.0,
            can_animate: false,
            context: 0,
            events: Vec::new(),
        }
    }

    /// `ModeTracker::Init`: animate with `table` (none leaves the object unanimated) from the
    /// table's initial mode in the desired element.
    pub fn init(
        &mut self,
        table: Option<Arc<Ac2BehaviorTable>>,
        source: &dyn Ac2AnimationSource,
    ) -> bool {
        let Some(table) = table else {
            self.can_animate = false;
            return true;
        };
        self.can_animate = true;
        self.desired_mode = table.init_mode;
        self.table = Some(table);
        let mut from = Ac2BehaviorTracker::new();
        let (element, mode) = (self.desired_element, self.desired_mode);
        if let Some(mut env) = env!(self, source) {
            from.init(&mut env, element, mode, None);
        }
        self.from = Some(from);
        true
    }

    /// The events of the calls since the last take, in retail callback order.
    pub fn take_events(&mut self) -> Vec<Ac2AnimEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn can_animate(&self) -> bool {
        self.can_animate
    }

    /// `ModeTracker::GetElement`: the element of the tracker being left, the desired element
    /// when not animated.
    pub fn element(&self) -> u32 {
        match (&self.from, self.can_animate) {
            (Some(from), true) => from.element(),
            _ => self.desired_element,
        }
    }

    /// `ModeTracker::GetModeState` mode.
    pub fn mode(&self) -> u32 {
        match (&self.from, self.can_animate) {
            (Some(from), true) => from.mode(),
            _ => self.desired_mode,
        }
    }

    pub fn desired_element(&self) -> u32 {
        self.desired_element
    }

    pub fn desired_mode(&self) -> u32 {
        self.desired_mode
    }

    /// The tracker being left (the only one when not transitioning) and the one ramping in, with
    /// the transition's progress (`CPartGroup::DoAnimation` weights them `1 - t` and `t`).
    pub fn trackers(
        &self,
    ) -> (
        Option<&Ac2BehaviorTracker>,
        Option<&Ac2BehaviorTracker>,
        f32,
    ) {
        (self.from.as_ref(), self.to.as_ref(), self.transition_ctime)
    }

    /// Every track of the outgoing then the incoming tracker, in retail iteration order, with
    /// its tracker's element and mode (`CPartArray::AddAnimation`'s trigger pass).
    pub fn visit_tracks_mut(
        &mut self,
        mut visit: impl FnMut(crate::Ac2TrackerState, &mut super::Ac2BehaviorTrack),
    ) {
        for tracker in [self.from.as_mut(), self.to.as_mut()].into_iter().flatten() {
            let state = tracker.state();
            for behavior in tracker.tracks.keys() {
                if let Some(track) = tracker.tracks.get_mut(behavior) {
                    visit(state, track);
                }
            }
        }
    }

    /// The `CharTemplate` the tracker was made with.
    pub fn template(&self) -> &Ac2CharTemplate {
        &self.template
    }

    /// `ModeTracker::GetNetworkBehaviorParams` -> `BehaviorTracker::GetNetworkBehaviorParams`:
    /// the parameters of the incoming tracker's (else the outgoing one's) tracks that are
    /// weighted in, not ramping down and sent to the weenie, in retail iteration order.
    pub fn network_behavior_params(&self) -> Vec<Ac2BehaviorParams> {
        let Some(tracker) = self.to.as_ref().or(self.from.as_ref()) else {
            return Vec::new();
        };
        tracker
            .tracks
            .iter()
            .filter(|(_, track)| {
                track.weight > 0.0 && !track.is_ramping_down && track.params.send_event
            })
            .map(|(_, track)| track.params.clone())
            .collect()
    }

    /// `ModeTracker::IsDoingBehavior`: the incoming tracker's behaviours during a transition;
    /// none while a transition with no incoming tracker is still timed.
    pub fn is_doing_behavior(&self, behavior: u32) -> bool {
        if !self.can_animate {
            return false;
        }
        if let Some(to) = &self.to {
            return to.is_doing_behavior(behavior);
        }
        match &self.from {
            Some(from) if self.transition_cvel == 0.0 => from.is_doing_behavior(behavior),
            _ => false,
        }
    }

    /// `ModeTracker::SetMode`: switch mode at once, carrying the current tracks over.
    pub fn set_mode(&mut self, mode: u32, source: &dyn Ac2AnimationSource) -> bool {
        if self.context != 0 {
            self.events.push(Ac2AnimEvent::ModeDone {
                context: self.context,
                error: ERROR_MODE_RESET,
            });
            self.context = 0;
        }
        self.transition_ctime = 0.0;
        self.transition_cvel = 0.0;
        self.desired_mode = mode;
        let Some(mut from) = self.from.take() else {
            return true;
        };
        if self.to.is_none() && from.mode() == mode {
            self.from = Some(from);
            return true;
        }
        let mut to = self.to.take();
        let mut next = Ac2BehaviorTracker::new();
        if let Some(mut env) = env!(self, source) {
            if let Some(to) = to.as_mut() {
                to.stop_all_weenie_behaviors(&mut env);
            }
            let element = from.element();
            next.init(&mut env, element, mode, Some(&mut from.tracks));
            from.stop_all_weenie_behaviors(&mut env);
        }
        self.from = Some(next);
        true
    }

    /// `ModeTracker::DoElement`: move to `element` in the desired mode, else in the default mode;
    /// false when neither has a mode path.
    pub fn do_element(&mut self, element: u32, source: &dyn Ac2AnimationSource) -> bool {
        if self.from.is_none() || self.table.is_none() || self.desired_element == element {
            return true;
        }
        let mode = self.desired_mode;
        self.start_transitioning(element, mode, source)
            || self.start_transitioning(element, MODE_DEFAULT, source)
    }

    /// `ModeTracker::DoMode`: transition to `mode`, telling `context` when it is reached.
    pub fn do_mode(&mut self, context: u32, mode: u32, source: &dyn Ac2AnimationSource) -> bool {
        if !self.can_animate {
            self.desired_mode = mode;
            if context != 0 {
                self.events
                    .push(Ac2AnimEvent::ModeDone { context, error: 0 });
            }
            return true;
        }
        if self.desired_mode == mode {
            if context != 0 {
                if self.context != 0 {
                    self.events.push(Ac2AnimEvent::ModeDone {
                        context: self.context,
                        error: 0,
                    });
                    self.context = context;
                    return true;
                }
                self.events
                    .push(Ac2AnimEvent::ModeDone { context, error: 0 });
            }
            return true;
        }
        if self.context != 0 {
            self.events.push(Ac2AnimEvent::ModeDone {
                context: self.context,
                error: ERROR_INTERRUPTED,
            });
        }
        self.context = context;
        let element = self.desired_element;
        if !self.start_transitioning(element, mode, source) {
            if self.context != 0 {
                self.events.push(Ac2AnimEvent::ModeDone {
                    context: self.context,
                    error: ERROR_NO_PATH,
                });
                self.context = 0;
            }
            return false;
        }
        true
    }

    /// `ModeTracker::StartTransitioning`: finish a transition in progress, then start toward
    /// `element`/`mode` along the mode path, with the transition behaviour and its effects.
    fn start_transitioning(
        &mut self,
        element: u32,
        mode: u32,
        source: &dyn Ac2AnimationSource,
    ) -> bool {
        let Some(table) = self.table.clone() else {
            return false;
        };
        if self.from.is_none() {
            return false;
        }
        if let Some(to) = self.to.take() {
            self.from = Some(to);
            self.transition_ctime = 0.0;
            self.transition_cvel = 0.0;
            if self.context != 0 {
                self.events.push(Ac2AnimEvent::ModeDone {
                    context: self.context,
                    error: 0,
                });
                self.context = 0;
            }
        }
        let Some(from) = self.from.as_ref() else {
            return false;
        };
        let mut key = Ac2ModeTransition {
            src_element: from.element(),
            src_mode: from.mode(),
            dst_element: element,
            dst_mode: mode,
        };
        if key.src_element == element && key.src_mode == mode {
            self.desired_element = element;
            self.desired_mode = mode;
            return true;
        }
        if !table.next_transition_along_mode_path(&mut key) {
            return false;
        }
        self.desired_element = element;
        self.desired_mode = mode;
        let ramp = table.mode_transition_ramp_time(&key);
        self.transition_cvel = (1.0 / f64::from(ramp)) as f32;
        let mut to = Ac2BehaviorTracker::new();
        let mut env = Env {
            table: &table,
            template: &self.template,
            source,
            events: &mut self.events,
        };
        let carried = self.from.as_mut().map(|from| &mut from.tracks);
        to.init(&mut env, key.dst_element, key.dst_mode, carried);
        self.to = Some(to);
        if let Some(info) = table.transition_info(&key) {
            if info.behavior != 0 {
                self.do_behavior(&Ac2BehaviorParams::new(info.behavior), source);
                let combo = table.combo_info(key.dst_element, key.dst_mode, info.behavior);
                // `GetComboCVel` is asked of the outgoing tracker.
                let state = self.from.as_ref().map(Ac2BehaviorTracker::state);
                let cvel = match (combo, state) {
                    (Some(combo), Some(state)) => source.combo_cvel(state, combo),
                    _ => 0.0,
                };
                if cvel != 0.0 {
                    self.transition_cvel = cvel;
                }
            }
            if info.effect != 0 {
                self.events.push(Ac2AnimEvent::StartFx {
                    fx: info.effect,
                    intensity: 1.0,
                });
            }
            if info.stop_effect != 0 {
                self.events.push(Ac2AnimEvent::StopFx {
                    fx: info.stop_effect,
                });
            }
        }
        true
    }

    /// `ModeTracker::DoBehavior`: on both trackers during a transition (the outgoing copy
    /// without its context), stopping idle where a weenie behaviour starts.
    pub fn do_behavior(
        &mut self,
        params: &Ac2BehaviorParams,
        source: &dyn Ac2AnimationSource,
    ) -> bool {
        if !self.can_animate {
            self.events.push(Ac2AnimEvent::BehaviorDone {
                context: params.context,
                error: 0,
            });
            return true;
        }
        let Some(mut env) = env!(self, source) else {
            return false;
        };
        let (Some(from), to) = (self.from.as_mut(), self.to.as_mut()) else {
            return false;
        };
        let stop_idle = |tracker: &mut Ac2BehaviorTracker, env: &mut Env| {
            if tracker.stop_idle {
                tracker.stop_behavior(env, AC2_BEHAVIOR_IDLE);
                tracker.stop_behavior(env, AC2_BEHAVIOR_IDLE_LAYER);
                tracker.stop_idle = false;
            }
        };
        match to {
            Some(to) => {
                let mut copy = params.clone();
                copy.context = 0;
                from.do_behavior(&mut env, &copy);
                stop_idle(from, &mut env);
                let started = to.do_behavior(&mut env, params);
                stop_idle(to, &mut env);
                started
            }
            None => {
                let started = from.do_behavior(&mut env, params);
                stop_idle(from, &mut env);
                started
            }
        }
    }

    /// `ModeTracker::DoAutonomousBehavior`.
    pub fn do_autonomous_behavior(
        &mut self,
        behavior: u32,
        source: &dyn Ac2AnimationSource,
    ) -> bool {
        if !self.can_animate {
            return true;
        }
        let Some(mut env) = env!(self, source) else {
            return false;
        };
        let Some(from) = self.from.as_mut() else {
            return false;
        };
        match self.to.as_mut() {
            Some(to) => {
                from.do_autonomous_behavior(&mut env, behavior);
                to.do_autonomous_behavior(&mut env, behavior)
            }
            None => from.do_autonomous_behavior(&mut env, behavior),
        }
    }

    /// `ModeTracker::ReleaseBehavior`: release a hold, starting `params`' behaviour in its
    /// place when it names one, and have the remaining track call back with release.
    pub fn release_behavior(
        &mut self,
        behavior: u32,
        params: &Ac2BehaviorParams,
        source: &dyn Ac2AnimationSource,
    ) -> bool {
        if !self.can_animate {
            return true;
        }
        if let Some(to) = self.to.as_mut() {
            if let Some(from) = self.from.as_mut() {
                from.release_behavior(behavior);
            }
            to.release_behavior(behavior);
        } else if let Some(from) = self.from.as_mut() {
            from.release_behavior(behavior);
        }
        let Some(mut env) = env!(self, source) else {
            return true;
        };
        let Some(tracker) = self.to.as_mut().or(self.from.as_mut()) else {
            return true;
        };
        let target = if params.behavior == 0 {
            behavior
        } else {
            tracker.do_behavior(&mut env, params);
            params.behavior
        };
        if let Some(track) = tracker.track_mut(target) {
            if params.behavior == 0 {
                track.params.context = params.context;
            }
            track.callback_with_release = true;
        }
        true
    }

    /// Runs `stop` on the incoming tracker (if any) then the outgoing one, restarting idle on
    /// each as it asks.
    fn stop_with_idle(
        &mut self,
        source: &dyn Ac2AnimationSource,
        stop: impl Fn(&mut Ac2BehaviorTracker, &mut Env) -> u32,
        incoming_first: bool,
    ) -> u32 {
        let Some(mut env) = env!(self, source) else {
            return 0;
        };
        let restart = |tracker: &mut Ac2BehaviorTracker, env: &mut Env| {
            if tracker.start_idle {
                tracker.do_autonomous_behavior(env, AC2_BEHAVIOR_IDLE);
                tracker.do_autonomous_behavior(env, AC2_BEHAVIOR_IDLE_LAYER);
                tracker.start_idle = false;
            }
        };
        let (Some(from), to) = (self.from.as_mut(), self.to.as_mut()) else {
            return 0;
        };
        match to {
            Some(to) if incoming_first => {
                stop(to, &mut env);
                restart(to, &mut env);
                stop(from, &mut env);
                restart(from, &mut env);
                0
            }
            Some(to) => {
                stop(from, &mut env);
                let result = stop(to, &mut env);
                restart(to, &mut env);
                result
            }
            None => {
                let result = stop(from, &mut env);
                restart(from, &mut env);
                result
            }
        }
    }

    /// `ModeTracker::StopBehavior`.
    pub fn stop_behavior(&mut self, behavior: u32, source: &dyn Ac2AnimationSource) -> u32 {
        if !self.can_animate {
            return 0;
        }
        self.stop_with_idle(source, |t, env| t.stop_behavior(env, behavior), false)
    }

    /// `ModeTracker::StopMoveToCancelsBehaviors`.
    pub fn stop_move_to_cancels_behaviors(&mut self, source: &dyn Ac2AnimationSource) {
        if self.can_animate {
            self.stop_with_idle(
                source,
                |t, env| {
                    t.stop_move_to_cancels_behaviors(env);
                    0
                },
                true,
            );
        }
    }

    /// `ModeTracker::StopLayer`.
    pub fn stop_layer(&mut self, layer: u32, source: &dyn Ac2AnimationSource) -> u32 {
        if !self.can_animate {
            return 0;
        }
        let Some(mut env) = env!(self, source) else {
            return 0;
        };
        let Some(from) = self.from.as_mut() else {
            return 0;
        };
        match self.to.as_mut() {
            Some(to) => {
                from.stop_layer(&mut env, layer);
                to.stop_layer(&mut env, layer)
            }
            None => from.stop_layer(&mut env, layer),
        }
    }

    /// `ModeTracker::StopAllWeenieBehaviors`.
    pub fn stop_all_weenie_behaviors(&mut self, source: &dyn Ac2AnimationSource) -> bool {
        if !self.can_animate {
            return false;
        }
        let Some(mut env) = env!(self, source) else {
            return false;
        };
        if let Some(to) = self.to.as_mut() {
            if let Some(from) = self.from.as_mut() {
                from.stop_all_weenie_behaviors(&mut env);
            }
            to.stop_all_weenie_behaviors(&mut env);
        } else if let Some(from) = self.from.as_mut() {
            from.stop_all_weenie_behaviors(&mut env);
        }
        true
    }

    /// `ModeTracker::Update`: advance both trackers (restarting idle as they ask), advance the
    /// transition and complete it at 1, then start the next step toward the desired element and
    /// mode. The caller poses the parts afterwards (`AnimationObj::DoAnimation`).
    pub fn update(&mut self, quantum: f32, source: &dyn Ac2AnimationSource) {
        if self.from.is_none() || self.table.is_none() {
            return;
        }
        self.update_trackers(quantum, source);
        self.transition_ctime = (f64::from(quantum) * f64::from(self.transition_cvel)
            + f64::from(self.transition_ctime)) as f32;
        if 1.0 <= self.transition_ctime {
            // Retail swaps in the incoming tracker unconditionally; one is always present while
            // the transition is timed.
            if let Some(to) = self.to.take() {
                self.from = Some(to);
            }
            let reached = self.from.as_ref().is_some_and(|from| {
                from.mode() == self.desired_mode && from.element() == self.desired_element
            });
            if self.context != 0 && reached {
                self.events.push(Ac2AnimEvent::ModeDone {
                    context: self.context,
                    error: 0,
                });
                self.context = 0;
            }
            self.transition_ctime = 0.0;
            self.transition_cvel = 0.0;
        }
        let pending = self.from.as_ref().is_some_and(|from| {
            self.desired_element != from.element() || self.desired_mode != from.mode()
        });
        if self.transition_ctime == 0.0 && pending {
            let (element, mode) = (self.desired_element, self.desired_mode);
            self.start_transitioning(element, mode, source);
        }
    }

    fn update_trackers(&mut self, quantum: f32, source: &dyn Ac2AnimationSource) {
        let Some(mut env) = env!(self, source) else {
            return;
        };
        let Some(from) = self.from.as_mut() else {
            return;
        };
        from.update(&mut env, quantum);
        if from.start_idle {
            from.do_autonomous_behavior(&mut env, AC2_BEHAVIOR_IDLE);
            from.do_autonomous_behavior(&mut env, AC2_BEHAVIOR_IDLE_LAYER);
            from.start_idle = false;
            // `IsDoingBehavior` asks the incoming tracker during a transition.
            let doing = match self.to.as_ref() {
                Some(to) => to.is_doing_behavior(AC2_BEHAVIOR_IDLE),
                None => self.transition_cvel == 0.0 && from.is_doing_behavior(AC2_BEHAVIOR_IDLE),
            };
            if doing {
                if let Some(track) = from.track_mut(AC2_BEHAVIOR_IDLE) {
                    track.update(quantum);
                }
            }
        }
        if let Some(to) = self.to.as_mut() {
            to.update(&mut env, quantum);
            if to.start_idle {
                to.do_autonomous_behavior(&mut env, AC2_BEHAVIOR_IDLE);
                to.start_idle = false;
                for idle in [AC2_BEHAVIOR_IDLE, AC2_BEHAVIOR_IDLE_LAYER] {
                    if let Some(track) = to.track_mut(idle) {
                        track.update(quantum);
                    }
                }
            }
        }
    }
}
