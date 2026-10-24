//! Grass frills as `FrillRender::RefreshFrillMesh` merges them: every frill of the landscape cells
//! within `m_current_lscape_radius` of the camera's cell whose xy distance from that cell's centre
//! is under `(radius + 1) * square_length` (0.65 of it at the short `LandscapeFrillDistance`
//! preference), nearest first until the merged mesh would pass 120000 indices or 65535 vertices.
//! Every vertex takes the frill's transformed up axis as its normal and the landscape shadow colour
//! at the frill as its diffuse colour.

use ac2_math::{Mat4, Vec2, Vec3};

/// `RefreshFrillMesh` stops before the merged index count exceeds this.
const MAX_INDICES: u32 = 119_999;
/// ... or the vertex count exceeds this.
const MAX_VERTICES: u32 = 0xFFFE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrillSettings {
    /// `FrillRender::m_current_lscape_radius`, in landscape cells. Game script sets it; the
    /// constructor default is 1.
    pub lscape_radius: u32,
    /// `LandscapeFrillDistance` preference 1: the radius shrinks to 0.65.
    pub short_distance: bool,
    /// `LandDefs::square_length`: a landscape cell's edge in metres.
    pub square_length: f32,
}

impl Default for FrillSettings {
    fn default() -> Self {
        Self {
            lscape_radius: 1,
            short_distance: false,
            square_length: 20.0,
        }
    }
}

/// One frill a landscape cell holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrillCandidate {
    /// World transform of the frill.
    pub world: Mat4,
    /// Index and vertex counts of its mesh's first triangle list.
    pub indices: u32,
    pub vertices: u32,
}

/// The candidates `RefreshFrillMesh` draws for a camera at `camera` (world AC metres), in no
/// particular order.
pub fn select_frills(
    candidates: &[FrillCandidate],
    camera: Vec3,
    settings: &FrillSettings,
) -> Vec<usize> {
    let square = settings.square_length;
    // The camera cell's centre (`gid_to_lcoord`, then the half-square offset).
    let centre = Vec2::new(
        (camera.x / square).floor() * square + square * 0.5,
        (camera.y / square).floor() * square + square * 0.5,
    );
    let mut limit = (settings.lscape_radius + 1) as f32 * square;
    if settings.short_distance {
        limit *= 0.65;
    }
    let limit = limit * limit;
    let mut refs: Vec<(f32, usize)> = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, frill)| {
            let origin = frill.world.w_axis;
            let distance = Vec2::new(origin.x, origin.y).distance_squared(centre);
            (distance < limit).then_some((distance, index))
        })
        .collect();
    // `_STL::sort` orders the refs farthest first; they are consumed from the end.
    refs.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (mut indices, mut vertices) = (0_u32, 0_u32);
    let mut chosen = Vec::new();
    for &(_, index) in refs.iter().rev() {
        let frill = &candidates[index];
        if indices + frill.indices > MAX_INDICES || vertices + frill.vertices > MAX_VERTICES {
            break;
        }
        indices += frill.indices;
        vertices += frill.vertices;
        chosen.push(index);
    }
    chosen
}

/// The normal every vertex of a frill takes: its world transform's up axis, normalised.
pub fn frill_normal(world: &Mat4) -> Vec3 {
    world.z_axis.truncate().normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, y: f32) -> FrillCandidate {
        FrillCandidate {
            world: Mat4::from_translation(Vec3::new(x, y, 0.0)),
            indices: 6,
            vertices: 4,
        }
    }

    #[test]
    fn frills_inside_the_radius_of_the_camera_cell_centre() {
        // Camera cell centre (10, 10); default radius (1 + 1) * 20 = 40 m.
        let chosen = select_frills(
            &[at(10.0, 49.0), at(10.0, 51.0), at(45.0, 30.0)],
            Vec3::new(3.0, 3.0, 50.0),
            &FrillSettings::default(),
        );
        assert_eq!(chosen, vec![0]);
        let short = FrillSettings {
            short_distance: true,
            ..FrillSettings::default()
        };
        assert!(select_frills(&[at(10.0, 40.0)], Vec3::new(3.0, 3.0, 0.0), &short).is_empty());
    }

    #[test]
    fn nearest_frills_fill_the_budget_first() {
        let big = |x: f32| FrillCandidate {
            indices: 60_000,
            ..at(x, 10.0)
        };
        let chosen = select_frills(
            &[big(30.0), big(12.0), big(20.0)],
            Vec3::new(10.0, 10.0, 0.0),
            &FrillSettings::default(),
        );
        assert_eq!(chosen, vec![1]);
    }
}
