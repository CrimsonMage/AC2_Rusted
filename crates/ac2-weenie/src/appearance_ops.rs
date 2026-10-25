//! The `VisualDesc` algebra the script's `gVisualDesc` efuns run (`portal/shared/src/objdesc/
//! visualdesc.cpp.c`, `appearance/partgroupdatadesc.cpp.c`, `appearance/appearanceinfo.cpp.c`):
//! the additive, subtractive and replacing merges, the removal of part groups and appearance
//! keys, and `MakeLocalPGDDFromParent`, which copies a part group from the parent's DAT
//! description.
//!
//! The parent is the DAT description `parent` names (`DBObj::Get(parent, 0x40000021)`). Callers
//! pass it in; `None` is where retail finds no parent (the description is empty, or the DAT
//! lacks it). Retail's tree pointers (`m_pParent`) are the description's `parent_key`s here: a
//! group has a parent in the description when that parent's key is one of its groups.

use super::{augment_table, Appearance, PartGroupAppearance, GLOBAL_PART_GROUP};

/// `PartGroupDataDesc::HasFX`: an FX table, or FX overrides.
pub(super) fn has_fx(group: &PartGroupAppearance) -> bool {
    group.fx_table != 0 || !group.fx_overrides.is_empty()
}

/// `AppearanceInfo::RemoveAppearanceKeys` over one table list: the incoming keys leave the
/// matching tables, and a table left without keys leaves too.
fn remove_keys(tables: &mut Vec<(u32, Vec<(u32, f32)>)>, incoming: &[(u32, Vec<(u32, f32)>)]) {
    for (table, keys) in incoming {
        let Some(index) = tables.iter().position(|(t, _)| t == table) else {
            continue;
        };
        tables[index]
            .1
            .retain(|(key, _)| !keys.iter().any(|(k, _)| k == key));
        if tables[index].1.is_empty() {
            tables.remove(index);
        }
    }
}

impl Appearance {
    /// `VisualDesc::Empty`: no parent, no root part group, no global appearance keys and no
    /// global FX.
    pub fn is_empty(&self) -> bool {
        self.parent == 0
            && self.root.is_none()
            && self.global.appearance.is_empty()
            && !has_fx(&self.global)
    }

    /// `VisualDesc::ApplyAdditiveVDesc`: the incoming description merged onto this one. True when
    /// every group merged.
    pub fn apply_additive_appearance(&mut self, incoming: &Appearance) -> bool {
        self.apply_additive_visited(incoming).is_some()
    }

    /// `ApplyAdditiveVDesc` with the visited table (`in_pgkey`): the incoming part-group keys that
    /// merged, or `None` where a merge failed. The DIDs replace ours when set, the scales and the
    /// child and particle scales multiply, the icon layers add (a layer id present takes the new
    /// image), the global modifiers merge, then each top-level group merges into our root when it
    /// is the root's key (its children following, `RecursiveAdditivePGDDMerge`) or joins with its
    /// whole subtree.
    pub(super) fn apply_additive_visited(&mut self, incoming: &Appearance) -> Option<Vec<u32>> {
        let set = |ours: &mut u32, theirs: u32| {
            if theirs != 0 {
                *ours = theirs;
            }
        };
        set(&mut self.motion_interp, incoming.motion_interp);
        set(&mut self.behavior_table, incoming.behavior_table);
        set(&mut self.modes, incoming.modes);
        for (ours, theirs) in self.scale.iter_mut().zip(incoming.scale) {
            *ours = (f64::from(*ours) * f64::from(theirs)) as f32;
        }
        self.child_scale = (f64::from(incoming.child_scale) * f64::from(self.child_scale)) as f32;
        self.particle_scale =
            (f64::from(incoming.particle_scale) * f64::from(self.particle_scale)) as f32;
        // `IconDesc::ApplyAdditiveIconDesc`: a layer id already present takes the new image.
        for &(layer, image, color) in &incoming.icon {
            match self.icon.iter_mut().find(|(id, ..)| *id == layer) {
                Some(existing) => *existing = (layer, image, color),
                None => self.icon.push((layer, image, color)),
            }
        }
        if !super::apply_additive_group(&mut self.global, &incoming.global) {
            return None;
        }
        let groups = &incoming.part_groups;
        let present = |key: u32| groups.iter().any(|g| g.key == key);
        let mut visited = Vec::new();
        for top in groups
            .iter()
            .filter(|g| g.parent_key == 0 || !present(g.parent_key))
        {
            if self.root == Some(top.key) {
                let merged = match self.group_mut(top.key) {
                    Some(root) => super::apply_additive_group(root, top),
                    None => false,
                };
                if !merged {
                    return None;
                }
                visited.push(top.key);
                for child in groups.iter().filter(|g| g.parent_key == top.key) {
                    if !self.merge_subtree(child, groups, &mut visited) {
                        return None;
                    }
                }
            } else if !self.merge_subtree(top, groups, &mut visited) {
                return None;
            }
        }
        Some(visited)
    }

