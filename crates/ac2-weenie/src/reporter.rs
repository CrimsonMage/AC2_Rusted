//! `PlayerMovementModule`: when the player's body becomes a `CPositionPack` report, and when
//! the head direction becomes a `CLookAtDir`. Both are checked every frame
//! (`PlayerMovementModule::UseTime`).

use ac2_net::protocol::{CellPosition, ClientPosition};

/// The player body facts the reports read, as plain data from physics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyState {
    /// `CPhysicsObj::get_position`: cell and origin.
    pub position: CellPosition,
    /// `CPhysicsObj::get_heading`, degrees in `[0, 360)`.
    pub heading_degrees: f32,
    /// The motion interpreter's last do-motion values.
    pub motion: [f32; 3],
    /// `CPhysicsObj::m_nTimesJumped`.
    pub times_jumped: u32,
    /// `CPhysicsObj::get_velocity`, read when a jump is reported.
    pub velocity: [f32; 3],
    /// `PlayerMovementModule::m_is_actively_moving`.
    pub actively_moving: bool,
    pub instance_stamp: u16,
    pub teleport_stamp: u16,
    pub force_position_stamp: u16,
    pub moveto_stamp: u16,
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

/// `PlayerMovementModule`'s report state.
#[derive(Debug, Default)]
pub struct PositionReporter {
    last: Option<ClientPosition>,
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

    /// `ShouldSendPositionEvent` then `SendPositionEvent`: the report to send at `cur_time`, if
    /// any. A new jump or a motion change reports at once; otherwise a moved or turned body
    /// reports once more than 0.2 s passed.
    pub fn report(&mut self, cur_time: f64, body: &BodyState) -> Option<ClientPosition> {
        if self.jump_stamp > body.times_jumped {
            return None;
        }
        if self.jump_stamp == body.times_jumped {
            if let Some(last) = self.last {
                let motion_changed = (0..3).any(|i| differs(body.motion[i], last.motion[i]));
                if !motion_changed {
                    if cur_time - self.last_send_time <= POSITION_SECONDS {
                        return None;
                    }
                    let same_place = body.position.cell == last.position.cell
                        && !(0..3)
                            .any(|i| differs(body.position.origin[i], last.position.origin[i]));
                    if same_place && heading_equals(body.heading_degrees, last.heading_degrees) {
                        return None;
                    }
                }
            }
        }
        let jump = self.jump_stamp < body.times_jumped;
        if jump {
            self.jump_stamp += 1;
        }
        let report = ClientPosition {
            time: cur_time,
            position: body.position,
            heading_degrees: body.heading_degrees,
            motion: body.motion,
            cancel_moveto: body.actively_moving,
            instance_stamp: body.instance_stamp,
            teleport_stamp: body.teleport_stamp,
            force_position_stamp: body.force_position_stamp,
            moveto_stamp: body.moveto_stamp,
            jump_velocity: jump.then_some(body.velocity),
        };
        self.last = Some(report);
        self.last_send_time = cur_time;
        Some(report)
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
    use super::{heading_equals, BodyState, PositionReporter};
    use ac2_net::protocol::CellPosition;

    fn body(x: f32, heading: f32, motion: [f32; 3]) -> BodyState {
        BodyState {
            position: CellPosition {
                cell: 0x0299_0109,
                origin: [x, 0.0, 0.0],
            },
            heading_degrees: heading,
            motion,
            times_jumped: 0,
            velocity: [0.0, 0.0, 4.0],
            actively_moving: false,
            instance_stamp: 3,
            teleport_stamp: 0,
            force_position_stamp: 0,
            moveto_stamp: 0,
        }
    }

    #[test]
    fn motion_changes_report_at_once_and_drift_every_fifth_of_a_second() {
        let mut r = PositionReporter::new(0);
        let first = r.report(0.0, &body(0.0, 0.0, [0.0; 3])).unwrap();
        assert_eq!((first.time, first.instance_stamp), (0.0, 3));
        // Standing still: nothing.
        assert!(r.report(5.0, &body(0.0, 0.0, [0.0; 3])).is_none());
        // Starting to walk reports immediately.
        assert!(r.report(5.0, &body(0.0, 0.0, [0.0, 1.0, 0.0])).is_some());
        // Moving with the same motion: only after more than 0.2 s.
        assert!(r.report(5.125, &body(1.0, 0.0, [0.0, 1.0, 0.0])).is_none());
        assert!(r.report(5.25, &body(1.0, 0.0, [0.0, 1.0, 0.0])).is_some());
        // A turn alone counts after 0.2 s too.
        assert!(r.report(5.5, &body(1.0, 10.0, [0.0, 1.0, 0.0])).is_some());
    }

    #[test]
    fn a_jump_reports_its_velocity_once() {
        let mut r = PositionReporter::new(0);
        r.report(0.0, &body(0.0, 0.0, [0.0; 3]));
        let mut jumping = body(0.0, 0.0, [0.0; 3]);
        jumping.times_jumped = 1;
        let report = r.report(0.05, &jumping).unwrap();
        assert_eq!(report.jump_velocity, Some([0.0, 0.0, 4.0]));
        assert!(r.report(0.06, &jumping).is_none());
    }

    #[test]
    fn headings_compare_within_two_ten_thousandths_of_a_radian() {
        assert!(heading_equals(10.0, 10.005));
        assert!(!heading_equals(10.0, 10.02));
        assert!(heading_equals(359.995, 0.0));
    }
}
