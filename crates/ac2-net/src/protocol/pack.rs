//! Script-call arguments: the `PackBuffer` an `InterpSEvent`/`InterpCEvent` carries.
//!
//! A script packs each argument with a type tag (`gPackBuffer::CallEFun_PackInt` and siblings:
//! 1 int, 2 long int, 3 float, 4 long float, 5 package reference); the receiver pushes them in
//! order (`PackBuffer::UnPackBuffer`). A reference is a packed reference graph
//! (`ReferenceTable::PackReference`, read by `ReferenceTable::UnPackReferenceArray`): the
//! packed reference ids (the first is the root), then one `ReferenceEntry` per id.
//! Alignment counts from the start of the buffer's data, which retail allocates 4-aligned.

use crate::protocol::native::NativeObject;
use crate::protocol::reader::{Reader, Writer, MAX_ITEMS};
use crate::protocol::ProtocolError;

/// `ReferenceEntry::m_field` bits (`ReferenceEntryField`: gcRun 0-7, type 8-9, offset 10-25,
/// recurse 26, process 27, root 28, singleton 29, loaded 30). `ReferenceEntry::Pack` writes
/// the word without `gcRun`.
pub mod entry_flags {
    pub const KIND_MASK: u32 = 0x300;
    /// A heap object (`HeapObject`).
    pub const KIND_HEAP: u32 = 0x100;
    /// A 4-byte value.
    pub const KIND_VALUE: u32 = 0x200;
    pub const RECURSE: u32 = 0x0400_0000;
    pub const PROCESS: u32 = 0x0800_0000;
    pub const ROOT: u32 = 0x1000_0000;
    /// A package singleton, packed as its `WState` DID.
    pub const SINGLETON: u32 = 0x2000_0000;
    pub const LOADED: u32 = 0x4000_0000;
}

/// One packed argument.
#[derive(Debug, Clone, PartialEq)]
pub enum PackValue {
    /// Tag 1 (`PackInt`).
    Int(u32),
    /// Tag 2 (`PackLInt`): instance ids and other 64-bit integers.
    LongInt(u64),
    /// Tag 3 (`PackFloat`).
    Float(f32),
    /// Tag 4 (`PackLFloat`).
    LongFloat(f64),
    /// Tag 5 (`PackPackage`).
    Reference(PackedReference),
}

impl PackValue {
    /// A reference to one native object (`HandleNewNative` then `PackPackage`).
    pub fn native(object: NativeObject) -> Self {
        Self::Reference(PackedReference::native(object))
    }

    /// The native object a reference's root holds.
    pub fn as_native(&self) -> Option<&NativeObject> {
        match self {
            Self::Reference(reference) => reference.root_native(),
            _ => None,
        }
    }
}

/// `PackBuffer`'s data for `values`, in order (without the size `PackBuffer::Pack` puts first).
pub fn pack_values(values: &[PackValue]) -> Vec<u8> {
    let mut w = Writer::new();
    for value in values {
        match value {
            PackValue::Int(v) => {
                w.u32(1).u32(*v);
            }
            PackValue::LongInt(v) => {
                w.u32(2).u64(*v);
            }
            PackValue::Float(v) => {
                w.u32(3).f32(*v);
            }
            PackValue::LongFloat(v) => {
                w.u32(4).f64(*v);
            }
            PackValue::Reference(reference) => {
                w.u32(5);
                reference.write(&mut w);
            }
        }
    }
    w.bytes
}

/// `PackBuffer::UnPackBuffer`: the tagged values, which must end exactly at the data's end; an
/// unknown tag or a short value fails the whole call.
pub fn unpack_values(bytes: &[u8]) -> Result<Vec<PackValue>, ProtocolError> {
    let mut r = Reader::new(bytes);
    let mut values = Vec::new();
    while r.remaining() > 0 {
        let value = match r.u32()? {
            1 => PackValue::Int(r.u32()?),
            2 => PackValue::LongInt(r.u64()?),
            3 => PackValue::Float(r.f32()?),
            4 => PackValue::LongFloat(r.f64()?),
            5 => PackValue::Reference(PackedReference::read(&mut r)?),
            _ => return Err(ProtocolError::Invalid("PackBuffer tag")),
        };
        values.push(value);
    }
    Ok(values)
}

/// One packed `ReferenceEntry` (`ReferenceEntry::Pack`/`UnPack`).
#[derive(Debug, Clone, PartialEq)]
pub enum PackedEntry {
    /// A heap object.
    Heap { flags: u32, object: HeapObject },
    /// A process-relative heap entry packed without its object (`process` set, `root` clear).
    Relative { flags: u32 },
    /// A package singleton by `WState` DID (`VM::GetWStateReference` on unpack).
    Singleton { flags: u32, did: u32 },
    /// A 4-byte value.
    Value { flags: u32, value: u32 },
}

