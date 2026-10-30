//! `gmIngredientUI`'s ingredient slots: one list item per ingredient the recipe asks for, in ordinal
//! order (`MultiOpAddIngredientSlot`, `MultiOpRemoveIngredientSlot`, `UpdateIngredientSlot`,
//! `UpdateIngredients`). An object dropped on a slot is checked (`CheckItem`), then requested
//! (`HandleListBoxDrop`); the server's answer places it (`RecvNotice_AddIngredient_Done`), and an
//! object that no longer fits leaves its slot (`HandleItemRemoval`). Auto-populate fills the free
//! slots the recipe fills itself from the inventory (`AutoPopulateIngredients`).

use ac2_weenie::Ingredient;

use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::element::UiTree;
use crate::ingredient::{
    IngredientCommand, IngredientSource, IngredientUi, Slot, AUTO_POPULATE, CLOSE_BUTTON,
    CRAFT_BUTTON, INGREDIENT_LABEL, INGREDIENT_LIST, INGREDIENT_TEMPLATE, ITEM_ICON,
    SPINNER_TYPING_WAIT, SPINNER_VALUE, TOKEN_DRAGABLE, TOKEN_ID, TOKEN_PENDING, TOKEN_STATELESS,
};
use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};

/// The item-removal element: an object dragged out of a slot (`gmIngredientUI::ListenToMessage`).
const INGREDIENT_ITEM: u32 = tokenize("IngredientItem");

/// The bit an ordinal holds in the mask (`2^(ordinal - 1)`); ordinals run from 1 to 32.
fn bit(ordinal: u32) -> u32 {
    1u32.checked_shl(ordinal.saturating_sub(1)).unwrap_or(0)
}

impl IngredientUi {
    /// The slot an ordinal names.
    fn slot_index(&self, ordinal: u32) -> Option<usize> {
        self.slots.iter().position(|slot| slot.ordinal == ordinal)
    }

    /// The slot holding `object`.
    fn slot_of_item(&self, object: u64) -> Option<usize> {
        self.slots.iter().position(|slot| slot.item == Some(object))
    }

    /// `gmIngredientUI::MultiOpAddIngredientSlot`: the ingredient's list item, at its ordinal's
    /// place in the list, with its description (`token_id` is the ordinal).
    pub(crate) fn add_slot(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        ingredient: &Ingredient,
    ) {
        let Some(list) = self.child(tree, INGREDIENT_LIST) else {
            return;
        };
        let Some(item) = self.create_template(tree, INGREDIENT_TEMPLATE) else {
            return;
        };
        tree.set_attribute_id(item, TOKEN_ID, ingredient.id);
        if let Some(icon) = tree.child_recursive(item, ITEM_ICON) {
            tree.set_attribute_id(icon, TOKEN_ID, 0);
        }
        if let Some(label) = tree.child_recursive(item, INGREDIENT_LABEL) {
            let text = source.text(&ingredient.description);
            tree.set_text(label, &text);
        }
        let at = self
            .slots
            .iter()
            .position(|slot| slot.ordinal > ingredient.id)
            .unwrap_or(self.slots.len());
        tree.list_insert_item(list, item, at);
        self.slots.insert(
            at,
            Slot {
                ordinal: ingredient.id,
                element: item,
                item: None,
                pending: false,
            },
        );
        self.mask |= bit(ingredient.id);
    }

    /// `gmIngredientUI::MultiOpRemoveIngredientSlot`: a slot whose ingredient the spinner no longer
    /// leaves goes, its object with it.
    pub(crate) fn remove_slot(&mut self, tree: &mut UiTree, ordinal: u32) {
        let Some(index) = self.slot_index(ordinal) else {
            return;
        };
        self.clear_slot(tree, index);
        if let Some(list) = self.child(tree, INGREDIENT_LIST) {
            tree.list_remove_item(list, index);
        }
        self.slots.remove(index);
        self.mask &= !bit(ordinal);
    }

    /// `gmIngredientUI::UpdateIngredientSlot`: a kept slot's description, and its object checked
    /// again at the new spinner value.
    fn update_slot(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        ingredient: &Ingredient,
    ) {
        let Some(index) = self.slot_index(ingredient.id) else {
            return;
        };
        let element = self.slots[index].element;
        if let Some(label) = tree.child_recursive(element, INGREDIENT_LABEL) {
            let text = source.text(&ingredient.description);
            tree.set_text(label, &text);
        }
        if let Some(object) = self.slots[index].item {
            self.check_item(tree, source, object, ingredient.id);
        }
    }

