//! Helpers of the live tradeskills chain tests: the pack's stacks, mining and refining as the
//! player works them (`ac2e_live_tradeskills_chain.rs`, `ac2e_live_tradeskills_spellbinding.rs`).

use super::trade::*;
use super::LiveRig;
use ac2_ui::ToolbarChild;

/// The strikes a deposit holds (`AC2E_DEV_DEPOSIT_CHARGES`).
pub const STRIKES: usize = 12;

pub fn quantity(rig: &LiveRig, id: u64) -> i32 {
    rig.client
        .frame()
        .inventory
        .items
        .iter()
        .find(|i| i.id == id)
        .map_or(0, |i| i.stats.quantity)
}

/// The stacks of definition `definition` in the pack: (id, units).
pub fn stacks(rig: &LiveRig, definition: u32) -> Vec<(u64, i32)> {
    objects_of(rig, |e| e == definition)
        .into_iter()
        .map(|id| (id, quantity(rig, id)))
        .filter(|&(_, q)| q > 0)
        .collect()
}

pub fn units(rig: &LiveRig, definition: u32) -> i32 {
    stacks(rig, definition).iter().map(|&(_, q)| q).sum()
}

/// The units a mining strike gives: the deposit's ten (its weenie description) and AC2Hero's
/// Surveying bonus, up to 40 at rank 150 (`survey.html`; AC2e `tradeskills::survey_bonus`), read
/// from the rank on the panel.
pub fn per_strike(rig: &mut LiveRig) -> i32 {
    let rank = rig.client.craft_skill_level(SURVEYING).map_or(1, |l| l.0);
    (10 + 40 * rank.min(150) / 150) as i32
}

/// The percent of its ingredients a refining recipe returns at `rank` (AC2Hero `refining.html`:
/// 34 to 100 percent, better with the skill; AC2e `tradeskills::refine_percent`).
pub fn refine_percent(rank: u32) -> i32 {
    (34 + 66 * (rank.clamp(1, 150) - 1) / 149) as i32
}

/// Wields `tool` in the primary hand.
pub fn wield(rig: &mut LiveRig, tool: u32) {
    let item = objects_of(rig, |e| e == tool)[0];
    rig.command(ac2_client::Command::EquipItem {
        item,
        location: PRIMARY_HAND,
    });
    let held = |r: &LiveRig| {
        r.client
            .frame()
            .inventory
            .equipped
            .iter()
            .any(|&(location, id)| location == PRIMARY_HAND && id == item)
    };
    assert!(
        rig.until(30.0, held),
        "the tool wielded; chat {:?}",
        lines(rig).iter().rev().take(4).collect::<Vec<_>>()
    );
}

/// Crafts a surveying recipe, uses the map it made, wields `tool` and works the deposit until it
/// is spent; returns the units the strikes gave.
pub fn gather(
    rig: &mut LiveRig,
    survey: u32,
    maps: std::ops::RangeInclusive<u32>,
    entity: u32,
    tool: u32,
    mote: u32,
) -> i32 {
    gather_n(rig, survey, maps, entity, tool, mote, STRIKES)
}

