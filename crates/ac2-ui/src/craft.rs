//! `gmCraftUI`: the craft panel on the skill toolbar (`CraftPage` in the toolbar layout). It pages
//! through three views: the craft skills (`craftskillview`), a skill's categories
//! (`craftskillcategoryview`) and a category's recipes (`recipeview`). Forward opens the next
//! view, back the previous one; a double click on a row does the same. Selecting a skill, category
//! or recipe asks the player's interface to select nothing and to examine it
//! (`gmCWInterface::SetSelection`, `SetExaminationRequest`); opening a recipe asks for its
//! ingredient panel (`OpenRecipe`), and the raise button asks for the skill to be raised.
//!
//! The data comes from a [`CraftSource`]: the craft skill and recipe adaptors, the lists the
//! player's registry fills (`InqCraftSkillList`, `InqRecipeList`) and the ingredient queries. The
//! pages and their rows are in `craft_pages.rs`.
//!
//! The selected skill's level text and its raise text (`RefreshCraftSkillDisplay`) are shown on the
//! category and recipe views.
//!
//! A drag on a recipe's row drags the page's `DragField` as the recipe's shortcut, which the
//! shortcut bar takes.
//!
//! Not modelled yet: the list-driven rows for a registry that is still empty (the lists answer
//! nothing until the server fills them), the level text on the skill rows, and the category sort
//! (`CraftSkillCategoryAdaptor::operator<` is not in the decompile: categories sort by their id
//! here).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_net::protocol::{ExaminationRequest, StringInfo};
use ac2_weenie::{CraftSkill, GameplayNotice, Recipe, SessionEvent};

use crate::drag::MSG_NOT_DRAGABLE;
use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::UiMessage;
use crate::panel::MSG_OPENED;
use crate::shortcut::SHORTCUT_RECIPE;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};

/// The craft views (`token_craftskillview`, `token_craftskillcategoryview`, `token_recipeview`).
pub const STATE_SKILL: u32 = tokenize("craftskillview");
pub const STATE_CATEGORY: u32 = tokenize("craftskillcategoryview");
pub const STATE_RECIPE: u32 = tokenize("recipeview");

/// The element names the panel uses (`CraftPage`'s children and the templates).
pub const CRAFT_PAGE: u32 = tokenize("CraftPage");
pub(crate) const CRAFT_TAB: u32 = tokenize("CraftTab");
pub(crate) const FORWARD_BUTTON: u32 = tokenize("ForwardButton");
pub(crate) const BACK_BUTTON: u32 = tokenize("BackButton");
pub(crate) const RAISE_CRAFT_BUTTON: u32 = tokenize("RaiseCraftButton");
pub(crate) const RECIPE_BUTTON: u32 = tokenize("RecipeButton");
pub(crate) const ENTRY_TEMPLATE: u32 = tokenize("EntryTemplate");
pub(crate) const CATEGORY_TEMPLATE: u32 = tokenize("CategoryTemplate");
pub(crate) const RECIPE_TEMPLATE: u32 = tokenize("RecipeTemplate");
pub(crate) const CATEGORY_HEADER: u32 = tokenize("CraftSkillCategoryHeader");
pub(crate) const RECIPE_HEADER: u32 = tokenize("RecipeHeader");
pub(crate) const NAME: u32 = tokenize("Name");
pub(crate) const RECIPE_ICON: u32 = tokenize("RecipeIcon");
pub(crate) const SELECTED_NAME: u32 = tokenize("SelectedName");
pub(crate) const SELECTED_ICON: u32 = tokenize("SelectedIcon");
pub(crate) const SELECTED_FIELD: u32 = tokenize("SelectedField");
pub(crate) const SKILL_LIST: u32 = tokenize("CraftSkillList");
pub(crate) const CATEGORY_LIST: u32 = tokenize("CraftSkillCategoryList");
pub(crate) const RECIPE_LIST: u32 = tokenize("RecipeList");
pub(crate) const TOKEN_DID: u32 = tokenize("did");
pub(crate) const TOKEN_SELECTED: u32 = tokenize("selected");
pub(crate) const TOKEN_ENABLED: u32 = tokenize("enabled");
pub(crate) const TOKEN_DISABLED: u32 = tokenize("disabled");
pub(crate) const SELECTED_LEVEL: u32 = tokenize("SelectedLevel");
/// A skill row's rating text and its states, the rating's string (`ID_Craft_Rating`) and the
/// ellipsis the long names end in (`ID_Ellipsis`); the symbols' initializers are in the admin
/// 7148 tree (`Tokenize`), the Legions addresses were matched by position in the same functions.
pub(crate) const LEVEL: u32 = tokenize("Level");
pub(crate) const TOKEN_BUFF: u32 = tokenize("buff");
pub(crate) const TOKEN_DEBUFF: u32 = tokenize("debuff");
pub(crate) const CRAFT_RATING: u32 = tokenize("ID_Craft_Rating");
pub(crate) const ELLIPSIS: u32 = tokenize("ID_Ellipsis");
/// The level text's strings (`ID_Craft_SkillLevelNormal`, `..Buffed`, `..Debuffed`), the raise
/// text's (`ID_Craft_CantRaise`, `ID_Craft_AvailableXPRaise`) and their variables (`level`,
/// `value`, `xp`, `cost`). The string ids are the names's tokens, as the UI table keys them.
pub(crate) const SKILL_LEVEL_NORMAL: u32 = tokenize("ID_Craft_SkillLevelNormal");
pub(crate) const SKILL_LEVEL_BUFFED: u32 = tokenize("ID_Craft_SkillLevelBuffed");
pub(crate) const SKILL_LEVEL_DEBUFFED: u32 = tokenize("ID_Craft_SkillLevelDebuffed");
pub(crate) const CANT_RAISE: u32 = tokenize("ID_Craft_CantRaise");
pub(crate) const AVAILABLE_XP_RAISE: u32 = tokenize("ID_Craft_AvailableXPRaise");
pub(crate) const TOKEN_LEVEL: u32 = tokenize("level");
pub(crate) const TOKEN_VALUE: u32 = tokenize("value");
pub(crate) const TOKEN_XP: u32 = tokenize("xp");
pub(crate) const TOKEN_COST: u32 = tokenize("cost");

