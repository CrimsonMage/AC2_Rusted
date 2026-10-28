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

/// The tooltip preferences (`UIElementManager::HandleApply`), and an owner hidden while it has the
/// tooltip (`ChangingElementVisibility`).
#[test]
fn the_tooltip_follows_its_preferences_and_its_owner_hiding() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut tree = build_gameplay_hud(&store, 1024, 768).unwrap().tree;
    let (target, x, y) = hover_target(&tree).expect("an element with a tooltip");
    tree.global_loop(1.0);
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
    // A two-second delay: nothing at 1.3, the tooltip once 2 seconds have passed.
    tree.apply_tooltip_prefs(2.0, true);
    tree.global_loop(1.3);
    assert!(tree.tooltip_element().is_none(), "the longer delay");
    tree.global_loop(3.1);
    assert!(tree.tooltip_element().is_some());
    // Tooltips off clears it, and none comes back.
    tree.apply_tooltip_prefs(2.0, false);
    assert!(tree.tooltip_element().is_none());
    tree.global_loop(6.0);
    assert!(tree.tooltip_element().is_none());
    // On again shows it at once (cleared and started), and hiding its owner clears it.
    tree.apply_tooltip_prefs(2.0, true);
    assert!(tree.tooltip_element().is_some());
    tree.set_visible(target, false);
    assert!(tree.tooltip_element().is_none());
    assert_eq!(tree.tooltip_owner(), None);
}

/// Names for the test: every section and key is named (as the DAT names them).
struct Names;

impl ac2_prefs::NameTable for Names {
    fn section_name(&self, id: u32) -> Option<String> {
        Some(format!("Section{id:08X}"))
    }

    fn key_name(&self, id: u32) -> Option<String> {
        Some(format!("Key{id:08X}"))
    }

    fn enum_name(&self, _map: u32, _label: u32) -> Option<String> {
        None
    }

    fn enum_label(&self, _map: u32, _name: &str) -> Option<u32> {
        None
    }
}

#[test]
fn the_tooltip_keys_register_in_their_section() {
    use crate::tooltip_prefs::{register, values, TOOLTIP_DELAY, TOOLTIP_ENABLE, TOOLTIP_SECTION};
    let mut bank = ac2_prefs::PreferenceBank::new(b"");
    assert_eq!(register(&mut bank, &Names), 2);
    assert_eq!(
        bank.list(TOOLTIP_SECTION),
        vec![TOOLTIP_DELAY, TOOLTIP_ENABLE]
    );
    assert_eq!(values(&bank), (0.25, true));
}
