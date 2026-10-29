//! The body's part-group tree (`PHYSICSOBJREP::m_partGroup`) and the `CPhysicsObj` and
//! `PHYSICSOBJREP` getters that read it: cylspheres, spheres, bounds, holding locations and
//! bones. The tree is built from the DATs on a loader thread (`load_part_group_tree`) and handed
//! to the body; a body without one answers as a representation-less `CPhysicsObj` does.

use std::sync::Arc;

use ac2_math::Mat4;

use crate::fpu::{ld, st};
use crate::held::{GroupHolding, HoldPoint};
use crate::los::Ray;
use crate::objects::HAS_PHYSICS_BSP_PS;
use crate::part_group::PartGroupTree;
use crate::shapes::cylsphere::CylSphere;
use crate::vector::{Sphere, Vector};

use super::PhysicsBody;

impl PhysicsBody {
    /// Gives the body its part-group tree (`PHYSICSOBJREP::InitVisualDescObject` built it).
    pub fn set_part_group(&mut self, tree: Arc<PartGroupTree>) {
        self.object.part_group = Some(tree);
    }

    /// `CPhysicsObj::GetPartGroup`: the tree, when the body has one.
    pub fn part_group(&self) -> Option<&PartGroupTree> {
        self.object.part_group.as_deref()
    }

    /// `PHYSICSOBJREP::GetPartGroup`: the index of the group with `key` in the tree.
    pub fn part_group_with_key(&self, key: u32) -> Option<usize> {
        self.part_group()?.get_part_group(0, key)
    }

    /// `CPhysicsObj::GetNumCylsphere` (`CPartArray::GetNumCylsphere`): the root Setup's count;
    /// 0 without a representation.
    pub fn num_cylsphere(&self) -> usize {
        self.part_group()
            .and_then(|tree| tree.group(0))
            .map_or(0, |group| group.array.num_cylsphere())
    }

    /// `CPhysicsObj::GetCylsphere` (`CPartArray::GetCylsphere`): the root Setup's cylspheres,
    /// unscaled; empty without a representation.
    pub fn cylspheres(&self) -> Vec<CylSphere> {
        self.part_group()
            .and_then(|tree| tree.group(0))
            .map_or_else(Vec::new, |group| group.array.cylspheres())
    }

    /// `CPhysicsObj::GetSortingSphere` (`PHYSICSOBJREP::GetBoundingSphere`): the Setup's bounding
    /// sphere, unscaled; the static zero sphere without a representation.
    pub fn sorting_sphere(&self) -> Sphere {
        self.part_group().map_or(
            Sphere::new(Vector::ZERO, 0.0),
            PartGroupTree::bounding_sphere,
        )
    }

    /// `PHYSICSOBJREP::GetSelectionSphere`: the Setup's selection sphere scaled by the
    /// representation's scale (the Setup's visual scale), radius by its z.
    fn rep_selection_sphere(&self) -> Option<Sphere> {
        let sphere = self.part_group()?.group(0)?.array.selection_sphere()?;
        let s = self.object.shape.scale;
        Some(scaled_sphere(sphere, Vector::new(s, s, s)))
    }

    /// `CPhysicsObj::GetSelectionSphere`: the representation's selection sphere, then scaled
    /// again by the object's own scale (`m_vScale`). `None` without a Setup.
    pub fn selection_sphere(&self) -> Option<Sphere> {
        let sphere = self.rep_selection_sphere()?;
        Some(scaled_sphere(sphere, self.object.scale))
    }

    /// `PHYSICSOBJREP::GetPhysicsBoundingBox` (`CPartArray::GetPhysicsBoundingBox`): the Setup's
    /// box (minimum, maximum), unscaled.
    pub fn physics_bounding_box(&self) -> Option<(Vector, Vector)> {
        self.part_group()?.group(0)?.array.physics_bounding_box()
    }

    /// `CPhysicsObj::SetBoundingObject` (`CPartArray::SetBoundingObject`): the root Setup's
    /// spheres are replaced, and with them the body's collision spheres (`GetSphere`). Retail
    /// writes into the shared `CSetup`; here only this body changes. False without a Setup.
    pub fn set_bounding_object(&mut self, spheres: &[Sphere]) -> bool {
        let Some(tree) = self.object.part_group.as_mut() else {
            return false;
        };
        let Some(group) = Arc::make_mut(tree).group_mut(0) else {
            return false;
        };
        if !group.array.set_bounding_object(spheres) {
            return false;
        }
        self.object.shape.spheres = spheres.to_vec();
        true
    }

    /// `PHYSICSOBJREP::CacheHasPhysicsBSP` as `CPhysicsObj::InitObjectEnd` uses it: any part of
    /// the tree has a physics BSP sets `m_state` bit 12, none clears it.
    pub fn cache_has_physics_bsp(&mut self) -> bool {
        let any = match self.object.part_group.as_mut() {
            Some(tree) => Arc::make_mut(tree).cache_has_physics_bsp(0),
            None => return false,
        };
        self.object.set_state_bit(HAS_PHYSICS_BSP_PS, any);
        any
    }

