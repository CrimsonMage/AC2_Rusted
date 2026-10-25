//! An object's local `VisualDesc` (`CPhysicsObj::m_local_vdesc`) as plain data: what the server
//! said the object looks like on top of its DAT `VisualDesc` chain.
//!
//! `CObjectMaint::CreatePhysicsObj` and `UpdateObject` apply the create's delta and
//! `CWPhysicsObject::RecvEvt_UpdateVisualDesc` applies an update's, both through
//! `CPhysicsObj::SetLocalVisualDesc`: the local description becomes the delta
//! (`VisualDesc::operator=`: `Clear`, the parent DID, then `ApplyAdditiveVDesc` of the delta onto
//! the empty description), plus the effects description (empty here). The drawn object is
//! `VisualDesc::GetTotalVisualDesc` of it: the parent chain from the DATs root first, this on
//! top, then `ModifyPGDTreeBasedOnApr`'s appearance-driven part-group swaps; that needs the DATs
//! and is the renderer's.

use ac2_net::protocol::{PartGroupDesc, VisualDesc};

/// `GLOBAL_PGD_KEY`: the key `VisualDesc::GetPGDDesc` maps to `m_global_app_mods`.
pub const GLOBAL_PART_GROUP: u32 = 0x4000_0002;

/// One part group's overrides (`PartGroupDataDesc`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PartGroupAppearance {
    pub key: u32,
    /// The group it hangs from (0 for the root) and the parent's connection point it uses.
    pub parent_key: u32,
    pub connection_point: u32,
    /// `m_setup_did` and `m_animmap_did` (0: inherited from the DAT chain).
    pub setup: u32,
    pub anim_map: u32,
    /// `m_app_hash`: appearance table DID -> its applied appearance keys and their scalars
    /// (the palette, texture and part swaps the appearance tables hold).
    pub appearance: Vec<(u32, Vec<(u32, f32)>)>,
    pub fx_table: u32,
    pub startup_fx: Vec<(u32, f32)>,
    /// FX ids whose `FXData` the group overrides.
    pub fx_overrides: Vec<u32>,
}

/// The local `VisualDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct Appearance {
    /// `m_parent_did`: the DAT `VisualDesc` this adds to.
    pub parent: u32,
    pub motion_interp: u32,
    pub behavior_table: u32,
    pub modes: u32,
    pub scale: [f32; 3],
    pub child_scale: f32,
    pub particle_scale: f32,
    /// `IconDesc` layers: layer id, image DID, shift colour.
    pub icon: Vec<(u32, u32, u32)>,
    /// `m_global_app_mods` (`GLOBAL_PGD_KEY`).
    pub global: PartGroupAppearance,
    /// `m_pgd_desc_table`, in the order the groups joined.
    pub part_groups: Vec<PartGroupAppearance>,
    /// `m_root`: the root group's key.
    pub root: Option<u32>,
    /// Children of each group, by key (`m_children_table`).
    pub children: Vec<(u32, Vec<u32>)>,
}

impl Default for Appearance {
    /// `VisualDesc::VisualDesc` / `Clear`: unit scales, nothing else.
    fn default() -> Self {
        Self {
            parent: 0,
            motion_interp: 0,
            behavior_table: 0,
            modes: 0,
            scale: [1.0; 3],
            child_scale: 1.0,
            particle_scale: 1.0,
            icon: Vec::new(),
            global: PartGroupAppearance {
                key: GLOBAL_PART_GROUP,
                ..PartGroupAppearance::default()
            },
            part_groups: Vec::new(),
            root: None,
            children: Vec::new(),
        }
    }
}

fn group(desc: &PartGroupDesc) -> PartGroupAppearance {
    PartGroupAppearance {
        key: desc.key,
        parent_key: desc.parent_key,
        connection_point: desc.connection_point,
        setup: desc.setup,
        anim_map: desc.anim_map,
        appearance: desc.appearance.clone(),
        fx_table: desc.fx_table,
        startup_fx: desc.startup_fx.clone(),
        fx_overrides: desc.fx_overrides.iter().map(|(id, _)| *id).collect(),
    }
}

