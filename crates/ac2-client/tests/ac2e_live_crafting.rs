//! Crafting against a running AC2e server (see `tests/live/mod.rs`): the craft panel lists the
//! player's craft skills and recipes, and a recipe is crafted from the panel the way the player
//! does it: down the skill, category and recipe rows, the ingredients dragged from the pack into
//! the recipe's slots, the craft button, the result in the pack.
//! Needs the development kit (`--training-kit true`: AC2e sends the craft registry only with it);
//! `scripts/ac2e-live-test.sh` with `AC2E_LIVE_TEST=ac2e_live_crafting` sets that up.

#![cfg(feature = "window")]

mod live;

use ac2_ui::{tokenize, AttributeValue, ToolbarChild};
use ac2_weenie::{GameplayNotice, SessionEvent};
use live::LiveRig;

const CRAFT_SKILL: u32 = 0x6F00_28D7;
const VIGOR_RECIPE: u32 = 0x6F00_1F18;
const PACK: &str = "PackMainContainer";
/// The second press of a double click as the input layer sends it (`ac2_ui::Controls`: a second
/// press of the same action inside the double-click time is the action plus three).
const DOUBLE_CLICK: u32 = 0x4000_0007;

fn rig_for(name: &str) -> Option<LiveRig> {
    if std::env::var_os("AC2E_LIVE_TRAINING_KIT").is_none() {
        eprintln!("the server has no craft recipes (--training-kit true); skipped");
        return None;
    }
    LiveRig::new(name)
}

fn wait(rig: &mut LiveRig, what: &str, mut ready: impl FnMut(&mut LiveRig) -> bool) {
    for _ in 0..60 * 60 {
        if ready(rig) {
            return;
        }
        rig.frame(&[]);
    }
    panic!("{what}");
}

/// The rows of the list `name` on the HUD.
fn rows(rig: &LiveRig, name: &str) -> Vec<usize> {
    let Some(tree) = rig.view.ui_tree() else {
        return Vec::new();
    };
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize(name)))
        .map(|list| tree.list_items(list).to_vec())
        .unwrap_or_default()
}

/// A click on the centre of the shown HUD element `name`, once it shows.
fn press(rig: &mut LiveRig, name: &str) {
    let mut center = None;
    wait(rig, name, |r| {
        center = r.view.ui_element_center(name);
        center.is_some()
    });
    rig.click(center.unwrap());
}

/// Waits for the player's craft registry, then opens the craft tab of the skill panel.
fn open_craft_tab(rig: &mut LiveRig) {
    wait(rig, "the craft recipes", |r| {
        r.client.recipes(CRAFT_SKILL).is_some_and(|x| x.len() == 2)
    });
    rig.panel(ToolbarChild::Skill, true);
    press(rig, "CraftTab");
    rig.run(0.5);
}

/// The craft page's view (`craftskillview`, `craftskillcategoryview` or `recipeview`).
fn craft_view(rig: &LiveRig) -> u32 {
    let tree = rig.view.ui_tree().expect("the HUD");
    let page = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("CraftPage")))
        .expect("the craft page");
    tree.get(page).unwrap().state()
}

/// A double click on the craft page's row `row` (an element index).
fn double_click_row(rig: &mut LiveRig, row: usize) {
    let at = rig.view.ui_index_center(row).expect("the HUD");
    rig.point(at);
    rig.button(true);
    rig.button(false);
    rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
    rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
    rig.run(0.3);
}

/// The `did` a craft row carries.
fn row_did(rig: &LiveRig, row: usize) -> u32 {
    let tree = rig.view.ui_tree().expect("the HUD");
    match tree.get(row).unwrap().attribute(tokenize("did")) {
        Some(AttributeValue::Id(did)) => *did,
        _ => 0,
    }
}

/// The objects in the ingredient panel's slots, in list order.
fn filled_slots(rig: &LiveRig) -> Vec<Option<u64>> {
    let tree = rig.view.ui_tree().expect("the HUD");
    rows(rig, "IngredientListBox")
        .into_iter()
        .map(|row| {
            let icon = tree.child_recursive(row, tokenize("ItemIcon"))?;
            match tree.get(icon)?.attribute(tokenize("id")) {
                Some(AttributeValue::InstanceId(id)) if *id != 0 => Some(*id),
                _ => None,
            }
        })
        .collect()
}

