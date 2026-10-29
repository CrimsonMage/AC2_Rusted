//! `ItemDisplay` and `ContainerDisplay`: an item's slot and a container's slots.
//!
//! An `ItemDisplay` is an `ItemBackground` with its `ItemIcon` child: the icon shows the item's
//! icon (its icon description's layers, else its icon id), the background is `pending` while a
//! request on the item waits or the item is not known yet, the icon is `selected` while the item
//! is the selection and `dragable` unless pending; its tooltip is the item's name at its quantity.
//! A press on the icon selects the item; a double click uses it, a right click also opens the
//! examination panel.
//!
//! A `ContainerDisplay` fills a container element's `ContainerListBox` with as many empty
//! `ItemBackground`s as the container has slots (`num_slots`, else `GetMaxContainerSize`), then
//! puts each item of its segment into the slot at its place (`GetContents`, `AddItem`); the
//! container's notices add, remove and refresh items and mark requests (`UpdateContainer`).

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP, STATELESS};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// One item as its slot shows it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemSlot {
    /// `GetIconID`.
    pub icon: u32,
    /// The icon description's layers, bottom first (`GetTotalIconDesc`); empty for none.
    pub icon_layers: Vec<crate::IconLayer>,
    /// `GetPotentiallyPluralizedName` at the shown quantity, rendered.
    pub name: String,
    /// `GetQuantity`, `GetStackable`, `GetMaxStackSize`.
    pub quantity: u32,
    pub stackable: bool,
    pub max_stack: u32,
}

/// What the inventory panels read from the player.
pub trait InventorySource {
    /// The item, if the client knows it (`CObjectMaint::GetWeenieObject`).
    fn item(&self, item: u64) -> Option<ItemSlot>;
    /// `ID_Item_TT_WaitingForServer` when the UI table holds it (`StringInfo::IsReady`); none
    /// by default, and then the icon's `tooltip_entry` shows (`StartTooltip`).
    fn waiting_tooltip(&self) -> Option<String> {
        None
    }
    /// `GetName(quantity)` (`gmEntity` 0x10B) of a known item, rendered; none by default, and
    /// then the item's own name shows.
    fn item_name(&self, _item: u64, _quantity: u32) -> Option<String> {
        None
    }
    /// `gmCWInterface::GetContentsInfo`: the container's contents, and the first slot and slot
    /// count of segment `segment`.
    fn contents_info(&self, container: u64, segment: u32) -> (Vec<u64>, u32, u32);
    /// `gmCWInterface::GetMaxContainerSize`.
    fn max_container_size(&self, container: u64) -> u32;
    /// `gmCWInterface::GetSelection`.
    fn selection(&self) -> u64;
    /// `gmCWInterface::GetPreferredInventoryLocation`: where a drop on the paper doll's body
    /// equips the item (0: nowhere).
    fn preferred_location(&self, _item: u64) -> u32 {
        0
    }
    /// `gmCWInterface::HasDurability` / `GetDurabilityLevel` / `GetMaxDurabilityLevel`: the
    /// current and maximum durability, for an item that has durability.
    fn durability(&self, _item: u64) -> Option<(i32, i32)> {
        None
    }
    /// `ID_Inventory_TT_Durability` with `current` and `max`, as the text shown.
    fn durability_tooltip(&self, _current: i32, _max: i32) -> Option<String> {
        None
    }
    /// The player (`CObjectMaint::GetPlayerWeenieObject`'s id; 0 before the player is in the world).
    fn player(&self) -> u64 {
        0
    }
    /// The player's money (`CWRepository::GetInt(player, 3100)`, `gmMoneyBagUI`'s amount).
    fn money(&self) -> u32 {
        0
    }
    /// `gInventoryGlobals::CallIFun_GetCoinMaxStackSize`: the most coins a stack holds.
    fn coin_max_stack(&self) -> u32 {
        u32::MAX
    }
    /// `gmMoneyBagUI::HandleMoneyBagDrop`'s test on the item's type (its `GetInt(item, 20)`
    /// has the melting bits `0xF0081`).
    fn melts_to_money(&self, _item: u64) -> bool {
        false
    }
    /// `gmCWInterface::GetBypassGoldMeltingConfirmationDlg`: the option that skips the melting
    /// confirmation.
    fn bypass_melting_confirmation(&self) -> bool {
        false
    }
}

