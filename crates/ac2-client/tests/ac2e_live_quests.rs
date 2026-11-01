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
            | SessionEvent::Notice(GameplayNotice::DisplayMessage { text, .. }) => text.literal.clone(),
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
            rig.client
                .session()
                .objects()
                .get(o.id)
                .is_some_and(|x| {
                    x.qualities.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F)
                        && x.qualities.inq_int(0x102).is_none_or(|h| h > 0)
                })
        })
        .map(|o| {
            let d: f32 = (0..3).map(|i| (o.origin[i] - player.origin[i]).powi(2)).sum();
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
        eprintln!("ATTACK {target:x} dead {dead}: {:?}", &lines[lines.len().saturating_sub(3)..]);
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
    assert!(underway.contains("Training creatures defeated: 0/3"), "{underway}");
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
        lines.iter().any(|l| l.starts_with("Pectar, Training Master:")),
        "{lines:?}"
    );
    assert!(
        rig.until(10.0, |r| r.view.quest_ids().is_some_and(|(_, f)| f == [TRAINING])),
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
        rig.until(5.0, |r| r.view.item_icons("PackMainContainer").len() + 1 == pack_before),
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
        r.view.quest_ids().is_some_and(|(u, f)| u.is_empty() && f.len() == 2)
    });
    assert_eq!(rig.view.quest_ids(), before.0);
    assert_eq!(wealth(&rig), before.1);
    let finished = journal(&rig).1;
    assert!(finished.contains("Training Grounds") && finished.contains("Field Supplies"));
}
