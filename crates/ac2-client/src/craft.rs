//! The crafting calls on the player's script (`ggmCWInterface::CallIFun_*`). The craft panels'
//! queries are the script's and live in `ac2_script::host::craft_queries`; the client keeps the
//! one-line delegates the panels call. The requests are the IFun's stubs: the VM packs them and
//! the session sends them. Without the script VM every call answers `None` (or nothing).

use ac2_net::protocol::StringInfo;
use ac2_script::host::craft_queries::CraftQueries;
use ac2_script::host::{IFunArg, IFunResult, IFunReturn};
use ac2_weenie::{CraftSkill, Ingredient, Recipe, SessionEvent};

use crate::client::Client;
use crate::gameplay::Command;

impl Client {
    /// One craft query: its IFun on the player's script, the notices it raised kept for the next
    /// frame. `None` without the VM, or when the call fails.
    pub(crate) fn script_query(
        &mut self,
        function: u32,
        args: &[IFunArg],
        ret: IFunReturn,
    ) -> Option<IFunResult> {
        ac2_script::host::ui_calls::script_query(self, function, args, ret)
    }

    /// A request's IFun on the player's script (a craft or skill request): its stub sends the
    /// request; the events it raised come back (`ui_calls::script_request`).
    pub(crate) fn script_request(&mut self, function: u32, args: &[IFunArg]) -> Vec<SessionEvent> {
        ac2_script::host::ui_calls::script_request(self, function, args)
    }

    /// `gmCWInterface::InqCraftSkillList`: the player's craft skills, from the script.
    pub fn craft_skills(&mut self) -> Option<Vec<CraftSkill>> {
        CraftQueries::craft_skills(self)
    }

    /// `gmCWInterface::InqCraftSkillAdaptor`: craft skill `skill`.
    pub fn craft_skill(&mut self, skill: u32) -> Option<CraftSkill> {
        CraftQueries::craft_skill(self, skill)
    }

    /// `gmCWInterface::InqCraftSkillLevel`: the raw and the enchanted level of `skill`.
    pub fn craft_skill_level(&mut self, skill: u32) -> Option<(u32, u32)> {
        CraftQueries::craft_skill_level(self, skill)
    }

    /// `gmCWInterface::InqCraftSkillCategoryName`: the category name of `skill`.
    pub fn craft_skill_category(&mut self, skill: u32) -> Option<StringInfo> {
        CraftQueries::craft_skill_category(self, skill)
    }

    /// `gmCWInterface::CanCraftSkillBeRaised` (the skill can be raised now).
    pub fn can_craft_skill_be_raised(&mut self, skill: u32) -> Option<bool> {
        CraftQueries::can_craft_skill_be_raised(self, skill)
    }

    /// `gmCWInterface::IsCraftSkillRaiseable`.
    pub fn is_craft_skill_raiseable(&mut self, skill: u32) -> Option<bool> {
        CraftQueries::is_craft_skill_raiseable(self, skill)
    }

    /// `gmCWInterface::InqRecipeList`: the recipes of craft skill `skill` the player knows.
    pub fn recipes(&mut self, skill: u32) -> Option<Vec<Recipe>> {
        CraftQueries::recipes(self, skill)
    }

    /// `gmCWInterface::InqRecipeAdaptor`: recipe `recipe`.
    pub fn recipe(&mut self, recipe: u32) -> Option<Recipe> {
        CraftQueries::recipe(self, recipe)
    }

    /// `gmCWInterface::InqRecipeIngredients`: the recipe's ingredient slots and the ordinal mask
    /// the spinner value leaves filled.
    pub fn recipe_ingredients(
        &mut self,
        recipe: u32,
        spinner: u32,
    ) -> Option<(u32, Vec<Ingredient>)> {
        CraftQueries::recipe_ingredients(self, recipe, spinner)
    }

    /// `gmCWInterface::InqRecipeDifficulty`: the difficulty and the name colour at `spinner`.
    pub fn recipe_difficulty(&mut self, recipe: u32, spinner: u32) -> Option<(u32, u32)> {
        CraftQueries::recipe_difficulty(self, recipe, spinner)
    }

    /// `gmCWInterface::InqRecipeCost`: the cost of `recipe` at `spinner`.
    pub fn recipe_cost(&mut self, recipe: u32, spinner: u32) -> Option<i32> {
        CraftQueries::recipe_cost(self, recipe, spinner)
    }

    /// `gmCWInterface::DoesIngredientAutoPopulate`: whether slot `ordinal` of `recipe` fills
    /// itself.
    pub fn ingredient_auto_populates(&mut self, recipe: u32, ordinal: u32) -> Option<bool> {
        CraftQueries::ingredient_auto_populates(self, recipe, ordinal)
    }

    /// `gmCWInterface::CheckIngredient`: whether `object` may fill slot `ordinal` of `recipe`.
    pub fn check_ingredient(
        &mut self,
        recipe: u32,
        ordinal: u32,
        object: u64,
        spinner: u32,
    ) -> Option<bool> {
        CraftQueries::check_ingredient(self, recipe, ordinal, object, spinner)
    }

    /// `gmCWInterface::GetAvailableCraftExperience`: the craft experience the player may use.
    pub fn available_craft_experience(&mut self) -> Option<u64> {
        CraftQueries::available_craft_experience(self)
    }
}

impl Client {
    /// The craft commands of [`Command`] (`gmCWInterface`'s craft requests), each one's IFun.
    pub(crate) fn craft_command(&mut self, command: &Command) -> Vec<SessionEvent> {
        match command {
            Command::OpenRecipe(recipe) => {
                self.script_request(crate::scripts::IFUN_OPEN_RECIPE, &[IFunArg::Int(*recipe)])
            }
            Command::AddIngredient {
                recipe,
                ordinal,
                object,
                spinner,
            } => self.script_request(
                crate::scripts::IFUN_REQUEST_ADD_INGREDIENT,
                &[
                    IFunArg::Int(*recipe),
                    IFunArg::Int(*ordinal),
                    IFunArg::Long(*object),
                    IFunArg::Int(*spinner),
                ],
            ),
            Command::ExecuteCraft {
                recipe,
                spinner,
                ingredients,
            } => self.script_request(
                crate::scripts::IFUN_REQUEST_EXECUTE_CRAFT,
                &[
                    IFunArg::Int(*recipe),
                    IFunArg::Int(*spinner),
                    IFunArg::ingredient_table(ingredients),
                ],
            ),
            Command::RaiseCraftSkill(skill) => self.script_request(
                crate::scripts::IFUN_REQUEST_RAISE_CRAFT_SKILL,
                &[IFunArg::Int(*skill)],
            ),
            _ => Vec::new(),
        }
    }
}