/// `ContainerDisplay::s_coinFieldElementName` (`CoinsField`): the money field a drag starts from.
pub(crate) const COINS_FIELD: &str = "CoinsField";

/// Whether the dragged copy is the coin field (`ContainerDisplay_DDCB` takes it without an item
/// type).
pub(crate) fn is_coin_copy(tree: &UiTree, copy: usize) -> bool {
    tree.get(copy).map(|el| el.id()) == Some(id(COINS_FIELD))
}

/// What a slot asks of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRequest {
    /// `SetSelection`.
    Select(u64),
    /// `SetSelection`, then `UseSelection`.
    Use(u64),
    /// `SetSelection`, then `CM_UI::SendNotice_OpenExaminationPanel`.
    Examine(u64),
    /// `gmCWInterface::MoveItem` (`ContainerDisplay::FinishListBoxDrop`).
    Move {
        item: u64,
        from: u64,
        from_slot: i32,
        target: u64,
        target_slot: i32,
        quantity: u32,
    },
    /// `gmCWInterface::UnEquipItem` into a slot of the player's pack.
    Unequip { item: u64, target_slot: i32 },
    /// `ContainerDisplay::HandleDragFrom` / `CM_UI::SendNotice_HandleDragFrom`: an element of
    /// another panel (`context`, its `Token_SourceContext`) dropped on the player's pack at
    /// `slot`; the panel whose context it is answers it (a purchase, a collection).
    DragFrom { item: u64, slot: u32, context: u32 },
    /// `CM_UI::SendNotice_InitStackUI`: how many of the stack `item` to move into `target`,
    /// `amount` the default and the most; the answer comes back with `context`.
    Split {
        context: u32,
        item: u64,
        target: u64,
        amount: u32,
    },
    /// `gmCWInterface::EquipItem` (`gmPaperDollUI::HandlePaperDollDrop`).
    Equip { item: u64, location: u32 },
    /// `CM_UI::SendNotice_InitStackUI` for the coin field dropped on a pack (`HandleDragFromMoneyBag`
    /// and the backpack's coin branch): the money stack's question, `amount` the default and the
    /// most (the money or the coin stack's maximum, whichever is less), `target` the container
    /// (0 on the backpack's pages). The answer comes back with `context`.
    MoneyStack {
        context: u32,
        target: u64,
        amount: u32,
    },
    /// `CM_UI::SendNotice_InitStackUI` for an item dropped on the money bag
    /// (`gmMoneyBagUI::HandleMoneyBagDrop`): how many of the stack `item` to melt, `from` the
    /// container it is in.
    MoneyBagSplit {
        context: u32,
        item: u64,
        from: u64,
        amount: u32,
    },
    /// `CM_UI::SendNotice_InitStackUI` for a stack dropped in the world
    /// (`gm3DItemsUI::MidItemDrop`): how many of `item` from the player's pack, `amount` the
    /// default and the most.
    DropSplit {
        context: u32,
        item: u64,
        amount: u32,
    },
    /// `gmCWInterface::DragToMoneyBag`: `quantity` of `item` from slot `from_slot` of
    /// `from_container` into the money bag.
    DragToMoneyBag {
        item: u64,
        quantity: u32,
        from_slot: i32,
        from_container: u64,
    },
    /// `gmCWInterface::DragFromMoneyBag`: `quantity` of money out to slot `to_slot` of
    /// `to_container` (0: the player's pack).
    DragFromMoneyBag {
        quantity: u32,
        to_slot: i32,
        to_container: u64,
    },
}

/// A drop waiting for the stack dialog (`m_itemToMove`, `m_moveFrom`, `m_oldIndex`,
/// `m_newIndex`, `m_stackSize`, `m_lbdContext`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingMove {
    item: u64,
    from: u64,
    old_index: u32,
    new_index: u32,
    stack_size: u32,
    context: u32,
}

