//! `gmPaperDollUI`: the inventory panel's paper doll. Its own root, `PaperDollViewport`, is the
//! player's avatar in a 3D scene (the client renders it); the equip slots around it, one per
//! [`EQUIP_LOCATIONS`] bit (`<Name>_Slot`, the names of `InitSlots`' `<Name>_InvLoc` table), each
//! holding the worn item's [`ItemDisplay`].
//!
//! The slots follow the player's equipment: `GetInventory` puts every worn item into its slot
//! (`InqSimpleInvByLocTable`, here the frame's `inventory.equipped`), then
//! `RecvNotice_UpdateInventory` adds and removes items and marks requests begun or failed.

use std::sync::Arc;

use ac2_weenie::EQUIP_LOCATIONS;

use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::element::UiTree;
use crate::item_display::{dragged_item, is_item_copy, InventorySource, ItemDisplay, ItemRequest};
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP, STATELESS};

/// `gmPaperDollUI::RecvNotice_UpdateInventory`'s codes.
const INITIAL_LIST: u32 = 0x4000_0001;
const ADD_ITEM: u32 = 0x4000_0002;
const REMOVE_ITEM: u32 = 0x4000_0003;
const REQUEST_FAILED: u32 = 0x4000_0004;
const BEGIN_REQUEST: u32 = 0x4000_0005;

/// The element names of an equip location (`InitSlots`' `<Name>_InvLoc` bits).
fn location_name(location: u32) -> Option<&'static str> {
    Some(match location {
        0x1 => "Head",
        0x2 => "Torso",
        0x4 => "Back",
        0x8 => "Legs",
        0x10 => "Feet",
        0x20 => "Hands",
        0x40 => "PrimaryHand",
        0x80 => "SecondaryHand",
        0x100 => "PrimaryRing",
        0x200 => "SecondaryRing",
        0x400 => "Necklace",
        _ => return None,
    })
}

/// `gmPaperDollUI::MeshTypeToInvLoc`: the location a body part's mesh type stands for (0: none).
pub fn mesh_type_to_inv_loc(mesh_type: u32) -> u32 {
    match mesh_type {
        0x8100_0001 => 0x10,
        0x8100_0002 => 0x20,
        0x8100_0003 | 0x8100_0004 | 0x8100_001B => 0x1,
        0x8100_001C => 0x2,
        0x8100_001D => 0x8,
        _ => 0,
    }
}

/// One location's slot element (`<Name>_Slot`), if the layout has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    location: u32,
    slot: Option<usize>,
    durability: Option<usize>,
}

/// `gmPaperDollUI`'s state.
#[derive(Debug, Clone)]
pub struct PaperDollUi {
    viewport: Option<usize>,
    player: u64,
    /// `_paperDollTokens`: each location's slot.
    slots: Vec<Slot>,
    /// `_paperDollItems` / `_paperDollLocs`: the worn items by location.
    items: Vec<(u32, ItemDisplay)>,
    /// `_dollClick`: a press on the doll waits for the scene's pick.
    doll_click: bool,
    /// The point the pick waits at (`UIScene::StartSelection`).
    pick: Option<(i32, i32)>,
    /// `_lastLoc`: the location the last pick or press on the doll was on.
    last_location: u32,
}

fn find(tree: &UiTree, name: &str) -> Option<usize> {
    let id = tokenize(name);
    tree.roots().iter().find_map(|&r| {
        if tree.get(r).is_some_and(|e| e.id() == id) {
            Some(r)
        } else {
            tree.child_recursive(r, id)
        }
    })
}

impl PaperDollUi {
    /// `gmPaperDollUI::gmPaperDollUI`: the viewport root, `InitSlots`, then `GetInventory`.
    pub fn create(
        tree: &mut UiTree,
        layout: &Arc<LayoutDesc>,
        s: &dyn InventorySource,
        player: u64,
        equipped: &[(u32, u64)],
    ) -> Self {
        let viewport = tree.create_root(Arc::clone(layout), tokenize("PaperDollViewport"));
        let mut ui = Self {
            viewport,
            player,
            slots: Vec::new(),
            items: Vec::new(),
            doll_click: false,
            pick: None,
            last_location: 0,
        };
        ui.init_slots(tree);
        ui.get_inventory(tree, s, equipped);
        ui
    }

    /// The doll's viewport root.
    pub fn viewport(&self) -> Option<usize> {
        self.viewport
    }

    /// `gmPaperDollUI::InitSlots`: each location's slot element and durability field, the
    /// fields hidden.
    fn init_slots(&mut self, tree: &mut UiTree) {
        self.slots = EQUIP_LOCATIONS
            .iter()
            .filter_map(|&location| {
                let name = location_name(location)?;
                Some(Slot {
                    location,
                    slot: find(tree, &format!("{name}_Slot")),
                    durability: find(tree, &format!("{name}_Durability")),
                })
            })
            .collect();
        for durability in self.slots.iter().filter_map(|s| s.durability) {
            tree.set_visible(durability, false);
        }
    }

