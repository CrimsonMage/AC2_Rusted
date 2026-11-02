//! Quests against a running AC2e server (`AC2E_LIVE`, see `tests/live/mod.rs`): the training
//! quest chain AC2e's new characters meet in the Menhir Vault (`quests.rs`: Training Grounds,
//! Field Supplies, Potion Practice, A Safe Journey), played as a player does and read from the
//! quest journal (`gmQuestsUI` / `gmJournalQuestsUI`, the toolbar's Quest panel) and the chat.
//!
//! Run with `AC2E_LIVE_TEST=ac2e_live_quests scripts/ac2e-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_ui::ToolbarChild;
use ac2_weenie::{GameplayNotice, SessionEvent};
use live::LiveRig;

/// `training_dialogue::MENTOR_ENTITY`: Pectar, the training mentor next to the new character.
const MENTOR: u32 = 0x4700_1699;
/// `quests::TRAINING_QUEST`, `SUPPLIES_QUEST`.
const TRAINING: u32 = 0x4000_ff01;
const SUPPLIES: u32 = 0x4000_ff02;
/// The player's gold quality (`Proto` int 0xC1C).
const GOLD: u32 = 0xC1C;

fn mentor(rig: &LiveRig) -> Option<u64> {
    rig.client
        .session()
        .objects()
        .iter()
        .find(|o| o.entity == Some(MENTOR))
        .map(|o| o.id)
}

/// The server's chat lines since event `from`: its notices' literals, what the chat window shows.
fn said_lines(rig: &LiveRig, from: usize) -> Vec<String> {
    rig.seen[from..]
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::DisplayStringInfo { text, .. })
            | SessionEvent::Notice(GameplayNotice::DisplayMessage { text, .. }) => {
                text.literal.clone()
            }
            _ => None,
        })
        .collect()
}

fn said(rig: &LiveRig, from: usize, part: &str) -> bool {
    said_lines(rig, from).iter().any(|l| l.contains(part))
}

/// The Underway and the Finished pages' texts, each joined.
fn journal(rig: &LiveRig) -> (String, String) {
    let (underway, finished) = rig.view.quest_journal_texts();
    (underway.join(" | "), finished.join(" | "))
}

/// The player's gold and total experience as the client shows them.
fn wealth(rig: &LiveRig) -> (i32, u64) {
    let player = rig.client.session().player().expect("the player");
    let gold = rig
        .client
        .session()
        .objects()
        .get(player)
        .and_then(|o| o.qualities.inq_int(GOLD))
        .unwrap_or(0);
    (gold, rig.client.frame().progress.experience.total_xp)
}

/// A character in the world with the Quest panel open and the mentor in view.
fn rig(name: &str) -> Option<LiveRig> {
    let mut rig = LiveRig::new(name)?;
    rig.wait("the mentor", |r| mentor(r).is_some());
    rig.panel(ToolbarChild::Quest, true);
    Some(rig)
}

