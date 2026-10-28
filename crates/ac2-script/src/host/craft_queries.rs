//! The crafting queries on the player's script (`ggmCWInterface::CallIFun_*`): the craft
//! skills, recipes, ingredients and craft experience the craft panels read. The VM answers from the
//! player's qualities and the DAT, and returns the adaptors it filled (the natives come back as
//! [`CraftSkill`], [`Recipe`] and [`Ingredient`]). Each query answers `None` (or `Err`) without
//! the VM's answer. Written against [`ScriptPort`]; the craft requests are the IFuns' stubs
//! ([`ui_calls::script_request`]).

use ac2_net::protocol::StringInfo;
use ac2_weenie::{CraftSkill, Ingredient, Recipe};

use super::ifun::{IFunArg, IFunResult, IFunReturn, OutValue};
use super::ifun_ids::*;
use super::port::ScriptPort;
use super::ui_calls;
use crate::native::NativeObject;

/// `RList`: the native a list out parameter is passed as.
const R_LIST: u32 = 0x65;
/// `CraftSkillAdaptor`, `RecipeAdaptor` and `StringInfo` (the natives the out parameters are
/// filled with).
const CRAFT_SKILL_ADAPTOR: u32 = 0x33A5;
const RECIPE_ADAPTOR: u32 = 0xAA;
const STRING_INFO: u32 = 0x6A;

/// The out value at `n` (the out parameters in order, the list and natives among them).
fn out(result: &IFunResult, n: usize) -> Option<&OutValue> {
    result.outs.get(n)?.as_ref()
}

/// The native an out parameter holds.
fn native(result: &IFunResult, n: usize) -> Option<&NativeObject> {
    match out(result, n)? {
        OutValue::Vm(native) => Some(native),
        _ => None,
    }
}

/// The natives a list out parameter holds.
fn items(result: &IFunResult, n: usize) -> Option<&[NativeObject]> {
    match out(result, n)? {
        OutValue::Items(items) => Some(items),
        _ => None,
    }
}

fn craft_skill_of(native: &NativeObject) -> Option<CraftSkill> {
    match native {
        NativeObject::CraftSkill(s) => Some((**s).clone()),
        _ => None,
    }
}

fn recipe_of(native: &NativeObject) -> Option<Recipe> {
    match native {
        NativeObject::Recipe(r) => Some((**r).clone()),
        _ => None,
    }
}

fn ingredient_of(native: &NativeObject) -> Option<Ingredient> {
    match native {
        NativeObject::Ingredient(i) => Some((**i).clone()),
        _ => None,
    }
}