/// `HeapObject` as packed: a package instance (4-byte slots with one tag byte each) or a
/// native object.
#[derive(Debug, Clone, PartialEq)]
pub struct HeapObject {
    pub pkg_state_id: u32,
    /// `m_pkgid`; `u32::MAX` when packed as 0xFFFF.
    pub pkg_id: u32,
    pub native: Option<NativeObject>,
    pub slots: Vec<u32>,
    pub tags: Vec<u8>,
}

/// A packed reference graph.
#[derive(Debug, Clone, PartialEq)]
pub struct PackedReference {
    /// The packed reference ids in pack order (the root first). They are the sender's
    /// reference-table indices: the receiver only maps them to its own.
    pub ids: Vec<u32>,
    pub entries: Vec<PackedEntry>,
}

impl PackedReference {
    /// The reference id the client's stand-in gives a native object it packs; retail uses the
    /// object's index in the VM's reference table, which only the receiver's id map sees.
    pub const NATIVE_ID: u32 = 1;

    /// One loaded native heap object with no package and no slots, as `HandleNewNative`
    /// allocates it (`loaded`, and `recurse` from the dependent-reference walk).
    pub fn native(object: NativeObject) -> Self {
        Self {
            ids: vec![Self::NATIVE_ID],
            entries: vec![PackedEntry::Heap {
                flags: entry_flags::LOADED | entry_flags::RECURSE | entry_flags::KIND_HEAP,
                object: HeapObject {
                    pkg_state_id: 0,
                    pkg_id: u32::MAX,
                    native: Some(object),
                    slots: Vec::new(),
                    tags: Vec::new(),
                },
            }],
        }
    }

    /// The root's native object.
    pub fn root_native(&self) -> Option<&NativeObject> {
        match self.entries.first()? {
            PackedEntry::Heap { object, .. } => object.native.as_ref(),
            _ => None,
        }
    }

    /// `ReferenceTable::UnPackReferenceArray`: `PSmartArray<unsigned long>` (a count, then the
    /// ids), then the entries.
    pub(crate) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let count = r.u32()? as usize;
        if count > MAX_ITEMS || count * 4 > r.remaining() {
            return Err(ProtocolError::Invalid("reference count"));
        }
        let ids = (0..count).map(|_| r.u32()).collect::<Result<Vec<_>, _>>()?;
        let entries = ids
            .iter()
            .map(|_| read_entry(r))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { ids, entries })
    }

    /// `ReferenceTable::PackReferenceArray`.
    pub(crate) fn write(&self, w: &mut Writer) {
        w.u32(self.ids.len() as u32);
        for id in &self.ids {
            w.u32(*id);
        }
        for entry in &self.entries {
            write_entry(entry, w);
        }
    }
}

/// `ReferenceEntry::UnPack`.
fn read_entry(r: &mut Reader<'_>) -> Result<PackedEntry, ProtocolError> {
    use entry_flags::*;
    let flags = r.u32()?;
    Ok(match flags & KIND_MASK {
        KIND_HEAP if flags & SINGLETON != 0 => PackedEntry::Singleton {
            flags,
            did: r.u32()?,
        },
        KIND_HEAP if flags & PROCESS != 0 && flags & ROOT == 0 => PackedEntry::Relative { flags },
        KIND_HEAP => PackedEntry::Heap {
            flags,
            object: read_heap_object(r)?,
        },
        KIND_VALUE => PackedEntry::Value {
            flags,
            value: r.u32()?,
        },
        _ => return Err(ProtocolError::Invalid("reference entry kind")),
    })
}

/// `ReferenceEntry::Pack`: the flag word (low byte cleared), then the entry's data.
fn write_entry(entry: &PackedEntry, w: &mut Writer) {
    match entry {
        PackedEntry::Heap { flags, object } => {
            w.u32(flags & 0xFFFF_FF00);
            write_heap_object(object, w);
        }
        PackedEntry::Relative { flags } => {
            w.u32(flags & 0xFFFF_FF00);
        }
        PackedEntry::Singleton { flags, did: value } | PackedEntry::Value { flags, value } => {
            w.u32(flags & 0xFFFF_FF00).u32(*value);
        }
    }
}

/// `HeapObject::UnPack`: `m_pkgStateID`, `m_pkgid << 16 | m_ntype`, the native object, the
/// slot count, the slots then their tags, alignment. (Retail also rejects a package instance
/// whose slot count is not its package's size; that needs the script library.)
fn read_heap_object(r: &mut Reader<'_>) -> Result<HeapObject, ProtocolError> {
    let pkg_state_id = r.u32()?;
    let word = r.u32()?;
    let pkg_id = match word >> 16 {
        0xFFFF => u32::MAX,
        id => id,
    };
    let native = match word & 0xFFFF {
        0 => None,
        ntype => Some(NativeObject::read(ntype, r)?),
    };
    let size = r.u32()? as usize;
    if size > MAX_ITEMS || size * 5 > r.remaining() {
        return Err(ProtocolError::Invalid("heap object size"));
    }
    let slots = (0..size).map(|_| r.u32()).collect::<Result<Vec<_>, _>>()?;
    let tags = r.take(size)?.to_vec();
    r.align()?;
    Ok(HeapObject {
        pkg_state_id,
        pkg_id,
        native,
        slots,
        tags,
    })
}

