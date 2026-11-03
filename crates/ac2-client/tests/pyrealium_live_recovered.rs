//! Pyrealium's recovered quests against a running Pyrealium (`PYREALIUM_LIVE`, see `tests/live/mod.rs`): the
//! delivery quest Gambling Debt (Kanten offers it, Orven completes it) and the hunt quests with
//! elixir and kill starters (`recovered_quests.rs`), played as a player does and read from the
//! quest journal (`gmJournalQuestsUI`) and the chat.
//!
//! Run with `PYREALIUM_LIVE_TEST=pyrealium_live_recovered AC2_DAT_DIR=.../dataset-ac2
//! scripts/pyrealium-live-test.sh` (the script starts Pyrealium's Legions profile for this file).

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_ui::ToolbarChild;
use ac2_weenie::{GameplayNotice, SessionEvent};
use live::LiveRig;

/// `recovered_quests::KANTEN`, `ORVEN`, `BAG`, `DEBT`.
const KANTEN: u32 = 0x4700_1a3c;
const ORVEN: u32 = 0x4700_1c27;
const DEBT: u32 = 0x4000_fe01;
const BAG: u32 = 0x4700_1c1e;

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
        .and_then(|o| o.qualities.inq_int(0xC1C))
        .unwrap_or(0);
    (gold, rig.client.frame().progress.experience.total_xp)
}

/// Travels by Pyrealium's test-server `@maptravel` to the land point (`x`, `y`) and waits for the
/// body to arrive (its cell changes). A second travel in one session is not applied by the
/// client (the server's forced position reaches it but the body stays), so the character logs
/// off and in again between travels, with the panels open as before.
fn travel(rig: &mut LiveRig, x: f64, y: f64, again: bool) {
    if again {
        rig.relog();
        rig.panel(ToolbarChild::Quest, true);
    }
    let cell = rig.client.frame().player.as_ref().map(|p| p.cell);
    let from = rig.seen.len();
    rig.command(Command::Say(format!("@maptravel {x:.2} {y:.2}")));
    assert!(
        rig.until(30.0, |r| {
            let frame = r.client.frame();
            frame.in_world && frame.player.as_ref().map(|p| p.cell) != cell
        }),
        "the map travel to {x} {y} from {cell:x?} did not take the player away: {:?}",
        said_lines(rig, from)
    );
    rig.run(3.0);
}

/// The NPC of entity `entity`, once it stands in the world within 3 m of the player (the map
/// travel put the player 2 m from it).
fn nearby(rig: &mut LiveRig, entity: u32) -> u64 {
    rig.wait("the NPC", |r| {
        r.object_of(entity).is_some_and(|n| r.place_of(n).is_some())
    });
    let npc = rig.object_of(entity).unwrap();
    let at = rig.place_of(npc).unwrap();
    let me = rig.origin();
    assert!((me[0] - at[0]).hypot(me[1] - at[1]) < 3.0, "{me:?} {at:?}");
    rig.run(1.0);
    npc
}

/// Uses `npc` and returns the index of the first event it caused.
fn talk(rig: &mut LiveRig, npc: u64) -> usize {
    let from = rig.seen.len();
    rig.command(Command::Use(npc));
    rig.run(3.0);
    from
}

