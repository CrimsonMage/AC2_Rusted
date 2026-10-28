//! `gm3DTextUI::CheckName`: the selection's name in the radar look's colour.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::layout::AttributeValue;
use crate::token::tokenize;
use crate::{build_gameplay_hud, GameplayHud};

#[test]
fn the_selection_name_takes_the_radar_look_colour() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let GameplayHud {
        mut tree, reticle, ..
    } = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut reticle = reticle.expect("the reticle");
    let name = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == tokenize("SelectionName"))
        .expect("SelectionName");
    let channel = |tree: &crate::UiTree, which: &str| {
        tree.get(name)
            .unwrap()
            .attribute(tokenize(&format!("font_color_{which}")))
            .cloned()
    };
    reticle.selection_changed(
        &mut tree,
        Some(7),
        Some("Gnome"),
        Some([0.5, 0.25, 1.0, 1.0]),
    );
    assert_eq!(channel(&tree, "red"), Some(AttributeValue::Float(0.5)));
    assert_eq!(channel(&tree, "green"), Some(AttributeValue::Float(0.25)));
    assert_eq!(channel(&tree, "blue"), Some(AttributeValue::Float(1.0)));
    // No look: white (`RGBAColor_White`).
    reticle.selection_changed(&mut tree, Some(8), Some("Rock"), None);
    assert_eq!(channel(&tree, "red"), Some(AttributeValue::Float(1.0)));
}
