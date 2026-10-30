//! The modern inventory drawer's All view (`modern_inventory.rs`; AGENTS.md: the HUD's two styles):
//! an All tab after the pack tabs shows the four pages at once, each under a heading with its name
//! and slot numbers, in one list the wheel scrolls. The pages are retail's own containers
//! (`gmBackpackUI`'s), moved under a clipping view and each shown whole (every row, its own
//! scroll bar hidden), so their slots, drops, selection and messages stay retail's. A press on a
//! pack tab (`gmBackpackUI::ShowPack`, which hides the other pages) leaves the view.

use std::cell::Cell;
use std::sync::Arc;

use crate::element::{UiBox, UiTree};
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_drawer_bar::{count_label, place_label};
use crate::modern_inventory::{keep, make_part};
use crate::mouse::{UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::text::Justification;
use crate::token::{tokenize, MOUSE_INVISIBLE};

/// The pages' headings, in the backpack's order.
const HEADINGS: [&str; 4] = ["MAIN PACK", "SIDE PACK 1", "SIDE PACK 2", "SIDE PACK 3"];
/// A heading's height and the gap after a page.
const HEADING: i32 = 16;
const PAGE_GAP: i32 = 6;
/// The All tab's size.
const TAB: (i32, i32) = (30, 26);
/// One wheel notch: a row of slots.
const NOTCH: i32 = art::ITEM_PITCH;

/// One page's heading: its name, its slot numbers, the rule under them.
#[derive(Debug, Clone, Copy)]
struct Heading {
    name: Option<usize>,
    slots: Option<usize>,
    rule: Option<usize>,
}

/// The All view's parts and state.
#[derive(Debug, Clone)]
pub(crate) struct AllView {
    tab: Option<usize>,
    tab_label: Option<usize>,
    /// The clipping view the pages move under while the view is on.
    view: Option<usize>,
    headings: Vec<Heading>,
    /// The view's scroll bar: its track and thumb.
    track: Option<usize>,
    thumb: Option<usize>,
    /// The pack area the pages come back to.
    area: usize,
    on: Cell<bool>,
    scroll: Cell<i32>,
    /// The grid's box (the pack area's coordinates) the view fills.
    grid: Cell<UiBox>,
    /// The pages' scroll bar parts and whether each showed before the view hid it.
    hidden: std::cell::RefCell<Vec<(usize, bool)>>,
}

fn part(
    tree: &mut UiTree,
    parent: usize,
    template: Option<usize>,
    id: &str,
    image: u32,
    nine: Option<i32>,
) -> Option<usize> {
    let part = make_part(tree, parent, template, id, image, nine, 0)?;
    keep(tree, part);
    Some(part)
}

impl AllView {
    /// The tab and the view's parts made under the pack area `area`, for `pages` pages.
    pub(crate) fn make(
        tree: &mut UiTree,
        layouts: &[&Arc<LayoutDesc>],
        area: usize,
        pages: usize,
        template: Option<usize>,
    ) -> Self {
        let tab = part(
            tree,
            area,
            template,
            "ModernAllTab",
            art::ALL_TAB,
            Some(art::ALL_TAB_BORDER),
        );
        if let Some(tab) = tab {
            tree.set_bool(tab, MOUSE_INVISIBLE, false);
        }
        let tab_label = tab.and_then(|tab| {
            count_label(
                tree,
                layouts,
                tab,
                "ModernAllTabLabel",
                Justification::Center,
            )
        });
        if let Some(label) = tab_label {
            keep(tree, label);
            tree.set_text(label, "All");
        }
        // The view: a plain field that clips the pages and takes no pointer itself.
        let view = make_part(tree, area, template, "ModernAllView", 0, None, 0);
        if let Some(view) = view {
            keep(tree, view);
            tree.clear_image(view);
            tree.set_visible(view, false);
        }
        let headings = (0..pages)
            .map(|i| {
                let Some(view) = view else {
                    return Heading {
                        name: None,
                        slots: None,
                        rule: None,
                    };
                };
                let label = |tree: &mut UiTree, id: String, j| {
                    let label = count_label(tree, layouts, view, &id, j)?;
                    keep(tree, label);
                    Some(label)
                };
                let name = label(tree, format!("ModernAllHeading{i}"), Justification::Left);
                let slots = label(tree, format!("ModernAllSlots{i}"), Justification::Right);
                let rule = part(
                    tree,
                    view,
                    template,
                    &format!("ModernAllRule{i}"),
                    art::SECTION_RULE,
                    None,
                );
                Heading { name, slots, rule }
            })
            .collect();
        let track =
            view.and_then(|v| part(tree, v, template, "ModernAllTrack", art::SCROLL_TRACK, None));
        let thumb =
            view.and_then(|v| part(tree, v, template, "ModernAllThumb", art::THUMB_MID, None));
        Self {
            tab,
            tab_label,
            view,
            headings,
            track,
            thumb,
            area,
            on: Cell::new(false),
            scroll: Cell::new(0),
            grid: Cell::new(UiBox::default()),
            hidden: Default::default(),
        }
    }

    pub(crate) fn is_on(&self) -> bool {
        self.on.get()
    }

    /// The tab put at `(x, y)` (the pack area's coordinates), its lower edge level with the
    /// pack tabs'.
    pub(crate) fn place_tab(&self, tree: &mut UiTree, (x, y): (i32, i32)) {
        let Some(tab) = self.tab else {
            return;
        };
        if tree.el(tab).rect.x0 != x || tree.el(tab).rect.y0 != y {
            tree.resize_to(tab, TAB.0, TAB.1);
            tree.move_to(tab, x, y);
        }
        place_label(tree, self.tab_label, UiBox::new(0, 0, TAB.0 - 1, TAB.1 - 1));
    }

    /// A press on the All tab turns the view on (`pages`: the pages' containers). Whether the
    /// press was the tab's.
    pub(crate) fn press(&self, tree: &mut UiTree, element: usize, pages: &[Option<usize>]) -> bool {
        if self.tab != Some(element) {
            return false;
        }
        if !self.on.get() {
            self.on.set(true);
            self.scroll.set(0);
            let mut hidden = self.hidden.borrow_mut();
            hidden.clear();
            for &container in pages.iter().flatten() {
                tree.set_visible(container, true);
                if let Some(view) = self.view {
                    tree.set_parent(container, Some(view));
                }
                for bar in tree.children(container).to_vec() {
                    if tree.el(bar).id() != tokenize("ContainerListBox") {
                        hidden.push((bar, tree.el(bar).visible));
                        tree.set_visible(bar, false);
                    }
                }
            }
        }
        true
    }

    /// The view left when a pack tab showed one page (`gmBackpackUI::ShowPack` hid the others):
    /// the pages back under the pack area, their bars as they were. Whether it was left.
    pub(crate) fn follow(&self, tree: &mut UiTree, pages: &[Option<usize>]) -> bool {
        if !self.on.get() {
            return false;
        }
        if pages
            .iter()
            .flatten()
            .all(|&c| tree.get(c).is_some_and(|e| e.visible))
        {
            return false;
        }
        self.on.set(false);
        for &container in pages.iter().flatten() {
            tree.set_parent(container, Some(self.area));
        }
        for (bar, shown) in self.hidden.borrow_mut().drain(..) {
            tree.set_visible(bar, shown);
        }
        true
    }

    /// The tab's look and the view laid out: while on, the view fills `grid` and the pages stack
    /// in it under their headings, scrolled; while off, the view hides.
    pub(crate) fn lay_out(&self, tree: &mut UiTree, pages: &[(Option<usize>, u32)], grid: UiBox) {
        self.grid.set(grid);
        if let Some(tab) = self.tab {
            let image = if self.on.get() {
                art::ALL_TAB_ON
            } else {
                art::ALL_TAB
            };
            tree.set_image(tab, image, crate::compose::DRAW_OVER);
        }
        let Some(view) = self.view else {
            return;
        };
        if tree.el(view).visible != self.on.get() {
            tree.set_visible(view, self.on.get());
        }
        if !self.on.get() {
            return;
        }
        if tree.el(view).rect != grid {
            tree.resize_to(view, grid.width(), grid.height());
            tree.move_to(view, grid.x0, grid.y0);
        }
        let width = grid.width() - art::SCROLL_WIDTH - 1;
        let total: i32 = pages
            .iter()
            .map(|&(_, size)| HEADING + rows(size) * art::ITEM_PITCH + PAGE_GAP)
            .sum();
        let scroll = self.scroll.get().clamp(0, (total - grid.height()).max(0));
        self.scroll.set(scroll);
        let mut y = -scroll;
        let mut first = 1;
        for (i, &(container, size)) in pages.iter().enumerate() {
            let heading = self.headings.get(i).copied();
            if let Some(h) = heading {
                let name = HEADINGS.get(i).copied().unwrap_or("PACK");
                set_text(tree, h.name, name);
                set_text(
                    tree,
                    h.slots,
                    &format!("{}-{}", first, first + size.max(1) - 1),
                );
                place_label(tree, h.name, UiBox::new(0, y, width - 1, y + HEADING - 3));
                place_label(tree, h.slots, UiBox::new(0, y, width - 1, y + HEADING - 3));
                if let Some(rule) = h.rule {
                    put(
                        tree,
                        rule,
                        UiBox::new(0, y + HEADING - 2, width - 1, y + HEADING - 2),
                    );
                }
            }
            first += size;
            y += HEADING;
            let height = rows(size) * art::ITEM_PITCH;
            if let Some(container) = container {
                put(tree, container, UiBox::new(0, y, width - 1, y + height - 1));
                if let Some(list) = tree.child(container, tokenize("ContainerListBox")) {
                    put(tree, list, UiBox::new(0, 0, width - 1, height - 1));
                }
            }
            y += height + PAGE_GAP;
        }
        // The bar: the track the view's height, the thumb its share of it.
        let x = grid.width() - art::SCROLL_WIDTH;
        let h = grid.height();
        if let Some(track) = self.track {
            put(
                tree,
                track,
                UiBox::new(x, 0, x + art::SCROLL_WIDTH - 1, h - 1),
            );
        }
        if let Some(thumb) = self.thumb {
            let length = (h * h / total.max(1)).clamp(12, h);
            let top = if total > h {
                (h - length) * scroll / (total - h)
            } else {
                0
            };
            put(
                tree,
                thumb,
                UiBox::new(x, top, x + art::SCROLL_WIDTH - 1, top + length - 1),
            );
        }
    }

    /// A wheel turn over the view scrolls it a row. Whether the turn was the view's.
    pub(crate) fn wheel(&self, tree: &UiTree, action: u32) -> bool {
        let Some(view) = self.view else {
            return false;
        };
        if !self.on.get() || !matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT) {
            return false;
        }
        let at = tree.screen_box(view);
        let (x, y) = tree.pointer();
        if x < at.x0 || x > at.x1 || y < at.y0 || y > at.y1 {
            return false;
        }
        let step = if action == UI_WHEEL_IN { -NOTCH } else { NOTCH };
        self.scroll.set(self.scroll.get() + step);
        true
    }
}

/// The rows a page of `size` slots takes, `COLUMNS` a row.
fn rows(size: u32) -> i32 {
    (size as i32 + crate::modern_inventory::COLUMNS - 1) / crate::modern_inventory::COLUMNS
}

fn set_text(tree: &mut UiTree, label: Option<usize>, text: &str) {
    if let Some(label) = label {
        if tree.el(label).text().as_deref() != Some(text) {
            tree.set_text(label, text);
        }
    }
}

/// `element` moved and sized to `at`, when it is not there.
fn put(tree: &mut UiTree, element: usize, at: UiBox) {
    if tree.el(element).rect != at {
        tree.resize_to(element, at.width(), at.height());
        tree.move_to(element, at.x0, at.y0);
    }
}