/// A dragged item copy's attributes: the item (`Token_ShortcutData`), the container it came
/// from (`FromID`), whether it is worn (`Equipped`) and its slot (`Token_OldIndex`).
pub(crate) struct DraggedItem {
    pub item: u64,
    pub from: u64,
    pub equipped: bool,
    pub old_index: i32,
}

/// `gmPaperDollUI_DDCB` and `ContainerDisplay_DDCB`: only an item (`Token_ShortcutType`
/// 0x40000002) is dropped on the slots.
pub(crate) fn is_item_copy(tree: &UiTree, copy: usize) -> bool {
    tree.get(copy)
        .and_then(|el| el.attribute(id("Token_ShortcutType")))
        == Some(&crate::layout::AttributeValue::Id(0x4000_0002))
}

/// A dragged copy's `Token_SourceContext`: the context of the panel it was dragged from.
pub(crate) fn source_context(tree: &UiTree, copy: usize) -> Option<u32> {
    match tree.get(copy)?.attribute(id("SourceContext")) {
        Some(crate::layout::AttributeValue::Int(context)) => Some(*context as u32),
        _ => None,
    }
}

pub(crate) fn dragged_item(tree: &UiTree, copy: usize) -> Option<DraggedItem> {
    let el = tree.get(copy)?;
    let instance = |name: &str| match el.attribute(id(name)) {
        Some(crate::layout::AttributeValue::InstanceId(v)) => *v,
        _ => 0,
    };
    let old_index = match el.attribute(id("Token_OldIndex")) {
        Some(crate::layout::AttributeValue::Int(i)) => *i,
        _ => 0,
    };
    Some(DraggedItem {
        item: instance("Token_ShortcutData"),
        from: instance("FromID"),
        equipped: tree.bool_attribute(copy, id("Equipped")),
        old_index,
    })
}

/// `ItemDisplay`'s state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDisplay {
    pub background: usize,
    pub icon: usize,
    pub item: u64,
    pending: bool,
    draggable: bool,
    waiting: bool,
    /// `m_quantityExternal` / `m_forceQuantity`: the quantity the name is given at instead of
    /// the item's own (`SetQuantity`).
    force_quantity: Option<u32>,
}

impl ItemDisplay {
    /// `ItemDisplay::Init`: the background's `child` as the icon, then `SetItem`.
    pub fn init(
        tree: &mut UiTree,
        s: &dyn InventorySource,
        background: usize,
        child: u32,
        item: u64,
        from: u64,
        equipped: bool,
    ) -> Option<Self> {
        let icon = tree.child(background, child)?;
        // `FromID`: the container it is in; `Equipped`.
        tree.el_mut(icon).attributes.insert(
            id("FromID"),
            crate::layout::AttributeValue::InstanceId(from),
        );
        tree.set_bool(icon, id("Equipped"), equipped);
        // The icon carries the item as a shortcut (`Token_ShortcutType` 0x40000002,
        // `Token_ShortcutData`).
        tree.el_mut(icon).attributes.insert(
            id("Token_ShortcutType"),
            crate::layout::AttributeValue::Id(0x4000_0002),
        );
        tree.el_mut(icon).attributes.insert(
            id("Token_ShortcutData"),
            crate::layout::AttributeValue::InstanceId(item),
        );
        let mut display = Self {
            background,
            icon,
            item,
            pending: false,
            draggable: true,
            waiting: false,
            force_quantity: None,
        };
        display.update(tree, s);
        Some(display)
    }

    /// `ItemDisplay::Update`.
    pub fn update(&mut self, tree: &mut UiTree, s: &dyn InventorySource) {
        let mut pending = self.pending;
        let background_state;
        match s.item(self.item) {
            None => {
                // `ID_Item_TT_WaitingForServer`.
                pending = true;
                tree.clear_image(self.icon);
                background_state = id("pending");
                self.waiting = true;
                tree.set_tooltip(self.icon, s.waiting_tooltip());
            }
            Some(slot) => {
                // `ItemDisplay::Update`: the icon description when the item has one
                // (`SetIconDesc`), else its icon id.
                if slot.icon_layers.is_empty() {
                    tree.set_image(self.icon, slot.icon, 0);
                } else {
                    tree.set_icon_desc(self.icon, &slot.icon_layers);
                }
                self.waiting = false;
                background_state = if pending { id("pending") } else { STATELESS };
                // `GetName` at the forced quantity, else the item's own.
                let name = self
                    .force_quantity
                    .and_then(|quantity| s.item_name(self.item, quantity))
                    .unwrap_or(slot.name);
                tree.set_tooltip(self.icon, Some(name));
            }
        }
        tree.set_state(self.background, background_state);
        tree.set_bool(self.icon, id("dragable"), !pending && self.draggable);
        let state = if s.selection() == self.item {
            id("selected")
        } else {
            STATELESS
        };
        tree.set_state(self.icon, state);
    }

