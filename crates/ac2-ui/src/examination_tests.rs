//! `gmExaminationUI` on the real toolbar layout (`AC2_DAT_DIR`): a profile's rows, the countdown
//! rewritten by the global loop, and the rows flushed by the next profile.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::examination::{ExaminationCall, ExaminationItem};
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};
use crate::token::{tokenize, MSG_MOUSE_PRESS};
use crate::ui_string::{UiString, UiStrings};
use crate::{build_gameplay_hud, ExaminationUi, SkillVar, ToolbarChild, UiTree};

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
        (0x203, 0x203),
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
    panel.update_profile(tree, &strings, 1030.0, &[], (0x203, 0x203));
    assert!(row_texts(tree, panel).is_empty());
}

/// The state token of each filter tab, in the order General, Details, Effects, Usage.
fn tab_states(tree: &UiTree) -> Vec<u32> {
    ["GeneralTab", "DetailsTab", "EffectsTab", "UsageTab"]
        .iter()
        .map(|name| {
            let field = tree
                .roots()
                .iter()
                .find_map(|&r| tree.child_recursive(r, tokenize("ExaminationField")))
                .expect("the examination field");
            let tab = tree
                .child_recursive(field, tokenize(name))
                .unwrap_or_else(|| panic!("{name}"));
            tree.get(tab).expect("tab").state()
        })
        .collect()
}

fn press(tree: &UiTree, name: &str) -> UiMessage {
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("ExaminationField")))
        .expect("the examination field");
    UiMessage {
        element_id: tokenize(name),
        element: tree.child_recursive(field, tokenize(name)),
        message: MSG_MOUSE_PRESS,
        data: 0,
        item: None,
    }
}

fn key(action: u32) -> UiMessage {
    UiMessage {
        element_id: 0,
        element: None,
        message: GLOBAL_KEY_PRESS,
        data: action as i32,
        item: None,
    }
}

/// Legions 0x00431170: a tab is active when the filter has its mask, inactive when the profile
/// has content for it, else disabled; a fresh panel shows the General tab only.
#[test]
fn the_tabs_follow_the_filter_and_the_profile() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.examination.as_mut().expect("the examination panel");
    let tree = &mut hud.tree;
    let (active, inactive, disabled) = (
        tokenize("active"),
        tokenize("inactive"),
        tokenize("disabled"),
    );
    assert_eq!(panel.filter(), 0x203);
    assert_eq!(tab_states(tree), vec![active, disabled, disabled, disabled]);

    // A profile for the General filter with content under Details and Usage.
    panel.update_profile(tree, &Strings, 0.0, &[], (0x203, 0x18C | 0x60));
    assert_eq!(tab_states(tree), vec![active, inactive, disabled, inactive]);
    // A profile answering a Details and Effects filter.
    panel.update_profile(tree, &Strings, 0.0, &[], (0x18C | 0x10, 0x203));
    assert_eq!(tab_states(tree), vec![inactive, active, active, disabled]);
}

/// A press on an enabled tab sets the filter (a Ctrl press toggles its bits) and starts the
/// examination again with it; a disabled tab does nothing; hidden, nothing happens.
#[test]
fn a_tab_press_sets_the_filter_and_restarts_the_examination() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let toolbar = &mut hud.toolbar;
    let panel = hud.examination.as_mut().expect("the examination panel");
    let tree = &mut hud.tree;

    // Hidden: a press is ignored.
    let details = press(tree, "DetailsTab");
    panel.message(tree, &details, false);
    assert!(panel.take_calls().is_empty());

    toolbar.show(tree, ToolbarChild::Examination, true);
    panel.follow_field(tree);
    // Opening starts the examination with the panel's word.
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x203)]);

    // Details has no content yet: disabled, so the press does nothing.
    panel.message(tree, &details, false);
    assert!(panel.take_calls().is_empty());
    assert_eq!(panel.filter(), 0x203);

    panel.update_profile(tree, &Strings, 0.0, &[], (0x203, 0x18C | 0x60));
    panel.message(tree, &details, false);
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x18C)]);
    assert_eq!(panel.filter(), 0x18C);

    // Ctrl on Usage adds its bits; Ctrl on it again takes them away.
    let usage = press(tree, "UsageTab");
    panel.message(tree, &usage, true);
    assert_eq!(
        panel.take_calls(),
        vec![ExaminationCall::Start(0x18C | 0x60)]
    );
    panel.message(tree, &usage, true);
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x18C)]);

    // The server's answer (the request changed) sets the filter the panel sends next.
    panel.request_changed(tree, 0x4000_0002, 0x10);
    assert_eq!(panel.filter(), 0x10);
}

/// The selection keys step to the next or previous tab that is not disabled, cycling.
#[test]
fn the_tab_keys_step_through_the_enabled_tabs() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let toolbar = &mut hud.toolbar;
    let panel = hud.examination.as_mut().expect("the examination panel");
    let tree = &mut hud.tree;
    toolbar.show(tree, ToolbarChild::Examination, true);
    panel.follow_field(tree);
    panel.take_calls();

    // Only the General tab is enabled: the keys choose it again.
    panel.message(tree, &key(0x4100_006A), false);
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x203)]);

    // General, Details and Usage have content (Effects does not): next skips Effects.
    panel.update_profile(tree, &Strings, 0.0, &[], (0x203, 0x18C | 0x60));
    panel.message(tree, &key(0x4100_006A), false);
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x18C)]);
    panel.message(tree, &key(0x4100_006A), false);
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x60)]);
    // Past the last tab it wraps to the first.
    panel.message(tree, &key(0x4100_006A), false);
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x203)]);
    // Previous from the first wraps to the last enabled one.
    panel.message(tree, &key(0x4100_006B), false);
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x60)]);
    panel.message(tree, &key(0x4100_006B), false);
    assert_eq!(panel.take_calls(), vec![ExaminationCall::Start(0x18C)]);
}

/// `ProcessNode` type 9: the image template shows the icon description's layers (`SetIconDesc`),
/// each with its shift colour, as the item slots do; type 5 shows the image alone.
#[test]
fn an_icon_node_shows_every_layer_of_its_icon_description() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.examination.as_mut().expect("the examination panel");
    let tree = &mut hud.tree;
    let layers = vec![
        crate::IconLayer {
            image: 0x4100_6F03,
            shift_color: 0xFFFF_FFFF,
        },
        crate::IconLayer {
            image: 0x4100_1AAB,
            shift_color: 0xFF80_4020,
        },
    ];
    panel.update_profile(
        tree,
        &Strings,
        0.0,
        &[ExaminationItem::Icon(layers.clone())],
        (0x203, 0x203),
    );
    fn find(tree: &UiTree, index: usize, out: &mut Vec<usize>) {
        if tree.get(index).is_some_and(|e| e.icon_layers.is_some()) {
            out.push(index);
        }
        for &child in tree.children(index) {
            find(tree, child, out);
        }
    }
    let mut icons = Vec::new();
    for &row in tree.list_items(panel.list().expect("the list")) {
        find(tree, row, &mut icons);
    }
    let [icon] = icons[..] else {
        panic!("one layered icon: {icons:?}");
    };
    let element = tree.get(icon).unwrap();
    assert_eq!(element.icon_layers.as_deref(), Some(&layers[..]));
    assert_eq!(element.image().map(|i| i.file), Some(0x4100_6F03));
}
