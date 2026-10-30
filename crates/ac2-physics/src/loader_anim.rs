//! Loading what a body animates with from the AC2 DATs (`PHYSICSOBJREP::InitVisualDescObject`'s
//! behaviour table, `CPartArray`'s `AnimMap` and Setup).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{load_ac2_db_animator, Ac2BehaviorTable, Ac2CharTemplate, Ac2Setup};
use ac2_assets::{
    resolve_ac2_visual_animation, resolve_ac2_visual_parts, resolve_ac2_visual_scale,
    resolve_drawn_groups,
};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourcePackedDecode, ResourceResult, ResourceStore, TypedResourceRef};

use crate::anim_rep::{BodyAnimation, ChildGroupAnimation};

/// `CharTemplate` (DB 0x40000030).
const CHAR_TEMPLATE: u32 = 0x2F00_0000;

fn load<T: ResourcePackedDecode + Send + Sync + Clone + 'static>(
    store: &ResourceStore,
    did: DataId,
) -> ResourceResult<Option<Arc<T>>> {
    Ok(store
        .load_cached_packed::<T>(TypedResourceRef::new(DatFileKind::Portal, did, 0))?
        .map(|cached| Arc::new((*cached).clone())))
}

/// What the body `visual` places animates with: its behaviour table (`BehaviorTable::Get`, a
/// `DBObj::Get` of type `0x40000040`), the `CharTemplate`, the
/// animator of every anim key the table's combos name, its root Setup and its `VisualDesc`
/// scale. `None` without a behaviour table or `AnimMap`.
pub fn load_body_animation(
    store: &ResourceStore,
    visual: DataId,
) -> ResourceResult<Option<BodyAnimation>> {
    load_body_animation_with(store, visual, None, None)
}

/// [`load_body_animation`] for a total description that names its own behaviour table
/// (`table`) and `VisualDesc` scale (`scale`) over the visual's chain
/// (`PHYSICSOBJREP::CompareAndModify`'s new table).
pub fn load_body_animation_with(
    store: &ResourceStore,
    visual: DataId,
    table: Option<DataId>,
    scale: Option<ac2_math::Vec3>,
) -> ResourceResult<Option<BodyAnimation>> {
    let (chain_table, Some(anim_map)) = resolve_ac2_visual_animation(store, visual)? else {
        return Ok(None);
    };
    let Some(table_did) = table.or(chain_table) else {
        return Ok(None);
    };
    let Some(table) = load::<Ac2BehaviorTable>(store, table_did)? else {
        return Ok(None);
    };
    let Some(template) = load::<Ac2CharTemplate>(store, DataId::new(CHAR_TEMPLATE))? else {
        return Ok(None);
    };
    let mut keys: Vec<u32> = table.combos.values().map(|combo| combo.anim_key).collect();
    keys.sort_unstable();
    keys.dedup();
    let animators = load_animators(store, anim_map, &keys)?;
    let child_groups = load_child_groups(store, visual, &keys)?;
    let setup = match resolve_ac2_visual_parts(store, visual)?.first() {
        Some(&(setup, _)) => load::<Ac2Setup>(store, setup)?,
        None => None,
    };
    let scale = match scale {
        Some(scale) => scale,
        None => resolve_ac2_visual_scale(store, visual)?,
    };
    Ok(Some(BodyAnimation {
        table_did: table_did.raw(),
        table,
        template,
        animators,
        setup,
        vdesc_scale: [scale.x, scale.y, scale.z],
        child_groups,
    }))
}

/// The animator `anim_map` gives each of `keys` (`CPartArray::GetAnimator` over
/// `AnimMap::GetAnimator`).
fn load_animators(
    store: &ResourceStore,
    anim_map: DataId,
    keys: &[u32],
) -> ResourceResult<HashMap<u32, Arc<ac2_anim::Ac2DbAnimator>>> {
    let mut animators = HashMap::new();
    for &key in keys {
        if let Some(animator) = load_ac2_db_animator(store, anim_map, key)? {
            animators.insert(key, animator);
        }
    }
    Ok(animators)
}

/// The child part groups of `visual`'s tree that name an `AnimMap` (`CPartArray::SetAnimMapID`),
/// in `CPartGroup::Init`'s child order, each with the animators `keys` map to there. The root
/// group is the body's own animation.
fn load_child_groups(
    store: &ResourceStore,
    visual: DataId,
    keys: &[u32],
) -> ResourceResult<Vec<ChildGroupAnimation>> {
    let Some(groups) = resolve_drawn_groups(store, visual, None)? else {
        return Ok(Vec::new());
    };
    let mut children = Vec::new();
    for group in groups.iter().skip(1) {
        let Some(anim_map) = group.anim_map else {
            continue;
        };
        let Some(setup) = load::<Ac2Setup>(store, group.setup)? else {
            continue;
        };
        children.push(ChildGroupAnimation {
            key: group.key,
            setup,
            animators: load_animators(store, anim_map, keys)?,
        });
    }
    Ok(children)
}
