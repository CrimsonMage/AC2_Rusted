//! `ReferenceTable`: every script-visible object is a reference id into this table. An entry is
//! a flag word (`ReferenceEntry::m_blob`) and either a heap object (kind 0x100) or another
//! reference plus a slot offset (kind 0x200, an address made by `SAddr`/`RAddr`; reference
//! `u32::MAX` there means "the running context's stack").
//!
//! Not ported yet: the mark-and-sweep collector (`StartGarbageCollection`) and reference reuse
//! (`FindNextReference`); entries are only ever appended.

use std::collections::HashMap;

use super::context::Context;
use super::tag;
use crate::native::{NativeObject, RefKey};
use crate::reference::{flags as packed, HeapObject, PackedEntry, PackedReference};

/// `ReferenceEntry::m_blob` bits beyond the packed ones in [`crate::reference::flags`].
pub mod flags {
    pub use crate::reference::flags::*;
    /// The heap object's data is present (a singleton is filled lazily).
    pub const FILLED: u32 = 0x4000_0000;
    /// Entries the collector traverses.
    pub const TRACED: u32 = 0x0400_0000;
    /// Bits 10-25: the slot offset of an address entry.
    pub const OFFSET_SHIFT: u32 = 10;
}

/// One `ReferenceEntry`.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Heap(Box<HeapObject>),
    /// An address entry's base reference (`u32::MAX`: the context stack).
    Ref(u32),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub blob: u32,
    pub target: Target,
}

impl Entry {
    pub fn kind(&self) -> u32 {
        self.blob & flags::KIND_MASK
    }

    pub fn offset(&self) -> u32 {
        (self.blob >> flags::OFFSET_SHIFT) & 0xFFFF
    }
}

/// Where a slot lives: on the running context's stack or in a heap object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Stack(usize),
    Heap { reference: u32, index: usize },
}

/// A reference that does not resolve (retail reads freed or foreign memory).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BadReference(pub u32);

#[derive(Debug, Clone, Default)]
pub struct ReferenceTable {
    pub entries: Vec<Entry>,
    /// `m_wstateTable`: package singletons by WState DID.
    pub wstates: HashMap<u32, u32>,
    /// `m_gcRun`, stamped into each new entry's low byte.
    pub gc_run: u8,
}

impl ReferenceTable {
    pub fn new() -> Self {
        Self {
            gc_run: 1,
            ..Self::default()
        }
    }

    /// `ReferenceTable::AddReference`.
    pub fn add(&mut self, blob: u32, target: Target) -> u32 {
        let id = self.entries.len() as u32;
        self.entries.push(Entry {
            blob: (blob & !0xFF) | u32::from(self.gc_run),
            target,
        });
        id
    }

    pub fn get(&self, reference: u32) -> Result<&Entry, BadReference> {
        self.entries
            .get(reference as usize)
            .ok_or(BadReference(reference))
    }

    pub fn get_mut(&mut self, reference: u32) -> Result<&mut Entry, BadReference> {
        self.entries
            .get_mut(reference as usize)
            .ok_or(BadReference(reference))
    }

    pub fn heap(&self, reference: u32) -> Result<&HeapObject, BadReference> {
        match &self.get(reference)?.target {
            Target::Heap(object) => Ok(object),
            _ => Err(BadReference(reference)),
        }
    }

    pub fn heap_mut(&mut self, reference: u32) -> Result<&mut HeapObject, BadReference> {
        match &mut self.get_mut(reference)?.target {
            Target::Heap(object) => Ok(object),
            _ => Err(BadReference(reference)),
        }
    }

    /// A package instance's heap object with `size` uninitialised slots
    /// (`HeapObject::InitializeData`).
    pub fn new_instance(pkg_id: u32, size: u32) -> HeapObject {
        HeapObject {
            pkg_state_id: 0,
            pkg_id,
            native_type: 0,
            native: None,
            slots: vec![0xCDCD_CDCD; size as usize],
            tags: vec![tag::UNINIT; size as usize],
        }
    }

