//! `PlayerMovementModule::UseTime` (Legions 0x005394E0, admin 0x00569550): every frame the
//! active movement actions become a do-motion vector, a turn rate or a turn-to heading, and in
//! mouse-look the camera direction steers the body. The keys arrive as the `ActionInterpreter`
//! resolved them (`ActionInterpreter::GetActionValues` through the key map) and the camera as
//! `CameraManager` state; the output is the calls `UseTime` makes on the player's
//! `CPhysicsObj` and `CMotionInterp`, as plain data. The reports that follow
//! (`ShouldSendPositionEvent` ...) are [`crate::PositionReporter`].

/// `ActionMap` class of movement actions (`GetActionValues`' `_action_class`).
pub const ACTION_CLASS_MOVEMENT: u32 = 0x4000_0001;
/// Movement action enum values `UseTime` switches on (`enum_value - 0x40000001`).
pub const ACTION_FORWARD: u32 = 0x4000_0001;
pub const ACTION_TURN_NEGATIVE: u32 = 0x4000_0003;
pub const ACTION_TURN_POSITIVE: u32 = 0x4000_0004;
pub const ACTION_STRAFE_POSITIVE: u32 = 0x4000_0005;
pub const ACTION_STRAFE_NEGATIVE: u32 = 0x4000_0006;
pub const ACTION_JUMP: u32 = 0x4000_0007;
pub const ACTION_ABOUT_FACE: u32 = 0x4000_0009;
pub const ACTION_TURN_PLUS_90: u32 = 0x4000_000A;
pub const ACTION_TURN_MINUS_90: u32 = 0x4000_000B;

/// One active action (`ActionInfo`), already resolved by `ActionInterpreter::GetActionValues`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveAction {
    pub class: u32,
    pub enum_value: u32,
    /// The key map's extent (`GetActionValues`' `_extent`).
    pub map_extent: f32,
    /// `ActionInfo::m_extent` (how far the key is pressed).
    pub extent: f32,
}

/// The `CameraManager` state `UseTime` reads.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CameraState {
    /// `m_mouseLook`.
    pub mouse_look: bool,
    /// `GetFirstPersonState`.
    pub first_person: bool,
    /// `m_prefs_3rdPersonSteersPlayer`.
    pub third_person_steers: bool,
    /// `GetGlobalDirection`.
    pub direction: [f32; 3],
}

/// What the frame knows about the player body.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BodyFacts {
    /// `CPhysicsObj::IsMovingTo`.
    pub moving_to: bool,
    /// `CMotionInterp::IsTurningTo`.
    pub turning_to: bool,
    /// `CPhysicsObj::IsInterpolating`.
    pub interpolating: bool,
    /// `CPhysicsObj::get_heading`.
    pub heading_degrees: f32,
}

/// The calls `UseTime` makes, in this order within a frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MovementCommands {
    /// `ActionInterpreter::TurnOffRunLock` (moving to: nothing else happens).
    pub turn_off_run_lock: bool,
    /// `CPhysicsObj::JumpLogic` calls (one per active jump action), during the action loop.
    pub jumps: u32,
    /// `CMotionInterp::TurnTo(heading, 0)` from an about-face or 90-degree action.
    pub turn_to: Option<f32>,
    /// `CPhysicsObj::set_movement_active(1)`: after the loop whenever actions are read.
    pub movement_active: bool,
    /// `CPhysicsObj::StopMoveToCancelsBehaviors` when a movement action is held.
    pub stop_move_to: bool,
    /// `CMotionInterp::DoMotion(vector)` unless interpolating.
    pub do_motion: Option<[f32; 3]>,
    /// Mouse-look steering: `CPhysicsObj::set_heading(h)` then `CMotionInterp::TurnTo(h, 1)`.
    pub steer: Option<f32>,
    /// `CMotionInterp::SetOmegaVelocity(turn)` otherwise, unless a turn-to is latched.
    pub omega: Option<f32>,
    /// `m_is_actively_moving`, which the next report carries as `m_cancel_moveto`.
    pub actively_moving: bool,
}

/// `Heading::set` / `set_degrees` / `operator+`: `fmod(degrees, 360)` stored as a float,
/// wrapped once when negative.
fn heading_set(degrees: f64) -> f32 {
    let r = degrees % 360.0;
    if r < 0.0 {
        let wrapped = r + 360.0;
        return if 360.0 <= wrapped {
            0.0
        } else {
            wrapped as f32
        };
    }
    r as f32
}

