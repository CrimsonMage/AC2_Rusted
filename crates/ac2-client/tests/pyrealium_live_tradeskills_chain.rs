//! The gathering-to-tool chain against a running Pyrealium server (see `tests/live/mod.rs`), as the
//! player works it: survey and mine iron and wood, merge the motes into stacks of 100 (a stack
//! dragged onto another), refine them to steel at the craft panel, and craft a pickaxe from
//! the steel with the rest of its ingredients.
//! Needs Pyrealium's tradeskills profile with the chain knobs (`scripts/pyrealium-live-test.sh` with
//! `PYREALIUM_LIVE_TEST=pyrealium_live_tradeskills_chain`): `PYREALIUM_DEV_CRAFT_XP` (ranks for the recipes'
//! minimum levels), `PYREALIUM_DEV_DEPOSIT_CHARGES` (12 strikes a deposit instead of the entity's 3,
//! so one deposit gives the 100 units a refining recipe wants) and `PYREALIUM_DEV_START_GOLD` (a purse
//! for the traders' coal and flawless pyreal, which the DAT's stores sell; no supplies are granted).

#![cfg(feature = "window")]

mod live;

use ac2_ui::ToolbarChild;
use live::chain::*;
use live::trade::*;

/// `rcp_surveying_linvak_wood`; its maps `wsl_surveyorsmap_linvak_wood1..6`.
const SURVEY_LINVAK_WOOD: u32 = 0x6F00_2AF3;
const WOOD_MAPS: std::ops::RangeInclusive<u32> = 0x4700_2EA6..=0x4700_2EAB;
/// `wsl_mine_wood_linvak`, `wsl_toolnew_hatchet_starter`, `wsl_mote_wood`.
const WOOD_DEPOSIT: u32 = 0x4700_2EF7;
const HATCHET: u32 = 0x4700_2F72;
const WOOD_MOTE: u32 = 0x4700_10F5;
const REFINING_STEEL: u32 = 0x6F00_283F;
/// `Refine Steel (100)`: Wood and Iron, 100 each (ordinals 2 and 1).
const REFINE_STEEL_100: u32 = 0x6F00_2A34;
const STEEL: u32 = 0x4700_27A6;
/// `rcp_toolnew_pickaxe`: coal 20, iron 1, steel 35, tallow 10, flawless pyreal 45.
const PICKAXE_RECIPE: u32 = 0x6F00_2854;
const NEW_PICKAXE: u32 = 0x4700_2DEB;

/// What the client's ingredient check (`gmCWInterface::CheckIngredient`, the player's script)
/// takes for the iron slot of `Refine Steel (100)`: a stack of 10 is refused, the merged 100
/// is taken, and an item of another kind is refused. A slot holds one item, so a slot of 100
/// is filled by one stack of 100: the player merges the strikes' stacks first.
#[test]
fn the_client_check_of_an_ingredient_slot_wants_the_recipes_quantity() {
    let Some(mut rig) = rig_for("Chainprobe") else {
        return;
    };
    open_craft_tab(&mut rig);
    wait(&mut rig, "the refining recipes", |r| {
        r.client
            .recipes(REFINING_STEEL)
            .is_some_and(|x| x.len() >= 4)
    });
    let iron_slot = 1;
    let units = gather(
        &mut rig,
        SURVEY_LINVAK_IRON,
        IRON_MAPS,
        IRON_DEPOSIT,
        PICKAXE,
        IRON_MOTE,
    );
    let per = per_strike(&mut rig);
    assert_eq!(units, 12 * per, "twelve strikes");
    let ten = stacks(&rig, IRON_MOTE)[0].0;
    assert_eq!(
        rig.client
            .check_ingredient(REFINE_STEEL_100, iron_slot, ten, 0),
        Some(false),
        "a stack of 10 does not fill a slot of 100"
    );
    merge_to(&mut rig, IRON_MOTE, 100);
    let merged = stacks(&rig, IRON_MOTE)[0];
    assert!(merged.1 >= 100, "merged to at least 100: {}", merged.1);
    assert_eq!(
        rig.client
            .check_ingredient(REFINE_STEEL_100, iron_slot, merged.0, 0),
        Some(true),
        "a stack of 100 fills it"
    );
    let tallow = objects_of(&rig, |e| e == 0x4700_27A7);
    assert_eq!(
        rig.client
            .check_ingredient(REFINE_STEEL_100, iron_slot, tallow[0], 0),
        Some(false),
        "tallow is not iron"
    );
}