    /// `ReferenceTable::CreateNativeObject`.
    pub fn create_native(&mut self, native_type: u32, native: NativeObject) -> u32 {
        let object = HeapObject {
            pkg_state_id: 0,
            pkg_id: u32::MAX,
            native_type,
            native: Some(native),
            slots: Vec::new(),
            tags: Vec::new(),
        };
        self.add(
            flags::FILLED | flags::KIND_HEAP,
            Target::Heap(Box::new(object)),
        )
    }

    /// The singleton entry for WState `did`, made unfilled if new (`ReferenceTable::GetSingleton`).
    pub fn singleton(&mut self, did: u32) -> u32 {
        if let Some(&reference) = self.wstates.get(&did) {
            return reference;
        }
        let object = HeapObject {
            pkg_state_id: did,
            pkg_id: u32::MAX,
            native_type: 0,
            native: None,
            slots: Vec::new(),
            tags: Vec::new(),
        };
        let reference = self.add(
            flags::PACKAGE_DID | flags::KIND_HEAP,
            Target::Heap(Box::new(object)),
        );
        self.wstates.insert(did, reference);
        reference
    }

    /// `ReferenceTable::GetReferenceTagAndPtr` / `GetReferencePtr`: the slot `offset` words
    /// into `reference`, following address entries. `Ok(None)` is a heap package that is not
    /// filled yet (the caller fills it and asks again).
    pub fn locate(&self, reference: u32, offset: i64) -> Result<Option<Slot>, BadReference> {
        let mut reference = reference;
        let mut offset = offset;
        for _ in 0..self.entries.len() + 1 {
            let entry = self.get(reference)?;
            if entry.kind() == flags::KIND_HEAP {
                if entry.blob & flags::FILLED == 0 {
                    return Ok(None);
                }
                return Ok(Some(Slot::Heap {
                    reference,
                    index: usize::try_from(offset).map_err(|_| BadReference(reference))?,
                }));
            }
            offset += i64::from(entry.offset());
            match entry.target {
                Target::Ref(u32::MAX) => {
                    return Ok(Some(Slot::Stack(
                        usize::try_from(offset).map_err(|_| BadReference(reference))?,
                    )))
                }
                Target::Ref(next) => reference = next,
                _ => return Err(BadReference(reference)),
            }
        }
        Err(BadReference(reference))
    }

    /// Reads a located slot.
    pub fn read(&self, ctx: &Context, slot: Slot) -> Result<(u32, u8), BadReference> {
        match slot {
            Slot::Stack(index) => ctx.slot(index).map_err(|_| BadReference(u32::MAX)),
            Slot::Heap { reference, index } => {
                let object = self.heap(reference)?;
                match (object.slots.get(index), object.tags.get(index)) {
                    (Some(&v), Some(&t)) => Ok((v, t)),
                    _ => Err(BadReference(reference)),
                }
            }
        }
    }

    /// Writes a located slot.
    pub fn write(
        &mut self,
        ctx: &mut Context,
        slot: Slot,
        value: u32,
        tag: u8,
    ) -> Result<(), BadReference> {
        match slot {
            Slot::Stack(index) => ctx
                .set_slot(index, value, tag)
                .map_err(|_| BadReference(u32::MAX)),
            Slot::Heap { reference, index } => {
                let object = self.heap_mut(reference)?;
                match (object.slots.get_mut(index), object.tags.get_mut(index)) {
                    (Some(v), Some(t)) => {
                        *v = value;
                        *t = tag;
                        Ok(())
                    }
                    _ => Err(BadReference(reference)),
                }
            }
        }
    }

