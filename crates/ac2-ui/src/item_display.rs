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
    /// The icon description's layer images, bottom first (`GetTotalIconDesc`); empty for none.
    pub icon_layers: Vec<u32>,
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
    /// `gmCWInterface::GetContentsInfo`: the container's contents, and the first slot and slot
    /// count of segment `segment`.
    fn contents_info(&self, container: u64, segment: u32) -> (Vec<u64>, u32, u32);
    /// `gmCWInterface::GetMaxContainerSize`.
    fn max_container_size(&self, container: u64) -> u32;
    /// `gmCWInterface::GetSelection`.
    fn selection(&self) -> u64;
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
                tree.set_tooltip(self.icon, None);
            }
            Some(slot) => {
                match slot.icon_layers.first() {
                    Some(&layer) => tree.set_image(self.icon, layer, 0),
                    None => tree.set_image(self.icon, slot.icon, 0),
                }
                self.waiting = false;
                background_state = if pending { id("pending") } else { STATELESS };
                tree.set_tooltip(self.icon, Some(slot.name));
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
    list: Option<usize>,
    max_size: u32,
    count: u32,
    first_slot: u32,
    segment: u32,
    /// `_itemList`, in slot order.
    pub items: Vec<ItemDisplay>,
}

impl ContainerDisplay {
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
        };
        let Some(list) = display.list else {
            return display;
        };
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

    /// `ContainerDisplay::HandleListBoxDrop` and `FinishListBoxDrop`: an item from a container
    /// moves to the slot under the pointer (one dropped past the segment's end from this
    /// container lands on the last slot); an equipped item is taken off into it. The stack
    /// dialog for part of a stack (`gmStackUI`) is not modelled: the whole stack moves.
    fn list_box_drop(
        &self,
        tree: &UiTree,
        s: &dyn InventorySource,
        copy: usize,
    ) -> Option<ItemRequest> {
        let el = tree.get(copy)?;
        let instance = |name: &str| match el.attribute(id(name)) {
            Some(crate::layout::AttributeValue::InstanceId(v)) => *v,
            _ => 0,
        };
        let item = instance("Token_ShortcutData");
        let from = instance("FromID");
        let equipped = tree.bool_attribute(copy, id("Equipped"));
        let mut new_index = self.new_index(tree, s);
        let slot = s.item(item)?;
        if equipped {
            return Some(ItemRequest::Unequip {
                item,
                target_slot: new_index as i32,
            });
        }
        let old_index = match el.attribute(id("Token_OldIndex")) {
            Some(crate::layout::AttributeValue::Int(i)) => *i as u32,
            _ => 0,
        };
        let quantity = slot.quantity;
        if from == self.container && old_index == new_index {
            return None;
        }
        let stack_size = if slot.max_stack != 0 {
            quantity.min(slot.max_stack)
        } else {
            quantity
        };
        if from == self.container {
            let (_, _, size) = s.contents_info(self.container, self.segment);
            if size <= new_index - self.first_slot && stack_size == quantity && new_index > 0 {
                new_index -= 1;
            }
        }
        Some(ItemRequest::Move {
            item,
            from,
            from_slot: old_index as i32,
            target: self.container,
            target_slot: new_index as i32,
            quantity,
        })
    }

    /// `ContainerDisplay::ListenToMessage` (a drop on its list) and the slots' presses.
    pub fn message(
        &self,
        tree: &UiTree,
        s: &dyn InventorySource,
        message: &UiMessage,
    ) -> Option<ItemRequest> {
        if message.message == crate::drag::MSG_STOP_DRAG_ITEM
            && message.element.is_some()
            && message.element == self.list
            && message.data != 0
        {
            // `ContainerDisplay_DDCB`: only items.
            let copy = message.item?;
            let kind = tree.get(copy)?.attribute(id("Token_ShortcutType")).cloned();
            if kind != Some(crate::layout::AttributeValue::Id(0x4000_0002)) {
                return None;
            }
            return self.list_box_drop(tree, s, copy);
        }
        self.items.iter().find_map(|d| d.message(message))
    }
}
