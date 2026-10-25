//! The inventory's script objects (Legions WLib 0x56000005): `InvMoveDesc` (package 0x1F8,
//! 51 slots) and `InvEquipDesc` (0x1F6, 19 slots, with an `EquipItemProfile` 0x1F5 in slot 1),
//! the descs the player's scripts send with `WM_Inventory` and the server sends back with the
//! outcome; and the container contents and equipment lists `WM_Inventory` 0x53 / 0x5F carry.
//!
//! The descs start from their package's class-default state (`Interp::HandleNew`), which the
//! package initializers also produce; the constructors below are the script functions that
//! fill them (`InvMoveDesc` 0x3F, `InvEquipDesc` 0x2B, ...). Slot names come from the scripts'
//! own dumps (`InvMoveDesc` 0x2E: "itemID", "fromID", ...). A desc keeps its whole packed graph,
//! so one read from the server packs back unchanged.

use crate::protocol::native::{NativeObject, ScriptHash, ScriptList};
use crate::protocol::pack::{entry_flags, HeapObject, PackedEntry, PackedReference};
use crate::protocol::ProtocolError;

const NULL: u32 = u32::MAX;
const R: u8 = 0x01;
const I: u8 = 0x02;
const L: u8 = 0x82;
const U: u8 = 0xCD;
const M: u32 = u32::MAX;

/// `InvMoveDesc`'s class default, (value, tag) per slot.
#[rustfmt::skip]
const MOVE_DEFAULT: [(u32, u8); 51] = [
    (1, I), (M, I), (0, L), (0, U), (0, I), (0, I), (0, I), (0, I), (0, L), (0, U),
    (1, I), (M, I), (0, I), (1, I), (0, I), (0, I), (0, L), (0, U), (0, L), (0, U),
    (0, I), (0, I), (0, I), (1, I), (M, I), (0, I), (1, I), (0, I), (0, L), (0, U),
    (NULL, R), (0, I), (0, L), (0, U), (0, I), (1, I), (0, L), (0, U), (0, I), (M, I),
    (0, I), (0, I), (0, I), (0, I), (0, L), (0, U), (0, I), (1, I), (0, I), (1, I),
    (M, I),
];

/// `InvEquipDesc`'s class default; slot 1 names its `EquipItemProfile` (id 2 in the graph).
#[rustfmt::skip]
const EQUIP_DEFAULT: [(u32, u8); 19] = [
    (0, I), (2, R), (0, I), (0, L), (0, U), (0, I), (M, I), (0, I), (0, L), (0, U),
    (0, L), (0, U), (158, I), (0, I), (0, I), (0, I), (0, L), (0, U), (M, I),
];

pub const INV_MOVE_DESC_PACKAGE: u32 = 0x1F8;
pub const INV_EQUIP_DESC_PACKAGE: u32 = 0x1F6;
pub const EQUIP_ITEM_PROFILE_PACKAGE: u32 = 0x1F5;

fn instance(pkg_id: u32, layout: &[(u32, u8)]) -> PackedEntry {
    PackedEntry::Heap {
        flags: entry_flags::LOADED | entry_flags::RECURSE | entry_flags::KIND_HEAP,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id,
            native: None,
            slots: layout.iter().map(|s| s.0).collect(),
            tags: layout.iter().map(|s| s.1).collect(),
        },
    }
}

/// A package-instance graph whose root is package `pkg_id` with `size` slots.
#[derive(Debug, Clone, PartialEq)]
struct Instance(PackedReference);

impl Instance {
    fn check(reference: &PackedReference, pkg_id: u32, size: usize) -> Result<Self, ProtocolError> {
        match reference.entries.first() {
            Some(PackedEntry::Heap { object, .. })
                if object.pkg_id == pkg_id
                    && object.slots.len() == size
                    && object.native.is_none() =>
            {
                Ok(Self(reference.clone()))
            }
            _ => Err(ProtocolError::Invalid("inventory desc")),
        }
    }

    fn root(&self) -> &HeapObject {
        match &self.0.entries[0] {
            PackedEntry::Heap { object, .. } => object,
            _ => unreachable!("checked"),
        }
    }

