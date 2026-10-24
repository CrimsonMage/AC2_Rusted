//! Double-precision frame and terrain helpers for the AC2 scenery generator: quaternion frames
//! (`w, x, y, z`, local to block), the landblock's terrain planes with the AC2 SW-NE cut, and
//! `CLandBlock::OnRoad`.

use crate::terrain_cut::ac2_sw_to_ne_cut;

pub(super) type V3 = [f64; 3];
/// Rotation quaternion `(w, x, y, z)` mapping local to block axes.
pub(super) type Q = [f64; 4];

pub(super) const IDENTITY: Q = [1.0, 0.0, 0.0, 0.0];
pub(super) const SIDE: usize = 17;
/// Terrain polygon edge (`LandDefs::square_length / vertex_per_cell`).
pub(super) const POLYGON: f64 = 10.0;
/// `LandDefs::block_length`.
pub(super) const BLOCK_LENGTH: f64 = 160.0;
const POLYGONS: usize = 16;
pub(super) const EPSILON: f64 = 0.0002;

pub(super) fn qmul(a: Q, b: Q) -> Q {
    let [aw, ax, ay, az] = a;
    let [bw, bx, by, bz] = b;
    [
        aw * bw - ax * bx - ay * by - az * bz,
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
    ]
}

/// `q v q*`.
pub(super) fn qrot(q: Q, v: V3) -> V3 {
    let [w, x, y, z] = q;
    let [vx, vy, vz] = v;
    let tx = 2.0 * (y * vz - z * vy);
    let ty = 2.0 * (z * vx - x * vz);
    let tz = 2.0 * (x * vy - y * vx);
    [
        vx + w * tx + y * tz - z * ty,
        vy + w * ty + z * tx - x * tz,
        vz + w * tz + x * ty - y * tx,
    ]
}

/// `Frame::set_heading` on an identity frame: forward (+Y) turns to `(sin h, cos h, 0)`.
pub(super) fn q_heading(degrees: f64) -> Q {
    let half = -degrees.to_radians() / 2.0;
    [half.cos(), 0.0, 0.0, half.sin()]
}

/// Rotation whose columns are the given right/forward/up axes.
pub(super) fn q_from_axes(right: V3, forward: V3, up: V3) -> Q {
    let m = [
        [right[0], forward[0], up[0]],
        [right[1], forward[1], up[1]],
        [right[2], forward[2], up[2]],
    ];
    let trace = m[0][0] + m[1][1] + m[2][2];
    if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        return [
            0.25 * s,
            (m[2][1] - m[1][2]) / s,
            (m[0][2] - m[2][0]) / s,
            (m[1][0] - m[0][1]) / s,
        ];
    }
    // Largest diagonal; ties keep the first, as `max` over `range(3)` does.
    let mut i = 0;
    for k in 1..3 {
        if m[k][k] > m[i][i] {
            i = k;
        }
    }
    let (j, k) = ((i + 1) % 3, (i + 2) % 3);
    let s = (1.0 + m[i][i] - m[j][j] - m[k][k]).sqrt() * 2.0;
    let mut q = [0.0; 4];
    q[0] = (m[k][j] - m[j][k]) / s;
    q[1 + i] = 0.25 * s;
    q[1 + j] = (m[j][i] + m[i][j]) / s;
    q[1 + k] = (m[k][i] + m[i][k]) / s;
    q
}

pub(super) fn normalize(v: V3) -> V3 {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / len, v[1] / len, v[2] / len]
}

pub(super) fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// `Frame::get_heading`: compass degrees of the forward (+Y) axis.
pub(super) fn heading_of(q: Q) -> f64 {
    let forward = qrot(q, [0.0, 1.0, 0.0]);
    forward[0].atan2(forward[1]).to_degrees().rem_euclid(360.0)
}

