//! `UsageDesc`, the script object a use request carries (package 0x245 in the Legions script
//! library, twenty slots): `CUsageSystem` 0x9 (`HandleItemUsage`) and 0x3
//! (`TryToAddToUsageList`) allocate one (`Interp::HandleNew`, from the package's class-default
//! WState), fill it with `UsageDesc` 0x37 and send it with `WM_Usage` 0x22; the server sends it
//! back with its outcome (`WM_Usage` 0x21 -> `CUsageSystem` 0xA).
//!
//! It packs as a two-entry reference graph (`ReferenceTable::GetDependentReferences`, root
//! first, then the reference slots in slot order): the instance, then the native `StringInfo`
//! slot 0 points at. A 64-bit value takes two slots; `rstore_L` writes only the first slot's
//! tag, so the second keeps the class default's `0xCD`.

use crate::protocol::native::NativeObject;
use crate::protocol::pack::{entry_flags, HeapObject, PackedEntry, PackedReference};
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

/// `UsageDesc`'s package id.
pub const USAGE_DESC_PACKAGE: u32 = 0x245;
/// Its slot count.
const SLOTS: usize = 20;
/// Slot tags: a reference, an int, a float, a 64-bit int, and the class default's
/// uninitialised tag (the second word of each 64-bit slot pair).
const TAG_REF: u8 = 0x01;
const TAG_INT: u8 = 0x02;
const TAG_FLOAT: u8 = 0x03;
const TAG_LONG: u8 = 0x82;
const TAG_UNSET: u8 = 0xCD;
/// A null reference slot.
const NULL_REF: u32 = u32::MAX;
/// The ids this client gives the graph's two entries (retail uses its reference-table
/// indices; the receiver maps them to its own).
const ROOT_ID: u32 = 1;
const MESSAGE_ID: u32 = 2;

/// Slot layout: (slot, tag) of each value, in slot order.
const LAYOUT: [u8; SLOTS] = [
    TAG_REF, TAG_REF, TAG_LONG, TAG_UNSET, TAG_REF, TAG_INT, TAG_LONG, TAG_UNSET, TAG_FLOAT,
    TAG_LONG, TAG_UNSET, TAG_INT, TAG_LONG, TAG_UNSET, TAG_INT, TAG_REF, TAG_INT, TAG_INT, TAG_INT,
    TAG_INT,
];

/// One `UsageDesc`. The slots the client neither sets nor reads keep their class-default
/// values; the reference slots 1, 4 and 15 stay null in what the client packs (and are not
/// kept when one is read).
#[derive(Debug, Clone, PartialEq)]
pub struct UsageDesc {
    /// Slot 0 (`UsageDesc` 0x2A): the message the outcome shows (`CUsageSystem` 0xA).
    pub message: StringInfo,
    /// Slots 2-3 (`UsageDesc` 0x3D): the item used.
    pub item: u64,
    /// Slots 6-7 (`UsageDesc` 0x16): the user (`CWRepository::GetID` of the player).
    pub user: u64,
    /// Slots 9-10 (`UsageDesc` 0x2E): the target, 0 for none (`UsageDesc` 0x2 tests it).
    pub target: u64,
    /// Slot 11 (`UsageDesc` 0xC / 0xD): the outcome's error, 0 for success.
    pub error: u32,
    /// Slot 17 (`UsageDesc` 0x12 tests a bit; 0x1B: bit 0x80, show no message).
    pub flags: u32,
    /// Slot 5.
    pub slot_5: u32,
    /// Slot 8.
    pub slot_8: f32,
    /// Slots 12-13.
    pub slot_12: u64,
    /// Slots 14, 16, 18 and 19.
    pub slot_14: u32,
    pub slot_16: u32,
    pub slot_18: u32,
    pub slot_19: u32,
}

impl Default for UsageDesc {
    /// The class-default state `Interp::HandleNew` starts from (`UsageDesc` 0x38 sets the same
    /// values): message 0x4E1AEC40 of string table 0x25000014, slot 8 -1.0, flags 0x13C
    /// (`UsageDesc` 0xF).
    fn default() -> Self {
        Self {
            message: StringInfo {
                string_id: 0x4E1A_EC40,
                table_id: 0x2500_0014,
                ..StringInfo::default()
            },
            item: 0,
            user: 0,
            target: 0,
            error: 0,
            flags: 0x13C,
            slot_5: 0,
            slot_8: -1.0,
            slot_12: 0,
            slot_14: 0,
            slot_16: 0,
            slot_18: 0,
            slot_19: 0,
        }
    }
}

/// The flag bit that keeps the outcome's message from showing (`UsageDesc` 0x1B).
pub const USAGE_FLAG_SILENT: u32 = 0x80;

impl UsageDesc {
    /// `UsageDesc` 0x37: `user` uses `item`, on `target` (0 for none).
    pub fn new(user: u64, item: u64, target: u64) -> Self {
        Self {
            item,
            user,
            target,
            ..Self::default()
        }
    }

    fn slots(&self) -> [u32; SLOTS] {
        let lo = |v: u64| v as u32;
        let hi = |v: u64| (v >> 32) as u32;
        [
            MESSAGE_ID,
            NULL_REF,
            lo(self.item),
            hi(self.item),
            NULL_REF,
            self.slot_5,
            lo(self.user),
            hi(self.user),
            self.slot_8.to_bits(),
            lo(self.target),
            hi(self.target),
            self.error,
            lo(self.slot_12),
            hi(self.slot_12),
            self.slot_14,
            NULL_REF,
            self.slot_16,
            self.flags,
            self.slot_18,
            self.slot_19,
        ]
    }

