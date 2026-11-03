//! Spellbinding against a running Pyrealium server (see `tests/live/mod.rs`;
//! `PYREALIUM_LIVE_TEST=pyrealium_live_tradeskills_spellbinding scripts/pyrealium-live-test.sh`), as the player
//! works it: butcher a Vermin Pilferer for bone (AC2Hero `traits.html`: Vermin give bone and
//! hide) and an Idol Spawn for sinew, mine stone and wood and refine the runewood, then craft
//! `rcp_spellbinding_practice` (wood 1, runewood 35, bone 10) at the craft panel and
//! `rcp_spellbinding_study` (coal, crystal, sinew; the DAT describes it as crafting and discarding
//! samples for experience only, "OBSOLETE - DNT"). The server has the chain file's knobs and a
//! Vermin Pilferer (entity `0x47000803`, level 1) beside the start (`PYREALIUM_EXTRA_WORLD`).

#![cfg(feature = "window")]

mod live;

use ac2_ui::ToolbarChild;
use live::chain::*;
use live::trade::*;
use live::LiveRig;

const SPELLBINDING: u32 = 0x6F00_282D;
/// `rcp_spellbinding_practice`: wood, 35 runewood and 10 bone; one of the six
/// `wsl_tradegoods_spellbinding_1..6` charms.
const PRACTICE: u32 = 0x6F00_3092;
/// `rcp_spellbinding_study`: coal, crystal and sinew, one each; no item.
const STUDY: u32 = 0x6F00_3093;
const CHARMS: [u32; 6] = [
    0x4700_2FBA,
    0x4700_2FBB,
    0x4700_2FBC,
    0x4700_353F,
    0x4700_3540,
    0x4700_3541,
];
/// `rcp_toolnew_huntingknife_starter`, `wsl_toolnew_huntingknife_starter`.
const KNIFE_RECIPE: u32 = 0x6F00_2F27;
const KNIFE: u32 = 0x4700_2F73;
const BONE: u32 = 0x4700_1400;
const HIDE: u32 = 0x4700_27A3;
const SINEW: u32 = 0x4700_141F;
const TALLOW: u32 = 0x4700_27A7;
const COAL_BOUGHT: i32 = 10;
/// `wsl_verminpilferer`.
const PILFERER: u32 = 0x4700_0803;
/// `wsl_mote_wood`, its Linvak survey recipe, maps, deposit and starter hatchet.
const WOOD_MOTE: u32 = 0x4700_10F5;
const SURVEY_WOOD: u32 = 0x6F00_2AF3;
const WOOD_MAPS: std::ops::RangeInclusive<u32> = 0x4700_2EA6..=0x4700_2EAB;
const WOOD_DEPOSIT: u32 = 0x4700_2EF7;
const HATCHET: u32 = 0x4700_2F72;
/// The same for stone (starter chisel) and crystal (starter rock hammer).
const STONE_MOTE: u32 = 0x4700_10F4;
const SURVEY_STONE: u32 = 0x6F00_2AF2;
const STONE_MAPS: std::ops::RangeInclusive<u32> = 0x4700_2EA0..=0x4700_2EA5;
const STONE_DEPOSIT: u32 = 0x4700_2EF5;
const CHISEL: u32 = 0x4700_2F76;
const CRYSTAL_MOTE: u32 = 0x4700_10F2;
const SURVEY_CRYSTAL: u32 = 0x6F00_2AEF;
const CRYSTAL_MAPS: std::ops::RangeInclusive<u32> = 0x4700_2E8E..=0x4700_2E93;
const CRYSTAL_DEPOSIT: u32 = 0x4700_2EF1;
const CRYSTAL_TOOL: u32 = 0x4700_2F71;
/// `Refining: Runewood` and its `Refine Runewood (100)`: stone and wood, 100 each.
const REFINING_RUNEWOOD: u32 = 0x6F00_283E;
const REFINE_RUNEWOOD_100: u32 = 0x6F00_2A31;
const RUNEWOOD: u32 = 0x4700_27A5;

/// The living creature of `entity` in view, if any.
fn creature_of(rig: &LiveRig, entity: u32) -> Option<u64> {
    rig.client.session().objects().iter().find_map(|o| {
        (o.entity == Some(entity) && o.qualities.inq_int(0x102).is_none_or(|health| health > 0))
            .then_some(o.id)
    })
}

/// Any living attackable creature but the pilferer.
fn idol_spawn(rig: &LiveRig) -> Option<u64> {
    rig.client.session().objects().iter().find_map(|o| {
        (o.entity != Some(PILFERER)
            && o.qualities.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F)
            && o.qualities.inq_int(0x102).is_some_and(|health| health > 0))
        .then_some(o.id)
    })
}

