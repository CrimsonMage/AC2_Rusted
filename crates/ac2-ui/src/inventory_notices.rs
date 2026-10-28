//! The inventory panels' gameplay notices (`gmPaperDollUI::RecvNotice_*`, `gmBackpackUI`,
//! `gmExternalContainerUI`, `gmStackUI`, `gmSecureTradeUI::RecvNotice_Trade_*`) and the stack
//! dialogs their requests open (`CM_UI::SendNotice_InitStackUI`, `gmStackUI::RecvNotice_InitStackUI`).
//! The player answers through `InventoryNoticeSource`. A request that needs no dialog comes back to
//! the client, which turns it into a gameplay command.

use ac2_net::protocol::StringInfo;
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::backpack::BackpackUi;
use crate::dialog::DialogFactory;
use crate::element::UiTree;
use crate::external_container::ExternalContainerUi;
use crate::item_display::{InventorySource, ItemRequest};
use crate::paper_doll::PaperDollUi;
use crate::secure_trade::{SecureTradeSplit, SecureTradeUi};
use crate::stack::StackUi;
use crate::ui_string::{UiString, UiStrings};

/// What the inventory panels read from the player: the items and containers, and the strings and
/// the equipped gear the notices need.
pub trait InventoryNoticeSource: InventorySource + UiStrings {
    /// The worn items as `(location, item)` pairs (`gmCWInterface::GetEquippedByLocation`).
    fn equipped(&self) -> Vec<(u32, u64)>;
    /// A string the server sent, rendered.
    fn render_wire(&self, info: &StringInfo) -> String;
}

/// One notice for the paper doll (`gmPaperDollUI::RecvNotice_UpdateInventory`,
/// `RecvNotice_ItemUpdate`, the slots' selection); `old_selection` is the selection before it.
pub fn paper_doll_notice<S: InventoryNoticeSource>(
    doll: &mut PaperDollUi,
    tree: &mut UiTree,
    source: &S,
    old_selection: u64,
    notice: &GameplayNotice,
) {
    match *notice {
        GameplayNotice::UpdateInventory {
            code,
            item,
            location,
        } => {
            let equipped = source.equipped();
            doll.update_inventory(tree, source, code, item, location, &equipped);
        }
        // `gmPaperDollUI::RecvNotice_CreateObject` (Legions 0x00443250): an object in the doll's
        // table refreshes its slot.
        GameplayNotice::ItemUpdate(item) | GameplayNotice::CreateObject(item) => {
            doll.item_changed(tree, source, item)
        }
        GameplayNotice::SelectionChanged(new) => doll.selection_changed(tree, old_selection, new),
        _ => {}
    }
}

/// One notice for the backpack; `selection` is the selection before it.
pub fn backpack_notice<S: InventoryNoticeSource>(
    backpack: &mut BackpackUi,
    tree: &mut UiTree,
    source: &S,
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
            backpack.update_container(tree, source, container, code, item, slot);
        }
        // `ItemDisplay::RecvNotice_CreateObject` / `RecvNotice_LocalVisualDescUpdate` (Legions
        // 0x0042EEF0 / 0x0042EEC0): the display of that item updates (`ItemDisplay::Update`).
        GameplayNotice::ItemUpdate(item)
        | GameplayNotice::CreateObject(item)
        | GameplayNotice::LocalVisualDescUpdate(item) => backpack.item_changed(tree, source, item),
        GameplayNotice::SelectionChanged(new) => {
            backpack.selection_changed(tree, *selection, new);
            *selection = new;
        }
        _ => {}
    }
}

