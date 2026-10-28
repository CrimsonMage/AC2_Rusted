//! `CPartGroup`: the tree of part arrays an object is built from.
//!
//! A `CPartGroup` is a `CPartArray` (its Setup, bones and parts) with a key, a parent and
//! children. A child hangs from one of its parent's Setup connection points: the point's bone
//! index and offset matrix place the child's whole array (`CPartGroup::SetParentOffset`). Here
//! the groups live in one arena; a group's index is stable until the tree is rebuilt, and a
//! removed group leaves a hole.

use std::sync::Arc;

use ac2_anim::Ac2Location;
use ac2_assets::Ac2TotalVisualDesc;
use ac2_core::DataId;
use ac2_math::Mat4;

use crate::los::Ray;
use crate::part_array::{PartArray, PartSetup};
use crate::vector::Sphere;

/// `m_parentBoneIndex` of a group with no parent bone (`0xffffffff`).
const NO_PARENT_BONE: usize = usize::MAX;

/// One `CPartGroup`.
#[derive(Debug, Clone)]
pub struct PartGroup {
    /// `m_pgkey`.
    pub key: u32,
    /// The group's own `CPartArray`.
    pub array: PartArray,
    parent: Option<usize>,
    /// `m_connectionPointID`.
    connection_point: u32,
    /// `m_parentBoneIndex`.
    parent_bone: usize,
    /// `m_parentOffsetMatrix`.
    parent_offset: Mat4,
    children: Vec<usize>,
    /// `m_matrix`-style cache: the matrix `RefreshMatrices` last gave the group.
    matrix: Mat4,
    /// `m_bNeedMatrixRefresh`.
    need_refresh: bool,
    alive: bool,
}

impl PartGroup {
    fn new() -> Self {
        Self {
            key: 0,
            array: PartArray::new(),
            parent: None,
            connection_point: 0,
            parent_bone: NO_PARENT_BONE,
            parent_offset: Mat4::IDENTITY,
            children: Vec::new(),
            matrix: Mat4::IDENTITY,
            need_refresh: true,
            alive: true,
        }
    }

    /// `m_connectionPointID`.
    pub fn connection_point(&self) -> u32 {
        self.connection_point
    }

    /// `m_pParent`, as an index in the tree.
    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    /// `m_children`, in the order they were added.
    pub fn children(&self) -> &[usize] {
        &self.children
    }

    /// The group's matrix in the object's space (`CPartGroup::RefreshMatrices`' `mFinal`).
    pub fn matrix(&self) -> &Mat4 {
        &self.matrix
    }
}

/// A holding location found in a tree (`CPartGroup::GetHoldingLocation`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoundHolding {
    /// The group whose Setup lists the location.
    pub group: usize,
    pub location: Ac2Location,
    /// That group's bone matrix for the location.
    pub bone_matrix: Mat4,
}

/// The tree: group 0 is the root (the object's `m_partGroup`).
#[derive(Debug, Clone)]
pub struct PartGroupTree {
    groups: Vec<PartGroup>,
}

impl Default for PartGroupTree {
    fn default() -> Self {
        Self::new()
    }
}

impl PartGroupTree {
    /// A tree with one empty root group (`CPartGroup::CPartGroup`).
    pub fn new() -> Self {
        Self {
            groups: vec![PartGroup::new()],
        }
    }

    /// The group at `index` while it exists.
    pub fn group(&self, index: usize) -> Option<&PartGroup> {
        self.groups.get(index).filter(|g| g.alive)
    }

    /// Mutable access to the group at `index`.
    pub fn group_mut(&mut self, index: usize) -> Option<&mut PartGroup> {
        self.groups.get_mut(index).filter(|g| g.alive)
    }

    /// Every living group's index, parents before children.
    pub fn indices(&self) -> Vec<usize> {
        let mut order = Vec::new();
        self.walk(0, &mut order);
        order
    }

    fn walk(&self, index: usize, order: &mut Vec<usize>) {
        if self.group(index).is_none() {
            return;
        }
        order.push(index);
        for &child in &self.groups[index].children {
            self.walk(child, order);
        }
    }

