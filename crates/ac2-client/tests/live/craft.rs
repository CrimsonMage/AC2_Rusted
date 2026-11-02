//! The craft panel as the player drives it, for the live tests (`gmCraftUI`, `gmIngredientUI`):
//! the skill panel's craft tab, down the skill, category and recipe rows to the ingredient
//! panel, and the craft button.

use ac2_ui::{tokenize, AttributeValue, ToolbarChild};
use ac2_weenie::{GameplayNotice, SessionEvent};

use super::LiveRig;

pub const CRAFT_SKILL: u32 = 0x6F00_28D7;
pub const HEALTH_RECIPE: u32 = 0x6F00_1F17;
pub const VIGOR_RECIPE: u32 = 0x6F00_1F18;
pub const PACK: &str = "PackMainContainer";
/// The second press of a double click as the input layer sends it (`ac2_ui::Controls`: a second
/// press of the same action inside the double-click time is the action plus three).
const DOUBLE_CLICK: u32 = 0x4000_0007;

/// The rows of the list `name` on the HUD.
pub fn rows(rig: &LiveRig, name: &str) -> Vec<usize> {
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
pub fn press(rig: &mut LiveRig, name: &str) {
    let mut center = None;
    for _ in 0..60 * 60 {
        center = rig.view.ui_element_center(name);
        if center.is_some() {
            break;
        }
        rig.frame(&[]);
    }
    rig.click(center.unwrap_or_else(|| panic!("{name} never showed")));
}

/// Waits for the player's craft registry, then opens the craft tab of the skill panel.
pub fn open_craft_tab(rig: &mut LiveRig) {
    for _ in 0..60 * 60 {
        if rig
            .client
            .recipes(CRAFT_SKILL)
            .is_some_and(|x| x.len() == 2)
        {
            break;
        }
        rig.frame(&[]);
    }
    assert!(
        rig.client
            .recipes(CRAFT_SKILL)
            .is_some_and(|x| x.len() == 2),
        "the craft recipes"
    );
    rig.panel(ToolbarChild::Skill, true);
    press(rig, "CraftTab");
    rig.run(0.5);
}

/// The craft page's view (`craftskillview`, `craftskillcategoryview` or `recipeview`).
pub fn craft_view(rig: &LiveRig) -> u32 {
    let tree = rig.view.ui_tree().expect("the HUD");
    let page = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("CraftPage")))
        .expect("the craft page");
    tree.get(page).unwrap().state()
}

/// A double click on the craft page's row `row` (an element index).
pub fn double_click_row(rig: &mut LiveRig, row: usize) {
    let at = rig.view.ui_index_center(row).expect("the HUD");
    rig.point(at);
    rig.button(true);
    rig.button(false);
    rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
    rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
    rig.run(0.3);
}

/// The `did` a craft row carries.
pub fn row_did(rig: &LiveRig, row: usize) -> u32 {
    let tree = rig.view.ui_tree().expect("the HUD");
    match tree.get(row).unwrap().attribute(tokenize("did")) {
        Some(AttributeValue::Id(did)) => *did,
        _ => 0,
    }
}

