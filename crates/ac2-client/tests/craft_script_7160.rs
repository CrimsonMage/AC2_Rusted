//! The crafting calls with the player's script on the VM (`AC2_DAT_DIR`). The craft queries
//! answer from the DAT records the VM reads (a craft skill and a recipe WState, through the
//! script's loaders); the requests are the IFuns' stubs, which the VM packs and the session sends;
//! the double answers them (`AddIngredient_Done` per ingredient, `ExecuteCraft_Done` with the
//! ingredients taken and the result given), and each answer reaches the VM's forwarder and raises
//! its notice.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::ScriptRequest;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, PLAYER, POTIONS, SWORD};

/// The craft skill whose record the double's recipe belongs to (`CraftSkill` WState).
const SKILL: u32 = 0x6F00_28D7;
/// A recipe of that skill (`Recipe` WState): its root's craft skill DID is `SKILL`.
const RECIPE: u32 = 0x6F00_1F0C;
/// The spinner value the craft is run at.
const SPINNER: u32 = 9;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// The script with the player attached, after the login's idle frames.
fn with_script() -> Option<(
    ac2_client::Client,
    std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
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
    Some((client, server))
}

/// Runs the commands for four frames; the notices the frames raised, in order.
fn run(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut notices = Vec::new();
    let mut first = true;
    // A craft's answers can wait on a DAT read (the VM stops for data and resumes on a frame).
    for _ in 0..10 {
        let frame = if first { &input } else { &idle };
        first = false;
        for event in step(client, server, frame) {
            if let SessionEvent::Notice(notice) = event {
                notices.push(notice);
            }
        }
    }
    notices
}

#[test]
fn craft_queries_answer_from_the_dat_records() {
    let Some((mut client, _server)) = with_script() else {
        return;
    };
    let skill = client.craft_skill(SKILL).expect("the skill's adaptor");
    assert_eq!(skill.did, SKILL);
    assert_ne!(skill.name, Default::default(), "the skill's name string");
    assert!(client.craft_skill_level(SKILL).is_some());
    assert!(client.craft_skill_category(SKILL).is_some());
    let recipe = client.recipe(RECIPE).expect("the recipe's adaptor");
    assert_eq!(recipe.did, RECIPE);
    assert_eq!(recipe.craft_skill_did, SKILL, "the recipe's craft skill");
    assert_eq!(client.recipe_cost(RECIPE, SPINNER), Some(recipe.cost));
    assert!(client.take_script_problems().is_empty());
    // The double's player has no craft skill and no known recipe yet: the lists are empty (the
    // membership the VM tests is not traced; see the known gaps).
    assert_eq!(client.craft_skills(), Some(Vec::new()));
    assert_eq!(client.recipes(SKILL), Some(Vec::new()));
}

#[test]
fn open_recipe_raises_the_ingredient_panel_without_a_request() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let notices = run(&mut client, &server, vec![Command::OpenRecipe(RECIPE)]);
    assert_eq!(notices, vec![GameplayNotice::OpenIngredientPanel(RECIPE)]);
    assert!(server.borrow().script_requests.is_empty());
}

#[test]
fn add_ingredients_is_answered_per_ingredient() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let notices = run(
        &mut client,
        &server,
        vec![
            Command::AddIngredient {
                recipe: RECIPE,
                ordinal: 0,
                object: POTIONS,
                spinner: SPINNER,
            },
            Command::AddIngredient {
                recipe: RECIPE,
                ordinal: 1,
                object: SWORD,
                spinner: SPINNER,
            },
        ],
    );
    // The request bytes are the stub's: spinner, IID, ordinal, recipe (in pack order).
    assert_eq!(
        server.borrow().script_requests,
        vec![
            ScriptRequest::CraftAddIngredient {
                recipe: RECIPE,
                ordinal: 0,
                object: POTIONS,
                spinner: SPINNER,
            },
            ScriptRequest::CraftAddIngredient {
                recipe: RECIPE,
                ordinal: 1,
                object: SWORD,
                spinner: SPINNER,
            },
        ]
    );
    assert_eq!(
        notices,
        vec![
            GameplayNotice::AddIngredientDone {
                recipe: RECIPE,
                ordinal: 0,
                ingredient: POTIONS,
                spinner: SPINNER,
                error: 0,
            },
            GameplayNotice::AddIngredientDone {
                recipe: RECIPE,
                ordinal: 1,
                ingredient: SWORD,
                spinner: SPINNER,
                error: 0,
            },
        ]
    );
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn execute_craft_takes_the_ingredients_and_gives_the_result() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let before = client
        .session()
        .inventory()
        .contents(PLAYER)
        .map(|c| c.items.clone())
        .unwrap_or_default();
    assert!(before.contains(&POTIONS) && before.contains(&SWORD));
    let notices = run(
        &mut client,
        &server,
        vec![Command::ExecuteCraft {
            recipe: RECIPE,
            spinner: SPINNER,
            ingredients: vec![(0, POTIONS), (1, SWORD)],
        }],
    );
    // The double sends the pack's update (the result given, the ingredients gone) before the answer.
    assert_eq!(
        notices,
        vec![
            GameplayNotice::UpdateContainer {
                container: PLAYER,
                code: 0x4000_0001,
                item: 0,
                slot: 0,
            },
            GameplayNotice::ExecuteCraftDone {
                recipe: RECIPE,
                error: 0,
            },
        ]
    );
    let after = client
        .session()
        .inventory()
        .contents(PLAYER)
        .map(|c| c.items.clone())
        .unwrap_or_default();
    assert!(
        !after.contains(&POTIONS) && !after.contains(&SWORD),
        "consumed"
    );
    assert_eq!(after.len(), before.len() - 1, "one result for two");
    assert!(
        after.iter().any(|item| !before.contains(item)),
        "the crafted item is in the pack"
    );
    assert!(server.borrow().script_requests.iter().any(|r| matches!(
        r,
        ScriptRequest::CraftExecute { recipe, spinner, ingredients, .. }
            if *recipe == RECIPE && *spinner == SPINNER && ingredients == &vec![(0, POTIONS), (1, SWORD)]
    )));
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn craft_calls_do_nothing_without_the_script_vm() {
    let (mut client, server) = entered(spawn());
    let idle = FrameInput::default();
    let input = FrameInput {
        commands: vec![
            Command::OpenRecipe(RECIPE),
            Command::AddIngredient {
                recipe: RECIPE,
                ordinal: 0,
                object: POTIONS,
                spinner: SPINNER,
            },
            Command::RaiseCraftSkill(SKILL),
        ],
        ..FrameInput::default()
    };
    step(&mut client, &server, &input);
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert!(server.borrow().script_requests.is_empty());
    assert_eq!(client.craft_skills(), None);
    assert_eq!(client.recipe(RECIPE), None);
}
