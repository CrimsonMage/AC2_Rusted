//! `CLandBlock::OnRoad`: whether a point of a landblock lies on a road strip.

use super::geometry::{Ac2SceneTerrain, POLYGON};

/// Rounds a value to the `float` a retail local stores it in; the expression itself is evaluated
/// at x87's 53-bit precision.
fn f32r(value: f64) -> f64 {
    f64::from(value as f32)
}

impl Ac2SceneTerrain {
    /// `CLandBlock::OnRoad(loc, dx)` (Legions 0x00481BE0; admin 7148 0x004883D0 is the same body).
    /// The four road bits of the polygon's corners pick one of sixteen point tests, each a strip
    /// `road_width` wide along the polygon's edges and diagonal.
    pub(super) fn on_road(&self, x: f64, y: f64) -> bool {
        // `LandDefs::pixel_length` and `road_width` are floats, as is `fVar2 = pixel_length -
        // road_width`.
        let l = f64::from(POLYGON as f32);
        let w = self.road_width;
        let mx = f32r(l - w);
        // `ix = floor((1.0 / pixel_length) * loc->x)`: the reciprocal is multiplied, not divided.
        let inv = f64::from(1.0_f32 / POLYGON as f32);
        let (x, y) = (f64::from(x as f32), f64::from(y as f32));
        let ix = (inv * x).floor() as i64;
        let iy = (inv * y).floor() as i64;
        let r00 = self.road(ix, iy) != 0;
        let r01 = self.road(ix, iy + 1) != 0;
        let r10 = self.road(ix + 1, iy) != 0;
        let r11 = self.road(ix + 1, iy + 1) != 0;
        if !(r00 || r01 || r10 || r11) {
            return false;
        }
        // `fVar3 = loc->x - (float)ix * pixel_length`, `fVar4` likewise for y.
        let dx = f32r(x - ix as f64 * l);
        let dy = f32r(y - iy as f64 * l);
        match (r00, r01, r10, r11) {
            (false, false, true, false) => (l - dx) + dy < w,
            (false, false, true, true) => dx > mx,
            (false, false, false, true) => f32r(2.0 * l - dx) - dy < w,
            (false, false, false, false) => false,
            (false, true, true, true) => dx > mx || dy > mx,
            (false, true, true, false) => f32r((dy + dx) - l).abs() < w,
            (false, true, false, true) => dy > mx,
            (false, true, false, false) => f32r(dx + l) - dy < w,
            (true, true, true, true) => true,
            (true, true, true, false) => dx < w || dy < w,
            (true, true, false, true) => dx < w || dy > mx,
            (true, true, false, false) => dx < w,
            (true, false, true, false) => dy < w,
            (true, false, true, true) => dx > mx || dy < w,
            (true, false, false, false) => dx + dy < w,
            (true, false, false, true) => f32r(dx - dy).abs() < w,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ac2_scene::geometry::{Ac2SceneTerrain, SIDE};

    /// A landblock whose polygon `(2, 3)` has the road bits `roads` at its corners `(r00, r01,
    /// r10, r11)` = vertices `(2,3)`, `(2,4)`, `(3,3)`, `(3,4)`. The road field sits at bit 24,
    /// four bits wide, as the DAT's `LandDefs` lays it out. Road width is 2, so the strip edge
    /// sits at `dx, dy = 8` (`pixel_length - road_width`).
    fn terrain(roads: [bool; 4]) -> Ac2SceneTerrain {
        let mut cell_info = vec![0u32; SIDE * SIDE];
        for (on, (ix, iy)) in roads.into_iter().zip([(2, 3), (2, 4), (3, 3), (3, 4)]) {
            if on {
                cell_info[ix * SIDE + iy] = 1 << 24;
            }
        }
        Ac2SceneTerrain {
            block_x: 0,
            block_y: 0,
            heights: vec![0.0; SIDE * SIDE],
            cell_info,
            road_width: 2.0,
            road_field: (24, 0xF),
        }
    }

    /// Each case: the corner road bits, then sample points `(dx, dy, on road)` inside polygon
    /// `(2, 3)`, derived from the Legions branch for those bits. Samples stay clear of the strip
    /// edges, so the float rounding does not decide them.
    #[test]
    fn sixteen_corner_cases_match_retail_branches() {
        let cases: [([bool; 4], &[(f64, f64, bool)]); 16] = [
            (
                [false, false, true, false],
                &[
                    (9.5, 0.5, true),
                    (9.0, 0.5, true),
                    (5.0, 5.0, false),
                    (0.5, 0.5, false),
                ],
            ),
            (
                [false, false, true, true],
                &[
                    (9.0, 5.0, true),
                    (9.0, 0.5, true),
                    (5.0, 5.0, false),
                    (8.0, 5.0, false),
                ],
            ),
            (
                [false, false, false, true],
                &[(9.5, 9.0, true), (5.0, 5.0, false), (9.0, 9.0, false)],
            ),
            (
                [false, true, true, true],
                &[
                    (9.0, 1.0, true),
                    (1.0, 9.0, true),
                    (5.0, 5.0, false),
                    (1.0, 1.0, false),
                ],
            ),
            (
                [false, true, true, false],
                &[
                    (5.0, 5.0, true),
                    (4.5, 6.0, true),
                    (1.0, 1.0, false),
                    (9.0, 9.0, false),
                ],
            ),
            (
                [false, true, false, true],
                &[(1.0, 9.0, true), (5.0, 5.0, false), (9.0, 1.0, false)],
            ),
            (
                [false, true, false, false],
                &[(1.0, 9.5, true), (1.0, 5.0, false), (9.0, 9.0, false)],
            ),
            (
                [true, true, true, true],
                &[(5.0, 5.0, true), (0.5, 9.5, true), (9.5, 0.5, true)],
            ),
            (
                [true, true, true, false],
                &[
                    (1.0, 5.0, true),
                    (5.0, 1.0, true),
                    (5.0, 5.0, false),
                    (9.0, 9.0, false),
                ],
            ),
            (
                [true, true, false, true],
                &[
                    (1.0, 5.0, true),
                    (5.0, 9.5, true),
                    (9.0, 9.0, true),
                    (9.0, 5.0, false),
                    (5.0, 5.0, false),
                ],
            ),
            (
                [true, true, false, false],
                &[(1.0, 5.0, true), (5.0, 1.0, false), (5.0, 9.0, false)],
            ),
            (
                [true, false, true, false],
                &[(5.0, 1.0, true), (1.0, 5.0, false), (5.0, 5.0, false)],
            ),
            (
                [true, false, true, true],
                &[
                    (9.0, 5.0, true),
                    (5.0, 1.0, true),
                    (5.0, 5.0, false),
                    (1.0, 5.0, false),
                ],
            ),
            (
                [true, false, false, false],
                &[(0.5, 0.5, true), (1.0, 1.0, false), (5.0, 5.0, false)],
            ),
            (
                [true, false, false, true],
                &[
                    (5.0, 5.0, true),
                    (5.0, 9.0, false),
                    (2.0, 1.0, true),
                    (1.0, 9.0, false),
                ],
            ),
            (
                [false, false, false, false],
                &[(5.0, 5.0, false), (0.5, 0.5, false), (9.5, 9.5, false)],
            ),
        ];
        for (roads, samples) in cases {
            let t = terrain(roads);
            for &(dx, dy, expected) in samples {
                assert_eq!(
                    t.on_road(20.0 + dx, 30.0 + dy),
                    expected,
                    "roads {roads:?} at dx {dx} dy {dy}"
                );
            }
        }
    }

    /// A `pixel_length` multiple belongs to the polygon it starts: the reciprocal product floors
    /// `(20, 30)` into polygon `(2, 3)`, where the lone road corner sits. The polygon to its west
    /// sees that corner only on its far edge, which `(15, 25)` (`dx = dy = 5`) is not on.
    #[test]
    fn exact_polygon_edge_indexes_the_next_polygon() {
        let t = terrain([true, false, false, false]);
        assert!(t.on_road(20.0, 30.0));
        assert!(!t.on_road(15.0, 25.0));
    }

    /// The road field is read from the DAT's offset and mask, not a fixed bit position.
    #[test]
    fn road_field_follows_the_dat_layout() {
        let mut t = terrain([true, false, false, false]);
        assert!(t.on_road(20.0, 30.0));
        t.road_field = (3, 0x3);
        assert!(!t.on_road(20.0, 30.0));
        t.cell_info[2 * SIDE + 3] = 1 << 3;
        assert!(t.on_road(20.0, 30.0));
    }
}
