//! `PlayerMovementModule`: when the player's body becomes a `CPositionPack` report, and when
//! the head direction becomes a `CLookAtDir`. Both are checked every frame
//! (`PlayerMovementModule::UseTime`).

use ac2_net::protocol::{CellPosition, ClientPosition};

/// The player body facts the reports read, as plain data from physics and input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyState {
    /// `CPhysicsObj::GetPosition`: cell and origin.
    pub position: CellPosition,
    /// `CPhysicsObj::get_heading`, degrees in `[0, 360)`.
    pub heading_degrees: f32,
    /// `CMotionInterp::GetTurnToDirection` while `CMotionInterp::IsTurningTo`: the report
    /// then carries the turn target instead of the current heading.
    pub turn_to_heading: Option<f32>,
    /// `CMotionInterp::mLastDoMotionValues`.
    pub motion: [f32; 3],
    /// `CPhysicsObj::m_nTimesJumped`.
    pub times_jumped: u32,
    /// `CPhysicsObj::get_velocity`, read when a jump is reported.
    pub velocity: [f32; 3],
    /// `PlayerMovementModule::m_is_actively_moving` (set by `UseTime` for movement actions):
    /// the report's `m_cancel_moveto`.
    pub actively_moving: bool,
}

/// The stamps a report carries: `CPhysicsObj::get_instance_stamp` and
/// `CPhysicsObj::get_update_time` for `TELEPORT`, `FORCE_POSITION` and `MOVE_TO`, which the
/// client keeps from server events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReportStamps {
    pub instance: u16,
    pub teleport: u16,
    pub force_position: u16,
    pub moveto: u16,
}

/// Seconds between unchanged-motion reports (`ShouldSendPositionEvent`: more than 0.2 s).
const POSITION_SECONDS: f64 = 0.2;
/// Seconds between head reports (`ShouldSendLookAtEvent`: more than 0.5 s).
const LOOK_SECONDS: f64 = 0.5;
/// Component tolerance of the motion and position comparisons (an f32 constant).
const TOLERANCE: f32 = 0.0002;

/// `Heading::EPSILON`: `fmod(57.29578f * 0.0002f, 360)`, 0.0002 radians in degrees.
fn heading_epsilon() -> f32 {
    ((f64::from(57.295_78f32) * f64::from(0.0002f32)) % 360.0) as f32
}

/// `Heading::is_in_range(rhs, EPSILON)` as `Heading::equals` calls it.
pub fn heading_equals(lhs: f32, rhs: f32) -> bool {
    let mut d = (f64::from(lhs) - f64::from(rhs)) % 360.0;
    if d < 0.0 {
        d += 360.0;
        if 360.0 <= d {
            d = 0.0;
        }
    }
    // `Heading::abs`: fold past 180, then `fmod(|d|, 360)`.
    let mut a = f64::from(d as f32);
    if 180.0 < a {
        a = f64::from((a - 360.0) as f32);
    }
    let mut a = a.abs() % 360.0;
    if a < 0.0 {
        a += 360.0;
        if 360.0 <= a {
            a = 0.0;
        }
    }
    (a as f32) < heading_epsilon()
}

/// Whether two floats differ by at least the tolerance, as the x87 `fcompp` tests do.
fn differs(a: f32, b: f32) -> bool {
    f64::from(TOLERANCE) <= (f64::from(a) - f64::from(b)).abs()
}

/// `PlayerMovementModule`'s report state. `Startup` zeroes the last send time and the
/// `CPositionPack` the comparisons read (cell 0, zero origin, motion and heading); the head
/// direction fields are left as allocated, taken as zero here.
#[derive(Debug, Default)]
pub struct PositionReporter {
    /// `m_last_sent_pack`: cell, origin, motion and heading of the last report.
    last_cell: u32,
    last_origin: [f32; 3],
    last_motion: [f32; 3],
    last_heading: f32,
    last_send_time: f64,
    jump_stamp: u32,
    last_look: (f32, f32),
    last_look_time: f64,
}

impl PositionReporter {
    /// `PlayerMovementModule::SetPlayerID`: the jump count starts where the body is.
    pub fn new(times_jumped: u32) -> Self {
        Self {
            jump_stamp: times_jumped,
            ..Self::default()
        }
    }

