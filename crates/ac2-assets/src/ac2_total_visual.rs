//! A `VisualDesc` merged the way `VisualDesc::GetTotalVisualDesc` builds the drawn description:
//! empty, then each additive delta through `VisualDesc::ApplyAdditiveVDesc`, with the part-group
//! tree rules of `AddPGDDesc` and `RecursiveAdditivePGDDMerge`, and the `SwapPGDD` that
//! `ModifyPGDTreeBasedOnApr` uses. FX fields are left out: they do not change what is drawn.

use ac2_core::DataId;
use ac2_math::Vec3;

use crate::ac2_icon::Ac2IconDesc;
use crate::ac2_visual_desc::Ac2VisualDesc;

/// `GLOBAL_PGD_KEY`: the key `VisualDesc::GetPGDDesc` maps to `m_global_app_mods`.
pub const GLOBAL_PART_GROUP: u32 = 0x4000_0002;

/// One merged part group (`PartGroupDataDesc` without its FX).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2TotalGroup {
    pub key: u32,
    pub parent_key: u32,
    pub connection_point: u32,
    pub setup: Option<DataId>,
    pub anim_map: Option<DataId>,
    /// `m_app_hash`: appearance table -> applied keys and their scalars, in join order.
    pub appearance: Vec<(DataId, Vec<(u32, f32)>)>,
}

impl Ac2TotalGroup {
    /// `PartGroupDataDesc::AugmentAppearanceInfo` for every incoming table: a new table joins,
    /// a known one has its keys set (`AppearanceInfo::AugmentExistingKeys`).
    fn augment_appearance(&mut self, incoming: &[(DataId, Vec<(u32, f32)>)]) {
        for (table, keys) in incoming {
            match self.appearance.iter_mut().find(|(t, _)| t == table) {
                Some((_, existing)) => {
                    for &(key, scalar) in keys {
                        match existing.iter_mut().find(|(k, _)| *k == key) {
                            Some(entry) => entry.1 = scalar,
                            None => existing.push((key, scalar)),
                        }
                    }
                }
                None => self.appearance.push((*table, keys.clone())),
            }
        }
    }
}

/// The merged description.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2TotalVisualDesc {
    pub motion_interp: Option<DataId>,
    pub behavior_table: Option<DataId>,
    pub modes: Option<DataId>,
    pub scale: Vec3,
    pub child_scale: f32,
    pub particle_scale: f32,
    /// `m_icon_desc`, merged by `IconDesc::ApplyAdditiveIconDesc`.
    pub icon: Ac2IconDesc,
    /// `m_global_app_mods`.
    pub global: Ac2TotalGroup,
    /// `m_pgd_desc_table`, in join order.
    pub groups: Vec<Ac2TotalGroup>,
    /// `m_root`: the root group's key.
    pub root: Option<u32>,
    /// Children of each group, by key, in join order.
    pub children: Vec<(u32, Vec<u32>)>,
}

impl Default for Ac2TotalVisualDesc {
    /// `VisualDesc::VisualDesc` / `Clear`: unit scales, nothing else.
    fn default() -> Self {
        Self {
            motion_interp: None,
            behavior_table: None,
            modes: None,
            scale: Vec3::ONE,
            child_scale: 1.0,
            particle_scale: 1.0,
            icon: Ac2IconDesc::default(),
            global: Ac2TotalGroup {
                key: GLOBAL_PART_GROUP,
                ..Ac2TotalGroup::default()
            },
            groups: Vec::new(),
            root: None,
            children: Vec::new(),
        }
    }
}

/// One part group of a delta as the merge reads it.
fn delta_groups(delta: &Ac2VisualDesc) -> Vec<Ac2TotalGroup> {
    delta
        .part_groups
        .iter()
        .enumerate()
        .map(|(index, group)| Ac2TotalGroup {
            key: group.key,
            parent_key: group.parent_key,
            connection_point: group.connection_point,
            setup: group.setup,
            anim_map: group.anim_map,
            appearance: delta
                .part_group_appearance
                .get(index)
                .map(|hash| hash.iter().map(|(t, keys)| (*t, keys.to_vec())).collect())
                .unwrap_or_default(),
        })
        .collect()
}

impl Ac2TotalVisualDesc {
    /// The part group `key` (`VisualDesc::GetPGDDesc`).
    pub fn group(&self, key: u32) -> Option<&Ac2TotalGroup> {
        if key == GLOBAL_PART_GROUP {
            return Some(&self.global);
        }
        self.groups.iter().find(|g| g.key == key)
    }

    fn group_mut(&mut self, key: u32) -> Option<&mut Ac2TotalGroup> {
        if key == GLOBAL_PART_GROUP {
            return Some(&mut self.global);
        }
        self.groups.iter_mut().find(|g| g.key == key)
    }

    /// The children of `key` in join order (`m_children_table`).
    pub fn children(&self, key: u32) -> &[u32] {
        self.children
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(&[], |(_, c)| c.as_slice())
    }

    /// `PartGroupDataDesc::IsConnPtEmpty`: no child of `key` uses the connection point.
    fn connection_point_empty(&self, key: u32, point: u32) -> bool {
        !self
            .children(key)
            .iter()
            .filter_map(|&child| self.group(child))
            .any(|child| child.connection_point == point)
    }

