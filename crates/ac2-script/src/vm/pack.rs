//! `ReferenceTable::PackReference`: a reference and everything it reaches, as a
//! [`PackedReference`] (`GetDependentReferences` then `PackReferenceArray`), and
//! `VM::CloneReference` (pack, then unpack as new references).

use std::collections::HashSet;

use super::refs::{flags, native_references, ReferenceTable, Target};
use super::tag;
use crate::reference::{PackedEntry, PackedReference};

impl ReferenceTable {
    /// `ReferenceTable::GetDependentReferences(ref, ..., checkpointOnly = 0)`: depth first,
    /// each reference before what it holds, each once. Singletons and relative entries (unless
    /// forced) stop the walk; an address entry continues to its base.
    fn dependents(&self, root: u32) -> Vec<u32> {
        let mut order = Vec::new();
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        while let Some(r) = stack.pop() {
            if r == u32::MAX || !seen.insert(r) {
                continue;
            }
            let Ok(entry) = self.get(r) else {
                continue;
            };
            order.push(r);
            let blob = entry.blob;
            let walk = blob & flags::PACKAGE_DID == 0
                && (blob & flags::RELATIVE == 0 || blob & flags::FORCE_PACK != 0);
            if !walk {
                continue;
            }
            let mut children = Vec::new();
            match &entry.target {
                Target::Heap(object) if entry.kind() == flags::KIND_HEAP => {
                    if let Some(native) = &object.native {
                        let mut native = native.clone();
                        native_references(object.native_type, &mut native, &mut |c| {
                            children.push(*c)
                        });
                    } else {
                        for (value, t) in object.slots.iter().zip(&object.tags) {
                            if *t != tag::UNINIT && t & 0xF == tag::REF {
                                children.push(*value);
                            }
                        }
                    }
                }
                Target::Ref(base) if entry.kind() == flags::KIND_VALUE => children.push(*base),
                _ => {}
            }
            // Visit children in order: push them reversed.
            stack.extend(children.into_iter().rev());
        }
        order
    }

    /// `ReferenceTable::PackReference`: the ids in walk order and each `ReferenceEntry::Pack`
    /// (the flag word without the GC stamp; a singleton as its WState DID; a relative entry
    /// without its object; an address as its base).
    pub fn pack(&self, root: u32) -> PackedReference {
        let ids = self.dependents(root);
        let entries = ids
            .iter()
            .filter_map(|&r| {
                let entry = self.get(r).ok()?;
                let blob = entry.blob & 0xFFFF_FF00;
                Some(match &entry.target {
                    Target::Heap(object) if blob & flags::PACKAGE_DID != 0 => {
                        PackedEntry::Package {
                            flags: blob,
                            did: object.pkg_state_id,
                        }
                    }
                    Target::Heap(_)
                        if blob & flags::RELATIVE != 0 && blob & flags::FORCE_PACK == 0 =>
                    {
                        PackedEntry::Relative { flags: blob }
                    }
                    Target::Heap(object) => PackedEntry::Heap {
                        flags: blob,
                        object: (**object).clone(),
                    },
                    Target::Ref(base) => PackedEntry::Value {
                        flags: blob,
                        value: *base,
                    },
                    Target::Empty => PackedEntry::Value {
                        flags: blob,
                        value: 0,
                    },
                })
            })
            .collect();
        PackedReference { ids, entries }
    }

    /// `VM::CloneReference`: a live reference packed and unpacked again (`u32::MAX` for a dead
    /// or unknown one).
    pub fn clone_reference(&mut self, reference: u32) -> u32 {
        let live = self
            .get(reference)
            .is_ok_and(|e| e.blob & 0xFF == u32::from(self.gc_run));
        if reference == u32::MAX || !live {
            return u32::MAX;
        }
        let packed = self.pack(reference);
        self.unpack(&packed, None).unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::{ntype, NativeObject, ScriptList};

    #[test]
    fn clone_copies_what_the_reference_reaches_but_not_singletons() {
        let mut refs = ReferenceTable::new();
        let single = refs.singleton(0x7000_0042);
        let list = refs.create_native(
            ntype::A_LIST,
            NativeObject::List(ScriptList {
                wide: false,
                items: vec![1, 2],
            }),
        );
        let mut inst = ReferenceTable::new_instance(7, 3);
        inst.slots = vec![list, single, 99];
        inst.tags = vec![tag::REF, tag::REF, tag::INT];
        let root = refs.add(
            flags::KIND_HEAP | flags::FILLED,
            Target::Heap(Box::new(inst)),
        );
        let packed = refs.pack(root);
        assert_eq!(packed.ids, vec![root, list, single]);
        assert!(matches!(
            packed.entries[2],
            PackedEntry::Package {
                did: 0x7000_0042,
                ..
            }
        ));
        let copy = refs.clone_reference(root);
        assert_ne!(copy, root);
        let slots = refs.heap(copy).unwrap().slots.clone();
        assert_ne!(slots[0], list);
        assert_eq!(slots[1], single);
        assert_eq!(slots[2], 99);
        assert!(matches!(
            refs.heap(slots[0]).unwrap().native,
            Some(NativeObject::List(ref l)) if l.items == vec![1, 2]
        ));
        assert_eq!(refs.clone_reference(u32::MAX), u32::MAX);
    }
}
