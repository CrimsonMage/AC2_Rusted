//! The session's inventory entry points: the IFuns the inventory panels call on the player's
//! weenie (`gmCWInterface::MoveItem`, `EquipItem`, `UnEquipItem`, `UpdateOpen`), as the
//! `CContainer` / `CInventorySystem` scripts send them, and the `WM_Inventory` answers kept in
//! [`InventoryState`].

use ac2_net::protocol::{
    script_function as function, InvEquipDesc, InvMoveDesc, ScriptMessage, ScriptRequest,
};

use super::{OnlineSession, SessionEvent};
use crate::inventory::InventoryState;

impl OnlineSession {
    /// The inventory as the player's scripts would keep it.
    pub fn inventory(&self) -> &InventoryState {
        &self.inventory
    }

    /// `gmCWInterface::MoveItem` -> `CContainer` 0x7 -> `CInventorySystem` 0x12
    /// (`DirectiveMoveItem`): move `quantity` of `item` from slot `from_slot` of `from` to slot
    /// `target_slot` (-1: anywhere) of `target`. A drop targets the world, a give another
    /// creature, a split moves less than the whole stack. A move inside the player's own pack
    /// goes as a reorganization (`CInventorySystem` 0x19), and one onto its own slot sends
    /// nothing.
    pub fn move_item(
        &mut self,
        item: u64,
        from: u64,
        from_slot: i32,
        target: u64,
        target_slot: i32,
        quantity: i32,
    ) {
        let desc = InvMoveDesc::new(item, from, from_slot, target, target_slot, quantity);
        let own_pack = Some(target) == self.world.player() && target == from;
        let request = if own_pack {
            if from_slot == target_slot {
                return;
            }
            ScriptRequest::ReorganizeContents(desc)
        } else {
            ScriptRequest::MoveItem(desc)
        };
        self.send(request.to_event());
    }

    /// `gmCWInterface::EquipItem` -> `Inventory` 0x21 -> `CInventorySystem` 0x16: wield `item`
    /// at `location`. Retail refuses an item not in the player's containers
    /// (`LocateItemInContents`), as this does.
    pub fn equip_item(&mut self, item: u64, location: u32) -> Result<(), String> {
        let player = self.world.player().ok_or("not in the world")?;
        let (from, slot) = self
            .inventory
            .locate(item)
            .ok_or_else(|| format!("{item:#018x} is not in the player's containers"))?;
        let desc = InvEquipDesc::equip(item, player, location, from, slot);
        self.send(ScriptRequest::EquipItem(desc).to_event());
        Ok(())
    }

    /// `gmCWInterface::UnEquipItem` -> `Inventory` 0x9: take `item` off into slot
    /// `target_slot` of the player's pack; its location is the one it is equipped at
    /// (`InventorySystem` 0x20).
    pub fn unequip_item(&mut self, item: u64, target_slot: i32) -> Result<(), String> {
        let player = self.world.player().ok_or("not in the world")?;
        let location = self
            .inventory
            .location_of(item)
            .ok_or_else(|| format!("{item:#018x} is not equipped"))?;
        let desc = InvEquipDesc::unequip(item, player, location, target_slot);
        self.send(ScriptRequest::UnequipItem(desc).to_event());
        Ok(())
    }

    /// `gmCWInterface::UpdateOpen` -> `CContainer` 0x9 / 0x3 -> `CInventorySystem` 0x11 / 0x18:
    /// open or close `container`.
    pub fn set_container_open(&mut self, container: u64, open: bool) {
        let request = if open {
            ScriptRequest::OpenContainer(container)
        } else {
            ScriptRequest::CloseContainer(container)
        };
        self.send(request.to_event());
    }

    /// A `WM_Inventory` answer the stand-in keeps; `None` for other script calls.
    pub(super) fn receive_inventory(
        &mut self,
        object: u64,
        func: u32,
        arguments: &[u8],
    ) -> Option<Vec<SessionEvent>> {
        let player = Some(object) == self.world.player();
        let known = matches!(
            func,
            function::CONTAINER_CONTENTS
                | function::EQUIPMENT
                | function::MOVE_ITEM_DONE
                | function::REORGANIZE_CONTENTS_DONE
                | function::EQUIP_ITEM_DONE
                | function::UNEQUIP_ITEM_DONE
        );
        if !known || (!player && func != function::CONTAINER_CONTENTS) {
            return None;
        }
        let message = match ScriptMessage::decode(func, arguments)? {
            Ok(message) => message,
            Err(error) => {
                return Some(vec![SessionEvent::Rejected(format!(
                    "script function {func:#010x}: {error}"
                ))])
            }
        };
        let inv = &mut self.inventory;
        let notices = match message {
            ScriptMessage::ContainerContents(c) => vec![inv.set_contents(object, c)],
            ScriptMessage::Equipment(e) => vec![inv.set_equipment(&e)],
            ScriptMessage::MoveItemDone(d) | ScriptMessage::ReorganizeContentsDone(d) => {
                inv.move_done(&d)
            }
            ScriptMessage::EquipItemDone(d) => inv.equip_done(&d),
            ScriptMessage::UnequipItemDone(d) => inv.unequip_done(&d),
            _ => return None,
        };
        Some(notices.into_iter().map(SessionEvent::Notice).collect())
    }
}