    /// `ReferenceTable::UnPackReferenceArray` + `UpdateReferences`: adds a packed graph and
    /// returns the root's new reference. With `lazy_root`, the first new heap object fills that
    /// existing entry instead (`FillPackage`). Package-DID entries resolve to their singleton
    /// (`VM::GetWStateReference`).
    pub fn unpack(&mut self, packed: &PackedReference, lazy_root: Option<u32>) -> Option<u32> {
        let mut map: HashMap<u32, u32> = HashMap::with_capacity(packed.ids.len());
        let mut new_refs = Vec::with_capacity(packed.ids.len());
        let mut root_used = false;
        for (old, entry) in packed.ids.iter().zip(&packed.entries) {
            let (blob, target) = match entry {
                PackedEntry::Package { did, .. } => {
                    let reference = self.singleton(*did);
                    map.insert(*old, reference);
                    continue;
                }
                PackedEntry::Relative { .. } => {
                    map.insert(*old, u32::MAX);
                    continue;
                }
                PackedEntry::Heap { flags, object } => {
                    // A forced relative entry loses the force bit once unpacked.
                    let blob = if flags & packed::RELATIVE != 0 {
                        flags & !packed::FORCE_PACK
                    } else {
                        *flags
                    };
                    (blob, Target::Heap(Box::new(object.clone())))
                }
                PackedEntry::Value { flags, value } => (*flags, Target::Ref(*value)),
            };
            let reference = match lazy_root {
                Some(root) if !root_used => {
                    root_used = true;
                    if let Ok(e) = self.get_mut(root) {
                        e.target = target;
                    }
                    root
                }
                _ => self.add(blob, target),
            };
            map.insert(*old, reference);
            new_refs.push(reference);
        }
        for reference in new_refs {
            self.update_references(reference, &map, lazy_root);
        }
        packed.ids.first().and_then(|root| map.get(root).copied())
    }

    /// `HeapObject::UpdateReferences` / `UpdateRelativeReferences` for one new entry.
    fn update_references(&mut self, reference: u32, map: &HashMap<u32, u32>, lazy: Option<u32>) {
        let Ok(entry) = self.get_mut(reference) else {
            return;
        };
        if entry.blob & flags::PACKAGE_DID != 0 && lazy != Some(reference) {
            return;
        }
        let remap = |value: &mut u32| {
            if *value != u32::MAX {
                if let Some(&new) = map.get(value) {
                    *value = new;
                }
            }
        };
        match &mut entry.target {
            Target::Ref(base) => {
                if *base != u32::MAX {
                    if let Some(&new) = map.get(base) {
                        *base = new;
                    }
                }
            }
            Target::Heap(object) => {
                if let Some(native) = &mut object.native {
                    native_references(object.native_type, native, &mut |r| remap(r));
                } else {
                    for (value, tag) in object.slots.iter_mut().zip(&object.tags) {
                        if *tag != tag::UNINIT && tag & 0xF == tag::REF {
                            remap(value);
                        }
                    }
                }
            }
            Target::Empty => {}
        }
    }
}

/// The reference-holding parts of a native object (`CNativeFactory::UpdateReferences`): the
/// elements of `R` lists and arrays, the values of `XR` hashes, the keys of `N` hashes.
pub fn native_references(
    native_type: u32,
    native: &mut NativeObject,
    visit: &mut dyn FnMut(&mut u32),
) {
    use crate::native::ntype;
    match native {
        NativeObject::List(list) if matches!(native_type, ntype::R_LIST | ntype::R_ARRAY) => {
            for item in &mut list.items {
                let mut r = *item as u32;
                visit(&mut r);
                *item = u64::from(r);
            }
        }
        NativeObject::Hash(hash) => {
            if matches!(
                native_type,
                ntype::AR_HASH | ntype::LR_HASH | ntype::NR_HASH
            ) {
                for value in &mut hash.values {
                    let mut r = *value as u32;
                    visit(&mut r);
                    *value = u64::from(r);
                }
            }
            if matches!(native_type, ntype::NA_HASH | ntype::NR_HASH) {
                for key in &mut hash.keys {
                    if let RefKey::Ref { id, .. } = key {
                        visit(id);
                    }
                }
            }
        }
        _ => {}
    }
}
