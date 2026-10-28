//! `gmIngredientUI`: the ingredient panel of a recipe on the gameplay layout (`RealIngredientField`,
//! the element `CreateManagedElements` makes). It opens on the recipe's
//! `OpenIngredientPanel` (`Init`): the recipe's name, cost, difficulty and spinner range, and a
//! slot per ingredient the recipe asks for (`ingredient_slots.rs`). The spinner sets the value the
//! recipe is made at (`UpdateRecipeFromSpinnerChange`), the craft button executes it
//! (`RequestExecuteCraft`), and the auto-populate button fills the free slots from the inventory.
//!
//! The data comes from an [`IngredientSource`]: the recipe and craft skill adaptors, the
//! ingredient slots and their checks, the cost and difficulty at a spinner value, and the
//! inventory. Requests go out as [`IngredientCommand`]s.
//!
//! The panel's height follows its ingredient count (`AutoSizeIngredientUI`).
//!
//! The strings are the `Tokenize` initializers of the admin 7148 tree matched by position to
//! Legions's `DAT_` symbols in the same functions (the Legions initializers are not decompiled).
//!
//! Not modelled yet: the name's colour (the colour attribute is not in the tree), and the 15
//! second dormancy that destroys the panel.

use std::sync::Arc;

use ac2_net::protocol::StringInfo;
use ac2_weenie::{CraftSkill, GameplayNotice, Ingredient, Recipe, SessionEvent};

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MAX_HEIGHT, MIN_HEIGHT, MIN_WIDTH};

/// `gmIngredientUI`'s element names (the gameplay layout's `IngredientField`'s children).
pub(crate) const INGREDIENT_LIST: u32 = tokenize("IngredientListBox");
pub(crate) const INGREDIENT_TEMPLATE: u32 = tokenize("IngredientTemplate");
pub(crate) const INGREDIENT_LABEL: u32 = tokenize("IngredientLabel");
pub(crate) const ITEM_ICON: u32 = tokenize("ItemIcon");
pub(crate) const SPINNER_FIELD: u32 = tokenize("SpinnerField");
pub(crate) const SPINNER_UP: u32 = tokenize("SpinnerUpButton");
pub(crate) const SPINNER_DOWN: u32 = tokenize("SpinnerDownButton");
pub(crate) const SPINNER_VALUE: u32 = tokenize("SpinnerValueField");
pub(crate) const AUTO_POPULATE: u32 = tokenize("AutoPopulateButton");
pub(crate) const CRAFT_BUTTON: u32 = tokenize("CraftButton");
pub(crate) const COST_FIELD: u32 = tokenize("CostField");
/// `ID_Craft_Cost`: the cost text, with its `value`.
pub(crate) const CRAFT_COST: u32 = tokenize("ID_Craft_Cost");
/// The elements `CreateManagedElements` looks up by DAT id (`RealIngredientField` is the root made
/// from the layout, `RecipeLabel` and `RecipeMiscText` the texts, `CraftButtonText` the craft
/// button's caption, `IngredientCloseButton` the close button); the names are the `Tokenize`
/// strings of the admin 7148 initializers for Legions's `DAT_00a46a44`, `00a468d0`, `00a469f4`,
/// `00a46770` and `00a469d4`.
pub(crate) const ROOT_FIELD: u32 = tokenize("RealIngredientField");
pub(crate) const LABEL_TEXT: u32 = tokenize("RecipeLabel");
pub(crate) const MISC_TEXT: u32 = tokenize("RecipeMiscText");
pub(crate) const CRAFT_BUTTON_TEXT: u32 = tokenize("CraftButtonText");
pub(crate) const CLOSE_BUTTON: u32 = tokenize("IngredientCloseButton");
/// The craft button's caption and tooltips (`UpdateCraftButton`: `DAT_00a467c0`, `00a468a0`,
/// `00a468f4`, `00a46a58`, `00a46aa4`).
pub(crate) const CRAFT_BUTTON_LABEL: u32 = tokenize("ID_Craft_CraftButton");
pub(crate) const CRAFT_BUTTON_DISABLED: u32 = tokenize("ID_Craft_CraftButton_Disabled");
pub(crate) const TT_INVALID_SPINNER: u32 = tokenize("ID_Craft_TT_InvalidSpinner");
pub(crate) const TT_TOO_HARD: u32 = tokenize("ID_Craft_TT_TooHard");
pub(crate) const TT_MOUNTED: u32 = tokenize("ID_Craft_TT_Mounted");
/// Seconds a typed spinner value waits before it is taken (`ListenToMessage`: `Timer::cur_time + 2.0`).
pub(crate) const SPINNER_TYPING_WAIT: f64 = 2.0;
pub(crate) const TOKEN_ID: u32 = tokenize("id");
pub(crate) const TOKEN_DRAGABLE: u32 = tokenize("dragable");
pub(crate) const TOKEN_PENDING: u32 = tokenize("pending");
pub(crate) const TOKEN_STATELESS: u32 = tokenize("stateless");

