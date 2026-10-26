//! The inventory panel's backpack (`gmBackpackUI` over `ContainerDisplay` / `ItemDisplay`) driven
//! from the client: the player answers what the slots show from the frame's inventory, the
//! container notices go to it, and its presses become selection, use and examination.

use ac2_ui::{
    BackpackUi, DialogFactory, ExternalContainerRequest, ExternalContainerUi, InventorySource,
    ItemRequest, ItemSlot, PaperDollUi, SecureTradeSplit, SecureTradeUi, StackUi, UiString,
    UiStrings, UiTree,
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
        let info = ac2_db::RenderInfo {
            string_id: ac2_ui::tokenize("ID_Inventory_TT_Durability"),
            table_id: self.ui_table,
            literal: None,
            variables: vec![
                (ac2_ui::tokenize("current"), ac2_db::RenderVar::Int(current)),
                (ac2_ui::tokenize("max"), ac2_db::RenderVar::Int(max)),
            ],
        };
        Some(String::from_utf16_lossy(&ac2_db::get_string(
            self.strings,
            &info,
            false,
        )))
    }
}

/// One notice for the paper doll (`gmPaperDollUI::RecvNotice_UpdateInventory`,
/// `RecvNotice_ItemUpdate`, the slots' selection); `old_selection` is the selection before it.
pub(crate) fn paper_doll_notice(
    doll: &mut PaperDollUi,
    tree: &mut UiTree,
    player: &ClientPlayer<'_>,
    old_selection: u64,
    notice: &GameplayNotice,
) {
    match *notice {
        GameplayNotice::UpdateInventory {
            code,
            item,
            location,
        } => {
            let equipped = &player.frame.inventory.equipped;
            doll.update_inventory(tree, player, code, item, location, equipped);
        }
        GameplayNotice::ItemUpdate(item) => doll.item_changed(tree, player, item),
        GameplayNotice::SelectionChanged(new) => doll.selection_changed(tree, old_selection, new),
        _ => {}
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

/// A slot's request: a split shows the stack dialog (`CM_UI::SendNotice_InitStackUI` to
/// `gmStackUI::RecvNotice_InitStackUI`, the item's name as the question); the rest are commands.
/// Whether the examination panel opens.
pub(crate) fn item_request(
    request: ItemRequest,
    tree: &mut UiTree,
    stack: Option<&mut StackUi>,
    player: &ClientPlayer<'_>,
    commands: &mut Vec<Command>,
) -> bool {
    match request {
        ItemRequest::Split {
            context,
            item,
            target,
            amount,
        } => {
            if let Some(dialog) = stack {
                let name = player.item(item).map(|i| i.name).unwrap_or_default();
                dialog.init(tree, context, &name, amount, amount, item, target);
            }
            false
        }
        // `gmMoneyBagUI::HandleMoneyBagDrop`: the stack's name, from the player.
        ItemRequest::MoneyBagSplit {
            context,
            item,
            from,
            amount,
        } => {
            if let Some(dialog) = stack {
                let name = player.item(item).map(|i| i.name).unwrap_or_default();
                dialog.init(tree, context, &name, amount, amount, from, 0);
            }
            false
        }
        // `gm3DItemsUI::MidItemDrop`: the stack's name, from the player.
        ItemRequest::DropSplit {
            context,
            item,
            amount,
        } => {
            if let Some(dialog) = stack {
                let name = player.item(item).map(|i| i.name).unwrap_or_default();
                dialog.init(
                    tree,
                    context,
                    &name,
                    amount,
                    amount,
                    ac2_ui::InventorySource::player(player),
                    0,
                );
            }
            false
        }
        // `ContainerDisplay::HandleDragFromMoneyBag`: `ID_Inventory_MoneyStack`, no target item.
        ItemRequest::MoneyStack {
            context,
            target,
            amount,
        } => {
            if let Some(dialog) = stack {
                let question = player.render_string(&UiString::ui(
                    ac2_ui::tokenize("ID_Inventory_MoneyStack"),
                    Vec::new(),
                ));
                dialog.init(tree, context, &question, amount, amount, 0, target);
            }
            false
        }
        request => item_commands(request, commands),
    }
}

/// `gmStackUI::RecvNotice_ContainerStateUpdate` and `RecvNotice_BeingDeleted`, which it hears
/// for the object the amount comes from.
pub(crate) fn stack_event(stack: &mut StackUi, tree: &mut UiTree, event: &SessionEvent) {
    match event {
        SessionEvent::Notice(GameplayNotice::ContainerStateUpdate { container, open }) => {
            stack.container_state_update(tree, *container, *open);
        }
        SessionEvent::ObjectRemoved(id) => stack.being_deleted(tree, *id),
        _ => {}
    }
}

/// One event for `gmExternalContainerUI` (it hears every object) and its display's container and
/// item notices; `selection` is the selection before the event. A cleanup closes the stack
/// dialog its drop waited on (`ContainerDisplay::Cleanup`'s `SendNotice_CloseStackUI`).
pub(crate) fn external_container_event(
    panel: &mut ExternalContainerUi,
    tree: &mut UiTree,
    player: &ClientPlayer<'_>,
    stack: Option<&mut StackUi>,
    selection: u64,
    event: &SessionEvent,
) {
    let close = match event {
        SessionEvent::Notice(GameplayNotice::ContainerStateUpdate { container, open }) => {
            panel.container_state_update(tree, player, *container, *open)
        }
        SessionEvent::ObjectRemoved(id) => panel.being_deleted(tree, *id),
        SessionEvent::Notice(notice) => {
            let Some(display) = panel.display_mut() else {
                return;
            };
            match *notice {
                GameplayNotice::UpdateContainer {
                    container,
                    code,
                    item,
                    slot,
                } if container == display.container => {
                    display.update_container(tree, player, code, item, slot);
                }
                GameplayNotice::ItemUpdate(item) => display.item_changed(tree, player, item),
                GameplayNotice::SelectionChanged(new) => {
                    display.selection_changed(tree, selection, new);
                }
                _ => {}
            }
            Vec::new()
        }
        _ => Vec::new(),
    };
    if let Some(stack) = stack {
        for context in close {
            stack.close_stack_ui(tree, context);
        }
    }
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
            return item_request(request, tree, stack, player, commands);
        }
        ExternalContainerRequest::Close(container) => {
            commands.push(Command::SetContainerOpen {
                container,
                open: false,
            });
        }
        ExternalContainerRequest::LootAll(container) => commands.push(Command::LootAll(container)),
        ExternalContainerRequest::TransmuteAll(container) => {
            commands.push(Command::TransmuteAll(container));
        }
    }
    false
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
        ItemRequest::MoveAndEquip {
            item,
            from,
            to,
            location,
        } => {
            commands.push(Command::MoveItem {
                item,
                from,
                from_slot: 0,
                target: to,
                target_slot: 0,
                quantity: 1,
            });
            commands.push(Command::EquipItem { item, location });
        }
    }
    false
}