/// The mouse actions the rows answer (`0x40000004` left, `0x40000005` right, `0x40000007` double,
/// `0x40000008`): a press on a row selects it, a double click also opens it.
const PRESS_LEFT: u32 = 0x4000_0004;
const PRESS_RIGHT: u32 = 0x4000_0005;
const DOUBLE_CLICK: u32 = 0x4000_0007;
const PRESS_OTHER: u32 = 0x4000_0008;

/// The examination types `SelectCraftSkill`, `SelectCraftSkillCategory` and `SelectRecipe` send.
const EXAMINE_CRAFT_SKILL: u32 = 0x4000_000B;
const EXAMINE_CRAFT_CATEGORY: u32 = 0x4000_000C;
const EXAMINE_RECIPE: u32 = 0x4000_0007;

/// What the panel reads from the player and the DAT.
pub trait CraftSource {
    /// `StringInfo::GetString` of a server string.
    fn text(&self, info: &StringInfo) -> String;
    /// `gmCWInterface::InqCraftSkillList`: the player's craft skills.
    fn craft_skills(&self) -> Vec<CraftSkill>;
    /// `gmCWInterface::InqCraftSkillAdaptor`.
    fn craft_skill(&self, skill: u32) -> Option<CraftSkill>;
    /// `gmCWInterface::InqCraftSkillCategoryName`.
    fn category_name(&self, category: u32) -> Option<StringInfo>;
    /// `gmCWInterface::InqRecipeList`: the recipes of `skill` the player knows.
    fn recipes(&self, skill: u32) -> Vec<Recipe>;
    /// `gmCWInterface::IsCraftSkillRaiseable`.
    fn is_raiseable(&self, skill: u32) -> bool;
    /// `gmCWInterface::CanCraftSkillBeRaised`.
    fn can_raise(&self, skill: u32) -> bool;
    /// `gmCWInterface::InqCraftSkillLevel`: the raw and the enchanted level of `skill`.
    fn skill_level(&self, skill: u32) -> Option<(u32, u32)>;
    /// `gmCWInterface::GetAvailableCraftExperience`.
    fn available_experience(&self) -> u64;
    /// A UI string of the UI table, with its integer variables (token, value).
    fn ui_text(&self, string_id: u32, vars: &[(u32, i64)]) -> String;
}

/// A request the panel makes of the player's interface, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CraftCommand {
    /// `gmCWInterface::SetSelection(INVALID_IID)`.
    ClearSelection,
    /// `gmCWInterface::SetExaminationRequest`.
    Examine(ExaminationRequest),
    /// `gmCWInterface::OpenRecipe`: the recipe's ingredient panel opens.
    OpenRecipe(u32),
    /// `gmCWInterface::RequestRaiseCraftSkill`.
    RaiseSkill(u32),
}