/// What the panel reads from the player and the DAT.
pub trait IngredientSource {
    /// `StringInfo::GetString` of a server string.
    fn text(&self, info: &StringInfo) -> String;
    /// A UI string with one integer variable (`value`), by its string id.
    fn string_with_value(&self, string_id: u32, value: i32) -> String;
    /// `gmCWInterface::InqRecipeAdaptor`.
    fn recipe(&self, recipe: u32) -> Option<Recipe>;
    /// `gmCWInterface::InqCraftSkillAdaptor`.
    fn craft_skill(&self, skill: u32) -> Option<CraftSkill>;
    /// `gmCWInterface::InqRecipeIngredients`: the ordinal mask the spinner leaves filled, and the
    /// slots.
    fn ingredients(&self, recipe: u32, spinner: u32) -> (u32, Vec<Ingredient>);
    /// `gmCWInterface::InqRecipeCost` at `spinner`.
    fn cost(&self, recipe: u32, spinner: u32) -> Option<i32>;
    /// `gmCWInterface::InqRecipeDifficulty` at `spinner`: the difficulty and the name colour.
    fn difficulty(&self, recipe: u32, spinner: u32) -> Option<(u32, u32)>;
    /// `gmCWInterface::DoesIngredientAutoPopulate`.
    fn auto_populates(&self, recipe: u32, ordinal: u32) -> bool;
    /// `gmCWInterface::CheckIngredient`.
    fn check_ingredient(&self, recipe: u32, ordinal: u32, object: u64, spinner: u32) -> bool;
    /// `gmCWInterface::DoIHaveItem`.
    fn has_item(&self, object: u64) -> bool;
    /// The player's inventory (`InqAllInventoryIIDList`).
    fn inventory(&self) -> Vec<u64>;
    /// The icon an object shows (`GetIconID`).
    fn item_icon(&self, object: u64) -> Option<u32>;
}

/// A request the panel makes of the player's interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngredientCommand {
    /// `gmCWInterface::RequestAddIngredient`: `object` goes into slot `ordinal`.
    Add {
        recipe: u32,
        ordinal: u32,
        object: u64,
        spinner: u32,
    },
    /// `gmCWInterface::RequestExecuteCraft`: the slots' objects, by ordinal.
    Execute {
        recipe: u32,
        spinner: u32,
        ingredients: Vec<(u32, u64)>,
    },
}

/// One ingredient slot: its list item, its ordinal and the object in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Slot {
    pub ordinal: u32,
    pub element: usize,
    pub item: Option<u64>,
    pub pending: bool,
}