    /// `ItemDisplay::SetPending`.
    pub fn set_pending(&mut self, tree: &mut UiTree, s: &dyn InventorySource, pending: bool) {
        self.pending = pending;
        self.update(tree, s);
    }

    /// `ItemDisplay::SetQuantity`: the item is named at `quantity` (a trade's offered count).
    pub fn set_quantity(&mut self, tree: &mut UiTree, s: &dyn InventorySource, quantity: u32) {
        self.force_quantity = Some(quantity);
        self.update(tree, s);
    }

    /// `ItemDisplay::SetDraggable`: the icon can be dragged (unless it is pending).
    pub fn set_draggable(&mut self, tree: &mut UiTree, s: &dyn InventorySource, draggable: bool) {
        self.draggable = draggable;
        self.update(tree, s);
    }

    /// `ItemDisplay::SetShortcutType`: the icon's shortcut type; 0 is not an item, so the item
    /// drop callbacks refuse a drag of it (`gmSecureTradeUI::AddItem`).
    pub fn set_shortcut_type(&self, tree: &mut UiTree, shortcut_type: u32) {
        tree.el_mut(self.icon).attributes.insert(
            id("Token_ShortcutType"),
            crate::layout::AttributeValue::Id(shortcut_type),
        );
    }

    /// `ItemDisplay::RecvNotice_SelectionChanged`: the icon `selected` while it is the selection.
    pub fn selection_changed(&self, tree: &mut UiTree, old: u64, new: u64) {
        if new == self.item && old != self.item {
            tree.set_state(self.icon, id("selected"));
        } else if old == self.item && new != self.item {
            tree.set_state(self.icon, STATELESS);
        }
    }

    /// `ItemDisplay::ListenToMessage`: a press on the icon.
    pub fn message(&self, message: &UiMessage) -> Option<ItemRequest> {
        if message.element != Some(self.icon)
            || message.message != MSG_MOUSE_PRESS_TOP
            || self.waiting
        {
            return None;
        }
        match message.data as u32 {
            0x4000_0004 => Some(ItemRequest::Select(self.item)),
            0x4000_0007 => Some(ItemRequest::Use(self.item)),
            0x4000_0005 => Some(ItemRequest::Examine(self.item)),
            _ => None,
        }
    }
}

/// `ContainerDisplay`'s state.
#[derive(Debug, Clone)]
pub struct ContainerDisplay {
    layout: Arc<LayoutDesc>,
    pub container: u64,
    pub(crate) list: Option<usize>,
    max_size: u32,
    count: u32,
    first_slot: u32,
    segment: u32,
    /// `_itemList`, in slot order.
    pub items: Vec<ItemDisplay>,
    pending: Option<PendingMove>,
    /// `m_newIndex`: the slot the last coin drop pointed at (the backpack's coin branch reads it).
    pub(crate) drop_index: u32,
    /// `m_dfmbContext`: the coin drop waiting for its stack dialog (0: none).
    money_context: u32,
}

impl ContainerDisplay {
    /// `_count < _maxSize`: the first slot after the page's items (`m_firstSlotNumber +
    /// _count`), if the page has room (`gmBackpackUI::ListenToMessage`'s drop on a page's tab).
    pub(crate) fn end_slot(&self) -> Option<u32> {
        (self.count < self.max_size).then(|| self.first_slot + self.count)
    }

