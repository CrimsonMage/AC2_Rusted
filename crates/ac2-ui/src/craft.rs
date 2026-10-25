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
//! Not modelled yet: the list-driven rows for a registry that is still empty (the lists answer
//! nothing until the server fills them), the level text and its states, the raise cost text, the
//! names' truncation with an ellipsis, the drag of a recipe's icon to the shortcut bar, and the
//! category sort (`CraftSkillCategoryAdaptor::operator<` is not in the decompile: categories sort
//! by their id here).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_net::protocol::{ExaminationRequest, StringInfo};
use ac2_weenie::{CraftSkill, Recipe};

use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::UiMessage;
use crate::panel::MSG_OPENED;
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