/// `Heading(radians, RADIANS)`: `degrees = radians * 57.29578f` stored, then `set`.
fn heading_from_radians(radians: f32) -> f32 {
    let degrees = (f64::from(57.295_78f32) * f64::from(radians)) as f32;
    heading_set(f64::from(degrees))
}

/// The steering heading from the camera's global direction: `atan2` of the normalised x/y,
/// subtracted from 5 pi / 2; `Heading::ZERO` below a 0.0002 length.
fn camera_heading(direction: [f32; 3]) -> f32 {
    let (x, y) = (f64::from(direction[0]), f64::from(direction[1]));
    let len = (y * y + x * x).sqrt();
    if len < f64::from(0.0002f32) {
        return 0.0;
    }
    let inv = f64::from(1.0f32) / len;
    let nx = f64::from((x * inv) as f32);
    let ny = inv * y;
    let radians = (7.853_981_633_974_483 - ny.atan2(nx)) as f32;
    heading_from_radians(radians)
}

/// `PlayerMovementModule`'s per-frame state.
#[derive(Debug, Default)]
pub struct MovementModule {
    /// `m_lock_actions` (`PlayerMovementModule::LockActions`).
    pub locked: bool,
    /// The function-static latch set by a turn-to action and cleared once the motion
    /// interpreter is no longer turning to.
    turn_latch: bool,
}

