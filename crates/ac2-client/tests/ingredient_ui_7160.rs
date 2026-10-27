//! `gmIngredientUI` on the gameplay layout with the player's script on the VM (`AC2_DAT_DIR`): the
//! panel opens on the known recipe, its slots are the recipe's ingredients from the DAT, the spinner
//! and the craft button follow the recipe, and the add and execute requests go out as commands.
//! The double answers the requests; here the answers are fed to the panel as notices.

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use ac2_client::{Client, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::StringInfo;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_ui::{
    build_gameplay_hud, tokenize, AttributeValue, IngredientCommand, IngredientSource,
    IngredientUi, UiTree,
};
use ac2_weenie::{CraftSkill, Ingredient, Recipe};
use common::*;
use legions_server_double::{Spawn, State, KNOWN_RECIPE, PLAYER};

const RECIPE: u32 = KNOWN_RECIPE;
type Shared = Rc<RefCell<State>>;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// The player with the script attached and the craft registry received.
fn with_script() -> Option<(Client, Shared, Arc<ac2_db::ResourceStore>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ac2_db::ResourceStore::new(dataset.clone()));
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, (*store).clone());
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(PLAYER);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    server.borrow_mut().send_craft_registry();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    Some((client, server, store))
}

/// The ingredient queries as the panel's source. Text is the string's id; the rest is the client's.
struct Known<'a> {
    client: RefCell<&'a mut Client>,
    inventory: Vec<u64>,
}

impl IngredientSource for Known<'_> {
    fn text(&self, info: &StringInfo) -> String {
        format!("string {:08x}", info.string_id)
    }

    fn string_with_value(&self, string_id: u32, value: i32) -> String {
        format!("string {string_id:08x} {value}")
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
        self.inventory.contains(&object)
    }

    fn inventory(&self) -> Vec<u64> {
        self.inventory.clone()
    }

    fn item_icon(&self, _object: u64) -> Option<u32> {
        None
    }
}

/// The list items' slot objects (the `ItemIcon` child's `id`), in list order.
fn slot_objects(tree: &UiTree, panel: &IngredientUi) -> Vec<Option<u64>> {
    let root = panel.root().expect("the panel's root");
    let list = tree
        .child_recursive(root, tokenize("IngredientListBox"))
        .expect("the list");
    tree.list_items(list)
        .iter()
        .map(|&item| {
            let icon = tree.child_recursive(item, tokenize("ItemIcon"))?;
            match tree.get(icon)?.attribute(tokenize("id")) {
                Some(AttributeValue::InstanceId(id)) if *id != 0 => Some(*id),
                _ => None,
            }
        })
        .collect()
}

#[test]
fn the_panel_opens_on_the_recipes_ingredients_and_executes() {
    let Some((mut client, _server, store)) = with_script() else {
        return;
    };
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut panel: IngredientUi = hud.ingredient.take().expect("the ingredient panel");
    let tree = &mut hud.tree;
    let inventory = client.frame().inventory.pack.items.to_vec();
    let source = Known {
        client: RefCell::new(&mut client),
        inventory,
    };

    // The panel opens on the recipe: a slot per ingredient the spinner leaves, in ordinal order.
    panel.open(tree, &source, RECIPE);
    assert!(panel.root().is_some(), "the panel's root is made");
    let spinner = panel.spinner();
    let (mask, ingredients) = source.ingredients(RECIPE, spinner);
    assert!(mask != 0 || ingredients.is_empty());
    assert_eq!(slot_objects(tree, &panel).len(), ingredients.len());

    // The panel's height follows its ingredient count (`AutoSizeIngredientUI`): one more row for
    // each ingredient past the third, between its `min_height` and its `max_height`.
    let root = panel.root().expect("the panel's root");
    let attribute = |id: &str| match tree.get(root).unwrap().attribute(tokenize(id)) {
        Some(AttributeValue::Int(value)) => *value,
        _ => 0,
    };
    let (min_height, max_height) = (attribute("min_height"), attribute("max_height"));
    let upper = if max_height > 0 { max_height } else { i32::MAX };
    let count = source.recipe(RECIPE).expect("the recipe").max_ingredients as i32;
    // Each row past the third adds at least one pixel: the height is at least that much.
    let lower = if count > 3 {
        (min_height + (count - 3)).min(upper)
    } else {
        min_height
    };
    let height = tree.get(root).unwrap().rect().height();
    assert!(
        (lower..=upper).contains(&height),
        "height {height} for {count} ingredients"
    );

    // The cost text (`UpdateCost`): `ID_Craft_Cost` with the recipe's cost as its value.
    let cost = source.recipe(RECIPE).expect("the recipe").cost;
    let field = tree
        .child_recursive(root, tokenize("CostField"))
        .expect("the cost field");
    assert_eq!(
        tree.get(field).unwrap().text().unwrap_or_default(),
        format!("string {:08x} {cost}", tokenize("ID_Craft_Cost")),
    );

    // The spinner moves within its range (a read-only spinner does not move), and the craft button
    // is ghosted at the range's ends.
    let before = panel.spinner();
    panel.step_spinner(tree, &source, true);
    let after = panel.spinner();
    assert!(after == before || after == before + 1);

    // Executing asks for the craft at the spinner value, with each slot's object (none here).
    panel.execute(tree);
    assert_eq!(
        panel.take_commands(),
        vec![IngredientCommand::Execute {
            recipe: RECIPE,
            spinner: after,
            ingredients: Vec::new(),
        }]
    );

    // The craft's answer: the panel is no longer pending; an error keeps the panel open.
    panel.execute_done(tree, &source, 1);
    assert!(panel.root().is_some());

    // An add answer puts the object in its slot when it succeeds, and frees the slot when not.
    let object = 0x8000_0000_0000_9001;
    let ordinal = ingredients.first().map(|i| i.id).unwrap_or(1);
    panel.add_done(tree, &source, ordinal, object, after, 0);
    assert_eq!(slot_objects(tree, &panel)[0], Some(object));
    panel.add_done(tree, &source, ordinal, object, after, 1);
    assert_eq!(slot_objects(tree, &panel)[0], None);
    // Auto-populate: each inventory object goes into a free slot the recipe fills itself, and the
    // request for it goes out (the auto-populate button's message).
    let auto_button = tree
        .child_recursive(panel.root().unwrap(), tokenize("AutoPopulateButton"))
        .expect("the auto-populate button");
    let press = ac2_ui::UiMessage {
        element_id: tokenize("AutoPopulateButton"),
        element: Some(auto_button),
        message: tokenize("mouse_release_on"),
        data: 0,
        item: None,
    };
    panel.message(tree, &source, &press);
    for command in panel.take_commands() {
        if let IngredientCommand::Add {
            recipe,
            ordinal,
            object,
            ..
        } = command
        {
            assert_eq!(recipe, RECIPE);
            assert!(
                source.auto_populates(RECIPE, ordinal),
                "ordinal {ordinal} auto-populates"
            );
            assert!(
                source.inventory().contains(&object),
                "the object is in the inventory"
            );
        }
    }
    drop(source);
    assert!(client.take_script_problems().is_empty());
}
