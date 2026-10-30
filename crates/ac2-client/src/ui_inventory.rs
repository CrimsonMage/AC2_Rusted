//! The inventory panels' player side: the answers they read from the player (`ClientPlayer`
//! implements their sources), and the requests they queue becoming gameplay commands. The panels'
//! notices, dialogs and request mapping live in `ac2_ui::inventory_notices`.

use ac2_net::protocol::StringInfo;
use ac2_ui::{InventoryNoticeSource, InventorySource, ItemRequest, ItemSlot};

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

impl InventorySource for ClientPlayer<'_> {
    fn waiting_tooltip(&self) -> Option<String> {
        self.ui().ready_ui_string("ID_Item_TT_WaitingForServer")
    }

    fn item_name(&self, item: u64, quantity: u32) -> Option<String> {
        let name = self.client.borrow().item_name_at(item, quantity)?;
        Some(self.render_info(&name))
    }

    fn item(&self, item: u64) -> Option<ItemSlot> {
        let frame = self.frame.inventory.items.iter().find(|i| i.id == item)?;
        Some(ItemSlot {
            icon: frame.stats.icon,
            icon_layers: frame
                .icon_layers
                .iter()
                .map(|&(_, image, shift_color)| ac2_ui::IconLayer { image, shift_color })
                .collect(),
            name: self.render_info(&frame.name),
            quantity: frame.stats.quantity.max(0) as u32,
            stackable: frame.stats.stackable,
            max_stack: frame.stats.max_stack.unwrap_or(0).max(0) as u32,
        })
    }

    /// `gmCWInterface::GetContentsInfo`: the player's script answers for the player
    /// ([`Client::contents_info`](crate::Client::contents_info)); without the VM, or for another
    /// container, its contents from the frame as one segment (the first page).
    fn contents_info(&self, container: u64, segment: u32) -> (Vec<u64>, u32, u32) {
        let inventory = &self.frame.inventory;
        let is_player = inventory.pack.id == container
            || self
                .frame
                .player
                .as_ref()
                .is_some_and(|p| p.id == container);
        if is_player {
            if let Ok(info) = self.client.borrow_mut().contents_info(segment) {
                return info;
            }
        }
        let found = if is_player {
            Some(&inventory.pack)
        } else {
            inventory.containers.iter().find(|c| c.id == container)
        };
        let Some(found) = found else {
            return (Vec::new(), 0, 0);
        };
        let size = if segment == 0 {
            found.capacity.max(found.items.len() as i32) as u32
        } else {
            0
        };
        (found.items.to_vec(), 0, size)
    }

    fn max_container_size(&self, container: u64) -> u32 {
        let inventory = &self.frame.inventory;
        let capacity = if inventory.pack.id == container {
            inventory.pack.capacity
        } else {
            inventory
                .containers
                .iter()
                .find(|c| c.id == container)
                .map_or(0, |c| c.capacity)
        };
        capacity.max(0) as u32
    }

    fn selection(&self) -> u64 {
        self.frame.selection.as_ref().map_or(0, |s| s.id)
    }

    fn preferred_location(&self, item: u64) -> u32 {
        self.frame
            .inventory
            .items
            .iter()
            .find(|i| i.id == item)
            .map_or(0, |i| i.stats.preferred_location)
    }

    fn durability(&self, item: u64) -> Option<(i32, i32)> {
        self.frame
            .inventory
            .items
            .iter()
            .find(|i| i.id == item)
            .and_then(|i| i.stats.durability)
    }

    fn player(&self) -> u64 {
        self.frame.player.as_ref().map_or(0, |p| p.id)
    }

    fn money(&self) -> u32 {
        self.frame.inventory.money.max(0) as u32
    }

    /// `gInventoryGlobals::CallIFun_GetCoinMaxStackSize`; without the script, no limit.
    fn coin_max_stack(&self) -> u32 {
        self.client
            .borrow_mut()
            .coin_max_stack_size()
            .map_or(u32::MAX, |m| m.max(0) as u32)
    }

    /// `gmCWInterface::DoIHaveItem`: the player's script, else the frame's mirror.
    fn has_item(&self, item: u64) -> bool {
        self.carries(item)
    }

    /// `gmCWInterface::IsOnInventory` (the worn test): the player's script, else the frame's worn
    /// list.
    fn is_on_inventory(&self, item: u64) -> bool {
        use ac2_script::host::inventory_queries::InventoryQueries;
        InventoryQueries::is_on_inventory(&mut **self.client.borrow_mut(), item).unwrap_or_else(
            || {
                self.frame
                    .inventory
                    .equipped
                    .iter()
                    .any(|&(_, i)| i == item)
            },
        )
    }

    fn melts_to_money(&self, item: u64) -> bool {
        self.client.borrow().melts_to_money(item)
    }

    /// `gmCWInterface::GetBypassGoldMeltingConfirmationDlg` (`ggmCWInterface::CallIFun_*`, 0xB400D7
    /// on the player).
    fn bypass_melting_confirmation(&self) -> bool {
        self.client
            .borrow_mut()
            .call_ifun(0x00B4_00D7, &[])
            .is_ok_and(|r| r.retval != 0)
    }

    fn durability_tooltip(&self, current: i32, max: i32) -> Option<String> {
        ClientPlayer::durability_tooltip(self, current, max)
    }
}

impl InventoryNoticeSource for ClientPlayer<'_> {
    fn equipped(&self) -> Vec<(u32, u64)> {
        ClientPlayer::equipped(self)
    }

    fn render_wire(&self, info: &StringInfo) -> String {
        self.render_info(info)
    }
}

/// A slot's press as commands; whether the examination panel opens.
pub(crate) fn item_commands(request: ItemRequest, commands: &mut Vec<Command>) -> bool {
    match request {
        ItemRequest::Select(item) => commands.push(Command::Select(Some(item))),
        ItemRequest::Use(item) => {
            commands.push(Command::Select(Some(item)));
            commands.push(Command::UseSelection);
        }
        ItemRequest::Examine(item) => {
            commands.push(Command::Select(Some(item)));
            return true;
        }
        ItemRequest::Move {
            item,
            from,
            from_slot,
            target,
            target_slot,
            quantity,
        } => commands.push(Command::MoveItem {
            item,
            from,
            from_slot,
            target,
            target_slot,
            quantity: quantity as i32,
        }),
        ItemRequest::Unequip { item, target_slot } => {
            commands.push(Command::UnequipItem { item, target_slot });
        }
        // Answered by the shop and the consignment in the HUD.
        ItemRequest::DragFrom { .. } => {}
        ItemRequest::Split { .. }
        | ItemRequest::DropSplit { .. }
        | ItemRequest::MoneyStack { .. }
        | ItemRequest::MoneyBagSplit { .. } => {}
        ItemRequest::DragToMoneyBag {
            item,
            quantity,
            from_slot,
            from_container,
        } => commands.push(Command::DragToMoneyBag {
            item,
            quantity,
            from_slot,
            from_container,
        }),
        ItemRequest::DragFromMoneyBag {
            quantity,
            to_slot,
            to_container,
        } => commands.push(Command::DragFromMoneyBag {
            quantity,
            to_slot,
            to_container,
        }),
        ItemRequest::Equip { item, location } => {
            commands.push(Command::EquipItem { item, location });
        }
    }
    false
}
