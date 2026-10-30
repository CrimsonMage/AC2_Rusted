//! `CPhysicsObj::ApplyVisualDesc`'s part rebuild: when an object's drawn description (its local
//! one with the FX's appearance keys merged on) changes what its part groups are, the body's
//! part-group tree follows it (`CPartGroup::CompareAndModify`, on a loader thread), so the
//! holding locations, bones and physics BSP flag are those of the object as drawn, and the body
//! takes the description's behaviour table, scale, child scale and particle scale
//! (`PHYSICSOBJREP::CompareAndModify`). A change that
//! only moves an appearance key's scalar (an FX ramping) changes no group and asks nothing.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ac2_physics::{Appearance, RebuiltVisual};

use crate::client::Client;
use crate::loader::{PartTreeJob, Request};

/// What a description makes of the part groups: the parent, each group's key, parent, connection
/// point and Setup, the appearance tables and key ids (not their scalars) of every group and of
/// the global mods, which can swap groups (`ModifyPGDTreeBasedOnApr`).
fn structure_key(visual: u32, appearance: &Appearance) -> u64 {
    let mut hasher = DefaultHasher::new();
    visual.hash(&mut hasher);
    appearance.parent.hash(&mut hasher);
    appearance.root.hash(&mut hasher);
    // What `PHYSICSOBJREP::CompareAndModify` takes besides the groups.
    appearance.behavior_table.hash(&mut hasher);
    for value in appearance
        .scale
        .iter()
        .chain([&appearance.child_scale, &appearance.particle_scale])
    {
        value.to_bits().hash(&mut hasher);
    }
    for group in std::iter::once(&appearance.global).chain(&appearance.part_groups) {
        (
            group.key,
            group.parent_key,
            group.connection_point,
            group.setup,
        )
            .hash(&mut hasher);
        for (table, keys) in &group.appearance {
            table.hash(&mut hasher);
            for (key, _) in keys {
                key.hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

/// Whether the description names no part group and no appearance key, no behaviour table of its
/// own and no scale: the tree the visual builds already follows it.
fn is_plain(appearance: &Appearance) -> bool {
    appearance.part_groups.is_empty()
        && appearance.global.appearance.is_empty()
        && appearance.behavior_table == 0
        && appearance.scale == [1.0; 3]
        && appearance.child_scale == 1.0
        && appearance.particle_scale == 1.0
}

impl Client {
    /// Asks the loader to rebuild the part tree of each object whose drawn description changed
    /// its structure since the tree was last built or asked for.
    pub(crate) fn refresh_part_trees(&mut self) {
        let candidates: Vec<(u64, u32, std::sync::Arc<Appearance>)> = self
            .session
            .objects()
            .iter()
            .filter_map(|o| Some((o.id, o.visual?, o.appearance.clone())))
            .collect();
        for (id, visual, local) in candidates {
            let drawn = self.drawn_appearance(id, local);
            let key = structure_key(visual, &drawn);
            let known = self.part_keys.get(&id).copied();
            if known == Some(key) {
                continue;
            }
            let Some((tree, current_table)) = self
                .script_body_mut(id)
                .and_then(|b| Some((b.part_group().cloned()?, b.behavior_table_did())))
            else {
                continue;
            };
            self.part_keys.insert(id, key);
            // The first sight of a plain description is the tree the visual built.
            if known.is_none() && is_plain(&drawn) {
                continue;
            }
            self.loader.request(Request::PartTree(
                id,
                key,
                Box::new(PartTreeJob {
                    tree,
                    visual,
                    local: ac2_physics::local_visual_desc(&drawn),
                    current_table,
                }),
            ));
        }
    }

    /// A rebuilt part tree: the body takes it when it is still the one asked for.
    pub(crate) fn part_tree_loaded(
        &mut self,
        id: u64,
        key: u64,
        tree: Result<Option<RebuiltVisual>, String>,
    ) {
        if self.part_keys.get(&id) != Some(&key) {
            return;
        }
        let Ok(Some(rebuilt)) = tree else {
            return;
        };
        if let Some(body) = self.script_body_mut(id) {
            body.set_part_group(std::sync::Arc::new(rebuilt.tree));
            body.cache_has_physics_bsp();
            // `PHYSICSOBJREP::CompareAndModify`: the new behaviour table, scale, child scale and
            // particle scale.
            body.compare_and_modify(rebuilt.facts);
        }
    }
}

#[cfg(feature = "test-support")]
impl Client {
    /// How many part groups the body of object `id` has (the player's or a remote's); `None`
    /// without a body or tree. For tests of the rebuild.
    pub fn part_group_count(&mut self, id: u64) -> Option<usize> {
        let body = self.script_body_mut(id)?;
        Some(body.part_group()?.indices().len())
    }

    /// What object `id`'s body took from its total description (`PHYSICSOBJREP::CompareAndModify`):
    /// the behaviour table DID, `VisualDesc` scale, child scale and particle scale. For tests.
    pub fn body_visual_facts(&mut self, id: u64) -> Option<(u32, [f32; 3], f32, f32)> {
        let body = self.script_body_mut(id)?;
        Some((
            body.behavior_table_did(),
            body.vdesc_scale(),
            body.child_scale(),
            body.particle_scale(),
        ))
    }
}
