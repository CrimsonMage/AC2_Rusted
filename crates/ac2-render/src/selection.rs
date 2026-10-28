//! Picking the object under the cursor, the way a selection render does it: `SceneTool::
//! StartSelection` with the cursor as a point, then every drawn, selectable object in
//! `CPhysicsObj::Draw` tests its bounding box projected to the screen against the point
//! (`SceneTool::CheckObjectSelection`, depth = the box's farthest projected z) and, on overlap,
//! traces the selection ray through its parts (`SceneTool::GetLocalSelectionRay`,
//! `CPhysicsObj::TraceRay`); a hit is selected at the ray's distance, a miss is only potential (and
//! the player is dropped). The third-person player's avatar is selected behind everything else
//! (depth 999999). `SceneTool::SelectObject` keeps the selections nearest first;
//! `gmSelectionUI::Finish3DSelection` takes the first.

use ac2_math::{Mat4, Vec3, Vec4};

use crate::object::{ObjectModel, ObjectPose};

/// The pick data of a model: its bounding box in model space, each part's rest transform
/// (mesh to model) and triangles in the part's mesh space, and each part's mesh type.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PickMesh {
    pub min: Vec3,
    pub max: Vec3,
    pub parts: Vec<(Mat4, Vec<[Vec3; 3]>)>,
    /// `CGfxObj::mesh_type` per part (`ObjectSelection::meshType` of a hit on it).
    pub part_types: Vec<u32>,
}

impl PickMesh {
    /// The model's triangles and bounds at rest (parts at their own mesh-to-model transforms).
    pub fn from_model(model: &ObjectModel) -> Self {
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        let parts = model
            .parts
            .iter()
            .map(|part| {
                for v in part.vertices.iter() {
                    let p = part
                        .transform
                        .transform_point3(Vec3::from_array(v.position));
                    min = min.min(p);
                    max = max.max(p);
                }
                // The full-detail degrade's triangles.
                let triangles = part
                    .drawn_batches(&crate::object::DrawDetail::FULL)
                    .iter()
                    .flat_map(|batch| batch.indices.chunks_exact(3))
                    .filter_map(|tri| {
                        let corner = |i: u32| {
                            part.vertices
                                .get(i as usize)
                                .map(|v| Vec3::from_array(v.position))
                        };
                        Some([corner(tri[0])?, corner(tri[1])?, corner(tri[2])?])
                    })
                    .collect();
                (part.transform, triangles)
            })
            .collect();
        if min.x > max.x {
            (min, max) = (Vec3::ZERO, Vec3::ZERO);
        }
        let part_types = model.parts.iter().map(|part| part.mesh_type).collect();
        Self {
            min,
            max,
            parts,
            part_types,
        }
    }
}

/// One object drawn this frame that may be selected.
pub struct PickCandidate<'a> {
    pub id: u64,
    /// Model to world.
    pub world: Mat4,
    pub pose: Option<&'a ObjectPose>,
    pub mesh: &'a PickMesh,
    /// The player seen from behind (`bIsThirdPersonPlayer`).
    pub third_person_player: bool,
}

/// `SceneTool::CheckObjectSelection`: whether the box projects over the cursor (`ndc` x, y), and
/// its farthest projected depth.
fn check_object_selection(
    view_projection: Mat4,
    world: Mat4,
    mesh: &PickMesh,
    ndc: (f32, f32),
) -> Option<f32> {
    let (lo, hi) = (mesh.min, mesh.max);
    let mut in_front = 0;
    let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for i in 0..8 {
        let corner = Vec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        );
        let clip: Vec4 = view_projection * world * corner.extend(1.0);
        if clip.w > 0.0 {
            in_front += 1;
        }
        let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
        let p = clip.truncate() / w;
        min = min.min(p);
        max = max.max(p);
    }
    if in_front == 0 {
        return None;
    }
    let inside = (min.x..=max.x).contains(&ndc.0) && (min.y..=max.y).contains(&ndc.1);
    inside.then_some(max.z)
}

/// Möller-Trumbore: the ray's distance to the triangle, when it hits the front or back.
fn ray_triangle(origin: Vec3, direction: Vec3, tri: &[Vec3; 3]) -> Option<f32> {
    let e1 = tri[1] - tri[0];
    let e2 = tri[2] - tri[0];
    let p = direction.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let s = origin - tri[0];
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = direction.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 0.0).then_some(t)
}