/// `gmIngredientUI`'s state.
#[derive(Debug, Clone, Default)]
pub struct IngredientUi {
    pub(crate) layout: Option<Arc<LayoutDesc>>,
    pub(crate) root: Option<usize>,
    pub(crate) recipe: u32,
    pub(crate) skill: u32,
    /// `m_spinnerVal`, `m_minSpinnerVal`, `m_maxSpinnerVal`, `m_readOnly`.
    pub(crate) spinner: u32,
    pub(crate) min_spinner: u32,
    pub(crate) max_spinner: u32,
    pub(crate) read_only: bool,
    /// `m_maxNumIngredients`, `m_difficulty`, `m_nameColor`, `m_hasDifficultyMinusSkillCutoff`,
    /// `m_difficultyMinusSkillCutoff`, `m_dynamicDifficulty`, `m_dynamicCost`, `m_cost`.
    pub(crate) max_ingredients: u32,
    pub(crate) difficulty: u32,
    pub(crate) name_color: u32,
    pub(crate) has_difficulty_cutoff: bool,
    pub(crate) difficulty_cutoff: i32,
    pub(crate) dynamic_difficulty: bool,
    pub(crate) dynamic_cost: bool,
    pub(crate) cost: i32,
    /// `m_craftSkillLevel`.
    pub(crate) skill_level: u32,
    /// `m_mounted`, `m_pending`, and whether the panel is shown.
    pub(crate) mounted: bool,
    pub(crate) pending: bool,
    pub(crate) shown: bool,
    /// The slots in list order (by ordinal), and `m_ingOrdinalMask`.
    pub(crate) slots: Vec<Slot>,
    pub(crate) mask: u32,
    pub(crate) commands: Vec<IngredientCommand>,
    /// `m_spinnerValueEdited` and `m_timeToUpdateSpinnerValue`: typed text waits two seconds.
    pub(crate) spinner_edited: bool,
    pub(crate) spinner_update_at: f64,
    /// The frame's time (`Timer::cur_time`).
    pub(crate) now: f64,
    /// The craft button's caption and tooltips, rendered when the panel opens.
    pub(crate) button_text: ButtonText,
}

/// The strings `UpdateCraftButton` shows.
#[derive(Debug, Clone, Default)]
pub(crate) struct ButtonText {
    pub craft: String,
    pub disabled: String,
    pub invalid_spinner: String,
    pub too_hard: String,
    pub mounted: String,
}

impl IngredientUi {
    /// `gmIngredientUI::gmIngredientUI`: the panel's layout; its elements are made when it opens
    /// (`CreateManagedElements`).
    pub fn new(layout: &Arc<LayoutDesc>) -> Self {
        Self {
            layout: Some(Arc::clone(layout)),
            ..Self::default()
        }
    }

    /// The requests the panel made since the last call, in order.
    /// The ingredient panel on the frame: its notices in `events` (the panel's own recipe's
    /// notices only, and the object removals), then its `messages`. Returns the requests the
    /// frame made, in order.
    pub fn frame_pass(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        events: &[SessionEvent],
        messages: &[UiMessage],
        now: f64,
    ) -> Vec<IngredientCommand> {
        self.now = now;
        for event in events {
            match event {
                SessionEvent::ObjectRemoved(object) => self.object_removed(tree, *object),
                SessionEvent::Notice(notice) => self.notice(tree, source, notice),
                _ => {}
            }
        }
        for message in messages {
            self.message(tree, source, message);
        }
        self.tick(tree, source);
        self.take_commands()
    }

    /// The panel's notices: the recipe's open, add and execute answers, the mount state, the
    /// craft skill, and the items the inventory changed.
    fn notice(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        notice: &GameplayNotice,
    ) {
        match notice {
            GameplayNotice::OpenIngredientPanel(recipe) => self.open_notice(tree, source, *recipe),
            GameplayNotice::AddIngredientDone {
                recipe,
                ordinal,
                ingredient,
                spinner,
                error,
            } if *recipe == self.recipe() => {
                self.add_done(tree, source, *ordinal, *ingredient, *spinner, *error)
            }
            GameplayNotice::ExecuteCraftDone { recipe, error } if *recipe == self.recipe() => {
                self.execute_done(tree, source, *error)
            }
            GameplayNotice::UpdateMountState(mounted) => self.mount_state(tree, *mounted),
            GameplayNotice::UpdateCraftSkill(skill) => {
                self.craft_skill_updated(tree, source, skill)
            }
            GameplayNotice::UpdateInventory { item, .. }
            | GameplayNotice::ItemUpdate(item)
            | GameplayNotice::UpdateContainer { item, .. } => {
                self.item_changed(tree, source, *item)
            }
            _ => {}
        }
    }