    fn root_mut(&mut self) -> &mut HeapObject {
        match &mut self.0.entries[0] {
            PackedEntry::Heap { object, .. } => object,
            _ => unreachable!("checked"),
        }
    }

    fn word(&self, slot: usize) -> u32 {
        self.root().slots[slot]
    }

    fn long(&self, slot: usize) -> u64 {
        let s = &self.root().slots;
        u64::from(s[slot]) | u64::from(s[slot + 1]) << 32
    }

    /// `rstore`: the value and the word's tag.
    fn set_word(&mut self, slot: usize, value: u32) {
        let root = self.root_mut();
        root.slots[slot] = value;
        root.tags[slot] = I;
    }

    /// `rstore_L`: both words, the first word's tag only (the second keeps its own).
    fn set_long(&mut self, slot: usize, value: u64) {
        let root = self.root_mut();
        root.slots[slot] = value as u32;
        root.slots[slot + 1] = (value >> 32) as u32;
        root.tags[slot] = L;
    }
}

/// One `InvMoveDesc`: a move of an item between containers, to the world (a drop), to another
/// creature (a give), or of part of a stack (a split, `quantity` below the stack's).
#[derive(Debug, Clone, PartialEq)]
pub struct InvMoveDesc(Instance);

impl Default for InvMoveDesc {
    fn default() -> Self {
        Self(Instance(PackedReference {
            ids: vec![1],
            entries: vec![instance(INV_MOVE_DESC_PACKAGE, &MOVE_DEFAULT)],
        }))
    }
}

impl InvMoveDesc {
    /// `gmCWInterface::MoveItem` -> `CContainer` 0x7 (`MoveItem`) -> `InvMoveDesc` 0x3F: move
    /// `quantity` of `item` from slot `from_slot` of `from` to slot `target_slot` of `target`
    /// (move type 0).
    pub fn new(
        item: u64,
        from: u64,
        from_slot: i32,
        target: u64,
        target_slot: i32,
        quantity: i32,
    ) -> Self {
        let mut d = Self::default();
        let i = &mut d.0;
        i.set_long(44, item);
        i.set_long(16, from);
        i.set_word(39, from_slot as u32);
        i.set_long(2, target);
        i.set_word(11, target_slot as u32);
        i.set_word(26, quantity as u32);
        i.set_word(12, 0);
        i.set_long(36, from);
        i.set_word(1, from_slot as u32);
        i.set_long(8, target);
        i.set_word(24, target_slot as u32);
        i.set_long(32, target);
        i.set_word(50, target_slot as u32);
        i.set_long(28, item);
        i.set_word(25, quantity as u32);
        d
    }

    pub fn from_reference(reference: &PackedReference) -> Result<Self, ProtocolError> {
        Instance::check(reference, INV_MOVE_DESC_PACKAGE, MOVE_DEFAULT.len()).map(Self)
    }

    pub fn to_reference(&self) -> PackedReference {
        self.0 .0.clone()
    }

    /// Slot 44, "itemID" (`InvMoveDesc` 0x42).
    pub fn item(&self) -> u64 {
        self.0.long(44)
    }
    /// Slot 16, "fromID" (0x45).
    pub fn from(&self) -> u64 {
        self.0.long(16)
    }
    /// Slot 39, "fromSlot" (0x44).
    pub fn from_slot(&self) -> i32 {
        self.0.word(39) as i32
    }
    /// Slot 2, "targetID" (0x2F).
    pub fn target(&self) -> u64 {
        self.0.long(2)
    }
    /// Slot 11, "targetSlot" (0x2B).
    pub fn target_slot(&self) -> i32 {
        self.0.word(11) as i32
    }
    /// Slot 26, "quantity".
    pub fn quantity(&self) -> i32 {
        self.0.word(26) as i32
    }
    /// Slot 12, "moveType" (0x33 / 0x1F).
    pub fn move_type(&self) -> u32 {
        self.0.word(12)
    }
    /// Slot 1, "actualFromSlot".
    pub fn actual_from_slot(&self) -> i32 {
        self.0.word(1) as i32
    }
    /// Slot 24, "actualTargetSlot".
    pub fn actual_target_slot(&self) -> i32 {
        self.0.word(24) as i32
    }
    /// Slot 28, "splitItemID".
    pub fn split_item(&self) -> u64 {
        self.0.long(28)
    }
    /// Slot 32, "mergeContainerID".
    pub fn merge_container(&self) -> u64 {
        self.0.long(32)
    }
    /// Slot 40, "status" (`InvMoveDesc` 0xA; 0 is success).
    pub fn status(&self) -> u32 {
        self.0.word(40)
    }
    /// Sets "status" (`InvMoveDesc` 0xB): what a server answers with.
    pub fn set_status(&mut self, status: u32) {
        self.0.set_word(40, status);
    }
    /// Sets "actualTargetSlot": where the server put the item.
    pub fn set_actual_target_slot(&mut self, slot: i32) {
        self.0.set_word(24, slot as u32);
    }
}

