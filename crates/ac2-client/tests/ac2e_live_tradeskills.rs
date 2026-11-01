//! Tradeskills against a running AC2e server (see `tests/live/mod.rs`): the craft panel lists
//! Surveying, Mining and Toolmaking beside the quest recipes, a Surveying recipe makes a
//! surveyor's map, the map reveals a deposit, a mining tool works it for motes, and crafting
//! experience raises the skill's rank on the panel.
//! Needs AC2e's tradeskills profile (`AC2E_DEV_TRADESKILLS`, with `--training-kit true`);
//! `scripts/ac2e-live-test.sh` with `AC2E_LIVE_TEST=ac2e_live_tradeskills` sets that up.

#![cfg(feature = "window")]

mod live;

use ac2_ui::{tokenize, AttributeValue, ToolbarChild};
use ac2_weenie::{GameplayNotice, SessionEvent};
use live::LiveRig;

const SURVEYING: u32 = 0x6F00_264A;
const MINING_IRON: u32 = 0x6F00_2579;
const TOOLMAKING: u32 = 0x6F00_2857;
/// `rcp_surveying_linvak_iron`: makes one of the six Linvak iron surveyor's maps.
const SURVEY_LINVAK_IRON: u32 = 0x6F00_2AF0;
/// `wsl_surveyorsmap_linvak_iron1..6`.
const IRON_MAPS: std::ops::RangeInclusive<u32> = 0x4700_2E94..=0x4700_2E99;
/// `wsl_mine_iron_linvak`, the deposit those maps reveal.
const IRON_DEPOSIT: u32 = 0x4700_2EEF;
/// `wsl_toolnew_pickaxe_starter`, `wsl_mote_iron`.
const PICKAXE: u32 = 0x4700_2F75;
const IRON_MOTE: u32 = 0x4700_10F3;
const PACK: &str = "PackMainContainer";
const PRIMARY_HAND: u32 = 0x40;
/// The second press of a double click as the input layer sends it.
const DOUBLE_CLICK: u32 = 0x4000_0007;