    pub fn take_commands(&mut self) -> Vec<IngredientCommand> {
        std::mem::take(&mut self.commands)
    }

    /// The panel's root element, once it is made.
    pub fn root(&self) -> Option<usize> {
        self.root
    }

    /// The spinner's value.
    pub fn spinner(&self) -> u32 {
        self.spinner
    }

    /// The recipe the panel shows.
    pub fn recipe(&self) -> u32 {
        self.recipe
    }

    /// `gmIngredientUI::CreateManagedElements`: the panel's root, made once from the layout.
    pub(crate) fn create_managed(&mut self, tree: &mut UiTree) -> bool {
        if self.root.is_some() {
            return true;
        }
        let Some(layout) = self.layout.clone() else {
            return false;
        };
        self.root = tree.create_root(layout, ROOT_FIELD);
        // `gmIngredientUI::CreateManagedElements`: `gmIngredientUI_DDCB` on the list.
        if let Some(list) = self.child(tree, INGREDIENT_LIST) {
            tree.set_drag_drop_check(list, Some(crate::DragCheck::IngredientDrop));
        }
        self.root.is_some()
    }

    /// A named element of the panel.
    pub(crate) fn child(&self, tree: &UiTree, name: u32) -> Option<usize> {
        tree.child_recursive(self.root?, name)
    }

    /// `gmIngredientUI::RecvNotice_OpenIngredientPanel`: the recipe's panel opens, unless it is
    /// shown already and its cost is dynamic.
    pub fn open_notice(&mut self, tree: &mut UiTree, source: &dyn IngredientSource, recipe: u32) {
        if self.shown && self.dynamic_cost {
            return;
        }
        self.open(tree, source, recipe);
    }

    /// `gmIngredientUI::Init` (`RecvNotice_OpenIngredientPanel`): the recipe's panel. A recipe the
    /// DAT does not know leaves the panel as it is.
    pub fn open(&mut self, tree: &mut UiTree, source: &dyn IngredientSource, recipe: u32) {
        if !self.create_managed(tree) {
            return;
        }
        let Some(record) = source.recipe(recipe) else {
            return;
        };
        self.recipe = recipe;
        self.pending = false;
        self.button_text = Self::render_button_text(source);
        if let Some(list) = self.child(tree, INGREDIENT_LIST) {
            tree.list_flush(list);
        }
        self.slots.clear();
        self.mask = 0;
        self.skill = record.craft_skill_did;
        self.min_spinner = record.min_spinner;
        self.max_spinner = record.max_spinner;
        self.spinner = record
            .last_spinner
            .clamp(record.min_spinner, record.max_spinner);
        self.read_only = record.read_only_spinner;
        self.max_ingredients = record.max_ingredients;
        self.name_color = record.name_color;
        self.difficulty = record.difficulty;
        self.dynamic_difficulty = record.dynamic_difficulty;
        self.has_difficulty_cutoff = record.has_difficulty_minus_skill_cutoff;
        self.difficulty_cutoff = record.difficulty_minus_skill_cutoff;
        self.dynamic_cost = record.dynamic_cost;
        self.cost = record.cost;
        self.skill_level = source
            .craft_skill(self.skill)
            .map_or(0, |skill| skill.level);
        self.show_spinner(tree);
        self.update_cost(tree, source);
        if let Some(label) = self.child(tree, LABEL_TEXT) {
            let text = source.text(&record.name);
            tree.set_text(label, &text);
        }
        // The misc text is the charges line (`ID_Craft_Charges`) while the recipe has charges, else
        // it is hidden; the recipe's description is not shown here.
        if let Some(misc) = self.child(tree, MISC_TEXT) {
            if record.charges > 0 {
                let text = source.string_with_value(CRAFT_CHARGES, record.charges);
                tree.set_text(misc, &text);
            }
            tree.set_visible(misc, record.charges > 0);
        }
        let (mask, ingredients) = source.ingredients(recipe, self.spinner);
        self.mask = mask;
        for ingredient in &ingredients {
            self.add_slot(tree, source, ingredient);
        }
        if let Some(list) = self.child(tree, INGREDIENT_LIST) {
            tree.list_multi_op_complete(list);
        }
        self.update_craft_button(tree);
        self.update_difficulty();
        self.auto_size(tree);
        self.shown = true;
        if let Some(root) = self.root {
            tree.set_visible(root, true);
        }
    }

