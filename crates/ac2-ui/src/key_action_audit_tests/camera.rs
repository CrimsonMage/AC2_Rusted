//! The camera actions: `CameraManager::StartAction` and `StopAction` (Legions 0x00546FF0 and 0x00547250).
//!
//! Part of the audit table in the parent module; see there for the statuses.

use super::{Row, Status};

pub(super) const ROWS: &[Row] = &[
    (
        0x4000_0028,
        "CameraSwingOutLeft",
        "CameraManager::StartAction -> FakeTransition",
        "camera.rs fake_transition (camera's own turn)",
        Status::Wired,
    ),
    (
        0x4000_0029,
        "CameraSwingOutRight",
        "CameraManager::StartAction -> SwingOutLeft (Legions)",
        "camera.rs SWING_OUT_RIGHT -> swing_out_left",
        Status::Wired,
    ),
    (
        0x4000_002A,
        "CameraShoulderView",
        "CameraManager::ShoulderView",
        "camera.rs SHOULDER_VIEW",
        Status::Handled,
    ),
    (
        0x4000_002E,
        "CameraRotateLeft",
        "CameraManager::StartAction (heading)",
        "camera.rs ROTATE_LEFT",
        Status::Handled,
    ),
    (
        0x4000_002F,
        "CameraRotateRight",
        "CameraManager::StartAction (heading)",
        "camera.rs ROTATE_RIGHT",
        Status::Handled,
    ),
    (
        0x4000_0030,
        "CameraReset",
        "CameraManager::SetDefaults",
        "camera.rs RESET",
        Status::Handled,
    ),
    (
        0x4000_00AE,
        "CameraLookAtGround",
        "CameraManager::LookAtGround",
        "camera.rs LOOK_AT_GROUND",
        Status::Handled,
    ),
    (
        0x4000_00B1,
        "CameraLookAtSelf",
        "CameraManager::LookAtSelf",
        "camera.rs LOOK_AT_SELF",
        Status::Wired,
    ),
    (
        0x4000_00B3,
        "CameraMapView",
        "CameraManager::SetMapView",
        "camera.rs MAP_VIEW (SceneTool degrade switch not modelled)",
        Status::Wired,
    ),
    (
        0x4000_00B4,
        "CameraZoomIn",
        "CameraManager::StartAction (scale 0.75^(extent*speed))",
        "camera.rs ZOOM_SCALE_IN",
        Status::Wired,
    ),
    (
        0x4000_00B5,
        "CameraZoomOut",
        "CameraManager::StartAction (scale 1.5^(extent*speed))",
        "camera.rs ZOOM_SCALE_OUT",
        Status::Wired,
    ),
    (
        0x4000_00B6,
        "CameraTiltUp",
        "CameraManager::StartAction (pitch)",
        "camera.rs PITCH_UP",
        Status::Handled,
    ),
    (
        0x4000_00B7,
        "CameraTiltDown",
        "CameraManager::StartAction (pitch)",
        "camera.rs PITCH_DOWN",
        Status::Handled,
    ),
    (
        0x4000_00A2,
        "CameraFirstPerson",
        "gmCameraUI::ListenToMessage -> SetFirstPersonState",
        "camera_ui.rs UI_FIRST_PERSON",
        Status::Handled,
    ),
    (
        0x4000_00BA,
        "CameraToggleMouseLook",
        "gmCameraUI::ListenToMessage -> SetMouseLookState",
        "camera_ui.rs UI_MOUSE_LOOK_TOGGLE",
        Status::Handled,
    ),
];