fn rig_for(name: &str) -> Option<LiveRig> {
    if std::env::var_os("AC2E_LIVE_TRAINING_KIT").is_none()
        || std::env::var_os("AC2E_DEV_TRADESKILLS").is_none()
    {
        eprintln!("the server has no tradeskills profile (AC2E_DEV_TRADESKILLS); skipped");
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

fn press(rig: &mut LiveRig, name: &str) {
    let mut center = None;
    wait(rig, name, |r| {
        center = r.view.ui_element_center(name);
        center.is_some()
    });
    rig.click(center.unwrap());
}

fn row_did(rig: &LiveRig, row: usize) -> u32 {
    let tree = rig.view.ui_tree().expect("the HUD");
    match tree.get(row).unwrap().attribute(tokenize("did")) {
        Some(AttributeValue::Id(did)) => *did,
        _ => 0,
    }
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

/// Waits for the player's craft registry and opens the craft tab of the skill panel.
fn open_craft_tab(rig: &mut LiveRig) {
    wait(rig, "the craft skills", |r| {
        r.client.craft_skills().is_some_and(|s| s.len() >= 7)
    });
    rig.panel(ToolbarChild::Skill, true);
    press(rig, "CraftTab");
    rig.run(0.5);
}

/// The ids of the world objects (the pack's items included) of entity `entity`.
fn objects_of(rig: &LiveRig, entity: impl Fn(u32) -> bool) -> Vec<u64> {
    rig.client
        .session()
        .objects()
        .iter()
        .filter(|o| o.entity.is_some_and(&entity))
        .map(|o| o.id)
        .collect()
}

/// Down the craft panel's views to the ingredient panel of `recipe`, as the player goes: the
/// skill row, the category that lists the recipe, the recipe row.
fn open_recipe(rig: &mut LiveRig, skill: u32, recipe: u32) {
    open_craft_tab(rig);
    // The panel's rows fill once the recipe records the script reads are resident.
    wait(rig, "the recipes", |r| {
        r.client.recipes(skill).is_some_and(|x| x.len() > 3)
    });
    rig.run(3.0);
    wait(rig, "the skill row", |r| {
        rows(r, "CraftSkillList")
            .iter()
            .any(|&row| row_did(r, row) == skill)
    });
    let row = rows(rig, "CraftSkillList")
        .into_iter()
        .find(|&row| row_did(rig, row) == skill)
        .unwrap();
    double_click_row(rig, row);
    wait(rig, "the categories", |r| {
        !rows(r, "CraftSkillCategoryList").is_empty()
    });
    let count = rows(rig, "CraftSkillCategoryList").len();
    for i in 0..count {
        if i > 0 {
            press(rig, "BackButton");
            rig.run(1.0);
        }
        let category = rows(rig, "CraftSkillCategoryList")[i];
        double_click_row(rig, category);
        rig.run(1.0);
        let listed = rows(rig, "RecipeList");
        if let Some(&r) = listed.iter().find(|&&r| row_did(rig, r) == recipe) {
            double_click_row(rig, r);
            wait(rig, "the ingredient panel", |r| {
                r.view.ui_element_center("RealIngredientField").is_some()
            });
            return;
        }
    }
    panic!("no category of skill {skill:08x} lists recipe {recipe:08x}");
}

/// Presses the craft button and waits for the craft to finish without an error.
fn craft_once(rig: &mut LiveRig, recipe: u32) {
    let mark = rig.seen.len();
    press(rig, "CraftButton");
    assert!(
        rig.until(15.0, |r| r.seen[mark..].iter().any(|e| matches!(
            e,
            SessionEvent::Notice(GameplayNotice::ExecuteCraftDone { recipe: done, error: 0 })
                if *done == recipe
        ))),
        "no ExecuteCraftDone for {recipe:08x}: {:?}",
        rig.client.take_script_problems()
    );
    rig.run(0.5);
}

#[test]
fn the_craft_panel_lists_the_tradeskills() {
    let Some(mut rig) = rig_for("Tradelist") else {
        return;
    };
    open_craft_tab(&mut rig);
    let listed: Vec<u32> = rows(&rig, "CraftSkillList")
        .iter()
        .map(|&r| row_did(&rig, r))
        .collect();
    for skill in [SURVEYING, MINING_IRON, TOOLMAKING] {
        assert!(listed.contains(&skill), "{skill:08x} in {listed:x?}");
    }
    assert_eq!(listed.len(), 8, "Quest recipes, Surveying, five Mining skills, Toolmaking");
    // Every Surveying recipe of the DAT is known at rank 1 (their minimum level is 0): the
    // registry's hash table must be sized for them or the client's lookups miss entries.
    wait(&mut rig, "the surveying recipes", |r| {
        r.client
            .recipes(SURVEYING)
            .is_some_and(|x| x.len() == 78)
    });
    // The rank shows on the panel: Surveying starts at 1, its raise costs the DAT curve's 42.
    assert_eq!(rig.client.craft_skill_level(SURVEYING), Some((1, 1)));
    assert_eq!(rig.client.craft_skill(SURVEYING).unwrap().cost_to_raise, 42);
}

#[test]
fn surveying_makes_a_map_that_reveals_a_deposit_the_tool_works() {
    let Some(mut rig) = rig_for("Tradesurvey") else {
        return;
    };
    // The surveying recipe, from the craft panel.
    open_recipe(&mut rig, SURVEYING, SURVEY_LINVAK_IRON);
    assert!(objects_of(&rig, |e| IRON_MAPS.contains(&e)).is_empty());
    craft_once(&mut rig, SURVEY_LINVAK_IRON);
    rig.panel(ToolbarChild::Skill, false);
    rig.panel(ToolbarChild::Inventory, true);
    assert!(
        rig.until(10.0, |r| objects_of(r, |e| IRON_MAPS.contains(&e)).len() == 1),
        "one Linvak iron surveyor's map in the pack"
    );
    let map = objects_of(&rig, |e| IRON_MAPS.contains(&e))[0];

    // The map is used from the pack (a double click on its icon): the deposit appears.
    assert!(objects_of(&rig, |e| e == IRON_DEPOSIT).is_empty());
    wait(&mut rig, "the map's icon", |r| {
        r.view.item_icon_center(PACK, map).is_some()
    });
    let at = rig.view.item_icon_center(PACK, map).unwrap();
    rig.point(at);
    rig.button(true);
    rig.button(false);
    rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
    rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
    assert!(
        rig.until(15.0, |r| objects_of(r, |e| e == IRON_DEPOSIT).len() == 1),
        "the map reveals a vein of iron"
    );
    assert!(
        rig.until(5.0, |r| objects_of(r, |e| IRON_MAPS.contains(&e)).is_empty()),
        "the map is spent"
    );
    let deposit = objects_of(&rig, |e| e == IRON_DEPOSIT)[0];

    // Without the tool wielded nothing comes of it.
    rig.command(ac2_client::Command::Use(deposit));
    rig.run(1.5);
    assert!(objects_of(&rig, |e| e == IRON_MOTE).is_empty());

    // The starter pickaxe is wielded, as the player must, and the deposit worked.
    let pickaxe = objects_of(&rig, |e| e == PICKAXE)[0];
    rig.command(ac2_client::Command::EquipItem {
        item: pickaxe,
        location: PRIMARY_HAND,
    });
    wait(&mut rig, "the pickaxe wielded", |r| {
        r.client
            .frame()
            .inventory
            .equipped
            .iter()
            .any(|&(location, _)| location == PRIMARY_HAND)
    });
    rig.command(ac2_client::Command::Use(deposit));
    assert!(
        rig.until(15.0, |r| objects_of(r, |e| e == IRON_MOTE).len() == 1),
        "iron comes into the pack"
    );
    let mote = objects_of(&rig, |e| e == IRON_MOTE)[0];
    let frame = rig.client.frame();
    let stack = frame
        .inventory
        .items
        .iter()
        .find(|i| i.id == mote)
        .expect("the mote in the pack");
    assert_eq!(stack.stats.quantity, 10, "ten units a strike");
}

/// Crafting experience raises the rank: Surveying's rank 1 gives 6 a craft and rank 2 starts at
/// 42 (`crafting-levels.tsv`, the Pyreal dump), so the seventh craft shows rank 2 on the panel.
#[test]
fn crafting_raises_the_surveying_rank() {
    let Some(mut rig) = rig_for("Tradelevel") else {
        return;
    };
    open_recipe(&mut rig, SURVEYING, SURVEY_LINVAK_IRON);
    assert_eq!(rig.client.craft_skill_level(SURVEYING), Some((1, 1)));
    for _ in 0..6 {
        craft_once(&mut rig, SURVEY_LINVAK_IRON);
    }
    assert_eq!(rig.client.craft_skill_level(SURVEYING), Some((1, 1)));
    craft_once(&mut rig, SURVEY_LINVAK_IRON);
    wait(&mut rig, "rank 2 after 42 crafting experience", |r| {
        r.client.craft_skill_level(SURVEYING) == Some((2, 2))
    });
}
