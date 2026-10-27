//! The movement actions: `PlayerMovementModule::UseTime` over the held actions.
//!
//! Part of the audit table in the parent module; see there for the statuses.

use super::{Row, Status};

pub(super) const ROWS: &[Row] = &[
    (
        0x4000_0020,
        "Forward_KEY",
        "PlayerMovementModule::UseTime (forward)",
        "ac2-weenie movement_module.rs ACTION_FORWARD",
        Status::Handled,
    ),
    (
        0x4000_0021,
        "Backward_KEY",
        "PlayerMovementModule::UseTime (forward, extent -1)",
        "movement_module.rs ACTION_FORWARD",
        Status::Handled,
    ),
    (
        0x4000_0022,
        "Strafe_Left_KEY",
        "PlayerMovementModule::UseTime (strafe negative)",
        "movement_module.rs ACTION_STRAFE_NEGATIVE",
        Status::Handled,
    ),
    (
        0x4000_0023,
        "Strafe_Right_KEY",
        "PlayerMovementModule::UseTime (strafe positive)",
        "movement_module.rs ACTION_STRAFE_POSITIVE",
        Status::Handled,
    ),
    (
        0x4000_0024,
        "Turn_Left_KEY",
        "PlayerMovementModule::UseTime (turn positive)",
        "movement_module.rs ACTION_TURN_POSITIVE",
        Status::Handled,
    ),
    (
        0x4000_0025,
        "Turn_Right_KEY",
        "PlayerMovementModule::UseTime (turn negative)",
        "movement_module.rs ACTION_TURN_NEGATIVE",
        Status::Handled,
    ),
    (
        0x4000_0026,
        "Jump_KEY",
        "PlayerMovementModule::UseTime (jump, one-shot)",
        "movement_module.rs ACTION_JUMP",
        Status::Handled,
    ),
    (
        0x4000_0027,
        "RunLock_KEY",
        "ActionInterpreter (run lock toggle)",
        "interpreter.rs TURN_OFF run lock",
        Status::Handled,
    ),
];