/// `gmCraftUI`'s state.
#[derive(Debug, Clone)]
pub struct CraftUi {
    pub(crate) layout: Arc<LayoutDesc>,
    pub(crate) page: usize,
    /// `m_currentState`: one of the view tokens, or 0 before the panel opens.
    pub(crate) state: u32,
    /// `m_selectedElement`: the row that shows as selected.
    pub(crate) selected_element: Option<usize>,
    /// `m_selectedCraftSkill`, `m_selectedCategory`, `m_selectedRecipe`.
    pub(crate) selected_skill: u32,
    pub(crate) selected_category: u32,
    pub(crate) selected_recipe: u32,
    /// `m_categories`: the categories of the selected skill's recipes, in order, with their names.
    pub(crate) categories: Vec<(u32, StringInfo)>,
    /// `m_recipes`: the recipes of the selected skill.
    pub(crate) recipes: Vec<Recipe>,
    /// `m_craftSkillHash`, `m_categoryHash`, `m_recipeHash`: the rows made, by DID.
    pub(crate) skill_rows: HashMap<u32, usize>,
    pub(crate) category_rows: HashMap<u32, usize>,
    pub(crate) recipe_rows: HashMap<u32, usize>,
    commands: Vec<CraftCommand>,
}

impl CraftUi {
    /// `gmCraftUI::gmCraftUI` over the `CraftPage` element of the toolbar layout.
    pub fn create(layout: &Arc<LayoutDesc>, page: usize) -> Self {
        Self {
            layout: Arc::clone(layout),
            page,
            state: 0,
            selected_element: None,
            selected_skill: 0,
            selected_category: 0,
            selected_recipe: 0,
            categories: Vec::new(),
            recipes: Vec::new(),
            skill_rows: HashMap::new(),
            category_rows: HashMap::new(),
            recipe_rows: HashMap::new(),
            commands: Vec::new(),
        }
    }

    /// The view the page shows (one of the `STATE_*` tokens; 0 before the panel opens).
    pub fn state(&self) -> u32 {
        self.state
    }

    /// The `CraftPage` element the panel's rows and buttons live in.
    pub fn page(&self) -> usize {
        self.page
    }

    /// Forward on the skills view: the skill is selected and its categories show.
    pub fn open_skill(&mut self, tree: &mut UiTree, source: &dyn CraftSource, skill: u32) {
        self.select_craft_skill(source, skill);
        self.set_state(tree, source, STATE_CATEGORY);
    }

    /// Forward on the categories view: the category is selected and its recipes show.
    pub fn open_category(&mut self, tree: &mut UiTree, source: &dyn CraftSource, category: u32) {
        self.select_category(category);
        self.set_state(tree, source, STATE_RECIPE);
    }

    /// `ListenToMessage`'s `not_dragable` on a recipe row: `InqRecipeAdaptor`, `GetRecipeIcon`,
    /// then the drag field set up as the recipe's shortcut and dragged
    /// (`IUIManager::StartDragandDrop(m_dragField, 0, 0)`).
    fn drag_recipe(&mut self, tree: &mut UiTree, source: &dyn CraftSource, did: u32) {
        let Some(recipe) = self.recipes.iter().find(|r| r.did == did) else {
            return;
        };
        let icon = self.recipe_icon(source, recipe);
        let Some(field) = tree.child_recursive(self.page, tokenize("DragField")) else {
            return;
        };
        tree.set_attribute_id(field, tokenize("Token_ShortcutData"), did);
        tree.set_attribute_id(field, tokenize("Token_ShortcutType"), SHORTCUT_RECIPE);
        tree.set_image(field, icon, 2);
        tree.start_drag(field);
    }

    /// The requests the panel made since the last call, in order.
    pub fn take_commands(&mut self) -> Vec<CraftCommand> {
        std::mem::take(&mut self.commands)
    }

    /// `gmCraftUI::SetState`: the page shows `state`, and the panel refreshes.
    pub fn set_state(&mut self, tree: &mut UiTree, source: &dyn CraftSource, state: u32) {
        tree.set_state(self.page, state);
        self.state = state;
        self.refresh_panel(tree, source);
    }

    /// `gmCraftUI::RefreshCraftPanel`: the recipes of the selected skill, then the current view.
    pub fn refresh_panel(&mut self, tree: &mut UiTree, source: &dyn CraftSource) {
        if self.selected_skill != 0 {
            self.recipes.clear();
            self.recipes = source.recipes(self.selected_skill);
        }
        match self.state {
            STATE_SKILL => self.refresh_skill_page(tree, source),
            STATE_CATEGORY => self.refresh_category_page(tree, source),
            STATE_RECIPE => self.refresh_recipe_page(tree, source),
            _ => {}
        }
    }

