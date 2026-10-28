//! `BehaviorParams` and `BehaviorTrack`: one playing behaviour, its clip time, its blend weight
//! ramping in and out, and the action/hold bookkeeping of `BehaviorTrack::Update`.

/// `BehaviorParams` (the fields the animation state machine reads, and the camera and
/// child-fade parameters it hands to the object; the clone, destroy and FX-script parameters are
/// not modelled).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2BehaviorParams {
    pub behavior: u32,
    pub time_scale: f32,
    /// `mModeID`: a mode `CPhysicsObj::DoBehavior` and `ReleaseBehavior` switch to first (0 for
    /// none).
    pub mode: u32,
    pub hold_cycles: u32,
    /// `mFxID`: an FX started with the behaviour and stopped as it ramps down.
    pub fx: u32,
    pub early_callback: bool,
    pub move_to_cancels: bool,
    pub lock_actions: bool,
    pub leave_idle_alone: bool,
    pub send_event: bool,
    /// The weenie context told when the behaviour finishes (0 for none).
    pub context: u32,
    /// `mFadeChildren`: the object's children fade out while the behaviour plays.
    pub fade_children: bool,
    /// `mCameraParent`, `mCameraTarget`, `mCameraHold`, `mCameraRestore`: the player's camera
    /// follows the object's parent or target part for the behaviour (held until released with
    /// `camera_hold`), or is restored.
    pub camera_parent: bool,
    pub camera_target: bool,
    pub camera_hold: bool,
    pub camera_restore: bool,
    /// `mCameraTargetID` (0 is `INVALID_IID`) and `mCameraBehavior`: an object to run
    /// `mCameraBehavior` on and aim the camera at.
    pub camera_target_id: u64,
    pub camera_behavior: u32,
}

impl Ac2BehaviorParams {
    /// `BehaviorParams::BehaviorParams(ulong)` over `SetDefaults`.
    pub fn new(behavior: u32) -> Self {
        Self {
            behavior,
            time_scale: 1.0,
            mode: 0,
            hold_cycles: 1,
            fx: 0,
            early_callback: false,
            move_to_cancels: false,
            lock_actions: false,
            leave_idle_alone: false,
            send_event: true,
            context: 0,
            fade_children: false,
            camera_parent: false,
            camera_target: false,
            camera_hold: false,
            camera_restore: false,
            camera_target_id: 0,
            camera_behavior: 0,
        }
    }
}

/// `BehaviorTrack` (with its `AnimTrack` base: `mCTime`, `mWeight`, `mLoop`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2BehaviorTrack {
    /// Clip time in cycles (the fraction is the phase).
    pub ctime: f32,
    pub weight: f32,
    /// `mLoop`: set when the clip time crossed a whole cycle.
    pub looped: bool,
    pub params: Ac2BehaviorParams,
    pub weight_vel: f32,
    pub ctime_vel: f32,
    pub is_action: bool,
    pub is_hold: bool,
    pub is_dynamic_fp_camera: bool,
    pub was_hold: bool,
    pub callback_with_release: bool,
    pub ramp_time: f32,
    /// Index into the behaviour table's layers.
    pub layer: u32,
    /// The `AnimMap` key the behaviour plays.
    pub anim_key: u32,
    pub is_ramping_down: bool,
    pub stopped_action: bool,
    pub started_ramping_down: bool,
    pub terminated_fx: bool,
    pub loop_start: f32,
    pub loop_end: f32,
    pub repeats: u32,
    /// `AnimTrack::mEffect`, `mHooks`, `mImpulses` and `mTriggeredEffects`: what the animation
    /// has reached (a hook or impulse fires once per track).
    pub triggers: crate::Ac2TrackTriggers,
}

impl Ac2BehaviorTrack {
    /// `BehaviorTrack::BehaviorTrack()`.
    pub(crate) fn new(params: Ac2BehaviorParams) -> Self {
        Self {
            ctime: 0.0,
            weight: 0.0,
            looped: false,
            params,
            weight_vel: 0.0,
            ctime_vel: 0.0,
            is_action: false,
            is_hold: false,
            is_dynamic_fp_camera: false,
            was_hold: false,
            callback_with_release: false,
            ramp_time: 0.0,
            layer: 0,
            anim_key: 0,
            is_ramping_down: false,
            stopped_action: false,
            started_ramping_down: false,
            terminated_fx: false,
            loop_start: 0.0,
            loop_end: 0.0,
            repeats: 0,
            triggers: crate::Ac2TrackTriggers::default(),
        }
    }

    /// `BehaviorTrack::Update`: advance the clip time (wrapping at 64 cycles), ramp the weight,
    /// shape an action's weight to ramp in over its first `mRampTime` seconds and out over its
    /// last, and loop a hold between its loop points.
    pub(crate) fn update(&mut self, quantum: f32) {
        let q = f64::from(quantum);
        let before = f64::from(self.ctime).floor();
        let advanced = f64::from(self.params.time_scale) * f64::from(self.ctime_vel) * q
            + f64::from(self.ctime);
        self.ctime = advanced as f32;
        if advanced.floor() != before {
            self.looped = true;
            // A plain behaviour clears its effect triggers each cycle (`mEffect.m_num = 0`).
            if !self.is_action && !self.is_hold {
                self.triggers.effects.clear();
            }
        }
        if !(f64::from(self.ctime) < 64.0) {
            self.ctime = (f64::from(self.ctime) - 64.0) as f32;
        }
        if f64::from(self.weight) > 1.0 {
            self.weight = 1.0;
            if self.weight_vel > 0.0 {
                self.weight_vel = 0.0;
            }
        }
        self.weight = (q * f64::from(self.weight_vel) + f64::from(self.weight)) as f32;
        let period = if self.ctime_vel == 0.0 {
            0.0
        } else {
            (1.0 / f64::from(self.ctime_vel)) as f32
        };
        let elapsed = (f64::from(period) * f64::from(self.ctime)) as f32;
        if self.is_action && !self.stopped_action {
            self.shape_action_weight(period, elapsed);
        }
        if self.is_hold {
            self.loop_hold();
        }
    }

    fn shape_action_weight(&mut self, period: f32, elapsed: f32) {
        let (t, ramp) = (f64::from(elapsed), f64::from(self.ramp_time));
        let tail = f64::from(period) - ramp;
        if !(t >= ramp) {
            self.weight = if self.ramp_time == 0.0 {
                0.0
            } else {
                (t / ramp) as f32
            };
        } else if !self.is_ramping_down && t > ramp && tail > t {
            self.weight = 1.0;
        } else if tail < t {
            self.weight = if self.ramp_time == 0.0 {
                0.0
            } else {
                ((f64::from(period) - t) / ramp) as f32
            };
            if !self.is_ramping_down {
                self.started_ramping_down = true;
            }
            self.is_ramping_down = true;
        }
    }

    fn loop_hold(&mut self) {
        let (start, end) = (f64::from(self.loop_start), f64::from(self.loop_end));
        if !(end > 0.0) || !(end > start) {
            self.repeats = self.repeats.wrapping_add(1);
            return;
        }
        while f64::from(self.ctime) > end {
            self.repeats = self.repeats.wrapping_add(1);
            self.looped = true;
            self.ctime = (f64::from(self.ctime) - end + start) as f32;
        }
    }
}
