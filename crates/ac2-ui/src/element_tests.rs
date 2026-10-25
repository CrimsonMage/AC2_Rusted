//! The element tree and compositor on hand-built layouts.

use std::collections::HashMap;
use std::sync::Arc;

use crate::layout::{
    AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc, Rect, StateDesc,
};
use crate::token::{tokenize, REFERENCE, REFERENT, STATELESS};
use crate::{UiBox, UiPixels, UiTree};

const FIELD: u32 = 0x4DA7_2ECC;

fn rect(x0: i16, y0: i16, x1: i16, y1: i16) -> Rect {
    Rect { x0, y0, x1, y1 }
}

fn state(
    attributes: Vec<AttributeDesc>,
    media: Vec<MediaDesc>,
    children: Vec<ElementDesc>,
) -> StateDesc {
    StateDesc {
        id: STATELESS,
        flags: 0,
        rect: rect(-1, -1, -1, -1),
        z_level: 0,
        attributes,
        media,
        transitions: Vec::new(),
        children: children.into_iter().map(Arc::new).collect(),
    }
}

fn element(id: &str, r: Rect, z: u16, edges: [u8; 4], state: StateDesc) -> ElementDesc {
    ElementDesc {
        id: tokenize(id),
        kind: FIELD,
        default_state: STATELESS,
        rect: r,
        z_level: z,
        edges,
        states: vec![state],
    }
}

fn image(file: u32, draw_mode: u32) -> MediaDesc {
    MediaDesc::Image {
        file,
        draw_mode,
        offset: (0, 0),
    }
}

fn layout(elements: Vec<ElementDesc>) -> Arc<LayoutDesc> {
    Arc::new(LayoutDesc {
        sub_ids: Vec::new(),
        string_table: 0,
        width: 100,
        height: 50,
        elements: elements.into_iter().map(Arc::new).collect(),
    })
}

/// A root that stretches with the display, a child anchored to its bottom-right corner and a
/// child centred in it.
fn anchored() -> Arc<LayoutDesc> {
    let corner = element(
        "corner",
        rect(80, 30, 89, 39),
        0,
        [2, 2, 1, 1],
        state(vec![], vec![], vec![]),
    );
    let centre = element(
        "centre",
        rect(40, 20, 59, 29),
        0,
        [3, 3, 3, 3],
        state(vec![], vec![], vec![]),
    );
    let root = element(
        "root",
        rect(0, 0, 99, 49),
        0,
        [1, 1, 1, 1],
        state(vec![], vec![], vec![corner, centre]),
    );
    layout(vec![root])
}

#[test]
fn edges_follow_the_parents_change_in_size() {
    let mut tree = UiTree::new(100, 50);
    let root = tree.create_root(anchored(), tokenize("root")).unwrap();
    let corner = tree.child_recursive(root, tokenize("corner")).unwrap();
    let centre = tree.child_recursive(root, tokenize("centre")).unwrap();
    assert_eq!(tree.screen_box(root), UiBox::new(0, 0, 99, 49));
    assert_eq!(tree.screen_box(corner), UiBox::new(80, 30, 89, 39));
    assert_eq!(tree.screen_box(centre), UiBox::new(40, 20, 59, 29));

    tree.set_display(200, 100);
    assert_eq!(tree.screen_box(root), UiBox::new(0, 0, 199, 99));
    assert_eq!(tree.screen_box(corner), UiBox::new(180, 80, 189, 89));
    assert_eq!(tree.screen_box(centre), UiBox::new(90, 45, 109, 54));
}

