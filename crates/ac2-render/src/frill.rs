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
    /// How many times the client draws each frill of retail's ring, jittered (`grass_copy`;
    /// `AGENTS.md` exception). 1 draws retail's frills only.
    pub copies: u32,
}

impl Default for FrillSettings {
    fn default() -> Self {
        Self {
            lscape_radius: 1,
            short_distance: false,
            square_length: 20.0,
            copies: 1,
        }
    }
}

/// Copies of each near frill for a `LandscapeDrawDistance` choice (0 to 4): the quality levels
/// raise the draw distance one step each, so the grass thickens with them.
pub fn grass_copies_for_draw_distance(choice: u32) -> u32 {
    [1, 2, 4, 8, 12][choice.min(4) as usize]
}

/// How far a copy strays from its frill on each axis, in metres: half the 2 m spacing of the DAT
/// meadow grid, so the copies fill the gaps between the authored clumps.
const COPY_SPREAD: f32 = 1.0;

/// Where copy `copy` of the frill at `origin` stands relative to it: an xy offset, a turn about
/// the up axis in radians, and a uniform scale. Copy 0 is the frill as authored.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrassCopy {
    pub offset: Vec2,
    pub yaw: f32,
    pub scale: f32,
}

/// Copy `copy` of the frill at `origin`, fixed by its position so it never moves between frames.
pub fn grass_copy(origin: Vec3, copy: u32) -> GrassCopy {
    if copy == 0 {
        return GrassCopy {
            offset: Vec2::ZERO,
            yaw: 0.0,
            scale: 1.0,
        };
    }
    let r = |lane: u32| hash_unit(origin, copy.wrapping_mul(4).wrapping_add(lane));
    GrassCopy {
        offset: Vec2::new(r(0) * 2.0 - 1.0, r(1) * 2.0 - 1.0) * COPY_SPREAD,
        yaw: r(2) * std::f32::consts::TAU,
        scale: 0.85 + 0.3 * r(3),
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

/// How far the client's grass reaches past retail's ring, in metres (`AGENTS.md` exception:
/// retail faded its merged frills out by `m_current_lscape_radius * square_length`, 20 m).
pub const FAR_GRASS_METRES: f32 = 160.0;
/// The most frills the far ring draws, nearest first.
pub const FAR_GRASS_MAX: usize = 3000;
/// The share of frills the ring keeps at its far edge; it thins linearly from all of them at the
/// near ring's edge.
const FAR_DENSITY: f32 = 0.3;

/// What the vertex stage needs to grow a frill in and out (`object.wgsl`): the near ring's radius
/// (full density inside), the far ring's radius, and the frill's fixed random value in [0, 1).
/// `[0.0; 3]` draws the frill as authored.
pub type GrassRing = [f32; 3];

/// The near and far radii for `settings`: the near one is `select_frills`' limit, and both shrink
/// to 0.65 at the short `LandscapeFrillDistance`.
pub fn grass_radii(settings: &FrillSettings) -> (f32, f32) {
    let scale = if settings.short_distance { 0.65 } else { 1.0 };
    let near = (settings.lscape_radius + 1) as f32 * settings.square_length * scale;
    (near, FAR_GRASS_METRES * scale)
}

/// A frill's fixed random value in [0, 1) from its position to the centimetre, so it thins the
/// same way from every viewpoint and nothing flickers as the camera moves.
pub fn grass_hash(origin: Vec3) -> f32 {
    hash_unit(origin, 0)
}

/// A fixed value in [0, 1) from `origin`'s xy to the centimetre and a `lane` picking an
/// independent stream.
fn hash_unit(origin: Vec3, lane: u32) -> f32 {
    let cell = |v: f32| (v * 100.0).floor() as i32 as u32;
    let mut h = cell(origin.x).wrapping_mul(0x9E37_79B1)
        ^ cell(origin.y).wrapping_mul(0x85EB_CA77)
        ^ lane.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// The share of frills kept `distance` metres out, between the radii: above 1 inside the near ring
/// (every frill whole) down to `FAR_DENSITY` at the far edge. `object.wgsl`'s `grass_scale` reads
/// the same curve.
pub fn grass_density(distance: f32, near: f32, far: f32) -> f32 {
    let t = ((distance - near) / (far - near).max(1e-3)).clamp(0.0, 1.0);
    1.1 + (FAR_DENSITY - 1.1) * t
}

/// The frills of the far ring: beyond `chosen` (the near ring), within the far radius of `camera`,
/// kept by their hash under the density there, nearest first up to `FAR_GRASS_MAX`.
pub fn select_far_frills(
    candidates: &[FrillCandidate],
    camera: Vec3,
    settings: &FrillSettings,
    chosen: &[usize],
) -> Vec<usize> {
    let (near, far) = grass_radii(settings);
    let mut near_set = vec![false; candidates.len()];
    for &index in chosen {
        near_set[index] = true;
    }
    let mut refs: Vec<(f32, usize)> = candidates
        .iter()
        .enumerate()
        .filter(|(index, _)| !near_set[*index])
        .filter_map(|(index, frill)| {
            let origin = frill.world.w_axis.truncate();
            let distance = origin.distance(camera);
            let kept =
                distance < far && grass_hash(origin) < grass_density(distance, near, far) + 0.05;
            kept.then_some((distance, index))
        })
        .collect();
    refs.sort_by(|a, b| a.0.total_cmp(&b.0));
    refs.truncate(FAR_GRASS_MAX);
    refs.into_iter().map(|(_, index)| index).collect()
}

/// The most bodies the grass bends away from in one frame (`object.wgsl`'s `grass_push`).
pub const GRASS_PUSHERS: usize = 8;
/// How far from the camera a body still bends the grass, in metres.
const PUSH_REACH: f32 = 60.0;
/// A body's reach into the grass at scale 1, in metres.
const PUSH_RADIUS: f32 = 0.75;

/// The bodies the grass bends away from (`AGENTS.md`): each body's origin (at its feet) and its
/// reach, `[x, y, z, radius]`, the nearest `GRASS_PUSHERS` within `PUSH_REACH` of `eye`. Retail's
/// frills stood still, so a body walked through them and their blades showed through its legs.
pub fn grass_pushers(bodies: impl Iterator<Item = (Vec3, f32)>, eye: Vec3) -> Vec<[f32; 4]> {
    let mut near: Vec<(f32, [f32; 4])> = bodies
        .filter_map(|(origin, scale)| {
            let distance = origin.distance(eye);
            (distance < PUSH_REACH).then_some((
                distance,
                [origin.x, origin.y, origin.z, PUSH_RADIUS * scale.max(0.1)],
            ))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    near.into_iter()
        .take(GRASS_PUSHERS)
        .map(|(_, pusher)| pusher)
        .collect()
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
    fn the_far_ring_thins_with_distance_and_skips_the_near_ring() {
        let settings = FrillSettings::default();
        assert_eq!(grass_radii(&settings), (40.0, FAR_GRASS_METRES));
        assert!(
            grass_density(10.0, 40.0, 160.0) > 1.0,
            "whole inside the near ring"
        );
        assert!((grass_density(160.0, 40.0, 160.0) - FAR_DENSITY).abs() < 1e-6);
        let candidates: Vec<_> = (0..2000)
            .map(|i| at((i % 200) as f32, (i / 200) as f32 * 20.0))
            .collect();
        let camera = Vec3::new(0.0, 0.0, 0.0);
        let near = select_frills(&candidates, camera, &settings);
        let far = select_far_frills(&candidates, camera, &settings, &near);
        assert!(far.iter().all(|i| !near.contains(i)));
        let kept_between = |lo: f32, hi: f32| {
            let total = candidates
                .iter()
                .filter(|c| (lo..hi).contains(&c.world.w_axis.truncate().length()))
                .count();
            // Both rings: the near one is measured from the camera cell's centre.
            let kept = near
                .iter()
                .chain(&far)
                .filter(|&&i| (lo..hi).contains(&candidates[i].world.w_axis.truncate().length()))
                .count();
            kept as f32 / total.max(1) as f32
        };
        assert!(
            kept_between(45.0, 60.0) > 0.85,
            "dense just past the near ring"
        );
        assert!(kept_between(140.0, 160.0) < 0.55, "thin at the far edge");
        assert!(far
            .iter()
            .all(|&i| candidates[i].world.w_axis.truncate().length() < FAR_GRASS_METRES));
    }

    #[test]
    fn the_grass_hash_is_stable_and_spread() {
        let a = grass_hash(Vec3::new(12.34, 56.78, 9.0));
        assert_eq!(
            a,
            grass_hash(Vec3::new(12.34, 56.78, 100.0)),
            "height does not change it"
        );
        let mean = (0..1000)
            .map(|i| grass_hash(Vec3::new(i as f32 * 0.37, i as f32 * 1.91, 0.0)))
            .sum::<f32>()
            / 1000.0;
        assert!((mean - 0.5).abs() < 0.05, "mean {mean}");
    }

    #[test]
    fn grass_copies_are_fixed_and_fill_the_gaps() {
        let origin = Vec3::new(101.25, 33.5, 4.0);
        assert_eq!(
            grass_copy(origin, 0),
            GrassCopy {
                offset: Vec2::ZERO,
                yaw: 0.0,
                scale: 1.0
            }
        );
        assert_eq!(grass_copy(origin, 3), grass_copy(origin, 3));
        assert_ne!(grass_copy(origin, 1), grass_copy(origin, 2));
        for copy in 1..64 {
            let c = grass_copy(origin, copy);
            assert!(c.offset.abs().max_element() <= COPY_SPREAD);
            assert!((0.85..=1.15).contains(&c.scale));
            assert!((0.0..std::f32::consts::TAU).contains(&c.yaw));
        }
        assert_eq!(grass_copies_for_draw_distance(0), 1, "level 1 is retail's");
        assert_eq!(grass_copies_for_draw_distance(9), 12);
    }

    #[test]
    fn the_grass_bends_from_the_nearest_bodies() {
        let eye = Vec3::new(0.0, 0.0, 0.0);
        let bodies = (0..20).map(|i| (Vec3::new(i as f32 * 4.0, 0.0, 0.0), 1.0));
        let pushers = grass_pushers(bodies, eye);
        assert_eq!(pushers.len(), GRASS_PUSHERS);
        assert_eq!(pushers[0], [0.0, 0.0, 0.0, PUSH_RADIUS]);
        assert!(
            pushers.windows(2).all(|w| w[0][0] < w[1][0]),
            "nearest first"
        );
        let far = grass_pushers([(Vec3::new(100.0, 0.0, 0.0), 1.0)].into_iter(), eye);
        assert!(far.is_empty(), "past the reach");
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