    fn slot(&self, location: u32) -> Option<Slot> {
        self.slots.iter().copied().find(|s| s.location == location)
    }

    /// `gmPaperDollUI::GetInventory`: each worn item into its slot. The list is the whole
    /// equipment, so an item it no longer has comes off too (retail only adds: the double
    /// answers an unequip with the list alone, not with `RemoveItem`).
    pub fn get_inventory(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        equipped: &[(u32, u64)],
    ) {
        let stale: Vec<(u32, u64)> = self
            .items
            .iter()
            .filter(|(at, d)| !equipped.contains(&(*at, d.item)))
            .map(|(at, d)| (*at, d.item))
            .collect();
        for (at, item) in stale {
            self.remove_item(tree, s, item, at);
        }
        for &(location, item) in equipped {
            if self.item_at(location) != Some(item) {
                self.add_item(tree, s, item, location);
            }
        }
    }

    /// `gmPaperDollUI::AddItem`: the item's display in the location's slot, worn by the player.
    pub fn add_item(&mut self, tree: &mut UiTree, s: &dyn InventorySource, item: u64, at: u32) {
        let Some(background) = self.slot(at).and_then(|slot| slot.slot) else {
            return;
        };
        let icon = tokenize("ItemIcon");
        let Some(display) = ItemDisplay::init(tree, s, background, icon, item, self.player, true)
        else {
            return;
        };
        self.items.retain(|(location, _)| *location != at);
        self.items.push((at, display));
        // `AddItem`: the durability meter shows for an item that has durability.
        if let (Some(slot), Some(durability)) = (self.slot(at), s.durability(item)) {
            self.set_durability(tree, s, slot, Some(durability));
        }
    }

    /// `gmPaperDollUI::RemoveItem`: the location's item comes off its slot and its durability
    /// meter goes.
    pub fn remove_item(&mut self, tree: &mut UiTree, s: &dyn InventorySource, item: u64, at: u32) {
        let Some(index) = self
            .items
            .iter()
            .position(|(location, d)| *location == at && d.item == item)
        else {
            return;
        };
        let (_, display) = self.items.remove(index);
        tree.clear_image(display.icon);
        tree.set_state(display.icon, STATELESS);
        tree.set_state(display.background, STATELESS);
        if let Some(slot) = self.slot(at) {
            self.set_durability(tree, s, slot, None);
        }
    }

    /// `gmPaperDollUI::UpdateDurabilityMeter` / `SetDurabilityTooltip` for a slot: the meter at
    /// `1 - current / max` (0 without a maximum) with its tooltip, the field shown when the item
    /// has durability (`None` clears it, as `RemoveItem` and `RecvNotice_ItemUpdate` do).
    fn set_durability(
        &self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        slot: Slot,
        durability: Option<(i32, i32)>,
    ) {
        let Some(field) = slot.durability else {
            return;
        };
        let (current, max) = durability.unwrap_or((0, 0));
        // `percentage = 1.0 - (float)cur / (float)max`: the division in 53-bit, rounded at the
        // `float` store.
        let percentage = if max != 0 {
            (1.0 - f64::from(current as f32) / f64::from(max as f32)) as f32
        } else {
            0.0
        };
        tree.set_tooltip(field, s.durability_tooltip(current, max));
        if let Some(meter) = tree.child_recursive(field, tokenize("DurabilityMeter")) {
            tree.set_float(meter, tokenize("position"), percentage);
        }
        tree.set_visible(field, durability.is_some());
    }

    /// `gmPaperDollUI::RecvNotice_UpdateInventory`.
    pub fn update_inventory(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        code: u32,
        item: u64,
        location: u32,
        equipped: &[(u32, u64)],
    ) {
        match code {
            INITIAL_LIST => self.get_inventory(tree, s, equipped),
            ADD_ITEM => self.add_item(tree, s, item, location),
            REMOVE_ITEM => self.remove_item(tree, s, item, location),
            REQUEST_FAILED | BEGIN_REQUEST => {
                if let Some((_, display)) = self
                    .items
                    .iter_mut()
                    .find(|(at, d)| *at == location && d.item == item)
                {
                    display.set_pending(tree, s, code == BEGIN_REQUEST);
                }
            }
            _ => {}
        }
    }

    /// `gmPaperDollUI::RecvNotice_ItemUpdate` of a worn item: its display and durability are
    /// refreshed (the meter hides for an item without durability).
    pub fn item_changed(&mut self, tree: &mut UiTree, s: &dyn InventorySource, item: u64) {
        let worn: Vec<u32> = self
            .items
            .iter()
            .filter(|(_, d)| d.item == item)
            .map(|(at, _)| *at)
            .collect();
        for (_, display) in self.items.iter_mut().filter(|(_, d)| d.item == item) {
            display.update(tree, s);
        }
        for at in worn {
            if let Some(slot) = self.slot(at) {
                self.set_durability(tree, s, slot, s.durability(item));
            }
        }
    }

