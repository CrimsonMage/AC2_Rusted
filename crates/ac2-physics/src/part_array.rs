//! `CPartArray`: one part group's Setup, its bones and its parts, on the physics side.
//!
//! Retail's `CPartArray` holds the `CSetup` (cylspheres, spheres, bounding and selection
//! spheres, heights, holding locations), one `CPhysicsPart` per Setup mesh and one matrix per
//! bone. The physics side keeps what collision, selection and the child graph read; drawing
//! (opacity, colours, particle systems, the compositor) stays in `ac2-render`.
//!
//! A part's mesh matters here only as `CPhysicsPart::HasPhysicsBSP` (the `m_state` bit
//! `CacheHasPhysicsBSP` sets) and as the triangles `TraceRayThroughParts` tests.

use std::sync::Arc;

use ac2_anim::{Ac2Location, Ac2Setup, Ac2Transform};
use ac2_assets::Ac2SetupPhysics;
use ac2_core::DataId;
use ac2_math::{Frame, Mat4, Quat, Vec3};

use crate::fpu::{ld, st, F_EPSILON};
use crate::los::Ray;
use crate::shapes::cylsphere::CylSphere;
use crate::vector::{Sphere, Vector};

/// `CPartArray::m_state` bit set by `CacheHasPhysicsBSP`.
pub const HAS_PHYSICS_BSP_STATE: u32 = 0x1000;
/// `CPartArray::m_state` bit set by `CacheHasMatrixWeights`: some part's mesh is skinned, so its
/// vertices already sit in the array's space and the bone matrices act on them only through the
/// vertex weights.
pub const HAS_MATRIX_WEIGHTS_STATE: u32 = 0x4000_0000;

/// One `CPhysicsPart`: the mesh a Setup lists for a bone.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PartMesh {
    /// `CGfxObj::mesh_type`, reported by a ray hit (`TraceRayThroughParts`'s `meshType`).
    pub mesh_type: u32,
    /// `CPhysicsPart::HasPhysicsBSP`.
    pub has_physics_bsp: bool,
    /// `CPhysicsPart::HasMatrixWeights`: the mesh's vertices carry bone weights.
    pub has_matrix_weights: bool,
    /// The first degrade's triangles (`m_Degrades[0]`), every material list, in the mesh's space.
    pub triangles: Arc<[[Vec3; 3]]>,
}

/// The `CSetup` data a part array reads, shared by every array built from the same DID.
#[derive(Debug, Clone, PartialEq)]
pub struct PartSetup {
    pub did: DataId,
    pub setup: Arc<Ac2Setup>,
    pub physics: Arc<Ac2SetupPhysics>,
    /// One per `CSetup::mesh_info` entry; `None` where the Setup lists mesh 0
    /// (`CPartArray::InitParts` makes no part for it).
    pub parts: Vec<Option<PartMesh>>,
}

/// `CPartArray` over one Setup.
#[derive(Debug, Clone, Default)]
pub struct PartArray {
    setup: Option<Arc<PartSetup>>,
    placement_id: u32,
    /// `m_boneTransforms`.
    bone_transforms: Vec<Ac2Transform>,
    /// `m_boneMatrices`: each bone in the group's space, scale left out.
    bone_matrices: Vec<Mat4>,
    /// `CPhysicsPart::m_mObject` per part: the part in the object's space after
    /// [`PartArray::refresh_matrices`].
    part_matrices: Vec<Mat4>,
    /// `CPartArray::SetBoundingObject`'s replacement of `CSetup::sphere`.
    bounding_object: Option<Vec<Sphere>>,
    state: u32,
}

/// A transform without its scale (`TransformBase::FillMatrix` flag 0).
fn unscaled(transform: &Ac2Transform) -> Mat4 {
    Ac2Transform {
        scale: Vec3::ONE,
        ..*transform
    }
    .to_mat4()
}

impl PartArray {
    /// An empty array (`CPartArray::CPartArray`): no Setup.
    pub fn new() -> Self {
        Self::default()
    }

