//! Tooltips on the real HUD (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::layout::AttributeValue;
use crate::token::tokenize;
use crate::{build_gameplay_hud, UiTree, UI_POINTER_X, UI_POINTER_Y};

/// A shown element with a tooltip: its centre on the screen.
fn hover_target(tree: &UiTree) -> Option<(usize, i32, i32)> {
    let on = tokenize("tooltips_on");
    (0..tree.slots_len()).find_map(|i| {
        let el = tree.get(i)?;
        let has = matches!(el.attribute(on), Some(AttributeValue::Bool(true)))
            && el.attribute(tokenize("tooltip_ID")).is_some()
            && el.attribute(tokenize("tooltip_entry")).is_some()
            && el.visible;
        let b = tree.screen_box(i);
        (has && b.width() > 4 && b.height() > 4).then(|| (i, (b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2))
    })
}

#[test]
fn a_resting_pointer_shows_the_tooltip_and_it_goes_after_ten_seconds() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut tree = build_gameplay_hud(&store, 1024, 768).unwrap().tree;
    let (target, x, y) = hover_target(&tree).expect("an element with a tooltip");
    tree.global_loop(1.0);
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
    tree.global_loop(1.1);
    assert!(tree.tooltip_element().is_none(), "not before the delay");
    tree.global_loop(1.3);
    let tip = tree.tooltip_element().expect("the tooltip");
    let texts: Vec<String> = (0..tree.slots_len())
        .filter(|&i| tree.root_of(i) == tip)
        .filter_map(|i| tree.get(i).and_then(|e| e.text()))
        .filter(|t| !t.is_empty())
        .collect();
    eprintln!("tooltip of {target}: {texts:?}");
    assert!(!texts.is_empty());
    // Below and right of the pointer, kept on the screen (the radar is at its right edge).
    let placement = tree
        .placements()
        .into_iter()
        .find(|p| p.root == tip)
        .unwrap();
    let width = tree.get(tip).unwrap().rect.width();
    assert_eq!(placement.x, (x + 32).min(1024 - width), "{placement:?}");
    assert_eq!(placement.y, y + 32, "{placement:?}");
    tree.global_loop(11.4);
    assert!(tree.tooltip_element().is_none(), "gone after ten seconds");
}