    /// `ContainerDisplay::Init`: the slots, then the segment's items; the slot count.
    pub fn init(
        tree: &mut UiTree,
        layout: &Arc<LayoutDesc>,
        s: &dyn InventorySource,
        container: u64,
        element: usize,
        first_slot: u32,
        segment: u32,
    ) -> Self {
        let mut display = Self {
            layout: Arc::clone(layout),
            container,
            list: tree.child(element, id("ContainerListBox")),
            max_size: 0,
            count: 0,
            first_slot,
            segment,
            items: Vec::new(),
            pending: None,
            drop_index: 0,
            money_context: 0,
        };
        let Some(list) = display.list else {
            return display;
        };
        // `ContainerDisplay::Init`: `ContainerDisplay_DDCB` on the list.
        tree.set_drag_drop_check(list, Some(crate::DragCheck::ContainerDrop));
        display.max_size = match tree.get(element).and_then(|e| e.attribute(id("num_slots"))) {
            Some(crate::layout::AttributeValue::Int(n)) => *n as u32,
            _ => s.max_container_size(container),
        };
        tree.list_multi_op_start(list);
        for _ in 0..display.max_size {
            if let Some(slot) = display.background(tree) {
                tree.list_add_item(list, slot);
            }
        }
        display.get_contents(tree, s);
        tree.list_multi_op_complete(list);
        display
    }

    /// The slot count (`_maxSize`).
    pub fn max_size(&self) -> u32 {
        self.max_size
    }

    pub fn first_slot(&self) -> u32 {
        self.first_slot
    }

    fn background(&self, tree: &mut UiTree) -> Option<usize> {
        let desc = self.layout.element(id("ItemBackground"))?.clone();
        tree.create_element(Arc::clone(&self.layout), desc, false)
    }

    /// `ContainerDisplay::GetContents`: the segment's items at their places.
    pub fn get_contents(&mut self, tree: &mut UiTree, s: &dyn InventorySource) {
        self.flush(tree);
        let (contents, start, size) = s.contents_info(self.container, self.segment);
        for (place, item) in contents.into_iter().enumerate() {
            let place = place as u32;
            if start <= place && place < start + size {
                self.add_item(tree, s, item, place - start);
            }
        }
    }

    /// `ContainerDisplay::AddItem`: an empty slot (or a new one past the slot count) at
    /// `location`, holding the item.
    pub fn add_item(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        item: u64,
        location: u32,
    ) {
        let Some(list) = self.list else {
            return;
        };
        let slot = if self.max_size < self.count + 1 {
            self.background(tree)
        } else {
            tree.list_remove_item(list, self.max_size as usize - 1)
        };
        let Some(slot) = slot else {
            return;
        };
        let location = (location as usize).min(tree.list_items(list).len());
        tree.list_insert_item(list, slot, location);
        if let Some(display) =
            ItemDisplay::init(tree, s, slot, id("ItemIcon"), item, self.container, false)
        {
            tree.set_int(
                display.icon,
                id("Token_OldIndex"),
                (self.first_slot + location as u32) as i32,
            );
            let at = location.min(self.items.len());
            self.items.insert(at, display);
            self.count += 1;
        }
    }

    /// `ContainerDisplay::FindItem`.
    pub fn find_item(&self, item: u64) -> Option<usize> {
        self.items.iter().position(|d| d.item == item)
    }

    /// `ContainerDisplay::RemoveItem`: the slot emptied and moved to the end (or dropped past the
    /// slot count).
    pub fn remove_item(&mut self, tree: &mut UiTree, item: u64) -> bool {
        let Some(list) = self.list else {
            return false;
        };
        let Some(index) = self.find_item(item) else {
            return false;
        };
        let display = self.items.remove(index);
        tree.set_state(display.background, STATELESS);
        tree.clear_image(display.icon);
        tree.set_tooltip(display.icon, None);
        tree.set_state(display.icon, STATELESS);
        let over = self.max_size < self.count;
        let Some(slot) = tree.list_remove_item(list, index) else {
            return false;
        };
        self.count -= 1;
        if over {
            tree.delete_element(slot);
            return true;
        }
        tree.list_insert_item(list, slot, self.max_size as usize - 1);
        true
    }

