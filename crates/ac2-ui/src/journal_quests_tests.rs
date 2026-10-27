//! `gmJournalQuestsUI` and `TimeDisplay`: the time words and the list order on their own, and
//! the two pages on the real toolbar layout (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::{GmQuestInfo, StringInfo};

use crate::journal_quests::{
    insert_unique_sort, QuestDescriptor, CODE_ADD, CODE_LIST, CODE_REMOVE, CODE_UPDATE,
    STATUS_COMPLETED, STATUS_UNDERWAY,
};
use crate::journal_time::time_left;
use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};
use crate::ui_string::{UiString, UiStrings};
use crate::{build_gameplay_hud, JournalSource, SkillVar, UiTree};

/// The journal's strings as `id|var|var`.
struct Strings;

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        let mut out = format!("{:08x}", string.string_id);
        for (_, value) in &string.vars {
            out.push('|');
            match value {
                SkillVar::UInt(n) => out.push_str(&n.to_string()),
                SkillVar::Int(n) => out.push_str(&n.to_string()),
                SkillVar::Text(t) => out.push_str(t),
                _ => out.push('?'),
            }
        }
        out
    }
}

impl JournalSource for Strings {
    fn render_info(&self, _info: &StringInfo) -> String {
        "quest".into()
    }
}

fn quest(id: u32, bestowal: f64, status: u32) -> GmQuestInfo {
    GmQuestInfo {
        id,
        bestowal,
        status,
        ..GmQuestInfo::default()
    }
}

fn var<'a>(text: &'a crate::ui_string::UiString, name: &str) -> Option<&'a SkillVar> {
    text.vars
        .iter()
        .find(|(id, _)| *id == tokenize(name))
        .map(|(_, v)| v)
}

#[test]
fn the_time_words_follow_the_thresholds() {
    // Under an hour: minutes and seconds, the next change a second off.
    let minutes = time_left(61, 1, 1, 60);
    assert_eq!(minutes.text.string_id, tokenize("ID_Time_Minutes"));
    assert_eq!(var(&minutes.text, "minutes"), Some(&SkillVar::Int(1)));
    assert_eq!(minutes.next_update, 1.0);

    // Over an hour, under a day: hours and the minutes past the hour, next at the minute.
    let hours = time_left(3_700, 1, 1, 60);
    assert_eq!(hours.text.string_id, tokenize("ID_Time_Hours"));
    assert_eq!(var(&hours.text, "hours"), Some(&SkillVar::Int(1)));
    assert_eq!(
        var(&hours.text, "minutes"),
        Some(&SkillVar::Text("01".into()))
    );
    assert_eq!(hours.next_update, 40.0);

    // Exactly a day is not over the day threshold (1): it is hours still.
    let day = time_left(86_400 + 10, 1, 1, 60);
    assert_eq!(day.text.string_id, tokenize("ID_Time_Hours"));
    assert_eq!(var(&day.text, "hours"), Some(&SkillVar::Int(24)));

    // Two days and three hours: days and hours, next at the hour.
    let days = time_left(2 * 86_400 + 3 * 3_600 + 5, 1, 1, 60);
    assert_eq!(days.text.string_id, tokenize("ID_Time_Days"));
    assert_eq!(var(&days.text, "days"), Some(&SkillVar::UInt(2)));
    assert_eq!(var(&days.text, "hours"), Some(&SkillVar::UInt(3)));
    assert_eq!(days.next_update, 5.0);

    // Over a month (the threshold is more than one): months, the days past it, next at the day.
    let months = time_left(2 * 31 * 86_400 + 4 * 86_400 + 9, 1, 1, 60);
    assert_eq!(months.text.string_id, tokenize("ID_Time_Months"));
    assert_eq!(var(&months.text, "months"), Some(&SkillVar::UInt(2)));
    assert_eq!(var(&months.text, "days"), Some(&SkillVar::UInt(4)));
    assert_eq!(months.next_update, 9.0);
}

#[test]
fn quests_sort_newest_bestowal_first_and_stay_unique() {
    let mut list = Vec::new();
    for (id, bestowal) in [(1, 10.0), (2, 30.0), (3, 20.0), (4, 30.0)] {
        let desc = QuestDescriptor::new(quest(id, bestowal, STATUS_UNDERWAY));
        assert!(insert_unique_sort(&mut list, desc));
    }
    // An equal bestowal goes after the earlier quest (`insert_sort` stops at the first earlier one).
    let ids: Vec<u32> = list.iter().map(|q| q.quest.id).collect();
    assert_eq!(ids, vec![2, 4, 3, 1]);
    // A quest already listed is not added again (`insert_unique_sort`).
    let again = QuestDescriptor::new(quest(3, 99.0, STATUS_UNDERWAY));
    assert!(!insert_unique_sort(&mut list, again));
    assert_eq!(list.len(), 4);
}

