//! The crafting calls with the player's script on the VM (`AC2_DAT_DIR`). The craft queries
//! answer from the DAT records the VM reads (a craft skill and a recipe WState, through the
//! script's loaders); the requests are the IFuns' stubs, which the VM packs and the session sends;
//! the double answers them (`AddIngredient_Done` per ingredient, `ExecuteCraft_Done` with the
//! ingredients taken and the result given, then the panel), and each answer reaches the VM's
//! forwarder and raises its notice. The registry adds (`CraftRegistry` hashes) are encoded and
//! decoded in ac2-net; their VM runs are blocked (see the known gaps in the catalog).

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{pack_values, script_function, CraftMessage, PackValue, ScriptRequest};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent};
use common::*;
use legions_server_double::{Spawn, CRAFT_SKILL, KNOWN_RECIPE, PLAYER, POTIONS, SWORD};

/// The craft skill whose record the double's registry holds (`CraftSkill` WState).
const SKILL: u32 = CRAFT_SKILL;
/// The recipe the double's registry holds (`Recipe` WState, of [`SKILL`]).
const RECIPE: u32 = KNOWN_RECIPE;
/// The spinner value the craft is run at.
const SPINNER: u32 = 9;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// The script with the player attached, after the login's idle frames.
fn with_script() -> Option<(ac2_client::Client, Shared)> {
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

/// Runs the commands for ten frames (a craft's answers can wait on a DAT read); the notices the
/// frames raised, in order.
fn run(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut notices = Vec::new();
    let mut first = true;
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

/// Sends a craft answer from the double, then runs the frames it reaches.
fn send_answer(
    client: &mut ac2_client::Client,
    server: &Shared,
    message: &CraftMessage,
) -> Vec<GameplayNotice> {
    server
        .borrow_mut()
        .send_script_call(message.function(), &message.arguments());
    run(client, server, Vec::new())
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
    // The registry is empty: the double sends no registry adds at login, and the VM does not run
    // the adds for these records yet (the list membership is the lead's open item).
    assert_eq!(client.craft_skills(), Some(Vec::new()));
    assert_eq!(client.recipes(SKILL), Some(Vec::new()));
    assert!(client.take_script_problems().is_empty());
}

/// The queries the registry does not touch, each answered by the script: ingredient checks,
/// difficulty, the raise checks and the craft experience.
#[test]
fn ingredient_and_raise_queries_answer_from_the_script() {
    let Some((mut client, _server)) = with_script() else {
        return;
    };
    assert!(client
        .check_ingredient(RECIPE, 0, POTIONS, SPINNER)
        .is_some());
    assert!(client.ingredient_auto_populates(RECIPE, 0).is_some());
    assert!(client.recipe_ingredients(RECIPE, SPINNER).is_some());
    assert!(client.recipe_difficulty(RECIPE, SPINNER).is_some());
    assert!(client.can_craft_skill_be_raised(SKILL).is_some());
    assert!(client.is_craft_skill_raiseable(SKILL).is_some());
    assert!(client.available_craft_experience().is_some());
    assert!(client.take_script_problems().is_empty());
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
        server
            .borrow()
            .script_requests
            .iter()
            .filter(|r| matches!(r, ScriptRequest::CraftAddIngredient { .. }))
            .cloned()
            .collect::<Vec<_>>(),
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
    // The double sends the pack's update (the result given, the ingredients gone) before the
    // answer, then the panel refresh.
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
            GameplayNotice::UpdateCraftPanel,
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

/// The panel forwarders that run in the VM (`WM_Craft` 0x19 and 0x1B) each raise
/// `UpdateCraftPanel` from their one reference. 0x18 is not driven: `CPlayer` 0xF9 calls an efun
/// the glue does not bind.
#[test]
fn update_panel_forwarders_raise_the_panel_notice() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    for forwarder in [
        script_function::CRAFT_UPDATE_PANEL_19,
        script_function::CRAFT_UPDATE_PANEL_1B,
    ] {
        let notices = send_answer(
            &mut client,
            &server,
            &CraftMessage::UpdateCraftPanel {
                forwarder,
                value: RECIPE,
            },
        );
        assert_eq!(
            notices,
            vec![GameplayNotice::UpdateCraftPanel],
            "forwarder {forwarder:#x}"
        );
    }
    assert!(client.take_script_problems().is_empty());
}

/// `WM_Craft` 0x37 (`CPlayer` 0x1C5) adds an `RList` of recipe records. An empty list changes
/// nothing and raises only the panel refresh that ends the forwarder.
#[test]
fn add_recipes_of_an_empty_list_changes_nothing() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let before = client.recipes(SKILL).expect("the recipe list");
    let notices = send_answer(
        &mut client,
        &server,
        &CraftMessage::AddRecipes {
            recipes: Vec::new(),
        },
    );
    assert_eq!(
        notices,
        vec![GameplayNotice::UpdateCraftPanel],
        "{notices:?}"
    );
    assert_eq!(client.recipes(SKILL), Some(before));
    assert!(client.take_script_problems().is_empty());
}

/// `RequestRaiseCraftSkill`'s stub packs the one int (the craft skill) as `WM_Craft` 0x2E; the
/// double gives no answer.
#[test]
fn raise_craft_skill_sends_the_stub_bytes_and_gets_no_answer() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let notices = run(&mut client, &server, vec![Command::RaiseCraftSkill(SKILL)]);
    {
        let state = server.borrow();
        let raises: Vec<_> = state
            .script_bytes
            .iter()
            .filter(|(function, _)| *function == script_function::CRAFT_RAISE_SKILL)
            .cloned()
            .collect();
        assert_eq!(
            raises,
            vec![(
                script_function::CRAFT_RAISE_SKILL,
                pack_values(&[PackValue::Int(SKILL)]),
            )],
            "the stub's PackBuffer bytes"
        );
        assert_eq!(
            state.script_requests.last(),
            Some(&ScriptRequest::CraftRaiseSkill(SKILL))
        );
    }
    assert!(
        notices.is_empty(),
        "the double answers no raise: {notices:?}"
    );
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