    /// `CPartArray::SetSetupID` with the Setup already loaded: the same DID changes nothing;
    /// otherwise the bones and parts are rebuilt (`DestroyBones`, `DestroyParts`, the placement
    /// id reset, `InitParts`, `InitBones`).
    pub fn set_setup(&mut self, setup: Arc<PartSetup>) -> bool {
        if self.setup.as_ref().is_some_and(|own| own.did == setup.did) {
            return true;
        }
        self.bone_transforms.clear();
        self.bone_matrices.clear();
        self.part_matrices.clear();
        self.bounding_object = None;
        self.placement_id = 0;
        self.setup = Some(setup);
        self.cache_has_matrix_weights();
        self.init_bones()
    }

    /// `CPartArray::InitInternal`: only the render box is rebuilt (render plumbing); true.
    pub fn init_internal(&self) -> bool {
        true
    }

    /// `CPartArray::InitBones`: one transform and matrix per Setup bone at the placement frame.
    fn init_bones(&mut self) -> bool {
        let Some(setup) = self.setup.clone() else {
            return false;
        };
        let count = setup.setup.bones.len();
        self.bone_transforms = vec![Ac2Transform::IDENTITY; count];
        self.bone_matrices = vec![Mat4::IDENTITY; count];
        self.part_matrices = vec![Mat4::IDENTITY; setup.parts.len()];
        let id = self.placement_id;
        self.set_placement_frame_id(id);
        self.update_bones();
        true
    }

    /// `CPartArray::SetPlacementFrameID`: the bones take the Setup's frame `id`; a frame the Setup
    /// lacks leaves the placement id 0 and the bones from frame 0. False without that frame.
    pub fn set_placement_frame_id(&mut self, id: u32) -> bool {
        let Some(setup) = self.setup.as_ref() else {
            return false;
        };
        let frame = |wanted: u32| setup.setup.placements.iter().find(|f| f.kind == wanted);
        self.placement_id = if frame(id).is_some() { id } else { 0 };
        let Some(frame) = frame(self.placement_id) else {
            return false;
        };
        for (index, transform) in self.bone_transforms.iter_mut().enumerate() {
            if let Some(bone) = frame.bones.get(index) {
                *transform = *bone;
            }
        }
        true
    }

    /// `CPartArray::UpdateBones`: each bone's matrix is its own (unscaled) transform on its
    /// parent bone's matrix.
    pub fn update_bones(&mut self) {
        let Some(setup) = self.setup.as_ref() else {
            return;
        };
        for index in 0..self.bone_transforms.len() {
            let own = unscaled(&self.bone_transforms[index]);
            let parent = setup.setup.bones[index].parent.filter(|p| *p < index);
            self.bone_matrices[index] = match parent {
                Some(parent) => self.bone_matrices[parent] * own,
                None => own,
            };
        }
    }

    /// `CPartArray::CacheHasMatrixWeights`: any part's mesh has bone weights sets `m_state` bit 30;
    /// none clears it.
    pub fn cache_has_matrix_weights(&mut self) -> bool {
        let any = self
            .setup
            .as_ref()
            .is_some_and(|s| s.parts.iter().flatten().any(|p| p.has_matrix_weights));
        if any {
            self.state |= HAS_MATRIX_WEIGHTS_STATE;
        } else {
            self.state &= !HAS_MATRIX_WEIGHTS_STATE;
        }
        any
    }

    /// `CPartArray::RefreshMatrices`: each part stands at its bone's matrix on `parent`, or, in
    /// an array with matrix weights (skinned meshes), at `parent` alone.
    pub fn refresh_matrices(&mut self, parent: &Mat4) {
        let Some(setup) = self.setup.as_ref() else {
            return;
        };
        for (index, part) in setup.parts.iter().enumerate() {
            if part.is_some() {
                let bone = self
                    .bone_matrices
                    .get(index)
                    .copied()
                    .unwrap_or(Mat4::IDENTITY);
                self.part_matrices[index] = if self.state & HAS_MATRIX_WEIGHTS_STATE == 0 {
                    *parent * bone
                } else {
                    *parent
                };
            }
        }
    }

    /// The Setup (`CPartArray::m_pSetup`).
    pub fn setup(&self) -> Option<&Arc<PartSetup>> {
        self.setup.as_ref()
    }

    /// `m_placementID`.
    pub fn placement_id(&self) -> u32 {
        self.placement_id
    }

    /// `m_numBone`.
    pub fn num_bones(&self) -> usize {
        self.bone_matrices.len()
    }

    /// `m_boneMatrices[index]` (`CPartGroup::GetBoneMatrix`).
    pub fn bone_matrix(&self, index: usize) -> Option<&Mat4> {
        self.bone_matrices.get(index)
    }