/// Selects `target` and attacks until it is defeated (the stance is asked again when the server
/// did not take it, e.g. before the scene is ready).
fn kill(rig: &mut LiveRig, target: u64) {
    let before = rig.seen.len();
    let dead = |r: &LiveRig| {
        said_since(r, before)
            .iter()
            .any(|l| l.contains("Creature defeated"))
    };
    for _ in 0..6 {
        rig.command(ac2_client::Command::Select(Some(target)));
        rig.run(0.5);
        rig.command(ac2_client::Command::SetAttacking(true));
        if rig.until(30.0, |r| dead(r)) {
            break;
        }
        rig.command(ac2_client::Command::SetAttacking(false));
        rig.run(1.0);
    }
    rig.command(ac2_client::Command::SetAttacking(false));
    rig.run(0.5);
    assert!(
        dead(rig),
        "the creature was defeated; said {:?}",
        said_since(rig, before)
            .iter()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
    );
}

/// The player's butchering: double-click the knife, then click the body.
fn butcher(rig: &mut LiveRig, knife: u64, body: u64) {
    rig.panel(ToolbarChild::Inventory, true);
    wait(rig, "the knife's icon", |r| {
        r.view.item_icon_center(PACK, knife).is_some()
    });
    double_click_item(rig, knife);
    rig.run(0.5);
    rig.command(ac2_client::Command::Select(Some(body)));
    rig.run(0.3);
    rig.command(ac2_client::Command::UseSelection);
}

/// Opens `recipe` and presses craft. The panel lists a slot only at the spinner value that
/// reaches its quantity (wood 1, runewood 35, bone 10), so the craft button sends the recipe
/// alone and the server takes the ingredients from the pack (as the chain file's pickaxe).
fn craft_from_pack(rig: &mut LiveRig, skill: u32, recipe: u32) {
    rig.panel(ToolbarChild::Inventory, false);
    open_recipe(rig, skill, recipe);
    rig.panel(ToolbarChild::Inventory, true);
    craft_once(rig, recipe);
}