    /// `CPartGroup::InitSetup`: the group at `index` takes the Setup (`CPartArray::SetSetupID`
    /// then `InitInternal`).
    pub fn init_setup(&mut self, index: usize, setup: Arc<PartSetup>) -> bool {
        match self.group_mut(index) {
            Some(group) => group.array.set_setup(setup) && group.array.init_internal(),
            None => false,
        }
    }

    /// `CPartGroup::SetParentOffset`: `child` hangs from `parent`'s bone of the connection point
    /// `cp`. Needs both groups to hold Setups and the point's bone within the parent's bones.
    pub fn set_parent_offset(&mut self, child: usize, parent: usize, cp: &Ac2Location) -> bool {
        let (Some(c), Some(p)) = (self.group(child), self.group(parent)) else {
            return false;
        };
        if c.array.setup().is_none() || cp.bone >= p.array.num_bones() {
            return false;
        }
        let offset = cp.transform.to_mat4();
        let group = &mut self.groups[child];
        group.connection_point = cp.id;
        group.parent_bone = cp.bone;
        group.parent_offset = offset;
        true
    }

    /// `CPartGroup::SetParentConnection`: the point `connection_id` of the parent's Setup.
    pub fn set_parent_connection(
        &mut self,
        child: usize,
        parent: usize,
        connection_id: u32,
    ) -> bool {
        let cp = self
            .group(parent)
            .and_then(|p| p.array.setup())
            .and_then(|setup| setup.setup.connection_point(connection_id));
        match cp {
            Some(cp) => self.set_parent_offset(child, parent, &cp),
            None => false,
        }
    }

    /// `CPartGroup::AddChild`: a group without a parent hangs from `parent` at the connection
    /// point; the child is appended. False when it already has a parent or the point fails.
    pub fn add_child(&mut self, parent: usize, child: usize, connection_id: u32) -> bool {
        if self.group(parent).is_none() || self.group(child).is_none_or(|c| c.parent.is_some()) {
            return false;
        }
        if !self.set_parent_connection(child, parent, connection_id) {
            return false;
        }
        self.groups[child].parent = Some(parent);
        self.groups[parent].children.push(child);
        self.mark_refresh();
        true
    }

    /// A new empty group in the arena (not yet attached).
    pub fn new_group(&mut self) -> usize {
        self.groups.push(PartGroup::new());
        self.groups.len() - 1
    }

    /// `CPartGroup::RemoveChild`: detaches `child` from `parent` and resets its offset to
    /// identity with no parent bone. False when it is not a child of `parent`.
    pub fn remove_child(&mut self, parent: usize, child: usize) -> bool {
        let Some(position) = self
            .group(parent)
            .and_then(|p| p.children.iter().position(|c| *c == child))
        else {
            return false;
        };
        self.groups[parent].children.remove(position);
        let group = &mut self.groups[child];
        group.parent = None;
        group.parent_bone = NO_PARENT_BONE;
        group.parent_offset = Mat4::IDENTITY;
        self.mark_refresh();
        true
    }

    /// `CPartGroup::DestroyChildren`: every descendant of `index` is destroyed.
    pub fn destroy_children(&mut self, index: usize) {
        let Some(group) = self.group_mut(index) else {
            return;
        };
        let children = std::mem::take(&mut group.children);
        for child in children {
            self.destroy_subtree(child);
        }
        self.mark_refresh();
    }

    fn destroy_subtree(&mut self, index: usize) {
        self.destroy_children(index);
        self.groups[index].alive = false;
        self.groups[index].parent = None;
    }

    /// `CPartGroup::GetPartGroup`: the group with `key` among `index` and its descendants, depth
    /// first in child order.
    pub fn get_part_group(&self, index: usize, key: u32) -> Option<usize> {
        let group = self.group(index)?;
        if group.key == key {
            return Some(index);
        }
        group
            .children
            .iter()
            .find_map(|&child| self.get_part_group(child, key))
    }

