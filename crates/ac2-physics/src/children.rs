//! Parent and child objects (`CPhysicsObj::m_pParent` and `m_pChildren`, `CHILDLIST`,
//! `CHILDINFO`): a held or wielded item is a child of its holder, and the holder places it every
//! frame at the holding location, bone or bone index the child list names
//! (`CPhysicsObj::UpdateChild`). Retail links physics objects by pointer; here an object is
//! named by its id, and [`ChildGraph`] holds every object's parent and child list, so the one
//! graph owns the relation and the client only draws what it places.

use std::collections::HashMap;

use ac2_anim::Ac2Transform;
use ac2_math::{Frame, Vec3};

use crate::held::HoldPoint;

/// `CPhysicsObj::UpdateChild`'s holding location id that means "the holder's own origin": the
/// matrix is the identity, no Setup holding location is read.
pub const HOLDING_LOCATION_ORIGIN: u32 = 0x4000_0003;

/// A `CHILDINFO` offset frame: origin and rotation in AC axes.
pub type ChildFrame = Frame;

/// `ChildType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildType {
    Invalid,
    HoldingLocation,
    BoneName,
    BoneIndex,
}

/// `CHILDINFO`: one entry of a holder's child list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChildInfo {
    /// `m_physobjID`.
    pub id: u64,
    pub kind: ChildType,
    /// `m_locationID`: a holding location id, a bone name or a bone index by `kind`.
    pub location: u32,
    /// `m_offset`: the child's frame in the placed frame.
    pub offset: Frame,
    /// `m_bRemoveRotation`: only the matrix's translation places the child.
    pub remove_rotation: bool,
}

impl ChildInfo {
    /// What `CPhysicsObj::set_parent` adds: a holding location child with no offset.
    pub fn holding(id: u64, location: u32) -> Self {
        Self {
            id,
            kind: ChildType::HoldingLocation,
            location,
            offset: Frame::IDENTITY,
            remove_rotation: false,
        }
    }
}

/// `CHILDLIST`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChildList {
    children: Vec<ChildInfo>,
}

impl ChildList {
    /// `CHILDLIST::AddChild`: appended at the end.
    pub fn add_child(&mut self, info: ChildInfo) {
        self.children.push(info);
    }

