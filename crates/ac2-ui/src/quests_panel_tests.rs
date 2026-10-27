//! `gmQuestsUI` on the real toolbar layout (`AC2_DAT_DIR`): the panel's show reaches the journal,
//! a quest notice makes its page's tab active, and the quest button's flash follows the panel.

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::GmQuestInfo;

use crate::journal_quests::{
    find_named, QuestTab, CODE_ADD, CODE_UPDATE, STATUS_COMPLETED, STATUS_UNDERWAY,
};
use crate::token::tokenize;
use crate::{build_gameplay_hud, UiTree};

fn quest(id: u32, status: u32) -> GmQuestInfo {
    GmQuestInfo {
        id,
        bestowal: f64::from(id),
        status,
        ..GmQuestInfo::default()
    }
}

fn state_is(tree: &UiTree, name: &str, state: &str) -> bool {
    find_named(tree, name)
        .and_then(|e| tree.get(e))
        .is_some_and(|e| e.state() == tokenize(state))
}

fn visible(tree: &UiTree, name: &str) -> bool {
    find_named(tree, name)
        .and_then(|e| tree.get(e))
        .is_some_and(|e| e.visible())
}

#[test]
fn the_panel_shows_the_journal_and_clears_its_flash() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let quests = hud.quests.as_mut().expect("the quest panel");
    assert!(!quests.shown());
    assert!(!quests.journal().shown());

    // A notice while the panel is hidden flashes the quest button.
    quests.recv_notice(tree, CODE_ADD, &quest(1, STATUS_UNDERWAY), &[]);
    assert!(state_is(tree, "QuestButton", "changed"));

    // Opening the panel shows the journal and stops the flash.
    quests.sync(tree, true);
    assert!(quests.shown());
    assert!(quests.journal().shown());
    assert!(state_is(tree, "QuestButton", "unchanged"));

    // Closing it with the added quest open clears the quest's expansion (`Show(0)`), and the
    // panel's state follows the toolbar's answer every frame without repeating the change.
    quests.sync(tree, true);
    quests.sync(tree, false);
    assert!(!quests.shown());
    assert!(!quests.journal().shown());
}

#[test]
fn a_notice_makes_its_page_tab_active() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let quests = hud.quests.as_mut().expect("the quest panel");

    quests.recv_notice(tree, CODE_ADD, &quest(1, STATUS_UNDERWAY), &[]);
    assert!(state_is(tree, "UnderwayTab", "open"));
    assert!(visible(tree, "UnderwayPage"));
    assert!(!visible(tree, "FinishedPage"));

    // Completed: the quest moves to the finished page and its tab opens, closing the underway one.
    quests.recv_notice(tree, CODE_UPDATE, &quest(1, STATUS_COMPLETED), &[]);
    assert!(state_is(tree, "FinishedTab", "open"));
    assert!(!state_is(tree, "UnderwayTab", "open"));
    assert!(visible(tree, "FinishedPage"));
    assert!(!visible(tree, "UnderwayPage"));
    assert_eq!(quests.journal().finished_ids(), vec![1]);

    // The panel's SetActivePanel on a tab that is open keeps it open.
    quests.set_active_panel(tree, QuestTab::Finished);
    assert!(state_is(tree, "FinishedTab", "open"));
}