fn named_list(tree: &UiTree, name: &str) -> usize {
    tree.roots()
        .iter()
        .find_map(|&root| tree.child_recursive(root, tokenize(name)))
        .unwrap_or_else(|| panic!("{name} in the toolbar"))
}

fn row_id(tree: &UiTree, row: usize) -> Option<u32> {
    match tree.get(row)?.attribute(tokenize("QuestIDToken")) {
        Some(AttributeValue::Id(id)) => Some(*id),
        _ => None,
    }
}

#[test]
fn the_pages_list_the_quests_and_follow_their_changes() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let journal = hud.quests.as_mut().expect("the quest panel").journal_mut();
    let tree = &mut hud.tree;
    let strings = Strings;
    let underway_list = named_list(tree, "UnderwayQuestListBox");
    let finished_list = named_list(tree, "FinishedQuestListBox");

    // A quest underway with an hour and a bit left: its row, its time counted down. An added
    // quest is the updated one, so it opens at once: the row and its extra text.
    let mut first = quest(1, 100.0, STATUS_UNDERWAY);
    first.seconds_remaining = 3_700.0;
    journal.recv_notice_update_quests(tree, CODE_ADD, &first, &[]);
    assert_eq!(journal.underway_ids(), vec![1]);
    journal.global_loop(tree, &strings, 1_000.0);
    let items = tree.list_items(underway_list).to_vec();
    assert_eq!(items.len(), 2);
    assert_eq!(row_id(tree, items[0]), Some(1));
    let time = tree
        .child_recursive(items[0], tokenize("QuestTime"))
        .expect("the row's time");
    assert_eq!(
        tree.get(time).and_then(|e| e.text()).as_deref(),
        Some(format!("{:08x}|1|01", tokenize("ID_Time_Hours"))).as_deref()
    );

    // Clicking the row closes it (the extra text goes), and clicking again opens it: the
    // description is listed under the row.
    let click = |journal: &mut crate::JournalQuestsUi, tree: &UiTree, row: usize| {
        journal.message(
            tree,
            &UiMessage {
                element_id: tokenize("QuestTemplate"),
                element: Some(row),
                message: MSG_BUTTON_RELEASE_ON,
                data: 0,
                item: None,
            },
        );
    };
    click(journal, tree, items[0]);
    journal.global_loop(tree, &strings, 1_001.0);
    assert_eq!(tree.list_items(underway_list).len(), 1);
    click(journal, tree, items[0]);
    journal.global_loop(tree, &strings, 1_001.5);
    let items = tree.list_items(underway_list).to_vec();
    assert_eq!(items.len(), 2);
    let extra = items[1];
    assert!(tree
        .get(extra)
        .and_then(|e| e.text())
        .is_some_and(|t| t.starts_with(&format!("{:08x}", tokenize("ID_Quest_Description")))));

    // Completed: the quest moves to the finished page, opened because it was the one updated.
    let mut done = first.clone();
    done.status = STATUS_COMPLETED;
    journal.recv_notice_update_quests(tree, CODE_UPDATE, &done, &[]);
    assert!(journal.underway_ids().is_empty());
    assert_eq!(journal.finished_ids(), vec![1]);
    journal.global_loop(tree, &strings, 1_002.0);
    assert!(tree.list_items(underway_list).is_empty());
    let finished = tree.list_items(finished_list).to_vec();
    assert_eq!(finished.len(), 2);
    assert!(tree
        .get(finished[1])
        .and_then(|e| e.text())
        .is_some_and(|t| t.starts_with(&format!("{:08x}", tokenize("ID_Quest_Completed")))));

    // Removed: the finished page empties.
    journal.recv_notice_update_quests(tree, CODE_REMOVE, &done, &[]);
    assert!(journal.finished_ids().is_empty());
    journal.global_loop(tree, &strings, 1_003.0);
    assert!(tree.list_items(finished_list).is_empty());

    // The whole list (code 2) fills both pages by status.
    let list = [
        quest(2, 5.0, STATUS_UNDERWAY),
        quest(3, 9.0, STATUS_COMPLETED),
    ];
    journal.recv_notice_update_quests(tree, CODE_LIST, &list[0], &list);
    assert_eq!(journal.underway_ids(), vec![2]);
    assert_eq!(journal.finished_ids(), vec![3]);
    // The list's first quest is the updated one, so it opens (row and extra text).
    journal.global_loop(tree, &strings, 1_004.0);
    assert_eq!(tree.list_items(underway_list).len(), 2);
    assert_eq!(tree.list_items(finished_list).len(), 1);
}
