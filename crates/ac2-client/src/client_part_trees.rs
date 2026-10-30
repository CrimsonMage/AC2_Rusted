//! `CPhysicsObj::ApplyVisualDesc`'s part rebuild: when an object's drawn description (its local
//! one with the FX's appearance keys merged on) changes what its part groups are, the body's
//! part-group tree follows it (`CPartGroup::CompareAndModify`, on a loader thread), so the
//! holding locations, bones and physics BSP flag are those of the object as drawn. A change that
//! only moves an appearance key's scalar (an FX ramping) changes no group and asks nothing.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ac2_physics::{Appearance, PartGroupTree};

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

/// Whether the description names no part group and no appearance key: the tree the visual
/// builds already follows it.
fn is_plain(appearance: &Appearance) -> bool {
    appearance.part_groups.is_empty() && appearance.global.appearance.is_empty()
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
            let Some(tree) = self
                .script_body_mut(id)
                .and_then(|b| b.part_group().cloned())
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
                }),
            ));
        }
    }

    /// A rebuilt part tree: the body takes it when it is still the one asked for.
    pub(crate) fn part_tree_loaded(
        &mut self,
        id: u64,
        key: u64,
        tree: Result<Option<PartGroupTree>, String>,
    ) {
        if self.part_keys.get(&id) != Some(&key) {
            return;
        }
        let Ok(Some(tree)) = tree else {
            return;
        };
        if let Some(body) = self.script_body_mut(id) {
            body.set_part_group(std::sync::Arc::new(tree));
            body.cache_has_physics_bsp();
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
}
