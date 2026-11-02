//! Tradeskills against a running AC2e server (see `tests/live/mod.rs`): the craft panel lists
//! Surveying, Mining and Toolmaking beside the quest recipes, a Surveying recipe makes a
//! surveyor's map, the map reveals a deposit, a mining tool works it for motes, and crafting
//! experience raises the skill's rank on the panel.
//! Needs AC2e's tradeskills profile (`AC2E_DEV_TRADESKILLS`, with `--training-kit true`);
//! `scripts/ac2e-live-test.sh` with `AC2E_LIVE_TEST=ac2e_live_tradeskills` sets that up.

#![cfg(feature = "window")]

mod live;

use ac2_ui::ToolbarChild;
use live::trade::*;
use live::LiveRig;

/// Every craft skill the 7230 DAT defines (`craftskilltype.emp`: types 2..24, the quest skill 25
/// and the 7230 additions 26..29, one package 0x33af state each) with the number of its recipes
/// whose minimum level a new character (rank 1) already has.
const CRAFT_SKILLS: [(u32, usize); 27] = [
    (0x6F00_23A8, 0), // Armor: Human
    (0x6F00_23AA, 1), // Armor: Lugian
    (0x6F00_23A9, 0), // Armor: Tumerok
    (0x6F00_2565, 2), // Weapons: Human Martial
    (0x6F00_256D, 1), // Weapons: Human Exotic
    (0x6F00_256E, 1), // Weapons: Lugian Exotic
    (0x6F00_256F, 2), // Weapons: Lugian Martial
    (0x6F00_2570, 1), // Weapons: Tumerok Exotic
    (0x6F00_2571, 2), // Weapons: Tumerok Martial
    (0x6F00_2560, 0), // Butchery (no recipes)
    (0x6F00_2579, 1), // Mining: Iron
    (0x6F00_25A8, 1), // Mining: Crystal
    (0x6F00_25A6, 1), // Mining: Silver
    (0x6F00_25A7, 1), // Mining: Stone
    (0x6F00_25A9, 1), // Mining: Wood
    (SURVEYING, 78),
    (0x6F00_282D, 0), // Spellbinding
    (0x6F00_283C, 0), // Refining: Dramastic (its seven recipes want rank 2)
    (0x6F00_283D, 0),
    (0x6F00_283E, 0),
    (0x6F00_283F, 0),
    (0x6F00_2840, 0),
    (TOOLMAKING, 6),
    (0x6F00_5242, 0), // Armor: Empyrean
    (0x6F00_5276, 2), // Weapons: Empyrean Exotic
    (0x6F00_5BB4, 0), // Armor: Drudge
    (0x6F00_5BB5, 0), // Weapons: Drudge Martial
];