#[test]
fn the_craft_panel_lists_the_players_craft_skills() {
    let Some(mut rig) = rig_for("Craftlist") else {
        return;
    };
    open_craft_tab(&mut rig);
    assert!(
        rig.until(10.0, |r| rows(r, "CraftSkillList").len() == 1),
        "the craft skill list: {} rows",
        rows(&rig, "CraftSkillList").len()
    );
}

/// The recipes of the craft skill, reached down the panel's views as the player does.
#[test]
fn a_recipe_is_crafted_from_the_panel() {
    let Some(mut rig) = rig_for("Craftdo") else {
        return;
    };
    open_craft_tab(&mut rig);
    assert!(
        rig.until(10.0, |r| rows(r, "CraftSkillList").len() == 1),
        "the craft skill list"
    );
    let skill = rows(&rig, "CraftSkillList")[0];
    double_click_row(&mut rig, skill);
    assert_eq!(craft_view(&rig), tokenize("craftskillcategoryview"));
    let categories = rows(&rig, "CraftSkillCategoryList");
    assert_eq!(categories.len(), 1, "one category of recipes");
    double_click_row(&mut rig, categories[0]);
    assert_eq!(craft_view(&rig), tokenize("recipeview"));
    let recipes = rows(&rig, "RecipeList");
    assert_eq!(recipes.len(), 2, "the health and the vigor recipe");
    let vigor = recipes
        .iter()
        .copied()
        .find(|&r| row_did(&rig, r) == VIGOR_RECIPE)
        .expect("the vigor recipe's row");
    double_click_row(&mut rig, vigor);
    assert!(
        rig.until(5.0, |r| r.view.ui_element_center("RealIngredientField").is_some()
            && rows(r, "IngredientListBox").len() == 2),
        "the ingredient panel opens on the recipe's two ingredients"
    );

    // The craft panel is out of the inventory drawer's way; the ingredient panel stays.
    rig.panel(ToolbarChild::Skill, false);
    rig.panel(ToolbarChild::Inventory, true);
    rig.run(1.0);
    let pack_before = rig.view.item_icons(PACK);
    assert!(pack_before.len() >= 2, "the development kit's items");

    // Each ingredient is dragged from the pack into its slot: the pack item the recipe accepts
    // for the slot (`CheckIngredient`, which the panel asks to take a drop), the slot's row.
    let (_, ingredients) = rig
        .client
        .recipe_ingredients(VIGOR_RECIPE, 0)
        .expect("the recipe's ingredients");
    let mut used = Vec::new();
    for (slot, ingredient) in rows(&rig, "IngredientListBox").iter().zip(&ingredients) {
        let item = pack_before
            .iter()
            .copied()
            .find(|&i| rig.client.check_ingredient(VIGOR_RECIPE, ingredient.id, i, 0) == Some(true))
            .expect("a pack item for the ingredient");
        let from = rig.view.item_icon_center(PACK, item).unwrap();
        let to = rig.view.ui_index_center(*slot).unwrap();
        rig.drag(from, to);
        used.push(item);
        assert!(
            rig.until(5.0, |r| filled_slots(r).contains(&Some(item))),
            "the slot takes the ingredient the server accepted"
        );
    }
    assert_eq!(filled_slots(&rig), used.iter().map(|&i| Some(i)).collect::<Vec<_>>());

    // The craft button: the result comes back and the ingredients are gone.
    let mark = rig.seen.len();
    press(&mut rig, "CraftButton");
    assert!(
        rig.until(10.0, |r| r.seen[mark..].iter().any(|e| matches!(
            e,
            SessionEvent::Notice(GameplayNotice::ExecuteCraftDone { recipe, error: 0 })
                if *recipe == VIGOR_RECIPE
        ))),
        "no ExecuteCraftDone: {:?}",
        rig.client.take_script_problems()
    );
    rig.run(1.0);
    let pack_after = rig.view.item_icons(PACK);
    assert!(
        used.iter().all(|i| !pack_after.contains(i)),
        "the ingredients left the pack: {pack_after:x?}"
    );
    let made: Vec<u64> = pack_after
        .iter()
        .copied()
        .filter(|i| !pack_before.contains(i))
        .collect();
    assert_eq!(made.len(), 1, "one crafted item in the pack");
    assert_eq!(pack_after.len(), pack_before.len() - 2 + 1);
    // The ingredient panel follows: the consumed objects left their slots
    // (`gmIngredientUI::RecvNotice_BeingDeleted`).
    assert!(
        rig.until(5.0, |r| filled_slots(r).iter().all(Option::is_none)),
        "the slots still hold {:x?}",
        filled_slots(&rig)
    );
}
