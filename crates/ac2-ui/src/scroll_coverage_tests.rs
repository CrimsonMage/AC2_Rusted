//! The mouse wheel over every scroll bar owner in every UI layout of the DAT (`AC2_DAT_DIR`).
//!
//! Each top-level element of each layout is made as a root (`UiTree::create_root`). A text with a
//! vertical bar, a list box (`listbox`, `listboxmsize`) and a tree with a bar are given content
//! that overflows them; a wheel turn over each must move its view, and a turn over a bar must move
//! the bar or what it drives. Not tested, and counted: an element that is hidden; one whose bar is
//! not in its own root (`unlinked`); one under something on top of it everywhere (`covered`); a
//! bar that is disabled, has one stop, or drives nothing (`inert`). A wheel over a child that does
//! not scroll must reach its scrollable parent (`bubbled`, tested in `list_scroll_tests.rs`).

use std::collections::BTreeMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{test_support::env_dataset, DatFileKind};
use ac2_db::ResourceStore;

use crate::hud::{build_gameplay_hud, GAMEPLAY_ROOTS};
use crate::layout::{load_layout, AttributeValue, LayoutDesc, LAYOUT_FIRST, LAYOUT_LAST};
use crate::list_box_select::is_list_kind;
use crate::text_data::StoreTextData;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};
use crate::{UiTree, UI_POINTER_X, UI_POINTER_Y, UI_WHEEL_IN, UI_WHEEL_OUT};

/// The points of an element, as fractions of its box, that a wheel test tries in turn.
const CANDIDATES: [(f32, f32); 13] = [
    (0.5, 0.5),
    (0.25, 0.25),
    (0.75, 0.25),
    (0.25, 0.75),
    (0.75, 0.75),
    (0.5, 0.2),
    (0.5, 0.8),
    (0.2, 0.5),
    (0.8, 0.5),
    (0.1, 0.1),
    (0.9, 0.1),
    (0.1, 0.9),
    (0.9, 0.9),
];

/// Where a scroll bar's track shows past its thumb, as fractions of its box.
const TRACK: [(f32, f32); 6] = [
    (0.5, 0.98),
    (0.98, 0.5),
    (0.02, 0.5),
    (0.5, 0.02),
    (0.5, 0.95),
    (0.95, 0.5),
];

/// The kinds of element the wheel scrolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Scroller {
    Text,
    List,
    Tree,
    Bar,
}

/// What the layouts held and what the wheel reached, per kind.
#[derive(Debug, Default)]
struct Counts {
    found: usize,
    tested: usize,
    /// Under no pointer position, because the element or an ancestor is hidden.
    hidden: usize,
    /// A bar that drives nothing and has no stop locations: a turn has nothing to move.
    inert: usize,
    /// A wheel over a non-scrolling child that reached the scrollable parent.
    bubbled: usize,
    /// Shown, but every point of its box is under an element on top of it.
    covered: usize,
    /// Names a scroll bar that is not in its own root (`GetScrollbar` looks it up across the UI
    /// unless `unique_msg`, which is not modelled: `text_scroll.rs`), so no bar drives it here.
    unlinked: usize,
}

fn classify(tree: &UiTree, index: usize) -> Option<Scroller> {
    let el = tree.get(index)?;
    let kind = el.kind();
    let has = |name: &str| el.attribute(tokenize(name)).is_some();
    if kind == tokenize("text") && has("vertical_scrollbar") {
        Some(Scroller::Text)
    } else if is_list_kind(kind) && (has("vertical_scrollbar") || has("horizontal_scrollbar")) {
        Some(Scroller::List)
    } else if kind == tokenize("tree") && has("vertical_scrollbar") {
        Some(Scroller::Tree)
    } else if kind == tokenize("scrollbar") {
        Some(Scroller::Bar)
    } else {
        None
    }
}

/// The view a turn of the wheel moves, or the bar's position with what it drives.
fn measure(tree: &UiTree, index: usize, kind: Scroller) -> f64 {
    let view = |i: usize| -> f64 {
        let el = tree.get(i);
        let (x, y) = match kind {
            Scroller::Text => el.and_then(|e| e.text.as_ref()).map_or((0, 0), |t| t.view),
            Scroller::List => el.and_then(|e| e.list.as_ref()).map_or((0, 0), |l| l.view),
            Scroller::Tree => el.and_then(|e| e.tree.as_ref()).map_or((0, 0), |t| t.view),
            Scroller::Bar => (0, 0),
        };
        f64::from(x + y)
    };
    match kind {
        Scroller::Bar => {
            let mut total = f64::from(tree.scrollbar_position(index));
            for text in tree.texts_bound_to(index) {
                total += view(text);
            }
            for (list, _) in tree.lists_bound_to(index) {
                total += view(list);
            }
            for node in tree.trees_bound_to(index) {
                total += view(node);
            }
            total
        }
        _ => view(index),
    }
}