    /// `VisualDesc::RecursiveAdditivePGDDMerge`.
    fn merge_subtree(
        &mut self,
        group: &PartGroupAppearance,
        all: &[PartGroupAppearance],
        visited: &mut Vec<u32>,
    ) -> bool {
        if !self.add_group(group) {
            return false;
        }
        visited.push(group.key);
        for child in all
            .iter()
            .filter(|g| g.parent_key == group.key && g.key != group.key)
        {
            if !self.merge_subtree(child, all, visited) {
                return false;
            }
        }
        true
    }

    /// Rebuilds the tree (`children`, `root`) from the groups' parent keys, for a description
    /// made from its groups.
    pub fn reset_tree(&mut self) {
        let mut children: Vec<(u32, Vec<u32>)> = Vec::new();
        for group in &self.part_groups {
            if group.parent_key == 0 || self.group(group.parent_key).is_none() {
                continue;
            }
            match children.iter_mut().find(|(k, _)| *k == group.parent_key) {
                Some((_, list)) => list.push(group.key),
                None => children.push((group.parent_key, vec![group.key])),
            }
        }
        self.children = children;
        self.reset_root();
    }

    /// `VisualDesc::Clear`: the description as a new one (unit scales, nothing set).
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// `VisualDesc::GetVisualDescFromDB`'s copy and `operator=`: this description becomes
    /// `incoming` (its parent included), cleared first, then its root is found again.
    pub fn load_from(&mut self, incoming: &Appearance) {
        self.clear();
        self.parent = incoming.parent;
        self.apply_additive_appearance(incoming);
        self.reset_root();
    }

    /// `VisualDesc::ReplaceVisualDesc`: with the same parent, the incoming description merges and
    /// the part groups it does not name leave (`ModVDesc_DeleteStragglers`); with two different
    /// parents, a copy (`operator=`); with one, a clear and a merge.
    pub fn replace_with(&mut self, incoming: &Appearance) -> bool {
        if self.parent == incoming.parent {
            let Some(visited) = self.apply_additive_visited(incoming) else {
                return false;
            };
            return self.remove_stragglers(&visited);
        }
        if self.parent != 0 && incoming.parent != 0 {
            self.load_from(incoming);
            return true;
        }
        self.clear();
        self.parent = incoming.parent;
        self.apply_additive_appearance(incoming);
        self.reset_root();
        true
    }

    /// `ModVDesc_DeleteStragglers`: every part group the merge did not visit leaves, in table
    /// order. False when one is missing.
    fn remove_stragglers(&mut self, visited: &[u32]) -> bool {
        let stragglers: Vec<u32> = self
            .part_groups
            .iter()
            .map(|g| g.key)
            .filter(|key| !visited.contains(key))
            .collect();
        stragglers.into_iter().all(|key| self.remove_group(key))
    }

    /// `VisualDesc::RemovePGDD`: a part group leaves. Its children lose their link to it
    /// (`PrepareForRemoval`), it leaves the table, and the root is found again when it was the
    /// root. False when the description holds no such group (the global group is not in the
    /// table).
    pub(super) fn remove_group(&mut self, key: u32) -> bool {
        let Some(index) = self.part_groups.iter().position(|g| g.key == key) else {
            return false;
        };
        self.part_groups.remove(index);
        self.children.retain(|(parent, _)| *parent != key);
        for (_, list) in &mut self.children {
            list.retain(|&child| child != key);
        }
        if self.root == Some(key) {
            self.reset_root();
        }
        true
    }