    /// `gmIngredientUI::UpdateIngredients`: the slots follow the recipe's ingredients at the spinner
    /// value: new ones are added, kept ones refreshed, the ones it no longer asks for removed.
    pub(crate) fn update_ingredients(&mut self, tree: &mut UiTree, source: &dyn IngredientSource) {
        if self.recipe == 0 {
            return;
        }
        let (mask, ingredients) = source.ingredients(self.recipe, self.spinner);
        for ingredient in &ingredients {
            if self.mask & bit(ingredient.id) == 0 {
                self.add_slot(tree, source, ingredient);
            } else {
                self.update_slot(tree, source, ingredient);
            }
        }
        for ordinal in 1..=32u32 {
            if self.mask & !mask & bit(ordinal) != 0 {
                self.remove_slot(tree, ordinal);
            }
        }
    }

    /// `gmIngredientUI::CheckItem`: an object in a slot stays if the player has it and the recipe
    /// takes it at the spinner value; otherwise it leaves its slot. Returns whether it fits.
    pub(crate) fn check_item(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        object: u64,
        ordinal: u32,
    ) -> bool {
        let was_in_slot = self.slot_of_item(object).is_some();
        if source.has_item(object)
            && source.check_ingredient(self.recipe, ordinal, object, self.spinner)
        {
            return true;
        }
        if was_in_slot {
            self.handle_item_removal(tree, object);
        }
        false
    }

    /// `gmIngredientUI::HandleItemRemoval`: the object leaves its slot (the icon clears, the slot is
    /// no longer draggable).
    pub(crate) fn handle_item_removal(&mut self, tree: &mut UiTree, object: u64) {
        if let Some(index) = self.slot_of_item(object) {
            self.clear_slot(tree, index);
        }
    }

    /// The slot's object goes: its icon clears and stops being draggable.
    pub(crate) fn clear_slot(&mut self, tree: &mut UiTree, index: usize) {
        let element = self.slots[index].element;
        self.slots[index].item = None;
        if let Some(icon) = tree.child_recursive(element, ITEM_ICON) {
            tree.clear_image(icon);
            tree.set_bool(icon, TOKEN_DRAGABLE, false);
            tree.set_attribute_id(icon, TOKEN_ID, 0);
        }
    }

    /// Places `object` in slot `index` as pending: the icon shows it and it stops being draggable
    /// until the server answers (`HandleListBoxDrop`'s pending item).
    fn place_pending(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        index: usize,
        object: u64,
    ) {
        let element = self.slots[index].element;
        self.slots[index].item = Some(object);
        self.slots[index].pending = true;
        tree.set_state(element, TOKEN_PENDING);
        if let Some(icon) = tree.child_recursive(element, ITEM_ICON) {
            tree.set_instance_attribute(icon, TOKEN_ID, object);
            if let Some(image) = source.item_icon(object) {
                tree.set_image(icon, image, 2);
            }
            tree.set_bool(icon, TOKEN_DRAGABLE, false);
        }
    }

    /// `gmIngredientUI::HandleListBoxDrop`: an object dropped on a slot. It leaves the slot it was
    /// in, the slot's object leaves, the slot goes pending, and the request goes out.
    pub(crate) fn drop_on_slot(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        object: u64,
        index: usize,
    ) {
        if self.slots[index].item == Some(object) {
            return;
        }
        if let Some(elsewhere) = self.slot_of_item(object) {
            self.clear_slot(tree, elsewhere);
        }
        if self.slots[index].item.is_some() {
            self.clear_slot(tree, index);
        }
        let ordinal = self.slots[index].ordinal;
        self.place_pending(tree, source, index, object);
        self.commands.push(IngredientCommand::Add {
            recipe: self.recipe,
            ordinal,
            object,
            spinner: self.spinner,
        });
    }

    /// `RecvNotice_AddIngredient_Done`: the server answered the add. A success puts the object in
    /// the slot (its icon, draggable); a failure frees the slot. A new spinner value follows.
    pub fn add_done(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        ordinal: u32,
        object: u64,
        spinner: u32,
        error: u32,
    ) {
        if spinner != self.spinner {
            self.spinner = spinner;
            self.spinner_changed(tree, source);
        }
        let Some(index) = self.slot_index(ordinal) else {
            return;
        };
        let element = self.slots[index].element;
        self.slots[index].pending = false;
        tree.set_state(element, TOKEN_STATELESS);
        if error != 0 {
            self.clear_slot(tree, index);
            return;
        }
        if let Some(elsewhere) = self.slot_of_item(object) {
            if elsewhere != index {
                self.clear_slot(tree, elsewhere);
            }
        }
        self.place_pending(tree, source, index, object);
        self.slots[index].pending = false;
        if let Some(icon) = tree.child_recursive(element, ITEM_ICON) {
            // The icon description when the object has one (`SetIconDesc`), else its icon.
            let layers = source.item_icon_layers(object);
            if !layers.is_empty() {
                tree.set_icon_desc(icon, &layers);
            }
            tree.set_bool(icon, TOKEN_DRAGABLE, true);
        }
    }

