//! The floating arrow's target and view (`ac2_ui::guide_arrow`, a modern addition): the Tracker's
//! picked match or a map coordinate pressed in chat, turned into the bearing relative to where the
//! camera looks and a label of the target's coordinates and distance.

use ac2_ui::guide_arrow::GuideArrowView;

use crate::frame::ClientFrame;

/// What the arrow points at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ArrowTarget {
    /// A world object (the Tracker's pick), followed as it moves.
    Object(u64),
    /// A place on the map: percent of the half map north and east of the middle.
    Place { north: f32, east: f32 },
}

/// Metres east and north of the world's south-west corner: a landblock is 160 m and an origin is
/// relative to its cell's landblock.
fn world_metres(cell: u32, origin: [f32; 3]) -> (f64, f64) {
    let bx = f64::from((cell >> 24) & 0xFF);
    let by = f64::from((cell >> 16) & 0xFF);
    (
        bx * 160.0 + f64::from(origin[0]),
        by * 160.0 + f64::from(origin[1]),
    )
}

/// A map coordinate's metres (a map unit is a hundredth of half the land's 2040 cells of 20 m).
fn place_metres(north: f32, east: f32) -> (f64, f64) {
    let metres = |v: f32| (f64::from(v) + 100.0) * 204.0;
    (metres(east), metres(north))
}

/// `12.3N, 45.6E`, as the radar writes coordinates.
pub(crate) fn format_place(north: f32, east: f32) -> String {
    let ns = if north < 0.0 { 'S' } else { 'N' };
    let ew = if east < 0.0 { 'W' } else { 'E' };
    format!("{:.1}{ns}, {:.1}{ew}", north.abs(), east.abs())
}

fn format_distance(metres: f64) -> String {
    if metres < 1000.0 {
        format!("{metres:.0} m")
    } else {
        format!("{:.1} km", metres / 1000.0)
    }
}

/// The compass heading the view faces (degrees clockwise from north): the camera's local +y
/// turned by its rotation, else the player's heading.
fn view_heading(frame: &ClientFrame) -> Option<f64> {
    if let Some(camera) = &frame.camera {
        let [w, x, y, z] = camera.rotation.map(f64::from);
        let vx = 2.0 * (x * y - w * z);
        let vy = 1.0 - 2.0 * (x * x + z * z);
        if vx != 0.0 || vy != 0.0 {
            return Some(vx.atan2(vy).to_degrees());
        }
    }
    frame.player.as_ref().map(|p| f64::from(p.heading_degrees))
}

/// The arrow's view of `target` and the planar distance to it; `None` without a player or when
/// the object is not in the frame.
pub(crate) fn arrow_view(
    frame: &ClientFrame,
    target: ArrowTarget,
) -> Option<(GuideArrowView, f64)> {
    let player = frame.player.as_ref()?;
    let (px, py) = world_metres(player.cell, player.origin);
    let ((tx, ty), name) = match target {
        ArrowTarget::Object(id) => {
            let object = frame.objects.iter().find(|o| o.id == id)?;
            let at = ac2_macro::tracker::coordinates_text(object.cell);
            let name = object.name.as_deref().unwrap_or("Target");
            let label = match at {
                Some(at) => format!("{name} · {at}"),
                None => name.to_owned(),
            };
            (world_metres(object.cell, object.origin), label)
        }
        ArrowTarget::Place { north, east } => {
            (place_metres(north, east), format_place(north, east))
        }
    };
    let (dx, dy) = (tx - px, ty - py);
    let distance = dx.hypot(dy);
    let bearing = dx.atan2(dy).to_degrees();
    let relative = (bearing - view_heading(frame)? + 180.0).rem_euclid(360.0) - 180.0;
    Some((
        GuideArrowView {
            relative_degrees: relative as f32,
            label: format!("{name} · {}", format_distance(distance)),
        },
        distance,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_and_cells_share_one_frame() {
        // Land cell (1020, 1020)'s south-west corner is the map's middle: 0.0N 0.0E.
        let (x, y) = world_metres(0x7F7F_0025, [80.0, 80.0, 0.0]);
        let (mx, my) = place_metres(0.0, 0.0);
        assert!(
            (x - (127.0 * 160.0 + 80.0)).abs() < 1e-9 && (x - mx).abs() < 0.01,
            "{x} {mx}"
        );
        assert!((y - my).abs() < 0.01);
        assert_eq!(format_place(-51.8, 39.0), "51.8S, 39.0E");
        assert_eq!(format_distance(1234.0), "1.2 km");
    }
}