/// One `InvEquipDesc`: an item wielded at, or taken off from, an equipment location.
#[derive(Debug, Clone, PartialEq)]
pub struct InvEquipDesc(Instance);

impl Default for InvEquipDesc {
    fn default() -> Self {
        Self(Instance(PackedReference {
            ids: vec![1, 2],
            entries: vec![
                instance(INV_EQUIP_DESC_PACKAGE, &EQUIP_DEFAULT),
                instance(EQUIP_ITEM_PROFILE_PACKAGE, &[(0, I); 7]),
            ],
        }))
    }
}

impl InvEquipDesc {
    /// `InvEquipDesc` 0x2B: `user` equips `item` at `location`.
    fn base(item: u64, user: u64, location: u32) -> Self {
        let mut d = Self::default();
        let i = &mut d.0;
        i.set_long(3, item);
        i.set_long(8, user);
        i.set_word(7, location);
        i.set_word(15, location);
        i.set_word(2, 1);
        i.set_long(10, user);
        i.set_word(18, 0);
        i.set_word(6, 0);
        d
    }

    /// `gmCWInterface::EquipItem` -> `Inventory` 0x21 -> `CInventorySystem` 0x16
    /// (`DirectiveEquipItem`): `item`, now in slot `from_slot` of `from`, to `location`
    /// (`InvEquipDesc` 0x6 and 0x1F record where it came from).
    pub fn equip(item: u64, user: u64, location: u32, from: u64, from_slot: i32) -> Self {
        let mut d = Self::base(item, user, location);
        d.0.set_long(10, from);
        d.0.set_word(18, from_slot as u32);
        d
    }

    /// `gmCWInterface::UnEquipItem` -> `Inventory` 0x9: take `item` off `location` into slot
    /// `target_slot` of the user's pack (`InvEquipDesc` 0x30, 0x1F, 0x6).
    pub fn unequip(item: u64, user: u64, location: u32, target_slot: i32) -> Self {
        let mut d = Self::base(item, user, location);
        d.0.set_word(6, target_slot as u32);
        d.0.set_word(18, target_slot as u32);
        d.0.set_long(10, user);
        d
    }

    pub fn from_reference(reference: &PackedReference) -> Result<Self, ProtocolError> {
        Instance::check(reference, INV_EQUIP_DESC_PACKAGE, EQUIP_DEFAULT.len()).map(Self)
    }

    pub fn to_reference(&self) -> PackedReference {
        self.0 .0.clone()
    }

    /// Slot 3: the item (`InvEquipDesc` 0x2E).
    pub fn item(&self) -> u64 {
        self.0.long(3)
    }
    /// Slot 7: the equipment location (`InvEquipDesc` 0x22).
    pub fn location(&self) -> u32 {
        self.0.word(7)
    }
    /// Slots 10-11: the container the item comes from or goes to.
    pub fn container(&self) -> u64 {
        self.0.long(10)
    }
    /// Slot 18: the slot in that container.
    pub fn slot(&self) -> i32 {
        self.0.word(18) as i32
    }
    /// Slot 0: the outcome (0 is success).
    pub fn status(&self) -> u32 {
        self.0.word(0)
    }
    pub fn set_status(&mut self, status: u32) {
        self.0.set_word(0, status);
    }
}

