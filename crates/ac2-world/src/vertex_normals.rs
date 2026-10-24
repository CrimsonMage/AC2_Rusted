//! Landblock vertex normals as `CLandBlockData::CacheFarValues` builds them: the normalised sum of
//! the unit normals of every terrain triangle touching the vertex, where each of the four
//! surrounding polygons contributes one or two triangles depending on its diagonal. Edge vertices
//! read the neighbouring blocks (`GetNeighborHeightValue`, `GetNeighborIsSWtoNECut`).

/// Normals of a `side` x `side` vertex grid (x-major). `height(x, y)` and `cut(x, y)` take block
/// vertex/polygon coordinates that may step one past the block on any side.
pub(crate) fn vertex_normals(
    side: usize,
    pixel_length: f32,
    height: impl Fn(i32, i32) -> f32,
    cut: impl Fn(i32, i32) -> bool,
) -> Vec<[f32; 3]> {
    let triangle = |p1: (i32, i32), p2: (i32, i32), p3: (i32, i32)| {
        normal_from_triangle(pixel_length, &height, p1, p2, p3)
    };
    let mut normals = Vec::with_capacity(side * side);
    for x in 0..side as i32 {
        for y in 0..side as i32 {
            let c = (x, y);
            let (n, s, e, w) = ((x, y + 1), (x, y - 1), (x + 1, y), (x - 1, y));
            let (ne, nw, se, sw) = (
                (x + 1, y + 1),
                (x - 1, y + 1),
                (x + 1, y - 1),
                (x - 1, y - 1),
            );
            let mut faces = Vec::with_capacity(8);
            // The polygons south-west, south-east, north-west and north-east of the vertex. A
            // diagonal through the vertex splits that polygon's corner into two triangles.
            if cut(x - 1, y - 1) {
                faces.extend([triangle(c, w, sw), triangle(sw, s, c)]);
            } else {
                faces.push(triangle(s, c, w));
            }
            if cut(x, y - 1) {
                faces.push(triangle(e, c, s));
            } else {
                faces.extend([triangle(se, e, c), triangle(c, s, se)]);
            }
            if cut(x - 1, y) {
                faces.push(triangle(w, c, n));
            } else {
                faces.extend([triangle(nw, w, c), triangle(c, n, nw)]);
            }
            if cut(x, y) {
                faces.extend([triangle(c, e, ne), triangle(ne, n, c)]);
            } else {
                faces.push(triangle(n, c, e));
            }
            let sum = faces.iter().fold([0.0; 3], |acc, f| {
                [acc[0] + f[0], acc[1] + f[1], acc[2] + f[2]]
            });
            normals.push(normalize(sum));
        }
    }
    normals
}

/// `CLandBlockData::GetNormalFromTriangle`: `normalize(cross(p2 - p1, p3 - p1))` over
/// (x, y, height), grid steps scaled by `pixel_length`.
fn normal_from_triangle(
    pixel_length: f32,
    height: &impl Fn(i32, i32) -> f32,
    p1: (i32, i32),
    p2: (i32, i32),
    p3: (i32, i32),
) -> [f32; 3] {
    let h1 = height(p1.0, p1.1);
    let u = [
        (p2.0 - p1.0) as f32 * pixel_length,
        (p2.1 - p1.1) as f32 * pixel_length,
        height(p2.0, p2.1) - h1,
    ];
    let v = [
        (p3.0 - p1.0) as f32 * pixel_length,
        (p3.1 - p1.1) as f32 * pixel_length,
        height(p3.0, p3.1) - h1,
    ];
    normalize([
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ])
}

/// `D3DXVec3Normalize`: a zero vector stays zero.
fn normalize([x, y, z]: [f32; 3]) -> [f32; 3] {
    let length = (x * x + y * y + z * z).sqrt();
    if length > 0.0 {
        [x / length, y / length, z / length]
    } else {
        [0.0; 3]
    }
}

#[cfg(test)]
mod tests {
    use super::vertex_normals;

    #[test]
    fn flat_ground_points_up_whatever_the_diagonals() {
        for cut in [false, true] {
            let normals = vertex_normals(17, 10.0, |_, _| 50.0, |_, _| cut);
            assert!(normals.iter().all(|n| *n == [0.0, 0.0, 1.0]));
        }
    }

    #[test]
    fn a_slope_rising_east_tilts_normals_west() {
        // 1 m up per 10 m east.
        let normals = vertex_normals(17, 10.0, |x, _| x as f32, |x, y| (x + y) % 2 == 0);
        let expected = [-0.1 / 1.01_f32.sqrt(), 0.0, 1.0 / 1.01_f32.sqrt()];
        for n in normals {
            for (a, e) in n.iter().zip(expected) {
                assert!((a - e).abs() < 1e-5, "{n:?}");
            }
        }
    }
}