#[test]
fn iron_and_wood_are_mined_refined_to_steel_and_made_into_a_pickaxe() {
    let Some(mut rig) = rig_for("Chainfull") else {
        return;
    };
    let iron = gather(
        &mut rig,
        SURVEY_LINVAK_IRON,
        IRON_MAPS,
        IRON_DEPOSIT,
        PICKAXE,
        IRON_MOTE,
    );
    let iron_per = per_strike(&mut rig);
    assert_eq!(iron, 12 * iron_per, "twelve strikes");
    let wood = gather(
        &mut rig,
        SURVEY_LINVAK_WOOD,
        WOOD_MAPS,
        WOOD_DEPOSIT,
        HATCHET,
        WOOD_MOTE,
    );
    let wood_per = per_strike(&mut rig);
    assert_eq!(wood, 12 * wood_per);
    assert_eq!(stacks(&rig, IRON_MOTE).len(), 12, "a stack a strike");
    // The strikes' stacks are merged into one of at least 100 (a slot takes one item with the
    // whole quantity).
    merge_to(&mut rig, IRON_MOTE, 100);
    merge_to(&mut rig, WOOD_MOTE, 100);
    assert!(stacks(&rig, IRON_MOTE)[0].1 >= 100);
    assert!(stacks(&rig, WOOD_MOTE)[0].1 >= 100);
    // The pickaxe's coal and flawless pyreal come from the DAT's traders (`store_cragstone.wc`:
    // coal at half a coin; `store_cavendo.wc`: pyreal at 50), the way a player gets them. They
    // are bought with the pack holding the mined stacks (about 30 items): the store's quantity
    // dialog opens at that size. (The earlier note that it did not rested on a wrong cause: the
    // dialog never opened after mining because the trader's travel did not happen. A second
    // `@maptravel` on one login leaves the client where it was, so the character logs in again
    // before each trader.)
    rig.relog();
    open_trader(&mut rig, WOLFGANG);
    buy_tens(&mut rig, "Coal", 2);
    assert!(rig.until(10.0, |r| units(r, COAL) == 20), "20 coal bought");
    rig.relog();
    open_trader(&mut rig, SABINE);
    buy_tens(&mut rig, "Flawless Pyreal", 5);
    assert!(
        rig.until(10.0, |r| units(r, PYREAL) == 50),
        "50 pyreal bought"
    );
    rig.relog();

    // Refine Steel (100) from the craft panel: Iron and Wood in their slots.
    rig.panel(ToolbarChild::Inventory, false);
    open_recipe(&mut rig, REFINING_STEEL, REFINE_STEEL_100);
    rig.panel(ToolbarChild::Inventory, true);
    let level = rig
        .client
        .craft_skill_level(REFINING_STEEL)
        .expect("the steel skill")
        .0;
    let used = fill_slots(&mut rig, REFINE_STEEL_100);
    assert_eq!(used.len(), 2);
    let mark = rig.seen.len();
    press(&mut rig, "CraftButton");
    assert!(
        rig.until(15.0, |r| r.seen[mark..].iter().any(|e| matches!(
            e,
            ac2_weenie::SessionEvent::Notice(ac2_weenie::GameplayNotice::ExecuteCraftDone {
                error: 0,
                ..
            })
        ))),
        "the refining done"
    );
    rig.run(1.0);
    // AC2Hero (`refining.html`): the recipe returns 34 to 100 percent of the 100, more with the
    // refining rank. The pickaxe wants 35 steel, so the rank must reach 35 percent.
    let expected = 100 * refine_percent(level) / 100;
    assert!(expected >= 35, "rank {level} refines {expected} of 100");
    assert_eq!(units(&rig, STEEL), expected, "steel at rank {level}");
    assert_eq!(
        units(&rig, IRON_MOTE),
        12 * iron_per - 100,
        "the refined hundred gone"
    );
    assert_eq!(units(&rig, WOOD_MOTE), 12 * wood_per - 100);

    // The pickaxe: coal 20, iron 1, steel 35, tallow 10, flawless pyreal 45.
    open_recipe(&mut rig, TOOLMAKING, PICKAXE_RECIPE);
    rig.panel(ToolbarChild::Inventory, true);
    // The panel lists no slot for these (a slot appears at the spinner value that reaches its
    // count, and the recipe's spinner is fixed): the craft button sends the recipe alone and the
    // server takes the coal, iron, steel, tallow and flawless pyreal from the pack.
    assert_eq!(
        rows(&rig, "IngredientListBox").len(),
        1,
        "a placeholder row only"
    );
    let pickaxes = objects_of(&rig, |e| e == NEW_PICKAXE).len();
    craft_once(&mut rig, PICKAXE_RECIPE);
    assert!(
        rig.until(10.0, |r| objects_of(r, |e| e == NEW_PICKAXE).len()
            == pickaxes + 1),
        "the new pickaxe in the pack"
    );
    assert_eq!(
        units(&rig, STEEL),
        expected - 35,
        "35 of the steel went into it"
    );
    assert_eq!(units(&rig, COAL), 0, "the 20 coal bought");
    assert_eq!(units(&rig, PYREAL), 5, "45 of the 50 flawless pyreal");
    assert_eq!(units(&rig, 0x4700_27A7), 90, "10 of the kit's 100 tallow");
    assert_eq!(
        units(&rig, IRON_MOTE),
        12 * iron_per - 101,
        "1 more iron used"
    );
}