/// `gmSecureTradeUI`'s notices (`RecvNotice_Trade_*`); `now` is the frame's time, which the
/// ended trade's delay runs from. The partner's name is rendered here, from the notice's string.
pub(crate) fn secure_trade_notice(
    panel: &mut SecureTradeUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    player: &ClientPlayer<'_>,
    now: f64,
    notice: &GameplayNotice,
) {
    match notice {
        GameplayNotice::TradeRegistered {
            initiator,
            partner,
            partner_name,
        } => {
            let name = player.render_info(partner_name);
            panel.recv_trade_be_registered(tree, dialogs, player, *initiator, *partner, &name);
        }
        GameplayNotice::TradeOpened => panel.recv_trade_be_opened(tree),
        GameplayNotice::TradeClosed { source, .. } => {
            panel.recv_trade_be_closed(tree, dialogs, player, *source, now);
        }
        GameplayNotice::TradeAccepted(src) => panel.recv_trade_be_accepted(tree, *src),
        GameplayNotice::TradeDeclined(src) => panel.recv_trade_be_declined(tree, *src),
        GameplayNotice::TradeReset => panel.recv_trade_be_reset(tree),
        GameplayNotice::TradeAcceptanceCleared => panel.recv_trade_be_acceptance_cleared(tree),
        GameplayNotice::TradeOffered {
            source,
            item,
            amount,
        } => panel.recv_trade_be_offered(tree, player, *source, *item, *amount),
        GameplayNotice::TradeRevoked { source, item } => {
            panel.recv_trade_be_revoked(tree, *source, *item);
        }
        _ => {}
    }
}

/// A stack dropped on the trade's list asks how many (`CM_UI::SendNotice_InitStackUI`): the
/// stack's question is the item's name, and the stack comes from the player.
pub(crate) fn secure_trade_split(
    split: SecureTradeSplit,
    tree: &mut UiTree,
    stack: Option<&mut StackUi>,
    player: &ClientPlayer<'_>,
) {
    if let Some(dialog) = stack {
        let name = player.item(split.item).map(|i| i.name).unwrap_or_default();
        dialog.init(
            tree,
            split.context,
            &name,
            split.amount,
            split.amount,
            player.player(),
            0,
        );
    }
}