    /// The spinner's range and value, as its buttons and text show them (`Init`'s spinner part).
    fn show_spinner(&self, tree: &mut UiTree) {
        let shown = self.max_spinner > self.min_spinner;
        if let Some(field) = self.child(tree, SPINNER_FIELD) {
            tree.set_visible(field, shown);
        }
        if let Some(text) = self.child(tree, SPINNER_VALUE) {
            let value = self.spinner.to_string();
            tree.set_text(text, &value);
        }
        self.set_button_ghosted(tree, SPINNER_DOWN, self.spinner <= self.min_spinner);
        self.set_button_ghosted(tree, SPINNER_UP, self.spinner >= self.max_spinner);
    }

    /// `gmIngredientUI::UpdateCost`: the cost text (`ID_Craft_Cost` with the cost as `value`) on
    /// the `CostField` element.
    fn update_cost(&self, tree: &mut UiTree, source: &dyn IngredientSource) {
        if let Some(field) = self.child(tree, COST_FIELD) {
            let text = source.string_with_value(CRAFT_COST, self.cost);
            tree.set_text(field, &text);
        }
    }

    /// `gmIngredientUI::UpdateCraftButton`: the button is ghosted while a craft is pending (nothing
    /// else changes); while mounted, while the spinner is out of range and while the difficulty is
    /// past the skill's cutoff it is ghosted with the disabled caption and the reason as its
    /// tooltip; otherwise a ghosted button gets its caption back, no tooltip, and is enabled.
    pub(crate) fn update_craft_button(&self, tree: &mut UiTree) {
        let Some(button) = self.child(tree, CRAFT_BUTTON) else {
            return;
        };
        let was_ghosted = tree.bool_attribute(button, BUTTON_GHOSTED);
        if self.pending {
            tree.set_bool(button, BUTTON_GHOSTED, true);
            return;
        }
        let text = &self.button_text;
        let reason = if self.mounted {
            &text.mounted
        } else if self.spinner < self.min_spinner || self.max_spinner < self.spinner {
            &text.invalid_spinner
        } else if self.difficulty_too_high() {
            &text.too_hard
        } else {
            if !was_ghosted {
                return;
            }
            tree.set_bool(button, BUTTON_GHOSTED, false);
            self.set_craft_caption(tree, &text.craft);
            tree.set_tooltip(button, None);
            return;
        };
        tree.set_bool(button, BUTTON_GHOSTED, true);
        self.set_craft_caption(tree, &text.disabled);
        tree.set_tooltip(button, Some(reason.clone()));
    }

    /// `UIElement_Text::SetStringInfo(m_craftButtonText, ..)`.
    fn set_craft_caption(&self, tree: &mut UiTree, caption: &str) {
        if let Some(element) = self.child(tree, CRAFT_BUTTON_TEXT) {
            tree.set_text(element, caption);
        }
    }

