//! The inventory panels' player side: the answers they read from the player (`ClientPlayer`
//! implements their sources), and the requests they queue becoming gameplay commands. The panels'
//! notices, dialogs and request mapping live in `ac2_ui::inventory_notices`.

use ac2_net::protocol::StringInfo;
use ac2_ui::{
    BackpackUi, DialogFactory, ExternalContainerRequest, ExternalContainerUi,
    InventoryNoticeSource, InventorySource, ItemRequest, ItemSlot, PaperDollUi, SecureTradeSplit,
    SecureTradeUi, StackUi, UiTree,
};
use ac2_weenie::{GameplayNotice, SessionEvent};

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

/// The player's effects for the status bar's strip (`gmCWInterface::InqEffectList`); none when
/// the script does not answer.
pub(crate) fn effect_entries(player: &ClientPlayer<'_>) -> Vec<ac2_ui::EffectEntry> {
    let nodes = player.client.borrow_mut().effect_list().unwrap_or_default();
    nodes
        .into_iter()
        .map(|n| ac2_ui::EffectEntry {
            eid: n.eid,
            pulsed: n.is_pulsed(),
            name: n.name,
            icon: n.icon,
            time_cast: n.time_cast,
            duration: n.duration,
        })
        .collect()
}

/// `gmPaperDollUI::RecvNotice_UpdateInventory`, `RecvNotice_ItemUpdate`, the slots' selection.
pub(crate) fn paper_doll_notice(
    doll: &mut PaperDollUi,
    tree: &mut UiTree,
    player: &ClientPlayer<'_>,
    old_selection: u64,
    notice: &GameplayNotice,
) {
    ac2_ui::paper_doll_notice(doll, tree, player, old_selection, notice);
}

/// The backpack's notices; `selection` is the selection before them.
pub(crate) fn backpack_notice(
    backpack: &mut BackpackUi,
    tree: &mut UiTree,
    player: &ClientPlayer<'_>,
    selection: &mut u64,
    notice: &GameplayNotice,
) {
    ac2_ui::backpack_notice(backpack, tree, player, selection, notice);
}

/// A slot's request: the stack dialogs it opens (`ac2_ui::item_dialog`), then the rest as
/// commands. Whether the examination panel opens.
pub(crate) fn item_request(
    request: ItemRequest,
    tree: &mut UiTree,
    stack: Option<&mut StackUi>,
    player: &ClientPlayer<'_>,
    commands: &mut Vec<Command>,
) -> bool {
    match ac2_ui::item_dialog(request, tree, stack, player) {
        Some(request) => item_commands(request, commands),
        None => false,
    }
}

/// `StackUI` event hearing; see `ac2_ui::stack_event`.
pub(crate) fn stack_event(stack: &mut StackUi, tree: &mut UiTree, event: &SessionEvent) {
    ac2_ui::stack_event(stack, tree, event);
}

/// One event for the external container; see `ac2_ui::external_container_event`.
pub(crate) fn external_container_event(
    panel: &mut ExternalContainerUi,
    tree: &mut UiTree,
    player: &ClientPlayer<'_>,
    stack: Option<&mut StackUi>,
    selection: u64,
    event: &SessionEvent,
) {
    ac2_ui::external_container_event(panel, tree, player, stack, selection, event);
}

/// The external container's request as commands; whether the examination panel opens.
pub(crate) fn external_container_request(
    request: ExternalContainerRequest,
    tree: &mut UiTree,
    stack: Option<&mut StackUi>,
    player: &ClientPlayer<'_>,
    commands: &mut Vec<Command>,
) -> bool {
    match request {
        ExternalContainerRequest::Item(request) => {
            item_request(request, tree, stack, player, commands)
        }
        ExternalContainerRequest::Close(container) => {
            commands.push(Command::SetContainerOpen {
                container,
                open: false,
            });
            false
        }
        ExternalContainerRequest::LootAll(container) => {
            commands.push(Command::LootAll(container));
            false
        }
        ExternalContainerRequest::TransmuteAll(container) => {
            commands.push(Command::TransmuteAll(container));
            false
        }
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

/// `gmSecureTradeUI`'s notices; see `ac2_ui::secure_trade_notice`.
pub(crate) fn secure_trade_notice(
    panel: &mut SecureTradeUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    player: &ClientPlayer<'_>,
    now: f64,
    notice: &GameplayNotice,
) {
    ac2_ui::secure_trade_notice(panel, tree, dialogs, player, now, notice);
}

/// A stack dropped on the trade's list asks how many; see `ac2_ui::secure_trade_split`.
pub(crate) fn secure_trade_split(
    split: SecureTradeSplit,
    tree: &mut UiTree,
    stack: Option<&mut StackUi>,
    player: &ClientPlayer<'_>,
) {
    ac2_ui::secure_trade_split(split, tree, stack, player);
}
