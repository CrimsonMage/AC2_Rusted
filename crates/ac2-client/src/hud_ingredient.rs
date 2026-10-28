//! `gmIngredientUI` on the frame: the ingredient panel's notices (`OpenIngredientPanel`, the add
//! and execute answers, the craft skill, mount, inventory and deletions) and its messages run over
//! the craft queries and the player's inventory; its requests go out as gameplay commands.

use std::cell::RefCell;

use ac2_net::protocol::StringInfo;
use ac2_ui::{tokenize, IngredientCommand, IngredientSource};
use ac2_weenie::{CraftSkill, Ingredient, Recipe, SessionEvent};

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

impl IngredientSource for ClientPlayer<'_> {
    fn text(&self, info: &StringInfo) -> String {
        self.render_info(info)
    }

    fn string_with_value(&self, string_id: u32, value: i32) -> String {
        let info = ac2_db::RenderInfo {
            string_id,
            table_id: self.ui_table,
            literal: None,
            variables: vec![(tokenize("value"), ac2_db::RenderVar::Int(value))],
        };
        String::from_utf16_lossy(&ac2_db::get_string(self.strings, &info, false))
    }

    fn recipe(&self, recipe: u32) -> Option<Recipe> {
        self.client.borrow_mut().recipe(recipe)
    }

    fn craft_skill(&self, skill: u32) -> Option<CraftSkill> {
        self.client.borrow_mut().craft_skill(skill)
    }

    fn ingredients(&self, recipe: u32, spinner: u32) -> (u32, Vec<Ingredient>) {
        self.client
            .borrow_mut()
            .recipe_ingredients(recipe, spinner)
            .unwrap_or((0, Vec::new()))
    }

    fn cost(&self, recipe: u32, spinner: u32) -> Option<i32> {
        self.client.borrow_mut().recipe_cost(recipe, spinner)
    }

    fn difficulty(&self, recipe: u32, spinner: u32) -> Option<(u32, u32)> {
        self.client.borrow_mut().recipe_difficulty(recipe, spinner)
    }

    fn auto_populates(&self, recipe: u32, ordinal: u32) -> bool {
        self.client
            .borrow_mut()
            .ingredient_auto_populates(recipe, ordinal)
            .unwrap_or(false)
    }

    fn check_ingredient(&self, recipe: u32, ordinal: u32, object: u64, spinner: u32) -> bool {
        self.client
            .borrow_mut()
            .check_ingredient(recipe, ordinal, object, spinner)
            .unwrap_or(false)
    }

    fn has_item(&self, object: u64) -> bool {
        self.carries(object)
    }

    fn inventory(&self) -> Vec<u64> {
        self.carried_ids()
    }

    fn item_icon(&self, object: u64) -> Option<u32> {
        self.client
            .borrow_mut()
            .item_display(object)
            .ok()
            .map(|display| display.icon)
            .filter(|&icon| icon != 0)
    }
}

/// An ingredient request as the gameplay command it is.
fn command(request: IngredientCommand) -> Command {
    match request {
        IngredientCommand::Add {
            recipe,
            ordinal,
            object,
            spinner,
        } => Command::AddIngredient {
            recipe,
            ordinal,
            object,
            spinner,
        },
        IngredientCommand::Execute {
            recipe,
            spinner,
            ingredients,
        } => Command::ExecuteCraft {
            recipe,
            spinner,
            ingredients,
        },
    }
}

impl HudView {
    /// The ingredient panel's notices and messages of the frame, then its requests as commands.
    pub(crate) fn ingredient_frameworks(
        &mut self,
        client: &mut Client,
        frame: &ClientFrame,
        events: &[SessionEvent],
    ) -> Vec<Command> {
        let Some(strings) = self.panels.strings.clone() else {
            return Vec::new();
        };
        if !frame.in_world {
            return Vec::new();
        }
        let messages = self.panels.player_messages.clone();
        let Some(panel) = self.panels.ingredient.as_mut() else {
            return Vec::new();
        };
        let tree = match self.hud.as_mut() {
            Some((tree, _)) => tree,
            None => return Vec::new(),
        };
        let player = ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings.tables,
            ui_table: strings.ui_table,
            enums: &strings.enums,
        };
        panel
            .frame_pass(tree, &player, events, &messages)
            .into_iter()
            .map(command)
            .collect()
    }
}
