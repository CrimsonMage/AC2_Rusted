//! The mouse wheel over every scroll bar owner in every UI layout of the DAT (`AC2_DAT_DIR`).
//!
//! Each owner (a text with a vertical bar, a list box, a tree with a bar, a bar) is tested in a
//! fresh UI made the way the HUD makes one: the owner's layout, with every top element a root
//! (`UiTree::create_root`), and the layouts it shares a scroll bar with (one defines a bar the
//! other names), since `GetScrollbar` finds a bar in any root. The owner is opened the way a
//! player would: a hidden owner has its hidden ancestors shown (`opened`); one under something on
//! top has its ancestors brought to the front (`raised`), then the wheel is turned over it.
//! Content is filled first: a text or tree gets overflowing paper, a list with no items gets
//! items, and a bar's own content is overflowed and the bar's disabled state refreshed as the
//! owner does (`UIElement_Text::UpdateScrollbar`). A turn over a bar is tried over its thumb as
//! well as its track; a thumb or arrow passes the wheel to its bar.
//!
//! Counted, not tested: `covered` (still under something after it is raised); and the bars that
//! drive nothing (`inert`): `shadowed` (a same-id bar is the one `GetScrollbar` finds, see
//! `UIElementManager::GetElement`), `unbound` (named, yet no owner in its UI resolves to it), and
//! `unnamed` (no layout of the DAT names it, so it has no owner in the layouts; such an owner, if
//! any, is made by code). A wheel over a child that does not scroll must reach its scrollable
//! parent (`bubbled`, tested in `list_scroll_tests.rs`).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{test_support::env_dataset, DatFileKind};
use ac2_db::ResourceStore;

use crate::hud::{build_gameplay_hud, GAMEPLAY_ROOTS};
use crate::layout::{
    load_layout, AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc,
    LAYOUT_FIRST, LAYOUT_LAST,
};
use crate::list_box_select::is_list_kind;
use crate::scrollbar::SCROLLBAR;
use crate::text_data::StoreTextData;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP, STATELESS};
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
    /// Hidden with an ancestor: shown as a player opens it, then tested.
    opened: usize,
    /// Under something on top everywhere: its root brought to the top, then tested.
    raised: usize,
    /// A bar a turn has nothing to move on: `shadowed` + `unbound` + `unnamed`.
    inert: usize,
    /// Named by a layout, but another bar with the same id is the one `GetScrollbar` finds
    /// (`UIElementManager::GetElement` returns one element per id), so this one drives nothing.
    shadowed: usize,
    /// Named by a layout, yet drives nothing in the UI (no owner resolves to it).
    unbound: usize,
    /// No layout of the DAT names it: no owner exists for it in the layouts.
    unnamed: usize,
    /// A wheel over a non-scrolling child that reached the scrollable parent.
    bubbled: usize,
    /// Still under something on top after its root is raised: no wheel turn reaches it.
    covered: usize,
    /// Names no scroll bar anywhere in the UI (`GetScrollbar` finds one in any root).
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

/// Whether the owner `index` names a scroll bar that exists somewhere in the UI (`GetScrollbar`).
fn linked(tree: &UiTree, index: usize) -> bool {
    ["vertical_scrollbar", "horizontal_scrollbar"]
        .iter()
        .any(|name| tree.named_scrollbar(index, name).is_some())
}

/// An item element of `kind` `field`, `width` by `height` pixels, as the list's own template.
fn item_desc(width: i16, height: i16) -> Arc<ElementDesc> {
    let rect = Rect {
        x0: 0,
        y0: 0,
        x1: width - 1,
        y1: height - 1,
    };
    Arc::new(ElementDesc {
        id: tokenize("ScrollTestItem"),
        kind: tokenize("field"),
        default_state: STATELESS,
        rect,
        z_level: 0,
        edges: [0; 4],
        states: vec![StateDesc {
            id: STATELESS,
            flags: 0,
            rect,
            z_level: 0,
            attributes: Vec::<AttributeDesc>::new(),
            media: Vec::new(),
            transitions: Vec::new(),
            children: Vec::new(),
        }],
    })
}