    /// `ReferenceTable::PackReference` of the instance: the instance, then its message.
    pub fn to_reference(&self) -> PackedReference {
        let flags = entry_flags::LOADED | entry_flags::RECURSE | entry_flags::KIND_HEAP;
        PackedReference {
            ids: vec![ROOT_ID, MESSAGE_ID],
            entries: vec![
                PackedEntry::Heap {
                    flags,
                    object: HeapObject {
                        pkg_state_id: 0,
                        pkg_id: USAGE_DESC_PACKAGE,
                        native: None,
                        slots: self.slots().to_vec(),
                        tags: LAYOUT.to_vec(),
                    },
                },
                PackedEntry::Heap {
                    flags,
                    object: HeapObject {
                        pkg_state_id: 0,
                        pkg_id: u32::MAX,
                        native: Some(NativeObject::StringInfo(self.message.clone())),
                        slots: Vec::new(),
                        tags: Vec::new(),
                    },
                },
            ],
        }
    }

    /// The instance a packed graph's root holds; its message is the native `StringInfo`
    /// slot 0 names (the default message when the slot is null).
    pub fn from_reference(reference: &PackedReference) -> Result<Self, ProtocolError> {
        let invalid = ProtocolError::Invalid("UsageDesc");
        let root = match reference.entries.first() {
            Some(PackedEntry::Heap { object, .. }) => object,
            _ => return Err(invalid),
        };
        if root.pkg_id != USAGE_DESC_PACKAGE || root.slots.len() != SLOTS || root.native.is_some() {
            return Err(invalid);
        }
        let s = &root.slots;
        let long = |at: usize| u64::from(s[at]) | u64::from(s[at + 1]) << 32;
        let message = match s[0] {
            NULL_REF => Self::default().message,
            id => {
                let at = reference.ids.iter().position(|&i| i == id).ok_or(invalid)?;
                match reference.entries.get(at) {
                    Some(PackedEntry::Heap { object, .. }) => match &object.native {
                        Some(NativeObject::StringInfo(text)) => text.clone(),
                        _ => return Err(ProtocolError::Invalid("UsageDesc message")),
                    },
                    _ => return Err(ProtocolError::Invalid("UsageDesc message")),
                }
            }
        };
        Ok(Self {
            message,
            item: long(2),
            user: long(6),
            target: long(9),
            error: s[11],
            flags: s[17],
            slot_5: s[5],
            slot_8: f32::from_bits(s[8]),
            slot_12: long(12),
            slot_14: s[14],
            slot_16: s[16],
            slot_18: s[18],
            slot_19: s[19],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::pack::{pack_values, unpack_values, PackValue};
    use crate::protocol::reader::Writer;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn a_use_packs_the_instance_then_its_message() {
        let desc = UsageDesc::new(0x5000_0000_0000_0001, 0x8000_0002_0000_0042, 0);
        let bytes = pack_values(&[PackValue::Reference(desc.to_reference())]);
        // Tag 5, two ids, the instance (state 0, package 0x245, twenty slots and tags).
        let mut expected = words(&[5, 2, 1, 2, 0x4400_0100, 0, 0x0245_0000, 20]);
        expected.extend(words(&[
            2,
            u32::MAX,
            0x42,
            0x8000_0002,
            u32::MAX,
            0,
            1,
            0x5000_0000,
            0xBF80_0000,
            0,
            0,
            0,
            0,
            0,
            0,
            u32::MAX,
            0,
            0x13C,
            0,
            0,
        ]));
        expected.extend([
            1, 1, 0x82, 0xCD, 1, 2, 0x82, 0xCD, 3, 0x82, 0xCD, 2, 0x82, 0xCD, 2, 1, 2, 2, 2, 2,
        ]);
        // Then the message: a loaded native StringInfo (type 0x6A) and no slots.
        expected.extend(words(&[0x4400_0100, 0, 0xFFFF_006A]));
        let mut text = Writer::new();
        desc.message.write(&mut text);
        expected.extend(text.bytes);
        expected.extend(words(&[0]));
        assert_eq!(bytes, expected);
        match &unpack_values(&bytes).unwrap()[0] {
            PackValue::Reference(r) => assert_eq!(UsageDesc::from_reference(r).unwrap(), desc),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn other_packages_and_shapes_are_refused() {
        let desc = UsageDesc::new(1, 2, 3);
        let mut reference = desc.to_reference();
        if let PackedEntry::Heap { object, .. } = &mut reference.entries[0] {
            object.pkg_id = 0x246;
        }
        assert!(UsageDesc::from_reference(&reference).is_err());
        let mut reference = desc.to_reference();
        reference.ids[1] = 9;
        assert!(UsageDesc::from_reference(&reference).is_err());
        // A null message slot reads as the class default's message.
        let mut reference = desc.to_reference();
        if let PackedEntry::Heap { object, .. } = &mut reference.entries[0] {
            object.slots[0] = u32::MAX;
        }
        reference.entries.truncate(1);
        reference.ids.truncate(1);
        assert_eq!(UsageDesc::from_reference(&reference).unwrap(), desc);
    }
}
