//! `gmCraftUI` on the HUD's craft panel with the player's script on the VM (`AC2_DAT_DIR`). The
//! craft queries answer from the DAT records (`craft_script_7160`). The panel's lists are the
//! registry's, which answer empty for now, so the source here supplies the known skill and recipe
//! as the lists' items; the pages, rows, selections and requests are the panel's own logic.

mod common;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use ac2_client::{Client, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{ExaminationRequest, StringInfo};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_ui::{
    build_gameplay_hud, tokenize, CraftCommand, CraftSource, CraftUi, UiMessage, UiTree,
    STATE_CATEGORY, STATE_RECIPE, STATE_SKILL,
};
use ac2_weenie::{CraftSkill, Recipe};
use common::*;
use legions_server_double::{Spawn, State, CRAFT_SKILL, KNOWN_RECIPE, PLAYER};

const SKILL: u32 = CRAFT_SKILL;
const RECIPE: u32 = KNOWN_RECIPE;
type Shared = Rc<RefCell<State>>;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// The player with the script attached, after the login's idle frames.
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

/// The craft queries as the panel's source. Text is the string's id here (the rendering is the
/// client's); the lists answer the known skill and recipe.
struct Known<'a> {
    client: RefCell<&'a mut Client>,
    /// The recipe's `shown` flag as the panel sees it. The DAT record of the known recipe is not
    /// shown, so the page drops it; the rows are exercised with it shown.
    shown: Cell<bool>,
}

impl Known<'_> {
    fn recipe_record(&self) -> Recipe {
        self.client.borrow_mut().recipe(RECIPE).expect("the recipe")
    }
}

impl CraftSource for Known<'_> {
    fn text(&self, info: &StringInfo) -> String {
        format!("string {:08x}", info.string_id)
    }

    fn craft_skills(&self) -> Vec<CraftSkill> {
        self.craft_skill(SKILL).into_iter().collect()
    }

    fn craft_skill(&self, skill: u32) -> Option<CraftSkill> {
        self.client.borrow_mut().craft_skill(skill)
    }

    fn category_name(&self, category: u32) -> Option<StringInfo> {
        self.client.borrow_mut().craft_skill_category(category)
    }

    fn recipes(&self, skill: u32) -> Vec<Recipe> {
        if skill == SKILL {
            let mut recipe = self.recipe_record();
            recipe.shown = self.shown.get();
            vec![recipe]
        } else {
            Vec::new()
        }
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
        format!("ui {string_id:08x} {vars:?}")
    }
}

/// A message from the element at `element` (its token `element_id`), a mouse action or a button.
fn message(element: usize, element_id: &str, kind: &str, data: i32) -> UiMessage {
    UiMessage {
        element_id: tokenize(element_id),
        element: Some(element),
        message: tokenize(kind),
        data,
        item: None,
    }
}

/// The items of a list under the craft page, by the list's name.
fn rows(tree: &UiTree, page: usize, list: &str) -> Vec<usize> {
    let list = tree
        .child_recursive(page, tokenize(list))
        .expect("the list");
    tree.list_items(list).to_vec()
}

/// A row's text: its `Name` child's.
fn name_text(tree: &UiTree, row: usize) -> String {
    let name = tree.child_recursive(row, tokenize("Name")).expect("a name");
    tree.get(name).unwrap().text().unwrap_or_default()
}

