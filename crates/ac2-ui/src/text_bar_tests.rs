//! `UIElement_Text::UpdateScrollbar` and `DetermineAndSetNewPage` on a hand-built layout.

use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, LayoutDesc, Rect};
use crate::text_tests::{element, flag, Data};
use crate::token::{tokenize, TEXT};
use crate::UiTree;

fn rect(x0: i16, y0: i16, x1: i16, y1: i16) -> Rect {
    Rect { x0, y0, x1, y1 }
}

/// A 10 pixel high text bound to a proportional `hide_when_disabled` bar with a widget.
fn panel() -> (UiTree, usize, usize, usize) {
    let text = element(
        "Text",
        TEXT,
        rect(0, 0, 49, 9),
        vec![AttributeDesc {
            id: tokenize("vertical_scrollbar"),
            value: AttributeValue::Id(tokenize("Bar")),
        }],
        vec![],
    );
    let widget = element("widget", 0x4DA7_2ECC, rect(0, 0, 9, 4), vec![], vec![]);
    let bar = element(
        "Bar",
        tokenize("scrollbar"),
        rect(60, 0, 69, 9),
        vec![flag("proportional"), flag("hide_when_disabled")],
        vec![widget],
    );
    let panel = element(
        "Panel",
        0x4DA7_2ECC,
        rect(0, 0, 99, 49),
        vec![],
        vec![text, bar],
    );
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 100,
        height: 50,
        elements: vec![Arc::new(panel)],
    });
    let mut tree = UiTree::new(100, 50);
    tree.set_text_data(Arc::new(Data));
    let root = tree.create_root(layout, tokenize("Panel")).unwrap();
    let text = tree.child_recursive(root, tokenize("Text")).unwrap();
    let bar = tree.child_recursive(root, tokenize("Bar")).unwrap();
    let widget = tree.child_recursive(bar, tokenize("widget")).unwrap();
    (tree, text, bar, widget)
}

#[test]
fn a_text_that_fits_disables_and_hides_its_bar() {
    let (mut tree, text, bar, _) = panel();
    tree.set_text(text, "a");
    tree.text_update_scrollbar(text);
    assert!(tree.bool_attribute(bar, tokenize("disabled")));
    assert!(!tree.get(bar).unwrap().visible());
}

#[test]
fn a_long_text_sizes_the_widget_and_pages_by_its_height() {
    let (mut tree, text, bar, widget) = panel();
    let long: Vec<&str> = std::iter::repeat_n("a", 40).collect();
    tree.set_text(text, &long.join("\n"));
    tree.text_update_scrollbar(text);
    assert!(!tree.bool_attribute(bar, tokenize("disabled")));
    assert!(tree.get(bar).unwrap().visible());
    let thumb = tree.get(widget).unwrap().rect().height();
    assert!(
        thumb < 10,
        "the widget is the shown share of the track: {thumb}"
    );
    tree.text_page_step(text, true);
    let view = tree.get(text).unwrap().text.as_ref().unwrap().view.1;
    assert_eq!(view, 10, "one page is the element's height");
}