    /// The craft button's strings, rendered from the UI table.
    fn render_button_text(source: &dyn IngredientSource) -> ButtonText {
        let render = |id| source.string_with_value(id, 0);
        ButtonText {
            craft: render(CRAFT_BUTTON_LABEL),
            disabled: render(CRAFT_BUTTON_DISABLED),
            invalid_spinner: render(TT_INVALID_SPINNER),
            too_hard: render(TT_TOO_HARD),
            mounted: render(TT_MOUNTED),
        }
    }

    /// `gmIngredientUI::AutoSizeIngredientUI`: the panel is as wide as its `min_width` and as tall
    /// as its `min_height`, plus one template row (the template's box height) for each ingredient
    /// past the third, up to `max_height`. The template is made to read its height and freed.
    fn auto_size(&self, tree: &mut UiTree) {
        let Some(root) = self.root else {
            return;
        };
        let attribute = |id| tree.int_attribute(root, id).unwrap_or(0);
        let (width, mut height) = (attribute(MIN_WIDTH), attribute(MIN_HEIGHT));
        let max_height = attribute(MAX_HEIGHT);
        if self.max_ingredients > 3 {
            let Some(template) = self.create_template(tree, INGREDIENT_TEMPLATE) else {
                return;
            };
            let row = tree.el(template).rect().height();
            tree.delete(template);
            let grown = row * (self.max_ingredients as i32 - 3) + height;
            height = if grown <= max_height {
                grown
            } else {
                max_height
            };
        }
        tree.resize_to(root, width, height);
    }

    /// The difficulty past the cutoff (`m_hasDifficultyMinusSkillCutoff` and the skill's level).
    fn difficulty_too_high(&self) -> bool {
        self.has_difficulty_cutoff
            && (i64::from(self.difficulty) - i64::from(self.skill_level))
                > i64::from(self.difficulty_cutoff)
    }

    /// `gmIngredientUI::UpdateApparentDifficulty`: the name colour of the recipe. The colour is
    /// `Scale_` of the difficulty against the skill, clamped to 0..5, unless the recipe has its own
    /// table; a difficulty past the cutoff is colour 6. The name is not recoloured yet (the tree
    /// has no name colour).
    pub(crate) fn update_difficulty(&mut self) {
        self.name_color = if self.difficulty_too_high() {
            6
        } else if self.difficulty == 0 {
            0
        } else {
            let value = f64::from(self.difficulty) - f64::from(self.skill_level) + 42.0;
            let scaled = scale(value, -50.0, 50.0, 0.0, 5.0);
            let class = scaled as i64;
            if class > 4 {
                5
            } else {
                class.max(0) as u32
            }
        };
    }

    /// `gmIngredientUI::UpdateRecipeFromSpinnerChange`: the spinner's text and buttons, the dynamic
    /// cost and difficulty at the new value, the craft button and the slots.
    pub(crate) fn spinner_changed(&mut self, tree: &mut UiTree, source: &dyn IngredientSource) {
        if self.child(tree, SPINNER_VALUE).is_some() {
            self.show_spinner(tree);
        }
        if self.mounted || self.recipe == 0 {
            self.update_craft_button(tree);
            return;
        }
        if self.dynamic_cost {
            if let Some(cost) = source.cost(self.recipe, self.spinner) {
                self.cost = cost;
                self.update_cost(tree, source);
            }
        }
        if self.dynamic_difficulty {
            if let Some((difficulty, color)) = source.difficulty(self.recipe, self.spinner) {
                self.difficulty = difficulty;
                self.name_color = color;
                self.update_difficulty();
            }
        }
        self.update_craft_button(tree);
        self.update_ingredients(tree, source);
    }

