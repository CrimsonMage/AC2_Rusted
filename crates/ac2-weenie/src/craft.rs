//! The crafting data the scripts fill for the craft panels: a recipe (`RecipeAdaptor`) and an
//! ingredient slot (`IngredientAdaptor`), over plain types. The members are those the shared
//! craft sources copy (`ac2/shared/src/craft/recipeadaptor.cpp.c`,
//! `ingredientadaptor.cpp.c`); the defaults are the constructors'.

use ac2_net::protocol::StringInfo;

/// `RecipeAdaptor`: one recipe as the craft panel shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    /// `m_didRecipe`: the recipe's DID.
    pub did: u32,
    /// `m_name`.
    pub name: StringInfo,
    /// `m_description`.
    pub description: StringInfo,
    /// `m_charges` (`-1`: no charges).
    pub charges: i32,
    /// `m_minSpinnerVal`, `m_maxSpinnerVal`, `m_lastSpinnerVal`: the spinner's range and last value.
    pub min_spinner: u32,
    pub max_spinner: u32,
    pub last_spinner: u32,
    /// `m_autoPopulateIngredients`.
    pub auto_populate: bool,
    /// `m_maxNumIngredients`.
    pub max_ingredients: u32,
    /// `m_bHasNameColoringTable` and `m_uiNameColor`.
    pub has_name_coloring: bool,
    pub name_color: u32,
    /// `m_difficulty`, `m_dynamicDifficulty`.
    pub difficulty: u32,
    pub dynamic_difficulty: bool,
    /// `m_difficultyMinusSkillCutoff` and `m_hasDifficultyMinusSkillCutoff`.
    pub difficulty_minus_skill_cutoff: i32,
    pub has_difficulty_minus_skill_cutoff: bool,
    /// `m_timeUseAgain` (a 64-bit time).
    pub time_use_again: u64,
    /// `m_cost`.
    pub cost: i32,
    /// `m_shown`, `m_dynamicCost`.
    pub shown: bool,
    pub dynamic_cost: bool,
    /// `m_iconDID`.
    pub icon_did: u32,
    /// `m_craftSkillDID` and `m_craftSkillCategory`.
    pub craft_skill_did: u32,
    pub craft_skill_category: u32,
    /// `m_bReadOnlySpinner`.
    pub read_only_spinner: bool,
}

impl Default for Recipe {
    /// `RecipeAdaptor::RecipeAdaptor`.
    fn default() -> Self {
        Self {
            did: 0,
            name: StringInfo::default(),
            description: StringInfo::default(),
            charges: -1,
            min_spinner: 0,
            max_spinner: 0,
            last_spinner: 0,
            auto_populate: false,
            max_ingredients: 0,
            has_name_coloring: false,
            name_color: 0,
            difficulty: 0,
            dynamic_difficulty: false,
            difficulty_minus_skill_cutoff: 0,
            has_difficulty_minus_skill_cutoff: false,
            time_use_again: 0,
            cost: 0,
            shown: false,
            dynamic_cost: false,
            icon_did: 0,
            craft_skill_did: 0,
            craft_skill_category: 0,
            read_only_spinner: false,
        }
    }
}

/// `IngredientAdaptor`: one ingredient slot of a recipe.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ingredient {
    /// `m_id`: the slot's ordinal.
    pub id: u32,
    /// `m_didRecipe`: the recipe the slot belongs to.
    pub did_recipe: u32,
    /// `m_description`.
    pub description: StringInfo,
    /// `m_autoPopulate`.
    pub auto_populate: bool,
}
