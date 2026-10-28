//! `CEnvCell::TraceRayAgainstMesh` over `RenderVertexBuffer::RayIntersectsTriList`: the exact
//! drawing representation of an interior cell, its first degrade's triangles.

use ac2_assets::Ac2Mesh;

use super::{Ray, Vector};
use crate::fpu::{ld, st, F_EPSILON};

/// The vertex in AC axes. `RayIntersectsTriList` reads the render vertex buffer, which keeps y
/// and z swapped, and swaps them back; the decoded mesh is in AC axes already.
fn vertex(mesh: &Ac2Mesh, index: u32) -> Option<[f64; 3]> {
    let p = mesh.vertices.get(index as usize)?.position;
    Some([ld(p.x), ld(p.y), ld(p.z)])
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] * b[0] + a[1] * b[1]) + a[2] * b[2]
}

fn stored(v: [f64; 3]) -> [f64; 3] {
    v.map(|c| ld(st(c)))
}

/// Which side of a plane a point is on (`Sidedness` by `F_EPSILON`): 0 front, 1 back, 2 on it.
fn side(distance: f64) -> u8 {
    if distance > ld(F_EPSILON) {
        0
    } else if distance < -ld(F_EPSILON) {
        1
    } else {
        2
    }
}

/// `RenderVertexBuffer::RayIntersectsTriList` for every triangle list of the mesh's first
/// degrade (`CEnvCell::TraceRayAgainstMesh`): the nearest hit's distance as a fraction of the
/// ray (`closest / length`, a hit behind the start counts) and its triangle normal. With
/// `check_back_facing` off a triangle seen from behind is skipped.
pub(crate) fn trace_ray_against_mesh(
    mesh: &Ac2Mesh,
    ray: &Ray,
    check_back_facing: bool,
) -> Option<(f32, Vector)> {
    let degrade = mesh.fragments.iter().map(|f| f.degrade).min()?;
    let pt = [ld(ray.pt.x), ld(ray.pt.y), ld(ray.pt.z)];
    let dir = [ld(ray.dir.x), ld(ray.dir.y), ld(ray.dir.z)];
    let length = ld(ray.length);
    let end = stored([
        length * dir[0] + pt[0],
        length * dir[1] + pt[1],
        length * dir[2] + pt[2],
    ]);
    let mut closest: Option<(f64, [f64; 3])> = None;
    for fragment in mesh.fragments.iter().filter(|f| f.degrade == degrade) {
        for tri in fragment.indices.chunks_exact(3) {
            let (Some(v0), Some(v1), Some(v2)) = (
                vertex(mesh, tri[0]),
                vertex(mesh, tri[1]),
                vertex(mesh, tri[2]),
            ) else {
                continue;
            };
            let e1 = stored([v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]]);
            let e2 = stored([v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]]);
            let n = stored(cross(e1, e2));
            let n_length = dot(n, n).sqrt();
            if n_length < ld(F_EPSILON) {
                continue;
            }
            let inv = 1.0 / n_length;
            let n = stored([n[0] * inv, n[1] * inv, n[2] * inv]);
            let d = st(-dot(n, v0));
            let s0 = side(dot(n, pt) + ld(d));
            let s1 = side(dot(n, end) + ld(d));
            let test = if s0 != 1 {
                !(s0 == 0 && s1 == 0)
            } else {
                check_back_facing && s1 != 1
            };
            if !test {
                continue;
            }
            let p = stored(cross(dir, e2));
            let det = dot(p, e1);
            if det > -ld(F_EPSILON) && det < ld(F_EPSILON) {
                continue;
            }
            let inv_det = 1.0 / det;
            let s = stored([pt[0] - v0[0], pt[1] - v0[1], pt[2] - v0[2]]);
            let u = ld(st(dot(s, p) * inv_det));
            if !(0.0..=1.0).contains(&u) {
                continue;
            }
            let q = stored(cross(s, e1));
            let v = ld(st(dot(q, dir) * inv_det));
            if v < 0.0 || v + u > 1.0 {
                continue;
            }
            let t = ld(st(dot(q, e2) * inv_det));
            if closest.is_none_or(|(best, _)| t < best) {
                closest = Some((t, n));
            }
        }
    }
    let (t, n) = closest?;
    Some((st(t / length), Vector::new(st(n[0]), st(n[1]), st(n[2]))))
}

#[cfg(test)]
pub(super) mod tests {
    use std::sync::Arc;

    use ac2_assets::{Ac2MeshFragment, DecodedMeshVertex};
    use ac2_math::{Vec2, Vec3};

    use super::*;

    /// A wall at cell-local x = 4 facing -x, two triangles over y 0..4 and z 0..3.
    pub(crate) fn wall_mesh() -> Ac2Mesh {
        let at = |x: f32, y: f32, z: f32| DecodedMeshVertex {
            position: Vec3::new(x, y, z),
            normal: Vec3::new(-1.0, 0.0, 0.0),
            uv: Vec2::ZERO,
        };
        Ac2Mesh {
            mesh_type: 0,
            materials: Arc::from([]),
            vertex_format: 0,
            vertices: Arc::from(vec![
                at(4.0, 0.0, 0.0),
                at(4.0, 0.0, 3.0),
                at(4.0, 4.0, 0.0),
                at(4.0, 4.0, 3.0),
            ]),
            colors: None,
            extra_uvs: Arc::from([]),
            tangents: None,
            skin: None,
            fragments: Arc::from(vec![Ac2MeshFragment {
                material_index: 0,
                degrade: 0,
                detail_levels: 7,
                indices: Arc::from(vec![0, 1, 2, 3, 2, 1]),
            }]),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::new(4.0, 3.0, 4.0),
        }
    }

    fn ray(from: Vector, to: Vector) -> Ray {
        Ray::line_segment(from, to)
    }

    #[test]
    fn a_front_face_is_hit_at_its_fraction_of_the_ray() {
        let mesh = wall_mesh();
        let (t, normal) = trace_ray_against_mesh(
            &mesh,
            &ray(Vector::new(0.0, 2.0, 1.0), Vector::new(8.0, 2.0, 1.0)),
            false,
        )
        .expect("hit");
        assert!((t - 0.5).abs() < 1e-6, "{t}");
        assert_eq!(normal, Vector::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn a_ray_beside_or_short_of_the_wall_misses() {
        let mesh = wall_mesh();
        // Past the wall's edge in y.
        assert!(trace_ray_against_mesh(
            &mesh,
            &ray(Vector::new(0.0, 6.0, 1.0), Vector::new(8.0, 6.0, 1.0)),
            false
        )
        .is_none());
        // Stops before it: the end is still in front.
        assert!(trace_ray_against_mesh(
            &mesh,
            &ray(Vector::new(0.0, 2.0, 1.0), Vector::new(3.0, 2.0, 1.0)),
            false
        )
        .is_none());
    }

    #[test]
    fn a_back_face_hits_only_when_back_facing_is_checked() {
        let mesh = wall_mesh();
        let from_behind = ray(Vector::new(8.0, 2.0, 1.0), Vector::new(0.0, 2.0, 1.0));
        assert!(trace_ray_against_mesh(&mesh, &from_behind, false).is_none());
        let (t, _) = trace_ray_against_mesh(&mesh, &from_behind, true).expect("hit");
        assert!((t - 0.5).abs() < 1e-6);
    }
}
