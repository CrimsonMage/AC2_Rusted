//! `gmExaminationUI` on the real toolbar layout (`AC2_DAT_DIR`): a profile's rows, the countdown
//! rewritten by the global loop, and the rows flushed by the next profile.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::examination::ExaminationItem;
use crate::token::tokenize;
use crate::ui_string::{UiString, UiStrings};
use crate::{build_gameplay_hud, ExaminationUi, SkillVar, UiTree};

/// The strings as `id|var|var`, so a countdown's words show their variables.
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

/// The text of every row's element, in tree order.
fn row_texts(tree: &UiTree, panel: &ExaminationUi) -> Vec<String> {
    fn walk(tree: &UiTree, index: usize, out: &mut Vec<String>) {
        if let Some(text) = tree.get(index).and_then(|e| e.text()) {
            if !text.is_empty() {
                out.push(text);
            }
        }
        for &child in tree.children(index) {
            walk(tree, child, out);
        }
    }
    let mut out = Vec::new();
    if let Some(list) = panel.list() {
        for &row in tree.list_items(list) {
            walk(tree, row, &mut out);
        }
    }
    out
}

#[test]
fn a_countdown_shows_its_time_and_the_global_loop_counts_it_down() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.examination.as_mut().expect("the examination panel");
    let strings = Strings;
    let tree = &mut hud.tree;

    // A text node, then a countdown of 61 seconds at time 1000.
    panel.update_profile(
        tree,
        &strings,
        1000.0,
        &[
            ExaminationItem::Text {
                appearance: 0,
                text: "Label".into(),
            },
            ExaminationItem::Countdown {
                appearance: 0,
                seconds: 61.0,
            },
        ],
    );
    let shown = row_texts(tree, panel);
    assert!(shown.contains(&"Label".to_string()), "{shown:?}");
    // One minute and one second: the minutes-and-seconds words, as `time_left` gives them.
    let at_start = shown
        .iter()
        .find(|t| t.starts_with(&format!("{:08x}", tokenize("ID_Time_Minutes"))))
        .cloned()
        .unwrap_or_default();
    assert!(at_start.ends_with("|1|01"), "{shown:?}");

    // Thirty seconds later the global loop rewrites the words, once the next change is due.
    panel.global_loop(tree, &strings, 1030.0);
    let later = row_texts(tree, panel);
    assert!(
        later.iter().any(
            |t| t.starts_with(&format!("{:08x}", tokenize("ID_Time_Minutes")))
                && t.ends_with("|0|31")
        ),
        "{later:?}"
    );

    // The next profile flushes the countdown with its rows.
    panel.update_profile(tree, &strings, 1030.0, &[]);
    assert!(row_texts(tree, panel).is_empty());
}