#[test]
fn spellbinding_practice_and_study_use_butchered_bone_and_sinew() {
    let Some(mut rig) = rig_for("Spellbinder") else {
        return;
    };
    // The pilferer attacks at once (AC2Hero `traits.html`: Vermin give bone and hide), so the
    // player fights it first; its body lasts a minute, long enough to craft the knife at the
    // craft panel and butcher it.
    assert!(
        rig.until(30.0, |r| creature_of(r, PILFERER).is_some()),
        "the pilferer stands beside the start"
    );
    rig.run(3.0);
    let vermin = creature_of(&rig, PILFERER).unwrap();
    kill(&mut rig, vermin);
    open_recipe(&mut rig, TOOLMAKING, KNIFE_RECIPE);
    craft_once(&mut rig, KNIFE_RECIPE);
    rig.panel(ToolbarChild::Skill, false);
    assert!(
        rig.until(10.0, |r| !objects_of(r, |e| e == KNIFE).is_empty()),
        "the knife in the pack"
    );
    let knife = objects_of(&rig, |e| e == KNIFE)[0];
    butcher(&mut rig, knife, vermin);
    assert!(
        rig.until(10.0, |r| units(r, BONE) == 10 && units(r, HIDE) == 10),
        "ten bone and ten hide; said {:?}",
        lines(&rig).iter().rev().take(4).collect::<Vec<_>>()
    );

    // Sinew: an Idol Spawn (`traits.html`: Spawn give sinew and tallow).
    assert!(
        rig.until(30.0, |r| idol_spawn(r).is_some()),
        "an Idol Spawn"
    );
    let spawn = idol_spawn(&rig).unwrap();
    let tallow = units(&rig, TALLOW);
    kill(&mut rig, spawn);
    butcher(&mut rig, knife, spawn);
    assert!(
        rig.until(10.0, |r| units(r, SINEW) == 10
            && units(r, TALLOW) == tallow + 10),
        "ten sinew and ten tallow"
    );

    // Wood and stone, merged into stacks of 100 and refined to runewood.
    let wood = gather(
        &mut rig,
        SURVEY_WOOD,
        WOOD_MAPS,
        WOOD_DEPOSIT,
        HATCHET,
        WOOD_MOTE,
    );
    let wood_per = per_strike(&mut rig);
    assert_eq!(wood, STRIKES as i32 * wood_per);
    merge_to(&mut rig, WOOD_MOTE, i32::MAX);
    let stone = gather(
        &mut rig,
        SURVEY_STONE,
        STONE_MAPS,
        STONE_DEPOSIT,
        CHISEL,
        STONE_MOTE,
    );
    let stone_per = per_strike(&mut rig);
    assert_eq!(stone, STRIKES as i32 * stone_per);
    merge_to(&mut rig, STONE_MOTE, i32::MAX);
    rig.panel(ToolbarChild::Inventory, false);
    open_recipe(&mut rig, REFINING_RUNEWOOD, REFINE_RUNEWOOD_100);
    let rank = rig
        .client
        .craft_skill_level(REFINING_RUNEWOOD)
        .expect("the runewood skill")
        .0;
    rig.panel(ToolbarChild::Inventory, true);
    fill_slots(&mut rig, REFINE_RUNEWOOD_100);
    craft_once(&mut rig, REFINE_RUNEWOOD_100);
    // AC2Hero `refining.html`: 34 to 100 percent by the refining rank.
    let runewood = 100 * refine_percent(rank) / 100;
    assert_eq!(units(&rig, RUNEWOOD), runewood, "runewood at rank {rank}");
    assert!(runewood >= 35, "the practice recipe wants 35 runewood");

    // Practice Spellbinding: wood 1, runewood 35, bone 10 make one of the six charms.
    let wood_left = units(&rig, WOOD_MOTE);
    let charms = objects_of(&rig, |e| CHARMS.contains(&e));
    craft_from_pack(&mut rig, SPELLBINDING, PRACTICE);
    assert!(
        rig.until(10.0, |r| objects_of(r, |e| CHARMS.contains(&e)).len()
            == charms.len() + 1),
        "a charm in the pack"
    );
    let charm = *objects_of(&rig, |e| CHARMS.contains(&e))
        .iter()
        .max()
        .unwrap();
    wait(&mut rig, "the charm's icon", |r| {
        r.view.item_icon_center(PACK, charm).is_some()
    });
    let name = rig
        .view
        .backpack_items()
        .into_iter()
        .find(|(id, _)| *id == charm)
        .map(|(_, n)| n)
        .unwrap_or_default();
    eprintln!("the charm's name: {name:?}");
    assert!(
        !name.is_empty() && !name.contains("Error"),
        "the charm is named: {name:?}"
    );
    assert_eq!(units(&rig, RUNEWOOD), runewood - 35, "35 runewood used");
    assert_eq!(units(&rig, BONE), 0, "10 bone used");
    assert_eq!(units(&rig, WOOD_MOTE), wood_left - 1, "1 wood used");

    // Study Spellbinding: coal, crystal and sinew, one each, are used up for experience.
    let crystal = gather_n(
        &mut rig,
        SURVEY_CRYSTAL,
        CRYSTAL_MAPS,
        CRYSTAL_DEPOSIT,
        CRYSTAL_TOOL,
        CRYSTAL_MOTE,
        1,
    );
    assert_eq!(crystal, per_strike(&mut rig));
    rig.relog();
    open_trader(&mut rig, WOLFGANG);
    buy_tens(&mut rig, "Coal", 1);
    assert!(
        rig.until(10.0, |r| units(r, COAL) == COAL_BOUGHT),
        "10 coal bought"
    );
    rig.relog();
    // After the log-in the client names the charm from the DAT's class defaults itself: the name
    // Pyrealium sent at its creation is that name.
    rig.panel(ToolbarChild::Inventory, true);
    wait(&mut rig, "the charm's icon after the log-in", |r| {
        r.view.item_icon_center(PACK, charm).is_some()
    });
    let named = rig
        .view
        .backpack_items()
        .into_iter()
        .find(|(id, _)| *id == charm)
        .map(|(_, n)| n);
    assert_eq!(
        named.as_deref(),
        Some(name.as_str()),
        "the DAT's name of the charm"
    );
    let before: Vec<i32> = [COAL, CRYSTAL_MOTE, SINEW]
        .iter()
        .map(|&d| units(&rig, d))
        .collect();
    let items = rig.client.frame().inventory.items.len();
    craft_from_pack(&mut rig, SPELLBINDING, STUDY);
    rig.run(1.0);
    for (definition, had) in [COAL, CRYSTAL_MOTE, SINEW].into_iter().zip(before) {
        assert_eq!(
            units(&rig, definition),
            had - 1,
            "{definition:08x}: one used"
        );
    }
    assert_eq!(
        rig.client.frame().inventory.items.len(),
        items,
        "the study makes no item"
    );
}