    /// `gmCraftUI::RecvNotice_UpdateCraftPanel` (`CM_Craft::SendNotice_UpdateCraftPanel`).
    pub fn update_craft_panel(&mut self, tree: &mut UiTree, source: &dyn CraftSource) {
        self.refresh_panel(tree, source);
    }

    /// `gmCraftUI::RecvNotice_ExaminationRequestChanged` (Legions 0x44af30): the row of the
    /// examined craft skill (by DID), category (by enum) or recipe (by DID) shows as selected,
    /// the previous one returns to its default state; another request, or one without a row,
    /// leaves none selected.
    pub fn examination_request_changed(&mut self, tree: &mut UiTree, request: &ExaminationRequest) {
        let row = match request.kind {
            EXAMINE_CRAFT_SKILL => self.skill_rows.get(&request.data_id),
            EXAMINE_CRAFT_CATEGORY => self.category_rows.get(&request.data_enum),
            EXAMINE_RECIPE => self.recipe_rows.get(&request.data_id),
            _ => None,
        }
        .copied();
        if let Some(previous) = self.selected_element.take() {
            let default = tree.default_state_of(previous);
            tree.set_state(previous, default);
        }
        if let Some(row) = row {
            self.selected_element = Some(row);
            tree.set_state(row, TOKEN_SELECTED);
        }
    }

    /// `gmCraftUI` on the frame: the craft notices in `events` refresh the panel
    /// (`RecvNotice_UpdateCraftPanel`) or select the examined row
    /// (`RecvNotice_ExaminationRequestChanged`), then its `messages` run [`CraftUi::message`]. Returns the
    /// requests the frame made, in order.
    pub fn frame_pass(
        &mut self,
        tree: &mut UiTree,
        source: &dyn CraftSource,
        events: &[SessionEvent],
        messages: &[UiMessage],
    ) -> Vec<CraftCommand> {
        for event in events {
            match event {
                SessionEvent::Notice(GameplayNotice::UpdateCraftPanel) => {
                    self.update_craft_panel(tree, source);
                }
                SessionEvent::Notice(GameplayNotice::ExaminationRequestChanged(request)) => {
                    self.examination_request_changed(tree, request);
                }
                _ => {}
            }
        }
        for message in messages {
            self.message(tree, source, message);
        }
        self.take_commands()
    }

    /// `gmCraftUI::ListenToMessage`: the buttons and rows the panel answers.
    pub fn message(&mut self, tree: &mut UiTree, source: &dyn CraftSource, message: &UiMessage) {
        let Some(element) = message.element else {
            return;
        };
        let did = match tree.el(element).attribute(TOKEN_DID) {
            Some(AttributeValue::Id(did)) => *did,
            _ => 0,
        };
        let (id, kind, action) = (message.element_id, message.message, message.data as u32);
        let release = kind == MSG_BUTTON_RELEASE_ON;
        let press = kind == MSG_MOUSE_PRESS_TOP && is_press(action);
        if id == FORWARD_BUTTON && release {
            match self.state {
                STATE_SKILL => {
                    self.select_craft_skill(source, did);
                    self.set_state(tree, source, STATE_CATEGORY);
                }
                STATE_CATEGORY => {
                    self.select_category(did);
                    self.set_state(tree, source, STATE_RECIPE);
                }
                STATE_RECIPE => {
                    self.select_recipe(did);
                    self.commands.push(CraftCommand::OpenRecipe(did));
                }
                _ => {}
            }
        } else if id == BACK_BUTTON && release {
            match self.state {
                STATE_CATEGORY => self.set_state(tree, source, STATE_SKILL),
                STATE_RECIPE => self.set_state(tree, source, STATE_CATEGORY),
                _ => {}
            }
        } else if id == RECIPE_HEADER && press {
            self.select_category(did);
            if action == DOUBLE_CLICK {
                self.set_state(tree, source, STATE_CATEGORY);
            }
        } else if id == CATEGORY_HEADER && press {
            self.select_craft_skill(source, did);
            if action == DOUBLE_CLICK {
                self.set_state(tree, source, STATE_SKILL);
            }
        } else {
            self.row_message(tree, source, id, kind, action, did);
        }
    }