    /// `CPartGroup::GetTransitionGTime`'s group count: the group and every descendant each add
    /// one to `numPartGroups` (the summed time itself is not touched in the Legions build).
    pub fn transition_group_count(&self, index: usize) -> usize {
        self.group(index).map_or(0, |group| {
            1 + group
                .children
                .iter()
                .map(|&child| self.transition_group_count(child))
                .sum::<usize>()
        })
    }

    /// `CPartGroup::SetPlacementFrameID`: every group's bones take the frame; true when all
    /// groups had it.
    pub fn set_placement_frame_id(&mut self, index: usize, id: u32) -> bool {
        let Some(group) = self.group_mut(index) else {
            return false;
        };
        let mut all = group.array.set_placement_frame_id(id);
        group.array.update_bones();
        let children = group.children.clone();
        for child in children {
            if !self.set_placement_frame_id(child, id) {
                all = false;
            }
        }
        self.mark_refresh();
        all
    }

    fn mark_refresh(&mut self) {
        for group in &mut self.groups {
            group.need_refresh = true;
        }
    }

    /// `CPartGroup::CheckRefreshMatrices`: refreshes from the identity when marked.
    pub fn check_refresh_matrices(&mut self) {
        if self.groups[0].need_refresh {
            self.refresh_matrices(0, &Mat4::IDENTITY);
            for group in &mut self.groups {
                group.need_refresh = false;
            }
        }
    }

    /// `CPartGroup::RefreshMatrices`: the group's matrix is its parent offset on its parent's
    /// bone (when the parent has that bone) on `parent`; its parts and children follow.
    pub fn refresh_matrices(&mut self, index: usize, parent: &Mat4) {
        let Some(group) = self.group(index) else {
            return;
        };
        let local = match group.parent.and_then(|p| {
            self.group(p)
                .and_then(|p| p.array.bone_matrix(group.parent_bone).copied())
        }) {
            Some(bone) => bone * group.parent_offset,
            None => group.parent_offset,
        };
        let final_matrix = *parent * local;
        let children = group.children.clone();
        let group = &mut self.groups[index];
        group.matrix = final_matrix;
        group.array.refresh_matrices(&final_matrix);
        for child in children {
            self.refresh_matrices(child, &final_matrix);
        }
    }

    /// `CPartGroup::CacheHasPhysicsBSP`: any group's parts have a physics BSP; every group's
    /// bit is refreshed.
    pub fn cache_has_physics_bsp(&mut self, index: usize) -> bool {
        let Some(group) = self.group_mut(index) else {
            return false;
        };
        let mut any = group.array.cache_has_physics_bsp();
        let children = group.children.clone();
        for child in children {
            if self.cache_has_physics_bsp(child) {
                any = true;
            }
        }
        any
    }

    /// `CPartGroup::HasHoldingLocation`: the group's own Setup first, then each child's search.
    pub fn has_holding_location(&self, index: usize, name: u32) -> bool {
        self.get_holding_location(index, name).is_some()
    }

    /// `CPartGroup::GetHoldingLocation`: the first group in search order whose Setup lists the
    /// location.
    pub fn get_holding_location(&self, index: usize, name: u32) -> Option<FoundHolding> {
        let group = self.group(index)?;
        if let Some((location, bone_matrix)) = group.array.get_holding_location(name) {
            return Some(FoundHolding {
                group: index,
                location,
                bone_matrix,
            });
        }
        group
            .children
            .iter()
            .find_map(|&child| self.get_holding_location(child, name))
    }

    /// Every group's holding locations in `CPartGroup::GetHoldingLocation`'s search order (a
    /// group's own list, then each child's, depth first): the list [`crate::find_holding_point`]
    /// searches.
    pub fn holdings(&self) -> Vec<crate::held::GroupHolding> {
        let mut found = Vec::new();
        for index in self.indices() {
            if let Some(setup) = self.groups[index].array.setup() {
                found.extend(setup.setup.holding_points.iter().map(|location| {
                    crate::held::GroupHolding {
                        group: index,
                        location: *location,
                    }
                }));
            }
        }
        found
    }

