//! `gmBackpackUI`: the inventory panel's four pages of the player's own contents
//! (`PackMainContainer`, `PackOneContainer`, `PackTwoContainer`, `PackThreeContainer`), each a
//! [`ContainerDisplay`] over the player's contents segment of its index, numbered on from the
//! previous page's slots. A page's button shows that page and presses its button; the others
//! hide (`ShowPack`).
//!
//! A coin field dropped on a page (`gmBackpackUI::ListenToMessage`'s coin branch) asks how much
//! money to take out for the page's slot; the answer is `FinishMoneyBagDrop`'s `DragFromMoneyBag`.
//!
//! Not modelled yet: an item dropped on a page outside its list (`FinishItemDrop`).

use std::sync::Arc;

use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::element::UiTree;
use crate::item_display::{is_coin_copy, ContainerDisplay, InventorySource, ItemRequest};
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
    /// `m_moneybagStackDialogContext` and `m_newSlot` of the coin branch (0: none).
    money_context: u32,
    money_slot: u32,
}

/// The element named `name` among the layout's roots, or below one of them.
pub(crate) fn find(tree: &UiTree, name: &str) -> Option<usize> {
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
            money_context: 0,
            money_slot: 0,
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

    /// `gmBackpackUI::ListenToMessage`'s coin branch: the coin field dropped on page `pack`
    /// (`stop_drag_item`, the page's index under the pointer below its slot count) asks how much
    /// to take out, for the slot the pointer is at.
    fn coin_drop(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        pack: usize,
        message: &UiMessage,
    ) -> Option<ItemRequest> {
        let copy = message.item?;
        if message.data == 0 || !is_coin_copy(tree, copy) {
            return None;
        }
        let page = &self.packs[pack];
        if page.drop_index >= page.max_size() {
            return None;
        }
        self.money_slot = page.first_slot() + page.drop_index;
        self.money_context = tree.request_stack_context();
        let amount = s.money().min(s.coin_max_stack());
        Some(ItemRequest::MoneyStack {
            context: self.money_context,
            target: 0,
            amount,
        })
    }

    /// `gmBackpackUI::ListenToMessage` (a page's button, a drop on a page) and the slots' presses.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        s: &dyn InventorySource,
        message: &UiMessage,
    ) -> Option<ItemRequest> {
        if message.message == MSG_STOP_DRAG_ITEM {
            let page = self
                .containers
                .iter()
                .position(|&c| message.element == Some(c));
            if let Some(pack) = page {
                return self.coin_drop(tree, s, pack, message);
            }
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            if let Some(pack) = PACKS
                .iter()
                .position(|(_, b)| tokenize(b) == message.element_id)
            {
                self.show_pack(tree, pack);
                return None;
            }
        }
        self.packs
            .iter_mut()
            .find_map(|p| p.message(tree, s, message))
    }

    /// `gmBackpackUI::RecvNotice_StackUIDone` (`FinishItemDrop`): the pages' drops waiting for
    /// the stack dialog.
    pub fn stack_ui_done(
        &mut self,
        s: &dyn InventorySource,
        context: u32,
        amount: u32,
    ) -> Option<ItemRequest> {
        if context != 0 && context == self.money_context {
            return Some(ItemRequest::DragFromMoneyBag {
                quantity: amount,
                to_slot: self.money_slot as i32,
                to_container: 0,
            });
        }
        self.packs
            .iter_mut()
            .find_map(|p| p.stack_ui_done(s, context, amount))
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