/// `HeapObject::Pack`.
fn write_heap_object(object: &HeapObject, w: &mut Writer) {
    let ntype = object.native.as_ref().map_or(0, NativeObject::native_type);
    w.u32(object.pkg_state_id)
        .u32((object.pkg_id << 16) | (ntype & 0xFFFF));
    if let Some(native) = &object.native {
        native.write(w);
    }
    w.u32(object.slots.len() as u32);
    for slot in &object.slots {
        w.u32(*slot);
    }
    w.bytes.extend_from_slice(&object.tags);
    w.align();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::native::SelectionInfo;
    use crate::protocol::string_info::StringInfo;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn scalars_take_their_tags() {
        let values = vec![
            PackValue::Int(7),
            PackValue::LongInt(0x0102_0304_0506_0708),
            PackValue::Float(1.0),
            PackValue::LongFloat(-2.0),
        ];
        let bytes = pack_values(&values);
        let mut expected = words(&[1, 7, 2]);
        expected.extend(0x0102_0304_0506_0708u64.to_le_bytes());
        expected.extend(words(&[3, 0x3F80_0000, 4]));
        expected.extend((-2.0f64).to_le_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(unpack_values(&bytes).unwrap(), values);
        for end in 1..bytes.len() {
            if ![8, 20, 28].contains(&end) {
                assert!(unpack_values(&bytes[..end]).is_err(), "{end}");
            }
        }
        assert!(unpack_values(&words(&[6, 0])).is_err());
        assert_eq!(unpack_values(&[]).unwrap(), vec![]);
    }

    #[test]
    fn a_native_string_reference_packs_as_one_loaded_heap_object() {
        let text = StringInfo {
            literal: Some("hi".into()),
            ..StringInfo::default()
        };
        let value = PackValue::native(NativeObject::StringInfo(text.clone()));
        let bytes = pack_values(&[PackValue::Int(0), value.clone()]);
        let mut expected = words(&[1, 0, 5, 1, 1, 0x4400_0100, 0, 0xFFFF_006A]);
        let mut literal = crate::protocol::reader::Writer::new();
        text.write(&mut literal);
        expected.extend(literal.bytes);
        expected.extend(words(&[0]));
        assert_eq!(bytes, expected);
        let values = unpack_values(&bytes).unwrap();
        assert_eq!(values[1], value);
        assert_eq!(values[1].as_native(), Some(&NativeObject::StringInfo(text)));
    }

    #[test]
    fn heap_objects_keep_slots_tags_and_alignment() {
        let reference = PackedReference {
            ids: vec![3, 4, 5, 6],
            entries: vec![
                PackedEntry::Heap {
                    flags: entry_flags::KIND_HEAP | entry_flags::LOADED,
                    object: HeapObject {
                        pkg_state_id: 2,
                        pkg_id: 0x1AA4,
                        native: None,
                        slots: vec![9, 4],
                        tags: vec![2, 1],
                    },
                },
                PackedEntry::Heap {
                    flags: entry_flags::KIND_HEAP,
                    object: HeapObject {
                        pkg_state_id: 0,
                        pkg_id: u32::MAX,
                        native: Some(NativeObject::SelectionInfo(SelectionInfo::default())),
                        slots: vec![],
                        tags: vec![],
                    },
                },
                PackedEntry::Singleton {
                    flags: entry_flags::KIND_HEAP | entry_flags::SINGLETON,
                    did: 0x7000_0001,
                },
                PackedEntry::Value {
                    flags: entry_flags::KIND_VALUE,
                    value: 42,
                },
            ],
        };
        let bytes = pack_values(&[PackValue::Reference(reference.clone())]);
        let mut expected = words(&[5, 4, 3, 4, 5, 6]);
        expected.extend(words(&[0x4000_0100, 2, 0x1AA4_0000, 2, 9, 4]));
        expected.extend([2, 1, 0, 0]);
        expected.extend(words(&[0x100, 0, 0xFFFF_00AB, 0, 0]));
        expected.extend(words(&[0x2000_0100, 0x7000_0001, 0x200, 42]));
        assert_eq!(bytes, expected);
        assert_eq!(
            unpack_values(&bytes).unwrap(),
            vec![PackValue::Reference(reference)]
        );
        // A relative entry carries nothing; a kind of 0 fails.
        let relative = words(&[5, 1, 0, 0x0800_0100]);
        assert!(matches!(
            &unpack_values(&relative).unwrap()[0],
            PackValue::Reference(r) if matches!(r.entries[0], PackedEntry::Relative { .. })
        ));
        assert!(unpack_values(&words(&[5, 1, 0, 0])).is_err());
        assert!(unpack_values(&words(&[5, 0x4000_0000])).is_err());
    }
}
