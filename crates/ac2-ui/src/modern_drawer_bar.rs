//! The modern inventory drawer's own parts beside retail's (`modern_inventory.rs`; AGENTS.md: the
//! HUD's two styles): each pack tab's count (`used/size`), the search field at the tabs' right, and
//! the footer naming the page shown with its count. They read the backpack and change nothing in
//! it: the search veils the shown page's slots whose item's name does not hold the typed text
//! (case-insensitively), leaving every slot, its order and its drops as they are.

use std::cell::Cell;
use std::sync::Arc;

use crate::backpack::BackpackUi;
use crate::element::{UiBox, UiTree};
use crate::item_display::InventorySource;
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_inventory::{keep, make_part};
use crate::text::Justification;
use crate::token::{tokenize, MOUSE_INVISIBLE};
use crate::window_parts::{entry_field, place_entry};

/// The pages' names in the footer, in the backpack's order.
const PAGE_NAMES: [&str; 4] = ["Main Pack", "Side Pack 1", "Side Pack 2", "Side Pack 3"];
/// A slot's veil, a child of the slot's background.
const VEIL: &str = "ModernSlotVeil";

/// The drawer's own parts.
#[derive(Debug, Clone)]
pub(crate) struct DrawerBar {
    /// Each pack tab's count label.
    counts: Vec<Option<usize>>,
    /// The search field and its pill.
    search: Option<(usize, Option<usize>)>,
    /// The footer's label.
    foot: Option<usize>,
    /// The template the veils are copied from.
    template: Option<usize>,
    /// What the labels and veils were last set for.
    shown_for: Cell<Option<u64>>,
}

/// A count label under `parent` named `id`: a copy of the shortcut slot's number
/// (`NumberDisplay`, the layouts' small outlined digits), justified by `justification`, taking no
/// pointer.
fn count_label(
    tree: &mut UiTree,
    layouts: &[&Arc<LayoutDesc>],
    parent: usize,
    id: &str,
    justification: Justification,
) -> Option<usize> {
    let (layout, template) = layouts
        .iter()
        .find_map(|l| Some((Arc::clone(l), l.find_element(tokenize("NumberDisplay"))?)))?;
    let mut desc = (*template).clone();
    desc.id = tokenize(id);
    desc.edges = [0; 4];
    let count = tree.create_element(layout, Arc::new(desc), false)?;
    tree.set_parent(count, Some(parent));
    tree.set_bool(count, MOUSE_INVISIBLE, true);
    if let Some(text) = tree.el_mut(count).text.as_mut() {
        text.justification = justification;
    }
    Some(count)
}

/// `label`'s text set.
fn set_label(tree: &mut UiTree, label: Option<usize>, text: &str) {
    let Some(label) = label else {
        return;
    };
    tree.set_text(label, text);
}

/// `label` put at `at`.
fn place_label(tree: &mut UiTree, label: Option<usize>, at: UiBox) {
    let Some(label) = label else {
        return;
    };
    if tree.el(label).rect != at {
        tree.move_to(label, at.x0, at.y0);
        tree.resize_to(label, at.width(), at.height());
        for child in tree.children(label).to_vec() {
            tree.move_to(child, 0, 0);
            tree.resize_to(child, at.width(), at.height());
        }
    }
}

impl DrawerBar {
    /// The parts made under the pack area (`parent`): a count per tab, the search field, the
    /// footer. `template` is a plain field of the layout the veils are copied from.
    pub(crate) fn make(
        tree: &mut UiTree,
        gameplay: &Arc<LayoutDesc>,
        layouts: &[&Arc<LayoutDesc>],
        parent: usize,
        tabs: usize,
        template: Option<usize>,
    ) -> Self {
        let counts: Vec<Option<usize>> = (0..tabs)
            .map(|i| {
                count_label(
                    tree,
                    layouts,
                    parent,
                    &format!("ModernPackCount{i}"),
                    Justification::Right,
                )
            })
            .collect();
        let search = entry_field(
            tree,
            gameplay,
            layouts,
            parent,
            ("ModernDrawerSearch", "ModernDrawerSearchPill"),
            (0, 0, 90, 20),
        );
        if let Some((_, Some(pill))) = search {
            tree.set_image(pill, art::SEARCH_PILL, crate::compose::DRAW_OVER);
            tree.set_int(pill, crate::compose::NINE_SLICE, art::SEARCH_PILL_BORDER);
        }
        let foot = count_label(
            tree,
            layouts,
            parent,
            "ModernDrawerFoot",
            Justification::Left,
        );
        // Each part (and its text) keeps where it is put when the drawer is laid out again.
        let mut parts: Vec<usize> = counts.iter().flatten().copied().collect();
        parts.extend(foot);
        if let Some((field, pill)) = search {
            parts.push(field);
            parts.extend(pill);
        }
        for part in parts {
            keep(tree, part);
            for child in tree.children(part).to_vec() {
                keep(tree, child);
            }
        }
        Self {
            counts,
            search,
            foot,
            template,
            shown_for: Cell::new(None),
        }
    }