/// The nearest monster that is alive and not fought yet.
fn next_monster(rig: &LiveRig, fought: &[u64]) -> Option<u64> {
    let frame = rig.client.frame();
    let player = frame.player.as_ref()?;
    frame
        .objects
        .iter()
        .filter(|o| o.id != player.id && !fought.contains(&o.id))
        .filter(|o| {
            rig.client.session().objects().get(o.id).is_some_and(|x| {
                x.qualities.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F)
                    && x.qualities.inq_int(0x102).is_none_or(|h| h > 0)
            })
        })
        .map(|o| {
            let d: f32 = (0..3)
                .map(|i| (o.origin[i] - player.origin[i]).powi(2))
                .sum();
            (o.id, d)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Kills the next creature that can be attacked: select it, start the attack, wait for the
/// defeat message.
fn kill_a_creature(rig: &mut LiveRig, fought: &mut Vec<u64>) {
    for _ in 0..6 {
        assert!(
            rig.until(30.0, |r| next_monster(r, fought).is_some()),
            "no creature to fight; fought {fought:x?}"
        );
        let target = next_monster(rig, fought).unwrap();
        fought.push(target);
        let before = rig.seen.len();
        rig.command(Command::Select(Some(target)));
        rig.run(0.5);
        rig.command(Command::SetAttacking(true));
        rig.until(40.0, |r| {
            said(r, before, "Creature defeated") || said(r, before, "not an attackable")
        });
        let dead = said(rig, before, "Creature defeated");
        let lines = said_lines(rig, before);
        eprintln!(
            "ATTACK {target:x} dead {dead}: {:?}",
            &lines[lines.len().saturating_sub(3)..]
        );
        rig.command(Command::SetAttacking(false));
        rig.run(0.5);
        if dead {
            return;
        }
    }
    panic!("no creature could be attacked");
}

/// Speaks to the mentor (a use) and returns the index of the first event it caused.
fn speak(rig: &mut LiveRig) -> usize {
    let from = rig.seen.len();
    let m = mentor(rig).expect("the mentor");
    rig.command(Command::Use(m));
    rig.run(3.0);
    // The mentor's words come as message boxes too, which the player closes (`MessageField`'s
    // OK button) before anything else takes a press.
    for _ in 0..8 {
        let Some(ok) = rig.view.ui_element_center("OkButton") else {
            break;
        };
        rig.click(ok);
        rig.run(0.3);
    }
    from
}

/// A new character's journal lists Training Grounds underway: its title, "no time limit", and
/// the expanded entry with the description, the step (the kill count) and the intended level
/// (`gmJournalQuestsUI::AddQuestElement`, `InqExtraText`).
#[test]
fn the_training_quest_is_in_the_journal() {
    let Some(mut rig) = rig("Questlisted") else {
        return;
    };
    rig.wait("the training quest", |r| {
        r.view
            .quest_ids()
            .is_some_and(|(underway, _)| underway == [TRAINING])
    });
    rig.run(1.0);
    // The row shows the title and the time; a press opens it (`QuestTemplate` released).
    let (underway, _) = journal(&rig);
    assert!(underway.contains("Training Grounds"), "{underway}");
    assert!(!underway.contains("Defeat three"), "{underway}");
    let row = rig.view.quest_row_centers().0[0];
    rig.click(row);
    let (underway, finished) = journal(&rig);
    assert!(underway.contains("Defeat three Idol Spawn"), "{underway}");
    assert!(
        underway.contains("Training creatures defeated: 0/3"),
        "{underway}"
    );
    assert!(underway.contains("Intended Level: 1"), "{underway}");
    assert_eq!(finished, "");
}

/// The training chain, played once (the server's three reachable training creatures have one
/// lifetime each, so the kills cannot be shared by several tests):
/// - a kill while the Quest panel is closed still reaches the journal; opening the panel shows
///   the changed quest already opened (`gmJournalQuestsUI::Update` expands `m_updatedQuestID`);
/// - the next kills count live, and the kills alone pay nothing and finish nothing;
/// - the mentor's turn-in pays 120 experience and 10 gold and moves the quest to Finished;
/// - the mentor then bestows Field Supplies, and hands it in (the pack's mushroom and tallow for
///   a potion, 80 experience and 6 gold), the quest joining the Finished page;
/// - a relog keeps the journal, the rewards and the pack.
#[test]
fn the_training_chain_is_played_through_and_survives_a_relog() {
    let Some(mut rig) = LiveRig::new("Questchain") else {
        return;
    };
    rig.wait("the mentor", |r| mentor(r).is_some());
    rig.wait("the training quest", |r| {
        r.view
            .quest_ids()
            .is_some_and(|(underway, _)| underway == [TRAINING])
    });
    let mut fought = Vec::new();
    for kills in 1..=3 {
        kill_a_creature(&mut rig, &mut fought);
        if kills == 1 {
            rig.panel(ToolbarChild::Quest, true);
        }
        let want = format!("Training creatures defeated: {kills}/3");
        let shown = rig.until(10.0, |r| journal(r).0.contains(&want));
        assert!(shown, "after kill {kills}: {:?}", journal(&rig));
    }
    assert_eq!(rig.view.quest_ids().unwrap().1, Vec::<u32>::new());

    // Turn in: the mentor speaks, pays, and the quest is Finished.
    let (gold0, xp0) = wealth(&rig);
    let from = speak(&mut rig);
    let lines = said_lines(&rig, from);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("Pectar, Training Master:")),
        "{lines:?}"
    );
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(_, f)| f == [TRAINING])),
        "{:?} {:?}",
        rig.view.quest_ids(),
        lines
    );
    let (gold1, xp1) = wealth(&rig);
    assert_eq!((gold1 - gold0, xp1 - xp0), (10, 120));
    assert!(journal(&rig).1.contains("Training Grounds"));

    // The next lesson comes from the mentor: Field Supplies is bestowed.
    let from = speak(&mut rig);
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(u, _)| u == [SUPPLIES])),
        "{:?} {:?}",
        rig.view.quest_ids(),
        said_lines(&rig, from)
    );
    assert!(journal(&rig).0.contains("Field Supplies"));

    // Handing in the supplies.
    rig.panel(ToolbarChild::Inventory, true);
    let pack_before = rig.view.item_icons("PackMainContainer").len();
    let (gold0, xp0) = wealth(&rig);
    let from = speak(&mut rig);
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(u, f)| u.is_empty() && f.len() == 2)),
        "{:?} {:?}",
        rig.view.quest_ids(),
        said_lines(&rig, from)
    );
    let (gold1, xp1) = wealth(&rig);
    assert_eq!((gold1 - gold0, xp1 - xp0), (6, 80));
    // The mushroom and the tallow left the pack and a potion came: one fewer.
    assert!(
        rig.until(5.0, |r| r.view.item_icons("PackMainContainer").len() + 1
            == pack_before),
        "{} -> {}",
        pack_before,
        rig.view.item_icons("PackMainContainer").len()
    );
    assert!(journal(&rig).1.contains("Field Supplies"));

    // A relog keeps the journal, the rewards and the pack.
    let before = (rig.view.quest_ids(), wealth(&rig));
    rig.relog();
    rig.panel(ToolbarChild::Quest, true);
    rig.wait("the journal after the relog", |r| {
        r.view
            .quest_ids()
            .is_some_and(|(u, f)| u.is_empty() && f.len() == 2)
    });
    assert_eq!(rig.view.quest_ids(), before.0);
    assert_eq!(wealth(&rig), before.1);
    let finished = journal(&rig).1;
    assert!(finished.contains("Training Grounds") && finished.contains("Field Supplies"));
}
/// The mentor's placement, from the frame's body list.
fn mentor_at(rig: &LiveRig) -> Option<[f32; 3]> {
    let id = mentor(rig)?;
    rig.client
        .frame()
        .objects
        .iter()
        .find(|o| o.id == id)
        .map(|o| o.origin)
}