/// The craft queries, on any [`ScriptPort`] (the client implements the port).
pub trait CraftQueries: ScriptPort {
    /// `gmCWInterface::InqCraftSkillList`: the player's craft skills, from the script.
    fn craft_skills(&mut self) -> Option<Vec<CraftSkill>> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_CRAFT_SKILL_LIST,
            &[IFunArg::Filled(R_LIST)],
            IFunReturn::Word,
        )?;
        Some(
            items(&result, 0)?
                .iter()
                .filter_map(craft_skill_of)
                .collect(),
        )
    }

    /// `gmCWInterface::InqCraftSkillAdaptor`: craft skill `skill`.
    fn craft_skill(&mut self, skill: u32) -> Option<CraftSkill> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_CRAFT_SKILL_ADAPTOR,
            &[IFunArg::Int(skill), IFunArg::Filled(CRAFT_SKILL_ADAPTOR)],
            IFunReturn::Word,
        )?;
        craft_skill_of(native(&result, 0)?)
    }

    /// `gmCWInterface::InqCraftSkillLevel`: the raw and the enchanted level of `skill`.
    fn craft_skill_level(&mut self, skill: u32) -> Option<(u32, u32)> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_CRAFT_SKILL_LEVEL,
            &[
                IFunArg::Int(skill),
                IFunArg::Out(OutValue::Word(0)),
                IFunArg::Out(OutValue::Word(0)),
            ],
            IFunReturn::Word,
        )?;
        Some((result.out_word(0), result.out_word(1)))
    }

    /// `gmCWInterface::InqCraftSkillCategoryName`: the category name of `skill`.
    fn craft_skill_category(&mut self, skill: u32) -> Option<StringInfo> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_CRAFT_SKILL_CATEGORY_NAME,
            &[IFunArg::Int(skill), IFunArg::Filled(STRING_INFO)],
            IFunReturn::Word,
        )?;
        match native(&result, 0)? {
            NativeObject::StringInfo(s) => Some(crate::wire::string_info_to_net(s)),
            _ => None,
        }
    }

    /// `gmCWInterface::CanCraftSkillBeRaised` (the skill can be raised now).
    fn can_craft_skill_be_raised(&mut self, skill: u32) -> Option<bool> {
        let result = ui_calls::script_query(
            self,
            IFUN_CAN_CRAFT_SKILL_BE_RAISED,
            &[IFunArg::Int(skill)],
            IFunReturn::Word,
        )?;
        Some(result.retval != 0)
    }

    /// `gmCWInterface::IsCraftSkillRaiseable`.
    fn is_craft_skill_raiseable(&mut self, skill: u32) -> Option<bool> {
        let result = ui_calls::script_query(
            self,
            IFUN_IS_CRAFT_SKILL_RAISEABLE,
            &[IFunArg::Int(skill)],
            IFunReturn::Word,
        )?;
        Some(result.retval != 0)
    }

    /// `gmCWInterface::InqRecipeList`: the recipes of craft skill `skill` the player knows.
    fn recipes(&mut self, skill: u32) -> Option<Vec<Recipe>> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_RECIPE_LIST,
            &[IFunArg::Int(skill), IFunArg::Filled(R_LIST)],
            IFunReturn::Word,
        )?;
        Some(items(&result, 0)?.iter().filter_map(recipe_of).collect())
    }

    /// `gmCWInterface::InqRecipeAdaptor`: recipe `recipe`.
    fn recipe(&mut self, recipe: u32) -> Option<Recipe> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_RECIPE_ADAPTOR,
            &[IFunArg::Int(recipe), IFunArg::Filled(RECIPE_ADAPTOR)],
            IFunReturn::Word,
        )?;
        recipe_of(native(&result, 0)?)
    }

    /// `gmCWInterface::InqRecipeIngredients`: the recipe's ingredient slots and the ordinal mask
    /// the spinner value leaves filled.
    fn recipe_ingredients(&mut self, recipe: u32, spinner: u32) -> Option<(u32, Vec<Ingredient>)> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_RECIPE_INGREDIENTS,
            &[
                IFunArg::Int(recipe),
                IFunArg::Int(spinner),
                IFunArg::Out(OutValue::Word(0)),
                IFunArg::Filled(R_LIST),
            ],
            IFunReturn::Word,
        )?;
        let ingredients = items(&result, 1)?
            .iter()
            .filter_map(ingredient_of)
            .collect();
        Some((result.out_word(0), ingredients))
    }

    /// `gmCWInterface::InqRecipeDifficulty`: the difficulty and the name colour at `spinner`.
    fn recipe_difficulty(&mut self, recipe: u32, spinner: u32) -> Option<(u32, u32)> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_RECIPE_DIFFICULTY,
            &[
                IFunArg::Int(recipe),
                IFunArg::Int(spinner),
                IFunArg::Out(OutValue::Word(0)),
                IFunArg::Out(OutValue::Word(0)),
            ],
            IFunReturn::Word,
        )?;
        Some((result.out_word(0), result.out_word(1)))
    }

    /// `gmCWInterface::InqRecipeCost`: the cost of `recipe` at `spinner`.
    fn recipe_cost(&mut self, recipe: u32, spinner: u32) -> Option<i32> {
        let result = ui_calls::script_query(
            self,
            IFUN_INQ_RECIPE_COST,
            &[
                IFunArg::Int(recipe),
                IFunArg::Int(spinner),
                IFunArg::Out(OutValue::Word(0)),
            ],
            IFunReturn::Word,
        )?;
        Some(result.out_word(0) as i32)
    }

    /// `gmCWInterface::DoesIngredientAutoPopulate`: whether slot `ordinal` of `recipe` fills
    /// itself.
    fn ingredient_auto_populates(&mut self, recipe: u32, ordinal: u32) -> Option<bool> {
        let result = ui_calls::script_query(
            self,
            IFUN_DOES_INGREDIENT_AUTO_POPULATE,
            &[IFunArg::Int(recipe), IFunArg::Int(ordinal)],
            IFunReturn::Word,
        )?;
        Some(result.retval != 0)
    }

    /// `gmCWInterface::CheckIngredient`: whether `object` may fill slot `ordinal` of `recipe`.
    fn check_ingredient(
        &mut self,
        recipe: u32,
        ordinal: u32,
        object: u64,
        spinner: u32,
    ) -> Option<bool> {
        let result = ui_calls::script_query(
            self,
            IFUN_CHECK_INGREDIENT,
            &[
                IFunArg::Int(recipe),
                IFunArg::Int(ordinal),
                IFunArg::Long(object),
                IFunArg::Int(spinner),
                IFunArg::Out(OutValue::Word(0)),
            ],
            IFunReturn::Word,
        )?;
        Some(result.out_word(0) != 0)
    }

    /// `gmCWInterface::GetAvailableCraftExperience`: the craft experience the player may use.
    fn available_craft_experience(&mut self) -> Option<u64> {
        let result = ui_calls::script_query(
            self,
            IFUN_GET_AVAILABLE_CRAFT_EXPERIENCE,
            &[],
            IFunReturn::Long,
        )?;
        Some(result.retval)
    }
}

impl<P: ScriptPort + ?Sized> CraftQueries for P {}