/// The panel lists the quest recipes and every craft skill of the DAT, each at rank 1 with the
/// recipes whose own minimum level it reaches (the registry's hash tables are sized for them, or
/// the client's lookups would miss entries).
#[test]
fn the_craft_panel_lists_every_craft_skill_of_the_dat() {
    let Some(mut rig) = rig_for("Tradelist") else {
        return;
    };
    open_craft_tab(&mut rig);
    // The panel fills as the skill records the script reads arrive.
    wait(&mut rig, "28 skill rows", |r| {
        rows(r, "CraftSkillList").len() >= 28
    });
    let listed: Vec<u32> = rows(&rig, "CraftSkillList")
        .iter()
        .map(|&r| row_did(&rig, r))
        .collect();
    assert_eq!(
        listed.len(),
        28,
        "the quest recipes and 27 craft skills: {listed:x?}"
    );
    for (skill, level_zero) in CRAFT_SKILLS {
        assert!(listed.contains(&skill), "{skill:08x} in {listed:x?}");
        assert_eq!(
            rig.client.craft_skill_level(skill),
            Some((1, 1)),
            "{skill:08x}"
        );
        wait(&mut rig, "the skill's level-0 recipes", |r| {
            r.client
                .recipes(skill)
                .is_some_and(|x| x.len() == level_zero)
        });
    }
    // The rank shows on the panel: Surveying starts at 1, its raise costs the DAT curve's 42.
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
        rig.until(10.0, |r| objects_of(r, |e| IRON_MAPS.contains(&e)).len()
            == 1),
        "one Linvak iron surveyor's map in the pack"
    );
    let map = objects_of(&rig, |e| IRON_MAPS.contains(&e))[0];

    // The map is used from the pack (a double click on its icon): the deposit appears.
    assert!(objects_of(&rig, |e| e == IRON_DEPOSIT).is_empty());
    wait(&mut rig, "the map's icon", |r| {
        r.view.item_icon_center(PACK, map).is_some()
    });
    use_map(&mut rig, map);
    assert!(
        rig.until(15.0, |r| objects_of(r, |e| e == IRON_DEPOSIT).len() == 1),
        "the map reveals a vein of iron: {:?}",
        lines(&rig)
            .iter()
            .filter(|l| !l.contains("reature"))
            .rev()
            .take(4)
            .map(|l| l.chars().take(220).collect::<String>())
            .collect::<Vec<_>>()
    );
    assert!(
        rig.until(5.0, |r| objects_of(r, |e| IRON_MAPS.contains(&e))
            .is_empty()),
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

/// The newer resource family: `rcp_surveying_omishan_gypsum` makes a gypsum map of Omishan, which
/// reveals `wsl_mine_gypsum_omishan`; the deposit names Mining Silver, so the mattock extracts gypsum motes.
/// (The tier recipes of the older family are copies of one another in the 7230 DAT, and the tier
/// deposits `wsl_mine_*_tier1..3` they name are not in the 7230 DAT, so there is no tier to wire.)
/// The deposit is then worked like the others, and the server's extraction line is retail's.
#[test]
fn the_newer_gypsum_family_is_surveyed_and_mined() {
    const GYPSUM_RECIPE: u32 = 0x6F00_369A;
    const GYPSUM_MAPS: std::ops::RangeInclusive<u32> = 0x4700_35FC..=0x4700_3601;
    const GYPSUM_DEPOSIT: u32 = 0x4700_3650;
    const GYPSUM_MOTE: u32 = 0x4700_3648;
    const ROCK_HAMMER: u32 = 0x4700_2F74; // the starter mattock: gypsum deposits name Mining Silver
    let Some(mut rig) = rig_for("Tradegypsum") else {
        return;
    };
    // The recipe sits deep in the third category; the double click on its row is its
    // `OpenRecipe` request, sent here directly to keep the run short (the idols at the spawn
    // kill an idle character after about half a minute).
    open_craft_tab(&mut rig);
    wait(&mut rig, "the recipes", |r| {
        r.client.recipes(SURVEYING).is_some_and(|x| x.len() > 70)
    });
    rig.command(ac2_client::Command::OpenRecipe(GYPSUM_RECIPE));
    wait(&mut rig, "the ingredient panel", |r| {
        r.view.ui_element_center("RealIngredientField").is_some()
    });
    craft_once(&mut rig, GYPSUM_RECIPE);
    rig.panel(ToolbarChild::Skill, false);
    rig.panel(ToolbarChild::Inventory, true);
    assert!(
        rig.until(10.0, |r| objects_of(r, |e| GYPSUM_MAPS.contains(&e)).len()
            == 1),
        "a gypsum map in the pack"
    );
    let map = objects_of(&rig, |e| GYPSUM_MAPS.contains(&e))[0];
    wait(&mut rig, "the map's icon", |r| {
        r.view.item_icon_center(PACK, map).is_some()
    });
    use_map(&mut rig, map);
    assert!(
        rig.until(15.0, |r| objects_of(r, |e| e == GYPSUM_DEPOSIT).len() == 1),
        "the gypsum map reveals a gypsum deposit"
    );
    let deposit = objects_of(&rig, |e| e == GYPSUM_DEPOSIT)[0];
    let pickaxe = objects_of(&rig, |e| e == ROCK_HAMMER)[0];
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
        rig.until(15.0, |r| objects_of(r, |e| e == GYPSUM_MOTE).len() == 1),
        "gypsum comes into the pack: {:?}",
        lines(&rig)
            .iter()
            .filter(|l| !l.contains("reature"))
            .rev()
            .take(4)
            .collect::<Vec<_>>()
    );
    // Retail's extraction line (craft string table 25007ace: "You extract  units of iron ore from
    // ...") with the units filled in.
    assert!(
        lines(&rig)
            .iter()
            .any(|l| l.starts_with("You extract 10 units of gypsum from")),
        "{:?}",
        lines(&rig)
    );
}

