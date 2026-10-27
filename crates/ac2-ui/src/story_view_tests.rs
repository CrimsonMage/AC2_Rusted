//! `gmStoryUI`'s drawn fields on the real gameplay layout (`AC2_DAT_DIR`): the keyframe goes into
//! the field `_lastState` names, the image-change flip swaps the Frame states, the subtitle lands
//! in the field's text, and the hide clears both fields.

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::StringInfo;

use crate::element::UiTree;
use crate::story::{StoryField, StoryShow};
use crate::token::{tokenize, STATELESS};
use crate::{build_gameplay_hud, StoryView};

/// The `StoryFieldN` root element index.
fn field_root(tree: &UiTree, name: &str) -> usize {
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).is_some_and(|e| e.id() == tokenize(name)))
        .unwrap_or_else(|| panic!("{name} is a root"))
}

/// A literal subtitle's text (the test's renderer).
fn literal(info: &StringInfo) -> String {
    info.literal.clone().unwrap_or_default()
}

/// A keyframe for `field` with `image` and `subtitle`.
fn show(field: StoryField, image: u32, changed: bool, subtitle: &str) -> StoryShow {
    StoryShow {
        index: 0,
        field,
        image: Some(image),
        image_changed: changed,
        music: None,
        voice: None,
        subtitle: Some(StringInfo::literal(subtitle)),
        seconds: 5.0,
    }
}

/// The image file and text of `name`'s field.
fn contents(tree: &UiTree, name: &str) -> (Option<u32>, Option<String>) {
    let root = field_root(tree, name);
    let image = tree
        .child_recursive(root, tokenize("ImageField"))
        .and_then(|i| tree.get(i))
        .and_then(|e| e.image())
        .map(|image| image.file);
    let text = tree
        .child_recursive(root, tokenize("TextField"))
        .and_then(|i| tree.get(i))
        .and_then(|e| e.text());
    (image, text)
}

#[test]
fn the_story_fields_draw_each_keyframe_and_hide_again() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut story: StoryView = hud.story.expect("the story fields in the gameplay layout");
    assert!(!story.is_shown());
    assert!(!tree.get(field_root(tree, "StoryField1")).unwrap().visible());

    // The first keyframe goes into field 1 (its `Frame1` state) with the subtitle.
    story.show(tree, &show(StoryField::First, 100, true, "Hello"), &literal);
    assert!(story.is_shown());
    assert!(tree.get(field_root(tree, "StoryField1")).unwrap().visible());
    assert_eq!(
        tree.get(field_root(tree, "StoryField1")).unwrap().state(),
        tokenize("Frame1")
    );
    assert_eq!(
        contents(tree, "StoryField1"),
        (Some(100), Some("Hello".to_owned()))
    );

    // A new image in field 2 cross-fades: field 2 takes `Frame2`, field 1 goes back.
    story.show(
        tree,
        &show(StoryField::Second, 200, true, "Second"),
        &literal,
    );
    assert_eq!(
        tree.get(field_root(tree, "StoryField2")).unwrap().state(),
        tokenize("Frame2")
    );
    assert_eq!(
        tree.get(field_root(tree, "StoryField1")).unwrap().state(),
        STATELESS
    );
    assert_eq!(
        contents(tree, "StoryField2"),
        (Some(200), Some("Second".to_owned()))
    );

    story.hide(tree);
    assert!(!story.is_shown());
    assert!(!tree.get(field_root(tree, "StoryField2")).unwrap().visible());
    // The image is gone and the subtitle is empty (`UIElement_Text::ClearAllText`).
    for name in ["StoryField1", "StoryField2"] {
        let (image, text) = contents(tree, name);
        assert_eq!(image, None, "{name}");
        assert!(text.unwrap_or_default().is_empty(), "{name}");
    }
}