/// Gambling Debt, offered by Kanten and completed at Orven (`recovered_quests::delivery`): the
/// offer (the journal's entry, the chat line, the bag in the pack), a second visit while the job
/// is open (refused, the bag not given twice), the delivery at Orven in Arwic (the bag taken,
/// 1,000 experience, the entry on the Finished page), and Orven's refusal without a job.
#[test]
fn kanten_offers_gambling_debt_and_orven_completes_it() {
    let Some(mut rig) = LiveRig::new("Gamblingdebt") else {
        return;
    };
    rig.panel(ToolbarChild::Quest, true);
    // Kanten's Bag in the pack (`recovered_quests::BAG`): the session's object of that entity.
    let pack = |r: &LiveRig| usize::from(r.object_of(BAG).is_some());

    // Orven first, with no job: he refuses.
    travel(&mut rig, 952.15, 1477.04, false);
    let orven = nearby(&mut rig, ORVEN);
    let from = talk(&mut rig, orven);
    assert!(
        said_lines(&rig, from)
            .iter()
            .any(|l| l.contains("Speak to Kanten for a delivery job")),
        "{:?}",
        said_lines(&rig, from)
    );
    assert_eq!(journal(&rig), (journal(&rig).0.clone(), String::new()));
    assert!(!journal(&rig).0.contains("Gambling Debt"));

    // Kanten offers the job.
    travel(&mut rig, 1310.55, 1185.38, true);
    let kanten = nearby(&mut rig, KANTEN);
    assert_eq!(pack(&rig), 0);
    let from = talk(&mut rig, kanten);
    let lines = said_lines(&rig, from);
    assert!(
        lines.iter().any(|l| l
            == "Gambling Debt: deliver Kanten's Bag to Orven in Arwic (44.9N, 7.1W) within two hours."),
        "{lines:?}"
    );
    assert!(
        rig.until(10.0, |r| journal(r).0.contains("Gambling Debt")),
        "{:?}",
        journal(&rig)
    );
    assert!(rig.until(5.0, |r| pack(r) == 1), "the bag in the pack");
    let ids = rig.view.quest_ids().unwrap();
    assert!(ids.0.contains(&DEBT), "{ids:?}");
    // The entry opens on its row: description, progress, time limit and level.
    // (A quest the server just updated opens by itself, `m_updatedQuestID`; a press opens or
    // closes a row.)
    if !journal(&rig).0.contains("Kanten needs a payoff") {
        let at = ids.0.iter().position(|&q| q == DEBT).unwrap();
        let row = rig.view.quest_row_centers().0[at];
        rig.click(row);
        rig.run(0.5);
    }
    let text = journal(&rig).0;
    assert!(
        text.contains("Kanten needs a payoff delivered to Orven in Arwic"),
        "{text}"
    );
    assert!(
        text.contains("Progress: 0 / 1. Reward: 1000 experience. Time limit: 120 minutes."),
        "{text}"
    );
    assert!(text.contains("Intended Level: 11"), "{text}");

    // Back at Kanten while the job is open: no second bag.
    let from = talk(&mut rig, kanten);
    assert!(
        said_lines(&rig, from)
            .iter()
            .any(|l| l.contains("Deliver Kanten's Bag to Orven in Arwic before returning")),
        "{:?}",
        said_lines(&rig, from)
    );
    assert_eq!(pack(&rig), 1);

    // Orven takes the bag and pays.
    let (_, xp0) = wealth(&rig);
    travel(&mut rig, 952.15, 1477.04, true);
    let orven = nearby(&mut rig, ORVEN);
    let from = talk(&mut rig, orven);
    let lines = said_lines(&rig, from);
    assert!(
        lines
            .iter()
            .any(|l| l == "Gambling Debt complete. You receive 1,000 experience."),
        "{lines:?}"
    );
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(u, f)| !u.contains(&DEBT) && f.contains(&DEBT))),
        "{:?} {lines:?}",
        rig.view.quest_ids()
    );
    assert!(
        journal(&rig).1.contains("Gambling Debt"),
        "{:?}",
        journal(&rig)
    );
    assert_eq!(wealth(&rig).1 - xp0, 1000);
    assert!(rig.until(5.0, |r| pack(r) == 0), "the bag left the pack");
}

/// Whether the session's object `id` is a creature of one of `entities` that is alive.
fn alive_of(rig: &LiveRig, id: u64, entities: &[u32]) -> bool {
    rig.client.session().objects().get(id).is_some_and(|o| {
        o.entity.is_some_and(|e| entities.contains(&e))
            && o.qualities.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F)
            && o.qualities.inq_int(0x102).is_none_or(|h| h > 0)
    })
}

