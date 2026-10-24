#![forbid(unsafe_code)]

//! Asheron's Call 2 animation: `CSetup` part hierarchies, `AnimMap` motion tables and the
//! `DBAnimator` tree (`AnimCube`, `AMHuman`, `AMSwitchME`, `AnimAsset`) that blends clips by
//! slider values.

mod animation;
mod animator;
mod setup;

pub use animation::{load_ac2_motion_animation, Ac2AnimMap, Ac2Animation, Ac2BoneTrack};
pub use animator::{
    load_ac2_motion_animator, Ac2Blend, Ac2MotionAnimator, AC2_SLIDER_FORWARD_SPEED,
    AC2_SLIDER_SIDESTEP_SPEED, AC2_SLIDER_TURN_RATE,
};
pub use setup::{Ac2Bone, Ac2Location, Ac2PlacementFrame, Ac2Setup, Ac2Transform};