    /// The spinner text taken as the value (`wcstol` base 0), kept within the range, then
    /// `UpdateRecipeFromSpinnerChange`: the commit of a typed value.
    pub(crate) fn commit_spinner_text(&mut self, tree: &mut UiTree, source: &dyn IngredientSource) {
        let text = self
            .child(tree, SPINNER_VALUE)
            .and_then(|e| tree.get(e).and_then(|e| e.text()))
            .unwrap_or_default();
        let value = wcstol_base0(&text);
        let value = if value < i64::from(self.min_spinner) {
            self.min_spinner
        } else if i64::from(self.max_spinner) < value {
            self.max_spinner
        } else {
            value as u32
        };
        self.spinner = value;
        self.spinner_changed(tree, source);
    }

    /// `gmIngredientUI::ListenToMessage` on the global loop (`0xffff0005`): typed text that has
    /// waited its two seconds is taken. (The 15 s dormancy destroy is not modelled.)
    pub fn tick(&mut self, tree: &mut UiTree, source: &dyn IngredientSource) {
        if self.shown && self.spinner_edited && self.spinner_update_at < self.now {
            self.spinner_edited = false;
            self.commit_spinner_text(tree, source);
        }
    }

    /// The spinner's up and down buttons (`ListenToMessage`): the value moves within its range,
    /// unless the spinner is read only.
    pub fn step_spinner(&mut self, tree: &mut UiTree, source: &dyn IngredientSource, up: bool) {
        if self.read_only {
            return;
        }
        if up && self.spinner < self.max_spinner {
            self.spinner += 1;
        } else if !up && self.spinner > self.min_spinner {
            self.spinner -= 1;
        }
        self.spinner_changed(tree, source);
    }

    /// Sets a button's ghosted attribute, when the button is there.
    fn set_button_ghosted(&self, tree: &mut UiTree, name: u32, ghosted: bool) {
        if let Some(button) = self.child(tree, name) {
            tree.set_bool(button, BUTTON_GHOSTED, ghosted);
        }
    }

    /// `gmIngredientUI::RecvNotice_UpdateCraftSkill`: the skill's level changed, so the difficulty
    /// and the craft button follow it.
    pub fn craft_skill_updated(
        &mut self,
        tree: &mut UiTree,
        source: &dyn IngredientSource,
        skill: &CraftSkill,
    ) {
        if self.recipe == 0 || skill.did != self.skill || self.skill_level == skill.level {
            return;
        }
        self.skill_level = skill.level;
        self.update_craft_button(tree);
        if let Some((difficulty, color)) = source.difficulty(self.recipe, self.spinner) {
            self.difficulty = difficulty;
            self.name_color = color;
        }
        self.update_difficulty();
    }

    /// `gmIngredientUI::RecvNotice_UpdateMountState`: mounted, the craft button shows as unusable.
    pub fn mount_state(&mut self, tree: &mut UiTree, mounted: bool) {
        self.mounted = mounted;
        self.update_craft_button(tree);
    }

    /// `gmIngredientUI::SetPending`: the slots and the craft button show a craft in progress.
    pub(crate) fn set_pending(&mut self, tree: &mut UiTree, pending: bool) {
        self.pending = pending;
        for index in 0..self.slots.len() {
            let element = self.slots[index].element;
            let state = if pending {
                TOKEN_PENDING
            } else {
                TOKEN_STATELESS
            };
            tree.set_state(element, state);
            self.slots[index].pending = pending;
            if let Some(icon) = tree.child_recursive(element, ITEM_ICON) {
                tree.set_bool(icon, TOKEN_DRAGABLE, !pending);
            }
        }
        self.update_craft_button(tree);
    }

    /// `gmIngredientUI::CraftButton`: the craft is pending and every slot's object goes to the
    /// player's interface (`SetPending(1)`, `RequestExecuteCraft`).
    pub fn execute(&mut self, tree: &mut UiTree) {
        if self.recipe == 0 {
            return;
        }
        self.set_pending(tree, true);
        let ingredients = self
            .slots
            .iter()
            .filter_map(|slot| slot.item.map(|item| (slot.ordinal, item)))
            .collect();
        self.commands.push(IngredientCommand::Execute {
            recipe: self.recipe,
            spinner: self.spinner,
            ingredients,
        });
    }

