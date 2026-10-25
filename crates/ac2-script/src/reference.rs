//! A packed reference graph: what `ReferenceTable::PackReference` writes into a `WState` and
//! `ReferenceTable::UnPackReference` reads back.
//!
//! `ReferenceTable::UnPackReferenceArray`: a `PSmartArray<unsigned long>` of the packed
//! references' ids (the first is the root), then one `ReferenceEntry` per id
//! (`ReferenceEntry::UnPack`): the entry's flag word, then by its storage kind either a
//! `HeapObject` (`HeapObject::UnPack`), a WState DID, or a 4-byte value. The byte codec is
//! ac2-net's; this is the VM's run-time form of the graph.

use crate::native::NativeObject;
use crate::unpack::{UnpackError, UnpackResult};

/// `ReferenceEntry::m_blob` bits.
pub mod flags {
    /// Bits 8-9: the storage kind.
    pub const KIND_MASK: u32 = 0x300;
    pub const KIND_HEAP: u32 = 0x100;
    pub const KIND_VALUE: u32 = 0x200;
    /// The heap entry is a package singleton stored as its WState DID.
    pub const PACKAGE_DID: u32 = 0x2000_0000;
    /// A "relative" heap entry: packed without its object unless [`FORCE_PACK`] is also set.
    pub const RELATIVE: u32 = 0x0800_0000;
    pub const FORCE_PACK: u32 = 0x1000_0000;
}

/// One packed `ReferenceEntry`.
#[derive(Debug, Clone, PartialEq)]
pub enum PackedEntry {
    /// A heap object (`HeapObject::UnPack`).
    Heap { flags: u32, object: HeapObject },
    /// A relative heap entry packed without its object.
    Relative { flags: u32 },
    /// A package singleton named by WState DID (`VM::GetWStateReference`).
    Package { flags: u32, did: u32 },
    /// A 4-byte value entry.
    Value { flags: u32, value: u32 },
}

impl PackedEntry {
    pub fn flags(&self) -> u32 {
        match self {
            Self::Heap { flags, .. }
            | Self::Relative { flags }
            | Self::Package { flags, .. }
            | Self::Value { flags, .. } => *flags,
        }
    }
}

/// `HeapObject` as packed: a package instance (4-byte slots, each with a tag byte saying what
/// it holds) or a native object.
#[derive(Debug, Clone, PartialEq)]
pub struct HeapObject {
    pub pkg_state_id: u32,
    /// `m_pkgid`, `u32::MAX` when the packed 16 bits were 0xFFFF.
    pub pkg_id: u32,
    /// `m_ntype`, 0 for a package instance.
    pub native_type: u32,
    pub native: Option<NativeObject>,
    /// `m_data`: the slots, as raw words.
    pub slots: Vec<u32>,
    /// `m_tags`: one tag per slot (`tag & 0xF == 1` marks a reference slot).
    pub tags: Vec<u8>,
}

/// A packed reference graph.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PackedReference {
    /// The packed reference ids, in pack order; the first is the root.
    pub ids: Vec<u32>,
    /// One entry per id.
    pub entries: Vec<PackedEntry>,
}

impl PackedReference {
    /// `ReferenceTable::UnPackReferenceArray` over all of `bytes`, through ac2-net's codec
    /// (`ac2_net::protocol::PackedReference::from_bytes`), into the VM's run-time objects.
    pub fn unpack(bytes: &[u8]) -> UnpackResult<Self> {
        let packed =
            ac2_net::protocol::PackedReference::from_bytes(bytes).map_err(|e| UnpackError {
                context: protocol_context(&e),
                position: 0,
            })?;
        Ok(crate::wire::from_net(&packed))
    }

    /// The root entry (the packed reference the WState saved).
    pub fn root(&self) -> Option<&PackedEntry> {
        self.entries.first()
    }
}

fn protocol_context(e: &ac2_net::protocol::ProtocolError) -> &'static str {
    use ac2_net::protocol::ProtocolError as E;
    match e {
        E::Truncated => "packed references end early",
        E::TrailingBytes => "bytes left after the packed references",
        E::Invalid(c) | E::Unsupported(c) => c,
        E::String(_) => "a packed string does not decode",
    }
}

impl HeapObject {
    /// `HeapObject::UnPack`'s library check: a package instance must have exactly
    /// `PackageInfo::size` slots for its package (native objects are not checked).
    pub fn matches_library(&self, package_info: &[crate::bytestream::PackageInfo]) -> bool {
        self.native_type != 0
            || package_info
                .get(self.pkg_id as usize)
                .is_some_and(|info| info.size as usize == self.slots.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn a_package_instance_and_a_native_list_unpack_to_the_last_byte() {
        let mut bytes = words(&[2, 7, 9]); // two packed ids
                                           // Entry 7: a package instance of package 0x12 with two slots.
        bytes.extend(words(&[
            flags::KIND_HEAP,
            0,
            0x0012_0000,
            2,
            0x11,
            0xFFFF_FFFF,
        ]));
        bytes.extend([4, 1, 0, 0]); // tags, padded to 4
                                    // Entry 9: an RList native (type 0x65, no package) holding one reference.
        bytes.extend(words(&[flags::KIND_HEAP
            | flags::FORCE_PACK
            | flags::RELATIVE]));
        bytes.extend(words(&[0, 0xFFFF_0065, 1, 7, 0]));
        let refs = PackedReference::unpack(&bytes).unwrap();
        assert_eq!(refs.ids, vec![7, 9]);
        let PackedEntry::Heap { object, .. } = &refs.entries[0] else {
            panic!("{:?}", refs.entries[0]);
        };
        assert_eq!((object.pkg_id, object.native_type), (0x12, 0));
        assert_eq!(object.slots, vec![0x11, 0xFFFF_FFFF]);
        assert_eq!(object.tags, vec![4, 1]);
        let PackedEntry::Heap { object, .. } = &refs.entries[1] else {
            panic!();
        };
        assert_eq!(object.pkg_id, u32::MAX);
        let Some(crate::native::NativeObject::List(list)) = &object.native else {
            panic!("{:?}", object.native);
        };
        assert_eq!(list.items, vec![7]);
        assert!(object.slots.is_empty());
    }

    #[test]
    fn relative_package_and_value_entries_carry_no_heap_object() {
        let bytes = words(&[
            3,
            0,
            1,
            2,
            flags::KIND_HEAP | flags::RELATIVE,
            flags::KIND_HEAP | flags::PACKAGE_DID,
            0x7000_0123,
            flags::KIND_VALUE,
            42,
        ]);
        let refs = PackedReference::unpack(&bytes).unwrap();
        assert!(matches!(refs.entries[0], PackedEntry::Relative { .. }));
        assert!(matches!(
            refs.entries[1],
            PackedEntry::Package {
                did: 0x7000_0123,
                ..
            }
        ));
        assert!(matches!(
            refs.entries[2],
            PackedEntry::Value { value: 42, .. }
        ));
        // A storage kind of 0 fails, as `ReferenceEntry::UnPack` does.
        assert!(PackedReference::unpack(&words(&[1, 0, 0])).is_err());
        // Trailing bytes fail.
        assert!(PackedReference::unpack(&words(&[0, 0])).is_err());
    }
}