impl MovementModule {
    /// One frame of `PlayerMovementModule::UseTime` before its reports.
    pub fn use_time(
        &mut self,
        body: &BodyFacts,
        actions: &[ActiveAction],
        camera: &CameraState,
    ) -> MovementCommands {
        let mut out = MovementCommands::default();
        if body.moving_to {
            out.turn_off_run_lock = true;
            return out;
        }
        if !body.turning_to {
            self.turn_latch = false;
        }
        if self.locked {
            return out;
        }
        let mut motion = [0.0f32; 3];
        let mut turn = 0.0f32;
        for action in actions {
            if action.class != ACTION_CLASS_MOVEMENT {
                continue;
            }
            let mut id = action.enum_value;
            if camera.mouse_look && camera.third_person_steers {
                id = match id {
                    ACTION_TURN_NEGATIVE => ACTION_STRAFE_POSITIVE,
                    ACTION_TURN_POSITIVE => ACTION_STRAFE_NEGATIVE,
                    other => other,
                };
            }
            let value = (f64::from(action.map_extent) * f64::from(action.extent)) as f32;
            let add = |a: f32, b: f32| (f64::from(a) + f64::from(b)) as f32;
            let sub = |a: f32, b: f32| (f64::from(a) - f64::from(b)) as f32;
            let turn_by = |degrees: f32| {
                heading_set(
                    f64::from(body.heading_degrees) + f64::from(heading_set(f64::from(degrees))),
                )
            };
            match id {
                ACTION_FORWARD => {
                    motion[1] = add(motion[1], value);
                    out.actively_moving = true;
                }
                ACTION_TURN_NEGATIVE => {
                    turn = -1.0;
                    out.actively_moving = true;
                }
                ACTION_TURN_POSITIVE => {
                    turn = 1.0;
                    out.actively_moving = true;
                }
                ACTION_STRAFE_POSITIVE => {
                    motion[0] = add(motion[0], value);
                    out.actively_moving = true;
                }
                ACTION_STRAFE_NEGATIVE => {
                    motion[0] = sub(motion[0], value);
                    out.actively_moving = true;
                }
                ACTION_JUMP => out.jumps += 1,
                ACTION_ABOUT_FACE | ACTION_TURN_PLUS_90 | ACTION_TURN_MINUS_90
                    if !self.turn_latch && !camera.mouse_look =>
                {
                    let degrees = match id {
                        ACTION_ABOUT_FACE => 180.0,
                        ACTION_TURN_PLUS_90 => 90.0,
                        _ => -90.0,
                    };
                    out.turn_to = Some(turn_by(degrees));
                    self.turn_latch = true;
                }
                _ => {}
            }
        }
        out.movement_active = true;
        out.stop_move_to = out.actively_moving;
        if !body.interpolating {
            out.do_motion = Some(motion);
        }
        if camera.mouse_look && (camera.first_person || camera.third_person_steers) {
            out.steer = Some(camera_heading(camera.direction));
        } else if !self.turn_latch {
            out.omega = Some(turn);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(enum_value: u32) -> ActiveAction {
        ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value,
            map_extent: 1.0,
            extent: 1.0,
        }
    }

    fn body(heading: f32) -> BodyFacts {
        BodyFacts {
            heading_degrees: heading,
            ..BodyFacts::default()
        }
    }

    #[test]
    fn forward_and_strafe_keys_build_the_do_motion_vector() {
        let mut m = MovementModule::default();
        let mut half = key(ACTION_STRAFE_NEGATIVE);
        half.extent = 0.5;
        let out = m.use_time(
            &body(0.0),
            &[key(ACTION_FORWARD), half, key(ACTION_TURN_POSITIVE)],
            &CameraState::default(),
        );
        assert_eq!(out.do_motion, Some([-0.5, 1.0, 0.0]));
        assert_eq!(out.omega, Some(1.0));
        assert!(out.actively_moving && out.stop_move_to && out.movement_active);
        // Other classes are ignored; no keys still sends a zero motion and omega.
        let other = ActiveAction {
            class: 0x4000_0002,
            ..key(ACTION_FORWARD)
        };
        let out = m.use_time(&body(0.0), &[other], &CameraState::default());
        assert_eq!((out.do_motion, out.omega), (Some([0.0; 3]), Some(0.0)));
        assert!(!out.actively_moving);
    }

    #[test]
    fn turn_keys_turn_to_once_until_the_turn_ends() {
        let mut m = MovementModule::default();
        let out = m.use_time(
            &body(300.0),
            &[key(ACTION_TURN_PLUS_90)],
            &CameraState::default(),
        );
        assert_eq!(out.turn_to, Some(30.0));
        // Latched: no omega, no second turn while the motion interpreter turns.
        let turning = BodyFacts {
            turning_to: true,
            ..body(310.0)
        };
        let out = m.use_time(&turning, &[key(ACTION_ABOUT_FACE)], &CameraState::default());
        assert_eq!((out.turn_to, out.omega), (None, None));
        // Arrived: the latch clears and the next press turns again.
        let out = m.use_time(
            &body(30.0),
            &[key(ACTION_TURN_MINUS_90)],
            &CameraState::default(),
        );
        assert_eq!(out.turn_to, Some(300.0));
        assert!(!out.actively_moving);
    }

    #[test]
    fn mouse_look_steers_by_the_camera_and_turn_keys_strafe() {
        let mut m = MovementModule::default();
        let camera = CameraState {
            mouse_look: true,
            third_person_steers: true,
            direction: [1.0, 0.0, 0.0],
            ..CameraState::default()
        };
        let out = m.use_time(&body(0.0), &[key(ACTION_TURN_NEGATIVE)], &camera);
        assert_eq!(out.do_motion, Some([1.0, 0.0, 0.0]));
        assert_eq!(out.steer, Some(90.0)); // east
        assert_eq!(out.omega, None);
        let north = CameraState {
            direction: [0.0, 1.0, 0.0],
            ..camera
        };
        assert_eq!(m.use_time(&body(0.0), &[], &north).steer, Some(0.0));
        let still = CameraState {
            direction: [0.0, 0.0, 1.0],
            ..camera
        };
        assert_eq!(m.use_time(&body(0.0), &[], &still).steer, Some(0.0));
    }

    #[test]
    fn moving_to_or_locked_frames_do_nothing_else() {
        let mut m = MovementModule::default();
        let moving = BodyFacts {
            moving_to: true,
            ..body(0.0)
        };
        let out = m.use_time(&moving, &[key(ACTION_FORWARD)], &CameraState::default());
        assert_eq!(
            out,
            MovementCommands {
                turn_off_run_lock: true,
                ..MovementCommands::default()
            }
        );
        m.locked = true;
        let out = m.use_time(&body(0.0), &[key(ACTION_FORWARD)], &CameraState::default());
        assert_eq!(out, MovementCommands::default());
        m.locked = false;
        let interpolating = BodyFacts {
            interpolating: true,
            ..body(0.0)
        };
        let out = m.use_time(&interpolating, &[key(ACTION_JUMP)], &CameraState::default());
        assert_eq!((out.jumps, out.do_motion), (1, None));
    }
}