/// `PartGroupDataDesc::ApplyAdditivePGDDesc`: the incoming appearance tables augment ours
/// (`AugmentAppearanceInfo`: a new table is added, a known one has its keys set,
/// `AppearanceInfo::AugmentExistingKeys`), a non-zero FX table replaces ours, startup FX we lack
/// are added, and the FX overrides join. Setup, anim map and connection point stay.
fn apply_additive_group(target: &mut PartGroupAppearance, incoming: &PartGroupAppearance) -> bool {
    if target.key != incoming.key {
        return false;
    }
    for (table, keys) in &incoming.appearance {
        match target.appearance.iter_mut().find(|(t, _)| t == table) {
            Some((_, existing)) => {
                for &(key, scalar) in keys {
                    match existing.iter_mut().find(|(k, _)| *k == key) {
                        Some(entry) => entry.1 = scalar,
                        None => existing.push((key, scalar)),
                    }
                }
            }
            None => target.appearance.push((*table, keys.clone())),
        }
    }
    if incoming.fx_table != 0 {
        target.fx_table = incoming.fx_table;
    }
    for &(fx, scalar) in &incoming.startup_fx {
        if !target.startup_fx.iter().any(|(id, _)| *id == fx) {
            target.startup_fx.push((fx, scalar));
        }
    }
    for &fx in &incoming.fx_overrides {
        if !target.fx_overrides.contains(&fx) {
            target.fx_overrides.push(fx);
        }
    }
    true
}

impl Appearance {
    /// `CPhysicsObj::SetLocalVisualDesc(delta)`: `VisualDesc::operator=`.
    pub fn from_delta(delta: &VisualDesc) -> Self {
        let mut local = Self {
            parent: delta.parent.unwrap_or(0),
            ..Self::default()
        };
        local.apply_additive(delta);
        local
    }

    /// The part group `key` (`VisualDesc::GetPGDDesc`).
    pub fn group(&self, key: u32) -> Option<&PartGroupAppearance> {
        if key == GLOBAL_PART_GROUP {
            return Some(&self.global);
        }
        self.part_groups.iter().find(|g| g.key == key)
    }

    fn group_mut(&mut self, key: u32) -> Option<&mut PartGroupAppearance> {
        if key == GLOBAL_PART_GROUP {
            return Some(&mut self.global);
        }
        self.part_groups.iter_mut().find(|g| g.key == key)
    }

    fn children_of(&self, key: u32) -> &[u32] {
        self.children
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(&[], |(_, c)| c.as_slice())
    }

    /// `PartGroupDataDesc::IsConnPtEmpty`: no child of `key` uses the connection point.
    fn connection_point_empty(&self, key: u32, point: u32) -> bool {
        !self
            .children_of(key)
            .iter()
            .filter_map(|&child| self.group(child))
            .any(|child| child.connection_point == point)
    }

    /// `VisualDesc::AddPGDDesc`: a known key merges into its group; a new one joins and becomes
    /// the root (no parent, or no root yet) or a child of its parent when the parent's
    /// connection point is free (an occupied one fails). A group whose parent is absent joins
    /// unattached.
    fn add_group(&mut self, incoming: &PartGroupAppearance) -> bool {
        if let Some(existing) = self.group_mut(incoming.key) {
            return apply_additive_group(existing, incoming);
        }
        self.part_groups.push(incoming.clone());
        if incoming.parent_key == 0 || self.root.is_none() {
            self.root = Some(incoming.key);
            return true;
        }
        if self.group(incoming.parent_key).is_none() {
            return true;
        }
        if !self.connection_point_empty(incoming.parent_key, incoming.connection_point) {
            return false;
        }
        match self
            .children
            .iter_mut()
            .find(|(k, _)| *k == incoming.parent_key)
        {
            Some((_, list)) => list.push(incoming.key),
            None => self
                .children
                .push((incoming.parent_key, vec![incoming.key])),
        }
        true
    }