    /// `ShouldSendPositionEvent` then `SendPositionEvent`: the report to send at `cur_time`
    /// (`Timer::cur_time`), if any. A new jump reports at once; so does a do-motion change of
    /// 0.0002 or more in any axis; otherwise a body whose cell, origin (0.0002 per axis) or
    /// heading (`Heading::equals`) changed reports once more than 0.2 s passed.
    pub fn report(
        &mut self,
        cur_time: f64,
        body: &BodyState,
        stamps: ReportStamps,
    ) -> Option<ClientPosition> {
        if self.jump_stamp > body.times_jumped {
            return None;
        }
        if body.times_jumped <= self.jump_stamp {
            let motion_changed = (0..3).any(|i| differs(body.motion[i], self.last_motion[i]));
            if !motion_changed {
                if cur_time - self.last_send_time <= POSITION_SECONDS {
                    return None;
                }
                let same_place = body.position.cell == self.last_cell
                    && !(0..3).any(|i| differs(body.position.origin[i], self.last_origin[i]));
                if same_place && heading_equals(body.heading_degrees, self.last_heading) {
                    return None;
                }
            }
        }
        let heading_degrees = body.turn_to_heading.unwrap_or(body.heading_degrees);
        let jump = self.jump_stamp < body.times_jumped;
        if jump {
            self.jump_stamp += 1;
        }
        self.last_cell = body.position.cell;
        self.last_origin = body.position.origin;
        self.last_motion = body.motion;
        self.last_heading = heading_degrees;
        self.last_send_time = cur_time;
        Some(ClientPosition {
            time: cur_time,
            position: body.position,
            heading_degrees,
            motion: body.motion,
            cancel_moveto: body.actively_moving,
            instance_stamp: stamps.instance,
            teleport_stamp: stamps.teleport,
            force_position_stamp: stamps.force_position,
            moveto_stamp: stamps.moveto,
            jump_velocity: jump.then_some(body.velocity),
        })
    }

    /// `ShouldSendLookAtEvent` then `SendLookAtEvent`: the head direction (head x and z
    /// sliders) when not looking at an object, more than 0.5 s after the last, if it changed.
    pub fn look(
        &mut self,
        cur_time: f64,
        looking_at_object: bool,
        head: (f32, f32),
    ) -> Option<(f32, f32)> {
        if looking_at_object || cur_time - self.last_look_time <= LOOK_SECONDS {
            return None;
        }
        let tolerance = f64::from(TOLERANCE);
        let moved = tolerance < (f64::from(head.0) - f64::from(self.last_look.0)).abs()
            || tolerance < (f64::from(head.1) - f64::from(self.last_look.1)).abs();
        if !moved {
            return None;
        }
        self.last_look = head;
        self.last_look_time = cur_time;
        Some(head)
    }
}

#[cfg(test)]
mod tests {
    use super::{heading_equals, BodyState, PositionReporter, ReportStamps};
    use ac2_net::protocol::{CellPosition, ClientEvent};

    const STAMPS: ReportStamps = ReportStamps {
        instance: 3,
        teleport: 4,
        force_position: 5,
        moveto: 6,
    };

    fn body(x: f32, heading: f32, motion: [f32; 3]) -> BodyState {
        BodyState {
            position: CellPosition {
                cell: 0x0299_0109,
                origin: [x, 2.0, 3.0],
            },
            heading_degrees: heading,
            turn_to_heading: None,
            motion,
            times_jumped: 0,
            velocity: [0.0, 0.0, 4.0],
            actively_moving: false,
        }
    }

    /// `ECM_Physics::SendEvt_CPosition` bytes, written out field by field.
    fn expected(time: f64, x: f32, word: u32, flags: u32, jump: Option<[f32; 3]>) -> Vec<u8> {
        let mut b = vec![0x8E, 0, 1, 0];
        b.extend(time.to_le_bytes());
        b.extend(0x0299_0109u32.to_le_bytes());
        for v in [x, 2.0, 3.0] {
            b.extend(v.to_le_bytes());
        }
        b.extend(word.to_le_bytes());
        b.extend(flags.to_le_bytes());
        for stamp in [3u16, 4, 5, 6] {
            b.extend(stamp.to_le_bytes());
        }
        for v in jump.into_iter().flatten() {
            b.extend(v.to_le_bytes());
        }
        b
    }

    fn bytes(report: Option<ac2_net::protocol::ClientPosition>) -> Vec<u8> {
        ClientEvent::Position(report.expect("a report")).encode().1
    }

    // Heading 90 degrees is byte 63 (`_ftol(90 * (1/360) * 255)`); motion axes 0 and 1 are
    // bytes 0x7F and 0xFF.
    const STAND: u32 = 0x3F7F_7F7F;
    const WALK: u32 = 0x3F7F_FF7F;