    /// `VisualDesc::AddPGDDesc`: a known key augments its group's appearance
    /// (`ApplyAdditivePGDDesc`; Setup, anim map and connection point stay); a new one joins and
    /// becomes the root (no parent, or no root yet) or a child of its parent when the parent's
    /// connection point is free (an occupied one fails). A group whose parent is absent joins
    /// unattached.
    pub fn add_group(&mut self, incoming: &Ac2TotalGroup) -> bool {
        if let Some(existing) = self.group_mut(incoming.key) {
            existing.augment_appearance(&incoming.appearance);
            return true;
        }
        self.groups.push(incoming.clone());
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

    /// `VisualDesc::SwapPGDD`: group `key` takes `replacement`'s Setup and anim map, and its
    /// appearance tables augment the group's.
    pub fn swap_group(&mut self, key: u32, replacement: &Ac2TotalGroup) -> bool {
        let Some(group) = self.group_mut(key) else {
            return false;
        };
        group.anim_map = replacement.anim_map;
        group.setup = replacement.setup;
        group.augment_appearance(&replacement.appearance);
        true
    }

    /// `VisualDesc::ApplyAdditiveVDesc`: the incoming DIDs replace ours when set, scales
    /// multiply, the global modifiers merge, then each incoming top-level part group merges into
    /// our root when it is the root's key (its children following, `RecursiveAdditivePGDDMerge`)
    /// or joins with its whole subtree.
    pub fn apply_additive(&mut self, delta: &Ac2VisualDesc) -> bool {
        let set = |ours: &mut Option<DataId>, theirs: Option<DataId>| {
            if theirs.is_some() {
                *ours = theirs;
            }
        };
        set(&mut self.motion_interp, delta.motion_interp);
        set(&mut self.behavior_table, delta.behavior_table);
        set(&mut self.modes, delta.modes);
        self.scale *= delta.scale.unwrap_or(Vec3::ONE);
        self.child_scale *= delta.child_scale.unwrap_or(1.0);
        self.particle_scale *= delta.particle_scale.unwrap_or(1.0);
        self.icon.apply_additive_desc(&delta.icon);
        let global: Vec<_> = delta
            .global_appearance
            .iter()
            .map(|(t, keys)| (*t, keys.to_vec()))
            .collect();
        self.global.augment_appearance(&global);
        let groups = delta_groups(delta);
        let present = |key: u32| groups.iter().any(|g| g.key == key);
        for top in groups
            .iter()
            .filter(|g| g.parent_key == 0 || !present(g.parent_key))
        {
            if self.root == Some(top.key) {
                match self.group_mut(top.key) {
                    Some(root) => root.augment_appearance(&top.appearance),
                    None => return false,
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
    fn merge_subtree(&mut self, group: &Ac2TotalGroup, all: &[Ac2TotalGroup]) -> bool {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::ac2_visual_desc::{Ac2PartGroup, Ac2VisualFx};

    fn delta(groups: &[(u32, u32, u32, u32)], appearance: &[(u32, u32, f32)]) -> Ac2VisualDesc {
        Ac2VisualDesc {
            parent: None,
            motion_interp: None,
            behavior_table: None,
            modes: None,
            scale: None,
            child_scale: None,
            particle_scale: None,
            icon: Ac2IconDesc::default(),
            part_groups: groups
                .iter()
                .map(|&(key, parent_key, connection_point, setup)| Ac2PartGroup {
                    key,
                    parent_key,
                    connection_point,
                    setup: Some(DataId::new(setup)),
                    anim_map: None,
                })
                .collect(),
            part_group_appearance: groups
                .iter()
                .map(|_| {
                    appearance
                        .iter()
                        .map(|&(t, k, s)| (DataId::new(t), Arc::from([(k, s)])))
                        .collect()
                })
                .collect(),
            global_appearance: Arc::from([]),
            fx: Ac2VisualFx::default(),
        }
    }

    #[test]
    fn a_tree_joins_and_later_deltas_augment_without_moving_setups() {
        let mut total = Ac2TotalVisualDesc::default();
        assert!(total.apply_additive(&delta(
            &[(0x10, 0, 0, 0x0400_0001), (0x20, 0x10, 3, 0x0400_0002)],
            &[]
        )));
        assert_eq!(total.root, Some(0x10));
        assert_eq!(total.children(0x10), &[0x20]);
        // A second delta on the root keeps its Setup and adds appearance keys.
        assert!(total.apply_additive(&delta(
            &[(0x10, 0, 0, 0x0400_0009)],
            &[(0x2000_0001, 1, 0.5)]
        )));
        let root = total.group(0x10).unwrap();
        assert_eq!(root.setup, Some(DataId::new(0x0400_0001)));
        assert_eq!(
            root.appearance,
            vec![(DataId::new(0x2000_0001), vec![(1, 0.5)])]
        );
        // A third group on an occupied connection point fails.
        assert!(!total.apply_additive(&delta(&[(0x30, 0x10, 3, 0x0400_0003)], &[])));
        assert!(total.swap_group(
            0x20,
            &Ac2TotalGroup {
                setup: Some(DataId::new(0x0400_0007)),
                ..Ac2TotalGroup::default()
            }
        ));
        assert_eq!(
            total.group(0x20).unwrap().setup,
            Some(DataId::new(0x0400_0007))
        );
    }
}