pub fn gather_n(
    rig: &mut LiveRig,
    survey: u32,
    maps: std::ops::RangeInclusive<u32>,
    entity: u32,
    tool: u32,
    mote: u32,
    strikes: usize,
) -> i32 {
    let before = units(rig, mote);
    open_recipe(rig, SURVEYING, survey);
    craft_once(rig, survey);
    rig.panel(ToolbarChild::Skill, false);
    rig.panel(ToolbarChild::Inventory, true);
    assert!(
        rig.until(10.0, |r| !objects_of(r, |e| maps.contains(&e)).is_empty()),
        "a map in the pack"
    );
    let map = objects_of(rig, |e| maps.contains(&e))[0];
    wait(rig, "the map's icon", |r| {
        r.view.item_icon_center(PACK, map).is_some()
    });
    // The scene is the server's: deposits other characters prospected and left unworked may come
    // into view as the map travel arrives, and only the surveyor works a deposit. AC2e numbers
    // the deposits it places in order, so the one this map revealed is the highest.
    let discovered = |r: &LiveRig| {
        lines(r)
            .iter()
            .filter(|l| l.contains("deposit nearby"))
            .count()
    };
    let found = discovered(rig);
    use_map(rig, map);
    assert!(
        rig.until(15.0, |r| discovered(r) > found
            && !objects_of(r, |e| e == entity).is_empty()),
        "the map reveals the deposit"
    );
    rig.run(1.0);
    let deposit = objects_of(rig, |e| e == entity).into_iter().max().unwrap();
    let per = per_strike(rig);
    // The server uses the map up in the same turn that reveals the deposit, but the destroy of
    // the map reaches the pack after the deposit's creation. The equip request carries the
    // item's pack slot (`CInventorySystem` builds `InvEquipDesc` from the script's list), and
    // AC2e checks it against its own bag: while the used map is still listed (it sits at the
    // pack's front) the slot is one too many and AC2e answers "Equip rejected" (chat only).
    // The player wields once the map is gone from the pack.
    wait(rig, "the used map leaves the pack", |r| {
        !r.client.frame().inventory.items.iter().any(|i| i.id == map)
    });
    wield(rig, tool);
    for _ in 0..strikes {
        let have = units(rig, mote);
        rig.command(ac2_client::Command::Use(deposit));
        assert!(
            rig.until(15.0, |r| units(r, mote) == have + per),
            "a strike gives {per} units: {} then {}; chat {:?}",
            have,
            units(rig, mote),
            lines(rig).iter().rev().take(5).collect::<Vec<_>>()
        );
        rig.run(0.3);
        // A pack page holds 42 slots: merge the strikes' stacks as the player does, so the pack
        // does not run past its first page.
        if strikes > 12 && stacks(rig, mote).len() >= 20 {
            merge_to(rig, mote, i32::MAX);
            rig.panel(ToolbarChild::Inventory, false);
        }
    }
    units(rig, mote) - before
}

/// Merges the stacks of `definition` as the player does: a stack dragged onto the first one, the
/// stack dialog's OK for all of it, until the first holds `to` units or no stack is left.
pub fn merge_to(rig: &mut LiveRig, definition: u32, to: i32) {
    rig.panel(ToolbarChild::Inventory, true);
    loop {
        let all = stacks(rig, definition);
        let target = all[0];
        if target.1 >= to || all.len() < 2 {
            return;
        }
        let moved = all[1];
        let from = rig
            .view
            .item_icon_center(PACK, moved.0)
            .expect("the stack's icon");
        let onto = rig
            .view
            .item_icon_center(PACK, target.0)
            .expect("the target's icon");
        // The drop asks how many (the dialog may take a few frames to show, as after a relog).
        let mut asked = false;
        for _ in 0..3 {
            rig.drag(from, onto);
            if rig.until(3.0, |r| r.view.ui_element_center("StackOkButton").is_some()) {
                asked = true;
                break;
            }
        }
        assert!(
            asked,
            "the stack dialog asks how many; said {:?}",
            lines(rig).iter().rev().take(3).collect::<Vec<_>>()
        );
        let ok = rig.view.ui_element_center("StackOkButton").unwrap();
        rig.click(ok);
        assert!(
            rig.until(5.0, |r| {
                let s = stacks(r, definition);
                s.iter().map(|&(_, q)| q).max() >= Some(target.1 + moved.1) || s.len() < all.len()
            }),
            "the stacks merged: {:?}; asked {:?}; said {:?}",
            stacks(rig, definition),
            rig.asked.iter().rev().take(4).collect::<Vec<_>>(),
            lines(rig).iter().rev().take(3).collect::<Vec<_>>()
        );
        rig.run(0.3);
    }
}

/// Drags the pack item the client's own `CheckIngredient` accepts into each slot of the open
/// recipe, in the panel's slot order; returns the items used.
pub fn fill_slots(rig: &mut LiveRig, recipe: u32) -> Vec<u64> {
    let slots = rows(rig, "IngredientListBox");
    let (_, ingredients) = rig
        .client
        .recipe_ingredients(recipe, 0)
        .expect("the recipe's ingredients");
    let ordinals: Vec<u32> = ingredients.iter().map(|i| i.id).collect();
    assert_eq!(slots.len(), ordinals.len(), "a slot an ingredient");
    let mut used = Vec::new();
    let pack: Vec<u64> = rig.view.item_icons(PACK);
    for (slot, ordinal) in slots.iter().zip(ordinals) {
        let item = pack
            .iter()
            .copied()
            .find(|&i| {
                !used.contains(&i)
                    && rig.client.check_ingredient(recipe, ordinal, i, 0) == Some(true)
            })
            .unwrap_or_else(|| panic!("a pack item for slot {ordinal}"));
        let from = rig.view.item_icon_center(PACK, item).unwrap();
        let to = rig.view.ui_index_center(*slot).unwrap();
        rig.drag(from, to);
        used.push(item);
    }
    used
}