/// Pectar hovers 2 m over the vault floor and stays there. His visual's motion interpreter
/// description (0x3200000F) is a flier's: no gravity, a hover height of 2 m in the air element
/// (`CMotionInterp::UpdateMovement`'s hover vertical acceleration). The height probe
/// (`CPhysicsObj::GetHeightAboveWalkable`) started inside his own body because remote bodies had
/// no `m_id` for its ethereal list, read a height of 0 and sent him climbing at 3 m/s for ever.
#[test]
fn the_mentor_hovers_over_the_floor_and_stays() {
    let Some(mut rig) = rig("Floorwatch") else {
        return;
    };
    // The hover settles slowly (the air element's lift and drag are both 0.1), so he is watched
    // once settled: 2 m over the cell's floor (z -44.887), and staying there.
    const HOVER_Z: f32 = -44.887 + 2.0;
    let settled = rig.until(30.0, |r| {
        mentor_at(r).is_some_and(|p| (p[2] - HOVER_Z).abs() < 0.15)
    });
    let first = mentor_at(&rig).expect("the mentor in the frame");
    assert!(
        settled,
        "the mentor hovers 2 m over the vault floor: {first:?}"
    );
    for _ in 0..5 {
        rig.run(2.0);
        let now = mentor_at(&rig).expect("the mentor in the frame");
        assert!(
            (now[2] - HOVER_Z).abs() < 0.3,
            "the mentor left his hover height: {first:?} then {now:?}"
        );
    }
}