/// Crafting experience raises the rank: Surveying's rank 1 gives 6 a craft and rank 2 starts at
/// 42 (`crafting-levels.tsv`, the Pyreal dump), so the seventh craft shows rank 2 on the panel;
/// the rank and its experience survive a relog (AC2e's `tradeskills.tsv` beside the journal).
#[test]
fn crafting_raises_the_surveying_rank_and_it_survives_a_relog() {
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
    assert!(
        lines(&rig)
            .iter()
            .any(|l| l.contains("You gain 6 crafting experience")),
        "{:?}",
        lines(&rig)
    );

    // `RequestRaiseCraftSkill` (function 0x8206002e, one integer: the skill's DID) is served:
    // the rank already is what the experience earns, and the server says so.
    rig.command(ac2_client::Command::RaiseCraftSkill(SURVEYING));
    wait(&mut rig, "the raise answered", |r| {
        lines(r).iter().any(|l| l.contains("already at the rank"))
    });

    rig.relog();
    open_craft_tab(&mut rig);
    wait(&mut rig, "the saved rank", |r| {
        r.client.craft_skill_level(SURVEYING) == Some((2, 2))
    });
}

/// Toolmaking levels with use and its rank unlocks recipes: its six level-0 recipes make the
/// starter tools without ingredients; rank 2 (40 experience at 6 a craft: the seventh craft)
/// adds the level-2 recipe (`6f0036ad`) to the panel.
#[test]
fn toolmaking_rank_two_unlocks_its_next_recipe() {
    let Some(mut rig) = rig_for("Tradetool") else {
        return;
    };
    /// `rcp_toolnew_pickaxe_starter`.
    const STARTER_PICKAXE_RECIPE: u32 = 0x6F00_2F29;
    /// The Toolmaking recipe whose minimum level is 2.
    const LEVEL_TWO_RECIPE: u32 = 0x6F00_36AD;
    open_recipe(&mut rig, TOOLMAKING, STARTER_PICKAXE_RECIPE);
    let known = |rig: &mut LiveRig| -> Vec<u32> {
        rig.client
            .recipes(TOOLMAKING)
            .unwrap_or_default()
            .iter()
            .map(|r| r.did)
            .collect()
    };
    wait(&mut rig, "the six starter recipes", |r| known(r).len() == 6);
    assert!(!known(&mut rig).contains(&LEVEL_TWO_RECIPE));
    for _ in 0..7 {
        craft_once(&mut rig, STARTER_PICKAXE_RECIPE);
    }
    wait(&mut rig, "rank 2", |r| {
        r.client.craft_skill_level(TOOLMAKING) == Some((2, 2))
    });
    wait(&mut rig, "the level-2 recipe", |r| {
        known(r).contains(&LEVEL_TWO_RECIPE)
    });
    // The seven pickaxes the crafts made are in the pack beside the kit's own.
    assert_eq!(objects_of(&rig, |e| e == PICKAXE).len(), 8);
}