    #[test]
    fn the_first_frame_reports_against_the_zeroed_pack_and_a_jump_reports_at_once() {
        let mut r = PositionReporter::new(0);
        let standing = body(1.0, 90.0, [0.0; 3]);
        assert_eq!(
            bytes(r.report(8.0, &standing, STAMPS)),
            expected(8.0, 1.0, STAND, 0, None)
        );
        let mut jumping = standing;
        jumping.times_jumped = 1;
        // Within the 0.2 s window: the jump goes anyway, flag 2 with the velocity.
        assert_eq!(
            bytes(r.report(8.0625, &jumping, STAMPS)),
            expected(8.0625, 1.0, STAND, 2, Some([0.0, 0.0, 4.0]))
        );
        assert!(r.report(8.125, &jumping, STAMPS).is_none());
    }

    #[test]
    fn a_do_motion_change_reports_inside_the_window() {
        let mut r = PositionReporter::new(0);
        r.report(8.0, &body(1.0, 90.0, [0.0; 3]), STAMPS).unwrap();
        let mut walking = body(1.0, 90.0, [0.0, 1.0, 0.0]);
        walking.actively_moving = true;
        assert_eq!(
            bytes(r.report(8.0625, &walking, STAMPS)),
            expected(8.0625, 1.0, WALK, 4, None)
        );
        // A change below 0.0002 is no change.
        let mut nudged = walking;
        nudged.motion[1] = 0.99985;
        assert!(r.report(8.125, &nudged, STAMPS).is_none());
    }

    #[test]
    fn moving_or_turning_reports_only_after_more_than_a_fifth_of_a_second() {
        let mut r = PositionReporter::new(0);
        r.report(8.0, &body(1.0, 90.0, [0.0, 1.0, 0.0]), STAMPS)
            .unwrap();
        let moved = body(1.5, 90.0, [0.0, 1.0, 0.0]);
        assert!(r.report(8.125, &moved, STAMPS).is_none());
        assert_eq!(
            bytes(r.report(8.25, &moved, STAMPS)),
            expected(8.25, 1.5, WALK, 0, None)
        );
        // The same place and heading never reports; a turn does, after the window.
        assert!(r.report(9.0, &moved, STAMPS).is_none());
        let turned = body(1.5, 180.0, [0.0, 1.0, 0.0]);
        // 180 degrees is byte 127.
        assert_eq!(
            bytes(r.report(9.0, &turned, STAMPS)),
            expected(9.0, 1.5, 0x7F7F_FF7F, 0, None)
        );
        let turned_more = body(1.5, 270.0, [0.0, 1.0, 0.0]);
        assert!(r.report(9.125, &turned_more, STAMPS).is_none());
        assert!(r.report(9.25, &turned_more, STAMPS).is_some());
    }

    #[test]
    fn a_turn_to_reports_its_target_heading() {
        let mut r = PositionReporter::new(0);
        let mut turning = body(1.0, 10.0, [0.0; 3]);
        turning.turn_to_heading = Some(90.0);
        assert_eq!(
            bytes(r.report(8.0, &turning, STAMPS)),
            expected(8.0, 1.0, STAND, 0, None)
        );
        // The current heading (10) now differs from the sent one (90).
        assert!(r.report(8.25, &turning, STAMPS).is_some());
    }

    #[test]
    fn the_head_direction_reports_more_than_half_a_second_apart() {
        let mut r = PositionReporter::new(0);
        let head = r.look(1.0, false, (0.25, -0.5)).unwrap();
        assert_eq!(
            ClientEvent::LookAtDir {
                x: head.0,
                y: head.1
            }
            .encode(),
            (3, vec![0xC8, 0, 1, 0, 0, 0, 0x80, 0x3E, 0, 0, 0, 0xBF])
        );
        assert!(r.look(1.25, false, (0.5, -0.5)).is_none());
        assert!(r.look(1.5, false, (0.5, -0.5)).is_none());
        assert!(r.look(1.625, true, (0.5, -0.5)).is_none());
        assert!(r.look(1.625, false, (0.2501, -0.5)).is_none());
        assert_eq!(r.look(1.625, false, (0.5, -0.5)), Some((0.5, -0.5)));
    }

    #[test]
    fn headings_compare_within_two_ten_thousandths_of_a_radian() {
        assert!(heading_equals(10.0, 10.005));
        assert!(!heading_equals(10.0, 10.02));
        assert!(heading_equals(359.995, 0.0));
    }
}