/// `quests::POTION_QUEST`, `JOURNEY_QUEST`.
const POTION: u32 = 0x4000_ff03;
const JOURNEY: u32 = 0x4000_ff04;
/// The training quartermaster, the lifestone and the portal of the start room
/// (`training-world.tsv`).
const QUARTERMASTER: u32 = 0x4700_3724;
const LIFESTONE: u32 = 0x4700_07DE;
const PORTAL: u32 = 0x4700_084D;
/// The start room's cell, left by the portal.
const START_CELL: u32 = 0x0299_0109;

/// The training chain up to the point the mentor offers Potion Practice: three kills, the
/// turn-in, and Field Supplies bestowed and handed in.
fn play_to_supplies(rig: &mut LiveRig) {
    rig.wait("the mentor", |r| mentor(r).is_some());
    let mut fought = Vec::new();
    for _ in 0..3 {
        kill_a_creature(rig, &mut fought);
    }
    type Done = fn(&LiveRig) -> bool;
    let steps: [(&str, Done); 3] = [
        ("Training Grounds finished", |r| {
            r.view.quest_ids().is_some_and(|(_, f)| f == [TRAINING])
        }),
        ("Field Supplies underway", |r| {
            r.view.quest_ids().is_some_and(|(u, _)| u == [SUPPLIES])
        }),
        ("Field Supplies finished", |r| {
            r.view
                .quest_ids()
                .is_some_and(|(u, f)| u.is_empty() && f.len() == 2)
        }),
    ];
    for (step, done) in steps {
        let from = speak(rig);
        assert!(
            rig.until(10.0, done),
            "{step}: {:?} {:?}",
            rig.view.quest_ids(),
            said_lines(rig, from)
        );
    }
}

/// Every element called `name` below the HUD's roots.
fn elements(rig: &LiveRig, name: &str) -> Vec<usize> {
    let Some(tree) = rig.view.ui_tree() else {
        return Vec::new();
    };
    let id = ac2_ui::tokenize(name);
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
fn sale_name(rig: &LiveRig, row: usize) -> Option<String> {
    let tree = rig.view.ui_tree()?;
    let name = tree.child_recursive(row, ac2_ui::tokenize("ProductDesc"))?;
    tree.get(name)?.text()
}

/// The store row selling `name`.
fn sale_row(rig: &LiveRig, name: &str) -> Option<usize> {
    elements(rig, "Sale")
        .into_iter()
        .find(|&row| sale_name(rig, row).as_deref() == Some(name))
}

/// The player's pack items.
fn pack(rig: &LiveRig) -> usize {
    rig.view.item_icons(live::craft::PACK).len()
}

/// Buys the sale `name` through the store panel: the row is selected, the purchase button
/// pressed, and the confirmation answered yes.
fn buy(rig: &mut LiveRig, name: &str) {
    let row = sale_row(rig, name).unwrap_or_else(|| panic!("the store sells no {name}"));
    let at = rig.view.ui_index_center(row).expect("the sale's row");
    rig.click(at);
    live::craft::press(rig, "PurchaseButton");
    assert!(
        rig.until(5.0, |r| r.view.ui_element_center("YesButton").is_some()),
        "{name}: no purchase confirmation; the server said {:?}",
        said_lines(rig, 0).last()
    );
    live::craft::press(rig, "YesButton");
}

/// Potion Practice as the player plays it: the mentor offers it, the ingredients are bought from
/// the quartermaster through the store panel, the potion is crafted on the craft panel, and the
/// mentor pays. Returns after the mentor's reward. `check` asserts each step.
fn play_potion_practice(rig: &mut LiveRig, check: bool) {
    let from = speak(rig);
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(u, _)| u == [POTION])),
        "Potion Practice was not offered: {:?} {:?}",
        rig.view.quest_ids(),
        said_lines(rig, from)
    );
    if check {
        assert!(
            journal(rig).0.contains("Potion Practice"),
            "{:?}",
            journal(rig)
        );
    }

    // The quartermaster: walk into trade range and use it; the store panel opens.
    let quartermaster = rig.object_of(QUARTERMASTER).expect("the quartermaster");
    let at = rig
        .place_of(quartermaster)
        .expect("the quartermaster is in the world");
    rig.walk_to(at, 3.0, 6.0);
    rig.run(1.5);
    rig.command(Command::Use(quartermaster));
    assert!(
        rig.until(10.0, |r| elements(r, "Sale").len() == 4),
        "the store lists its four sales"
    );
    if check {
        let names: Vec<_> = elements(rig, "Sale")
            .into_iter()
            .filter_map(|r| sale_name(rig, r))
            .collect();
        assert_eq!(
            names,
            [
                "Health Potion",
                "Training Weapon",
                "Mucor Mushroom",
                "Tallow Bundle (100)"
            ]
        );
    }
    rig.panel(ToolbarChild::Inventory, true);
    let (gold, _) = wealth(rig);
    let items = pack(rig);
    buy(rig, "Mucor Mushroom");
    assert!(
        rig.until(5.0, |r| pack(r) == items + 1),
        "the mushroom reached the pack"
    );
    buy(rig, "Tallow Bundle (100)");
    assert!(
        rig.until(5.0, |r| pack(r) == items + 2),
        "the tallow reached the pack"
    );
    assert_eq!(wealth(rig).0, gold - 4, "two sales at 2 gold each");
    live::craft::press(rig, "ShopCloseButton");
    rig.run(0.5);

    // The craft panel: a vigor potion from the two purchases.
    let before = live::craft::open_recipe(rig, live::craft::VIGOR_RECIPE);
    let used = live::craft::drag_ingredients(rig, live::craft::VIGOR_RECIPE, &before);
    assert_eq!(used.len(), 2, "a mushroom and a tallow bundle");
    let mark = rig.seen.len();
    live::craft::press(rig, "CraftButton");
    assert!(
        rig.until(10.0, |r| live::craft::crafted(
            r,
            mark,
            live::craft::VIGOR_RECIPE
        )),
        "the craft did not finish: {:?}",
        rig.client.take_script_problems()
    );
    rig.run(1.0);
    assert_eq!(
        pack(rig),
        before.len() - 1,
        "two ingredients out, one potion in"
    );

    // The quest knows, and the mentor pays.
    rig.panel(ToolbarChild::Quest, true);
    if check {
        let crafted = rig.until(10.0, |r| journal(r).0.contains("you crafted a potion"));
        assert!(crafted, "the journal after the craft: {:?}", journal(rig));
    }
    let (gold0, xp0) = wealth(rig);
    let from = speak(rig);
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(u, f)| u.is_empty() && f.len() == 3)),
        "Potion Practice was not paid: {:?} {:?}",
        rig.view.quest_ids(),
        said_lines(rig, from)
    );
    let (gold1, xp1) = wealth(rig);
    assert_eq!((gold1 - gold0, xp1 - xp0), (8, 80));
    assert!(journal(rig).1.contains("Potion Practice"));
}