/// A use the server refuses can be made again by the same double click. The map used far from
/// its place is refused ("You are not near the place the map marks"); its slot is the last of the
/// pack's grid, where the modern item ring (which fans out of the selected slot) used to put a
/// button over the slot, so the second press hit the button and no use was sent.
#[test]
fn a_refused_use_is_sent_again_by_the_same_double_click() {
    let Some(mut rig) = rig_for("Tradeagain") else {
        return;
    };
    open_recipe(&mut rig, SURVEYING, SURVEY_LINVAK_IRON);
    craft_once(&mut rig, SURVEY_LINVAK_IRON);
    rig.panel(ToolbarChild::Skill, false);
    rig.panel(ToolbarChild::Inventory, true);
    assert!(
        rig.until(10.0, |r| objects_of(r, |e| IRON_MAPS.contains(&e)).len()
            == 1)
    );
    let map = objects_of(&rig, |e| IRON_MAPS.contains(&e))[0];
    wait(&mut rig, "the map's icon", |r| {
        r.view.item_icon_center(PACK, map).is_some()
    });
    let refusals = |rig: &LiveRig| {
        lines(rig)
            .iter()
            .filter(|l| l.contains("not near the place"))
            .count()
    };
    let uses = |rig: &LiveRig| {
        rig.asked
            .iter()
            .filter(|c| matches!(c, ac2_client::Command::UseSelection))
            .count()
    };
    for n in 0..3 {
        let (before, told) = (uses(&rig), refusals(&rig));
        let at = rig
            .view
            .item_icon_center(PACK, map)
            .expect("the map's icon");
        rig.point(at);
        rig.button(true);
        rig.button(false);
        rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
        rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
        assert!(
            rig.until(5.0, |r| refusals(r) == told + 1),
            "double click {n}: the server never refused the use again (uses sent {before} -> {})",
            uses(&rig)
        );
        assert_eq!(uses(&rig), before + 1, "double click {n}");
    }
}

/// A weapon crafted at the panel by a new character: `rcp_humanmartial_swordstarter` (Weapons:
/// Human Martial, minimum level 0, no ingredient slots) makes `wsl_swordhumanstarter`. The item
/// comes into the pack with its own name and icon. The other weapon and armour recipes want
/// motes, refined metals, coal, pyreal and butchered motes (hide, bone, sinew, chitin), and the
/// armour recipes' minimum level is 5 (see the report in AGENTS.md).
#[test]
fn a_new_character_crafts_the_human_martial_starter_sword() {
    let Some(mut rig) = rig_for("Tradesword") else {
        return;
    };
    const HUMAN_MARTIAL: u32 = 0x6F00_2565;
    const STARTER_SWORD_RECIPE: u32 = 0x6F00_2B43;
    const STARTER_SWORD: u32 = 0x4700_07EE;
    open_recipe(&mut rig, HUMAN_MARTIAL, STARTER_SWORD_RECIPE);
    // The development kit already gives one starter sword: the craft makes a second.
    let have = objects_of(&rig, |e| e == STARTER_SWORD);
    craft_once(&mut rig, STARTER_SWORD_RECIPE);
    rig.panel(ToolbarChild::Skill, false);
    rig.panel(ToolbarChild::Inventory, true);
    assert!(
        rig.until(10.0, |r| objects_of(r, |e| e == STARTER_SWORD).len()
            == have.len() + 1),
        "the crafted sword in the pack"
    );
    let sword = *objects_of(&rig, |e| e == STARTER_SWORD)
        .iter()
        .find(|id| !have.contains(id))
        .unwrap();
    wait(&mut rig, "the sword's icon", |r| {
        r.view.item_icon_center(PACK, sword).is_some()
    });
    let name = rig
        .view
        .backpack_items()
        .into_iter()
        .find(|(id, _)| *id == sword)
        .map(|(_, n)| n);
    // The server names the item in its description (AC2e `item_names`): the DAT's name for the
    // entity, not the "Error! Name_StringInfoStat missing" a nameless pack item shows.
    assert_eq!(
        name.as_deref(),
        Some("Rusty Sword"),
        "the crafted sword's name"
    );
}