    /// `m_boneTransforms`.
    pub fn bone_transforms(&self) -> &[Ac2Transform] {
        &self.bone_transforms
    }

    /// `CPhysicsPart::m_mObject` of part `index`.
    pub fn part_matrix(&self, index: usize) -> Option<&Mat4> {
        self.part_matrices.get(index)
    }

    /// `m_numParts`.
    pub fn num_parts(&self) -> usize {
        self.part_matrices.len()
    }

    /// `CPartArray::GetNumSphere`.
    pub fn num_sphere(&self) -> usize {
        self.spheres().len()
    }

    /// `CPartArray::GetSphere`: the Setup's spheres, or what `SetBoundingObject` replaced them
    /// with.
    pub fn spheres(&self) -> Vec<Sphere> {
        if let Some(own) = &self.bounding_object {
            return own.clone();
        }
        self.setup.as_ref().map_or_else(Vec::new, |setup| {
            setup
                .physics
                .spheres
                .iter()
                .map(|s| Sphere::new(Vector::from(s.origin), s.radius))
                .collect()
        })
    }

    /// `CPartArray::SetBoundingObject`: replaces the Setup's sphere list. False without a Setup.
    pub fn set_bounding_object(&mut self, spheres: &[Sphere]) -> bool {
        if self.setup.is_none() {
            return false;
        }
        self.bounding_object = Some(spheres.to_vec());
        true
    }

    /// `CPartArray::GetNumCylsphere`.
    pub fn num_cylsphere(&self) -> usize {
        self.setup
            .as_ref()
            .map_or(0, |s| s.physics.cylspheres.len())
    }

    /// `CPartArray::GetCylsphere`: the Setup's cylspheres.
    pub fn cylspheres(&self) -> Vec<CylSphere> {
        self.setup.as_ref().map_or_else(Vec::new, |setup| {
            setup
                .physics
                .cylspheres
                .iter()
                .map(|c| CylSphere {
                    low: Vector::from(c.low_point),
                    height: c.height,
                    radius: c.radius,
                })
                .collect()
        })
    }

    /// `CPartArray::GetRadius` (`CSetup::radius`, unscaled).
    pub fn radius(&self) -> f32 {
        self.setup.as_ref().map_or(0.0, |s| s.physics.radius)
    }

    /// `CPartArray::GetHeight` (`CSetup::height`, unscaled).
    pub fn height(&self) -> f32 {
        self.setup.as_ref().map_or(0.0, |s| s.physics.height)
    }

    /// `CPartArray::GetSelectionSphere`: the Setup's selection sphere; false without a Setup.
    pub fn selection_sphere(&self) -> Option<Sphere> {
        let sphere = self.setup.as_ref()?.physics.selection_sphere;
        Some(Sphere::new(Vector::from(sphere.origin), sphere.radius))
    }

    /// `CPartArray::GetBoundingSphere`: the Setup's bounding sphere, else the static zero sphere
    /// retail returns without a Setup.
    pub fn bounding_sphere(&self) -> Sphere {
        self.setup.as_ref().map_or_else(
            || Sphere::new(Vector::ZERO, 0.0),
            |setup| {
                let s = setup.physics.bounding_sphere;
                Sphere::new(Vector::from(s.origin), s.radius)
            },
        )
    }

    /// `CPartArray::GetPhysicsBoundingBox`: the Setup's box (minimum, maximum).
    pub fn physics_bounding_box(&self) -> Option<(Vector, Vector)> {
        let physics = &self.setup.as_ref()?.physics;
        Some((
            Vector::from(physics.bounds_min),
            Vector::from(physics.bounds_max),
        ))
    }

    /// `CPartArray::CacheHasPhysicsBSP`: any part's mesh has a physics BSP sets `m_state` bit
    /// 12; none clears it.
    pub fn cache_has_physics_bsp(&mut self) -> bool {
        let any = self
            .setup
            .as_ref()
            .is_some_and(|s| s.parts.iter().flatten().any(|p| p.has_physics_bsp));
        if any {
            self.state |= HAS_PHYSICS_BSP_STATE;
        } else {
            self.state &= !HAS_PHYSICS_BSP_STATE;
        }
        any
    }

    /// `CPartArray::m_state`.
    pub fn state(&self) -> u32 {
        self.state
    }