/// Whether the bar `index` names (`vertical_scrollbar` or `horizontal_scrollbar`) is in its root.
fn linked(tree: &UiTree, index: usize) -> bool {
    let root = tree.root_of(index);
    ["vertical_scrollbar", "horizontal_scrollbar"]
        .iter()
        .any(|name| {
            matches!(
                tree.get(index).and_then(|e| e.attribute(tokenize(name))),
                Some(AttributeValue::Id(id)) if tree.child_recursive(root, *id).is_some()
            )
        })
}

/// Makes `index` overflow its view: long text, or a paper three times the box.
fn overflow(tree: &mut UiTree, index: usize, kind: Scroller) {
    match kind {
        Scroller::Text => {
            let lines: String = (0..200).map(|n| format!("line {n}\n")).collect();
            tree.set_text(index, &lines);
        }
        Scroller::List => {
            let vertical = tree
                .get(index)
                .is_some_and(|e| e.attribute(tokenize("vertical_scrollbar")).is_some());
            let height = tree.get(index).map_or(0, |e| e.rect().height());
            let width = tree.get(index).map_or(0, |e| e.rect().width());
            // A list makes its state when it is first laid out; the test makes it here.
            let state = tree.list(index);
            if vertical {
                state.item.1 = state.item.1.max(10);
                state.paper.1 = height * 3;
            } else {
                state.item.0 = state.item.0.max(10);
                state.paper.0 = width * 3;
            }
            // The bar takes the view's position (`UpdateControls`).
            tree.list_update_controls(index);
        }
        Scroller::Tree => {
            let height = tree.get(index).map_or(0, |e| e.rect().height());
            tree.el_mut(index)
                .attributes
                .insert(tokenize("node_height"), AttributeValue::Int(12));
            tree.tree_state(index).paper = (1, height * 3);
            tree.tree_update_scrollbar(index);
        }
        Scroller::Bar => {}
    }
}

/// Whether `element` is `ancestor` or one of its descendants.
fn within(tree: &UiTree, mut element: usize, ancestor: usize) -> bool {
    loop {
        if element == ancestor {
            return true;
        }
        match tree.get(element).and_then(|e| e.parent_index()) {
            Some(parent) => element = parent,
            None => return false,
        }
    }
}

/// Puts the pointer where `index` is the top element (or inside it), so a wheel turn reaches it.
/// A bar is tried on its track only (`exact`): its thumb is a child that takes the pointer over
/// most of the track. Reports whether such a point exists in its box.
fn hit(tree: &mut UiTree, index: usize, exact: bool) -> bool {
    let rect = tree.screen_box(index);
    let (width, height) = (rect.width() as f32, rect.height() as f32);
    let points: &[(f32, f32)] = if exact { &TRACK } else { &CANDIDATES };
    for &(fx, fy) in points {
        let x = rect.x0 as f32 + width * fx;
        let y = rect.y0 as f32 + height * fy;
        tree.start_input(UI_POINTER_X, x.floor());
        tree.start_input(UI_POINTER_Y, y.floor());
        let top = tree.mouse_top();
        let reached = if exact {
            top == Some(index)
        } else {
            top.is_some_and(|top| within(tree, top, index))
        };
        if reached {
            return true;
        }
    }
    false
}

/// Whether `index` and each of its ancestors is visible.
fn shown(tree: &UiTree, mut index: usize) -> bool {
    loop {
        match tree.get(index) {
            Some(el) if !el.visible() => return false,
            Some(_) => {}
            None => return false,
        }
        match tree.get(index).and_then(|e| e.parent_index()) {
            Some(parent) => index = parent,
            None => return true,
        }
    }
}

/// Turns the wheel over `index`. A turn out scrolls down; one already at the bottom of its range
/// cannot go further, so the turn must then move it back up. Either way the position moves.
fn wheel_moves(tree: &mut UiTree, index: usize, kind: Scroller, what: &str) {
    let before = measure(tree, index, kind);
    tree.start_input(UI_WHEEL_OUT, 1.0);
    let down = measure(tree, index, kind);
    if down != before {
        return;
    }
    tree.start_input(UI_WHEEL_IN, 1.0);
    let up = measure(tree, index, kind);
    assert!(
        up != down,
        "{what} (element {index}, {kind:?}): a wheel turn either way left {before} unchanged"
    );
}

