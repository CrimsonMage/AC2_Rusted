//! `gm3DItemsUI`: dragging to and from the 3D world, over `ItemSelectionField` (the root the
//! world is seen through).
//!
//! The field's drop callback is `gm3DItems_DDCB` (the coin field, or an item that came from a
//! container). An item dropped on it that was not worn and came from the player's own pack
//! (`FromID` the player) starts a target search (`StartItemDrop`: `StartFind3DTarget` asks the
//! selection render for the object under the pointer); on the next global loop
//! (`ListenToMessage`) `FinishFind3DTarget` has the answer (the player itself counts as none)
//! and `MidItemDrop` asks how many of a stack of more than one (`gmStackUI`), then
//! `FinishItemDrop` moves the item to the target (`gmCWInterface::MoveItem`; no target is the
//! ground). The coin field does the same with the money (`StartMoneyDrop`, `MidMoneyDrop`;
//! the answer is `DragFromMoneyBag` to the target).
//!
//! A press on the field that no element takes (`not_dragable`) with something selected starts
//! the selection's own drag (`Start3DDrag`): the field's `ItemIcon` takes the item's image, is
//! `dragable` and carries it as a shortcut (type 0x40000002), and the drag starts from it; with
//! no `FromID` the callbacks that need a container refuse it.

use crate::drag::{DragCheck, MSG_STOP_DRAG_ITEM};
use crate::element::UiTree;
use crate::item_display::{dragged_item, InventorySource, ItemRequest};
use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::token::tokenize;

/// `gm3DItemsUI`'s state.
#[derive(Debug, Clone, Default)]
pub struct Items3dUi {
    field: Option<usize>,
    /// `m_itemIcon`.
    icon: Option<usize>,
    /// `m_playerID`.
    player: u64,
    /// `m_itemContext`, `m_moneyContext`: the stack dialogs asked (0: none).
    item_context: u32,
    money_context: u32,
    /// `m_doItemDrop`, `m_doMoneyDrop`: a target search is on its way.
    do_item: bool,
    do_money: bool,
    /// `m_itemToDrop`, `m_targetID`, `m_fromSlot`.
    item: u64,
    target: u64,
    from_slot: i32,
    /// `StartFind3DTarget`'s point, until the selection render answers.
    pick: Option<(i32, i32)>,
}