    /// `VisualDesc::ApplyAdditiveVDesc`: the incoming DIDs replace ours when set, scales
    /// multiply, icon layers add, the global modifiers merge, then each incoming top-level
    /// part group merges into our root when it is the root's key (its children following,
    /// `RecursiveAdditivePGDDMerge`) or joins with its whole subtree.
    pub fn apply_additive(&mut self, incoming: &VisualDesc) -> bool {
        let set = |ours: &mut u32, theirs: Option<u32>| {
            if let Some(value) = theirs.filter(|&v| v != 0) {
                *ours = value;
            }
        };
        set(&mut self.motion_interp, incoming.motion_interp);
        set(&mut self.behavior_table, incoming.behavior_table);
        set(&mut self.modes, incoming.modes);
        let scale = incoming.scale.unwrap_or([1.0; 3]);
        for (ours, theirs) in self.scale.iter_mut().zip(scale) {
            *ours = (f64::from(*ours) * f64::from(theirs)) as f32;
        }
        self.child_scale =
            (f64::from(incoming.child_scale.unwrap_or(1.0)) * f64::from(self.child_scale)) as f32;
        self.particle_scale = (f64::from(incoming.particle_scale.unwrap_or(1.0))
            * f64::from(self.particle_scale)) as f32;
        // `IconDesc::ApplyAdditiveIconDesc`: a layer id already present takes the new image.
        for layer in &incoming.icon {
            let entry = (layer.layer, layer.image, layer.color);
            match self.icon.iter_mut().find(|(id, ..)| *id == layer.layer) {
                Some(existing) => *existing = entry,
                None => self.icon.push(entry),
            }
        }
        if let Some(global) = &incoming.global_mods {
            let mut global = group(global);
            global.key = GLOBAL_PART_GROUP;
            if !apply_additive_group(&mut self.global, &global) {
                return false;
            }
        }
        let groups: Vec<PartGroupAppearance> =
            incoming.part_groups.iter().map(|(_, g)| group(g)).collect();
        let present = |key: u32| groups.iter().any(|g| g.key == key);
        for top in groups
            .iter()
            .filter(|g| g.parent_key == 0 || !present(g.parent_key))
        {
            if self.root == Some(top.key) {
                let merged = match self.group_mut(top.key) {
                    Some(root) => apply_additive_group(root, top),
                    None => false,
                };
                if !merged {
                    return false;
                }
                for child in groups.iter().filter(|g| g.parent_key == top.key) {
                    if !self.merge_subtree(child, &groups) {
                        return false;
                    }
                }
            } else if !self.merge_subtree(top, &groups) {
                return false;
            }
        }
        true
    }

    /// `VisualDesc::RecursiveAdditivePGDDMerge`.
    fn merge_subtree(&mut self, group: &PartGroupAppearance, all: &[PartGroupAppearance]) -> bool {
        if !self.add_group(group) {
            return false;
        }
        for child in all
            .iter()
            .filter(|g| g.parent_key == group.key && g.key != group.key)
        {
            if !self.merge_subtree(child, all) {
                return false;
            }
        }
        true
    }
}

/// The protocol part group a `PartGroupAppearance` was merged from (FX override variants are
/// not kept, only their ids).
fn desc(group: &PartGroupAppearance) -> PartGroupDesc {
    PartGroupDesc {
        bitfield: 0,
        key: group.key,
        parent_key: group.parent_key,
        connection_point: group.connection_point,
        setup: group.setup,
        anim_map: group.anim_map,
        appearance: group.appearance.clone(),
        fx_table: group.fx_table,
        startup_fx: group.startup_fx.clone(),
        fx_overrides: group
            .fx_overrides
            .iter()
            .map(|id| (*id, Vec::new()))
            .collect(),
    }
}

impl Appearance {
    /// This description as an additive delta (what `ApplyAdditiveVDesc` reads from it): every
    /// DID, scale, icon layer, the global modifiers and the part groups in join order.
    pub fn to_visual_desc(&self) -> VisualDesc {
        let some = |v: u32| (v != 0).then_some(v);
        VisualDesc {
            parent: some(self.parent),
            motion_interp: some(self.motion_interp),
            behavior_table: some(self.behavior_table),
            modes: some(self.modes),
            scale: Some(self.scale),
            child_scale: Some(self.child_scale),
            particle_scale: Some(self.particle_scale),
            icon: self
                .icon
                .iter()
                .map(|&(layer, image, color)| ac2_net::protocol::IconLayer {
                    layer,
                    image,
                    color,
                })
                .collect(),
            global_mods: Some(desc(&self.global)),
            part_groups: self.part_groups.iter().map(|g| (g.key, desc(g))).collect(),
            ..VisualDesc::default()
        }
    }

