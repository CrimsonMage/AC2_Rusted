//! The crafting against the double: the server's answers to the player's craft requests
//! (`WM_Craft`). The client's scripts run the craft calls (the IFuns and their stubs), so the
//! double only answers what the stubs send: an ingredient added is answered at once (`AddIngredient_Done`),
//! and an executed craft takes its ingredients out of the pack and gives the result, then
//! `ExecuteCraft_Done`. The double's rules, not retail's: an ingredient must be in the pack
//! (no check of the recipe's ingredient kinds or the spinner), a craft takes two ingredients, the
//! result is a generic item, and the craft skill's experience is not updated (no
//! `UpdateCraftSkill`).

use ac2_net::protocol::{CraftMessage, ScriptRequest};

use crate::items::{item_object_bytes, ItemSpawn};
use crate::{State, PLAYER};

/// The first id of a crafted item (`answer_execute` counts up from it).
const CRAFTED_BASE: u64 = 0x8000_0000_0000_0400;
/// `CItem`, the package the double's items use.
const ITEM_PACKAGE: u32 = 0x1AA9;
/// The ingredients a craft takes (the double's recipe).
pub const INGREDIENTS: usize = 2;
/// The error a craft or an ingredient refuses with (any non-zero value).
const REFUSED: u32 = 1;

impl State {
    /// The server's answer to a craft request (`WM_Craft` 0x06, 0x12); a skill raise is not
    /// answered (the double keeps the skills as they are).
    pub(crate) fn answer_craft(&mut self, request: &ScriptRequest) {
        match request {
            ScriptRequest::CraftAddIngredient {
                recipe,
                ordinal,
                object,
                spinner,
            } => {
                let error = if self.in_pack(*object) { 0 } else { REFUSED };
                self.send_craft(&CraftMessage::AddIngredientDone {
                    recipe: *recipe,
                    ordinal: *ordinal,
                    ingredient: *object,
                    spinner: *spinner,
                    error,
                });
            }
            ScriptRequest::CraftExecute {
                recipe,
                ingredients,
                ..
            } => {
                let usable = ingredients.len() == INGREDIENTS
                    && ingredients.iter().all(|(_, object)| self.in_pack(*object));
                let error = if usable {
                    self.take_craft(ingredients.iter().map(|(_, object)| *object).collect());
                    0
                } else {
                    REFUSED
                };
                self.send_craft(&CraftMessage::ExecuteCraftDone {
                    notify: 1,
                    recipe: *recipe,
                    error,
                });
            }
            _ => {}
        }
    }

    /// Whether `object` is one of the player's pack's items.
    fn in_pack(&self, object: u64) -> bool {
        self.script_world
            .contents
            .get(&PLAYER)
            .is_some_and(|(_, items)| items.contains(&object))
    }

    /// Takes `ingredients` out of the pack and puts the crafted item in it.
    fn take_craft(&mut self, ingredients: Vec<u64>) {
        self.script_world.crafts += 1;
        let item = ItemSpawn::new(
            CRAFTED_BASE + self.script_world.crafts,
            ITEM_PACKAGE,
            "Crafted item",
            PLAYER,
        );
        self.send_event(&item_object_bytes(&item));
        let items = &mut self.script_world.contents.entry(PLAYER).or_default().1;
        items.retain(|object| !ingredients.contains(object));
        items.push(item.id);
        self.send_contents(PLAYER);
    }

    fn send_craft(&mut self, message: &CraftMessage) {
        self.send_script_call(message.function(), &message.arguments());
    }
}