/// Potion Practice: the ingredients are bought from the quartermaster through the store panel
/// (purchase confirmation answered), the potion crafted through the craft panel, and the quest
/// updates in the journal and pays 80 experience and 8 gold at the mentor.
#[test]
fn potion_practice_is_bought_crafted_and_handed_in() {
    let Some(mut rig) = rig("Potionpractice") else {
        return;
    };
    play_to_supplies(&mut rig);
    play_potion_practice(&mut rig, true);
}

/// A Safe Journey: the mentor offers it, the lifestone is bound, the portal is used, and the
/// mentor at the destination pays 80 experience and 10 gold.
#[test]
fn a_safe_journey_binds_the_lifestone_and_travels_the_portal() {
    let Some(mut rig) = rig("Safejourney") else {
        return;
    };
    play_to_supplies(&mut rig);
    play_potion_practice(&mut rig, false);

    let from = speak(&mut rig);
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(u, _)| u == [JOURNEY])),
        "A Safe Journey was not offered: {:?} {:?}",
        rig.view.quest_ids(),
        said_lines(&rig, from)
    );
    assert!(
        journal(&rig).0.contains("use a lifestone to bind it"),
        "{:?}",
        journal(&rig)
    );

    // The lifestone: walk into range and use it.
    let stone = rig.object_of(LIFESTONE).expect("the lifestone");
    let at = rig.place_of(stone).expect("the lifestone is in the world");
    rig.walk_to(at, 3.0, 6.0);
    rig.run(1.5);
    let from = rig.seen.len();
    rig.command(Command::Use(stone));
    assert!(
        rig.until(10.0, |r| said(r, from, "bound")),
        "no binding: {:?}",
        said_lines(&rig, from)
    );
    assert!(
        rig.until(10.0, |r| journal(r).0.contains("your lifestone is bound")),
        "the journal after the binding: {:?}",
        journal(&rig)
    );

    // The portal: walk into range and use it; the body leaves the vault.
    let portal = rig.object_of(PORTAL).expect("the portal");
    let at = rig.place_of(portal).expect("the portal is in the world");
    rig.walk_to(at, 3.0, 6.0);
    // AC2e makes a player wait a few seconds between a binding and a portal.
    rig.run(8.0);
    let from = rig.seen.len();
    rig.command(Command::Use(portal));
    let left = rig.until(30.0, |r| {
        let frame = r.client.frame();
        frame.in_world && frame.player.as_ref().is_some_and(|p| p.cell != START_CELL)
    });
    assert!(
        left,
        "the portal did not take the player away: {:?}",
        said_lines(&rig, from)
    );
    let travelled = rig.until(10.0, |r| {
        journal(r)
            .0
            .contains("you bound a lifestone and used a training portal")
    });
    // The server's quest update comes in the batch of the portal's forced move, while the
    // player is out of the world: the journal takes it when the player is back.
    assert!(
        travelled,
        "the journal after the portal: {:?}",
        journal(&rig)
    );
    assert!(
        !journal(&rig).0.contains("your lifestone is bound"),
        "{:?}",
        journal(&rig)
    );

    // A mentor at the destination pays.
    rig.wait("a mentor at the destination", |r| {
        mentor(r).is_some_and(|m| r.place_of(m).is_some())
    });
    let (gold0, xp0) = wealth(&rig);
    let m = mentor(&rig).unwrap();
    let at = rig.place_of(m).unwrap();
    rig.walk_to(at, 3.0, 6.0);
    let from = speak(&mut rig);
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(u, f)| u.is_empty() && f.len() == 4)),
        "A Safe Journey was not paid: {:?} {:?}",
        rig.view.quest_ids(),
        said_lines(&rig, from)
    );
    let (gold1, xp1) = wealth(&rig);
    assert_eq!((gold1 - gold0, xp1 - xp0), (10, 80));
    assert!(journal(&rig).1.contains("A Safe Journey"));
}