    /// The counts over the tabs' lower edges (`tabs`, in the pack area's coordinates), the search
    /// field `search`, the footer `foot`.
    pub(crate) fn place(&self, tree: &mut UiTree, tabs: &[UiBox], search: UiBox, foot: UiBox) {
        for (count, tab) in self.counts.iter().zip(tabs) {
            place_label(
                tree,
                *count,
                UiBox::new(tab.x0, tab.y1 - 12, tab.x1 - 2, tab.y1),
            );
        }
        if let Some((field, pill)) = self.search {
            place_entry(
                tree,
                field,
                pill,
                (search.x0, search.y0, search.width(), search.height()),
            );
            // The pill's ends are round, not the book's stone caps: the text sits closer in.
            tree.move_to(field, search.x0 + 8, search.y0 + 1);
            tree.resize_to(field, search.width() - 16, search.height() - 2);
        }
        place_label(tree, self.foot, foot);
        self.shown_for.set(None);
    }

    /// The search field's text, trimmed and lower-cased (empty without the field).
    fn query(&self, tree: &UiTree) -> String {
        self.search
            .and_then(|(field, _)| tree.get(field).and_then(|e| e.text()))
            .map(|t| t.trim().to_lowercase())
            .unwrap_or_default()
    }

    /// The counts, the footer and the veils set for the backpack as it is: run every frame,
    /// changing the tree only when what they show changed.
    pub(crate) fn tick(
        &self,
        tree: &mut UiTree,
        backpack: &BackpackUi,
        source: &dyn InventorySource,
    ) {
        let query = self.query(tree);
        let shown = backpack
            .packs
            .iter()
            .position(|p| {
                // The page shown: its list's container is (`gmBackpackUI::ShowPack`).
                p.list
                    .and_then(|l| tree.get(l))
                    .and_then(|l| l.parent_index())
                    .and_then(|c| tree.get(c))
                    .is_some_and(|c| c.visible)
            })
            .unwrap_or(0);
        let used: Vec<usize> = backpack
            .packs
            .iter()
            .map(|p| p.items.iter().filter(|d| d.item != 0).count())
            .collect();
        let key = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            (&query, shown, &used).hash(&mut h);
            for pack in &backpack.packs {
                pack.max_size().hash(&mut h);
                for d in &pack.items {
                    (d.item, d.background).hash(&mut h);
                }
            }
            h.finish()
        };
        if self.shown_for.get() == Some(key) {
            return;
        }
        self.shown_for.set(Some(key));
        for (i, pack) in backpack.packs.iter().enumerate() {
            let text = format!("{}/{}", used[i], pack.max_size());
            set_label(tree, self.counts.get(i).copied().flatten(), &text);
        }
        if let Some(pack) = backpack.packs.get(shown) {
            let name = PAGE_NAMES.get(shown).copied().unwrap_or("Pack");
            let text = format!("{name} \u{b7} {}/{}", used[shown], pack.max_size());
            set_label(tree, self.foot, &text);
        }
        for (i, pack) in backpack.packs.iter().enumerate() {
            for d in &pack.items {
                let veiled = i == shown
                    && !query.is_empty()
                    && d.item != 0
                    && !source
                        .item(d.item)
                        .is_some_and(|slot| slot.name.to_lowercase().contains(&query));
                self.veil(tree, d.background, d.icon, veiled);
            }
        }
    }

    /// The veil over `slot` shown or hidden, made the first time it shows.
    fn veil(&self, tree: &mut UiTree, slot: usize, icon: usize, shown: bool) {
        if tree.get(slot).is_none() {
            return;
        }
        let id = tokenize(VEIL);
        let existing = tree
            .children(slot)
            .iter()
            .copied()
            .find(|&c| tree.el(c).id() == id);
        let veil = match (existing, shown) {
            (Some(veil), _) => veil,
            (None, false) => return,
            (None, true) => {
                // In front of the icon (a lower z draws in front).
                let z = tree.get(icon).map_or(0, |e| e.z) - 1;
                let Some(veil) =
                    make_part(tree, slot, self.template, VEIL, art::SLOT_VEIL, None, z)
                else {
                    return;
                };
                let side = tree.el(slot).rect.width();
                tree.resize_to(veil, side, side);
                tree.move_to(veil, 0, 0);
                veil
            }
        };
        if tree.el(veil).visible() != shown {
            tree.set_visible(veil, shown);
        }
    }
}
