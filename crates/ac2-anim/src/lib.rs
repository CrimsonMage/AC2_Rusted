#![forbid(unsafe_code)]

//! Asheron's Call 2 animation: `CSetup` part hierarchies, `AnimMap` motion tables and the
//! `DBAnimator` tree (`AnimCube`, `AMHuman`, `AMSwitchME`, `AnimAsset`) that blends clips by
//! slider values.

mod animation;
mod animator;
mod behavior;
mod behavior_table;
mod blend;
mod dbanimator;
mod fx_scripts;
mod fxscript;
mod fxscript_tracker;
mod setup;

pub use animation::{load_ac2_motion_animation, Ac2AnimMap, Ac2Animation, Ac2BoneTrack};
pub use animator::{
    load_ac2_motion_animator, Ac2Blend, Ac2MotionAnimator, AC2_SLIDER_FORWARD_SPEED,
    AC2_SLIDER_SIDESTEP_SPEED, AC2_SLIDER_TURN_RATE,
};
pub use behavior::{
    Ac2AnimEvent, Ac2AnimationSource, Ac2BehaviorParams, Ac2BehaviorTrack, Ac2BehaviorTracker,
    Ac2ModeTracker, Ac2SliderInterpolator, Ac2SliderTracker, Ac2TriggerAction, Ac2TriggerUpdate,
    AC2_BEHAVIOR_IDLE, AC2_BEHAVIOR_IDLE_LAYER, AC2_BEHAVIOR_MOVE, AC2_SLIDER_INIT,
    AC2_STOP_LAYER_BUSY, AC2_STOP_NOT_ACTIVE, AC2_STOP_NOT_PLAYING,
};
pub use behavior_table::{
    Ac2BehaviorInfo, Ac2BehaviorTable, Ac2CharTemplate, Ac2ComboInfo, Ac2ModeState,
    Ac2ModeTransition, Ac2TransitionInfo, AC2_BEHAVIOR_ACTION, AC2_BEHAVIOR_DYNAMIC_FP_CAMERA,
    AC2_BEHAVIOR_HOLD, AC2_SLIDER_CLIENT_CONTROLLED, AC2_SLIDER_WEENIE_CONTROLLED,
};
pub use blend::blend_transform;
pub use dbanimator::{
    load_ac2_db_animator, Ac2AnimAsset, Ac2AnimInfo, Ac2AnimTrigger, Ac2AnimatorContext,
    Ac2AssetBone, Ac2BoneTransform, Ac2Compositor, Ac2CubeSlider, Ac2DbAnimator, Ac2ImpulseTrigger,
    Ac2Interpolator, Ac2PoseInput, Ac2Random, Ac2TrackTriggers, Ac2TrackerState, Ac2WeenieHook,
};
pub use fx_scripts::{load_fx_scripts, FxScripts};
pub use fxscript::{load_ac2_fx_script, Ac2FxScript, FX_SCRIPT_PREFIX};
pub use fxscript_tracker::Ac2FxScriptTracker;
pub use setup::{
    Ac2Bone, Ac2DegradeDistances, Ac2DegradeInfo, Ac2Location, Ac2PlacementFrame, Ac2Setup,
    Ac2Transform,
};
