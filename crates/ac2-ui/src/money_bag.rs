//! `gmMoneyBagUI`: the inventory panel's money field (`MoneyBag`, whose `AmountLabel` shows the
//! player's money as `ID_Inventory_Amount`) and the drops onto it. An item dropped on the bag from
//! the player is melted into money (`gmCWInterface::DragToMoneyBag`) when its type melts; a stack
//! first asks how many (`CM_UI::SendNotice_InitStackUI`, the less of the stack and its maximum);
//! then a confirmation asks (`ID_Inventory_DragToMoney`) unless shift is held or the bypass option
//! is set (`IsConfirmationDialogAppropriate`). An equipped item is not melted.
//!
//! The bag's drop callback (`gmMoneyBagUI_DDCB`) is [`DragCheck::MoneyBagDrop`]: the tree asks it
//! when the drop lands, so a refused item goes back to where it came from.
//!
//! Not modelled: the `not_dragable` branch (retail's `m_curAmount` is only ever 0).

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::drag::DragCheck;
use crate::element::UiTree;
use crate::item_display::{dragged_item, is_item_copy, InventorySource, ItemRequest};
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::tokenize;
use crate::ui_string::{UiString, UiStrings};

/// The item drop the money bag is working on (`m_curItemID`, `m_curQuantity`, `m_curOldIndex`,
/// `m_curFromID`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct MoneyDrop {
    item: u64,
    quantity: u32,
    old_index: i32,
    from: u64,
}

/// `gmMoneyBagUI`'s state.
#[derive(Debug, Clone, Default)]
pub struct MoneyBagUi {
    /// `m_moneyBag` and `m_amountLabel`.
    bag: Option<usize>,
    label: Option<usize>,
    /// The money shown (`ID_Inventory_Amount`'s `value`).
    money: u32,
    drop: MoneyDrop,
    /// `m_splitContext`: the stack dialog asking how many to melt (0: none).
    split_context: u32,
    /// `m_dragToContext`: the confirmation dialog for the melt (0: none).
    confirm_context: u32,
}

impl MoneyBagUi {
    /// `gmMoneyBagUI::gmMoneyBagUI`: the `MoneyBag` element and its `AmountLabel`, found in the
    /// layout's roots (the inventory panel is one of them).
    pub fn create(tree: &mut UiTree) -> Self {
        let bag = crate::backpack::find(tree, "MoneyBag");
        if let Some(bag) = bag {
            tree.set_drag_drop_check(bag, Some(DragCheck::MoneyBagDrop));
        }
        let label = bag.and_then(|b| tree.child_recursive(b, tokenize("AmountLabel")));
        Self {
            bag,
            label,
            ..Self::default()
        }
    }

    /// `gmMoneyBagUI_DDCB`'s inventory part for the dragged `copy`: an item the player has that
    /// is not worn (`DoIHaveItem`, `IsOnInventory`). The HUD hands it to the tree every frame.
    pub fn drag_accepts(tree: &UiTree, s: &dyn InventorySource, copy: usize) -> bool {
        dragged_item(tree, copy)
            .is_some_and(|d| s.has_item(d.item) && !s.is_on_inventory(d.item) && !d.equipped)
    }

    /// The money the bag shows.
    pub fn money(&self) -> u32 {
        self.money
    }

    /// `gmMoneyBagUI::RecvNotice_UpdateMoney`: the amount label reads `ID_Inventory_Amount`.
    pub fn set_money(&mut self, tree: &mut UiTree, strings: &dyn UiStrings, money: u32) {
        self.money = money;
        if let Some(label) = self.label {
            let text = UiString::ui(
                tokenize("ID_Inventory_Amount"),
                vec![(tokenize("value"), SkillVar::UInt(u64::from(money)))],
            );
            tree.set_text(label, &strings.render_string(&text));
        }
    }

    /// `gmMoneyBagUI::IsConfirmationDialogAppropriate`: no confirmation with the bypass option
    /// set or shift held.
    fn confirmation_appropriate(&self, s: &dyn InventorySource, shift: bool) -> bool {
        !s.bypass_melting_confirmation() && !shift
    }

    /// `gmMoneyBagUI::MakeConfirmationDialog`: `ID_Inventory_DragToMoney` with the item's name.
    fn make_confirmation(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        strings: &dyn UiStrings,
        dialogs: &mut DialogFactory,
    ) {
        let Some(item) = s.item(self.drop.item) else {
            return;
        };
        let text = UiString::ui(
            tokenize("ID_Inventory_DragToMoney"),
            vec![(tokenize("name"), SkillVar::Text(item.name))],
        );
        let text = strings.render_string(&text);
        self.confirm_context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
    }

    /// `gmMoneyBagUI::DoDragTo`: the melt of the drop when `melt` (`DragToMoneyBag` with the
    /// drop's item, quantity, slot and container), then the drop is forgotten.
    fn do_drag_to(&mut self, melt: bool) -> Option<ItemRequest> {
        let drop = std::mem::take(&mut self.drop);
        self.confirm_context = 0;
        melt.then_some(ItemRequest::DragToMoneyBag {
            item: drop.item,
            quantity: drop.quantity,
            from_slot: drop.old_index,
            from_container: drop.from,
        })
    }