    /// `ListenToMessage`'s rows, the recipe button, the raise button and the craft tab.
    fn row_message(
        &mut self,
        tree: &mut UiTree,
        source: &dyn CraftSource,
        id: u32,
        kind: u32,
        action: u32,
        did: u32,
    ) {
        let double = action == DOUBLE_CLICK;
        let press = kind == MSG_MOUSE_PRESS_TOP && is_press(action);
        if id == ENTRY_TEMPLATE && press && self.state == STATE_SKILL {
            self.select_craft_skill(source, did);
            if double {
                self.set_state(tree, source, STATE_CATEGORY);
            }
            return;
        }
        if id == CATEGORY_TEMPLATE && press && self.state == STATE_CATEGORY {
            self.select_category(did);
            if double {
                self.set_state(tree, source, STATE_RECIPE);
            }
            return;
        }
        if id == RECIPE_TEMPLATE && press && self.state == STATE_RECIPE {
            self.select_recipe(did);
            if double {
                self.commands.push(CraftCommand::OpenRecipe(did));
            }
            return;
        }
        if id == RECIPE_BUTTON && kind == MSG_BUTTON_RELEASE_ON && self.state == STATE_RECIPE {
            self.select_recipe(did);
            self.commands.push(CraftCommand::OpenRecipe(did));
            return;
        }
        // A drag on a recipe row (the row is not dragable): the page's `DragField` takes the
        // recipe as a shortcut (`Token_ShortcutData` the DID, `Token_ShortcutType` 0x40000006)
        // and its icon (draw mode 2), and is dragged instead, so the shortcut bar can take it.
        if id == RECIPE_TEMPLATE && kind == MSG_NOT_DRAGABLE {
            self.drag_recipe(tree, source, did);
            return;
        }
        if id == RAISE_CRAFT_BUTTON && kind == MSG_BUTTON_RELEASE_ON {
            self.commands
                .push(CraftCommand::RaiseSkill(self.selected_skill));
            return;
        }
        // The craft tab opens the panel on the skills: no skill selected, the view is the skills.
        if id == CRAFT_TAB && kind == MSG_OPENED {
            self.clear_selection_info();
            self.select_craft_skill(source, 0);
            self.set_state(tree, source, STATE_SKILL);
        }
    }

    /// `gmCraftUI::SelectCraftSkill`: the skill's recipes and categories, and its examination.
    pub(crate) fn select_craft_skill(&mut self, source: &dyn CraftSource, skill: u32) {
        self.selected_skill = skill;
        self.selected_category = 0;
        self.selected_recipe = 0;
        self.recipes.clear();
        if skill != 0 {
            self.recipes = source.recipes(skill);
        }
        self.update_categories(source);
        if skill != 0 {
            self.commands.push(CraftCommand::ClearSelection);
            self.commands.push(CraftCommand::Examine(examine(
                EXAMINE_CRAFT_SKILL,
                0,
                skill,
            )));
        }
    }

    /// `gmCraftUI::SelectCraftSkillCategory`: the category's examination.
    pub(crate) fn select_category(&mut self, category: u32) {
        self.selected_category = category;
        self.selected_recipe = 0;
        if category != 0 {
            self.commands.push(CraftCommand::ClearSelection);
            self.commands.push(CraftCommand::Examine(examine(
                EXAMINE_CRAFT_CATEGORY,
                category,
                0,
            )));
        }
    }

    /// `gmCraftUI::SelectRecipe`: the recipe's examination (and the selected recipe, even 0).
    pub(crate) fn select_recipe(&mut self, recipe: u32) {
        self.selected_recipe = recipe;
        if recipe != 0 {
            self.commands.push(CraftCommand::ClearSelection);
            self.commands
                .push(CraftCommand::Examine(examine(EXAMINE_RECIPE, 0, recipe)));
        }
    }

    /// `gmCraftUI::ClearSelectionInfo`: the rows made are forgotten, nothing is selected.
    pub(crate) fn clear_selection_info(&mut self) {
        self.skill_rows.clear();
        self.category_rows.clear();
        self.recipe_rows.clear();
        self.selected_element = None;
    }

    /// `gmCraftUI::SelectElement`: the row shows as selected (its previous one returns to its
    /// default state).
    pub(crate) fn select_element(&mut self, tree: &mut UiTree, row: usize) {
        if let Some(previous) = self.selected_element {
            let default = tree.default_state_of(previous);
            tree.set_state(previous, default);
        }
        self.selected_element = Some(row);
        tree.set_state(row, TOKEN_SELECTED);
    }
}

/// Whether a mouse action on a row is a press (`MakeCraftSkill`'s data checks).
fn is_press(action: u32) -> bool {
    matches!(
        action,
        PRESS_LEFT | PRESS_RIGHT | DOUBLE_CLICK | PRESS_OTHER
    )
}

/// An examination request for `kind`: the category's enum or the skill's or recipe's DID.
fn examine(kind: u32, data_enum: u32, data_id: u32) -> ExaminationRequest {
    ExaminationRequest {
        kind,
        data_enum,
        data_id,
        ..ExaminationRequest::default()
    }
}
