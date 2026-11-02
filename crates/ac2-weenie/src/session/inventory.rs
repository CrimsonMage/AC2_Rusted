//! The `WM_Inventory` answers kept in [`InventoryState`] (`CInventorySystem`'s state). The
//! requests themselves (`gmCWInterface::MoveItem`, `EquipItem`, `UnEquipItem`, `UpdateOpen`,
//! `LootAllFromContainer`) are the player's script's: it builds and sends the descriptors.

use ac2_net::protocol::{
    script_function as function, unpack_values, ContainerContents, Equipment, ScriptMessage,
};

use super::{OnlineSession, SessionEvent};
use crate::inventory::InventoryState;

impl OnlineSession {
    /// The inventory as the player's scripts would keep it.
    pub fn inventory(&self) -> &InventoryState {
        &self.inventory
    }

    /// `CPlayer` 0x65's inventory part (`WM_Player` 0x27): arguments 7-9 to
    /// `CInventorySystem` 0x6 (the equipment), 10-12 to 0x1A (the player's contents). Nothing
    /// when the arguments do not decode.
    pub(super) fn init_player_inventory(&mut self, arguments: &[u8]) -> Vec<SessionEvent> {
        let (Ok(values), Some(player)) = (unpack_values(arguments), self.world.player()) else {
            return Vec::new();
        };
        if values.len() != 14 {
            return Vec::new();
        }
        let mut notices = Vec::new();
        if let Ok(contents) = ContainerContents::from_values(&values[9..12]) {
            notices.push(self.inventory.set_contents(player, contents));
        }
        if let Ok(equipment) = Equipment::from_values(&values[6..9]) {
            notices.push(self.inventory.set_equipment(&equipment));
        }
        notices.into_iter().map(SessionEvent::Notice).collect()
    }

    /// `CItem` 0x3 (`OnStatChanged`) for an item: quantity (int 0x106) and durability (0x26A,
    /// 0x26B) raise `SendNotice_ItemUpdate`.
    pub(super) fn item_quality_changed(
        &self,
        object: u64,
        table: u32,
        quality: u32,
    ) -> Option<SessionEvent> {
        (Some(object) != self.world.player()
            && table == 4
            && matches!(quality, 0x106 | 0x26A | 0x26B))
        .then_some(SessionEvent::Notice(crate::GameplayNotice::ItemUpdate(
            object,
        )))
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
                | function::LOOT_ALL_DONE
                | function::TRANSMUTE_ALL_DONE
                | function::OPEN_CONTAINER_DONE
                | function::CLOSE_CONTAINER_DONE
                | function::DISCRETE_DESTROY_CONTAINED_ITEM
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
            ScriptMessage::LootAllDone(d) => inv.loot_all_done(&d),
            // `CInventorySystem` 0x20: the container's contents change; the server's lists
            // follow.
            ScriptMessage::TransmuteAllDone(d) => vec![crate::GameplayNotice::UpdateContainer {
                container: d.container(),
                code: crate::INVENTORY_REFRESH,
                item: 0,
                slot: 0,
            }],
            ScriptMessage::OpenContainerDone {
                error,
                packs,
                items,
                container,
            } => inv.open_done(error, packs, items, container),
            ScriptMessage::CloseContainerDone { error, container } => {
                inv.close_done(error, container)
            }
            ScriptMessage::DiscreteDestroyContainedItem(item) => inv.discrete_destroy(item),
            _ => return None,
        };
        Some(notices.into_iter().map(SessionEvent::Notice).collect())
    }
}
