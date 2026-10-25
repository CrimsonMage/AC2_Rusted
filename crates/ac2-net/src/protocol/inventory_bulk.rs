//! Taking or transmuting a container's whole contents: `InvTakeAllDesc` (package 0x20EF, 14
//! slots) and `InvTransmuteAllDesc` (0x234B, 15 slots), as `CPlayer` 0x157
//! (`LootAllFromContainer`) / 0x158 (`TransmuteAllFromContainer`) build them and
//! `CInventorySystem` 0x1C / 0x21 send them (`WM_Inventory` 0x69 / 0x6E); the server sends them
//! back with the outcome (`WM_Inventory` 0x6A / 0x6D -> `CInventorySystem` 0x1D / 0x20).

use super::inventory::{instance, Instance, I, L, NULL, R, U};
use crate::protocol::pack::PackedReference;
use crate::protocol::ProtocolError;

pub const INV_TAKE_ALL_DESC_PACKAGE: u32 = 0x20EF;
pub const INV_TRANSMUTE_ALL_DESC_PACKAGE: u32 = 0x234B;

#[rustfmt::skip]
const TAKE_ALL_DEFAULT: [(u32, u8); 14] = [
    (0, I), (0, I), (1, I), (0, I), (0, I), (0, I), (0, L), (0, U), (NULL, R), (0, I),
    (0, I), (NULL, R), (0, L), (0, U),
];

#[rustfmt::skip]
const TRANSMUTE_ALL_DEFAULT: [(u32, u8); 15] = [
    (0, I), (0, I), (NULL, R), (1, I), (0, I), (0, I), (0, I), (0, I), (0, L), (0, U),
    (NULL, R), (0, I), (0, I), (0, L), (0, U),
];

/// One `InvTakeAllDesc`: everything in a container into the player's pack.
#[derive(Debug, Clone, PartialEq)]
pub struct InvTakeAllDesc(Instance);

impl InvTakeAllDesc {
    /// `InvTakeAllDesc` 0x17 (container, player, 0); status 0 as `CInventorySystem` 0x1C
    /// records when its checks pass (`InvTakeAllDesc` 0x4).
    pub fn new(container: u64, player: u64) -> Self {
        let mut i = Instance(PackedReference {
            ids: vec![1],
            entries: vec![instance(INV_TAKE_ALL_DESC_PACKAGE, &TAKE_ALL_DEFAULT)],
        });
        i.set_long(6, container);
        i.set_long(12, player);
        i.set_word(3, 0);
        i.set_word(5, 0);
        Self(i)
    }

    pub fn from_reference(reference: &PackedReference) -> Result<Self, ProtocolError> {
        Instance::check(reference, INV_TAKE_ALL_DESC_PACKAGE, TAKE_ALL_DEFAULT.len()).map(Self)
    }

    pub fn to_reference(&self) -> PackedReference {
        self.0 .0.clone()
    }

    /// Slots 6-7 (`InvTakeAllDesc` 0x12).
    pub fn container(&self) -> u64 {
        self.0.long(6)
    }
    /// Slots 12-13 (0x2).
    pub fn player(&self) -> u64 {
        self.0.long(12)
    }
    /// Slot 5 (0x4 sets it).
    pub fn status(&self) -> u32 {
        self.0.word(5)
    }
    pub fn set_status(&mut self, status: u32) {
        self.0.set_word(5, status);
    }
}

/// One `InvTransmuteAllDesc`: a container's contents transmuted.
#[derive(Debug, Clone, PartialEq)]
pub struct InvTransmuteAllDesc(Instance);

impl InvTransmuteAllDesc {
    /// `InvTransmuteAllDesc` 0x19 (container, player, 0); status 0 (`InvTransmuteAllDesc` 0x5).
    pub fn new(container: u64, player: u64) -> Self {
        let mut i = Instance(PackedReference {
            ids: vec![1],
            entries: vec![instance(
                INV_TRANSMUTE_ALL_DESC_PACKAGE,
                &TRANSMUTE_ALL_DEFAULT,
            )],
        });
        i.set_long(8, container);
        i.set_long(13, player);
        i.set_word(5, 0);
        i.set_word(7, 0);
        Self(i)
    }

    pub fn from_reference(reference: &PackedReference) -> Result<Self, ProtocolError> {
        Instance::check(
            reference,
            INV_TRANSMUTE_ALL_DESC_PACKAGE,
            TRANSMUTE_ALL_DEFAULT.len(),
        )
        .map(Self)
    }

    pub fn to_reference(&self) -> PackedReference {
        self.0 .0.clone()
    }

    /// Slots 8-9 (`InvTransmuteAllDesc` 0x14).
    pub fn container(&self) -> u64 {
        self.0.long(8)
    }
    /// Slots 13-14 (0x3).
    pub fn player(&self) -> u64 {
        self.0.long(13)
    }
    /// Slot 7 (0x5 sets it).
    pub fn status(&self) -> u32 {
        self.0.word(7)
    }
    pub fn set_status(&mut self, status: u32) {
        self.0.set_word(7, status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::pack::{pack_values, PackValue, PackedEntry};

    #[test]
    fn bulk_descs_name_the_container_and_player() {
        let take = InvTakeAllDesc::new(0x100, 1);
        assert_eq!(
            (take.container(), take.player(), take.status()),
            (0x100, 1, 0)
        );
        let r = take.to_reference();
        let PackedEntry::Heap { object, .. } = &r.entries[0] else {
            panic!()
        };
        assert_eq!(&object.slots[6..8], &[0x100, 0]);
        assert_eq!(InvTakeAllDesc::from_reference(&r).unwrap(), take);
        let bytes = pack_values(&[PackValue::Reference(r)]);
        assert_eq!(bytes.len(), 4 * 7 + 14 * 4 + 16);
        let t = InvTransmuteAllDesc::new(0x100, 1);
        assert_eq!((t.container(), t.player()), (0x100, 1));
        assert!(InvTakeAllDesc::from_reference(&t.to_reference()).is_err());
    }
}