/// `CPhysicsObj::TraceRay`: the nearest hit of the world-space ray on the posed parts, and the
/// part it hit.
fn trace_ray(candidate: &PickCandidate<'_>, origin: Vec3, direction: Vec3) -> Option<(f32, usize)> {
    let mut nearest: Option<(f32, usize)> = None;
    for (index, (rest, triangles)) in candidate.mesh.parts.iter().enumerate() {
        let part = candidate
            .pose
            .and_then(|pose| pose.parts.get(index))
            .copied()
            .unwrap_or(*rest);
        // Into the part's mesh space, where its triangles are.
        let to_world = candidate.world * part;
        let to_local = to_world.inverse();
        let local_origin = to_local.transform_point3(origin);
        let local_direction = to_local.transform_vector3(direction);
        let scale = local_direction.length();
        if scale < 1e-9 {
            continue;
        }
        let local_direction = local_direction / scale;
        for tri in triangles {
            if let Some(t) = ray_triangle(local_origin, local_direction, tri) {
                let t = t / scale;
                if nearest.is_none_or(|(n, _)| t < n) {
                    nearest = Some((t, index));
                }
            }
        }
    }
    nearest
}

/// The object under `cursor` (pixels) on a `screen` (pixels) view through `view_projection`
/// from `eye`, nearest selected first; `None` when nothing is hit.
pub fn pick(
    view_projection: Mat4,
    eye: Vec3,
    screen: (u32, u32),
    cursor: (i32, i32),
    candidates: &[PickCandidate<'_>],
) -> Option<u64> {
    pick_part(view_projection, eye, screen, cursor, candidates).map(|(id, _)| id)
}

/// [`pick`], with the selection's `meshType`: the mesh type of the part the ray hit.
pub fn pick_part(
    view_projection: Mat4,
    eye: Vec3,
    screen: (u32, u32),
    cursor: (i32, i32),
    candidates: &[PickCandidate<'_>],
) -> Option<(u64, u32)> {
    pick_list(view_projection, eye, screen, cursor, candidates)
        .first()
        .copied()
}

/// The whole potential list under `cursor`, as `SceneTool::SelectObject` keeps it: each object
/// with its `meshType`, nearest first (`gmSelectionUI::Finish3DSelection` reads its first entry).
pub fn pick_list(
    view_projection: Mat4,
    eye: Vec3,
    screen: (u32, u32),
    cursor: (i32, i32),
    candidates: &[PickCandidate<'_>],
) -> Vec<(u64, u32)> {
    let ndc = (
        (cursor.0 as f32 + 0.5) / screen.0 as f32 * 2.0 - 1.0,
        1.0 - (cursor.1 as f32 + 0.5) / screen.1 as f32 * 2.0,
    );
    // The selection ray: the cursor unprojected from the near to the far plane.
    let inverse = view_projection.inverse();
    let near = inverse.project_point3(Vec3::new(ndc.0, ndc.1, 0.0));
    let far = inverse.project_point3(Vec3::new(ndc.0, ndc.1, 1.0));
    let direction = (far - near).normalize_or_zero();
    let origin = eye;
    let mut selected: Vec<(f32, u64, u32)> = Vec::new();
    for candidate in candidates {
        if check_object_selection(view_projection, candidate.world, candidate.mesh, ndc).is_none() {
            continue;
        }
        let Some((depth, part)) = trace_ray(candidate, origin, direction) else {
            // A miss is only a potential selection; the player's is dropped.
            continue;
        };
        let depth = if candidate.third_person_player {
            999_999.0
        } else {
            depth
        };
        let mesh_type = candidate.mesh.part_types.get(part).copied().unwrap_or(0);
        // `SelectObject`: before the first entry farther than it.
        let at = selected
            .iter()
            .position(|&(d, _, _)| depth < d)
            .unwrap_or(selected.len());
        selected.insert(at, (depth, candidate.id, mesh_type));
    }
    selected
        .into_iter()
        .map(|(_, id, mesh_type)| (id, mesh_type))
        .collect()
}
