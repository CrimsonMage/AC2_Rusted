//! `gPlacesTable` (`gPlacesTable.cpp`): named places from the `PlacesTable`
//! (`PlacesTable::InqPlace`, `InqPlacesHash`). `InqPlaceStr` (`Position::ToWPString`) and
//! `WslDump` are not bound.

use super::hashes::with_hash;
use super::{word, Call, INT_RESULT};
use crate::native::{ntype, NativeObject, RefKey, DEFAULT_BUCKETS};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0058_0001, inq_place);
    efuns.register(0x0058_0004, inq_places_hash);
}

/// `InqPlace(name, out position)`: 1 and the place's cell and frame, or 0.
fn inq_place(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let name = vm.wstring(word(ctx, 2)).unwrap_or(&[]).to_vec();
    let out = word(ctx, 3);
    let place = vm
        .data()
        .places_table()
        .and_then(|t| t.place(&name).copied());
    if let Some(place) = place {
        let target = vm.read_through(ctx, out).unwrap_or(u32::MAX);
        if let Some(NativeObject::Position(p)) = vm.native_mut(target) {
            *p = place;
        }
    }
    Call::new(ctx).ret(4, u32::from(place.is_some()), INT_RESULT);
    InterpStatus::Success
}

/// `InqPlacesHash(out NRHash)`: the table copied into a fresh `PHashTable` (47 buckets, each
/// place at its bucket's head), then `PlacesHashToNRHash`: a new `wpstring` and `Position` per
/// place, keyed by `ReferenceIDWrapper(name)`. 0 without a table.
fn inq_places_hash(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let out = word(ctx, 2);
    let Some(table) = vm.data().places_table().map(|t| t.places.clone()) else {
        Call::new(ctx).ret(3, 0, INT_RESULT);
        return InterpStatus::Success;
    };
    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); DEFAULT_BUCKETS as usize];
    for (i, (name, _)) in table.iter().enumerate() {
        let b = (super::compute_hash(name) % DEFAULT_BUCKETS) as usize;
        if !buckets[b].iter().any(|&j| table[j].0 == *name) {
            buckets[b].insert(0, i);
        }
    }
    let pairs: Vec<(u32, u32)> = buckets
        .into_iter()
        .flatten()
        .map(|i| {
            let (name, place) = &table[i];
            let key = vm.new_wstring(name.clone());
            let value = vm
                .refs
                .create_native(ntype::POSITION, NativeObject::Position(*place));
            (key, value)
        })
        .collect();
    let target = vm.read_through(ctx, out).unwrap_or(u32::MAX);
    with_hash(vm, target, |h, keys, _| {
        h.flush();
        for (k, v) in pairs {
            let key = RefKey::Ref {
                id: k,
                cached_hash: 0,
                use_cache: 0,
            };
            h.add(key, u64::from(v), keys);
        }
    });
    Call::new(ctx).ret(3, 1, INT_RESULT);
    InterpStatus::Success
}
