//! The story fields in the gameplay layout (`gmStoryUI::gmStoryUI`'s `StoryField1` / `StoryField2`
//! roots, their `ImageField` and `TextField`, and the `Frame1` / `Frame2` cross-fade transitions).
//! Real data (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use super::*;
use crate::layout::ElementDesc;
use crate::token::tokenize;

/// Whether `id` is `element` or one of its descendants.
fn contains(element: &ElementDesc, id: u32) -> bool {
    element.id == id
        || element
            .states
            .iter()
            .flat_map(|state| state.children.iter())
            .any(|child| contains(child, id))
}

#[test]
fn the_gameplay_layout_has_the_story_fields_and_their_cross_fade() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    let layout = load_layout_by_enum(&store, crate::hud::LAYOUT_GAMEPLAY)
        .unwrap()
        .expect("gameplay layout");
    let frame1 = tokenize("Frame1");
    let frame2 = tokenize("Frame2");
    for field in ["StoryField1", "StoryField2"] {
        let root = layout
            .element(tokenize(field))
            .unwrap_or_else(|| panic!("{field} is a top-level element"));
        // The image and the subtitle are found by `GetChildRecursive`.
        assert!(contains(root, tokenize("ImageField")), "{field} image");
        assert!(contains(root, tokenize("TextField")), "{field} subtitle");
    }
    // The field's stateless state holds the fades into and out of the two frames.
    let field = layout.element(tokenize("StoryField1")).unwrap();
    let state = field.state(tokenize("stateless")).expect("stateless state");
    let targets: Vec<u32> = state.transitions.iter().map(|t| t.state).collect();
    assert!(
        targets.contains(&frame1) && targets.contains(&frame2),
        "{targets:x?}"
    );
}
