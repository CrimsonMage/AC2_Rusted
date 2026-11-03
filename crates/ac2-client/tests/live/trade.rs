//! Helpers of the live tradeskills tests: the craft panel's views, the maps and the pack, as the
//! player works them (`pyrealium_live_tradeskills.rs`, `pyrealium_live_tradeskills_chain.rs`).

use super::LiveRig;
use ac2_net::protocol::ScriptMessage;
use ac2_ui::{tokenize, AttributeValue, ToolbarChild};
use ac2_weenie::{GameplayNotice, SessionEvent};

pub const SURVEYING: u32 = 0x6F00_264A;
pub const MINING_IRON: u32 = 0x6F00_2579;
pub const TOOLMAKING: u32 = 0x6F00_2857;
/// `rcp_surveying_linvak_iron`: makes one of the six Linvak iron surveyor's maps.
pub const SURVEY_LINVAK_IRON: u32 = 0x6F00_2AF0;
/// `wsl_surveyorsmap_linvak_iron1..6`.
pub const IRON_MAPS: std::ops::RangeInclusive<u32> = 0x4700_2E94..=0x4700_2E99;
/// `wsl_mine_iron_linvak`, the deposit those maps reveal.
pub const IRON_DEPOSIT: u32 = 0x4700_2EEF;
/// `wsl_toolnew_pickaxe_starter`, `wsl_mote_iron`.
pub const PICKAXE: u32 = 0x4700_2F75;
pub const IRON_MOTE: u32 = 0x4700_10F3;
pub const PACK: &str = "PackMainContainer";
pub const PRIMARY_HAND: u32 = 0x40;
/// The second press of a double click as the input layer sends it.
pub const DOUBLE_CLICK: u32 = 0x4000_0007;

pub fn rig_for(name: &str) -> Option<LiveRig> {
    if std::env::var_os("PYREALIUM_LIVE_TRAINING_KIT").is_none()
        || std::env::var_os("PYREALIUM_DEV_TRADESKILLS").is_none()
    {
        eprintln!("the server has no tradeskills profile (PYREALIUM_DEV_TRADESKILLS); skipped");
        return None;
    }
    LiveRig::new(name)
}

pub fn wait(rig: &mut LiveRig, what: &str, mut ready: impl FnMut(&mut LiveRig) -> bool) {
    for _ in 0..60 * 60 {
        if ready(rig) {
            return;
        }
        rig.frame(&[]);
    }
    panic!("{what}");
}

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

pub fn press(rig: &mut LiveRig, name: &str) {
    let mut center = None;
    wait(rig, name, |r| {
        center = r.view.ui_element_center(name);
        center.is_some()
    });
    rig.click(center.unwrap());
}