/// The nearest live creature of `entities` that stands in the world within 25 m and has not
/// been fought (`fought`).
fn nearest_of(rig: &LiveRig, entities: &[u32], fought: &[u64]) -> Option<u64> {
    let frame = rig.client.frame();
    let player = frame.player.as_ref()?;
    frame
        .objects
        .iter()
        .filter(|o| o.in_world && !fought.contains(&o.id) && alive_of(rig, o.id, entities))
        .map(|o| {
            let d = (o.origin[0] - player.origin[0]).hypot(o.origin[1] - player.origin[1]);
            (o.id, d)
        })
        .filter(|&(_, d)| d < 25.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Kills the creature `target`: selects it, starts the attack and waits for the defeat message.
/// Whether it was defeated.
fn kill(rig: &mut LiveRig, target: u64) -> bool {
    let before = rig.seen.len();
    rig.command(Command::Select(Some(target)));
    rig.run(0.5);
    rig.command(Command::SetAttacking(true));
    rig.until(40.0, |r| {
        said_lines(r, before)
            .iter()
            .any(|l| l.contains("Creature defeated") || l.contains("not an attackable"))
    });
    let dead = said_lines(rig, before)
        .iter()
        .any(|l| l.contains("Creature defeated"));
    rig.command(Command::SetAttacking(false));
    rig.run(0.5);
    dead
}

/// Opens the journal entry of quest `id` (a press on its row) unless its text shows.
fn open_entry(rig: &mut LiveRig, id: u32, shown: &str) {
    if journal(rig).0.contains(shown) {
        return;
    }
    let ids = rig.view.quest_ids().expect("the journal");
    let at = ids
        .0
        .iter()
        .position(|&q| q == id)
        .expect("the quest underway");
    let row = rig.view.quest_row_centers().0[at];
    rig.click(row);
    rig.run(0.5);
}

/// `Incursion: Green Phyntos Wasp` (0x40000097), a kill-starter quest: killing a Green Wasp
/// Striker bestows it without counting; five Green Phyntos Wasps finish it for 240 experience
/// (`recovered_quests::Book::kill`). Played at two wasp camps (a striker and four wasps each),
/// the second after a relog, as the server's kill credit is per kill.
#[test]
fn a_green_wasp_striker_bestows_the_incursion_and_five_wasps_finish_it() {
    const STRIKER: u32 = 0x4700_1623;
    const WASPS: [u32; 2] = [0x4700_0f9c, 0x4700_0f9d];
    const QUEST: u32 = 0x4000_0097;
    /// Camps of Green Phyntos Wasps (land coordinates, 2 m from one), after the first.
    const OTHER_CAMPS: [(f64, f64); 8] = [
        (1044.95, 1334.19),
        (957.6, 1221.21),
        (1133.4, 1292.99),
        (1047.0, 1399.46),
        (757.95, 1428.83),
        (765.05, 1419.56),
        (781.1, 1419.12),
        (788.5, 1402.87),
    ];
    let Some(mut rig) = LiveRig::new("Waspincursion") else {
        return;
    };
    rig.panel(ToolbarChild::Quest, true);
    assert!(journal(&rig).0.contains("Training Grounds"));
    assert!(!rig.view.quest_ids().unwrap().0.contains(&QUEST));

    // A wasp killed before the starter counts for nothing and bestows nothing.
    travel(&mut rig, 1058.4, 1387.4, false);
    rig.wait("the wasps", |r| nearest_of(r, &WASPS, &[]).is_some());
    let first = nearest_of(&rig, &WASPS, &[]).unwrap();
    assert!(kill(&mut rig, first));
    assert!(!rig.view.quest_ids().unwrap().0.contains(&QUEST));

    // The striker bestows the quest with no count.
    rig.wait("the striker", |r| nearest_of(r, &[STRIKER], &[]).is_some());
    let striker = nearest_of(&rig, &[STRIKER], &[]).unwrap();
    assert!(kill(&mut rig, striker));
    assert!(
        rig.until(10.0, |r| r.view.quest_ids().unwrap().0.contains(&QUEST)),
        "{:?}",
        rig.view.quest_ids()
    );
    open_entry(&mut rig, QUEST, "Kill 5 Green Phyntos Wasps");
    let text = journal(&rig).0;
    assert!(text.contains("Incursion: Green Phyntos Wasp"), "{text}");
    assert!(text.contains("Kill 5 Green Phyntos Wasps."), "{text}");
    assert!(
        text.contains("Progress: 0 / 5. Reward: 240 experience. Time limit: 60 minutes."),
        "{text}"
    );
    assert!(text.contains("Intended Level: 2"), "{text}");

    // The wasps count: progress shows as they fall, at this camp and the camps after it (the
    // spawns are few and live once; the character logs in again at each).
    let (_, xp0) = wealth(&rig);
    let mark = rig.seen.len();
    let mut counted = 0;
    let mut fought = vec![first];
    let mut site = 0;
    while counted < 5 {
        while counted < 5 {
            let Some(target) = nearest_of(&rig, &WASPS, &fought) else {
                break;
            };
            fought.push(target);
            if kill(&mut rig, target) {
                counted += 1;
                if counted < 5 {
                    let want = format!("Progress: {counted} / 5.");
                    open_entry(&mut rig, QUEST, &want);
                    assert!(
                        rig.until(10.0, |r| journal(r).0.contains(&want)),
                        "after {counted} kills: {:?}",
                        journal(&rig)
                    );
                }
            }
        }
        if counted < 5 {
            let (x, y) = OTHER_CAMPS[site];
            site += 1;
            travel(&mut rig, x, y, true);
            rig.run(3.0);
            let here = nearest_of(&rig, &WASPS, &fought).is_some();
            eprintln!("CAMP {x} {y}: wasps in view {here}, counted so far {counted}");
        }
    }
    assert!(
        rig.until(10.0, |r| r
            .view
            .quest_ids()
            .is_some_and(|(u, f)| !u.contains(&QUEST) && f.contains(&QUEST))),
        "{:?}",
        rig.view.quest_ids()
    );
    let finished = journal(&rig).1;
    assert!(
        finished.contains("Incursion: Green Phyntos Wasp"),
        "{finished}"
    );
    let (_, xp1) = wealth(&rig);
    // The experience: each kill's own (the server's "Earned N XP") and the quest's 240.
    let earned: u64 = said_lines(&rig, mark)
        .iter()
        .filter_map(|l| {
            l.split("Earned ")
                .nth(1)?
                .split(" XP")
                .next()?
                .parse::<u64>()
                .ok()
        })
        .sum();
    assert_eq!(xp1 - xp0, earned + 240, "{:?}", said_lines(&rig, mark));
}

/// A quest elixir (`recovered_quests` `use_starters`): the Bold Vermin Pilferer and the Brave
/// Crawlspace Vermin drop the Beetle Slayer and the Wasp Bane elixir (Pyrealium's observed loot
/// bundles, rolled at a chance per body). The elixir is looted from a body, and drinking it
/// (a use) bestows the quest: the elixir leaves the pack, the chat says so, the journal lists it
/// with no progress. The vermin are no targets of either quest, so the kills before it count for
/// nothing.
#[test]
fn a_vermin_elixir_is_looted_and_drunk_to_bestow_its_quest() {
    /// (elixir entity, quest, journal title, journal text of the target) per elixir.
    const ELIXIRS: [(u32, u32, &str, &str); 2] = [
        (
            0x4700_0f42,
            0x4000_0009,
            "Beetle Slayer",
            "Kill 10 Edge Beetles.",
        ),
        (0x4700_0f43, 0x4000_000a, "Wasp Bane", "Kill 10"),
    ];
    const VERMIN: [u32; 2] = [0x4700_1072, 0x4700_0f7e];
    /// Places of vermin spawns (land coordinates, from the audited captures), a block each; the
    /// travel puts the player within 2 m of one, and the others of the block stand near.
    const SITES: [(f64, f64); 10] = [
        (935.2, 1429.2),
        (939.9, 1451.2),
        (913.8, 1405.4),
        (911.6, 1387.4),
        (917.5, 1419.8),
        (926.4, 1434.9),
        (941.5, 1444.9),
        (917.2, 1394.8),
        (969.9, 1452.3),
        (978.2, 1479.4),
    ];
    let Some(mut rig) = LiveRig::new("Vermin") else {
        return;
    };
    rig.panel(ToolbarChild::Quest, true);
    let elixir_in_pack = |r: &LiveRig| ELIXIRS.iter().find(|e| r.object_of(e.0).is_some()).copied();
    let mut fought = Vec::new();
    let mut found = None;
    'sites: for (site, &(x, y)) in SITES.iter().enumerate() {
        travel(&mut rig, x, y, site > 0);
        rig.run(3.0);
        eprintln!(
            "SITE {x} {y}: vermin in view {}",
            nearest_of(&rig, &VERMIN, &fought).is_some()
        );
        while let Some(target) = nearest_of(&rig, &VERMIN, &fought) {
            fought.push(target);
            let mark = rig.seen.len();
            if !kill(&mut rig, target) {
                eprintln!("NOKILL {target:x}: {:?}", said_lines(&rig, mark));
                continue;
            }
            // The body: a use opens its window; Loot All takes what it holds.
            rig.command(Command::Use(target));
            rig.run(2.0);
            eprintln!(
                "VERMIN {target:x} body holds {:?}",
                rig.view.item_icons("ExternalContainer")
            );
            if let Some(button) = rig.view.ui_element_center("LootAllButton") {
                if !rig.view.item_icons("ExternalContainer").is_empty() {
                    rig.click(button);
                    rig.run(2.0);
                }
            }
            if let Some(e) = elixir_in_pack(&rig) {
                found = Some(e);
                break 'sites;
            }
        }
    }
    eprintln!("{} vermin killed", fought.len());
    let (entity, quest, title, text) = found.expect("no vermin body held an elixir");
    assert!(!rig.view.quest_ids().unwrap().0.contains(&quest));
    // Drinking it bestows the quest.
    let elixir = rig.object_of(entity).unwrap();
    let from = rig.seen.len();
    rig.command(Command::Use(elixir));
    assert!(
        rig.until(10.0, |r| r.view.quest_ids().unwrap().0.contains(&quest)),
        "{:?} {:?}",
        rig.view.quest_ids(),
        said_lines(&rig, from)
    );
    assert!(
        rig.until(5.0, |r| r.object_of(entity).is_none()),
        "the elixir left the pack"
    );
    assert!(
        said_lines(&rig, from)
            .iter()
            .any(|l| l.contains(&format!("{title} accepted."))),
        "{:?}",
        said_lines(&rig, from)
    );
    open_entry(&mut rig, quest, "Progress: 0 / 10");
    let shown = journal(&rig).0;
    assert!(shown.contains(title), "{shown}");
    assert!(shown.contains(text), "{shown}");
    assert!(shown.contains("Progress: 0 / 10."), "{shown}");
}

/// The Quest panel's Story page (`gmStoryQuestsUI::ReloadActs`) on a new character: the player's
/// script (`QuestSystem`, `gmCWInterface::InqActInfoList`) lists the story's eight acts from the
/// DAT alone (names in the `act*.stt` string tables: Prologue, Osteth, Omishan, Linvak, Monthly
/// Events, Quests, and the two invisible ones, Quests Invisible and Transitions), the first six
/// visible, and no scene is playable: Pyrealium bestows none. Retail's server bestowed scenes with
/// its `Eff_BestowScene*` effect packages (58 in the wlib, run by the retail server's script VM,
/// which Pyrealium does not run); which event bestowed which scene is in no DAT table or client code, so
/// Pyrealium has no rule to apply and sends none.
#[test]
fn the_story_page_lists_the_dat_acts_and_no_scene_is_playable() {
    /// (act number, name table, visible, scenes) per act, in the script's order.
    const ACTS: [(u32, u32, u32, usize); 8] = [
        (1, 0x2500_14be, 1, 3),
        (2, 0x2500_0a57, 1, 7),
        (3, 0x2500_1391, 1, 8),
        (4, 0x2500_1390, 1, 8),
        (5, 0x2500_17aa, 1, 26),
        (6, 0x2500_6de7, 1, 8),
        (7, 0x2500_7ff7, 0, 1),
        (8, 0x2500_8133, 0, 33),
    ];
    let Some(mut rig) = LiveRig::new("Storyacts") else {
        return;
    };
    rig.panel(ToolbarChild::Quest, true);
    assert!(
        rig.until(10.0, |r| r.view.story_acts().len() == ACTS.len()),
        "{:?}",
        rig.view.story_acts().len()
    );
    let acts = rig.view.story_acts();
    for (act, want) in acts.iter().zip(ACTS) {
        assert_eq!(
            (
                act.act_num,
                act.name.table_id,
                act.visible,
                act.scenes.len()
            ),
            want,
            "act {:x}",
            act.id
        );
        assert_eq!(act.playable, 0, "act {:x}", act.id);
        assert!(
            act.scenes.iter().all(|s| s.is_playable == 0),
            "act {:x}: a scene is playable",
            act.id
        );
    }
}
