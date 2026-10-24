use std::time::{Duration, Instant};

use ac2_net::protocol::{CellPosition, ClientPosition};

/// Shortest interval between movement reports.
const MIN_INTERVAL: Duration = Duration::from_millis(100);
/// Longest interval without a report while the body is moving.
const MAX_INTERVAL: Duration = Duration::from_secs(1);
/// Displacement that alone justifies a report, metres.
const MOVE_THRESHOLD: f32 = 0.05;
/// Heading change that alone justifies a report, degrees.
const TURN_THRESHOLD: f32 = 1.5;

/// Decides when the player's body state becomes a `CPositionPack` report, and stamps it.
#[derive(Debug)]
pub struct PositionReporter {
    started: Instant,
    last_time: f64,
    last: Option<(Instant, CellPosition, f32, [f32; 3])>,
    /// Instance, teleport, forced-position and move-to stamps echoed in every report.
    pub stamps: [u16; 4],
}

impl PositionReporter {
    pub fn new(instance: u16, now: Instant) -> Self {
        Self {
            started: now,
            last_time: 0.0,
            last: None,
            stamps: [instance, 0, 0, 0],
        }
    }

    /// The report for this body state, or `None` while nothing changed enough. Physics time is
    /// strictly increasing, as the server requires.
    pub fn report(
        &mut self,
        now: Instant,
        position: CellPosition,
        heading_degrees: f32,
        motion: [f32; 3],
        jump_velocity: Option<[f32; 3]>,
    ) -> Option<ClientPosition> {
        if let Some((at, last_position, last_heading, last_motion)) = self.last {
            let since = now.saturating_duration_since(at);
            if since < MIN_INTERVAL && jump_velocity.is_none() {
                return None;
            }
            let moved = last_position.cell != position.cell
                || last_position
                    .origin
                    .iter()
                    .zip(position.origin)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f32>()
                    > MOVE_THRESHOLD * MOVE_THRESHOLD;
            let turn = (heading_degrees - last_heading + 540.0).rem_euclid(360.0) - 180.0;
            let changed = moved
                || turn.abs() > TURN_THRESHOLD
                || motion != last_motion
                || jump_velocity.is_some();
            let moving = motion != [0.0; 3];
            if !changed && !(moving && since >= MAX_INTERVAL) {
                return None;
            }
        }
        let mut time = now.saturating_duration_since(self.started).as_secs_f64();
        if time <= self.last_time {
            time = self.last_time + 0.001;
        }
        self.last_time = time;
        self.last = Some((now, position, heading_degrees, motion));
        Some(ClientPosition {
            time,
            position,
            heading_degrees,
            motion,
            cancel_moveto: false,
            stamps: self.stamps,
            jump_velocity,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::PositionReporter;
    use ac2_net::protocol::CellPosition;

    fn at(x: f32) -> CellPosition {
        CellPosition {
            cell: 0x0299_0109,
            origin: [x, 0.0, 0.0],
        }
    }

    #[test]
    fn reports_on_change_with_increasing_time_and_rate_limits() {
        let now = Instant::now();
        let mut reporter = PositionReporter::new(3, now);
        let first = reporter.report(now, at(0.0), 0.0, [0.0; 3], None).unwrap();
        assert_eq!(first.stamps, [3, 0, 0, 0]);
        assert!(reporter
            .report(
                now + Duration::from_millis(50),
                at(1.0),
                0.0,
                [0.0; 3],
                None
            )
            .is_none());
        let moved = reporter
            .report(
                now + Duration::from_millis(150),
                at(1.0),
                0.0,
                [0.0, 1.0, 0.0],
                None,
            )
            .unwrap();
        assert!(moved.time > first.time);
        assert!(reporter
            .report(
                now + Duration::from_millis(300),
                at(1.0),
                0.0,
                [0.0, 1.0, 0.0],
                None
            )
            .is_none());
        assert!(reporter
            .report(
                now + Duration::from_millis(1200),
                at(1.0),
                0.0,
                [0.0, 1.0, 0.0],
                None
            )
            .is_some());
        assert!(reporter
            .report(now + Duration::from_secs(5), at(1.0), 0.0, [0.0; 3], None)
            .is_some());
        assert!(reporter
            .report(now + Duration::from_secs(9), at(1.0), 0.0, [0.0; 3], None)
            .is_none());
    }
}
