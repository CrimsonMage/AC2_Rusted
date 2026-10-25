//! `gmCraftUI`'s pages and rows: `RefreshCraftSkillPage`, `RefreshCraftSkillCategoryPage` and
//! `RefreshRecipePage` fill the lists of their view (`MakeCraftSkill`, `MakeCraftSkillCategory`,
//! `MakeRecipe`), `RefreshCraftSkillDisplay` shows the selected skill, and `UpdateCategories`
//! collects the categories of the selected skill's recipes.

use ac2_net::protocol::StringInfo;
use ac2_weenie::{CraftSkill, Recipe};

use crate::craft::{
    CraftSource, CraftUi, BACK_BUTTON, CATEGORY_HEADER, CATEGORY_LIST, CATEGORY_TEMPLATE,
    ENTRY_TEMPLATE, FORWARD_BUTTON, NAME, RAISE_CRAFT_BUTTON, RECIPE_BUTTON, RECIPE_HEADER,
    RECIPE_ICON, RECIPE_LIST, RECIPE_TEMPLATE, SELECTED_FIELD, SELECTED_ICON, SELECTED_NAME,
    SKILL_LIST, STATE_CATEGORY, STATE_RECIPE, STATE_SKILL, TOKEN_DID, TOKEN_DISABLED,
    TOKEN_ENABLED, TOKEN_SELECTED,
};
use crate::element::UiTree;
use crate::token::BUTTON_GHOSTED;

impl CraftUi {
    /// `UIFramework::CreateElement(0x41000003, template)`: a template of the toolbar layout.
    fn create_template(&self, tree: &mut UiTree, template: u32) -> Option<usize> {
        tree.create_nested_element(&self.layout, template)
    }

    /// A named element of the page, looked up when the page needs it (`GetChildRecursive`).
    fn child(&self, tree: &UiTree, name: u32) -> Option<usize> {
        tree.child_recursive(self.page, name)
    }

    /// `gmCraftUI::RefreshCraftSkillPage`: a row per craft skill in the skills view.
    pub(crate) fn refresh_skill_page(&mut self, tree: &mut UiTree, source: &dyn CraftSource) {
        if self.state != STATE_SKILL {
            return;
        }
        self.clear_selection_info();
        let Some(list) = self.child(tree, SKILL_LIST) else {
            return;
        };
        tree.list_flush(list);
        for skill in source.craft_skills() {
            let Some(row) = self.create_template(tree, ENTRY_TEMPLATE) else {
                continue;
            };
            self.make_skill(tree, source, &skill, row);
            tree.list_add_item(list, row);
            self.skill_rows.insert(skill.did, row);
        }
        tree.list_multi_op_complete(list);
    }

    /// `gmCraftUI::RefreshCraftSkillCategoryPage`: the selected skill in the header, then a row per
    /// category of its recipes. A missing header leaves the page as it is.
    pub(crate) fn refresh_category_page(&mut self, tree: &mut UiTree, source: &dyn CraftSource) {
        if self.state != STATE_CATEGORY || self.selected_skill == 0 {
            return;
        }
        self.clear_selection_info();
        if let Some(skill) = source.craft_skill(self.selected_skill) {
            let Some(header) = self.child(tree, CATEGORY_HEADER) else {
                return;
            };
            self.make_skill(tree, source, &skill, header);
            self.skill_rows.insert(skill.did, header);
            self.refresh_skill_display(tree, source, &skill);
        }
        let Some(list) = self.child(tree, CATEGORY_LIST) else {
            return;
        };
        tree.list_flush(list);
        for (category, name) in self.categories.clone() {
            let Some(row) = self.create_template(tree, CATEGORY_TEMPLATE) else {
                continue;
            };
            self.make_category(tree, source, category, &name, row, true);
            tree.list_add_item(list, row);
            self.category_rows.insert(category, row);
        }
        tree.list_multi_op_complete(list);
    }

    /// `gmCraftUI::RefreshRecipePage`: the selected category in the header, then the shown recipes
    /// of that category, and the skill's display.
    pub(crate) fn refresh_recipe_page(&mut self, tree: &mut UiTree, source: &dyn CraftSource) {
        if self.state != STATE_RECIPE || self.selected_skill == 0 || self.selected_category == 0 {
            return;
        }
        self.clear_selection_info();
        let Some(list) = self.child(tree, RECIPE_LIST) else {
            return;
        };
        tree.list_flush(list);
        let category = self.selected_category;
        let name = self
            .categories
            .iter()
            .find(|(c, _)| *c == category)
            .map(|(_, name)| name.clone());
        if let Some(name) = name {
            let Some(header) = self.child(tree, RECIPE_HEADER) else {
                return;
            };
            self.make_category(tree, source, category, &name, header, false);
            self.category_rows.insert(category, header);
        }
        for recipe in self.recipes.clone() {
            if !recipe.shown || recipe.craft_skill_category != category {
                continue;
            }
            let Some(row) = self.create_template(tree, RECIPE_TEMPLATE) else {
                continue;
            };
            self.make_recipe(tree, source, &recipe, row);
            tree.list_add_item(list, row);
            self.recipe_rows.insert(recipe.did, row);
        }
        tree.list_multi_op_complete(list);
        if let Some(skill) = source.craft_skill(self.selected_skill) {
            self.refresh_skill_display(tree, source, &skill);
        }
    }