    /// `gmIngredientUI::RecvNotice_ExecuteCraft_Done`: the craft is over. Without an error and with
    /// no charges left the panel closes; with charges, the charges show (`ID_Craft_Charges`).
    pub fn execute_done(&mut self, tree: &mut UiTree, source: &dyn IngredientSource, error: u32) {
        self.set_pending(tree, false);
        let charges = source
            .recipe(self.recipe)
            .map_or(0, |recipe| recipe.charges);
        if error == 0 && charges == 0 {
            self.close(tree);
            return;
        }
        if charges > 0 {
            if let Some(misc) = self.child(tree, MISC_TEXT) {
                let text = source.string_with_value(CRAFT_CHARGES, charges);
                tree.set_text(misc, &text);
            }
        } else if let Some(misc) = self.child(tree, MISC_TEXT) {
            tree.set_visible(misc, false);
        }
    }

    /// `gmIngredientUI::DoClose`: the slots' objects go, and the panel hides.
    pub fn close(&mut self, tree: &mut UiTree) {
        for index in (0..self.slots.len()).rev() {
            self.clear_slot(tree, index);
        }
        if let Some(list) = self.child(tree, INGREDIENT_LIST) {
            tree.list_flush(list);
        }
        self.slots.clear();
        self.mask = 0;
        self.pending = false;
        self.shown = false;
        if let Some(root) = self.root {
            tree.set_visible(root, false);
        }
    }
}

/// `wcstol(text, 0, 0)` on a 32-bit `long`: leading white space, a sign, then `0x` hex, a leading
/// 0 octal or decimal digits up to the first other character; no digits is 0; it saturates.
pub(crate) fn wcstol_base0(text: &str) -> i64 {
    let text = text.trim_start();
    let (negative, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (radix, digits) =
        if let Some(hex) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
            if hex.chars().next().is_some_and(|c| c.is_ascii_hexdigit()) {
                (16, hex)
            } else {
                (10, rest)
            }
        } else if rest.starts_with('0') {
            (8, rest)
        } else {
            (10, rest)
        };
    let mut value: i64 = 0;
    for c in digits.chars() {
        let Some(digit) = c.to_digit(radix) else {
            break;
        };
        value = (value * i64::from(radix) + i64::from(digit)).min(i64::from(i32::MAX) + 1);
    }
    let value = if negative { -value } else { value };
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX))
}

/// `ID_Craft_Charges`: the charges line when a craft leaves charges.
const CRAFT_CHARGES: u32 = tokenize("ID_Craft_Charges");

/// `gmIngredientUI::Scale_` in f64: `value` clamped to `min` and `max` (the retail min and max),
/// mapped onto `scaled_min` and `scaled_max`.
pub(crate) fn scale(value: f64, min: f64, max: f64, scaled_min: f64, scaled_max: f64) -> f64 {
    let mut v = min;
    if min <= value {
        v = value;
        if max < value {
            v = max;
        }
    }
    (scaled_max - scaled_min) * ((v - min) / (max - min)) + scaled_min
}

#[cfg(test)]
mod tests {
    use super::wcstol_base0;

    /// `wcstol(text, 0, 0)`: the spinner's typed text.
    #[test]
    fn typed_spinner_text_reads_like_wcstol_base_zero() {
        assert_eq!(wcstol_base0("12"), 12);
        assert_eq!(wcstol_base0("  -7x"), -7);
        assert_eq!(wcstol_base0("0x1F"), 31);
        assert_eq!(wcstol_base0("010"), 8);
        assert_eq!(wcstol_base0("0x"), 0);
        assert_eq!(wcstol_base0("abc"), 0);
        assert_eq!(wcstol_base0(""), 0);
        assert_eq!(wcstol_base0("99999999999"), i64::from(i32::MAX));
        assert_eq!(wcstol_base0("-99999999999"), i64::from(i32::MIN));
    }
}