    /// `ItemDisplay::RecvNotice_SelectionChanged` of the slots.
    pub fn selection_changed(&self, tree: &mut UiTree, old: u64, new: u64) {
        for (_, display) in &self.items {
            display.selection_changed(tree, old, new);
        }
    }

    /// The worn item at `location`, if any.
    pub fn item_at(&self, location: u32) -> Option<u64> {
        self.items
            .iter()
            .find(|(at, _)| *at == location)
            .map(|(_, d)| d.item)
    }

    /// `gmPaperDollUI::ListenToMessage`: a press on the doll starts a pick
    /// (`HandlePaperDollClick`); a press on the held item is dragged from its slot
    /// (`HandlePaperDollDrag`); an item dropped on the doll or its slots (`HandlePaperDollDrop`).
    /// The worn items' presses are their `ItemDisplay`s'.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        message: &UiMessage,
    ) -> Option<ItemRequest> {
        let on_doll = message.element.is_some() && message.element == self.viewport;
        if on_doll && message.message == MSG_MOUSE_PRESS_TOP && !self.doll_click {
            self.doll_click = true;
            self.last_location = 0;
            self.pick = Some(tree.pointer());
            return None;
        }
        if on_doll && message.message == tokenize("not_dragable") && self.last_location != 0 {
            let icon = self
                .items
                .iter()
                .find(|(at, _)| *at == self.last_location)
                .map(|(_, d)| d.icon);
            if let Some(icon) = icon {
                tree.start_drag(icon);
            }
        }
        if message.message == MSG_STOP_DRAG_ITEM && message.data == 1 {
            let target = message.element?;
            return self.drop(tree, s, target, message.item?);
        }
        self.items.iter().find_map(|(_, d)| d.message(message))
    }

    /// Where a press on the doll waits for the scene's pick (`UIScene::StartSelection`'s point),
    /// until [`Self::finish_pick`].
    pub fn wants_pick(&self) -> Option<(i32, i32)> {
        self.pick.filter(|_| self.doll_click)
    }

    /// `gmPaperDollUI::FinishPaperDollClick` with the scene's answer: the mesh type of the
    /// avatar's part under the point (`None` when the avatar was not hit). With the left button
    /// still down, the item worn at that part's location becomes the selection.
    pub fn finish_pick(&mut self, tree: &UiTree, mesh_type: Option<u32>) -> Option<ItemRequest> {
        self.doll_click = false;
        self.pick = None;
        let mesh_type = mesh_type?;
        if !tree.drag.left_down {
            return None;
        }
        self.last_location = mesh_type_to_inv_loc(mesh_type);
        if self.last_location == 0 {
            return None;
        }
        self.item_at(self.last_location).map(ItemRequest::Select)
    }

    /// `gmPaperDollUI::HandlePaperDollDrop`: the dropped item is equipped at the slot's location,
    /// or on the doll at its preferred one. One from another container comes into the player
    /// first (`MoveItem` into the player's pack, then the equip: Legions 7160).
    fn drop(
        &self,
        tree: &UiTree,
        s: &dyn InventorySource,
        target: usize,
        copy: usize,
    ) -> Option<ItemRequest> {
        let location = if Some(target) == self.viewport {
            None
        } else {
            Some(self.slot_at(target)?.location)
        };
        if !is_item_copy(tree, copy) {
            return None;
        }
        let dragged = dragged_item(tree, copy)?;
        if dragged.from == 0 {
            return None;
        }
        let location = location.unwrap_or_else(|| s.preferred_location(dragged.item));
        if location == 0 {
            return None;
        }
        if dragged.from != self.player {
            return Some(ItemRequest::MoveAndEquip {
                item: dragged.item,
                from: dragged.from,
                to: self.player,
                location,
            });
        }
        Some(ItemRequest::Equip {
            item: dragged.item,
            location,
        })
    }

    /// The slot whose element is `element`.
    fn slot_at(&self, element: usize) -> Option<Slot> {
        self.slots.iter().copied().find(|s| s.slot == Some(element))
    }

    /// The doll shows while its slots do (`UIChildFramework::Show` with the inventory panel).
    pub fn follow_panel(&self, tree: &mut UiTree) {
        let Some(viewport) = self.viewport else {
            return;
        };
        let anchor = self.slots.iter().find_map(|s| s.slot);
        let shown = anchor.is_some_and(|mut at| loop {
            let Some(el) = tree.get(at) else {
                break false;
            };
            if !el.visible() {
                break false;
            }
            match el.parent_index() {
                Some(parent) => at = parent,
                None => break true,
            }
        });
        if tree.get(viewport).is_some_and(|e| e.visible() != shown) {
            tree.set_visible(viewport, shown);
        }
    }
}