    /// `CPartGroup::FindMatrixForBoneType`: the bone matrix (in its own group) of the first group
    /// in search order that has the bone type.
    pub fn find_matrix_for_bone_type(&self, index: usize, bone_type: u32) -> Option<Mat4> {
        let group = self.group(index)?;
        if let Some(matrix) = group.array.find_matrix_for_bone_type(bone_type) {
            return Some(matrix);
        }
        group
            .children
            .iter()
            .find_map(|&child| self.find_matrix_for_bone_type(child, bone_type))
    }

    /// `PHYSICSOBJREP::HasBoneType`.
    pub fn has_bone_type(&self, index: usize, bone_type: u32) -> bool {
        self.find_matrix_for_bone_type(index, bone_type).is_some()
    }

    /// `CPartGroup::TraceRay`: the nearest hit on the group and its descendants as the fraction of
    /// the ray's length and the hit mesh's type; no hit is `None` (delta 1, mesh type 0).
    pub fn trace_ray(&self, index: usize, ray: &Ray) -> Option<(f32, u32)> {
        let group = self.group(index)?;
        let mut nearest = 100_000.0_f32;
        let mut best = None;
        let own = group.array.trace_ray_through_parts(ray);
        for hit in own.into_iter().chain(
            group
                .children
                .iter()
                .filter_map(|&child| self.trace_ray(child, ray)),
        ) {
            if hit.0 < nearest {
                nearest = hit.0;
                best = Some(hit);
            }
        }
        best
    }

    /// `PHYSICSOBJREP::GetBoundingSphere` / `CPartArray::GetBoundingSphere` of the root group.
    pub fn bounding_sphere(&self) -> Sphere {
        self.group(0).map_or_else(
            || Sphere::new(crate::vector::Vector::ZERO, 0.0),
            |g| g.array.bounding_sphere(),
        )
    }

    /// `CPartGroup::ResetParentConnection`: the group takes connection point `connection_id` of
    /// its parent again. Without a parent the id is only stored and the offset reset. False when
    /// the parent's Setup lacks the point.
    pub fn reset_parent_connection(&mut self, index: usize, connection_id: u32) -> bool {
        let Some(group) = self.group(index) else {
            return false;
        };
        match group.parent {
            None => {
                let group = &mut self.groups[index];
                group.connection_point = connection_id;
                group.parent_bone = NO_PARENT_BONE;
                group.parent_offset = Mat4::IDENTITY;
                self.mark_refresh();
                true
            }
            Some(parent) => {
                let ok = self.set_parent_connection(index, parent, connection_id);
                self.mark_refresh();
                ok
            }
        }
    }

    /// `CPartGroup::UpdatePartGroupFromPGDD`: the group takes the key and the Setup, then loses
    /// its children.
    pub fn update_part_group(&mut self, index: usize, key: u32, setup: Arc<PartSetup>) -> bool {
        let Some(group) = self.group_mut(index) else {
            return false;
        };
        group.key = key;
        if !(group.array.set_setup(setup) && group.array.init_internal()) {
            return false;
        }
        self.destroy_children(index);
        true
    }

    /// `CPartGroup::SubInit` over a description node: Setup, key, then `AddChild` to `parent`.
    fn sub_init(
        &mut self,
        index: usize,
        parent: Option<usize>,
        key: u32,
        setup_did: Option<DataId>,
        connection: u32,
        setups: &mut dyn FnMut(DataId) -> Option<Arc<PartSetup>>,
    ) -> bool {
        let Some(setup) = setup_did.and_then(&mut *setups) else {
            return false;
        };
        if !self.init_setup(index, setup) {
            return false;
        }
        self.destroy_children(index);
        self.groups[index].key = key;
        match parent {
            Some(parent) => self.add_child(parent, index, connection),
            None => true,
        }
    }

    /// `CPartGroup::Init` of the description's root: the tree below `index` rebuilt from
    /// `desc`. `setups` loads a Setup by DID. False when a node fails.
    pub fn init_visual_desc(
        &mut self,
        index: usize,
        desc: &Ac2TotalVisualDesc,
        setups: &mut dyn FnMut(DataId) -> Option<Arc<PartSetup>>,
    ) -> bool {
        match desc.root {
            Some(root) => self.init_node(index, None, root, desc, setups),
            None => false,
        }
    }

