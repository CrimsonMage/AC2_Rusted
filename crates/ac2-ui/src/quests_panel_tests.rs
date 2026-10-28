//! `gmQuestsUI` on the real toolbar layout (`AC2_DAT_DIR`): the panel's show reaches the journal,
//! a quest notice makes its page's tab active, and the quest button's flash follows the panel.

use std::cell::Cell;
use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::{GmActInfo, GmQuestInfo, StringInfo};
use ac2_weenie::{GameplayNotice, SceneInfo};

use crate::journal_quests::{
    find_named, JournalSource, QuestTab, CODE_ADD, CODE_LIST, CODE_UPDATE, STATUS_COMPLETED,
    STATUS_UNDERWAY,
};
use crate::token::tokenize;
use crate::{build_gameplay_hud, QuestSource, UiTree, STORY_CODE_LIST};

/// A quest source that records whether the panel asked the script for the quest list.
struct Source {
    list_asked: Cell<bool>,
}

impl crate::ui_string::UiStrings for Source {
    fn render_string(&self, string: &crate::ui_string::UiString) -> String {
        format!("{:08x}", string.string_id)
    }
}

impl JournalSource for Source {
    fn render_info(&self, _info: &StringInfo) -> String {
        "name".into()
    }
}

impl QuestSource for Source {
    fn quest_list(&self) -> Vec<GmQuestInfo> {
        self.list_asked.set(true);
        Vec::new()
    }

    fn act_list(&self) -> Vec<GmActInfo> {
        Vec::new()
    }

    fn play_story_scene(&self, _scene: SceneInfo) {}
}

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

#[test]
fn gameplay_notices_reach_the_panel_and_its_list() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let quests = hud.quests.as_mut().expect("the quest panel");
    let source = Source {
        list_asked: Cell::new(false),
    };

    // Other notices are not the panel's: no list is asked for.
    quests.recv_gameplay_notice(tree, &source, &GameplayNotice::SelectionChanged(0));
    assert!(!source.list_asked.get());
    quests.recv_gameplay_notice(
        tree,
        &source,
        &GameplayNotice::UpdateQuests {
            code: CODE_ADD,
            quest: Box::new(quest(1, STATUS_UNDERWAY)),
        },
    );
    assert!(
        !source.list_asked.get(),
        "only the list code asks the script"
    );

    // The list code asks the script for the quests first.
    quests.recv_gameplay_notice(
        tree,
        &source,
        &GameplayNotice::UpdateQuests {
            code: CODE_LIST,
            quest: Box::new(quest(1, STATUS_UNDERWAY)),
        },
    );
    assert!(source.list_asked.get());

    // A story notice reloads the Story page's acts on the global loop.
    quests.recv_gameplay_notice(
        tree,
        &source,
        &GameplayNotice::UpdateStoryQuests {
            code: STORY_CODE_LIST,
            act: 0,
        },
    );
    assert!(quests.story().reload_pending());
}