/// Adds items to an empty list, two hundred of them 20 pixels across, so its paper overflows its
/// view in the list's columns (`UIElement_ListBox::UpdateControls`).
fn fill_list(tree: &mut UiTree, layout: &Arc<LayoutDesc>, list: usize, vertical: bool) {
    let (width, height) = tree.get(list).map_or((0, 0), |e| {
        (e.rect().width() as i16, e.rect().height() as i16)
    });
    let desc = if vertical {
        item_desc(width.max(1), 20)
    } else {
        item_desc(20, height.max(1))
    };
    let items: Vec<usize> = (0..200)
        .filter_map(|_| tree.create_element(Arc::clone(layout), Arc::clone(&desc), false))
        .collect();
    tree.list_multi_op_start(list);
    for item in items {
        tree.list_add_item(list, item);
    }
    tree.list_multi_op_complete(list);
}

/// Makes `index` overflow its view: long text, or a paper three times the box.
fn overflow(tree: &mut UiTree, layout: &Arc<LayoutDesc>, index: usize, kind: Scroller) {
    match kind {
        Scroller::Text => {
            let lines: String = (0..200).map(|n| format!("line {n}\n")).collect();
            tree.set_text(index, &lines);
            // The text refreshes its bar as its paper grows (`UIElement_Text::UpdateScrollbar`).
            tree.text_update_scrollbar(index);
        }
        Scroller::List => {
            let vertical = tree
                .get(index)
                .is_some_and(|e| e.attribute(tokenize("vertical_scrollbar")).is_some());
            let height = tree.get(index).map_or(0, |e| e.rect().height());
            let width = tree.get(index).map_or(0, |e| e.rect().width());
            // A list holds the items the game adds: content, not a paper size, so it scrolls.
            if tree.list_items(index).is_empty() {
                fill_list(tree, layout, index, vertical);
            }
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
/// A bar is tried over its thumb too: the thumb is a child that the wheel must pass on to the bar
/// (`UIElement_Scrollbar::MouseDown`). Reports whether such a point exists in its box.
fn hit(tree: &mut UiTree, index: usize) -> bool {
    let rect = tree.screen_box(index);
    let (width, height) = (rect.width() as f32, rect.height() as f32);
    for &(fx, fy) in &CANDIDATES {
        let x = rect.x0 as f32 + width * fx;
        let y = rect.y0 as f32 + height * fy;
        tree.start_input(UI_POINTER_X, x.floor());
        tree.start_input(UI_POINTER_Y, y.floor());
        if tree.mouse_top().is_some_and(|top| within(tree, top, index)) {
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
    let top = tree.mouse_top().map(|t| (t, tree.get(t).map(|e| e.id())));
    assert!(
        up != down,
        "{what} (element {index}, {kind:?}): a wheel turn either way left {before} unchanged; \
         top {top:?}, disabled {}, stop_locations {}, position {:?}, texts {:?}, lists {:?}, trees {:?}",
        tree.bool_attribute(index, tokenize("disabled")),
        tree.bool_attribute(index, tokenize("stop_locations")),
        tree.get(index).and_then(|e| e.attribute(tokenize("y_position")).or(e.attribute(tokenize("x_position")))),
        tree.texts_bound_to(index).iter().map(|&t| (t, tree.get(t).and_then(|e| e.text.as_ref()).map(|x| x.view))).collect::<Vec<_>>(),
        tree.lists_bound_to(index).iter().map(|&(l, v)| (l, v, tree.get(l).map(|e| (e.rect(), e.visible(), e.mouse_visible, e.list.as_ref().map(|x| (x.view, x.item, x.paper)))))).collect::<Vec<_>>(),
        tree.trees_bound_to(index).iter().map(|&t| (t, tree.get(t).and_then(|e| e.tree.as_ref()).map(|x| x.view))).collect::<Vec<_>>()
    );
}

/// The UI the HUD makes from `layouts`: each top element of each a root, in order. Also the
/// roots of the first layout, the one under test.
fn layout_ui(store: &Arc<ResourceStore>, layouts: &[&Arc<LayoutDesc>]) -> (UiTree, Vec<usize>) {
    let mut tree = UiTree::new(1024, 768);
    // The fonts the texts lay out in come from the store, as the HUD gives them.
    tree.set_text_data(Arc::new(StoreTextData::new(Arc::clone(store))));
    let mut own = Vec::new();
    for (n, layout) in layouts.iter().enumerate() {
        for top in layout.elements.clone() {
            if let Some(root) = tree.create_root(Arc::clone(layout), top.id) {
                if n == 0 {
                    own.push(root);
                }
            }
        }
    }
    (tree, own)
}

/// The scroll bar names of one layout: the bars it defines, and the bars its elements name.
#[derive(Default)]
struct Names {
    defines: BTreeSet<u32>,
    refs: BTreeSet<u32>,
}

/// The names `layout` defines and refers to (`vertical_scrollbar` / `horizontal_scrollbar`).
fn names_of(layout: &LayoutDesc) -> Names {
    fn visit(el: &ElementDesc, names: &mut Names) {
        if el.kind == SCROLLBAR {
            names.defines.insert(el.id);
        }
        for state in &el.states {
            for attr in &state.attributes {
                let named = attr.id == tokenize("vertical_scrollbar")
                    || attr.id == tokenize("horizontal_scrollbar");
                if let (true, AttributeValue::Id(bar)) = (named, &attr.value) {
                    names.refs.insert(*bar);
                }
            }
            for child in &state.children {
                visit(child, names);
            }
        }
    }
    let mut names = Names::default();
    for top in &layout.elements {
        visit(top, &mut names);
    }
    names
}

/// One layout of the DAT, loaded, with its scroll bar names.
struct Loaded {
    did: u32,
    layout: Arc<LayoutDesc>,
    names: Names,
}

/// The layouts a layout's scroll bars share with: one that defines a bar this layout names, or
/// names a bar this layout defines. The HUD makes them one UI, so a bar there drives a list here.
fn linked_layouts(all: &[Loaded], a: usize) -> Vec<usize> {
    (0..all.len())
        .filter(|&b| {
            b != a
                && (!all[a].names.defines.is_disjoint(&all[b].names.refs)
                    || !all[a].names.refs.is_disjoint(&all[b].names.defines))
        })
        .collect()
}

/// Shows `index` and each hidden ancestor, as a player opens the panel that holds it.
fn open(tree: &mut UiTree, index: usize) {
    let mut at = Some(index);
    while let Some(i) = at {
        if tree.get(i).is_some_and(|e| !e.visible()) {
            tree.set_visible(i, true);
        }
        at = tree.get(i).and_then(|e| e.parent_index());
    }
}

/// Brings `index` to the top, as a player does by taking the window it is in to the front: each
/// ancestor, the owner first, goes to the lowest z among its siblings (`UIElement::SetZLevel`),
/// so the root of the owner's panel ends up the one the pointer meets first.
fn raise(tree: &mut UiTree, index: usize) {
    let mut at = Some(index);
    while let Some(i) = at {
        tree.set_z_level(i, i32::MIN);
        at = tree.get(i).and_then(|e| e.parent_index());
    }
}

/// Tests the wheel over one owner, in a tree of its layout.
fn scan_owner(
    tree: &mut UiTree,
    layout: &Arc<LayoutDesc>,
    layout_did: u32,
    index: usize,
    kind: Scroller,
    counts: &mut BTreeMap<Scroller, Counts>,
    referenced: &BTreeSet<u32>,
) {
    let label = format!("layout {layout_did:08x}");
    if kind != Scroller::Bar && !linked(tree, index) {
        counts.entry(kind).or_default().unlinked += 1;
        return;
    }
    if !shown(tree, index) {
        open(tree, index);
        counts.entry(kind).or_default().opened += 1;
    }
    if kind == Scroller::Bar {
        // A bar's turn moves what it drives: give that content overflow first, which refreshes
        // the bar's position and its disabled state.
        for text in tree.texts_bound_to(index) {
            overflow(tree, layout, text, Scroller::Text);
        }
        for (list, _) in tree.lists_bound_to(index) {
            overflow(tree, layout, list, Scroller::List);
        }
        for node in tree.trees_bound_to(index) {
            overflow(tree, layout, node, Scroller::Tree);
        }
        let drives = !tree.texts_bound_to(index).is_empty()
            || !tree.lists_bound_to(index).is_empty()
            || !tree.trees_bound_to(index).is_empty();
        // A bar of one stop (or none) has nowhere to step (`HandleMoveSteps`).
        let stops = match tree
            .get(index)
            .and_then(|e| e.attribute(tokenize("num_stop_locations")))
        {
            Some(AttributeValue::Int(n)) => *n,
            _ => 0,
        };
        let stop_located = tree.bool_attribute(index, tokenize("stop_locations"));
        let bar_id = tree.get(index).map_or(0, |e| e.id());
        if stop_located && stops <= 1 {
            // One stop (`HandleMoveSteps` moves to the end on any turn): start at the top, so a
            // turn out has something to move.
            tree.scrollbar_set_position(index, 0.0);
        }
        if !drives && !stop_located {
            // Named by a layout, but a same-id bar is the one `GetScrollbar` finds (shadowed), or
            // named by nothing in the DAT (no owner at all).
            let same_id = (0..tree.slots_len())
                .filter(|&i| {
                    tree.get(i).is_some_and(|e| {
                        e.id() == bar_id && e.kind() == crate::scrollbar::SCROLLBAR
                    })
                })
                .count();
            let count = counts.entry(kind).or_default();
            count.inert += 1;
            if !referenced.contains(&bar_id) {
                count.unnamed += 1;
            } else if same_id > 1 {
                count.shadowed += 1;
            } else {
                count.unbound += 1;
            }
            return;
        }
    } else {
        overflow(tree, layout, index, kind);
    }
    if !hit(tree, index) {
        raise(tree, index);
        if hit(tree, index) {
            counts.entry(kind).or_default().raised += 1;
        } else {
            // Every point of its box is under something on top of it, even raised.
            counts.entry(kind).or_default().covered += 1;
            return;
        }
    }
    wheel_moves(tree, index, kind, &label);
    counts.entry(kind).or_default().tested += 1;

    // A child that does not scroll passes the wheel to the scrollable element above it.
    if kind != Scroller::Bar {
        let children: Vec<usize> = tree.children(index).to_vec();
        for child in children {
            if classify(tree, child).is_some() || !shown(tree, child) {
                continue;
            }
            if hit(tree, child) && tree.get(index).is_some_and(|e| e.mouse_over()) {
                wheel_moves(tree, index, kind, &format!("{label}, over child {child}"));
                counts.entry(kind).or_default().bubbled += 1;
                break;
            }
        }
    }
}

/// Tests the wheel over each scroll bar owner of layout `all[a]`. The owners are found in the
/// layout's own UI; each is then tested in a fresh UI made of its layout and the layouts it is
/// linked with. That UI creates the layout's tops first, so an owner keeps its index.
fn scan_layout(
    store: &Arc<ResourceStore>,
    all: &[Loaded],
    a: usize,
    counts: &mut BTreeMap<Scroller, Counts>,
    referenced: &BTreeSet<u32>,
) {
    let linked = linked_layouts(all, a);
    let mut chain: Vec<&Arc<LayoutDesc>> = vec![&all[a].layout];
    chain.extend(linked.iter().map(|&b| &all[b].layout));
    let (own_ui, _) = layout_ui(store, &[&all[a].layout]);
    let owners: Vec<(usize, Scroller)> = (0..own_ui.slots_len())
        .filter(|&i| own_ui.get(i).is_some())
        .filter_map(|i| classify(&own_ui, i).map(|kind| (i, kind)))
        .collect();
    for (index, kind) in owners {
        counts.entry(kind).or_default().found += 1;
        let (mut tree, _) = layout_ui(store, &chain);
        scan_owner(
            &mut tree,
            &all[a].layout,
            all[a].did,
            index,
            kind,
            counts,
            referenced,
        );
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
    let mut all = Vec::new();
    for did in dids {
        let Some(layout) = load_layout(&store, DataId::new(did)).unwrap() else {
            continue;
        };
        let layout: Arc<LayoutDesc> = Arc::new((*layout).clone());
        let names = names_of(&layout);
        all.push(Loaded { did, layout, names });
    }
    let referenced: BTreeSet<u32> = all
        .iter()
        .flat_map(|l| l.names.refs.iter().copied())
        .collect();
    let mut counts: BTreeMap<Scroller, Counts> = BTreeMap::new();
    for a in 0..all.len() {
        scan_layout(&store, &all, a, &mut counts, &referenced);
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
