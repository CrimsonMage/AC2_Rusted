//! Crafting against a running AC2e server (see `tests/live/mod.rs`): the craft panel lists the
//! player's craft skills and recipes, and a recipe is crafted from the panel the way the player
//! does it: down the skill, category and recipe rows, the ingredients dragged from the pack into
//! the recipe's slots, the craft button, the result in the pack.
//! Needs the development kit (`--training-kit true`: AC2e sends the craft registry only with it);
//! `scripts/ac2e-live-test.sh` with `AC2E_LIVE_TEST=ac2e_live_crafting` sets that up.
//! The panel's driving is shared with the other live files (`live/craft.rs`).

#![cfg(feature = "window")]

mod live;

use ac2_weenie::{GameplayNotice, SessionEvent};
use live::craft::*;
use live::LiveRig;

fn rig_for(name: &str) -> Option<LiveRig> {
    if std::env::var_os("AC2E_LIVE_TRAINING_KIT").is_none() {
        eprintln!("the server has no craft recipes (--training-kit true); skipped");
        return None;
    }
    LiveRig::new(name)
}

/// The craft skills AC2e lists: every discipline of its profession catalog
/// (`crafting::initialization_with_quests` sends `professions::catalog().skills` whole once the
/// development kit authorizes crafting; the server prints the count at start, "Professions: 26
/// disciplines").
const AC2E_CRAFT_SKILLS: usize = 26;

#[test]
fn the_craft_panel_lists_the_players_craft_skills() {
    let Some(mut rig) = rig_for("Craftlist") else {
        return;
    };
    open_craft_tab(&mut rig);
    assert!(
        rig.until(10.0, |r| rows(r, "CraftSkillList").len()
            == AC2E_CRAFT_SKILLS),
        "the craft skill list: {} rows",
        rows(&rig, "CraftSkillList").len()
    );
}

/// The crafted result: the craft is done, the ingredients left the pack, one new item came,
/// and the slots emptied (`gmIngredientUI::RecvNotice_BeingDeleted`).
fn assert_crafted(rig: &mut LiveRig, mark: usize, pack_before: &[u64]) {
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
    let made: Vec<u64> = pack_after
        .iter()
        .copied()
        .filter(|i| !pack_before.contains(i))
        .collect();
    assert_eq!(
        made.len(),
        1,
        "one crafted item in the pack: {pack_after:x?}"
    );
    assert_eq!(pack_after.len(), pack_before.len() - 2 + 1);
    assert!(
        rig.until(5.0, |r| filled_slots(r).iter().all(Option::is_none)),
        "the slots still hold {:x?}",
        filled_slots(rig)
    );
}

/// The recipes of the craft skill, reached down the panel's views as the player does.
#[test]
fn a_recipe_is_crafted_from_the_panel() {
    let Some(mut rig) = rig_for("Craftdo") else {
        return;
    };
    let pack_before = open_recipe(&mut rig, VIGOR_RECIPE);

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
            .find(|&i| {
                rig.client
                    .check_ingredient(VIGOR_RECIPE, ingredient.id, i, 0)
                    == Some(true)
            })
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
    assert_eq!(
        filled_slots(&rig),
        used.iter().map(|&i| Some(i)).collect::<Vec<_>>()
    );

    // The craft button: the result comes back and the ingredients are gone.
    let mark = rig.seen.len();
    press(&mut rig, "CraftButton");
    assert_crafted(&mut rig, mark, &pack_before);
    let pack_after = rig.view.item_icons(PACK);
    assert!(
        used.iter().all(|i| !pack_after.contains(i)),
        "the ingredients left the pack: {pack_after:x?}"
    );
}

/// Auto-Populate (`gmIngredientUI::AutoPopulateIngredients`, which only adds ingredients and
/// selects nothing) fills the slots from the pack, and the craft goes through: the request's
/// leading id is the player's selection (`RequestExecuteCraft` reads CPlayer's selected object,
/// 0 here, as retail's script sends it), which the server takes as the first ingredient.
#[test]
fn an_auto_populated_recipe_is_crafted() {
    let Some(mut rig) = rig_for("Craftauto") else {
        return;
    };
    let pack_before = open_recipe(&mut rig, VIGOR_RECIPE);
    press(&mut rig, "AutoPopulateButton");
    assert!(
        rig.until(5.0, |r| filled_slots(r).iter().all(Option::is_some)),
        "Auto-Populate fills both slots: {:x?}",
        filled_slots(&rig)
    );
    let mark = rig.seen.len();
    press(&mut rig, "CraftButton");
    assert_crafted(&mut rig, mark, &pack_before);
}