    fn init_node(
        &mut self,
        index: usize,
        parent: Option<usize>,
        key: u32,
        desc: &Ac2TotalVisualDesc,
        setups: &mut dyn FnMut(DataId) -> Option<Arc<PartSetup>>,
    ) -> bool {
        let Some(node) = desc.group(key) else {
            return false;
        };
        if !self.sub_init(
            index,
            parent,
            key,
            node.setup,
            node.connection_point,
            setups,
        ) {
            return false;
        }
        for &child_key in desc.children(key) {
            let child = self.new_group();
            if !self.init_node(child, Some(index), child_key, desc, setups) {
                self.groups[child].alive = false;
                return false;
            }
        }
        true
    }

    /// `CPartGroup::CompareAndModify`: the group follows the new description. A key the
    /// description lacks rebuilds from its root; a changed key or Setup is reloaded; the
    /// connection point is reset; the children are compared and the residual ones (keys the
    /// description no longer has) removed. The atom collection (meshes and materials) is the
    /// renderer's. False when the description cannot be followed.
    pub fn compare_and_modify(
        &mut self,
        index: usize,
        desc: &Ac2TotalVisualDesc,
        setups: &mut dyn FnMut(DataId) -> Option<Arc<PartSetup>>,
    ) -> bool {
        let Some(group) = self.group(index) else {
            return false;
        };
        let key = group.key;
        let mut node = desc.group(key);
        if node.is_none() {
            if let Some(root) = desc.root {
                self.init_node(index, None, root, desc, setups);
            }
            node = desc.group(self.groups[index].key);
            if node.is_none() {
                return false;
            }
        }
        let Some(node) = node else {
            return false;
        };
        let node_key = node.key;
        if node_key != 0 {
            let same = {
                let g = &self.groups[index];
                g.key == node_key && g.array.setup().is_some_and(|s| Some(s.did) == node.setup)
            };
            if !same {
                let loaded = node.setup.and_then(&mut *setups);
                match loaded {
                    Some(setup) => {
                        self.update_part_group(index, node_key, setup);
                    }
                    None => self.groups[index].key = node_key,
                }
            }
        }
        self.reset_parent_connection(index, node.connection_point);
        if !self.compare_and_modify_children(index, desc, node_key, setups) {
            return false;
        }
        self.handle_residual_children(index, desc)
    }

    /// `CPartGroup::CompareAndModifyChildren`: each child key of `key` in the description is
    /// found (or made and initialised) below `index` and compared in turn.
    fn compare_and_modify_children(
        &mut self,
        index: usize,
        desc: &Ac2TotalVisualDesc,
        key: u32,
        setups: &mut dyn FnMut(DataId) -> Option<Arc<PartSetup>>,
    ) -> bool {
        for &child_key in desc.children(key) {
            let child = match self.get_part_group(index, child_key) {
                Some(child) => child,
                None => {
                    let child = self.new_group();
                    let node = desc.group(child_key);
                    let ok = node.is_some_and(|n| {
                        self.sub_init(
                            child,
                            Some(index),
                            child_key,
                            n.setup,
                            n.connection_point,
                            setups,
                        )
                    });
                    if !ok {
                        self.groups[child].alive = false;
                        return false;
                    }
                    child
                }
            };
            if !self.compare_and_modify(child, desc, setups) {
                return false;
            }
        }
        true
    }

    /// `CPartGroup::HandleResidualChildren`: the direct children whose key the description no
    /// longer has are removed and destroyed.
    pub fn handle_residual_children(&mut self, index: usize, desc: &Ac2TotalVisualDesc) -> bool {
        let Some(group) = self.group(index) else {
            return false;
        };
        let doomed: Vec<usize> = group
            .children
            .iter()
            .copied()
            .filter(|&c| desc.group(self.groups[c].key).is_none())
            .collect();
        for child in doomed {
            self.remove_child(index, child);
            self.destroy_subtree(child);
        }
        true
    }
}
