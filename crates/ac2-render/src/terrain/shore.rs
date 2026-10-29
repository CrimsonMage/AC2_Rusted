//! `LScapeTerrainMesh::GenerateShore`: the strips of breaking water along the near landscape's
//! shores. Every terrain triangle of a near block with water that crosses the water's height gives
//! the two points where it does (`CalculatePointsOfIntersection`, `Plane::IntersectLineSegment`);
//! points that meet are joined, and each chain of them becomes a strip (closed where it comes
//! round). Each point leans out over the water by its triangles' summed normal: the strip runs
//! from 5 cm inland of the shore to `12 * normal.z` metres out, its texture coordinates the
//! distance along the strip and the strip's random phase (`DrawShore` draws them as triangle
//! strips with the shore material).

use ac2_core::Random;
use ac2_math::Vec3;

use crate::decal::GroundTriangle;
use crate::object::ObjectVertex;
use crate::particles::ParticleGeometry;

/// `ShorePointType`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ShorePoint {
    origin: Vec3,
    normal: Vec3,
    far: Vec3,
    phase: f32,
    distance: f32,
    neighbours: [Option<usize>; 2],
    strip: Option<usize>,
}

impl ShorePoint {
    fn new(origin: Vec3) -> Self {
        Self {
            origin,
            normal: Vec3::ZERO,
            far: Vec3::ZERO,
            phase: 0.0,
            distance: 0.0,
            neighbours: [None, None],
            strip: None,
        }
    }

    /// The neighbour other than `from` (`nNeighbourA` unless it is unset or `from`, then
    /// `nNeighbourB` unless it is unset or `from`).
    fn next(&self, from: Option<usize>) -> Option<usize> {
        let [a, b] = self.neighbours;
        if a.is_some() && a != from {
            return a;
        }
        b.filter(|_| b != from)
    }
}

/// `ShoreStripType`: its points in order, whether it closes on itself, and its length.
#[derive(Debug, Clone, PartialEq)]
pub struct ShoreStrip {
    points: Vec<usize>,
    pub circular: bool,
    pub length: f32,
}

/// The shore's strips and their geometry: two vertices a point (the shore, then out over the
/// water), each strip's triangles in `indices[ranges[i]]`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shore {
    pub strips: Vec<ShoreStrip>,
    pub vertices: Vec<ObjectVertex>,
    pub ranges: Vec<std::ops::Range<usize>>,
    pub indices: Vec<u32>,
}

impl Shore {
    /// The strips `DrawShore(true)` draws, as one geometry: all but the closed ones shorter than
    /// 20 m (pools).
    pub fn breaking_waves(&self) -> ParticleGeometry {
        let indices = self
            .strips
            .iter()
            .zip(&self.ranges)
            .filter(|(strip, _)| !strip.circular || strip.length >= 20.0)
            .flat_map(|(_, range)| self.indices[range.clone()].iter().copied())
            .collect();
        ParticleGeometry {
            vertices: self.vertices.clone(),
            indices,
        }
    }
}

/// `Plane::IntersectLineSegment` against the water at `height`: an end within 0.01 of it counts
/// as on it, and only ends on either side beyond that cross.
fn crossing(a: Vec3, b: Vec3, height: f32) -> Option<Vec3> {
    let side = |d: f32| {
        if d > 0.01 {
            0
        } else if d < -0.01 {
            1
        } else {
            2
        }
    };
    let (da, db) = (a.z - height, b.z - height);
    if !matches!((side(da), side(db)), (0, 1) | (1, 0)) {
        return None;
    }
    let t = if (da - db).abs() > 0.0002 {
        f64::from(da) / (f64::from(da) - f64::from(db))
    } else {
        0.0
    };
    let lerp = |p: f32, q: f32| (f64::from(p) + (f64::from(q) - f64::from(p)) * t) as f32;
    Some(Vec3::new(lerp(a.x, b.x), lerp(a.y, b.y), lerp(a.z, b.z)))
}

/// `G_CalculateNormal`: `(v1 - v0) x (v2 - v0)`, normalised.
fn triangle_normal(v: [Vec3; 3]) -> Vec3 {
    (v[1] - v[0]).cross(v[2] - v[0]).normalize_or_zero()
}

/// `FindShorePointIndexByOrigin`: a point within 0.0002 on every axis.
fn find_point(points: &[ShorePoint], origin: Vec3) -> Option<usize> {
    points
        .iter()
        .position(|p| (p.origin - origin).abs().cmplt(Vec3::splat(0.0002)).all())
}

