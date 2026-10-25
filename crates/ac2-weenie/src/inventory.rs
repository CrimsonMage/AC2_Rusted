//! A native stand-in for the inventory the player's scripts keep (`InventorySystem`,
//! `CInventorySystem`, `Container` and `Inventory` in the Legions script library): each
//! container's packs and items as the server last listed them (`WM_Inventory` 0x53 ->
//! `Container` slots 19 and 15), the equipment by location (`WM_Inventory` 0x5F -> `Inventory`
//! slot 21), and the server's answers to moves and equips applied to them
//! (`CInventorySystem::MoveItem_Done`, `EquipItem_Done`, `UnEquipItem_Done`), with the
//! `CM_Inventory` notices the scripts raise for the UI.
//!
//! Not mirrored: the scripts' own checks and optimistic updates before a request goes out
//! (`CInventorySystem` 0x4, 0xB, 0x15), stack merging and splitting into new items, burden, the
//! container segments (kept packed, unread), and the inventory profiles beyond each location's
//! item. Retired once the script VM runs these packages.

use std::collections::BTreeMap;

use ac2_net::protocol::{ContainerContents, Equipment, InvEquipDesc, InvMoveDesc, InvTakeAllDesc};

use crate::player_script::GameplayNotice;

/// `CM_Inventory` notice code for a whole refresh (`CInventorySystem` 0x1A, 0x6).
pub const INVENTORY_REFRESH: u32 = 0x4000_0001;

/// One container's contents, in slot order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contents {
    /// The containers it holds (side packs), `Container` slot 19.
    pub packs: Vec<u64>,
    /// Its items, `Container` slot 15.
    pub items: Vec<u64>,
}

/// The inventory as the stand-in knows it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InventoryState {
    contents: BTreeMap<u64, Contents>,
    /// Location and item, in the server's table order (`InqSimpleInvByLocTable`).
    equipped: Vec<(u32, u64)>,
    /// The containers open on the player (`CContainer::Open_Done` .. `Close_Done`).
    open: Vec<u64>,
}

fn refresh_container(container: u64) -> GameplayNotice {
    GameplayNotice::UpdateContainer {
        container,
        code: INVENTORY_REFRESH,
        item: 0,
        slot: 0,
    }
}

fn refresh_inventory() -> GameplayNotice {
    GameplayNotice::UpdateInventory {
        code: INVENTORY_REFRESH,
        item: 0,
        location: 0,
    }
}

impl InventoryState {
    /// The listed containers and their contents.
    pub fn containers(&self) -> impl Iterator<Item = (u64, &Contents)> {
        self.contents.iter().map(|(id, c)| (*id, c))
    }

    pub fn contents(&self, container: u64) -> Option<&Contents> {
        self.contents.get(&container)
    }

    pub fn equipped(&self) -> &[(u32, u64)] {
        &self.equipped
    }

    /// `InventorySystem` 0x20: where `item` is equipped.
    pub fn location_of(&self, item: u64) -> Option<u32> {
        self.equipped
            .iter()
            .find(|(_, i)| *i == item)
            .map(|(loc, _)| *loc)
    }

    /// `InventorySystem` 0x23 (`LocateItemInContents`): the container and slot holding `item`.
    pub fn locate(&self, item: u64) -> Option<(u64, i32)> {
        self.contents.iter().find_map(|(id, c)| {
            c.items
                .iter()
                .chain(&c.packs)
                .position(|i| *i == item)
                .map(|slot| (*id, slot as i32))
        })
    }

    /// `CInventorySystem` 0x1A: `container`'s contents replaced, then
    /// `SendNotice_UpdateContainer(container, 0x40000001, 0, 0)`.
    pub fn set_contents(&mut self, container: u64, c: ContainerContents) -> GameplayNotice {
        self.contents.insert(
            container,
            Contents {
                packs: c.packs,
                items: c.items,
            },
        );
        refresh_container(container)
    }

    /// `CInventorySystem` 0x6: the equipment replaced, then
    /// `SendNotice_UpdateInventory(0x40000001, 0, 0)`. A table that does not read keeps the
    /// old equipment.
    pub fn set_equipment(&mut self, e: &Equipment) -> GameplayNotice {
        if let Ok(locations) = e.locations() {
            self.equipped = locations;
        }
        refresh_inventory()
    }

    /// The containers open on the player, in the order they opened.
    pub fn open_containers(&self) -> &[u64] {
        &self.open
    }

    /// `CInventorySystem` 0xF (`WM_Inventory` 0x5D): on success the container's lists
    /// (`CContainer` 0x8 `Open_Done`, when sent), then
    /// `SendNotice_ContainerStateUpdate(container, 1)` and a refresh. An error shows a message
    /// (not mirrored) and changes nothing.
    pub fn open_done(
        &mut self,
        error: u32,
        packs: Option<Vec<u64>>,
        items: Option<Vec<u64>>,
        container: u64,
    ) -> Vec<GameplayNotice> {
        if error != 0 {
            return Vec::new();
        }
        if let Some(items) = items {
            self.contents.insert(
                container,
                Contents {
                    packs: packs.unwrap_or_default(),
                    items,
                },
            );
        }
        if !self.open.contains(&container) {
            self.open.push(container);
        }
        vec![
            GameplayNotice::ContainerStateUpdate {
                container,
                open: true,
            },
            refresh_container(container),
        ]
    }