/// A slot's request that opens a stack dialog: a split shows it with the item's name as the
/// question (`CM_UI::SendNotice_InitStackUI`). Returns the requests no dialog takes, for the
/// client's gameplay commands.
pub fn item_dialog<S: InventoryNoticeSource>(
    request: ItemRequest,
    tree: &mut UiTree,
    stack: Option<&mut StackUi>,
    source: &S,
) -> Option<ItemRequest> {
    match request {
        ItemRequest::Split {
            context,
            item,
            target,
            amount,
        } => {
            if let Some(dialog) = stack {
                let name = source.item(item).map(|i| i.name).unwrap_or_default();
                dialog.init(tree, context, &name, amount, amount, item, target);
            }
            None
        }
        // `gmMoneyBagUI::HandleMoneyBagDrop`: the stack's name, from the player.
        ItemRequest::MoneyBagSplit {
            context,
            item,
            from,
            amount,
        } => {
            if let Some(dialog) = stack {
                let name = source.item(item).map(|i| i.name).unwrap_or_default();
                dialog.init(tree, context, &name, amount, amount, from, 0);
            }
            None
        }
        // `gm3DItemsUI::MidItemDrop`: the stack's name, from the player.
        ItemRequest::DropSplit {
            context,
            item,
            amount,
        } => {
            if let Some(dialog) = stack {
                let name = source.item(item).map(|i| i.name).unwrap_or_default();
                dialog.init(
                    tree,
                    context,
                    &name,
                    amount,
                    amount,
                    InventorySource::player(source),
                    0,
                );
            }
            None
        }
        // `ContainerDisplay::HandleDragFromMoneyBag`: `ID_Inventory_MoneyStack`, no target item.
        ItemRequest::MoneyStack {
            context,
            target,
            amount,
        } => {
            if let Some(dialog) = stack {
                let question = source.render_string(&UiString::ui(
                    crate::tokenize("ID_Inventory_MoneyStack"),
                    Vec::new(),
                ));
                dialog.init(tree, context, &question, amount, amount, 0, target);
            }
            None
        }
        request => Some(request),
    }
}

/// `gmStackUI::RecvNotice_ContainerStateUpdate` and `RecvNotice_BeingDeleted`, which it hears
/// for the object the amount comes from.
pub fn stack_event(stack: &mut StackUi, tree: &mut UiTree, event: &SessionEvent) {
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
pub fn external_container_event<S: InventoryNoticeSource>(
    panel: &mut ExternalContainerUi,
    tree: &mut UiTree,
    source: &S,
    stack: Option<&mut StackUi>,
    selection: u64,
    event: &SessionEvent,
) {
    let close = match event {
        SessionEvent::Notice(GameplayNotice::ContainerStateUpdate { container, open }) => {
            panel.container_state_update(tree, source, *container, *open)
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
                    display.update_container(tree, source, code, item, slot);
                }
                GameplayNotice::ItemUpdate(item)
                | GameplayNotice::CreateObject(item)
                | GameplayNotice::LocalVisualDescUpdate(item) => {
                    display.item_changed(tree, source, item)
                }
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

/// `gmSecureTradeUI`'s notices (`RecvNotice_Trade_*`); `now` is the frame's time, which the
/// ended trade's delay runs from. The partner's name is rendered here, from the notice's string.
pub fn secure_trade_notice<S: InventoryNoticeSource>(
    panel: &mut SecureTradeUi,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    source: &S,
    now: f64,
    notice: &GameplayNotice,
) {
    match notice {
        GameplayNotice::TradeRegistered {
            initiator,
            partner,
            partner_name,
        } => {
            let name = source.render_wire(partner_name);
            panel.recv_trade_be_registered(tree, dialogs, source, *initiator, *partner, &name);
        }
        GameplayNotice::TradeOpened => panel.recv_trade_be_opened(tree),
        GameplayNotice::TradeClosed { source: from, .. } => {
            panel.recv_trade_be_closed(tree, dialogs, source, *from, now);
        }
        GameplayNotice::TradeAccepted(src) => panel.recv_trade_be_accepted(tree, *src),
        GameplayNotice::TradeDeclined(src) => panel.recv_trade_be_declined(tree, *src),
        GameplayNotice::TradeReset => panel.recv_trade_be_reset(tree),
        GameplayNotice::TradeAcceptanceCleared => panel.recv_trade_be_acceptance_cleared(tree),
        GameplayNotice::TradeOffered {
            source: from,
            item,
            amount,
        } => panel.recv_trade_be_offered(tree, source, *from, *item, *amount),
        GameplayNotice::TradeRevoked { source: from, item } => {
            panel.recv_trade_be_revoked(tree, *from, *item);
        }
        _ => {}
    }
}

/// A stack dropped on the trade's list asks how many (`CM_UI::SendNotice_InitStackUI`): the
/// stack's question is the item's name, and the stack comes from the player.
pub fn secure_trade_split<S: InventoryNoticeSource>(
    split: SecureTradeSplit,
    tree: &mut UiTree,
    stack: Option<&mut StackUi>,
    source: &S,
) {
    if let Some(dialog) = stack {
        let name = source.item(split.item).map(|i| i.name).unwrap_or_default();
        dialog.init(
            tree,
            split.context,
            &name,
            split.amount,
            split.amount,
            InventorySource::player(source),
            0,
        );
    }
}