/// `LScapeTerrainMesh::GenerateShore` over the near blocks' ground triangles, each block with
/// its water height; `centre` is the centre block's water height, if it has water (a block whose
/// water lies more than 0.1 under it makes no shore). `random` rolls each strip's phase.
pub fn generate_shore(
    blocks: &[(f32, Vec<GroundTriangle>)],
    centre: Option<f32>,
    random: &mut Random,
) -> Shore {
    let mut points: Vec<ShorePoint> = Vec::new();
    for (height, triangles) in blocks {
        if centre.is_some_and(|centre| height + 0.1 < centre) {
            continue;
        }
        for triangle in triangles {
            let v = triangle.map(|corner| Vec3::from_array(corner.position));
            let above = v.iter().any(|p| *height <= p.z);
            let below = v.iter().any(|p| p.z <= *height);
            if !(above && below) {
                continue;
            }
            // `CalculatePointsOfIntersection`: edges 0-1, 1-2, 2-0; the first two that cross.
            let mut ends = [0, 1, 2]
                .into_iter()
                .filter_map(|i| crossing(v[i], v[(i + 1) % 3], *height));
            let (Some(first), Some(second)) = (ends.next(), ends.next()) else {
                continue;
            };
            let normal = triangle_normal(v);
            let ids = [first, second].map(|origin| {
                find_point(&points, origin).unwrap_or_else(|| {
                    points.push(ShorePoint::new(origin));
                    points.len() - 1
                })
            });
            for (index, other) in [(ids[0], ids[1]), (ids[1], ids[0])] {
                let point = &mut points[index];
                point.normal += normal;
                if point.neighbours[0].is_none() {
                    point.neighbours[0] = Some(other);
                } else {
                    point.neighbours[1] = Some(other);
                }
            }
        }
    }
    let strips = chain_strips(&mut points);
    finish(points, strips, random)
}

/// The strips: from each point not yet in one, back to an end of its chain (or round to itself,
/// a closed strip), then forward collecting the points; a closed strip repeats its first point.
fn chain_strips(points: &mut Vec<ShorePoint>) -> Vec<ShoreStrip> {
    let mut strips = Vec::new();
    let mut start = 0;
    while start < points.len() {
        if points[start].strip.is_some() {
            start += 1;
            continue;
        }
        // Walk back to an open end; coming round to the start closes the strip.
        let mut circular = false;
        let mut previous = None;
        let mut at = Some(start);
        let mut end = start;
        while let Some(current) = at {
            let next = points[current].next(previous);
            previous = Some(current);
            end = current;
            if next == Some(start) {
                circular = true;
                break;
            }
            at = next;
        }
        let index = strips.len();
        let mut chain = Vec::new();
        let mut from = None;
        let mut at = Some(end);
        while let Some(current) = at {
            chain.push(current);
            points[current].strip = Some(index);
            let next = points[current].next(from);
            from = Some(current);
            if next == Some(end) {
                break;
            }
            at = next;
        }
        if circular {
            let first = chain[0];
            let copy = ShorePoint {
                neighbours: [Some(first), None],
                ..points[first]
            };
            points.push(copy);
            chain.push(points.len() - 1);
        }
        strips.push(ShoreStrip {
            points: chain,
            circular,
            length: 0.0,
        });
        start += 1;
    }
    strips
}

/// Each strip's length, phase and far edge, then the two vertices a point and each strip's
/// triangles.
fn finish(mut points: Vec<ShorePoint>, mut strips: Vec<ShoreStrip>, random: &mut Random) -> Shore {
    for strip in &mut strips {
        let phase = random.roll_dice_f32(0.0, 0.8);
        let mut length = 0.0_f32;
        for (k, &index) in strip.points.iter().enumerate() {
            if k > 0 {
                length += points[index]
                    .origin
                    .distance(points[strip.points[k - 1]].origin);
            }
            let point = &mut points[index];
            let normal = point.normal.normalize_or_zero();
            let reach = normal.z * 12.0;
            let flat = Vec3::new(normal.x, normal.y, 0.0).normalize_or_zero();
            point.normal = flat;
            point.origin -= flat * 0.05;
            point.far = point.origin + flat * reach;
            point.distance = length;
            point.phase = phase;
        }
        strip.length = length;
    }
    let vertex = |position: Vec3, distance: f32, v0: f32, v1: f32| ObjectVertex {
        position: position.to_array(),
        normal: [0.0, 0.0, 1.0],
        color: [1.0; 4],
        uvs: [[distance, v0], [distance, v1], [0.0; 2], [0.0; 2]],
        tangents: [[0.0; 3]; 2],
    };
    let vertices = points
        .iter()
        .flat_map(|p| {
            [
                vertex(p.origin, p.distance, p.phase, 0.0),
                vertex(p.far, p.distance, p.phase + 1.0, 1.0),
            ]
        })
        .collect();
    let mut indices = Vec::new();
    let mut ranges = Vec::with_capacity(strips.len());
    for strip in &strips {
        let start = indices.len();
        // The strip `2p, 2p + 1, 2q, 2q + 1, ...` as triangles: each pair of points gives the
        // strip's even triangle and its odd one turned back to the same winding.
        for pair in strip.points.windows(2) {
            let (a, b) = (pair[0] as u32 * 2, pair[1] as u32 * 2);
            indices.extend([a, a + 1, b, b, a + 1, b + 1]);
        }
        ranges.push(start..indices.len());
    }
    Shore {
        strips,
        vertices,
        ranges,
        indices,
    }
}

#[cfg(test)]
#[path = "shore_tests.rs"]
mod tests;
