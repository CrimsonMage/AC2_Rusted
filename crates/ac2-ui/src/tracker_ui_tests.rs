//! The Tracker's interface on the real gameplay and toolbar layouts (`AC2_DAT_DIR`): the eyeball
//! sits on the radar and toggles the window, the window's buttons and list make requests, and the
//! close button hides it.

use ac2_db::ResourceStore;
use ac2_macro::tracker::{Presentation, PresentedMatch, PresentedPattern};

use super::*;
use crate::build_gameplay_hud;

fn release(element: Option<usize>, element_id: u32) -> UiMessage {
    UiMessage {
        element_id,
        element,
        message: MSG_BUTTON_RELEASE_ON,
        data: 0,
        item: None,
    }
}

fn presentation() -> Presentation {
    Presentation {
        pattern_revision: 1,
        match_revision: 1,
        pointer_revision: 0,
        pointer_target: None,
        patterns: vec![PresentedPattern {
            expression: "rabbit".into(),
            display: "rabbit".into(),
            exact: false,
        }]
        .into(),
        matches: vec![PresentedMatch {
            object: 7,
            name: "White Rabbit".into(),
            distance_decimetres: 118,
            coordinates: Some("43.0N, 11.7E".into()),
        }]
        .into(),
    }
}

#[test]
fn the_eyeball_toggles_the_window_and_the_window_makes_requests() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut tracker = hud.tracker.take().expect("the Tracker interface");
    let button = tracker.button().expect("the eyeball on the radar");
    assert_eq!(tree.get(button).unwrap().rect().width(), 13);
    assert!(!tracker.is_shown());

    // The eyeball opens and closes the window.
    assert_eq!(
        tracker.message(tree, &release(Some(button), BUTTON_ID)),
        (true, None)
    );
    assert!(tracker.is_shown());
    assert_eq!(
        tracker.message(tree, &release(Some(button), BUTTON_ID)),
        (true, None)
    );
    assert!(!tracker.is_shown());
    tracker.show(tree, true);

    // A press on the match's name picks it; on the pattern's `[x]` removes the pattern.
    tracker.present(tree, &presentation());
    let text = tracker.text(tree).expect("the list");
    let rect = tree.screen_box(text);
    let line = line_height(tree, text);
    let indent = margin_left(tree, text) + text_width(tree, text, "  ");
    let press_at = |tracker: &mut TrackerUi, tree: &mut UiTree, row: i32| {
        tree.start_input(crate::UI_POINTER_X, (rect.x0 + indent + 2) as f32);
        tree.start_input(
            crate::UI_POINTER_Y,
            (rect.y0 + row * line + line / 2) as f32,
        );
        let press = UiMessage {
            message: MSG_MOUSE_PRESS_TOP,
            ..release(Some(text), tokenize("BookClientAreaText"))
        };
        tracker.message(tree, &press).1
    };
    // Rows: "Tracking", the pattern, a blank, "Found", the match.
    assert_eq!(
        press_at(&mut tracker, tree, 4),
        Some(TrackerRequest::Select(7))
    );
    assert_eq!(
        press_at(&mut tracker, tree, 1),
        Some(TrackerRequest::RemovePattern(0))
    );

    // Add takes the typed pattern and clears the field; Add selected needs no text.
    let entry = tracker.entry.expect("the pattern field");
    tree.set_text(entry, "Gold Ore");
    let (add, _) = tracker.buttons[0];
    assert_eq!(
        tracker.message(tree, &release(Some(add), 0)).1,
        Some(TrackerRequest::Add {
            pattern: "Gold Ore".into(),
            exact: false
        })
    );
    assert_eq!(tree.get(entry).unwrap().text().as_deref(), Some(""));
    let (selected, _) = tracker.buttons[2];
    assert_eq!(
        tracker.message(tree, &release(Some(selected), 0)).1,
        Some(TrackerRequest::AddSelected)
    );

    // The close button hides the window.
    let window = tracker.window().unwrap();
    let close = tree.child_recursive(window, tokenize("BookCloseButton"));
    tracker.message(tree, &release(close, tokenize("BookCloseButton")));
    assert!(!tracker.is_shown());
}

#[test]
fn the_floating_arrow_turns_to_its_target_and_hides_without_one() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut arrow = hud.guide_arrow.take().expect("the floating arrow");
    let view = |relative_degrees: f32| crate::guide_arrow::GuideArrowView {
        relative_degrees,
        label: "43.0N, 11.7E · 118 m".into(),
    };
    // Straight ahead points up (the indicator's 90), to the right is 0, behind is 270.
    for (relative, rotation) in [(0.0, 90), (90.0, 0), (180.0, 270), (-90.0, 180)] {
        arrow.update(tree, Some(&view(relative)));
        let element = tree.get(arrow.arrow()).unwrap();
        assert_eq!(element.rotation, rotation, "{relative}");
        assert!(element.shown);
    }
    arrow.update(tree, None);
    assert!(!tree.get(arrow.arrow()).unwrap().shown);
}

#[test]
fn a_coordinate_in_read_only_text_is_found_under_a_press() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    // The Tracker window's list is a read-only text, as the chat log is.
    let tracker = hud.tracker.take().unwrap();
    let text = tracker.text(tree).unwrap();
    let line = "[Tracker] Found White Rabbit at 43.0N, 11.7E";
    tree.set_text(text, line);
    let on = line.find("11.7E").unwrap();
    assert_eq!(tree.coordinate_at(text, on), Some((43.0, 11.7)));
    assert_eq!(tree.coordinate_at(text, line.find("Rabbit").unwrap()), None);
}