/// Tests the wheel over each scroll bar owner of one root's tree.
fn scan_tree(tree: &mut UiTree, layout_did: u32, counts: &mut BTreeMap<Scroller, Counts>) {
    let indices: Vec<usize> = (0..tree.slots_len())
        .filter(|&i| tree.get(i).is_some())
        .collect();
    for index in indices {
        let Some(kind) = classify(tree, index) else {
            continue;
        };
        let count = counts.entry(kind).or_default();
        count.found += 1;
        let label = format!("layout {layout_did:08x}");
        if !shown(tree, index) {
            count.hidden += 1;
            continue;
        }
        if kind != Scroller::Bar && !linked(tree, index) {
            count.unlinked += 1;
            continue;
        }
        if kind == Scroller::Bar {
            // A bar's turn moves what it drives: give that content overflow first.
            let texts = tree.texts_bound_to(index);
            let lists = tree.lists_bound_to(index);
            let trees = tree.trees_bound_to(index);
            for text in texts {
                overflow(tree, text, Scroller::Text);
            }
            for (list, _) in lists {
                overflow(tree, list, Scroller::List);
            }
            for node in trees {
                overflow(tree, node, Scroller::Tree);
            }
            let drives = !tree.texts_bound_to(index).is_empty()
                || !tree.lists_bound_to(index).is_empty()
                || !tree.trees_bound_to(index).is_empty();
            // A disabled bar takes no wheel (`UIElement_Scrollbar::MouseDown`).
            let disabled = tree.bool_attribute(index, tokenize("disabled"));
            // A bar of one stop (or none) has nowhere to step (`HandleMoveSteps`).
            let stops = match tree
                .get(index)
                .and_then(|e| e.attribute(tokenize("num_stop_locations")))
            {
                Some(AttributeValue::Int(n)) => *n,
                _ => 0,
            };
            let stepless = tree.bool_attribute(index, tokenize("stop_locations")) && stops <= 1;
            if disabled
                || stepless
                || (!drives && !tree.bool_attribute(index, tokenize("stop_locations")))
            {
                count.inert += 1;
                continue;
            }
        } else {
            overflow(tree, index, kind);
        }
        if !hit(tree, index, kind == Scroller::Bar) {
            // Every point of its box is under something on top of it: no wheel turn reaches it.
            count.covered += 1;
            continue;
        }
        wheel_moves(tree, index, kind, &label);
        count.tested += 1;

        // A child that does not scroll passes the wheel to the scrollable element above it.
        if kind != Scroller::Bar {
            let children: Vec<usize> = tree.children(index).to_vec();
            for child in children {
                if classify(tree, child).is_some() || !shown(tree, child) {
                    continue;
                }
                if hit(tree, child, false) && tree.get(index).is_some_and(|e| e.mouse_over()) {
                    wheel_moves(tree, index, kind, &format!("{label}, over child {child}"));
                    count.bubbled += 1;
                    break;
                }
            }
        }
    }
}

#[test]
fn the_wheel_scrolls_every_scroll_bar_owner_in_every_ui_layout() {
    let Some(dataset) = env_dataset() else {
        return;
    };
    let dids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Local, LAYOUT_FIRST, LAYOUT_LAST)
        .collect();
    assert!(!dids.is_empty(), "the DAT holds UI layouts");
    let store = Arc::new(ResourceStore::new(dataset));
    let mut counts: BTreeMap<Scroller, Counts> = BTreeMap::new();
    for did in dids {
        let Some(layout) = load_layout(&store, DataId::new(did)).unwrap() else {
            continue;
        };
        let layout: Arc<LayoutDesc> = Arc::new((*layout).clone());
        for top in layout.elements.clone() {
            let mut tree = UiTree::new(1024, 768);
            // The fonts the texts lay out in come from the store, as the HUD gives them.
            tree.set_text_data(Arc::new(StoreTextData::new(Arc::clone(&store))));
            if tree.create_root(Arc::clone(&layout), top.id).is_none() {
                continue;
            }
            scan_tree(&mut tree, did, &mut counts);
        }
    }
    for (kind, count) in &counts {
        eprintln!("{kind:?}: {count:?}");
    }
    for kind in [
        Scroller::Text,
        Scroller::List,
        Scroller::Tree,
        Scroller::Bar,
    ] {
        let count = counts.get(&kind).map_or(0, |c| c.tested);
        assert!(count > 0, "no {kind:?} in any layout was wheeled");
    }
}

#[test]
fn the_wheel_over_no_ui_reaches_the_world_in_the_gameplay_hud() {
    let Some(dataset) = env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let world_name = tokenize(GAMEPLAY_ROOTS[2].1);
    let world = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).is_some_and(|e| e.id() == world_name))
        .expect("the item selection field (the world) is a root");
    // Find a point where nothing but the world is under the pointer.
    let mut found = None;
    'search: for y in (0..768).step_by(16) {
        for x in (0..1024).step_by(16) {
            tree.start_input(UI_POINTER_X, x as f32);
            tree.start_input(UI_POINTER_Y, y as f32);
            if tree.mouse_top() == Some(world) {
                found = Some((x, y));
                break 'search;
            }
        }
    }
    let (x, y) = found.expect("some point of the HUD shows the world");
    tree.take_messages();
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
    tree.start_input(UI_WHEEL_OUT, 1.0);
    let messages = tree.take_messages();
    assert!(
        messages.iter().any(|m| m.element == Some(world)
            && m.message == MSG_MOUSE_PRESS_TOP
            && m.data == UI_WHEEL_OUT as i32),
        "a wheel over the world reaches the item selection field (camera zoom)"
    );
}