pub fn row_did(rig: &LiveRig, row: usize) -> u32 {
    let tree = rig.view.ui_tree().expect("the HUD");
    match tree.get(row).unwrap().attribute(tokenize("did")) {
        Some(AttributeValue::Id(did)) => *did,
        _ => 0,
    }
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

/// Waits for the player's craft registry and opens the craft tab of the skill panel.
pub fn open_craft_tab(rig: &mut LiveRig) {
    wait(rig, "the craft skills", |r| {
        r.client.craft_skills().is_some_and(|s| s.len() >= 28)
    });
    rig.panel(ToolbarChild::Skill, true);
    press(rig, "CraftTab");
    rig.run(0.5);
}

/// The ids of the world objects (the pack's items included) of entity `entity`.
pub fn objects_of(rig: &LiveRig, entity: impl Fn(u32) -> bool) -> Vec<u64> {
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
pub fn open_recipe(rig: &mut LiveRig, skill: u32, recipe: u32) {
    // An ingredient panel left open by an earlier craft keeps its recipe: close it first
    // (`gmIngredientUI::DoClose`), so the recipe asked for is the one it shows.
    if let Some(close) = rig.view.ui_element_center("IngredientCloseButton") {
        rig.click(close);
        rig.run(0.5);
    }
    open_craft_tab(rig);
    // The panel's rows fill once the recipe records the script reads are resident.
    // A skill with a few recipes known (the weapon skills list their starters) never reaches four.
    wait(rig, "the recipes", |r| {
        r.client.recipes(skill).is_some_and(|x| !x.is_empty())
    });
    for _ in 0..5 {
        if rig.client.recipes(skill).is_some_and(|x| x.len() > 3) {
            break;
        }
        rig.run(1.0);
    }
    rig.run(3.0);
    // The panel keeps the view it was left in (a recipe page, a category list): back to the skills.
    for _ in 0..4 {
        match rig.view.ui_element_center("BackButton") {
            Some(back) => {
                rig.click(back);
                rig.run(0.7);
            }
            None => break,
        }
    }
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
    // A row below the list's visible part cannot be clicked (the list shows about 17 of the 28
    // skills): the double click's own request (`gmCraftUI::ListenToMessage`) is sent instead.
    if !rig.until(4.0, |r| !rows(r, "CraftSkillCategoryList").is_empty()) {
        // The recipe's record is read when the player lists it; asked here, so it is resident
        // when the panel makes its slots.
        let mut has_slots = false;
        for _ in 0..10 {
            has_slots = rig
                .client
                .recipe_ingredients(recipe, 1)
                .is_some_and(|(_, slots)| !slots.is_empty());
            if has_slots {
                break;
            }
            rig.run(0.5);
        }
        rig.command(ac2_client::Command::OpenRecipe(recipe));
        wait(rig, "the ingredient panel", |r| {
            r.view.ui_element_center("RealIngredientField").is_some()
        });
        // The panel of the recipe asked for: its slots (one row for a recipe without ingredients).
        // Made before the record was resident it has none; closed and asked again it has them.
        rig.run(2.0);
        if has_slots && rows(rig, "IngredientListBox").len() <= 1 {
            if let Some(close) = rig.view.ui_element_center("IngredientCloseButton") {
                rig.click(close);
                rig.run(0.5);
            }
            rig.command(ac2_client::Command::OpenRecipe(recipe));
            wait(rig, "the ingredient panel again", |r| {
                r.view.ui_element_center("RealIngredientField").is_some()
            });
            rig.run(2.0);
        }
        return;
    }
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
        if listed.iter().any(|&r| row_did(rig, r) == recipe) {
            // The rows are rebuilt as records arrive: find the row again right before the click.
            rig.run(1.0);
            let r = rows(rig, "RecipeList")
                .into_iter()
                .find(|&r| row_did(rig, r) == recipe)
                .expect("the recipe row");
            double_click_row(rig, r);
            // A row below the list's visible part cannot be clicked: the double click's own
            // request (`gmCraftUI::ListenToMessage`, the recipe state's forward) is sent instead.
            if !rig.until(3.0, |r| {
                r.view.ui_element_center("RealIngredientField").is_some()
            }) {
                rig.command(ac2_client::Command::OpenRecipe(recipe));
            }
            wait(rig, "the ingredient panel", |r| {
                r.view.ui_element_center("RealIngredientField").is_some()
            });
            return;
        }
    }
    panic!("no category of skill {skill:08x} lists recipe {recipe:08x}");
}

/// Presses the craft button and waits for the craft to finish without an error.
pub fn craft_once(rig: &mut LiveRig, recipe: u32) {
    let mark = rig.seen.len();
    press(rig, "CraftButton");
    assert!(
        rig.until(15.0, |r| r.seen[mark..].iter().any(|e| matches!(
            e,
            SessionEvent::Notice(GameplayNotice::ExecuteCraftDone { recipe: done, error: 0 })
                if *done == recipe
        ))),
        "no ExecuteCraftDone for {recipe:08x}: {:?}; said {:?}",
        rig.client.take_script_problems(),
        lines(rig).iter().rev().take(4).collect::<Vec<_>>()
    );
    rig.run(0.5);
}

/// Uses a surveyor's map from the pack (a double click on its icon). The map works near the place
/// its text names ("begin your search near 14.9S, 40.0W"): where it is used elsewhere the server
/// repeats the text, and the player goes there (here by Pyrealium's `@maptravel`, the test server's
/// travel in land cells: `half the world * (1 + degrees / 100)`) and uses it again.
pub fn use_map(rig: &mut LiveRig, map: u64) {
    let double_click = |rig: &mut LiveRig| {
        let at = rig
            .view
            .item_icon_center(PACK, map)
            .expect("the map's icon");
        rig.point(at);
        rig.button(true);
        rig.button(false);
        rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
        rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
    };
    double_click(rig);
    let told = rig.until(5.0, |r| {
        lines(r).iter().any(|l| l.contains("not near the place"))
    });
    if !told {
        return;
    }
    let line = lines(rig)
        .into_iter()
        .rev()
        .find(|l| l.contains("not near the place"))
        .unwrap();
    // "... You should begin your search near 14.9S, 40.0W. Then use the map ..."
    let place = line.split("search near ").nth(1).expect("the place");
    let place = place.split(". ").next().unwrap();
    let (latitude, longitude) = place.split_once(", ").expect("latitude, longitude");
    let degrees = |text: &str| -> f64 {
        let (number, letter) = text.split_at(text.len() - 1);
        let number: f64 = number.parse().unwrap();
        if matches!(letter, "S" | "W") {
            -number
        } else {
            number
        }
    };
    let x = 1020.0 * (1.0 + degrees(longitude) / 100.0);
    let y = 1020.0 * (1.0 + degrees(latitude) / 100.0);
    rig.command(ac2_client::Command::Say(format!(
        "@maptravel {x:.1} {y:.1}"
    )));
    // The world changes under the character: wait for the new place, then use the map.
    rig.run(8.0);
    rig.panel(ToolbarChild::Inventory, true);
    rig.run(2.0);
    wait(rig, "the map's icon", |r| {
        r.view.item_icon_center(PACK, map).is_some()
    });
    rig.command(ac2_client::Command::Use(map));
}

/// Double-clicks the pack icon of `item` (the player's use of an item in the pack).
pub fn double_click_item(rig: &mut LiveRig, item: u64) {
    let at = rig
        .view
        .item_icon_center(PACK, item)
        .expect("the item's icon");
    rig.point(at);
    rig.button(true);
    rig.button(false);
    rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
    rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
}

/// The server's chat lines so far (its `DisplayStringInfo` literals).
pub fn lines(rig: &LiveRig) -> Vec<String> {
    rig.seen
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Interp {
                function,
                arguments,
                ..
            } => match ScriptMessage::decode(*function, arguments) {
                Some(Ok(ScriptMessage::DisplayStringInfo { text, .. })) => text.literal,
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// The raw-material traders (`wsl_trader_*`, their stores `store_*.wc`) with where each stands
/// (land cells, from the world table's cell and offset): Sabine of Cavendo sells flawless pyreal,
/// Wolfgang of Cragstone coal; both the starter tools.
pub const SABINE: (u32, [f64; 2]) = (0x4700_2F8E, [1314.44, 1182.63]);
pub const WOLFGANG: (u32, [f64; 2]) = (0x4700_2F8F, [883.82, 1347.63]);
pub const COAL: u32 = 0x4700_2AD4;
pub const PYREAL: u32 = 0x4700_10FA;

/// Every element called `name` below the HUD's roots.
pub fn elements(rig: &LiveRig, name: &str) -> Vec<usize> {
    let Some(tree) = rig.view.ui_tree() else {
        return Vec::new();
    };
    let id = tokenize(name);
    let mut found = Vec::new();
    let mut stack: Vec<usize> = tree.roots().to_vec();
    while let Some(index) = stack.pop() {
        if tree.get(index).is_some_and(|e| e.id() == id) {
            found.push(index);
        }
        stack.extend_from_slice(tree.children(index));
    }
    found.sort_unstable();
    found
}

/// The name a store row shows (the sale's item name text).
pub fn sale_name(rig: &LiveRig, row: usize) -> Option<String> {
    let tree = rig.view.ui_tree()?;
    let name = tree.child_recursive(row, tokenize("ProductDesc"))?;
    tree.get(name)?.text()
}

/// The names the open store lists, in its order.
pub fn sale_names(rig: &LiveRig) -> Vec<String> {
    elements(rig, "Sale")
        .into_iter()
        .filter_map(|row| sale_name(rig, row))
        .collect()
}

/// Buys `tens` x 10 units of the sale `name` from the open store, as the player does: the row is
/// selected, the purchase button asks how many (the entry opens on 0 with the cursor in front, so
/// one typed digit makes a number of tens), OK, and the confirmation's yes.
pub fn buy_tens(rig: &mut LiveRig, name: &str, tens: u8) {
    let row = elements(rig, "Sale")
        .into_iter()
        .find(|&r| sale_name(rig, r).as_deref() == Some(name))
        .unwrap_or_else(|| panic!("the store sells no {name}"));
    let at = rig.view.ui_index_center(row).expect("the sale's row");
    rig.click(at);
    press(rig, "PurchaseButton");
    assert!(
        rig.until(10.0, |r| r.view.ui_element_center("AmountEntry").is_some()),
        "the quantity dialog for {name}; confirm {:?}; sales {:?}; said {:?}",
        rig.view.ui_element_center("YesButton"),
        sale_names(rig),
        lines(rig).iter().rev().take(3).collect::<Vec<_>>()
    );
    let entry = rig.view.ui_element_center("AmountEntry").unwrap();
    rig.click(entry);
    let typed = [(u16::from(b'0') + u16::from(tens), 0)];
    rig.view
        .text_input(&[], &typed, ac2_ui::Modifiers::default());
    rig.run(0.3);
    assert_eq!(
        rig.view.element_text("AmountEntry"),
        Some(format!("{}0", tens)),
        "the amount typed"
    );
    press(rig, "StackOkButton");
    wait(rig, "the purchase confirmation", |r| {
        r.view.ui_element_center("YesButton").is_some()
    });
    press(rig, "YesButton");
    rig.run(1.0);
}

/// Travels (the test server's `@maptravel`) to a place ten metres east of the trader, walks to it
/// (the body walks west) and uses it: the store panel opens.
pub fn open_trader(rig: &mut LiveRig, trader: (u32, [f64; 2])) {
    let (entity, [x, y]) = trader;
    // A store left open stays with the trader: leave it (`ShopCloseButton`); the server refuses to
    // travel within five seconds of a world use (the last trader used).
    if let Some(close) = rig.view.ui_element_center("ShopCloseButton") {
        rig.click(close);
    }
    // The craft panels would cover the store's rows: close them, as the player does.
    if let Some(close) = rig.view.ui_element_center("IngredientCloseButton") {
        rig.click(close);
    }
    rig.panel(ToolbarChild::Skill, false);
    rig.panel(ToolbarChild::Inventory, false);
    rig.run(6.0);
    rig.command(ac2_client::Command::Say(format!(
        "@maptravel {:.2} {:.2}",
        x + 0.1,
        y
    )));
    rig.run(8.0);
    assert!(
        rig.until(30.0, |r| r.object_of(entity).is_some()),
        "the trader in view; said {:?}",
        lines(rig).iter().rev().take(3).collect::<Vec<_>>()
    );
    let id = rig.object_of(entity).unwrap();
    assert!(
        rig.until(30.0, |r| r.place_of(id).is_some()),
        "the trader in the world; said {:?}; at {:?}",
        lines(rig).iter().rev().take(3).collect::<Vec<_>>(),
        rig.origin()
    );
    let at = rig.place_of(id).unwrap();
    rig.walk_to(at, 3.0, 12.0);
    rig.run(3.0);
    rig.command(ac2_client::Command::Use(id));
    if !rig.until(6.0, |r| !elements(r, "Sale").is_empty()) {
        // The server may still hold the position before the walk: use it again.
        rig.run(3.0);
        rig.command(ac2_client::Command::Use(id));
    }
    assert!(
        rig.until(10.0, |r| !elements(r, "Sale").is_empty()),
        "the store panel; at {:?} for {:?}; said {:?}",
        rig.origin(),
        at,
        lines(rig).iter().rev().take(4).collect::<Vec<_>>()
    );
}

/// The chat lines (the server's `DisplayStringInfo` and `DisplayText` literals) since `from`.
pub fn said_since(rig: &LiveRig, from: usize) -> Vec<String> {
    rig.seen[from..]
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::DisplayStringInfo { text, .. }) => {
                text.literal.clone()
            }
            SessionEvent::Interp {
                function,
                arguments,
                ..
            } => match ac2_net::protocol::ScriptMessage::decode(*function, arguments) {
                Some(Ok(ac2_net::protocol::ScriptMessage::DisplayStringInfo { text, .. })) => {
                    text.literal
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}