impl Items3dUi {
    /// `gm3DItemsUI::gm3DItemsUI`: `gm3DItems_DDCB` on `ItemSelectionField` and its `ItemIcon`.
    pub fn create(tree: &mut UiTree, player: u64) -> Self {
        let field = tree.roots().iter().copied().find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("ItemSelectionField"))
        });
        if let Some(field) = field {
            tree.set_drag_drop_check(field, Some(DragCheck::WorldDrop));
        }
        Self {
            field,
            icon: field.and_then(|f| tree.child_recursive(f, tokenize("ItemIcon"))),
            player,
            ..Self::default()
        }
    }

    /// The point the selection render waits at (`WorldRender::StartSelection`), if a target
    /// search does.
    pub fn wants_pick(&self) -> Option<(i32, i32)> {
        self.pick
    }

    /// `gm3DItemsUI::StartFind3DTarget`: the search starts at the pointer, unless one is on.
    fn start_find_target(&mut self, tree: &UiTree) -> bool {
        if self.pick.is_some() {
            return false;
        }
        self.pick = Some(tree.pointer());
        true
    }

    /// `gm3DItemsUI::ListenToMessage`: a drop on the field, or a press on it nothing took.
    /// One frame of `gm3DItemsUI`: the target search the last frame started finishes with
    /// `picked` (`finish_pick`), then the frame's `messages` are heard. Returns the requests the
    /// drops made, in order.
    pub fn frame_pass(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        picked: Option<Option<u64>>,
        messages: &[UiMessage],
    ) -> Vec<ItemRequest> {
        let mut requests = Vec::new();
        if let Some(picked) = picked {
            requests.extend(self.finish_pick(tree, s, picked));
        }
        for message in messages {
            self.message(tree, s, message);
        }
        requests
    }

    pub fn message(&mut self, tree: &mut UiTree, s: &dyn InventorySource, message: &UiMessage) {
        if message.element != self.field || self.field.is_none() {
            return;
        }
        if message.message == MSG_STOP_DRAG_ITEM {
            let Some(copy) = message.item.filter(|_| message.data != 0) else {
                return;
            };
            if crate::item_display::is_coin_copy(tree, copy) {
                // `StartMoneyDrop`.
                self.do_money = self.start_find_target(tree) || self.do_money;
            } else {
                self.start_item_drop(tree, copy);
            }
        } else if message.message == tokenize("not_dragable") {
            self.start_3d_drag(tree, s);
        }
    }

    /// `gm3DItemsUI::StartItemDrop`: an item that is not worn and is in the player's own pack.
    fn start_item_drop(&mut self, tree: &mut UiTree, copy: usize) {
        let Some(dragged) = dragged_item(tree, copy) else {
            return;
        };
        if dragged.equipped || dragged.from != self.player {
            return;
        }
        if self.start_find_target(tree) {
            self.do_item = true;
            self.item = dragged.item;
            self.from_slot = dragged.old_index;
        }
    }

    /// `gm3DItemsUI::Start3DDrag`: the selection's image on the field's icon, dragable, and the
    /// drag started from it.
    fn start_3d_drag(&mut self, tree: &mut UiTree, s: &dyn InventorySource) {
        let (Some(icon), selection) = (self.icon, s.selection()) else {
            return;
        };
        if self.pick.is_some() || selection == 0 {
            return;
        }
        let Some(item) = s.item(selection) else {
            return;
        };
        if !item.icon_layers.is_empty() {
            tree.set_icon_desc(icon, &item.icon_layers);
        } else if item.icon != 0 {
            tree.set_image(icon, item.icon, 0);
        } else {
            return;
        }
        tree.set_bool(icon, tokenize("dragable"), true);
        let attributes = &mut tree.el_mut(icon).attributes;
        attributes.insert(
            tokenize("Token_ShortcutType"),
            AttributeValue::Id(0x4000_0002),
        );
        attributes.insert(
            tokenize("Token_ShortcutData"),
            AttributeValue::InstanceId(selection),
        );
        tree.start_drag(icon);
    }

    /// `FinishFind3DTarget` and the global loop's `MidItemDrop` / `MidMoneyDrop`: the object the
    /// selection render found under the pointer (`None`: nothing), and what the drop asks.
    pub fn finish_pick(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        picked: Option<u64>,
    ) -> Option<ItemRequest> {
        self.pick.take()?;
        // `FinishFind3DTarget`: the player itself is no target.
        self.target = picked.filter(|&p| p != self.player).unwrap_or(0);
        if self.do_item {
            self.do_item = false;
            return self.mid_item_drop(tree, s);
        }
        if std::mem::take(&mut self.do_money) {
            return self.mid_money_drop(tree, s);
        }
        None
    }

    /// `gm3DItemsUI::MidItemDrop`: a stack of more than one asks how many first.
    fn mid_item_drop(&mut self, tree: &mut UiTree, s: &dyn InventorySource) -> Option<ItemRequest> {
        let slot = s.item(self.item)?;
        if slot.stackable && 1 < slot.quantity {
            self.item_context = tree.request_stack_context();
            let amount = if slot.max_stack != 0 {
                slot.quantity.min(slot.max_stack)
            } else {
                slot.quantity
            };
            return Some(ItemRequest::DropSplit {
                context: self.item_context,
                item: self.item,
                amount,
            });
        }
        self.finish_item_drop(slot.quantity)
    }

    /// `gm3DItemsUI::MidMoneyDrop`: how much money, up to the most a coin stack holds.
    fn mid_money_drop(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
    ) -> Option<ItemRequest> {
        self.money_context = tree.request_stack_context();
        Some(ItemRequest::MoneyStack {
            context: self.money_context,
            target: 0,
            amount: s.money().min(s.coin_max_stack()),
        })
    }

    /// `gm3DItemsUI::FinishItemDrop`: the move to the target (the ground when none).
    fn finish_item_drop(&mut self, quantity: u32) -> Option<ItemRequest> {
        let request = ItemRequest::Move {
            item: self.item,
            from: self.player,
            from_slot: self.from_slot,
            target: self.target,
            target_slot: 0,
            quantity,
        };
        self.item = 0;
        self.target = 0;
        self.from_slot = 0;
        Some(request)
    }

    /// `gm3DItemsUI::RecvNotice_StackUIDone`: the dialog's answer to the drop it asked for.
    pub fn stack_ui_done(&mut self, context: u32, amount: u32) -> Option<ItemRequest> {
        if context != 0 && context == self.item_context {
            self.item_context = 0;
            return self.finish_item_drop(amount);
        }
        if context != 0 && context == self.money_context {
            self.money_context = 0;
            return Some(ItemRequest::DragFromMoneyBag {
                quantity: amount,
                to_slot: 0,
                to_container: self.target,
            });
        }
        None
    }

    /// `gm3DItemsUI::~gm3DItemsUI`: the stack dialogs it asked for, to close (item, money).
    pub fn contexts(&self) -> [u32; 2] {
        [self.item_context, self.money_context]
    }
}