    /// `gmCraftUI::RefreshCraftSkillDisplay`: the selected skill's name, icon and raise button, on
    /// the category and recipe views. The raise text and the level text are not shown yet (their
    /// retail string ids are not in the decompile).
    pub(crate) fn refresh_skill_display(
        &mut self,
        tree: &mut UiTree,
        source: &dyn CraftSource,
        skill: &CraftSkill,
    ) {
        if self.state != STATE_CATEGORY && self.state != STATE_RECIPE {
            return;
        }
        if let Some(name) = self.child(tree, SELECTED_NAME) {
            let text = source.text(&skill.name);
            tree.set_text(name, &text);
        }
        if let Some(icon) = self.child(tree, SELECTED_ICON) {
            set_icon(tree, icon, skill.icon, 2);
        }
        if let Some(field) = self.child(tree, SELECTED_FIELD) {
            tree.set_state(field, TOKEN_SELECTED);
        }
        if let Some(raise) = self.child(tree, RAISE_CRAFT_BUTTON) {
            let can = source.can_raise(skill.did);
            tree.set_bool(raise, BUTTON_GHOSTED, !can);
        }
    }

    /// `gmCraftUI::MakeCraftSkill`: a skill's row (or the header): its DID on the row and its
    /// back button, its name in the enabled or disabled state (a modifier of at least 1.0 enables
    /// it), and the row selected when it is the selected skill.
    pub(crate) fn make_skill(
        &mut self,
        tree: &mut UiTree,
        source: &dyn CraftSource,
        skill: &CraftSkill,
        row: usize,
    ) {
        tree.set_attribute_id(row, TOKEN_DID, skill.did);
        if let Some(button) = tree.child_recursive(row, BACK_BUTTON) {
            tree.set_attribute_id(button, TOKEN_DID, skill.did);
        }
        if let Some(name) = tree.child_recursive(row, NAME) {
            let state = if skill.xp_mod >= 1.0 {
                TOKEN_ENABLED
            } else {
                TOKEN_DISABLED
            };
            tree.set_state(name, state);
            let text = source.text(&skill.name);
            tree.set_text(name, &text);
        }
        if self.selected_skill == skill.did {
            self.select_element(tree, row);
        }
    }

    /// `gmCraftUI::MakeCraftSkillCategory`: a category's row (or the recipes' header): its DID, its
    /// forward button (`forward`) or back button, its name, and the row selected when it is the
    /// selected category.
    pub(crate) fn make_category(
        &mut self,
        tree: &mut UiTree,
        source: &dyn CraftSource,
        category: u32,
        name: &StringInfo,
        row: usize,
        forward: bool,
    ) {
        tree.set_attribute_id(row, TOKEN_DID, category);
        let button = if forward { FORWARD_BUTTON } else { BACK_BUTTON };
        if let Some(button) = tree.child_recursive(row, button) {
            tree.set_attribute_id(button, TOKEN_DID, category);
        }
        if let Some(text) = tree.child_recursive(row, NAME) {
            let string = source.text(name);
            tree.set_text(text, &string);
            if self.selected_category == category {
                self.select_element(tree, row);
            }
        }
    }

    /// `gmCraftUI::MakeRecipe`: a recipe's row: its icon (the recipe's, or the skill's), its
    /// button and name, and the row selected when it is the selected recipe.
    pub(crate) fn make_recipe(
        &mut self,
        tree: &mut UiTree,
        source: &dyn CraftSource,
        recipe: &Recipe,
        row: usize,
    ) {
        let (Some(icon), Some(button)) = (
            tree.child_recursive(row, RECIPE_ICON),
            tree.child_recursive(row, RECIPE_BUTTON),
        ) else {
            return;
        };
        tree.set_attribute_id(row, TOKEN_DID, recipe.did);
        tree.set_attribute_id(button, TOKEN_DID, recipe.did);
        let image = self.recipe_icon(source, recipe);
        set_icon(tree, icon, image, 0);
        if let Some(text) = tree.child_recursive(row, NAME) {
            let string = source.text(&recipe.name);
            tree.set_text(text, &string);
            if self.selected_recipe == recipe.did {
                self.select_element(tree, row);
            }
        }
    }

    /// `gmCraftUI::GetRecipeIcon`: the recipe's icon, or the selected skill's when it has none.
    fn recipe_icon(&self, source: &dyn CraftSource, recipe: &Recipe) -> u32 {
        if recipe.icon_did != 0 {
            return recipe.icon_did;
        }
        source
            .craft_skill(self.selected_skill)
            .map_or(0, |skill| skill.icon)
    }

    /// `gmCraftUI::UpdateCategories`: the categories of the selected skill's recipes, each once,
    /// with its name (sorted by id: `CraftSkillCategoryAdaptor::operator<` is not in the
    /// decompile).
    pub(crate) fn update_categories(&mut self, source: &dyn CraftSource) {
        self.categories.clear();
        for recipe in &self.recipes {
            let category = recipe.craft_skill_category;
            let Some(name) = source.category_name(category) else {
                continue;
            };
            if self.categories.iter().any(|(c, _)| *c == category) {
                continue;
            }
            let at = self
                .categories
                .iter()
                .position(|(c, _)| *c > category)
                .unwrap_or(self.categories.len());
            self.categories.insert(at, (category, name));
        }
    }
}

/// `UIElement::SetImage` / `ClearImage` of an icon element.
fn set_icon(tree: &mut UiTree, icon: usize, file: u32, draw_mode: u32) {
    if file == 0 {
        tree.clear_image(icon);
    } else {
        tree.set_image(icon, file, draw_mode);
    }
}