/// `Refine Steel (500)`: Iron and Wood, 500 each. The DAT's stack limit for a mote is 1,000,000
/// (`wsl_mote_iron` property `0x41000089` and the description's `0x105`), a slot holds one item,
/// and the client's `CheckIngredient` wants that item to hold the slot's whole quantity: the
/// player merges the strikes' stacks into one of 500.
const REFINE_STEEL_500: u32 = 0x6F00_2A36;

#[test]
fn a_refining_recipe_of_five_hundred_is_filled_by_one_stack_of_five_hundred() {
    let Some(mut rig) = rig_for("Refinefive") else {
        return;
    };
    let iron = gather_n(
        &mut rig,
        SURVEY_LINVAK_IRON,
        IRON_MAPS,
        IRON_DEPOSIT,
        PICKAXE,
        IRON_MOTE,
        50,
    );
    let iron_per = per_strike(&mut rig);
    assert_eq!(iron, 50 * iron_per, "fifty strikes");
    let wood = gather_n(
        &mut rig,
        SURVEY_LINVAK_WOOD,
        WOOD_MAPS,
        WOOD_DEPOSIT,
        HATCHET,
        WOOD_MOTE,
        50,
    );
    let wood_per = per_strike(&mut rig);
    assert_eq!(wood, 50 * wood_per);
    merge_to(&mut rig, IRON_MOTE, i32::MAX);
    merge_to(&mut rig, WOOD_MOTE, i32::MAX);
    assert_eq!(stacks(&rig, IRON_MOTE).len(), 1);
    assert_eq!(units(&rig, IRON_MOTE), 50 * iron_per);
    assert_eq!(stacks(&rig, WOOD_MOTE).len(), 1);
    rig.panel(ToolbarChild::Inventory, false);
    open_recipe(&mut rig, REFINING_STEEL, REFINE_STEEL_500);
    rig.panel(ToolbarChild::Inventory, true);
    let level = rig
        .client
        .craft_skill_level(REFINING_STEEL)
        .expect("the steel skill")
        .0;
    let used = fill_slots(&mut rig, REFINE_STEEL_500);
    assert_eq!(used.len(), 2);
    let mark = rig.seen.len();
    press(&mut rig, "CraftButton");
    assert!(
        rig.until(15.0, |r| r.seen[mark..].iter().any(|e| matches!(
            e,
            ac2_weenie::SessionEvent::Notice(ac2_weenie::GameplayNotice::ExecuteCraftDone {
                error: 0,
                ..
            })
        ))),
        "the refining done"
    );
    rig.run(1.0);
    let expected = 500 * refine_percent(level) / 100;
    assert_eq!(units(&rig, STEEL), expected, "steel at rank {level}");
    assert_eq!(
        units(&rig, IRON_MOTE),
        50 * iron_per - 500,
        "the five hundred are used"
    );
    assert_eq!(units(&rig, WOOD_MOTE), 50 * wood_per - 500);
}