#[test]
fn the_craft_pages_show_the_known_skill_and_recipe() {
    let Some((mut client, _server, store)) = with_script() else {
        return;
    };
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut craft: CraftUi = hud.craft.take().expect("the craft panel");
    let page = craft.page();
    let source = Known {
        client: RefCell::new(&mut client),
        shown: Cell::new(true),
    };
    let tree = &mut hud.tree;

    // The skills view: the known skill is a row, named from its DAT record.
    craft.set_state(tree, &source, STATE_SKILL);
    assert_eq!(craft.state(), STATE_SKILL);
    let skill_rows = rows(tree, page, "CraftSkillList");
    assert_eq!(skill_rows.len(), 1);
    let skill_name = source.text(&source.craft_skill(SKILL).expect("the skill").name);
    assert_eq!(name_text(tree, skill_rows[0]), skill_name);
    assert!(craft.take_commands().is_empty(), "no request on open");

    // A double click on the skill's row selects it, examines it and opens its categories.
    let entry = message(
        skill_rows[0],
        "EntryTemplate",
        "mouse_press_top",
        0x4000_0007,
    );
    craft.message(tree, &source, &entry);
    assert_eq!(craft.state(), STATE_CATEGORY);
    assert_eq!(
        craft.take_commands(),
        vec![
            CraftCommand::ClearSelection,
            CraftCommand::Examine(ExaminationRequest {
                kind: 0x4000_000B,
                data_id: SKILL,
                ..ExaminationRequest::default()
            }),
        ]
    );
    // The category page's header is the skill; its rows are the recipes' categories.
    let header = tree
        .child_recursive(page, tokenize("CraftSkillCategoryHeader"))
        .expect("the header");
    assert_eq!(name_text(tree, header), skill_name);
    let categories = rows(tree, page, "CraftSkillCategoryList");
    assert_eq!(categories.len(), 1);
    let recipe = source.recipe_record();
    let category_name = source.text(
        &source
            .category_name(recipe.craft_skill_category)
            .expect("the category's name"),
    );
    assert_eq!(name_text(tree, categories[0]), category_name);
    // The examination of that category selects its row; another kind of request leaves none
    // selected (`RecvNotice_ExaminationRequestChanged`).
    let examined = |request: ExaminationRequest| {
        [ac2_weenie::SessionEvent::Notice(
            ac2_weenie::GameplayNotice::ExaminationRequestChanged(request),
        )]
    };
    let category_request = ExaminationRequest {
        kind: 0x4000_000C,
        data_enum: recipe.craft_skill_category,
        ..ExaminationRequest::default()
    };
    let events = examined(category_request);
    craft.frame_pass(tree, &source, &events, &[]);
    assert_eq!(
        tree.get(categories[0]).unwrap().state(),
        tokenize("selected")
    );
    let events = examined(ExaminationRequest {
        kind: 0x4000_0001,
        ..ExaminationRequest::default()
    });
    craft.frame_pass(tree, &source, &events, &[]);
    assert_ne!(
        tree.get(categories[0]).unwrap().state(),
        tokenize("selected")
    );

    // A double click on the category opens its recipes.
    let category = message(
        categories[0],
        "CategoryTemplate",
        "mouse_press_top",
        0x4000_0007,
    );
    craft.message(tree, &source, &category);
    assert_eq!(craft.state(), STATE_RECIPE);
    assert_eq!(
        craft.take_commands(),
        vec![
            CraftCommand::ClearSelection,
            CraftCommand::Examine(ExaminationRequest {
                kind: 0x4000_000C,
                data_enum: recipe.craft_skill_category,
                ..ExaminationRequest::default()
            }),
        ]
    );
    let recipes = rows(tree, page, "RecipeList");
    assert_eq!(recipes.len(), 1);
    assert_eq!(name_text(tree, recipes[0]), source.text(&recipe.name));

    // The recipe's button opens its ingredient panel, after the examination.
    let button = tree
        .child_recursive(recipes[0], tokenize("RecipeButton"))
        .expect("the recipe's button");
    let open = message(button, "RecipeButton", "mouse_release_on", 0);
    craft.message(tree, &source, &open);
    assert_eq!(
        craft.take_commands(),
        vec![
            CraftCommand::ClearSelection,
            CraftCommand::Examine(ExaminationRequest {
                kind: 0x4000_0007,
                data_id: RECIPE,
                ..ExaminationRequest::default()
            }),
            CraftCommand::OpenRecipe(RECIPE),
        ]
    );

    // The selected skill's level text (`RefreshCraftSkillDisplay` through `MakeLevelText`): the
    // enchanted level, with its buffed or debuffed difference when it differs from the raw one.
    // The raise button is not under this layout's craft page (the test above), so the raise text
    // does not replace it here.
    let (raw, enchanted) = source.skill_level(SKILL).expect("the skill's level");
    let mut vars = vec![(tokenize("level"), i64::from(enchanted))];
    let id = if enchanted == raw {
        "ID_Craft_SkillLevelNormal"
    } else {
        vars.push((
            tokenize("value"),
            (i64::from(enchanted) - i64::from(raw)).abs(),
        ));
        if enchanted > raw {
            "ID_Craft_SkillLevelBuffed"
        } else {
            "ID_Craft_SkillLevelDebuffed"
        }
    };
    let level_text = tree
        .child_recursive(page, tokenize("SelectedLevel"))
        .and_then(|level| tree.get(level)?.text())
        .expect("the level element's text");
    assert_eq!(
        level_text,
        format!("ui {:08x} {vars:?}", tokenize(id)),
        "the level text"
    );

    // The raise button's request: the selected skill is raised. The button itself is a page
    // element the layout links (`SelectedField` / `ButtonField`), not under the craft page here,
    // so the message is sent to the page.
    let raise_click = message(page, "RaiseCraftButton", "mouse_release_on", 0);
    craft.message(tree, &source, &raise_click);
    assert_eq!(craft.take_commands(), vec![CraftCommand::RaiseSkill(SKILL)]);

    // A recipe the DAT does not show gets no row (`RefreshRecipePage` checks `m_shown`).
    source.shown.set(false);
    craft.set_state(tree, &source, STATE_RECIPE);
    assert!(rows(tree, page, "RecipeList").is_empty());
}
