//! The crafting calls on the player's script (`ggmCWInterface::CallIFun_*`). The craft panels'
//! queries are the script's: the VM reads the craft skills, recipes and ingredients from the
//! player's qualities and the DAT, and answers with the adaptors it filled (the natives come back
//! as [`CraftSkill`], [`Recipe`] and [`Ingredient`]). The requests are the IFun's stubs: the VM
//! packs them and the session sends them. Without the script VM every call answers `None` (or
//! nothing).

use ac2_net::protocol::StringInfo;
use ac2_script::host::{IFunArg, IFunResult, IFunReturn, OutValue};
use ac2_script::native::NativeObject;
use ac2_weenie::{CraftSkill, Ingredient, Recipe, SessionEvent};

use crate::client::Client;
use crate::gameplay::Command;
use crate::scripts::*;

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

impl Client {
    /// One craft query: its IFun on the player's script, the notices it raised kept for the next
    /// frame. `None` without the VM, or when the call fails.
    pub(crate) fn script_query(
        &mut self,
        function: u32,
        args: &[IFunArg],
        ret: IFunReturn,
    ) -> Option<IFunResult> {
        if !self.scripts_own(function) {
            return None;
        }
        let mut produced = Vec::new();
        let result = self
            .ifun_return_into(function, args, ret, &mut produced)
            .ok()?;
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.produced.extend(produced);
        }
        Some(result)
    }

    /// A request's IFun on the player's script (a craft or skill request): its stub sends the
    /// request; the events it raised come back.
    pub(crate) fn script_request(&mut self, function: u32, args: &[IFunArg]) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        if self.scripts_own(function) {
            if let Err(e) = self.ifun_into(function, args, &mut out) {
                if let Some(scripts) = self.scripts.as_mut() {
                    scripts.problems.push(e);
                }
            }
        }
        out
    }

    /// `gmCWInterface::InqCraftSkillList`: the player's craft skills, from the script.
    pub fn craft_skills(&mut self) -> Option<Vec<CraftSkill>> {
        let result = self.script_query(
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
    pub fn craft_skill(&mut self, skill: u32) -> Option<CraftSkill> {
        let result = self.script_query(
            IFUN_INQ_CRAFT_SKILL_ADAPTOR,
            &[IFunArg::Int(skill), IFunArg::Filled(CRAFT_SKILL_ADAPTOR)],
            IFunReturn::Word,
        )?;
        craft_skill_of(native(&result, 0)?)
    }

    /// `gmCWInterface::InqCraftSkillLevel`: the raw and the enchanted level of `skill`.
    pub fn craft_skill_level(&mut self, skill: u32) -> Option<(u32, u32)> {
        let result = self.script_query(
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
    pub fn craft_skill_category(&mut self, skill: u32) -> Option<StringInfo> {
        let result = self.script_query(
            IFUN_INQ_CRAFT_SKILL_CATEGORY_NAME,
            &[IFunArg::Int(skill), IFunArg::Filled(STRING_INFO)],
            IFunReturn::Word,
        )?;
        match native(&result, 0)? {
            NativeObject::StringInfo(s) => Some(ac2_script::wire::string_info_to_net(s)),
            _ => None,
        }
    }

    /// `gmCWInterface::CanCraftSkillBeRaised` (the skill can be raised now).
    pub fn can_craft_skill_be_raised(&mut self, skill: u32) -> Option<bool> {
        let result = self.script_query(
            IFUN_CAN_CRAFT_SKILL_BE_RAISED,
            &[IFunArg::Int(skill)],
            IFunReturn::Word,
        )?;
        Some(result.retval != 0)
    }

    /// `gmCWInterface::IsCraftSkillRaiseable`.
    pub fn is_craft_skill_raiseable(&mut self, skill: u32) -> Option<bool> {
        let result = self.script_query(
            IFUN_IS_CRAFT_SKILL_RAISEABLE,
            &[IFunArg::Int(skill)],
            IFunReturn::Word,
        )?;
        Some(result.retval != 0)
    }

    /// `gmCWInterface::InqRecipeList`: the recipes of craft skill `skill` the player knows.
    pub fn recipes(&mut self, skill: u32) -> Option<Vec<Recipe>> {
        let result = self.script_query(
            IFUN_INQ_RECIPE_LIST,
            &[IFunArg::Int(skill), IFunArg::Filled(R_LIST)],
            IFunReturn::Word,
        )?;
        Some(items(&result, 0)?.iter().filter_map(recipe_of).collect())
    }

    /// `gmCWInterface::InqRecipeAdaptor`: recipe `recipe`.
    pub fn recipe(&mut self, recipe: u32) -> Option<Recipe> {
        let result = self.script_query(
            IFUN_INQ_RECIPE_ADAPTOR,
            &[IFunArg::Int(recipe), IFunArg::Filled(RECIPE_ADAPTOR)],
            IFunReturn::Word,
        )?;
        recipe_of(native(&result, 0)?)
    }

    /// `gmCWInterface::InqRecipeIngredients`: the recipe's ingredient slots and the ordinal mask
    /// the spinner value leaves filled.
    pub fn recipe_ingredients(
        &mut self,
        recipe: u32,
        spinner: u32,
    ) -> Option<(u32, Vec<Ingredient>)> {
        let result = self.script_query(
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
    pub fn recipe_difficulty(&mut self, recipe: u32, spinner: u32) -> Option<(u32, u32)> {
        let result = self.script_query(
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
    pub fn recipe_cost(&mut self, recipe: u32, spinner: u32) -> Option<i32> {
        let result = self.script_query(
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
    pub fn ingredient_auto_populates(&mut self, recipe: u32, ordinal: u32) -> Option<bool> {
        let result = self.script_query(
            IFUN_DOES_INGREDIENT_AUTO_POPULATE,
            &[IFunArg::Int(recipe), IFunArg::Int(ordinal)],
            IFunReturn::Word,
        )?;
        Some(result.retval != 0)
    }

    /// `gmCWInterface::CheckIngredient`: whether `object` may fill slot `ordinal` of `recipe`.
    pub fn check_ingredient(
        &mut self,
        recipe: u32,
        ordinal: u32,
        object: u64,
        spinner: u32,
    ) -> Option<bool> {
        let result = self.script_query(
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
    pub fn available_craft_experience(&mut self) -> Option<u64> {
        let result =
            self.script_query(IFUN_GET_AVAILABLE_CRAFT_EXPERIENCE, &[], IFunReturn::Long)?;
        Some(result.retval)
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