    /// `gmMoneyBagUI::HandleMoneyBagDrop`: an item dropped on the bag. `gmMoneyBagUI_DDCB` takes
    /// only an item (`Token_ShortcutType` 0x40000002) the player has (`DoIHaveItem`: a pack's,
    /// side packs too) that is not worn (`IsOnInventory`); a melting item goes straight in; any
    /// other stack asks how many, then the confirmation asks (or the melt goes at once).
    #[allow(clippy::too_many_arguments)]
    fn handle_drop(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        strings: &dyn UiStrings,
        dialogs: &mut DialogFactory,
        copy: usize,
        shift: bool,
    ) -> Option<ItemRequest> {
        let dragged = dragged_item(tree, copy)?;
        // The copy's `Equipped` flag (a paper doll slot's item) is the worn state as the UI
        // knows it.
        if !is_item_copy(tree, copy) || dragged.equipped {
            return None;
        }
        self.melt(
            tree,
            s,
            strings,
            dialogs,
            (dragged.item, dragged.from, dragged.old_index),
            shift,
        )
    }

    /// `HandleMoneyBagDrop` after the drop's checks: `item` in slot `old_index` of container
    /// `from` melted, as dropping it on the bag does (also the modern drawer's Melt,
    /// `modern_item_ring`). Only an item the player has (`DoIHaveItem`) that is not worn
    /// (`IsOnInventory`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn melt(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        strings: &dyn UiStrings,
        dialogs: &mut DialogFactory,
        (item, from, old_index): (u64, u64, i32),
        shift: bool,
    ) -> Option<ItemRequest> {
        if !s.has_item(item) || s.is_on_inventory(item) {
            return None;
        }
        let dragged = crate::item_display::DraggedItem {
            item,
            from,
            old_index,
            equipped: false,
        };
        let slot = s.item(dragged.item)?;
        let old_index = dragged.old_index;
        if s.melts_to_money(dragged.item) {
            return Some(ItemRequest::DragToMoneyBag {
                item: dragged.item,
                quantity: slot.quantity,
                from_slot: old_index,
                from_container: dragged.from,
            });
        }
        // The previous drop's confirmation closes (`CloseDialog`) and its drop is forgotten.
        if self.confirm_context != 0 {
            dialogs.close_dialog(tree, self.confirm_context);
            self.confirm_context = 0;
        }
        self.drop = MoneyDrop {
            item: dragged.item,
            quantity: slot.quantity,
            old_index,
            from: dragged.from,
        };
        if slot.stackable && 1 < slot.quantity {
            let context = tree.request_stack_context();
            self.split_context = context;
            return Some(ItemRequest::MoneyBagSplit {
                context,
                item: dragged.item,
                from: dragged.from,
                amount: slot.quantity.min(slot.max_stack),
            });
        }
        if self.confirmation_appropriate(s, shift) {
            self.make_confirmation(tree, s, strings, dialogs);
            return None;
        }
        self.do_drag_to(true)
    }

    /// `gmMoneyBagUI::ListenToMessage`: a drop on the bag (`stop_drag_item` with an item copy).
    /// `shift` is whether shift is held.
    #[allow(clippy::too_many_arguments)]
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        strings: &dyn UiStrings,
        dialogs: &mut DialogFactory,
        message: &UiMessage,
        shift: bool,
    ) -> Option<ItemRequest> {
        if message.message != crate::drag::MSG_STOP_DRAG_ITEM
            || message.element.is_none()
            || message.element != self.bag
            || message.data == 0
        {
            return None;
        }
        // `ContainerDisplay`-style `DDCB`: only an item is dropped on the bag.
        let copy = message.item?;
        if !is_item_copy(tree, copy) {
            return None;
        }
        self.handle_drop(tree, s, strings, dialogs, copy, shift)
    }

    /// `gmMoneyBagUI::RecvNotice_StackUIDone`: the amount of the split answers the drop. Then the
    /// confirmation asks, or the melt goes at once.
    #[allow(clippy::too_many_arguments)]
    pub fn stack_ui_done(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        strings: &dyn UiStrings,
        dialogs: &mut DialogFactory,
        context: u32,
        amount: u32,
        shift: bool,
    ) -> Option<ItemRequest> {
        if context == 0 || context != self.split_context {
            return None;
        }
        self.drop.quantity = amount;
        if self.confirmation_appropriate(s, shift) {
            self.make_confirmation(tree, s, strings, dialogs);
            return None;
        }
        self.do_drag_to(true)
    }

    /// `gmMoneyBagUI::RecvNotice_CloseConfirmationDialog`: the answer to the melt's confirmation
    /// (`yes` melts; the drop is forgotten either way).
    pub fn take_outcome(&mut self, outcomes: &[DialogOutcome]) -> Option<ItemRequest> {
        outcomes.iter().find_map(|outcome| match *outcome {
            DialogOutcome::Confirmation { context, yes }
                if context != 0 && context == self.confirm_context =>
            {
                Some(self.do_drag_to(yes))
            }
            _ => None,
        })?
    }
}
