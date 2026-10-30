//! Loading what a body animates with from the AC2 DATs (`PHYSICSOBJREP::InitVisualDescObject`'s
//! behaviour table, `CPartArray`'s `AnimMap` and Setup).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_anim::{load_ac2_db_animator, Ac2BehaviorTable, Ac2CharTemplate, Ac2Setup};
use ac2_assets::{
    resolve_ac2_visual_animation, resolve_ac2_visual_parts, resolve_ac2_visual_scale,
};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourcePackedDecode, ResourceResult, ResourceStore, TypedResourceRef};

use crate::anim_rep::BodyAnimation;

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
    let (Some(table_did), Some(anim_map)) = resolve_ac2_visual_animation(store, visual)? else {
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
    let mut animators = HashMap::new();
    for key in keys {
        if let Some(animator) = load_ac2_db_animator(store, anim_map, key)? {
            animators.insert(key, animator);
        }
    }
    let setup = match resolve_ac2_visual_parts(store, visual)?.first() {
        Some(&(setup, _)) => load::<Ac2Setup>(store, setup)?,
        None => None,
    };
    let scale = resolve_ac2_visual_scale(store, visual)?;
    Ok(Some(BodyAnimation {
        table,
        template,
        animators,
        setup,
        vdesc_scale: [scale.x, scale.y, scale.z],
    }))
}