    /// `CPartArray::HasHoldingLocation`.
    pub fn has_holding_location(&self, name: u32) -> bool {
        self.holding_location(name).is_some()
    }

    fn holding_location(&self, name: u32) -> Option<Ac2Location> {
        self.setup
            .as_ref()?
            .setup
            .holding_points
            .iter()
            .copied()
            .find(|location| location.id == name)
    }

    /// `CPartArray::GetHoldingLocation`: the location and its bone's matrix.
    pub fn get_holding_location(&self, name: u32) -> Option<(Ac2Location, Mat4)> {
        let location = self.holding_location(name)?;
        let matrix = self.bone_matrices.get(location.bone).copied()?;
        Some((location, matrix))
    }

    /// `CPartArray::FindBoneType` (`CharacterSetup::FindBoneType`): the first bone with the name.
    pub fn find_bone_type(&self, bone_type: u32) -> Option<usize> {
        self.setup
            .as_ref()?
            .setup
            .bones
            .iter()
            .position(|bone| bone.name == bone_type)
    }

    /// `CPartArray::FindMatrixForBoneType`: that bone's matrix.
    pub fn find_matrix_for_bone_type(&self, bone_type: u32) -> Option<Mat4> {
        let index = self.find_bone_type(bone_type)?;
        self.bone_matrices.get(index).copied()
    }

    /// `CPartArray::CalculateBoneIndexAndOffset`: the bone whose origin is nearest `local`
    /// (squared distance, the first of equals) and `local` relative to that bone's frame. The
    /// offset's origin is then zeroed, as retail's trailing writes of `Vector::ZERO_VECTOR` do,
    /// so only the rotation relative to the bone remains. `None` without bones.
    pub fn calculate_bone_index_and_offset(&self, local: &Frame) -> Option<(usize, Frame)> {
        let mut best: Option<(usize, f32, Quat)> = None;
        for (index, matrix) in self.bone_matrices.iter().enumerate() {
            let origin = matrix.w_axis.truncate();
            let d = origin - local.origin;
            let squared = st(ld(d.y) * ld(d.y) + ld(d.z) * ld(d.z) + ld(d.x) * ld(d.x));
            if best.is_none_or(|(_, least, _)| squared < least) {
                let (_, rotation, _) = matrix.to_scale_rotation_translation();
                best = Some((index, squared, rotation));
            }
        }
        let (index, _, bone) = best?;
        let relative = (bone.conjugate() * local.orientation).normalize();
        Some((index, Frame::new(Vec3::ZERO, relative)))
    }

    /// `CPartArray::TraceRayThroughParts`: the nearest hit of `ray` (in the object's space) on any
    /// part's first-degrade triangles, as the fraction of the ray's length and the hit mesh's
    /// type. `None` is retail's `0` with delta 1 and mesh type 0.
    pub fn trace_ray_through_parts(&self, ray: &Ray) -> Option<(f32, u32)> {
        let setup = self.setup.as_ref()?;
        let mut nearest = 100_000.0_f32;
        let mut hit = None;
        for (index, part) in setup.parts.iter().enumerate() {
            let Some(part) = part else {
                continue;
            };
            let matrix = self
                .part_matrices
                .get(index)
                .copied()
                .unwrap_or(Mat4::IDENTITY);
            let inverse = matrix.inverse();
            let start = inverse.project_point3(vec3(ray.pt));
            let end_point = ray.end_point();
            let end = inverse.project_point3(vec3(end_point));
            let local = Ray::from_offset(Vector::from(start), Vector::from(end - start));
            if let Some(delta) = ray_intersects_tri_list(&local, &part.triangles, true) {
                if delta < nearest {
                    nearest = delta;
                    hit = Some((delta, part.mesh_type));
                }
            }
        }
        hit
    }
}

fn vec3(v: Vector) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

