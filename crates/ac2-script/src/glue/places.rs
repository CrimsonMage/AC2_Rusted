//! `gPlacesTable` (`gPlacesTable.cpp`): named places from the `PlacesTable`
//! (`PlacesTable::InqPlace`, `InqPlaceStr`, `InqPlacesHash`). `WslDump` is not bound.

use super::hashes::with_hash;
use super::position::to_wp_string;
use super::{out_native, word, Call, INT_RESULT};
use crate::native::{ntype, NativeObject, RefKey, DEFAULT_BUCKETS};
use crate::vm::{Context, EFuns, InterpStatus, Vm};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x0058_0001, inq_place);
    efuns.register(0x0058_0002, inq_place_str);
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

/// `PlacesTable::InqPlaceStr(name, out wpstring)`: `Position::ToWPString` of the place, written
/// to the out string and 1 when the place exists and its string is not empty; 0 (out untouched)
/// otherwise, or without a table.
fn inq_place_str(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let name = vm.wstring(word(ctx, 2)).unwrap_or(&[]).to_vec();
    let out = word(ctx, 3);
    let text = vm
        .data()
        .places_table()
        .and_then(|t| t.place(&name).copied())
        .map(|place| to_wp_string(&place))
        .filter(|text| !text.is_empty());
    let found = text.is_some();
    if let Some(text) = text {
        let target = out_native(vm, ctx, out);
        if let Some(NativeObject::WString(slot)) = vm.native_mut(target) {
            *slot = text.encode_utf16().collect();
        }
    }
    Call::new(ctx).ret(4, u32::from(found), INT_RESULT);
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

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::sync::Arc;

    use super::*;
    use crate::data::PlacesTable;
    use crate::glue::register_pure;
    use crate::vm::{tag, Program, ScriptData, StateRef};
    use ac2_net::protocol::Position;

    struct Data(Arc<PlacesTable>);
    struct Held(Arc<PlacesTable>);
    impl Deref for Held {
        type Target = PlacesTable;
        fn deref(&self) -> &PlacesTable {
            &self.0
        }
    }
    impl ScriptData for Data {
        fn wstate(&self, _did: u32) -> Option<StateRef> {
            None
        }
        fn places_table(&self) -> Option<Box<dyn Deref<Target = PlacesTable>>> {
            Some(Box::new(Held(self.0.clone())))
        }
    }

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    fn call(vm: &mut Vm, name: u32, out: u32) -> u32 {
        let mut ctx = Context::new(1, None);
        ctx.enter(0).unwrap();
        ctx.push(0, tag::INT).unwrap();
        ctx.push(out, tag::REF).unwrap();
        ctx.push(name, tag::REF).unwrap();
        ctx.push(u32::MAX, tag::REF).unwrap();
        assert_eq!(vm.run_efun(0x0058_0002, &mut ctx), InterpStatus::Success);
        ctx.data[0]
    }

    #[test]
    fn inq_place_str_writes_the_places_position_string() {
        let table = PlacesTable {
            did: 0x4000_0054,
            bucket_index: 0,
            places: vec![(
                w("fort"),
                Position {
                    cell: 0x7F7F_001E,
                    origin: [70.0, 110.0, 0.5],
                    rotation: [1.0, 0.0, 0.0, 0.0],
                },
            )],
        };
        let mut efuns = EFuns::new();
        register_pure(&mut efuns);
        let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
        let mut vm = Vm::new(Arc::new(program), Box::new(Data(Arc::new(table))), efuns);
        // Names are looked up lowercased.
        let name = vm.new_wstring(w("FORT"));
        let out = vm.new_wstring(w("old"));
        assert_eq!(call(&mut vm, name, out), 1);
        assert_eq!(
            vm.wstring(out).unwrap(),
            w("0x7F7F001E [70.000000 110.000000 0.500000] 1.000000 0.000000 0.000000 0.000000")
                .as_slice()
        );
        // A missing place returns 0 and leaves the out string alone.
        let name = vm.new_wstring(w("nowhere"));
        let out = vm.new_wstring(w("old"));
        assert_eq!(call(&mut vm, name, out), 0);
        assert_eq!(vm.wstring(out).unwrap(), w("old").as_slice());
    }
}