/// A container's contents as `WM_Inventory` 0x53 carries them (`CInventorySystem` 0x1A ->
/// `Player` 0x2D into `Container` slots 18, 19, 15): the segments (an `RList` of segment
/// instances, kept as packed), the packs it holds (`LList` of ids) and its items (`LList`).
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerContents {
    pub segments: PackedReference,
    pub packs: Vec<u64>,
    pub items: Vec<u64>,
}

/// The equipment as `WM_Inventory` 0x5F carries it (`CInventorySystem` 0x6 -> `Player` 0x73
/// into `Inventory` slots 22, 21, 23): a word, location -> `InventProfile` (`ARHash`) and item
/// -> `InventProfile` (`LRHash`), kept as packed; [`Equipment::by_location`] reads them.
#[derive(Debug, Clone, PartialEq)]
pub struct Equipment {
    pub first: crate::protocol::PackValue,
    pub by_location: PackedReference,
    pub by_item: PackedReference,
}

/// A packed `LList` of ids.
pub fn id_list(reference: &PackedReference) -> Result<Vec<u64>, ProtocolError> {
    match reference.root_native() {
        Some(NativeObject::List(ScriptList { items, .. })) => Ok(items.clone()),
        _ => Err(ProtocolError::Invalid("id list")),
    }
}

/// A packed `LList` of `ids`, as a client or server packs one.
pub fn pack_id_list(ids: &[u64]) -> PackedReference {
    PackedReference::native(NativeObject::List(ScriptList {
        native_type: crate::protocol::ntype::L_LIST,
        items: ids.to_vec(),
    }))
}

/// `InventProfile` (package 0x1F9, six slots: a reference, three words, the item).
pub const INVENT_PROFILE_PACKAGE: u32 = 0x1F9;

impl Equipment {
    /// The server's side: each location's item as the two tables `WM_Inventory` 0x5F packs,
    /// one `InventProfile` per item (its other slots at their class defaults), and a 0 word.
    pub fn from_locations(locations: &[(u32, u64)]) -> Self {
        let profiles: Vec<PackedEntry> = locations
            .iter()
            .map(|(_, item)| {
                instance(
                    INVENT_PROFILE_PACKAGE,
                    &[
                        (NULL, R),
                        (0, I),
                        (0, I),
                        (0, I),
                        (*item as u32, L),
                        ((item >> 32) as u32, U),
                    ],
                )
            })
            .collect();
        let table = |native_type: u32, keys: Vec<crate::protocol::HashKey>| {
            let mut graph = PackedReference::native(NativeObject::Hash(ScriptHash {
                native_type,
                bucket_index: 0,
                keys,
                values: (2..2 + locations.len() as u64).collect(),
            }));
            graph.ids.extend(2..2 + locations.len() as u32);
            graph.entries.extend(profiles.iter().cloned());
            graph
        };
        let value = |v: u64| crate::protocol::HashKey::Value(v);
        Self {
            first: crate::protocol::PackValue::Int(0),
            by_location: table(
                crate::protocol::ntype::AR_HASH,
                locations
                    .iter()
                    .map(|(l, _)| value(u64::from(*l)))
                    .collect(),
            ),
            by_item: table(
                crate::protocol::ntype::LR_HASH,
                locations.iter().map(|(_, i)| value(*i)).collect(),
            ),
        }
    }

