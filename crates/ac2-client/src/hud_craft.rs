//! `gmCraftUI`'s client side: the player's craft queries (`ac2_ui::CraftSource`) over the
//! client's script and session, which the frame's craft pass (`Hud::frame`) reads. A query that
//! fails reads as an empty answer. The craft requests map to gameplay commands in
//! [`craft_command`].

use ac2_net::protocol::StringInfo;
use ac2_ui::{CraftCommand, CraftSource};
use ac2_weenie::{CraftSkill, Recipe};

use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

impl CraftSource for ClientPlayer<'_> {
    fn text(&self, info: &StringInfo) -> String {
        self.render_info(info)
    }

    fn craft_skills(&self) -> Vec<CraftSkill> {
        self.client.borrow_mut().craft_skills().unwrap_or_default()
    }

    fn craft_skill(&self, skill: u32) -> Option<CraftSkill> {
        self.client.borrow_mut().craft_skill(skill)
    }

    fn category_name(&self, category: u32) -> Option<StringInfo> {
        self.client.borrow_mut().craft_skill_category(category)
    }

    fn recipes(&self, skill: u32) -> Vec<Recipe> {
        self.client.borrow_mut().recipes(skill).unwrap_or_default()
    }

    fn is_raiseable(&self, skill: u32) -> bool {
        self.client
            .borrow_mut()
            .is_craft_skill_raiseable(skill)
            .unwrap_or(false)
    }

    fn can_raise(&self, skill: u32) -> bool {
        self.client
            .borrow_mut()
            .can_craft_skill_be_raised(skill)
            .unwrap_or(false)
    }

    fn skill_level(&self, skill: u32) -> Option<(u32, u32)> {
        self.client.borrow_mut().craft_skill_level(skill)
    }

    fn available_experience(&self) -> u64 {
        self.client
            .borrow_mut()
            .available_craft_experience()
            .unwrap_or(0)
    }

    fn ui_text(&self, string_id: u32, vars: &[(u32, i64)]) -> String {
        let info = ac2_db::RenderInfo {
            string_id,
            table_id: self.ui_table,
            literal: None,
            variables: vars
                .iter()
                .map(|&(name, value)| {
                    let value = i32::try_from(value).unwrap_or(i32::MAX);
                    (name, ac2_db::RenderVar::Int(value))
                })
                .collect(),
        };
        String::from_utf16_lossy(&ac2_db::get_string(self.strings, &info, false))
    }
}

/// A craft request as the gameplay command it is.
pub(crate) fn craft_command(request: CraftCommand) -> Command {
    match request {
        CraftCommand::ClearSelection => Command::Select(None),
        CraftCommand::Examine(request) => Command::Examine(request),
        CraftCommand::OpenRecipe(recipe) => Command::OpenRecipe(recipe),
        CraftCommand::RaiseSkill(skill) => Command::RaiseCraftSkill(skill),
    }
}