/// Compass heading pointing down the slope of a plane with normal `n`.
pub(super) fn downhill_heading(n: V3) -> f64 {
    let len = n[0].hypot(n[1]);
    if len < EPSILON {
        return 0.0;
    }
    (7.853_981_633_974_483 - (n[1] / len).atan2(n[0] / len))
        .to_degrees()
        .rem_euclid(360.0)
}

/// The landblock surface the generator samples: 17 x 17 x-major heights and cell-info words.
pub(super) struct Ac2SceneTerrain {
    pub(super) block_x: u32,
    pub(super) block_y: u32,
    pub(super) heights: Vec<f64>,
    pub(super) cell_info: Vec<u32>,
    pub(super) road_width: f64,
}

/// A terrain triangle's upward unit normal and its corners.
pub(super) type TerrainTriangle = (V3, [V3; 3]);

impl Ac2SceneTerrain {
    pub(super) fn height(&self, ix: usize, iy: usize) -> f64 {
        self.heights[ix * SIDE + iy]
    }

    fn road(&self, ix: i64, iy: i64) -> u32 {
        if (0..SIDE as i64).contains(&ix) && (0..SIDE as i64).contains(&iy) {
            (self.cell_info[ix as usize * SIDE + iy as usize] >> 24) & 0xF
        } else {
            0
        }
    }

    fn cut(&self, px: usize, py: usize) -> bool {
        ac2_sw_to_ne_cut(
            (self.block_x * POLYGONS as u32 + px as u32) as i32,
            (self.block_y * POLYGONS as u32 + py as u32) as i32,
        )
    }

    fn corners(&self, px: usize, py: usize) -> [V3; 4] {
        let corner = |cx: usize, cy: usize| {
            [
                cx as f64 * POLYGON,
                cy as f64 * POLYGON,
                self.height(cx, cy),
            ]
        };
        [
            corner(px, py),
            corner(px + 1, py),
            corner(px, py + 1),
            corner(px + 1, py + 1),
        ]
    }

    /// `CLandCell::find_terrain_plane`: the plane `(normal, d)` of the triangle under `(x, y)`.
    pub(super) fn plane(&self, x: f64, y: f64) -> (V3, f64) {
        let px = ((x / POLYGON).floor() as i64).clamp(0, POLYGONS as i64 - 1) as usize;
        let py = ((y / POLYGON).floor() as i64).clamp(0, POLYGONS as i64 - 1) as usize;
        let (fx, fy) = (x / POLYGON - px as f64, y / POLYGON - py as f64);
        let v = self.corners(px, py);
        let tri = if self.cut(px, py) {
            if fx >= fy {
                [0, 1, 3]
            } else {
                [0, 3, 2]
            }
        } else if fx + fy <= 1.0 {
            [0, 1, 2]
        } else {
            [3, 2, 1]
        };
        let (n, a) = triangle_normal(v[tri[0]], v[tri[1]], v[tri[2]]);
        (n, -(n[0] * a[0] + n[1] * a[1] + n[2] * a[2]))
    }

    /// Both triangles of polygon `(px, py)`.
    pub(super) fn polygon_triangles(&self, px: usize, py: usize) -> [TerrainTriangle; 2] {
        let v = self.corners(px, py);
        let tris = if self.cut(px, py) {
            [[0, 1, 3], [0, 3, 2]]
        } else {
            [[0, 1, 2], [3, 2, 1]]
        };
        tris.map(|t| {
            (
                triangle_normal(v[t[0]], v[t[1]], v[t[2]]).0,
                [v[t[0]], v[t[1]], v[t[2]]],
            )
        })
    }