    /// `CHILDLIST::RemoveChild`: the first entry for `id` leaves and the last entry fills its
    /// place (the order changes). `false` when `id` is not listed.
    pub fn remove_child(&mut self, id: u64) -> bool {
        match self.children.iter().position(|c| c.id == id) {
            Some(at) => {
                self.children.swap_remove(at);
                true
            }
            None => false,
        }
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&ChildInfo> {
        self.children.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChildInfo> {
        self.children.iter()
    }
}

/// Every object's parent and child list.
#[derive(Debug, Clone, Default)]
pub struct ChildGraph {
    parents: HashMap<u64, u64>,
    children: HashMap<u64, ChildList>,
}

impl ChildGraph {
    /// `CPhysicsObj::AddChildInternal`: `obj` joins `this`'s child list, unless it is `this`
    /// or `this`'s own parent. Does not set `obj`'s parent (callers do: `set_parent`).
    pub fn add_child_internal(
        &mut self,
        this: u64,
        obj: u64,
        kind: ChildType,
        location: u32,
        offset: Frame,
        remove_rotation: bool,
    ) -> bool {
        if obj == this || self.parents.get(&this) == Some(&obj) {
            return false;
        }
        self.children.entry(this).or_default().add_child(ChildInfo {
            id: obj,
            kind,
            location,
            offset,
            remove_rotation,
        });
        true
    }

    /// `CPhysicsObj::AddChild`: a non-zero `location_id` makes a holding location child,
    /// otherwise a bone name child on `bone_id`; `offset` is the child's origin offset.
    pub fn add_child(
        &mut self,
        this: u64,
        obj: u64,
        location_id: u32,
        bone_id: u32,
        offset: Vec3,
        remove_rotation: bool,
    ) -> bool {
        let offset = Frame::new(offset, ac2_math::Quat::IDENTITY);
        if location_id == 0 {
            self.add_child_internal(
                this,
                obj,
                ChildType::BoneName,
                bone_id,
                offset,
                remove_rotation,
            )
        } else {
            self.add_child_internal(
                this,
                obj,
                ChildType::HoldingLocation,
                location_id,
                offset,
                remove_rotation,
            )
        }
    }

    /// `CPhysicsObj::RemoveChild`: always reports success, listed or not.
    pub fn remove_child(&mut self, this: u64, obj: u64) -> bool {
        if let Some(list) = self.children.get_mut(&this) {
            list.remove_child(obj);
        }
        true
    }

    /// `CPhysicsObj::set_parent(newParent, location_id)`: `child` becomes a holding location
    /// child of `parent`. `parent_has_location` is `PHYSICSOBJREP::HasHoldingLocation` of the
    /// parent (not asked for [`HOLDING_LOCATION_ORIGIN`]). `parent`'s `AddChild` runs first; on
    /// success `child` leaves its old parent (`unset_parent`) and the caller takes it out of the
    /// world (the retail virtual `leave_world`).
    pub fn set_parent(
        &mut self,
        child: u64,
        parent: u64,
        location: u32,
        parent_has_location: bool,
    ) -> bool {
        if location != HOLDING_LOCATION_ORIGIN && !parent_has_location {
            return false;
        }
        if !self.add_child(parent, child, location, 0, Vec3::ZERO, false) {
            return false;
        }
        self.unset_parent(child);
        self.parents.insert(child, parent);
        true
    }

    /// `CPhysicsObj::StickySetParent`: `child` becomes a bone-index child of `parent` at `bone`
    /// with `offset` (`AddChildInternal(BONE_INDEX_CHILDTYPE)`); on success it leaves its old
    /// parent (`unset_parent`) and the caller takes it out of the world, as for
    /// [`ChildGraph::set_parent`]. False when `AddChildInternal` refuses.
    pub fn sticky_set_parent(&mut self, child: u64, parent: u64, bone: u32, offset: Frame) -> bool {
        if !self.add_child_internal(parent, child, ChildType::BoneIndex, bone, offset, false) {
            return false;
        }
        self.unset_parent(child);
        self.parents.insert(child, parent);
        true
    }

    /// `CPhysicsObj::unset_parent`: `child` leaves its parent's child list.
    pub fn unset_parent(&mut self, child: u64) {
        if let Some(parent) = self.parents.remove(&child) {
            if let Some(list) = self.children.get_mut(&parent) {
                list.remove_child(child);
            }
        }
    }

    /// `CPhysicsObj::unparent_children`: every child of `this` is unparented.
    pub fn unparent_children(&mut self, this: u64) {
        while let Some(child) = self.child(this, 0).map(|c| c.id) {
            self.unset_parent(child);
            // A child listed without a matching parent link (`AddChild` alone) would never
            // leave; drop the entry the way `unset_parent` would for a linked one.
            if self.child(this, 0).is_some_and(|c| c.id == child) {
                self.remove_child(this, child);
            }
        }
    }

    /// `CPhysicsObj::HasParent`.
    pub fn has_parent(&self, id: u64) -> bool {
        self.parents.contains_key(&id)
    }

    /// `CPhysicsObj::m_pParent`.
    pub fn parent_of(&self, id: u64) -> Option<u64> {
        self.parents.get(&id).copied()
    }

    /// `CPhysicsObj::TopParent`: the last parent up the chain, `None` for an object without one.
    pub fn top_parent(&self, id: u64) -> Option<u64> {
        let mut top = None;
        let mut at = self.parent_of(id);
        let mut steps = 0;
        while let Some(parent) = at {
            top = Some(parent);
            at = self.parent_of(parent);
            steps += 1;
            if steps > self.parents.len() {
                break;
            }
        }
        top
    }

    /// `CPhysicsObj::GetNumChildren`.
    pub fn num_children(&self, id: u64) -> usize {
        self.children.get(&id).map_or(0, ChildList::len)
    }

    /// `CPhysicsObj::GetChild`.
    pub fn child(&self, id: u64, index: usize) -> Option<&ChildInfo> {
        self.children.get(&id)?.get(index)
    }

    /// `CPhysicsObj::m_pChildren`.
    pub fn children_of(&self, id: u64) -> impl Iterator<Item = &ChildInfo> {
        self.children.get(&id).into_iter().flat_map(ChildList::iter)
    }

    /// The `ChildInfo` that places `child` under its parent.
    pub fn info_of(&self, child: u64) -> Option<&ChildInfo> {
        let parent = self.parent_of(child)?;
        self.children_of(parent).find(|c| c.id == child)
    }

    /// `CPhysicsObj::LookUpInChildren`: the direct child named `id`.
    pub fn look_up_in_children(&self, this: u64, id: u64) -> Option<u64> {
        self.children_of(this).find(|c| c.id == id).map(|c| c.id)
    }

    /// `CPhysicsObj::LookUpInAllChildren`: `id` among the children, each child checked before
    /// its own children. The recursive virtual is read as this function itself.
    pub fn look_up_in_all_children(&self, this: u64, id: u64) -> Option<u64> {
        self.look_up_in_all(this, id, 0)
    }

    fn look_up_in_all(&self, this: u64, id: u64, depth: usize) -> Option<u64> {
        if depth > self.children.len() {
            return None;
        }
        for child in self.children_of(this) {
            if child.id == id {
                return Some(id);
            }
            if let Some(found) = self.look_up_in_all(child.id, id, depth + 1) {
                return Some(found);
            }
        }
        None
    }

    /// `CPhysicsObj::GetTotalScale`: the object's own total scale (`own`, the representation's
    /// scale times `GetScale`) times, for a child that is not a missile (`has_missile_manager`),
    /// its parent's total scale and the parent representation's `m_fChildScale`
    /// (`child_scale`), recursively up the chain. Each product is stored as `float`.
    pub fn total_scale(
        &self,
        id: u64,
        own: &dyn Fn(u64) -> Vec3,
        child_scale: &dyn Fn(u64) -> f32,
        has_missile_manager: &dyn Fn(u64) -> bool,
    ) -> Vec3 {
        self.total_scale_at(id, own, child_scale, has_missile_manager, 0)
    }

    fn total_scale_at(
        &self,
        id: u64,
        own: &dyn Fn(u64) -> Vec3,
        child_scale: &dyn Fn(u64) -> f32,
        has_missile_manager: &dyn Fn(u64) -> bool,
        depth: usize,
    ) -> Vec3 {
        let total = own(id);
        let Some(parent) = self.parent_of(id) else {
            return total;
        };
        if has_missile_manager(id) || depth > self.parents.len() {
            return total;
        }
        let up = self.total_scale_at(parent, own, child_scale, has_missile_manager, depth + 1);
        scaled_by_parent(total, up, child_scale(parent))
    }

    /// `CPhysicsObj::UpdateAnimationRecursive`: `this` updates, then each direct child (a
    /// child's own children are not reached).
    pub fn animation_update_order(&self, this: u64) -> Vec<u64> {
        std::iter::once(this)
            .chain(self.children_of(this).map(|c| c.id))
            .collect()
    }

    /// `CPhysicsObj::UpdateFXRecursive`: `this`, then each child's own recursion, depth first.
    pub fn fx_update_order(&self, this: u64) -> Vec<u64> {
        let mut order = Vec::new();
        self.collect_fx(this, &mut order);
        order
    }

    fn collect_fx(&self, this: u64, order: &mut Vec<u64>) {
        if order.len() > self.parents.len() + 1 {
            return;
        }
        order.push(this);
        for child in self.children_of(this) {
            self.collect_fx(child.id, order);
        }
    }

    /// The order to place `ids` in: each child after its parent when both are listed, so a held
    /// object that holds another is placed first (`UpdateChild` runs holder first).
    pub fn placement_order(&self, ids: &[u64]) -> Vec<usize> {
        let index: HashMap<u64, usize> = ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        let holder_of: Vec<Option<usize>> = ids
            .iter()
            .map(|&id| self.parent_of(id).and_then(|p| index.get(&p).copied()))
            .collect();
        crate::held::placement_order(&holder_of)
    }
}

/// `CPhysicsObj::GetTotalScale`'s parent term: the child's own total scale times its parent's
/// total scale and the parent representation's `m_fChildScale` (1.0 unless set), each product
/// stored as `float`.
pub fn scaled_by_parent(own: Vec3, parent_total: Vec3, child_scale: f32) -> Vec3 {
    let k = f64::from(child_scale);
    let one = |t: f32, u: f32| ((f64::from(t) * f64::from(u)) * k) as f32;
    Vec3::new(
        one(own.x, parent_total.x),
        one(own.y, parent_total.y),
        one(own.z, parent_total.z),
    )
}

/// The matrix `CPhysicsObj::UpdateChild` builds for a holding location child: the holder's
/// posed root `bone` then the location's transform in that bone's space; the identity for
/// [`HOLDING_LOCATION_ORIGIN`].
pub fn holding_matrix(location: u32, bone: Ac2Transform, point: &HoldPoint) -> Ac2Transform {
    if location == HOLDING_LOCATION_ORIGIN {
        return Ac2Transform::IDENTITY;
    }
    bone.compose(&point.transform)
}

/// `CPhysicsObj::UpdateChild`: the child's frame. The holder's drawing frame is combined with
/// `matrix` (scaling its offset by the holder's total scale; only its translation when the
/// child removes rotation), then with the child's own offset.
pub fn update_child_frame(
    holder: Frame,
    holder_scale: Vec3,
    info: &ChildInfo,
    matrix: Ac2Transform,
) -> Frame {
    let placed = if info.remove_rotation {
        Frame::new(matrix.offset, ac2_math::Quat::IDENTITY)
    } else {
        Frame::new(matrix.offset, matrix.rotation.normalize())
    };
    let at = Frame::new(
        holder.origin + holder.orientation * (placed.origin * holder_scale),
        (holder.orientation * placed.orientation).normalize(),
    );
    at.compose_frame(info.offset)
}

#[cfg(test)]
mod tests {
    use ac2_anim::Ac2Transform;
    use ac2_math::{Frame, Quat, Vec3};

    use super::{
        holding_matrix, update_child_frame, ChildGraph, ChildInfo, ChildType,
        HOLDING_LOCATION_ORIGIN,
    };
    use crate::held::HoldPoint;

    fn point(offset: Vec3) -> HoldPoint {
        HoldPoint {
            group: 0,
            group_key: None,
            bone: 0,
            transform: Ac2Transform {
                offset,
                ..Ac2Transform::IDENTITY
            },
        }
    }

    #[test]
    fn set_parent_lists_the_child_and_moves_it_off_its_old_holder() {
        let mut g = ChildGraph::default();
        assert!(g.set_parent(9, 1, 5, true));
        assert_eq!(g.parent_of(9), Some(1));
        assert_eq!(g.num_children(1), 1);
        assert_eq!(g.child(1, 0).map(|c| (c.id, c.location)), Some((9, 5)));
        assert!(g.set_parent(9, 2, 6, true));
        assert_eq!(g.num_children(1), 0);
        assert_eq!(g.parent_of(9), Some(2));
        g.unset_parent(9);
        assert!(!g.has_parent(9) && g.num_children(2) == 0);
    }

    #[test]
    fn set_parent_needs_the_holding_location_unless_it_is_the_origin() {
        let mut g = ChildGraph::default();
        assert!(!g.set_parent(9, 1, 5, false));
        assert!(!g.has_parent(9));
        assert!(g.set_parent(9, 1, HOLDING_LOCATION_ORIGIN, false));
    }

    #[test]
    fn a_child_cannot_hold_itself_or_its_own_holder() {
        let mut g = ChildGraph::default();
        assert!(g.set_parent(2, 1, 5, true));
        assert!(!g.set_parent(1, 1, 5, true));
        // 1 is 2's parent: 2 cannot take 1 as a child.
        assert!(!g.set_parent(1, 2, 5, true));
    }

    #[test]
    fn remove_child_fills_the_gap_with_the_last_entry() {
        let mut g = ChildGraph::default();
        for child in [10, 11, 12] {
            assert!(g.set_parent(child, 1, 5, true));
        }
        assert!(g.remove_child(1, 10));
        let order: Vec<u64> = g.children_of(1).map(|c| c.id).collect();
        assert_eq!(order, vec![12, 11]);
        // Removing an unlisted child still reports success.
        assert!(g.remove_child(1, 99));
    }

    #[test]
    fn unparent_children_empties_the_list_and_top_parent_walks_up() {
        let mut g = ChildGraph::default();
        g.set_parent(2, 1, 5, true);
        g.set_parent(3, 2, 5, true);
        assert_eq!(g.top_parent(3), Some(1));
        assert_eq!(g.top_parent(1), None);
        assert_eq!(g.look_up_in_children(1, 3), None);
        assert_eq!(g.look_up_in_all_children(1, 3), Some(3));
        g.unparent_children(2);
        assert!(!g.has_parent(3) && g.num_children(2) == 0);
        assert_eq!(g.parent_of(2), Some(1));
    }

    #[test]
    fn a_held_child_is_placed_at_the_holding_point_and_holders_come_first() {
        let mut g = ChildGraph::default();
        g.set_parent(9, 7, 1, true);
        g.set_parent(8, 9, 1, true);
        // Listed child first, holder last: the holder is placed before the child it holds.
        assert_eq!(g.placement_order(&[8, 9, 7]), vec![2, 1, 0]);
        let info = *g.info_of(9).expect("the child's entry");
        let holder = Frame::new(Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY);
        let matrix = holding_matrix(info.location, Ac2Transform::IDENTITY, &point(Vec3::X));
        let at = update_child_frame(holder, Vec3::splat(2.0), &info, matrix);
        assert!((at.origin - Vec3::new(12.0, 0.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn a_childs_total_scale_carries_its_parents_and_the_child_scale() {
        let mut g = ChildGraph::default();
        g.set_parent(2, 1, 5, true);
        g.set_parent(3, 2, 5, true);
        let own = |id: u64| Vec3::splat(id as f32);
        let child_scale = |_: u64| 0.5_f32;
        let missile = |id: u64| id == 3;
        // 2's scale 2 x parent 1's scale 1 x 0.5.
        assert_eq!(
            g.total_scale(2, &own, &child_scale, &missile),
            Vec3::splat(1.0)
        );
        assert_eq!(
            g.total_scale(1, &own, &child_scale, &missile),
            Vec3::splat(1.0)
        );
        // A missile child ignores its parent chain.
        assert_eq!(
            g.total_scale(3, &own, &child_scale, &missile),
            Vec3::splat(3.0)
        );
        assert_eq!(
            g.total_scale(3, &own, &child_scale, &|_| false),
            Vec3::splat(1.5)
        );
    }

    #[test]
    fn recursive_updates_walk_the_children() {
        let mut g = ChildGraph::default();
        g.set_parent(2, 1, 5, true);
        g.set_parent(3, 1, 5, true);
        g.set_parent(4, 2, 5, true);
        assert_eq!(g.animation_update_order(1), vec![1, 2, 3]);
        assert_eq!(g.fx_update_order(1), vec![1, 2, 4, 3]);
    }

    #[test]
    fn remove_rotation_keeps_only_the_translation_and_the_offset_applies_last() {
        let turned = Ac2Transform {
            offset: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::from_rotation_z(1.0),
            ..Ac2Transform::IDENTITY
        };
        let info = ChildInfo {
            id: 1,
            kind: ChildType::BoneIndex,
            location: 0,
            offset: Frame::new(Vec3::new(0.0, 0.0, 3.0), Quat::IDENTITY),
            remove_rotation: true,
        };
        let at = update_child_frame(Frame::IDENTITY, Vec3::ONE, &info, turned);
        assert!((at.origin - Vec3::new(1.0, 0.0, 3.0)).length() < 1e-5);
        assert!(at.orientation.angle_between(Quat::IDENTITY) < 1e-5);
        assert_eq!(
            holding_matrix(HOLDING_LOCATION_ORIGIN, turned, &point(Vec3::X)),
            Ac2Transform::IDENTITY
        );
    }
}
