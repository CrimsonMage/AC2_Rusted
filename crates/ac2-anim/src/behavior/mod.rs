//! The animsys runtime: `ModeTracker` over two `BehaviorTracker`s of `BehaviorTrack`s, and the
//! `SliderTracker`. Where retail calls back into its `AnimationObj` (`PHYSICSOBJREP`), the
//! queries go through [`Ac2AnimationSource`] and the notifications are queued as
//! [`Ac2AnimEvent`]s in call order; the effect, hook and impulse triggers the tracks reach go
//! through the [`Ac2TriggerUpdate`]. The camera and child-fade parameters of a behaviour are
//! events too; FX scripts are the [`Ac2FxScriptTracker`](crate::Ac2FxScriptTracker)'s.

// Negated float comparisons keep retail's unordered (NaN) branches.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

pub(crate) mod hash;
mod mode_tracker;
mod slider;
mod track;
mod tracker;
mod triggers;

pub use mode_tracker::Ac2ModeTracker;
pub use slider::{Ac2SliderInterpolator, Ac2SliderTracker, AC2_SLIDER_INIT};
pub use track::{Ac2BehaviorParams, Ac2BehaviorTrack};
pub use tracker::{
    Ac2BehaviorTracker, AC2_BEHAVIOR_IDLE, AC2_BEHAVIOR_IDLE_LAYER, AC2_BEHAVIOR_MOVE,
    AC2_STOP_LAYER_BUSY, AC2_STOP_NOT_ACTIVE, AC2_STOP_NOT_PLAYING,
};
pub use triggers::{Ac2TriggerAction, Ac2TriggerUpdate};

use crate::behavior_table::{Ac2BehaviorTable, Ac2CharTemplate, Ac2ComboInfo};
use crate::dbanimator::Ac2TrackerState;

/// `F_EPSILON` (0.0002), the slider and weight tolerance.
const F_EPSILON: f32 = 0.0002;

/// What the animation state machine asks of the animated object (`AnimationObj`).
pub trait Ac2AnimationSource {
    /// `GetComboCVel`: the inverse period of the animation `combo` plays for a tracker in
    /// `tracker`'s element and mode at the current slider values (`CPartArray::GetComboCVel`),
    /// 0 when the object has none.
    fn combo_cvel(&self, tracker: Ac2TrackerState, combo: Ac2ComboInfo) -> f32;
    /// `GetHoldLocations`: the hold loop of `combo`'s animation, `None` without one.
    fn hold_loop(&self, tracker: Ac2TrackerState, combo: Ac2ComboInfo) -> Option<(f32, f32)>;
    /// `IsThePlayer` (behaviours lock the player's actions).
    fn is_the_player(&self) -> bool {
        false
    }
}

/// A notification retail makes through `AnimationObj`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ac2AnimEvent {
    /// `HandleBehaviorDone(context, error)`.
    BehaviorDone {
        context: u32,
        error: u32,
    },
    /// `HandleReleaseDone`: a released hold finished.
    ReleaseDone {
        context: u32,
        error: u32,
    },
    /// The mode callback (`DoMode`'s context reached, replaced or failed).
    ModeDone {
        context: u32,
        error: u32,
    },
    /// The slider callback.
    SliderDone {
        context: u32,
        error: u32,
    },
    /// `DoFX(fx, intensity)` (behaviour FX and effect triggers).
    StartFx {
        fx: u32,
        intensity: f32,
    },
    /// `StopFX(fx)`.
    StopFx {
        fx: u32,
    },
    LockActions,
    UnlockActions,
    /// `SetChildrenOpacity(opacity, rampTime)` (`mFadeChildren`).
    ChildrenOpacity {
        opacity: f32,
        ramp_time: f32,
    },
    /// `SetCameraAnimParent(period, rampTime)`: a period of -1 holds the camera.
    CameraAnimParent {
        period: f32,
        ramp_time: f32,
    },
    /// `SetCameraAnimTarget(period, rampTime)`.
    CameraAnimTarget {
        period: f32,
        ramp_time: f32,
    },
    /// `RestoreCamera`.
    RestoreCamera,
    /// `DoBehaviorOnObject(target, behavior)` and, when it takes, `SetTargetedCamera(target,
    /// parent, target part, period, rampTime)` (`mCameraTargetID`).
    TargetedCamera {
        target: u64,
        behavior: u32,
        period: f32,
    },
    /// A `WeenieHook` the animation reached (`CWeenieObject`'s anim hook and
    /// `StoryManager::HandleAnimHook`).
    WeenieHook {
        hook_type: u32,
        hook_num: u32,
        data: u32,
    },
}

/// The table, template, source and event queue a tracker call runs against.
pub(crate) struct Env<'a> {
    pub table: &'a Ac2BehaviorTable,
    pub template: &'a Ac2CharTemplate,
    pub source: &'a dyn Ac2AnimationSource,
    pub events: &'a mut Vec<Ac2AnimEvent>,
    pub triggers: &'a mut Ac2TriggerUpdate,
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod trigger_tests;
