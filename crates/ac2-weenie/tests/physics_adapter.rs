//! Where `ac2-physics` meets the movement reports: a `MovementState` becomes the plain
//! `BodyState` `PlayerMovementModule` reads, and the report it yields.

use ac2_net::protocol::{CellPosition, ClientEvent};
use ac2_physics::{BodyShape, Frame, MovementState, PhysicsBody, Position, Quaternion, Vector};
use ac2_weenie::{BodyState, PositionReporter, ReportStamps};

/// The adapter: `CPhysicsObj::GetPosition`, `get_heading`, `m_nTimesJumped`,
/// `get_velocity` and `CMotionInterp::mLastDoMotionValues` from physics; the turn target
/// (`CMotionInterp::GetTurnToDirection` while `IsTurningTo`) and the movement-action flag from
/// the client's input.
fn body_state(state: &MovementState, turn_to: Option<f32>, actively_moving: bool) -> BodyState {
    BodyState {
        position: CellPosition {
            cell: state.cell_id,
            origin: state.origin,
        },
        heading_degrees: state.heading_degrees,
        turn_to_heading: turn_to,
        motion: state.do_motion,
        times_jumped: state.times_jumped,
        velocity: state.velocity,
        actively_moving,
    }
}

fn state() -> MovementState {
    MovementState {
        cell_id: 0x0299_0109,
        origin: [59.0, 240.0, -44.5],
        rotation: [1.0, 0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 5.0],
        omega: [0.0; 3],
        do_motion: [0.0, 1.0, 0.0],
        turn: 0.0,
        element: 0,
        mode: 0,
        contact: false,
        on_walkable: false,
        heading_degrees: 90.0,
        times_jumped: 1,
    }
}

#[test]
fn a_physics_jump_becomes_a_jump_report() {
    let stamps = ReportStamps {
        instance: 7,
        teleport: 1,
        force_position: 2,
        moveto: 0,
    };
    let mut reporter = PositionReporter::new(0);
    let report = reporter
        .report(100.0, &body_state(&state(), None, true), stamps)
        .unwrap();
    let mut b = vec![0x8E, 0, 1, 0];
    b.extend(100.0f64.to_le_bytes());
    b.extend(0x0299_0109u32.to_le_bytes());
    for v in [59.0f32, 240.0, -44.5] {
        b.extend(v.to_le_bytes());
    }
    b.extend(0x3F7F_FF7Fu32.to_le_bytes()); // heading 90 -> 63; motion (0, 1, 0)
    b.extend(6u32.to_le_bytes()); // cancel move-to 4 | jump 2
    for stamp in [7u16, 1, 2, 0] {
        b.extend(stamp.to_le_bytes());
    }
    for v in [0.0f32, 0.0, 5.0] {
        b.extend(v.to_le_bytes());
    }
    assert_eq!(ClientEvent::Position(report).encode(), (3, b));
}

#[test]
fn a_physics_body_feeds_the_reporter() {
    let frame = Frame::new(
        Vector::new(10.0, 20.0, 0.0),
        Quaternion::new(1.0, 0.0, 0.0, 0.0),
    );
    let shape = BodyShape {
        spheres: Vec::new(),
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    };
    let body = PhysicsBody::new(Position::new(0x0299_0109, frame), shape, None);
    let state = body.movement_state();
    let mut reporter = PositionReporter::new(state.times_jumped);
    let report = reporter
        .report(
            5.0,
            &body_state(&state, None, false),
            ReportStamps::default(),
        )
        .unwrap();
    assert_eq!(report.position.origin, [10.0, 20.0, 0.0]);
    assert_eq!(report.heading_degrees, state.heading_degrees);
    assert!(report.jump_velocity.is_none());
}

/// The frame's `UseTime` calls as physics input. `PhysicsBody::set_input` folds `DoMotion`,
/// `TurnTo` and `SetOmegaVelocity` into one call (a `None` heading stops a turn-to, which retail
/// leaves running), so a latched turn-to keeps its target here.
fn movement_input(
    commands: &ac2_weenie::MovementCommands,
    turning_to: Option<f32>,
) -> ac2_physics::MovementInput {
    ac2_physics::MovementInput {
        motion: commands.do_motion.unwrap_or([0.0; 3]),
        heading: commands.steer.or(commands.turn_to).or(turning_to),
        turn: commands.omega.unwrap_or(0.0),
    }
}

#[test]
fn movement_commands_drive_a_physics_body() {
    use ac2_weenie::{ActiveAction, BodyFacts, CameraState, MovementModule, ACTION_CLASS_MOVEMENT};
    let frame = Frame::new(
        Vector::new(10.0, 20.0, 0.0),
        Quaternion::new(1.0, 0.0, 0.0, 0.0),
    );
    let shape = BodyShape {
        spheres: Vec::new(),
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    };
    let mut body = PhysicsBody::new(Position::new(0x0299_0109, frame), shape, None);
    let mut module = MovementModule::default();
    let forward = ActiveAction {
        class: ACTION_CLASS_MOVEMENT,
        enum_value: ac2_weenie::ACTION_FORWARD,
        map_extent: 1.0,
        extent: 1.0,
    };
    let state = body.movement_state();
    let facts = BodyFacts {
        heading_degrees: state.heading_degrees,
        ..BodyFacts::default()
    };
    let commands = module.use_time(&facts, &[forward], &CameraState::default());
    body.set_input(movement_input(&commands, None));
    let state = body.movement_state();
    let report = PositionReporter::new(0)
        .report(
            1.0,
            &BodyState {
                actively_moving: commands.actively_moving,
                ..body_state(&state, None, commands.actively_moving)
            },
            ReportStamps::default(),
        )
        .unwrap();
    assert!(report.cancel_moveto);
}