    /// `Inventory` 0xE (`InqSimpleInvByLocTable`): each location's item, the `InventProfile`'s
    /// slots 4-5 (`InventProfile` 0x1), in table order.
    pub fn locations(&self) -> Result<Vec<(u32, u64)>, ProtocolError> {
        let invalid = ProtocolError::Invalid("equipment table");
        let hash = match self.by_location.root_native() {
            Some(NativeObject::Hash(ScriptHash { keys, values, .. })) => (keys, values),
            _ => return Err(invalid),
        };
        let mut out = Vec::new();
        for (key, value) in hash.0.iter().zip(hash.1) {
            let crate::protocol::HashKey::Value(location) = key else {
                return Err(invalid);
            };
            let at = self
                .by_location
                .ids
                .iter()
                .position(|&id| u64::from(id) == *value)
                .ok_or(ProtocolError::Invalid("equipment profile"))?;
            let item = match &self.by_location.entries[at] {
                PackedEntry::Heap { object, .. } if object.slots.len() >= 6 => {
                    u64::from(object.slots[4]) | u64::from(object.slots[5]) << 32
                }
                _ => return Err(ProtocolError::Invalid("equipment profile")),
            };
            out.push((*location as u32, item));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::pack::{pack_values, unpack_values, PackValue};

    #[test]
    fn a_move_fills_the_constructors_slots_and_round_trips() {
        let d = InvMoveDesc::new(0x42, 0x5000_0000_0000_0001, 3, 0x43, -1, 2);
        let r = d.to_reference();
        let PackedEntry::Heap { object, .. } = &r.entries[0] else {
            panic!()
        };
        assert_eq!((object.pkg_id, object.slots.len()), (0x1F8, 51));
        assert_eq!(
            (
                d.item(),
                d.from(),
                d.from_slot(),
                d.target(),
                d.target_slot()
            ),
            (0x42, 0x5000_0000_0000_0001, 3, 0x43, -1)
        );
        assert_eq!(
            (d.quantity(), d.split_item(), d.merge_container()),
            (2, 0x42, 0x43)
        );
        assert_eq!(&object.slots[36..38], &[1, 0x5000_0000]);
        assert_eq!(&object.tags[36..38], &[0x82, 0xCD]);
        let bytes = pack_values(&[PackValue::Reference(r)]);
        // Tag 5, one id, the instance's flags, state, package and size, then its slots.
        assert_eq!(bytes.len(), 4 * 7 + 51 * 4 + 52);
        let PackValue::Reference(back) = &unpack_values(&bytes).unwrap()[0] else {
            panic!()
        };
        assert_eq!(InvMoveDesc::from_reference(back).unwrap(), d);
        assert!(InvEquipDesc::from_reference(back).is_err());
    }

    #[test]
    fn equip_descs_carry_their_profile_and_origin() {
        let d = InvEquipDesc::equip(0x42, 1, 0x10, 1, 4);
        assert_eq!(
            (d.item(), d.location(), d.container(), d.slot()),
            (0x42, 0x10, 1, 4)
        );
        let r = d.to_reference();
        assert_eq!(r.ids, [1, 2]);
        let PackedEntry::Heap { object, .. } = &r.entries[1] else {
            panic!()
        };
        assert_eq!((object.pkg_id, object.slots.len()), (0x1F5, 7));
        let u = InvEquipDesc::unequip(0x42, 1, 0x10, 2);
        assert_eq!((u.container(), u.slot()), (1, 2));
        assert_eq!(InvEquipDesc::from_reference(&u.to_reference()).unwrap(), u);
    }

    #[test]
    fn equipment_reads_each_locations_profile() {
        let profile = |item: u64| PackedEntry::Heap {
            flags: entry_flags::KIND_HEAP | entry_flags::LOADED,
            object: HeapObject {
                pkg_state_id: 0,
                pkg_id: 0x1F9,
                native: None,
                slots: vec![NULL, 0, 0, 0, item as u32, (item >> 32) as u32],
                tags: vec![1, 2, 2, 2, 0x82, 0xCD],
            },
        };
        let mut by_location = PackedReference::native(NativeObject::Hash(ScriptHash {
            native_type: crate::protocol::ntype::AR_HASH,
            bucket_index: 0,
            keys: vec![crate::protocol::HashKey::Value(0x10)],
            values: vec![2],
        }));
        by_location.ids.push(2);
        by_location.entries.push(profile(0x8000_0002_0000_0042));
        let equipment = Equipment {
            first: PackValue::Int(0),
            by_location,
            by_item: pack_id_list(&[]),
        };
        assert_eq!(
            equipment.locations().unwrap(),
            vec![(0x10, 0x8000_0002_0000_0042)]
        );
        assert_eq!(id_list(&pack_id_list(&[5, 6])).unwrap(), vec![5, 6]);
        let built = Equipment::from_locations(&[(0x10, 7), (0x20, 8)]);
        assert_eq!(built.locations().unwrap(), vec![(0x10, 7), (0x20, 8)]);
    }
}
