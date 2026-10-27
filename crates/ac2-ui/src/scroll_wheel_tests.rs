//! The mouse wheel over the texts with a vertical scroll bar in the real gameplay HUD
//! (`AC2_DAT_DIR`): the view and the bar move (`text_scroll.rs`). Every layout is covered by
//! `scroll_coverage_tests.rs`.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::hud::build_gameplay_hud;
use crate::token::{tokenize, TEXT};
use crate::{UiTree, UI_POINTER_X, UI_POINTER_Y, UI_WHEEL_OUT};

/// The elements of the tree whose kind is `kind` and that name a vertical scroll bar.
fn with_vertical_bar(tree: &UiTree, kind: u32) -> Vec<usize> {
    (0..tree.slots_len())
        .filter(|&i| {
            tree.get(i).is_some_and(|e| {
                e.desc.kind == kind && e.attributes.contains_key(&tokenize("vertical_scrollbar"))
            })
        })
        .collect()
}

/// Puts the pointer at the middle of element `index` and reports whether the element is under it.
fn point_at(tree: &mut UiTree, index: usize) -> bool {
    let rect = tree.screen_box(index);
    tree.start_input(UI_POINTER_X, ((rect.x0 + rect.x1) / 2) as f32);
    tree.start_input(UI_POINTER_Y, ((rect.y0 + rect.y1) / 2) as f32);
    tree.get(index).is_some_and(|e| e.mouse_over)
}

#[test]
fn the_wheel_scrolls_each_text_with_a_vertical_bar_in_the_gameplay_hud() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let texts = with_vertical_bar(tree, TEXT);
    assert!(
        !texts.is_empty(),
        "the gameplay layout has a text with a bar"
    );
    let mut scrolled = 0;
    for text in texts {
        let bar = tree
            .get(text)
            .and_then(|e| e.attributes.get(&tokenize("vertical_scrollbar")))
            .and_then(|v| match v {
                crate::layout::AttributeValue::Id(id) => {
                    tree.child_recursive(tree.root_of(text), *id)
                }
                _ => None,
            })
            .expect("the bar the text names");
        // Enough lines to overflow any of the HUD's text elements.
        let lines: String = (0..200).map(|n| format!("line {n}\n")).collect();
        tree.set_text(text, &lines);
        if !point_at(tree, text) {
            // Covered by another element in this HUD state: nothing to wheel over.
            continue;
        }
        let before = tree.get(text).unwrap().text.as_ref().unwrap().view.1;
        tree.start_input(UI_WHEEL_OUT, 1.0);
        let after = tree.get(text).unwrap().text.as_ref().unwrap().view.1;
        let paper = tree.text_paper_height(text).unwrap();
        let height = tree.get(text).unwrap().rect().height();
        if paper <= height {
            // The text fits its element: the bar is disabled and the wheel moves nothing.
            assert_eq!(after, before);
            continue;
        }
        assert!(after > before, "text {text}: view {before} -> {after}");
        let range = f64::from(paper - height);
        let share = tree.scrollbar_position(bar);
        assert!(
            (f64::from(share) - f64::from(after) / range).abs() < 1e-3,
            "bar {share} for view {after} of {range}"
        );
        scrolled += 1;
    }
    assert!(scrolled > 0, "no text in the HUD scrolled under the wheel");
}