    /// `CPartGroup::GetBoneMatrix` of the root group as the transform `UpdateChild` combines a
    /// bone-index child with (`ChildType::BoneIndex`): the bone's unscaled matrix.
    pub fn bone_transform(&self, index: u32) -> Option<ac2_anim::Ac2Transform> {
        let matrix = self
            .part_group()?
            .group(0)?
            .array
            .bone_matrix(index as usize)?;
        let (_, rotation, offset) = matrix.to_scale_rotation_translation();
        Some(ac2_anim::Ac2Transform {
            offset,
            scale: ac2_math::Vec3::ONE,
            rotation,
        })
    }

    /// `CPhysicsObj::HasBoneType` (`PHYSICSOBJREP::HasBoneType`): some group's Setup names the
    /// bone type.
    pub fn has_bone_type(&self, bone_type: u32) -> bool {
        self.part_group()
            .is_some_and(|tree| tree.has_bone_type(0, bone_type))
    }

    /// `PHYSICSOBJREP::GetBoneTypeMatrix`: the first group's bone matrix for the type, in its
    /// own group's space.
    pub fn bone_type_matrix(&self, bone_type: u32) -> Option<Mat4> {
        self.part_group()?.find_matrix_for_bone_type(0, bone_type)
    }

    /// `PHYSICSOBJREP::HasHoldingLocation` over the tree.
    pub fn has_holding_location_in_parts(&self, location: u32) -> bool {
        self.part_group()
            .is_some_and(|tree| tree.has_holding_location(0, location))
    }

    /// `PHYSICSOBJREP::GetHoldingLocation` over the tree: where `CPhysicsObj::UpdateChild`
    /// places a child (the same point [`crate::find_holding_point`] finds in the groups'
    /// holding list).
    pub fn holding_point_in_parts(&self, location: u32) -> Option<HoldPoint> {
        let found = self.part_group()?.get_holding_location(0, location)?;
        Some(HoldPoint {
            group: found.group,
            bone: found.location.bone,
            transform: found.location.transform,
        })
    }

    /// [`Self::holding_point_in_parts`] with the bone matrix `CPartArray::GetHoldingLocation`
    /// hands back with it: the listing group's `m_boneMatrices` entry on the group's matrix
    /// (model space, unscaled; the identity group matrix for the root), as the transform
    /// `CPhysicsObj::UpdateChild` composes the location with. The tree's matrices must be
    /// current (`check_refresh_matrices`).
    pub fn holding_bone_in_parts(
        &self,
        location: u32,
    ) -> Option<(HoldPoint, ac2_anim::Ac2Transform)> {
        let tree = self.part_group()?;
        let found = tree.get_holding_location(0, location)?;
        let matrix = *tree.group(found.group)?.matrix() * found.bone_matrix;
        let (_, rotation, offset) = matrix.to_scale_rotation_translation();
        let point = HoldPoint {
            group: found.group,
            bone: found.location.bone,
            transform: found.location.transform,
        };
        let bone = ac2_anim::Ac2Transform {
            offset,
            scale: ac2_math::Vec3::ONE,
            rotation,
        };
        Some((point, bone))
    }

    /// The tree's holding locations in `CPartGroup::GetHoldingLocation`'s search order, as the
    /// list [`crate::find_holding_point`] searches.
    pub fn group_holdings(&self) -> Vec<GroupHolding> {
        self.part_group()
            .map_or_else(Vec::new, PartGroupTree::holdings)
    }

    /// `CPartGroup::TraceRay` through the body's tree: the nearest drawn-mesh hit of `ray` (in
    /// the object's space) as the fraction of its length and the mesh type; the tree's matrices
    /// must be current (`check_refresh_matrices`).
    pub fn trace_ray_parts(&self, ray: &Ray) -> Option<(f32, u32)> {
        self.part_group()?.trace_ray(0, ray)
    }
}

impl PhysicsBody {
    /// `CPhysicsObj::StickySetParent`'s placement: where `child` (a position in the world) hangs
    /// on this body. The child's position relative to this body's (`Position::subtract2`) picks
    /// the nearest bone of the root part array and the rotation relative to it
    /// (`CPartArray::CalculateBoneIndexAndOffset`); the offset's origin is zero. `None` without a
    /// part tree or bones.
    pub fn sticky_attachment(&self, child: &crate::Position) -> Option<(u32, crate::ChildFrame)> {
        let parent = &self.object.position;
        let origin = parent.local_to_local(child, Vector::ZERO);
        let q = |q: crate::Quaternion| ac2_math::Quat::from_xyzw(q.x, q.y, q.z, q.w);
        let rotation = (q(parent.frame.q).conjugate() * q(child.frame.q)).normalize();
        let local =
            ac2_math::Frame::new(ac2_math::Vec3::new(origin.x, origin.y, origin.z), rotation);
        let array = &self.part_group()?.group(0)?.array;
        let (bone, offset) = array.calculate_bone_index_and_offset(&local)?;
        Some((u32::try_from(bone).ok()?, offset))
    }
}

/// A sphere scaled the way `GetSelectionSphere` scales: each centre axis by the matching scale
/// axis, the radius by the z scale, each stored as `float`.
fn scaled_sphere(sphere: Sphere, scale: Vector) -> Sphere {
    Sphere::new(
        Vector::new(
            st(ld(scale.x) * ld(sphere.center.x)),
            st(ld(scale.y) * ld(sphere.center.y)),
            st(ld(scale.z) * ld(sphere.center.z)),
        ),
        st(ld(scale.z) * ld(sphere.radius)),
    )
}