    /// `ContainerDisplay::Flush`: the items gone, the slots past the slot count dropped.
    pub fn flush(&mut self, tree: &mut UiTree) {
        for display in std::mem::take(&mut self.items) {
            tree.set_state(display.background, STATELESS);
            tree.clear_image(display.icon);
            tree.set_tooltip(display.icon, None);
            tree.set_state(display.icon, STATELESS);
        }
        if let Some(list) = self.list {
            while self.max_size < self.count {
                if let Some(slot) = tree.list_remove_item(list, 0) {
                    tree.delete_element(slot);
                }
                self.count -= 1;
            }
        }
        self.count = 0;
    }

    /// `ContainerDisplay::Cleanup`: the drops waiting for their stack dialogs dropped (their
    /// contexts returned, for `CM_UI::SendNotice_CloseStackUI`: the list drop's, then the coin's),
    /// the items and the slots gone.
    pub fn cleanup(&mut self, tree: &mut UiTree) -> Vec<u32> {
        let mut contexts: Vec<u32> = self.pending.take().map(|p| p.context).into_iter().collect();
        contexts.push(std::mem::take(&mut self.money_context));
        contexts.retain(|&c| c != 0);
        self.container = 0;
        self.flush(tree);
        if let Some(list) = self.list.take() {
            tree.list_flush(list);
        }
        self.max_size = 0;
        contexts
    }

    /// `ContainerDisplay::RecvNotice_UpdateContainer`.
    pub fn update_container(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        code: u32,
        item: u64,
        slot: i32,
    ) {
        match code {
            0x4000_0001 => self.get_contents(tree, s),
            0x4000_0002 => {
                let (_, start, size) = s.contents_info(self.container, self.segment);
                let slot = slot as u32;
                if start <= slot && slot < start + size {
                    self.add_item(tree, s, item, slot - start);
                }
            }
            0x4000_0003 => {
                self.remove_item(tree, item);
            }
            0x4000_0004 | 0x4000_0005 => {
                if let Some(i) = self.find_item(item) {
                    let mut display = self.items[i].clone();
                    display.set_pending(tree, s, code == 0x4000_0005);
                    self.items[i] = display;
                }
            }
            _ => {}
        }
    }

    /// `RecvNotice_ItemUpdate` / `LocalVisualDescUpdate` / `CreateObject` of one item.
    pub fn item_changed(&mut self, tree: &mut UiTree, s: &dyn InventorySource, item: u64) {
        for display in self.items.iter_mut().filter(|d| d.item == item) {
            display.update(tree, s);
        }
    }

    /// The selection moved.
    pub fn selection_changed(&self, tree: &mut UiTree, old: u64, new: u64) {
        for display in &self.items {
            display.selection_changed(tree, old, new);
        }
    }

    /// `ContainerDisplay::GetNewIndex`: the slot under the pointer (at most the segment's size),
    /// numbered from the display's first slot.
    fn new_index(&self, tree: &UiTree, s: &dyn InventorySource) -> u32 {
        let Some(list) = self.list else {
            return 0;
        };
        let mut index = 0;
        if let Some(item) = tree.list_item_under_mouse(list) {
            let (_, _, size) = s.contents_info(self.container, self.segment);
            let at = tree.list_what_num(list, item).map_or(-1, |i| i as i64);
            index = at.min(i64::from(size)).max(0) as u32;
        }
        self.first_slot + index
    }

    /// `ContainerDisplay::HandleListBoxDrop`: an item from a container moves to the slot under
    /// the pointer; an equipped item is taken off into it. A stack of more than one asks how many
    /// first (`gmStackUI`, default and most the stack's size up to its maximum).
    fn list_box_drop(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        copy: usize,
    ) -> Option<ItemRequest> {
        let DraggedItem {
            item,
            from,
            equipped,
            old_index,
        } = dragged_item(tree, copy)?;
        let new_index = self.new_index(tree, s);
        let slot = s.item(item)?;
        if equipped {
            return Some(ItemRequest::Unequip {
                item,
                target_slot: new_index as i32,
            });
        }
        let old_index = old_index as u32;
        let quantity = slot.quantity;
        if from == self.container && old_index == new_index {
            return None;
        }
        let stack_size = if slot.max_stack != 0 {
            quantity.min(slot.max_stack)
        } else {
            quantity
        };
        let mut pending = PendingMove {
            item,
            from,
            old_index,
            new_index,
            stack_size,
            context: 0,
        };
        if slot.stackable && 1 < quantity {
            pending.context = tree.request_stack_context();
            self.pending = Some(pending);
            return Some(ItemRequest::Split {
                context: pending.context,
                item,
                target: self.container,
                amount: stack_size,
            });
        }
        self.finish_list_box_drop(s, pending, quantity)
    }