    /// `CLandBlock::OnRoad`: the road bits at the four pixel corners around `(x, y)` define road
    /// strips `road_width` wide.
    pub(super) fn on_road(&self, x: f64, y: f64) -> bool {
        let (l, w) = (POLYGON, self.road_width);
        let mx = l - w;
        let ix = (x / l).floor() as i64;
        let iy = (y / l).floor() as i64;
        let r00 = self.road(ix, iy) != 0;
        let r01 = self.road(ix, iy + 1) != 0;
        let r10 = self.road(ix + 1, iy) != 0;
        let r11 = self.road(ix + 1, iy + 1) != 0;
        if !(r00 || r01 || r10 || r11) {
            return false;
        }
        let dx = x - ix as f64 * l;
        let dy = y - iy as f64 * l;
        match (r00, r01, r10, r11) {
            (false, false, true, false) => (l - dx) + dy < w,
            (false, false, true, true) => dx > mx,
            (false, false, false, _) => (2.0 * l - dx - dy) < w,
            (false, true, true, true) => dx > mx || dy > mx,
            (false, true, true, false) => (dx + dy - l).abs() < w,
            (false, true, false, true) => dy > mx,
            (false, true, false, false) => (dx + l - dy) < w,
            (true, true, true, true) => true,
            (true, true, true, false) => dx < w || dy < w,
            (true, true, false, true) => dx < w || dy > mx,
            (true, true, false, false) => dx < w,
            (true, false, true, false) => dy < w,
            (true, false, true, true) => dx > mx || dy < w,
            (true, false, false, false) => dx + dy < w,
            (true, false, false, true) => (dx - dy).abs() < w,
        }
    }
}

/// Plane height at `(x, y)`, when the plane is not vertical.
pub(super) fn plane_z(plane: (V3, f64), x: f64, y: f64) -> Option<f64> {
    let (n, d) = plane;
    (n[2].abs() > EPSILON).then(|| -(x * n[0] + y * n[1] + d) / n[2])
}

fn triangle_normal(a: V3, b: V3, c: V3) -> (V3, V3) {
    let n = normalize(cross(
        [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
    ));
    (if n[2] < 0.0 { [-n[0], -n[1], -n[2]] } else { n }, a)
}

/// Whether a circle of radius `r` at `(x, y)` touches triangle `t` (in XY).
pub(super) fn circle_in_triangle(t: &[V3; 3], x: f64, y: f64, r: f64) -> bool {
    let side = |p: V3, q: V3| (q[0] - p[0]) * (y - p[1]) - (q[1] - p[1]) * (x - p[0]);
    let (s1, s2, s3) = (side(t[0], t[1]), side(t[1], t[2]), side(t[2], t[0]));
    if (s1 >= 0.0 && s2 >= 0.0 && s3 >= 0.0) || (s1 <= 0.0 && s2 <= 0.0 && s3 <= 0.0) {
        return true;
    }
    [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])]
        .into_iter()
        .any(|(p, q)| {
            let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
            let s = (((x - p[0]) * dx + (y - p[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            (p[0] + s * dx - x).powi(2) + (p[1] + s * dy - y).powi(2) <= r * r
        })
}

#[cfg(test)]
mod tests {
    use super::{downhill_heading, heading_of, q_from_axes, q_heading, qmul, qrot};

    #[test]
    fn heading_round_trips_through_frames() {
        for degrees in [0.0, 26.9, 96.2, 180.0, 346.75] {
            assert!((heading_of(q_heading(degrees)) - degrees).abs() < 1e-9);
        }
        // Forward of a 90 degree heading points east.
        let east = qrot(q_heading(90.0), [0.0, 1.0, 0.0]);
        assert!((east[0] - 1.0).abs() < 1e-12 && east[1].abs() < 1e-12);
        let composed = qmul(q_heading(30.0), q_heading(40.0));
        assert!((heading_of(composed) - 70.0).abs() < 1e-9);
    }

    #[test]
    fn axes_quaternion_reproduces_the_basis() {
        let q = q_from_axes([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let right = qrot(q, [1.0, 0.0, 0.0]);
        assert!((right[1] + 1.0).abs() < 1e-12);
        // A plane rising to the east slopes downhill toward the west (270 degrees).
        assert!((downhill_heading([-0.6, 0.0, 0.8]) - 270.0).abs() < 1e-9);
    }
}