#[test]
fn a_reference_takes_the_referents_class_and_children() {
    let leaf = element(
        "leaf",
        rect(1, 1, 4, 4),
        0,
        [0; 4],
        state(vec![], vec![], vec![]),
    );
    let template = element(
        "template",
        rect(0, 0, 9, 9),
        0,
        [0; 4],
        state(vec![], vec![], vec![leaf]),
    );
    let referent = AttributeDesc {
        id: REFERENT,
        value: AttributeValue::Id(tokenize("template")),
    };
    let mut button = element(
        "button",
        rect(20, 0, 29, 9),
        0,
        [0; 4],
        state(vec![referent], vec![], vec![]),
    );
    button.kind = REFERENCE;
    let root = element(
        "root",
        rect(0, 0, 99, 49),
        0,
        [0; 4],
        state(vec![], vec![], vec![button]),
    );
    let mut tree = UiTree::new(100, 50);
    let root = tree
        .create_root(layout(vec![root, template]), tokenize("root"))
        .unwrap();
    let button = tree.child_recursive(root, tokenize("button")).unwrap();
    assert_eq!(tree.get(button).unwrap().kind(), FIELD);
    assert_eq!(tree.screen_box(button), UiBox::new(20, 0, 29, 9));
    let leaf = tree.child_recursive(button, tokenize("leaf")).unwrap();
    assert_eq!(tree.screen_box(leaf), UiBox::new(21, 1, 24, 4));
}

fn solid(width: u32, height: u32, rgba: [u8; 4]) -> UiPixels {
    UiPixels {
        width,
        height,
        rgba: rgba.repeat((width * height) as usize).into(),
    }
}

#[test]
fn lower_z_draws_on_top_and_overlays_blend_by_alpha() {
    let under = element(
        "under",
        rect(0, 0, 3, 3),
        5,
        [0; 4],
        state(vec![], vec![image(2, 0)], vec![]),
    );
    let over = element(
        "over",
        rect(2, 0, 5, 3),
        1,
        [0; 4],
        state(vec![], vec![image(3, 2)], vec![]),
    );
    let root = element(
        "root",
        rect(0, 0, 7, 3),
        0,
        [0; 4],
        state(vec![], vec![image(1, 0)], vec![under, over]),
    );
    let mut tree = UiTree::new(100, 50);
    let root = tree
        .create_root(layout(vec![root]), tokenize("root"))
        .unwrap();
    let images = HashMap::from([
        (1, solid(8, 4, [0, 0, 0, 0])),
        (2, solid(4, 4, [200, 0, 0, 255])),
        (3, solid(4, 4, [0, 0, 255, 128])),
    ]);
    let surfaces = tree.compose(&images);
    assert_eq!(surfaces.len(), 1);
    let pixel = |x: usize| &surfaces[0].rgba[x * 4..x * 4 + 4];
    // Mode 0 copies, alpha included.
    assert_eq!(pixel(0), [200, 0, 0, 255]);
    assert_eq!(pixel(6), [0, 0, 0, 0]);
    // Mode 2 blends the colour by the source alpha and keeps the surface's alpha:
    // 200 - 200 * 128 / 256 = 100, 0 - (0 - 255) * 128 / 256 = 127.
    assert_eq!(pixel(2), [100, 0, 127, 255]);
    assert_eq!(pixel(4), [0, 0, 127, 0]);
    assert_eq!(tree.children(root).len(), 2);
}

#[test]
fn mode_zero_tiles_from_the_elements_corner() {
    let root = element(
        "root",
        rect(0, 0, 4, 0),
        0,
        [0; 4],
        state(vec![], vec![image(1, 0)], vec![]),
    );
    let mut tree = UiTree::new(100, 50);
    tree.create_root(layout(vec![root]), tokenize("root"))
        .unwrap();
    let images = HashMap::from([(
        1,
        UiPixels {
            width: 2,
            height: 1,
            rgba: [[1, 0, 0, 255], [2, 0, 0, 255]].concat().into(),
        },
    )]);
    let surface = &tree.compose(&images)[0];
    let reds: Vec<u8> = surface.rgba.chunks_exact(4).map(|p| p[0]).collect();
    assert_eq!(reds, [1, 2, 1, 2, 1]);
}
