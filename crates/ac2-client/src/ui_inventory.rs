//! The inventory panel's backpack (`gmBackpackUI` over `ContainerDisplay` / `ItemDisplay`) driven
//! from the client: the player answers what the slots show from the frame's inventory, the
//! container notices go to it, and its presses become selection, use and examination.

use ac2_ui::{BackpackUi, InventorySource, ItemRequest, ItemSlot, UiTree};
use ac2_weenie::GameplayNotice;

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

impl InventorySource for ClientPlayer<'_> {
    fn item(&self, item: u64) -> Option<ItemSlot> {
        let frame = self.frame.inventory.items.iter().find(|i| i.id == item)?;
        Some(ItemSlot {
            icon: frame.stats.icon,
            icon_layers: frame
                .icon_layers
                .iter()
                .map(|&(_, image, _)| image)
                .collect(),
            name: self.render_info(&frame.name),
            quantity: frame.stats.quantity.max(0) as u32,
            stackable: frame.stats.stackable,
            max_stack: frame.stats.max_stack.unwrap_or(0).max(0) as u32,
        })
    }

    /// The player's own contents as one segment (the first page); retail's script answers the
    /// side pages' segments, which are not asked for yet.
    fn contents_info(&self, container: u64, segment: u32) -> (Vec<u64>, u32, u32) {
        let inventory = &self.frame.inventory;
        let found = if inventory.pack.id == container
            || self
                .frame
                .player
                .as_ref()
                .is_some_and(|p| p.id == container)
        {
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
        (found.items.clone(), 0, size)
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
}

/// One notice for the backpack; `selection` is the selection before it.
pub(crate) fn backpack_notice(
    backpack: &mut BackpackUi,
    tree: &mut UiTree,
    player: &ClientPlayer<'_>,
    selection: &mut u64,
    notice: &GameplayNotice,
) {
    match *notice {
        GameplayNotice::UpdateContainer {
            container,
            code,
            item,
            slot,
        } => {
            backpack.update_container(tree, player, container, code, item, slot);
        }
        GameplayNotice::ItemUpdate(item) => backpack.item_changed(tree, player, item),
        GameplayNotice::SelectionChanged(new) => {
            backpack.selection_changed(tree, *selection, new);
            *selection = new;
        }
        _ => {}
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
    }
    false
}
