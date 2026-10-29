use super::*;
use crate::decal::GroundVertex;

/// The two triangles of each 1 m polygon of a `side` x `side` grid of heights `height(x, y)`,
/// split as the near mesh splits an uncut polygon.
fn grid(side: usize, height: impl Fn(f32, f32) -> f32) -> Vec<GroundTriangle> {
    let corner = |x: usize, y: usize| GroundVertex {
        position: [x as f32, y as f32, height(x as f32, y as f32)],
        normal: [0.0, 0.0, 1.0],
    };
    let mut out = Vec::new();
    for y in 0..side {
        for x in 0..side {
            let (sw, se, nw, ne) = (
                corner(x, y),
                corner(x + 1, y),
                corner(x, y + 1),
                corner(x + 1, y + 1),
            );
            out.push([sw, se, nw]);
            out.push([se, ne, nw]);
        }
    }
    out
}

#[test]
fn a_slope_into_the_water_makes_one_open_strip_leaning_out_over_the_water() {
    // The ground rises east: below the water (height 0) west of x = 3.5.
    let triangles = grid(8, |x, _| x - 3.5);
    let shore = generate_shore(&[(0.0, triangles)], None, &mut Random::default());
    assert_eq!(shore.strips.len(), 1, "{:?}", shore.strips);
    let strip = &shore.strips[0];
    assert!(!strip.circular);
    // Along the whole 8 m edge of the grid (each step measured from the last point after its
    // 5 cm inland shift, as retail measures it).
    assert!((strip.length - 8.0).abs() < 0.1, "{}", strip.length);
    // Each point sits 5 cm inland of x = 3.5 and reaches out west over the water.
    for range in &shore.ranges {
        assert!(!range.is_empty());
    }
    for pair in shore.vertices.chunks(2) {
        let (shore_edge, far) = (pair[0].position, pair[1].position);
        assert!((shore_edge[0] - 3.55).abs() < 1e-3, "{shore_edge:?}");
        assert!(far[0] < shore_edge[0], "{far:?} reaches over the water");
        assert_eq!(pair[1].uvs[0][1], pair[0].uvs[0][1] + 1.0);
    }
}

#[test]
fn an_island_makes_a_closed_strip_that_small_pools_leave_out() {
    // A pyramid island 3 m across at the water line (0.5; the vertices stay off it, which
    // `Plane::IntersectLineSegment` counts as no crossing).
    let triangles = grid(8, |x, y| 2.0 - (x - 4.0).abs().max((y - 4.0).abs()));
    let shore = generate_shore(&[(0.5, triangles)], None, &mut Random::default());
    assert_eq!(shore.strips.len(), 1);
    assert!(shore.strips[0].circular);
    // About 12 m round (the diagonals cut the corners), under 20 m: `DrawShore(true)` leaves it
    // out.
    let length = shore.strips[0].length;
    assert!((10.0..20.0).contains(&length), "{length}");
    assert!(shore.breaking_waves().indices.is_empty());
}

#[test]
fn a_block_whose_water_lies_under_the_centres_makes_no_shore() {
    let triangles = grid(8, |x, _| x - 3.5);
    let shore = generate_shore(&[(0.0, triangles)], Some(0.5), &mut Random::default());
    assert!(shore.strips.is_empty());
}