/// The objects in the ingredient panel's slots, in list order.
pub fn filled_slots(rig: &LiveRig) -> Vec<Option<u64>> {
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

/// Down the craft panel's views as the player goes (the skill, its category, the recipe
/// `recipe`), to the ingredient panel open on the recipe's two slots; then the inventory drawer
/// in place of the craft panel. Returns the pack's items.
pub fn open_recipe(rig: &mut LiveRig, recipe: u32) -> Vec<u64> {
    open_craft_tab(rig);
    // AC2e lists every craft skill of the DAT beside the quest recipes' (28 rows, about 17 of
    // them visible): the skill row is found by its id.
    assert!(
        rig.until(10.0, |r| rows(r, "CraftSkillList")
            .iter()
            .any(|&row| row_did(r, row) == CRAFT_SKILL)),
        "the craft skill list has {} rows",
        rows(rig, "CraftSkillList").len()
    );
    let skill = rows(rig, "CraftSkillList")
        .into_iter()
        .find(|&row| row_did(rig, row) == CRAFT_SKILL)
        .unwrap();
    double_click_row(rig, skill);
    if rig.until(4.0, |r| !rows(r, "CraftSkillCategoryList").is_empty()) {
        assert_eq!(craft_view(rig), tokenize("craftskillcategoryview"));
        // The rows fill once the records they name are read (the panel refreshes then).
        assert!(
            rig.until(10.0, |r| rows(r, "CraftSkillCategoryList").len() == 1),
            "one category of recipes"
        );
        let categories = rows(rig, "CraftSkillCategoryList");
        double_click_row(rig, categories[0]);
        assert_eq!(craft_view(rig), tokenize("recipeview"));
        assert!(
            rig.until(10.0, |r| rows(r, "RecipeList").len() == 2),
            "the health and the vigor recipe"
        );
        let row = rows(rig, "RecipeList")
            .into_iter()
            .find(|&r| row_did(rig, r) == recipe)
            .expect("the recipe's row");
        double_click_row(rig, row);
    } else {
        // A row below the list's visible part cannot be clicked: the double click's own
        // request (`gmCraftUI::ListenToMessage`) is sent instead.
        rig.command(ac2_client::Command::OpenRecipe(recipe));
    }
    assert!(
        rig.until(5.0, |r| r
            .view
            .ui_element_center("RealIngredientField")
            .is_some()
            && rows(r, "IngredientListBox").len() == 2),
        "the ingredient panel opens on the recipe's two ingredients"
    );
    // The craft panel is out of the inventory drawer's way; the ingredient panel stays.
    rig.panel(ToolbarChild::Skill, false);
    rig.panel(ToolbarChild::Inventory, true);
    rig.run(1.0);
    rig.view.item_icons(PACK)
}

/// Drags each pack item the recipe accepts into its slot (`CheckIngredient` names the item for
/// each ingredient), as the player does; returns the items used.
pub fn drag_ingredients(rig: &mut LiveRig, recipe: u32, pack: &[u64]) -> Vec<u64> {
    let (_, ingredients) = rig
        .client
        .recipe_ingredients(recipe, 0)
        .expect("the recipe's ingredients");
    let mut used = Vec::new();
    let slots = rows(rig, "IngredientListBox");
    for (slot, ingredient) in slots.iter().zip(&ingredients) {
        let mut item = None;
        for _ in 0..100 {
            item = pack.iter().copied().find(|&i| {
                !used.contains(&i)
                    && rig.client.check_ingredient(recipe, ingredient.id, i, 0) == Some(true)
            });
            if item.is_some() {
                break;
            }
            rig.run(0.1);
        }
        let item = item.expect("a pack item for the ingredient");
        let from = rig.view.item_icon_center(PACK, item).unwrap();
        let to = rig.view.ui_index_center(*slot).unwrap();
        rig.drag(from, to);
        used.push(item);
        assert!(
            rig.until(5.0, |r| filled_slots(r).contains(&Some(item))),
            "the slot takes the ingredient the server accepted: {item:x} from {from:?} to {to:?}"
        );
        // The dropped item stays selected, and the modern drawer fans a ring of buttons out of
        // its slot over the slots beside it; clearing the selection leaves them free to press.
        rig.command(ac2_client::Command::Select(None));
        rig.run(0.3);
    }
    used
}

/// Whether the server finished a craft of `recipe` without an error since event `mark`.
pub fn crafted(rig: &LiveRig, mark: usize, recipe: u32) -> bool {
    rig.seen[mark..].iter().any(|e| {
        matches!(
            e,
            SessionEvent::Notice(GameplayNotice::ExecuteCraftDone { recipe: r, error: 0 })
                if *r == recipe
        )
    })
}