/// `RenderVertexBuffer::RayIntersectsTriList` over a triangle list: the nearest hit's distance
/// along the ray divided by the ray's length. A triangle is skipped when its plane normal is
/// shorter than `F_EPSILON`, when both ends of the ray lie strictly on one side of its plane
/// (`check_back_facing`: a ray starting behind the plane also skips when it ends there; without
/// the flag a start behind the plane skips), and when the ray is parallel to it; the hit
/// itself is not limited to the ray's length.
pub(crate) fn ray_intersects_tri_list(
    ray: &Ray,
    triangles: &[[Vec3; 3]],
    check_back_facing: bool,
) -> Option<f32> {
    let eps = ld(F_EPSILON);
    let side = |distance: f64| {
        if distance <= eps {
            if distance >= -eps {
                2
            } else {
                1
            }
        } else {
            0
        }
    };
    let (pt, dir) = (ray.pt, ray.dir);
    let end = ray.end_point();
    let mut closest: Option<f32> = None;
    for tri in triangles {
        let v0 = tri[0];
        let e1 = [
            st(ld(tri[1].x) - ld(v0.x)),
            st(ld(tri[1].y) - ld(v0.y)),
            st(ld(tri[1].z) - ld(v0.z)),
        ];
        let e2 = [
            st(ld(tri[2].x) - ld(v0.x)),
            st(ld(tri[2].y) - ld(v0.y)),
            st(ld(tri[2].z) - ld(v0.z)),
        ];
        let n = [
            st(ld(e2[2]) * ld(e1[1]) - ld(e2[1]) * ld(e1[2])),
            st(ld(e1[2]) * ld(e2[0]) - ld(e2[2]) * ld(e1[0])),
            st(ld(e2[1]) * ld(e1[0]) - ld(e1[1]) * ld(e2[0])),
        ];
        let length = (ld(n[0]) * ld(n[0]) + ld(n[1]) * ld(n[1]) + ld(n[2]) * ld(n[2])).sqrt();
        if length < eps {
            continue;
        }
        let inverse = 1.0 / length;
        let n = n.map(|c| st(ld(c) * inverse));
        let d = -(ld(n[0]) * ld(v0.x) + ld(n[1]) * ld(v0.y) + ld(n[2]) * ld(v0.z));
        let start_side = side(ld(n[0]) * ld(pt.x) + ld(n[1]) * ld(pt.y) + ld(n[2]) * ld(pt.z) + d);
        let end_side = side(ld(end.x) * ld(n[0]) + ld(n[1]) * ld(end.y) + ld(n[2]) * ld(end.z) + d);
        let test = if check_back_facing {
            if start_side != 1 {
                end_side != 0 || (start_side != 0 && end_side != 1)
            } else {
                end_side != 1
            }
        } else {
            start_side != 1 && (end_side != 0 || (start_side != 0 && end_side != 1))
        };
        if !test {
            continue;
        }
        // Möller-Trumbore (RayIntersectsTriList's terms, in the ray's own axes).
        let p = [
            st(ld(e2[2]) * ld(dir.y) - ld(e2[1]) * ld(dir.z)),
            st(ld(e2[0]) * ld(dir.z) - ld(e2[2]) * ld(dir.x)),
            st(ld(e2[1]) * ld(dir.x) - ld(e2[0]) * ld(dir.y)),
        ];
        let det = st(ld(p[0]) * ld(e1[0]) + ld(p[1]) * ld(e1[1]) + ld(p[2]) * ld(e1[2]));
        if !(ld(det) <= -eps || eps <= ld(det)) {
            continue;
        }
        let inv_det = 1.0 / ld(det);
        let s = [
            st(ld(pt.x) - ld(v0.x)),
            st(ld(pt.y) - ld(v0.y)),
            st(ld(pt.z) - ld(v0.z)),
        ];
        let u = ld(st((ld(s[0]) * ld(p[0])
            + ld(s[1]) * ld(p[1])
            + ld(s[2]) * ld(p[2]))
            * inv_det));
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        let q = [
            st(ld(s[1]) * ld(e1[2]) - ld(s[2]) * ld(e1[1])),
            st(ld(s[2]) * ld(e1[0]) - ld(e1[2]) * ld(s[0])),
            st(ld(e1[1]) * ld(s[0]) - ld(s[1]) * ld(e1[0])),
        ];
        let v = ld(st((ld(q[0]) * ld(dir.x)
            + ld(q[1]) * ld(dir.y)
            + ld(q[2]) * ld(dir.z))
            * inv_det));
        if v < 0.0 || v + u > 1.0 {
            continue;
        }
        let t = st((ld(q[0]) * ld(e2[0]) + ld(q[1]) * ld(e2[1]) + ld(q[2]) * ld(e2[2])) * inv_det);
        if closest.is_none_or(|nearest| t < nearest) {
            closest = Some(t);
        }
    }
    closest.map(|t| st(ld(t) / ld(ray.length)))
}