    /// `VisualDesc::AddPGDDesc` of one part group.
    pub fn add_part_group(&mut self, group: &PartGroupAppearance) -> bool {
        self.add_group(group)
    }

    /// `VisualDesc::SwapPGDD`: group `key` takes `replacement`'s Setup, anim map and FX, and its
    /// appearance tables augment the group's (`AugmentAppearanceInfo`).
    pub fn swap_part_group(&mut self, key: u32, replacement: &PartGroupAppearance) -> bool {
        let Some(group) = self.group_mut(key) else {
            return false;
        };
        group.anim_map = replacement.anim_map;
        group.setup = replacement.setup;
        group.startup_fx = replacement.startup_fx.clone();
        group.fx_overrides = replacement.fx_overrides.clone();
        let appearance = PartGroupAppearance {
            key,
            appearance: replacement.appearance.clone(),
            ..PartGroupAppearance::default()
        };
        apply_additive_group(group, &appearance)
    }

    /// The child groups of `key` in join order (`m_children_table`).
    pub fn children(&self, key: u32) -> &[u32] {
        self.children_of(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_net::protocol::IconLayer;

    fn part(key: u32, parent_key: u32, connection_point: u32, setup: u32) -> (u32, PartGroupDesc) {
        (
            key,
            PartGroupDesc {
                key,
                parent_key,
                connection_point,
                setup,
                ..PartGroupDesc::default()
            },
        )
    }

    #[test]
    fn a_delta_becomes_the_local_description_with_its_tree() {
        let mut head = part(0x20, 0x10, 3, 0x0200_0002);
        head.1.appearance = vec![(0x2000_0001, vec![(0x4000_0010, 0.5)])];
        let delta = VisualDesc {
            parent: Some(0x1F00_0023),
            scale: Some([1.0, 1.0, 1.1]),
            icon: vec![IconLayer {
                layer: 1,
                image: 0x0600_0001,
                color: 0,
            }],
            part_groups: vec![head, part(0x10, 0, 0, 0x0200_0001), part(0x30, 0x10, 3, 7)],
            ..VisualDesc::default()
        };
        let local = Appearance::from_delta(&delta);
        assert_eq!(local.parent, 0x1F00_0023);
        assert_eq!(local.scale, [1.0, 1.0, 1.1]);
        assert_eq!(local.root, Some(0x10));
        // The head attaches at connection point 3; a second group on the same point is in the
        // table but attached nowhere (`AddPGDDesc` added it before failing), and the merge
        // stops there.
        assert_eq!(local.children, vec![(0x10, vec![0x20])]);
        assert!(local.group(0x30).is_some());
        assert_eq!(
            local.group(0x20).unwrap().appearance,
            vec![(0x2000_0001, vec![(0x4000_0010, 0.5)])]
        );
    }

    #[test]
    fn appearance_tables_augment_and_scales_multiply() {
        let mut local = Appearance::default();
        let mut first = part(0x10, 0, 0, 1);
        first.1.appearance = vec![(0x2000_0001, vec![(1, 0.25)])];
        local.apply_additive(&VisualDesc {
            part_groups: vec![first],
            scale: Some([2.0, 2.0, 2.0]),
            ..VisualDesc::default()
        });
        let mut second = part(0x10, 0, 0, 9);
        second.1.appearance = vec![(0x2000_0001, vec![(1, 0.75), (2, 1.0)])];
        local.apply_additive(&VisualDesc {
            part_groups: vec![second],
            scale: Some([0.5, 1.0, 1.0]),
            ..VisualDesc::default()
        });
        let root = local.group(0x10).unwrap();
        // The root keeps its Setup; the known table takes the new scalars.
        assert_eq!(root.setup, 1);
        assert_eq!(
            root.appearance,
            vec![(0x2000_0001, vec![(1, 0.75), (2, 1.0)])]
        );
        assert_eq!(local.scale, [1.0, 2.0, 2.0]);
    }
}
