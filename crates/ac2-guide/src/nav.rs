//! Steering towards a map coordinate: where it lies on the landscape (`LandDefs`), how far and
//! which way from the player. The in-world arrow and the route map both read these.

use ac2_physics::landdefs::{game_map_coords_to_gid, LAND_WIDTH, SQUARE_LENGTH};

use crate::guide::MapCoord;

/// The land cell a coordinate names (`LandDefs::game_map_coords_to_gid`, as
/// `Position::FromRealMapCoords` places it); 0 off the map.
pub fn cell(at: MapCoord) -> u32 {
    game_map_coords_to_gid(at.east, at.north)
}

/// Metres east and north from `from` to `to` (one map unit is a hundredth of half the land's
/// width).
pub fn offset(from: MapCoord, to: MapCoord) -> (f64, f64) {
    let metres = f64::from(LAND_WIDTH) * 0.5 * 0.01 * f64::from(SQUARE_LENGTH);
    (
        (f64::from(to.east) - f64::from(from.east)) * metres,
        (f64::from(to.north) - f64::from(from.north)) * metres,
    )
}

/// Ground distance in metres.
pub fn distance(from: MapCoord, to: MapCoord) -> f64 {
    let (e, n) = offset(from, to);
    e.hypot(n)
}

/// Compass bearing from `from` to `to` in degrees clockwise from north, 0..360.
pub fn bearing(from: MapCoord, to: MapCoord) -> f64 {
    let (e, n) = offset(from, to);
    e.atan2(n).to_degrees().rem_euclid(360.0)
}

/// The bearing relative to the way the player faces (a compass heading in degrees): 0 ahead,
/// positive to the right, -180..180.
pub fn relative_bearing(from: MapCoord, facing: f64, to: MapCoord) -> f64 {
    (bearing(from, to) - facing + 180.0).rem_euclid(360.0) - 180.0
}

/// `12.3N 45.6E`, one decimal as the site writes coordinates.
pub fn format(at: MapCoord) -> String {
    let ns = if at.north < 0.0 { 'S' } else { 'N' };
    let ew = if at.east < 0.0 { 'W' } else { 'E' };
    format!("{:.1}{ns} {:.1}{ew}", at.north.abs(), at.east.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    const O: MapCoord = MapCoord {
        north: 0.0,
        east: 0.0,
    };

    #[test]
    fn steering() {
        let ne = MapCoord {
            north: 1.0,
            east: 1.0,
        };
        assert!((bearing(O, ne) - 45.0).abs() < 1e-9);
        assert!(
            (distance(
                O,
                MapCoord {
                    north: 1.0,
                    east: 0.0
                }
            ) - 204.0)
                .abs()
                < 1e-9
        );
        assert!((relative_bearing(O, 90.0, ne) + 45.0).abs() < 1e-9);
        assert!(
            (relative_bearing(
                O,
                0.0,
                MapCoord {
                    north: -1.0,
                    east: -0.001
                }
            )
            .abs()
                - 180.0)
                .abs()
                < 0.1
        );
    }

    #[test]
    fn cells() {
        // The middle of the map: land cell (1020, 1020), block 0x7F7F, cell 4 * 8 + 4 + 1.
        assert_eq!(cell(O), 0x7F7F_0025);
        assert_eq!(
            cell(MapCoord {
                north: 101.0,
                east: 0.0
            }),
            0
        );
        assert_eq!(
            format(MapCoord {
                north: -51.8,
                east: 39.0
            }),
            "51.8S 39.0E"
        );
    }
}
