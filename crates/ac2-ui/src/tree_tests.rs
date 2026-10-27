//! `UIElement_Tree` on a hand-built layout, and `SurfaceWindow::DrawLine`.

use std::collections::HashMap;
use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, STATELESS};
use crate::tree_view::draw_line;
use crate::{UiBox, UiTree};

const FIELD: u32 = 0x4DA7_2ECC;

fn element(id: &str, kind: u32, r: Rect, attrs: Vec<(&str, AttributeValue)>) -> ElementDesc {
    ElementDesc {
        id: tokenize(id),
        kind,
        default_state: STATELESS,
        rect: r,
        z_level: 0,
        edges: [0; 4],
        states: vec![StateDesc {
            id: STATELESS,
            flags: 0,
            rect: Rect {
                x0: -1,
                y0: -1,
                x1: -1,
                y1: -1,
            },
            z_level: 0,
            attributes: attrs
                .into_iter()
                .map(|(name, value)| AttributeDesc {
                    id: tokenize(name),
                    value,
                })
                .collect(),
            media: Vec::new(),
            transitions: Vec::new(),
            children: Vec::new(),
        }],
    }
}

fn rect(x0: i16, y0: i16, x1: i16, y1: i16) -> Rect {
    Rect { x0, y0, x1, y1 }
}

/// A 100 x 100 tree of 20 x 20 cells and a 10 x 10 node template.
fn layout() -> Arc<LayoutDesc> {
    let tree = element(
        "Tree",
        tokenize("tree"),
        rect(0, 0, 99, 99),
        vec![
            ("grid", AttributeValue::Bool(true)),
            ("node_width", AttributeValue::Int(20)),
            ("node_height", AttributeValue::Int(20)),
            ("line_color_red", AttributeValue::Float(1.0)),
        ],
    );
    let node = element("Node", FIELD, rect(0, 0, 9, 9), vec![]);
    Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 100,
        height: 100,
        elements: vec![Arc::new(tree), Arc::new(node)],
    })
}

fn node(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> usize {
    let template = Arc::clone(layout.element(tokenize("Node")).unwrap());
    tree.create_element(Arc::clone(layout), template, false)
        .unwrap()
}

#[test]
fn lines_include_both_ends_whichever_axis_is_longer() {
    let width = 16;
    let pixels = |from, to| {
        let mut rgba = vec![0u8; width * width * 4];
        let clip = UiBox::new(0, 0, width as i32 - 1, width as i32 - 1);
        draw_line(&mut rgba, width, clip, from, to, [255, 0, 0, 255]);
        let mut set: Vec<(usize, usize)> = (0..width * width)
            .filter(|i| rgba[i * 4] == 255)
            .map(|i| (i % width, i / width))
            .collect();
        set.sort();
        set
    };
    assert_eq!(
        pixels((2, 3), (6, 3)),
        (2..=6).map(|x| (x, 3)).collect::<Vec<_>>()
    );
    assert_eq!(
        pixels((4, 1), (4, 5)),
        (1..=5).map(|y| (4, y)).collect::<Vec<_>>()
    );
    let steep = pixels((1, 1), (3, 9));
    assert!(
        steep.contains(&(1, 1)) && steep.contains(&(3, 9)),
        "{steep:?}"
    );
    assert_eq!(steep.len(), 9, "one pixel a row: {steep:?}");
    let shallow = pixels((9, 2), (1, 5));
    assert!(
        shallow.contains(&(9, 2)) && shallow.contains(&(1, 5)),
        "{shallow:?}"
    );
    assert_eq!(shallow.len(), 9, "one pixel a column: {shallow:?}");
}

#[test]
fn a_vertical_grid_tree_grows_up_and_links_parents_to_children() {
    let layout = layout();
    let mut tree = UiTree::new(200, 200);
    let root = tree
        .create_root(Arc::clone(&layout), tokenize("Tree"))
        .unwrap();
    tree.tree_multi_op_start(root);
    let base = node(&mut tree, &layout);
    let left = node(&mut tree, &layout);
    let right = node(&mut tree, &layout);
    assert!(tree.tree_add_item(root, base, 0, 0));
    assert!(tree.tree_add_item(root, left, 0, 1));
    assert!(tree.tree_add_item(root, right, 1, 1));
    // A taken cell keeps its item.
    let extra = node(&mut tree, &layout);
    tree.tree_add_item(root, extra, 1, 1);
    assert_eq!(tree.tree_item(root, 1, 1), Some(right));
    assert!(tree.tree_set_items_parent(root, left, 0, 0));
    assert!(tree.tree_set_items_parent(root, right, 0, 0));
    tree.tree_multi_op_complete(root);
    // Row 0 sits on the bottom of the 100-pixel paper.
    let at = |tree: &UiTree, i: usize| {
        let r = tree.get(i).unwrap().rect;
        (r.x0, r.y0)
    };
    assert_eq!(at(&tree, base), (0, 80));
    assert_eq!(at(&tree, left), (0, 60));
    assert_eq!(at(&tree, right), (20, 60));
    // Lines run from the parent's top middle to each child's bottom middle, in red.
    let surfaces = tree.compose(&HashMap::new());
    let s = &surfaces[0];
    let px = |x: usize, y: usize| s.rgba[(y * s.width as usize + x) * 4..][..4].to_vec();
    assert_eq!(px(4, 75), vec![255, 0, 0, 255]);
    assert_eq!(px(4, 80), vec![255, 0, 0, 255]);
    assert_eq!(px(4, 90), vec![0, 0, 0, 0]);
    // Flushing removes the items.
    tree.tree_flush(root);
    assert_eq!(tree.tree_item(root, 0, 0), None);
    assert!(tree.get(base).is_none());
}
