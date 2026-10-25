//! `gmBackpackUI`: the inventory panel's four pages of the player's own contents
//! (`PackMainContainer`, `PackOneContainer`, `PackTwoContainer`, `PackThreeContainer`), each a
//! [`ContainerDisplay`] over the player's contents segment of its index, numbered on from the
//! previous page's slots. A page's button shows that page and presses its button; the others
//! hide (`ShowPack`).
//!
//! Not modelled yet: dropping items onto the pages (`FinishItemDrop`, the stack dialog) and the
//! money bag's drop.

use std::sync::Arc;

use crate::element::UiTree;
use crate::item_display::{ContainerDisplay, InventorySource, ItemRequest};
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, STATE_NORMAL, STATE_NORMAL_PRESSED};

/// `PackElementTable`: (container element, button).
const PACKS: [(&str, &str); 4] = [
    ("PackMainContainer", "PackMainButton"),
    ("PackOneContainer", "PackOneButton"),
    ("PackTwoContainer", "PackTwoButton"),
    ("PackThreeContainer", "PackThreeButton"),
];

/// `gmBackpackUI`'s state.
#[derive(Debug, Clone)]
pub struct BackpackUi {
    pub packs: Vec<ContainerDisplay>,
    containers: Vec<usize>,
    buttons: Vec<usize>,
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

impl BackpackUi {
    /// `gmBackpackUI::gmBackpackUI`: a container display per page over the player, then page 0.
    pub fn create(
        tree: &mut UiTree,
        layout: &Arc<LayoutDesc>,
        s: &dyn InventorySource,
        player: u64,
    ) -> Self {
        let mut ui = Self {
            packs: Vec::new(),
            containers: Vec::new(),
            buttons: Vec::new(),
        };
        let mut first_slot = 0;
        for (segment, (container, button)) in PACKS.iter().enumerate() {
            if let Some(element) = find(tree, container) {
                let display = ContainerDisplay::init(
                    tree,
                    layout,
                    s,
                    player,
                    element,
                    first_slot,
                    segment as u32,
                );
                first_slot += display.max_size();
                ui.packs.push(display);
                ui.containers.push(element);
            }
            if let Some(button) = find(tree, button) {
                ui.buttons.push(button);
            }
        }
        ui.show_pack(tree, 0);
        ui
    }

    /// `gmBackpackUI::ShowPack`.
    pub fn show_pack(&self, tree: &mut UiTree, pack: usize) {
        if pack >= 4 {
            return;
        }
        for (i, &container) in self.containers.iter().enumerate() {
            tree.set_visible(container, i == pack);
        }
        for (i, &button) in self.buttons.iter().enumerate() {
            tree.set_state(
                button,
                if i == pack {
                    STATE_NORMAL_PRESSED
                } else {
                    STATE_NORMAL
                },
            );
        }
    }

    /// `gmBackpackUI::ListenToMessage` (a page's button) and the slots' presses.
    pub fn message(
        &self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        message: &UiMessage,
    ) -> Option<ItemRequest> {
        if message.message == MSG_BUTTON_RELEASE_ON {
            if let Some(pack) = PACKS
                .iter()
                .position(|(_, b)| tokenize(b) == message.element_id)
            {
                self.show_pack(tree, pack);
                return None;
            }
        }
        self.packs.iter().find_map(|p| p.message(tree, s, message))
    }

    /// `ContainerDisplay::RecvNotice_UpdateContainer` for the pages over `container`.
    pub fn update_container(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        container: u64,
        code: u32,
        item: u64,
        slot: i32,
    ) {
        for pack in self.packs.iter_mut().filter(|p| p.container == container) {
            pack.update_container(tree, s, code, item, slot);
        }
    }

    /// An item's display changed (`RecvNotice_ItemUpdate` and the like).
    pub fn item_changed(&mut self, tree: &mut UiTree, s: &dyn InventorySource, item: u64) {
        for pack in &mut self.packs {
            pack.item_changed(tree, s, item);
        }
    }

    /// `ItemDisplay::RecvNotice_SelectionChanged`.
    pub fn selection_changed(&self, tree: &mut UiTree, old: u64, new: u64) {
        for pack in &self.packs {
            pack.selection_changed(tree, old, new);
        }
    }
}