    /// `CInventorySystem` 0x1B (`CloseContainer_Done`): on success `CContainer` 0x4
    /// (`Close_Done`, its lists dropped), then `SendNotice_ContainerStateUpdate(container, 0)`.
    pub fn close_done(&mut self, error: u32, container: u64) -> Vec<GameplayNotice> {
        if error != 0 {
            return Vec::new();
        }
        self.open.retain(|c| *c != container);
        self.contents.remove(&container);
        vec![GameplayNotice::ContainerStateUpdate {
            container,
            open: false,
        }]
    }

    /// `CInventorySystem` 0x1D: a successful take-all leaves the container's items in the
    /// player's pack, after its own (the server's later lists stay the truth).
    pub fn loot_all_done(&mut self, d: &InvTakeAllDesc) -> Vec<GameplayNotice> {
        if d.status() != 0 {
            return Vec::new();
        }
        let taken = self
            .contents
            .get_mut(&d.container())
            .map(|c| std::mem::take(&mut c.items))
            .unwrap_or_default();
        if let Some(pack) = self.contents.get_mut(&d.player()) {
            pack.items.extend(taken);
        }
        vec![
            refresh_container(d.container()),
            refresh_container(d.player()),
        ]
    }

    fn remove(&mut self, item: u64) -> Option<u64> {
        for (id, c) in &mut self.contents {
            let before = c.items.len() + c.packs.len();
            c.items.retain(|i| *i != item);
            c.packs.retain(|i| *i != item);
            if c.items.len() + c.packs.len() != before {
                return Some(*id);
            }
        }
        None
    }

    fn insert(&mut self, container: u64, item: u64, slot: i32, is_pack: bool) -> bool {
        let Some(c) = self.contents.get_mut(&container) else {
            return false;
        };
        let list = if is_pack { &mut c.packs } else { &mut c.items };
        let at = usize::try_from(slot).map_or(list.len(), |s| s.min(list.len()));
        list.insert(at, item);
        true
    }

    /// `CInventorySystem` 0x2 / 0xE (`MoveItem_Done`, `ReorganizeContents_Done`): a successful
    /// move leaves its container and, when the target is a listed container, lands at the
    /// server's slot (or the end); a drop or a give only leaves.
    pub fn move_done(&mut self, d: &InvMoveDesc) -> Vec<GameplayNotice> {
        if d.status() != 0 {
            return Vec::new();
        }
        let item = d.item();
        let is_pack = self.contents.values().any(|c| c.packs.contains(&item));
        let mut out = Vec::new();
        if let Some(from) = self.remove(item) {
            out.push(refresh_container(from));
        }
        if self.insert(d.target(), item, d.actual_target_slot(), is_pack) {
            let target = refresh_container(d.target());
            if !out.contains(&target) {
                out.push(target);
            }
        }
        out
    }

    /// `CInventorySystem` 0xC (`EquipItem_Done`): the item leaves its container and takes its
    /// location.
    pub fn equip_done(&mut self, d: &InvEquipDesc) -> Vec<GameplayNotice> {
        if d.status() != 0 {
            return Vec::new();
        }
        let mut out = Vec::new();
        if let Some(from) = self.remove(d.item()) {
            out.push(refresh_container(from));
        }
        self.equipped
            .retain(|(loc, i)| *loc != d.location() && *i != d.item());
        self.equipped.push((d.location(), d.item()));
        out.push(refresh_inventory());
        out
    }

    /// `CInventorySystem` 0x14 (`UnEquipItem_Done`): the item leaves its location for the
    /// desc's container and slot.
    pub fn unequip_done(&mut self, d: &InvEquipDesc) -> Vec<GameplayNotice> {
        if d.status() != 0 {
            return Vec::new();
        }
        self.equipped.retain(|(_, i)| *i != d.item());
        let mut out = vec![refresh_inventory()];
        if self.insert(d.container(), d.item(), d.slot(), false) {
            out.push(refresh_container(d.container()));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_net::protocol::pack_id_list;

    fn contents(packs: &[u64], items: &[u64]) -> ContainerContents {
        ContainerContents {
            segments: pack_id_list(&[]),
            packs: packs.to_vec(),
            items: items.to_vec(),
        }
    }

    #[test]
    fn moves_and_equips_apply_to_the_listed_contents() {
        let mut inv = InventoryState::default();
        inv.set_contents(1, contents(&[9], &[5, 6, 7]));
        inv.set_contents(9, contents(&[], &[]));
        assert_eq!(inv.locate(6), Some((1, 1)));
        // Into the side pack.
        let mut d = InvMoveDesc::new(6, 1, 1, 9, 0, 1);
        d.set_actual_target_slot(0);
        let notices = inv.move_done(&d);
        assert_eq!(notices.len(), 2);
        assert_eq!(inv.contents(1).unwrap().items, [5, 7]);
        assert_eq!(inv.contents(9).unwrap().items, [6]);
        // A failed move changes nothing; a drop only leaves.
        let mut failed = InvMoveDesc::new(5, 1, 0, 0x77, -1, 1);
        failed.set_status(0x2711);
        assert!(inv.move_done(&failed).is_empty());
        inv.move_done(&InvMoveDesc::new(5, 1, 0, 0x77, -1, 1));
        assert_eq!(inv.contents(1).unwrap().items, [7]);
        // Equip and take off.
        inv.equip_done(&InvEquipDesc::equip(7, 1, 0x10, 1, 0));
        assert_eq!((inv.location_of(7), inv.locate(7)), (Some(0x10), None));
        inv.unequip_done(&InvEquipDesc::unequip(7, 1, 0x10, 0));
        assert_eq!((inv.location_of(7), inv.locate(7)), (None, Some((1, 0))));
    }
}