const PRICES: [(&str, &str); 4] = [
    ("Health Potion", "5"),
    ("Training Weapon", "12"),
    ("Mucor Mushroom", "2"),
    ("Tallow Bundle (100)", "2"),
];

/// The store rows' price texts: each sale's cost shows `ID_Store_Quantity` of its price
/// (`gmShopUI::PostSale`, whose `Token_Quantity` is `Tokenize("quantity")`: the UI string
/// table's entry is only that variable), and the coin icon's tooltip names the cost's item.
#[test]
fn the_store_rows_show_their_prices() {
    let Some(mut rig) = rig("Pricewatch") else {
        return;
    };
    play_to_supplies(&mut rig);
    let quartermaster = rig.object_of(QUARTERMASTER).expect("the quartermaster");
    let at = rig.place_of(quartermaster).unwrap();
    rig.walk_to(at, 3.0, 6.0);
    rig.run(1.5);
    rig.command(Command::Use(quartermaster));
    assert!(rig.until(10.0, |r| elements(r, "Sale").len() == 4));
    let tree = rig.view.ui_tree().unwrap();
    let mut shown = Vec::new();
    for row in elements(&rig, "Sale") {
        let text_of = |name: &str| {
            let i = tree.child_recursive(row, ac2_ui::tokenize(name))?;
            Some((tree.get(i)?.text(), tree.tooltip_text(i)))
        };
        let name = sale_name(&rig, row).unwrap();
        let price = text_of("CostQuantity").and_then(|t| t.0).unwrap();
        let coin = text_of("CostIcon").and_then(|t| t.1).unwrap();
        shown.push((name, price, coin));
    }
    eprintln!("PRICES {shown:?}");
    for (name, price, coin) in &shown {
        assert!(!price.contains("string table error"), "{name}: {price}");
        assert_eq!(coin, "Coin", "{name}");
    }
    let prices: Vec<_> = shown.iter().map(|s| (s.0.as_str(), s.1.as_str())).collect();
    assert_eq!(prices, PRICES);
}