    /// `VisualDesc::RemovePGDDRecursively`: the group's children leave first (each recursively),
    /// then the group. False when the group is missing or a child cannot leave.
    pub(super) fn remove_group_recursive(&mut self, key: u32) -> bool {
        if key == GLOBAL_PART_GROUP {
            return false;
        }
        if !self.part_groups.iter().any(|g| g.key == key) {
            return false;
        }
        let children: Vec<u32> = self.children_of(key).to_vec();
        for child in children {
            if self.part_groups.iter().any(|g| g.key == child)
                && !self.remove_group_recursive(child)
            {
                return false;
            }
        }
        self.remove_group(key)
    }

    /// `VisualDesc::ResetRootPointer`: the first part group (table order) with no parent in the
    /// description is the root; with none, there is no root.
    pub(super) fn reset_root(&mut self) {
        self.root = self
            .part_groups
            .iter()
            .find(|g| self.is_parentless(g))
            .map(|g| g.key);
    }

    /// Whether a group has no parent group in this description (`m_pParent` is null).
    fn is_parentless(&self, group: &PartGroupAppearance) -> bool {
        group.parent_key == 0 || self.group(group.parent_key).is_none()
    }

    /// `PartGroupDataDesc::HasAppliedAppKey` (through `VisualDesc::HasAppliedAppKey`): the scalar
    /// of key `key` in table `table` of group `pg`, when applied.
    pub fn applied_scalar(&self, pg: u32, table: u32, key: u32) -> Option<f32> {
        self.group(pg)?
            .appearance
            .iter()
            .find(|(t, _)| *t == table)?
            .1
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, scalar)| *scalar)
    }

    /// `VisualDesc::AddAppearanceKey(pg_key, app_file, applied_key)`: the applied keys (key and
    /// scalar) of appearance table `table` join part group `pg` (the global group for
    /// `GLOBAL_PART_GROUP`). A group the description does not hold is copied from the parent's
    /// description first (`MakeLocalPGDDFromParent`). False for a zero key or table, or when the
    /// group cannot be had.
    pub fn add_appearance_key(
        &mut self,
        pg: u32,
        table: u32,
        keys: &[(u32, f32)],
        parent: Option<&Appearance>,
    ) -> bool {
        if pg == 0 || table == 0 {
            return false;
        }
        if self.group(pg).is_none() && !self.make_local_from_parent(pg, parent) {
            return false;
        }
        let Some(group) = self.group_mut(pg) else {
            return false;
        };
        augment_table(&mut group.appearance, table, keys);
        true
    }

    /// `VisualDesc::MakeLocalPGDDFromParent`: the part group `pg` of the parent's description
    /// joins this one with its key, parent key and connection point, and the children this
    /// description holds of it link to it. False without a parent, without the group there, or
    /// where the group cannot join (its connection point is taken).
    pub fn make_local_from_parent(&mut self, pg: u32, parent: Option<&Appearance>) -> bool {
        let Some(parent) = parent else {
            return false;
        };
        if self.parent == 0 {
            return false;
        }
        let Some(source) = parent.group(pg) else {
            return false;
        };
        let joined = PartGroupAppearance {
            key: source.key,
            parent_key: source.parent_key,
            connection_point: source.connection_point,
            ..PartGroupAppearance::default()
        };
        if !self.add_group(&joined) {
            return false;
        }
        for child in parent.children_of(pg).to_vec() {
            if let Some(local) = self.part_groups.iter_mut().find(|g| g.key == child) {
                local.parent_key = pg;
                match self.children.iter_mut().find(|(k, _)| *k == pg) {
                    Some((_, list)) if !list.contains(&child) => list.push(child),
                    Some(_) => {}
                    None => self.children.push((pg, vec![child])),
                }
            }
        }
        if self
            .root
            .and_then(|r| self.group(r))
            .is_some_and(|root| !self.is_parentless(root))
        {
            self.reset_root();
        }
        true
    }

    /// `VisualDesc::RemoveAppearanceKey(pg_key, app_file, applied_key)`: the key leaves table
    /// `table` of group `pg` (the table leaves with its last key). A group or table the
    /// description does not hold, with a parent that applies the key, overrides the parent's key
    /// with scalar -1 (`AddAppearanceKey`). False when neither holds.
    pub fn remove_appearance_key(
        &mut self,
        pg: u32,
        table: u32,
        key: u32,
        parent: Option<&Appearance>,
    ) -> bool {
        if pg == 0 || table == 0 {
            return false;
        }
        if let Some(group) = self.group_mut(pg) {
            if let Some(index) = group.appearance.iter().position(|(t, _)| *t == table) {
                let keys = &mut group.appearance[index].1;
                keys.retain(|(k, _)| *k != key);
                if keys.is_empty() {
                    group.appearance.remove(index);
                }
                return true;
            }
        }
        if self.parent == 0 {
            return false;
        }
        let Some(parent) = parent else {
            return false;
        };
        if parent.applied_scalar(pg, table, key).is_none() {
            return false;
        }
        self.add_appearance_key(pg, table, &[(key, -1.0)], Some(parent))
    }

    /// `VisualDesc::InqAppInfoHashForPartGroup`: the group's appearance tables (`None`: no group).
    pub fn app_info_of(&self, pg: u32) -> Option<Vec<(u32, Vec<(u32, f32)>)>> {
        self.group(pg).map(|g| g.appearance.clone())
    }

    /// `VisualDesc::AddAppInfoHashToPartGroup`: each table of `infos` joins group `pg`. True when
    /// every one did (none is true).
    pub fn add_app_info(
        &mut self,
        pg: u32,
        infos: &[(u32, Vec<(u32, f32)>)],
        parent: Option<&Appearance>,
    ) -> bool {
        let mut all = true;
        for (table, keys) in infos {
            all &= self.add_appearance_key(pg, *table, keys, parent);
        }
        all
    }

    /// `VisualDesc::ApplySubtractiveVDesc`: the incoming description's appearance keys leave this
    /// one (its global keys from the same tables). A part group it names with no keys and no FX
    /// leaves with its children (`RemovePGDDRecursively`); one it names with keys loses those keys
    /// (`AttemptToRemoveVDesc`). True; an empty incoming changes nothing.
    pub fn apply_subtractive(
        &mut self,
        incoming: &Appearance,
        parent: Option<&Appearance>,
    ) -> bool {
        if incoming.is_empty() {
            return true;
        }
        remove_keys(&mut self.global.appearance, &incoming.global.appearance);
        for group in &incoming.part_groups {
            if group.appearance.is_empty() {
                if !has_fx(group) {
                    self.remove_group_recursive(group.key);
                }
                continue;
            }
            for (table, keys) in &group.appearance {
                for (key, _) in keys {
                    self.remove_appearance_key(group.key, *table, *key, parent);
                }
            }
        }
        true
    }

    /// `VisualDesc::GetTotalIconDesc`'s icon layers: the parent's layers (the DAT chain), with this
    /// description's layers applied on top (`IconDesc::ApplyAdditiveIconLayer`). The appearance
    /// tables' icon modifiers (`AugmentAtomCollection`) are not modelled.
    pub fn total_icon_layers(&self, parent: Option<&Appearance>) -> Vec<(u32, u32, u32)> {
        let mut layers = parent.map_or_else(Vec::new, |p| p.icon.clone());
        for &(layer, image, color) in &self.icon {
            match layers.iter_mut().find(|(id, ..)| *id == layer) {
                Some(existing) => *existing = (layer, image, color),
                None => layers.push((layer, image, color)),
            }
        }
        layers
    }

    /// `VisualDesc::GetScale`.
    pub fn scale_vector(&self) -> [f32; 3] {
        self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: u32 = 0x8100_0001;
    const ARM: u32 = 0x8100_0002;
    const HAND: u32 = 0x8100_0003;
    const TABLE: u32 = 0x2000_0010;

    fn group_of(
        key: u32,
        parent_key: u32,
        point: u32,
        tables: &[(u32, &[(u32, f32)])],
    ) -> PartGroupAppearance {
        PartGroupAppearance {
            key,
            parent_key,
            connection_point: point,
            appearance: tables.iter().map(|(t, k)| (*t, k.to_vec())).collect(),
            ..PartGroupAppearance::default()
        }
    }

    /// A DAT description: root, an arm on point 2 and a hand on the arm's point 1.
    fn dat(parent: u32) -> Appearance {
        let mut a = Appearance {
            parent,
            ..Appearance::default()
        };
        a.part_groups = vec![
            group_of(ROOT, 0, 0, &[(TABLE, &[(1, 0.5)])]),
            group_of(ARM, ROOT, 2, &[(TABLE, &[(2, 0.25)])]),
            group_of(HAND, ARM, 1, &[]),
        ];
        a.root = Some(ROOT);
        a.children = vec![(ROOT, vec![ARM]), (ARM, vec![HAND])];
        a
    }

    fn keys_of(a: &Appearance, pg: u32) -> Vec<(u32, Vec<(u32, f32)>)> {
        a.group(pg).unwrap().appearance.clone()
    }

    /// `VisualDesc::ApplyAdditiveVDesc`: DIDs replace when set, scales multiply, icon layers
    /// replace by id, and the merged groups come back in the visited list.
    #[test]
    fn apply_additive_merges_dids_scales_icons_and_groups() {
        let mut ours = Appearance {
            scale: [2.0, 1.0, 1.0],
            icon: vec![(1, 10, 0xFF)],
            ..Appearance::default()
        };
        let incoming = Appearance {
            motion_interp: 0x4000_0004,
            scale: [1.5, 1.0, 1.0],
            icon: vec![(1, 11, 0xEE), (2, 12, 0xDD)],
            part_groups: vec![group_of(ROOT, 0, 0, &[(TABLE, &[(1, 0.5)])])],
            ..Appearance::default()
        };
        let visited = ours.apply_additive_visited(&incoming).expect("merged");
        assert_eq!(ours.motion_interp, 0x4000_0004);
        assert_eq!(ours.scale[0], 3.0, "scales multiply");
        assert_eq!(ours.icon, vec![(1, 11, 0xEE), (2, 12, 0xDD)]);
        assert_eq!(visited, vec![ROOT], "the group that merged is visited");
        assert_eq!(keys_of(&ours, ROOT), vec![(TABLE, vec![(1, 0.5)])]);
        // A zero DID leaves ours.
        let none = Appearance::default();
        ours.apply_additive_appearance(&none);
        assert_eq!(ours.motion_interp, 0x4000_0004);
    }

    /// `VisualDesc::Empty` and `Clear`.
    #[test]
    fn empty_is_no_parent_root_or_global_keys() {
        assert!(Appearance::default().is_empty());
        assert!(!dat(0x1F00_0023).is_empty());
        let mut a = dat(0x1F00_0023);
        a.clear();
        assert!(a.is_empty());
        assert_eq!(a.scale, [1.0; 3]);
        let mut global = Appearance::default();
        global.global.appearance = vec![(TABLE, vec![(1, 1.0)])];
        assert!(!global.is_empty());
    }

    /// `VisualDesc::GetVisualDescFromDB`: the description is cleared, takes the DAT's parent and
    /// groups, and its root is found.
    #[test]
    fn load_from_copies_the_dat_description_with_its_parent() {
        let mut ours = Appearance {
            modes: 7,
            part_groups: vec![group_of(0x9000_0000, 0, 0, &[])],
            root: Some(0x9000_0000),
            ..Appearance::default()
        };
        ours.load_from(&dat(0x1F00_0023));
        assert_eq!(ours.parent, 0x1F00_0023);
        assert_eq!(ours.modes, 0, "cleared first");
        assert!(ours.group(0x9000_0000).is_none());
        assert_eq!(ours.root, Some(ROOT));
        assert_eq!(keys_of(&ours, ARM), vec![(TABLE, vec![(2, 0.25)])]);
    }

    /// `VisualDesc::RemovePGDD`: a removed root leaves the root to the next parentless group; the
    /// children of a removed group lose their link.
    #[test]
    fn remove_group_unlinks_children_and_finds_the_root_again() {
        let mut a = dat(0);
        assert!(a.remove_group(ROOT));
        assert_eq!(a.root, Some(ARM), "the arm has no parent left");
        assert!(a.group(HAND).is_some(), "the hand stays, linked to the arm");
        assert!(!a.remove_group(ROOT), "a missing group is refused");
        assert!(a.remove_group(ARM));
        assert_eq!(a.root, Some(HAND));
    }

    /// `VisualDesc::RemovePGDDRecursively`: children leave first, then the group.
    #[test]
    fn remove_group_recursive_takes_the_subtree() {
        let mut a = dat(0);
        assert!(a.remove_group_recursive(ARM));
        assert_eq!(
            a.part_groups.iter().map(|g| g.key).collect::<Vec<_>>(),
            vec![ROOT]
        );
        assert!(a.children.iter().all(|(p, _)| *p != ARM));
        assert!(!a.remove_group_recursive(ARM));
    }

    /// `VisualDesc::ReplaceVisualDesc` with the same parent: the groups the incoming does not
    /// name leave (`ModVDesc_DeleteStragglers`), the global group stays.
    #[test]
    fn replace_removes_the_groups_the_incoming_does_not_name() {
        let mut ours = dat(0x1F00_0023);
        ours.global.appearance = vec![(0x2000_0002, vec![(9, 0.5)])];
        let incoming = Appearance {
            parent: 0x1F00_0023,
            part_groups: vec![group_of(ROOT, 0, 0, &[(TABLE, &[(1, 0.75)])])],
            ..Appearance::default()
        };
        assert!(ours.replace_with(&incoming));
        assert_eq!(
            ours.part_groups.iter().map(|g| g.key).collect::<Vec<_>>(),
            vec![ROOT]
        );
        assert_eq!(keys_of(&ours, ROOT), vec![(TABLE, vec![(1, 0.75)])]);
        assert_eq!(ours.global.appearance, vec![(0x2000_0002, vec![(9, 0.5)])]);
    }

    /// `ReplaceVisualDesc` with two different parents is `operator=`: a copy of the incoming.
    #[test]
    fn replace_with_another_parent_copies_it() {
        let mut ours = dat(0x1F00_0023);
        let incoming = dat(0x1F00_0099);
        assert!(ours.replace_with(&incoming));
        assert_eq!(ours.parent, 0x1F00_0099);
        assert_eq!(ours.part_groups.len(), 3);
    }

    /// `VisualDesc::AddAppearanceKey` on a group the description does not hold copies it from the
    /// parent (`MakeLocalPGDDFromParent`) and links the children it holds of it; without a parent
    /// nothing is added.
    #[test]
    fn add_appearance_key_copies_a_group_from_the_parent() {
        let parent = dat(0x1F00_0023);
        let mut ours = Appearance {
            parent: 0x1F00_0023,
            ..Appearance::default()
        };
        assert!(
            !ours.add_appearance_key(ARM, TABLE, &[(5, 1.0)], None),
            "no parent, no copy"
        );
        assert!(ours.add_appearance_key(ARM, TABLE, &[(5, 1.0)], Some(&parent)));
        let arm = ours.group(ARM).unwrap();
        assert_eq!(arm.parent_key, ROOT);
        assert_eq!(arm.connection_point, 2);
        assert_eq!(
            arm.appearance,
            vec![(TABLE, vec![(5, 1.0)])],
            "the parent's arm keys are not copied"
        );
        // A parentless description has no parent to copy from.
        let mut orphan = Appearance::default();
        assert!(!orphan.make_local_from_parent(ARM, Some(&parent)));
        // Zero keys and tables add nothing.
        assert!(!ours.add_appearance_key(ARM, 0, &[(5, 1.0)], Some(&parent)));
        assert!(!ours.add_appearance_key(0, TABLE, &[(5, 1.0)], Some(&parent)));
    }

    /// `MakeLocalPGDDFromParent` links the children already held of the copied group.
    #[test]
    fn make_local_links_the_held_children() {
        let parent = dat(0x1F00_0023);
        let mut ours = Appearance {
            parent: 0x1F00_0023,
            part_groups: vec![group_of(HAND, ARM, 1, &[])],
            ..Appearance::default()
        };
        assert!(ours.make_local_from_parent(ARM, Some(&parent)));
        assert!(ours
            .children
            .iter()
            .any(|(p, c)| *p == ARM && c.contains(&HAND)));
        assert_eq!(ours.group(HAND).unwrap().parent_key, ARM);
    }

    /// `VisualDesc::RemoveAppearanceKey`: a held key leaves its table, and the table with its last
    /// key; a key the parent applies and this does not hold is overridden with scalar -1.
    #[test]
    fn remove_appearance_key_removes_or_overrides_the_parents_key() {
        let parent = dat(0x1F00_0023);
        let mut ours = dat(0x1F00_0023);
        assert!(ours.remove_appearance_key(ARM, TABLE, 2, Some(&parent)));
        assert!(keys_of(&ours, ARM).is_empty() || ours.group(ARM).unwrap().appearance.is_empty());
        // The root's table holds one key: removing it removes the table.
        assert!(ours.remove_appearance_key(ROOT, TABLE, 1, Some(&parent)));
        assert!(ours.group(ROOT).unwrap().appearance.is_empty());
        // A group this description holds nothing of, applied by the parent: the -1 override.
        let mut fresh = Appearance {
            parent: 0x1F00_0023,
            ..Appearance::default()
        };
        assert!(fresh.remove_appearance_key(ROOT, TABLE, 1, Some(&parent)));
        assert_eq!(fresh.applied_scalar(ROOT, TABLE, 1), Some(-1.0));
        // A key the parent does not apply, from a table this description does not hold: nothing.
        let mut other = Appearance {
            parent: 0x1F00_0023,
            ..Appearance::default()
        };
        assert!(!other.remove_appearance_key(ROOT, TABLE, 99, Some(&parent)));
        // A table it holds answers true whether or not the key is there (`AppearanceInfo::
        // RemoveAppearanceKey` always answers 1).
        assert!(fresh.remove_appearance_key(ROOT, TABLE, 99, Some(&parent)));
        // No parent: nothing.
        let mut bare = Appearance::default();
        assert!(!bare.remove_appearance_key(ROOT, TABLE, 1, None));
    }

    /// `ApplySubtractiveVDesc`: the incoming keys leave; a group the incoming names with no keys
    /// and no FX leaves with its children; an empty incoming changes nothing.
    #[test]
    fn apply_subtractive_removes_keys_and_empty_groups() {
        let mut ours = dat(0);
        let mut incoming = Appearance {
            part_groups: vec![
                group_of(ROOT, 0, 0, &[(TABLE, &[(1, 0.5)])]),
                group_of(ARM, ROOT, 2, &[]),
            ],
            ..Appearance::default()
        };
        incoming.reset_root();
        assert!(ours.apply_subtractive(&incoming, None));
        assert!(
            ours.group(ROOT).unwrap().appearance.is_empty(),
            "the key left"
        );
        assert!(
            ours.group(ARM).is_none(),
            "a keyless group leaves with its children"
        );
        assert!(ours.group(HAND).is_none());
        assert!(ours.apply_subtractive(&Appearance::default(), None));
    }

    /// `VisualDesc::AddAppInfoHashToPartGroup`: each table joins; the answer is the AND, and an
    /// empty hash is true.
    #[test]
    fn add_app_info_adds_every_table() {
        let mut ours = dat(0);
        let infos = vec![(TABLE, vec![(7, 0.5)]), (0x2000_0011, vec![(8, 1.0)])];
        assert!(ours.add_app_info(ROOT, &infos, None));
        assert_eq!(ours.applied_scalar(ROOT, 0x2000_0011, 8), Some(1.0));
        assert_eq!(ours.app_info_of(ROOT).unwrap().len(), 2);
        assert!(ours.add_app_info(ROOT, &[], None));
        assert!(!ours.add_app_info(0, &infos, None));
    }

    /// `GetTotalIconDesc`'s layers: the parent's, with this description's applied on top.
    #[test]
    fn total_icon_is_the_parent_chain_with_ours_on_top() {
        let mut parent = dat(0);
        parent.icon = vec![(1, 10, 1), (2, 20, 2)];
        let ours = Appearance {
            icon: vec![(2, 21, 3), (3, 30, 4)],
            ..Appearance::default()
        };
        assert_eq!(
            ours.total_icon_layers(Some(&parent)),
            vec![(1, 10, 1), (2, 21, 3), (3, 30, 4)]
        );
        assert_eq!(ours.total_icon_layers(None), vec![(2, 21, 3), (3, 30, 4)]);
    }
}