    /// `gmIngredientUI::AutoPopulateIngredients`: each inventory object not in a slot goes into the
    /// first free slot the recipe auto-populates and takes, and its request goes out.
    pub(crate) fn auto_populate(&mut self, tree: &mut UiTree, source: &dyn IngredientSource) {
        for object in source.inventory() {
            if self.slot_of_item(object).is_some() {
                continue;
            }
            let free = (0..self.slots.len()).find(|&index| {
                let slot = &self.slots[index];
                slot.item.is_none()
                    && source.auto_populates(self.recipe, slot.ordinal)
                    && source.check_ingredient(self.recipe, slot.ordinal, object, self.spinner)
            });
            if let Some(index) = free {
                let ordinal = self.slots[index].ordinal;
                self.place_pending(tree, source, index, object);
                self.commands.push(IngredientCommand::Add {
                    recipe: self.recipe,
                    ordinal,
                    object,
                    spinner: self.spinner,
                });
            }
        }
    }

    /// `gmIngredientUI::RecvNotice_BeingDeleted`: an object that is gone leaves its slot.
    pub fn object_removed(&mut self, tree: &mut UiTree, object: u64) {
        self.handle_item_removal(tree, object);
    }

    /// `RecvNotice_UpdateInventory`, `RecvNotice_UpdateContainer` and `RecvNotice_ItemUpdate`: an
    /// object in a slot is checked again.
    pub fn item_changed(&mut self, tree: &mut UiTree, source: &dyn IngredientSource, object: u64) {
        if let Some(index) = self.slot_of_item(object) {
            let ordinal = self.slots[index].ordinal;
            self.check_item(tree, source, object, ordinal);
        }
    }

    /// The panel's messages: the spinner's buttons, the craft and auto-populate buttons, a drop on
    /// the list, and an object dragged out of its slot.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        message: &UiMessage,
    ) {
        // `ListenToMessage` answers nothing while the panel is not shown.
        if !self.shown {
            return;
        }
        let element_id = message.element_id;
        if element_id == SPINNER_VALUE {
            // The typed value: leaving the field takes it at once, any other message of the
            // field (typing) starts the two second wait.
            if message.message == tokenize("deactivated") {
                self.commit_spinner_text(tree, source);
            } else {
                self.spinner_edited = true;
                self.spinner_update_at = self.now + SPINNER_TYPING_WAIT;
            }
            return;
        }
        if message.message == MSG_STOP_DRAG_ITEM {
            let Some(dragged) = message.item else {
                return;
            };
            if element_id == INGREDIENT_LIST {
                // `HandleListBoxDrop` reads the dragged copy's `Token_ShortcutData`.
                let object = crate::item_display::dragged_item(tree, dragged)
                    .map(|d| d.item)
                    .filter(|&object| object != 0);
                let Some(object) = object else {
                    return;
                };
                if let Some(list) = self.child(tree, INGREDIENT_LIST) {
                    if let Some(index) = tree.list_index_under_mouse(list) {
                        if index < self.slots.len() {
                            self.drop_on_slot(tree, source, object, index);
                        }
                    }
                }
            } else if element_id == INGREDIENT_ITEM {
                // An object dragged out of its slot carries it as the slot icon's `token_id`.
                if let Some(object) = instance_of(tree, dragged) {
                    self.handle_item_removal(tree, object);
                }
            }
            return;
        }
        if message.message != MSG_BUTTON_RELEASE_ON {
            return;
        }
        if element_id == tokenize("SpinnerUpButton") {
            self.step_spinner(tree, source, true);
        } else if element_id == tokenize("SpinnerDownButton") {
            self.step_spinner(tree, source, false);
        } else if element_id == AUTO_POPULATE {
            self.auto_populate(tree, source);
        } else if element_id == CRAFT_BUTTON {
            self.execute(tree);
        } else if element_id == CLOSE_BUTTON && !self.pending {
            self.close(tree);
        }
    }

    /// Creates a panel template by name (`UIFramework::CreateElement(0x41000009, template)`).
    pub(crate) fn create_template(&self, tree: &mut UiTree, name: u32) -> Option<usize> {
        let layout = self.layout.clone()?;
        tree.create_nested_element(&layout, name)
    }
}

/// The object id an element carries as its `id` attribute (`GetAttribute_InstanceID(item, token_id)`).
fn instance_of(tree: &UiTree, element: usize) -> Option<u64> {
    match tree.get(element)?.attribute(TOKEN_ID) {
        Some(AttributeValue::InstanceId(id)) => Some(*id),
        _ => None,
    }
}