/// `craftskillhumanarmor` and its `rcp_ha1tb`: iron 100 and tallow 100 (the training kit's bundle)
/// make `wsl_brackettb`, an armour component, at rank 2. Armour pieces themselves want hide, sinew,
/// chitin or refined metals with coal, deruwood and pyreal; this is the one armour recipe whose
/// ingredients a new character reaches: the pickaxe's iron and the kit's tallow.
#[test]
fn the_human_armour_skill_crafts_a_bracket_from_iron_and_the_kits_tallow() {
    const HUMAN_ARMOR: u32 = 0x6F00_23A8;
    const BRACKET_RECIPE: u32 = 0x6F00_367E;
    const BRACKET: u32 = 0x4700_35CC;
    let Some(mut rig) = rig_for("Armourone") else {
        return;
    };
    let iron = gather(
        &mut rig,
        SURVEY_LINVAK_IRON,
        IRON_MAPS,
        IRON_DEPOSIT,
        PICKAXE,
        IRON_MOTE,
    );
    let iron_per = per_strike(&mut rig);
    assert_eq!(iron, 12 * iron_per);
    merge_to(&mut rig, IRON_MOTE, 100);
    rig.panel(ToolbarChild::Inventory, false);
    open_recipe(&mut rig, HUMAN_ARMOR, BRACKET_RECIPE);
    rig.panel(ToolbarChild::Inventory, true);
    let used = fill_slots(&mut rig, BRACKET_RECIPE);
    assert_eq!(used.len(), 2);
    assert!(objects_of(&rig, |e| e == BRACKET).is_empty());
    craft_once(&mut rig, BRACKET_RECIPE);
    assert!(
        rig.until(10.0, |r| !objects_of(r, |e| e == BRACKET).is_empty()),
        "the bracket in the pack"
    );
    let bracket = objects_of(&rig, |e| e == BRACKET)[0];
    wait(&mut rig, "the bracket's icon", |r| {
        r.view.item_icon_center(PACK, bracket).is_some()
    });
    // The server names the item in its description (Pyrealium `item_names`); the entity's name is
    // string table 250080b9 "Iron Bracket[v]".
    assert_eq!(
        rig.view
            .backpack_items()
            .into_iter()
            .find(|(id, _)| *id == bracket)
            .map(|(_, n)| n)
            .as_deref(),
        Some("Iron Bracket"),
        "the crafted bracket's name"
    );
    assert_eq!(units(&rig, IRON_MOTE), 12 * iron_per - 100, "100 iron used");
    assert_eq!(units(&rig, 0x4700_27A7), 0, "the kit's 100 tallow used");
}