    /// `ContainerDisplay::FinishListBoxDrop`: the move of `quantity` (one dropped past the
    /// segment's end from this container, a whole stack, lands on the last slot).
    fn finish_list_box_drop(
        &self,
        s: &dyn InventorySource,
        mut pending: PendingMove,
        quantity: u32,
    ) -> Option<ItemRequest> {
        if pending.from == self.container && pending.old_index == pending.new_index {
            return None;
        }
        if pending.from == self.container {
            let (_, _, size) = s.contents_info(self.container, self.segment);
            if size <= pending.new_index.wrapping_sub(self.first_slot)
                && pending.stack_size == quantity
            {
                pending.new_index = pending.new_index.wrapping_sub(1);
            }
        }
        Some(ItemRequest::Move {
            item: pending.item,
            from: pending.from,
            from_slot: pending.old_index as i32,
            target: self.container,
            target_slot: pending.new_index as i32,
            quantity,
        })
    }

    /// `ContainerDisplay::RecvNotice_StackUIDone`: the stack dialog's answer to this display's
    /// drop finishes it.
    pub fn stack_ui_done(
        &mut self,
        s: &dyn InventorySource,
        context: u32,
        amount: u32,
    ) -> Option<ItemRequest> {
        if context != 0 && context == self.money_context {
            // `ContainerDisplay::RecvNotice_StackUIDone`'s coin branch: `DragFromMoneyBag`.
            self.money_context = 0;
            return Some(ItemRequest::DragFromMoneyBag {
                quantity: amount,
                to_slot: self.drop_index as i32,
                to_container: self.container,
            });
        }
        let pending = self.pending.filter(|p| p.context == context)?;
        self.pending = None;
        self.finish_list_box_drop(s, pending, amount)
    }

    /// `ContainerDisplay::HandleDragFromMoneyBag`: the coin field dropped on the list. The slot
    /// under the pointer is kept (`m_newIndex`); the stack dialog asks how much, with the money
    /// and the coin stack's maximum as the limits (the less of them is the default and the most).
    fn drag_from_money_bag(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
    ) -> Option<ItemRequest> {
        self.drop_index = self.new_index(tree, s);
        let context = tree.request_stack_context();
        self.money_context = context;
        let amount = s.money().min(s.coin_max_stack());
        Some(ItemRequest::MoneyStack {
            context,
            target: self.container,
            amount,
        })
    }

    /// `ContainerDisplay::ListenToMessage` (a drop on its list) and the slots' presses.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        message: &UiMessage,
    ) -> Option<ItemRequest> {
        if message.message == crate::drag::MSG_STOP_DRAG_ITEM
            && message.element.is_some()
            && message.element == self.list
            && message.data != 0
        {
            let copy = message.item?;
            // `ContainerDisplay::ListenToMessage`: the coin field, else `ContainerDisplay_DDCB`
            // takes only items.
            if is_coin_copy(tree, copy) {
                return self.drag_from_money_bag(tree, s);
            }
            // A copy from another panel's context goes back to that panel when it lands on the
            // player's own pack (`HandleDragFrom`); elsewhere it is dropped.
            if let Some(context) = source_context(tree, copy) {
                if self.container != s.player() {
                    return None;
                }
                let item = dragged_item(tree, copy)?.item;
                let slot = self.new_index(tree, s);
                return Some(ItemRequest::DragFrom {
                    item,
                    slot,
                    context,
                });
            }
            if !is_item_copy(tree, copy) {
                return None;
            }
            return self.list_box_drop(tree, s, copy);
        }
        self.items.iter().find_map(|d| d.message(message))
    }
}
