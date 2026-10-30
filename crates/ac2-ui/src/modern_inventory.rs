//! The modern HUD's inventory (AGENTS.md: the HUD's two styles; the design "variant D"): the
//! inventory panel as a drawer on the right edge, between the radar and the strip. Retail's panel
//! (`InventoryField` in `RightSidePanel`) drew its stone frame (`0x41007118`, `0x41007119`) with
//! the gold field at the top, the equipment slots in two columns right of the doll, and the pack
//! pages and their buttons under them. Here the same elements, with their states, drops and
//! messages, are laid out as the design has them:
//!
//! - the drawer 316 pixels wide on the bevelled glass, its header a bronze fall with the title
//!   and the help and close buttons on their own backings;
//! - the gold row on dark glass;
//! - the equipment: a column of five slots, the doll's frame (the 3D viewport docked to it), a
//!   column of five slots;
//! - retail's rail under them, then the pack buttons as tabs and the page's grid, seven slots of
//!   36 pixels 4 apart (`restyle`), its scroll bar at the right.
//!
//! The drawer is laid out by the display's edges (not a saved window), again when the display
//! changes ([`Drawer::arrange`]).

use std::sync::Arc;

use crate::compose::{DRAW_OVER, NINE_SLICE};
use crate::element::{UiBox, UiTree};
use crate::layout::{ElementDesc, LayoutDesc, MediaDesc, Rect};
use crate::modern_art as art;
use crate::token::{tokenize, MOUSE_INVISIBLE};

/// The drawer's width, its distance from the display's top (under the radar) and bottom (over
/// the strip), its padding and the gaps between its parts.
const WIDTH: i32 = 316;
const TOP: i32 = 172;
const BOTTOM: i32 = 68;
const PAD: i32 = 8;
const HEADER: i32 = 30;
const MONEY: i32 = 32;
const GAP: i32 = 6;
/// The equipment block: a slot's frame (the slot and its durability bar), the gap between rows,
/// the doll's frame and the gap beside it.
const SLOT: (i32, i32) = (40, 36);
const SLOT_GAP: i32 = 4;
const DOLL: (i32, i32) = (132, 196);
const RAIL: i32 = 31;
/// The pack tabs' overlap with the rail, the main tab's size and the others'.
const TAB_OVERLAP: i32 = 12;
const MAIN_TAB: (i32, i32) = (46, 36);
const TAB: (i32, i32) = (38, 26);
const TAB_GAP: i32 = 2;
/// The grid's columns and its scroll bar's width.
pub(crate) const COLUMNS: i32 = 7;
/// The footer's height and the search field's size.
const FOOT: i32 = 18;
const SEARCH: (i32, i32) = (90, 20);
/// The filter chips' row, with the gap under it.
const CHIPS: i32 = 21;
const SCROLL: i32 = art::SCROLL_WIDTH;
/// The tallest of retail's scroll arrows (15), so an arrow is told from the track.
const ARROW_MAX: i32 = 16;
/// A header button's backing and the button on it.
const HEADER_BUTTON: i32 = 20;
const HEADER_ICON: i32 = 16;
/// The depths of the drawer's backings (a lower z draws in front).
const BACKING_Z: i32 = 200;
const PART_Z: i32 = 150;

/// The inventory layout's elements the drawer lays out that have no name in it: the top frame
/// (the gold field and the equipment), the title, the equipment's holder, the rail, the pack area.
const TOP_FRAME: u32 = 0x27ED_4815;
const TITLE: u32 = 0x654E_1370;
const EQUIPMENT: u32 = 0x22D1_BC45;
const RAIL_ID: u32 = 0x523D_41C6;
const PACK_AREA: u32 = 0x1BB9_C51C;
/// The gold field's icon and its label.
const COIN_ICON: u32 = 0xBCF1_586D;
const GOLD_LABEL: u32 = 0x17CE_0391;

const PAGES: [(&str, &str); 4] = [
    ("PackMainContainer", "PackMainButton"),
    ("PackOneContainer", "PackOneButton"),
    ("PackTwoContainer", "PackTwoButton"),
    ("PackThreeContainer", "PackThreeButton"),
];

fn sized(x: i32, y: i32, (w, h): (i32, i32)) -> UiBox {
    UiBox::new(x, y, x + w - 1, y + h - 1)
}

fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
    Rect {
        x0: x0 as i16,
        y0: y0 as i16,
        x1: x1 as i16,
        y1: y1 as i16,
    }
}

/// The toolbar layout with an item slot (`ItemBackground`) the pitch of the design's grid (its
/// well, `modern_art::ITEM_WELL`, the DAT slot in its corner; its parts kept 36 square there).
/// Every other element is shared.
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    let mut layout = layout.clone();
    let pitch = art::ITEM_PITCH;
    let edit = |d: &mut ElementDesc| {
        if d.id == tokenize("ItemBackground") {
            d.rect = rect(0, 0, pitch - 1, pitch - 1);
            for state in &mut d.states {
                for media in &mut state.media {
                    if let MediaDesc::Image {
                        file, draw_mode, ..
                    } = media
                    {
                        if *file == art::ITEM_SLOT {
                            *file = art::ITEM_WELL;
                            *draw_mode = DRAW_OVER;
                        }
                    }
                }
                for child in &mut state.children {
                    if child.edges == [1, 1, 1, 1] {
                        let mut c = (**child).clone();
                        c.edges = [0; 4];
                        c.rect = rect(0, 0, 35, 35);
                        *child = Arc::new(c);
                    }
                }
            }
        }
    };
    fn walk(d: &ElementDesc, edit: &dyn Fn(&mut ElementDesc)) -> ElementDesc {
        let mut d = d.clone();
        for state in &mut d.states {
            for child in &mut state.children {
                *child = Arc::new(walk(child, edit));
            }
        }
        edit(&mut d);
        d
    }
    for element in &mut layout.elements {
        *element = Arc::new(walk(element, &edit));
    }
    layout
}

/// The inventory's elements the drawer lays out, and the parts it adds.
#[derive(Debug, Clone)]
pub(crate) struct Drawer {
    panel: usize,
    field: usize,
    top_frame: Option<usize>,
    title: Option<usize>,
    help: Option<usize>,
    close: Option<usize>,
    money: Option<usize>,
    coin_icon: Option<usize>,
    gold_label: Option<usize>,
    amount: Option<usize>,
    equipment: Option<usize>,
    /// The equipment's slot frames, the left column's then the right's, top to bottom.
    slots: Vec<usize>,
    rail: Option<usize>,
    pack_area: Option<usize>,
    /// The pages' containers and buttons.
    pages: Vec<(Option<usize>, Option<usize>)>,
    /// The parts the drawer adds: its glass, header, the help and close buttons' backings, the
    /// gold row's backing, the doll's frame.
    glass: Option<usize>,
    header: Option<usize>,
    backings: [Option<usize>; 2],
    money_row: Option<usize>,
    pub(crate) doll: Option<usize>,
    /// The drawer's own parts: tab counts, search, footer (`modern_drawer_bar`).
    bar: Option<crate::modern_drawer_bar::DrawerBar>,
    /// The item ring (`modern_item_ring`).
    ring: Option<crate::modern_item_ring::ItemRing>,
    /// The All view (`modern_all_view`).
    all: Option<crate::modern_all_view::AllView>,
    /// The pages' slot counts, as the backpack last had them.
    sizes: std::cell::Cell<[u32; 4]>,
    /// The display the drawer was last laid out for.
    laid_out: std::cell::Cell<Option<(i32, i32)>>,
}

/// A copy of `desc` whose edges are all `0`: once sized, the element keeps where it is put.
fn kept_in_place(desc: &ElementDesc) -> ElementDesc {
    let mut desc = desc.clone();
    desc.edges = [0; 4];
    desc
}

/// `element` kept where it is put: its edges `0` in its description (and its referent's).
pub(crate) fn keep(tree: &mut UiTree, element: usize) {
    let (reference, desc) = {
        let el = tree.el(element);
        (Arc::clone(&el.reference), Arc::clone(&el.desc))
    };
    let new_reference = Arc::new(kept_in_place(&reference));
    let new_desc = if Arc::ptr_eq(&reference, &desc) {
        Arc::clone(&new_reference)
    } else {
        Arc::new(kept_in_place(&desc))
    };
    let el = tree.el_mut(element);
    el.reference = new_reference;
    el.desc = new_desc;
}

/// The pack pages' scroll bar art (`0x410002A5`..`0x410002B8`, `0x41007FDC`, `0x41007FDD`) and
/// the thin bar's that stands for it (`modern_art`).
pub(crate) const SCROLL_ART: [(u32, u32); 11] = [
    (0x4100_02A8, art::SCROLL_TRACK),
    (0x4100_02A5, art::THUMB_TOP),
    (0x4100_02A6, art::THUMB_MID),
    (0x4100_02A7, art::THUMB_BOTTOM),
    (0x4100_02B5, art::SCROLL_UP),
    (0x4100_02B6, art::SCROLL_UP_LIT),
    (0x4100_7FDD, art::SCROLL_UP_LIT),
    (0x4100_02B7, art::SCROLL_DOWN),
    (0x4100_02B8, art::SCROLL_DOWN_LIT),
    (0x4100_7FDC, art::SCROLL_DOWN_LIT),
    (0x4100_02BA, art::SCROLL_TRACK),
];

/// A copy of `desc` whose images in every state are swapped by `map` (from, to), drawn over.
fn remapped(desc: &ElementDesc, map: &[(u32, u32)]) -> ElementDesc {
    let mut desc = desc.clone();
    for state in &mut desc.states {
        for media in &mut state.media {
            if let MediaDesc::Image {
                file, draw_mode, ..
            } = media
            {
                if let Some(&(_, to)) = map.iter().find(|(from, _)| from == file) {
                    *file = to;
                    *draw_mode = DRAW_OVER;
                }
            }
        }
    }
    desc
}

/// `element` (and, when `recursive`, everything under it) drawn with the client's art: its
/// states' images swapped by `map` (in the description and its referent, so a state change keeps
/// them) and the image shown now swapped.
pub(crate) fn remap_art(tree: &mut UiTree, element: usize, map: &[(u32, u32)], recursive: bool) {
    let (reference, desc) = {
        let el = tree.el(element);
        (Arc::clone(&el.reference), Arc::clone(&el.desc))
    };
    let new_reference = Arc::new(remapped(&reference, map));
    let new_desc = if Arc::ptr_eq(&reference, &desc) {
        Arc::clone(&new_reference)
    } else {
        Arc::new(remapped(&desc, map))
    };
    let el = tree.el_mut(element);
    el.reference = new_reference;
    el.desc = new_desc;
    let shown = tree.el(element).image().map(|i| i.file);
    if let Some(&(_, to)) = map.iter().find(|(from, _)| Some(*from) == shown) {
        tree.set_image(element, to, DRAW_OVER);
    }
    if recursive {
        for child in tree.children(element).to_vec() {
            remap_art(tree, child, map, true);
        }
    }
}

/// `element` moved and sized to `at` (its parent's coordinates), when it is not there.
fn put(tree: &mut UiTree, element: Option<usize>, at: UiBox) {
    let Some(element) = element else {
        return;
    };
    if tree.el(element).rect != at {
        tree.resize_to(element, at.width(), at.height());
        tree.move_to(element, at.x0, at.y0);
    }
}

impl Drawer {
    /// The inventory panel made a drawer: its art cleared, its parts kept where they are put, the
    /// design's parts made. `None` without the panel. Then laid out.
    pub(crate) fn adopt(
        tree: &mut UiTree,
        gameplay: Option<&Arc<LayoutDesc>>,
        layouts: &[&Arc<LayoutDesc>],
    ) -> Option<Self> {
        let panel = tree
            .roots()
            .iter()
            .copied()
            .find(|&r| tree.el(r).id() == tokenize("RightSidePanel"))?;
        let field = tree.child_recursive(panel, tokenize("InventoryField"))?;
        let find = |tree: &UiTree, id: u32| tree.child_recursive(field, id);
        let named = |tree: &UiTree, name: &str| find(tree, tokenize(name));
        let top_frame = find(tree, TOP_FRAME);
        let equipment = find(tree, EQUIPMENT);
        let mut slots: Vec<usize> = equipment
            .map(|e| tree.children(e).to_vec())
            .unwrap_or_default();
        // The slots in the holder's two columns (a slot outside the holder's box stays as it is).
        if let Some(holder) = equipment {
            let b = tree.el(holder).rect;
            slots.retain(|&s| {
                let r = tree.el(s).rect;
                r.x0 >= 0 && r.y0 >= 0 && r.x1 < b.width() && r.y1 < b.height()
            });
        }
        let middle = slots
            .iter()
            .map(|&s| tree.el(s).rect.x0)
            .sum::<i32>()
            .checked_div(slots.len() as i32)
            .unwrap_or(0);
        slots.sort_by_key(|&s| {
            let r = tree.el(s).rect;
            (r.x0 > middle, r.y0)
        });
        let pages: Vec<(Option<usize>, Option<usize>)> = PAGES
            .iter()
            .map(|(container, button)| (named(tree, container), named(tree, button)))
            .collect();
        let mut drawer = Self {
            panel,
            field,
            top_frame,
            title: find(tree, TITLE),
            help: named(tree, "InventoryHelpButton"),
            close: named(tree, "InventoryCloseButton"),
            money: named(tree, "MoneyBag"),
            coin_icon: find(tree, COIN_ICON),
            gold_label: find(tree, GOLD_LABEL),
            amount: named(tree, "AmountLabel"),
            equipment,
            slots,
            rail: find(tree, RAIL_ID),
            pack_area: find(tree, PACK_AREA),
            pages,
            glass: None,
            header: None,
            backings: [None, None],
            money_row: None,
            doll: None,
            bar: None,
            all: None,
            ring: None,
            sizes: std::cell::Cell::new([0; 4]),
            laid_out: std::cell::Cell::new(None),
        };
        // Every element the drawer places keeps where it is put; the frames lose their stone art.
        let mut placed: Vec<usize> = [
            Some(field),
            drawer.top_frame,
            drawer.title,
            drawer.help,
            drawer.close,
            drawer.money,
            drawer.coin_icon,
            drawer.gold_label,
            drawer.amount,
            drawer.equipment,
            drawer.rail,
            drawer.pack_area,
        ]
        .into_iter()
        .flatten()
        .collect();
        placed.extend(drawer.slots.iter().copied());
        for (container, button) in &drawer.pages {
            placed.extend(container.iter().chain(button.iter()).copied());
            if let Some(container) = container {
                placed.extend(tree.children(*container).to_vec());
                // The page's scroll bar drawn thin.
                for part in tree.children(*container).to_vec() {
                    if tree.el(part).id() != tokenize("ContainerListBox") {
                        remap_art(tree, part, &SCROLL_ART, true);
                    }
                }
                // The page's list `COLUMNS` wide (`UIElement_ListBox::SetAttribute_Int`).
                if let Some(list) = tree.child_recursive(*container, tokenize("ContainerListBox")) {
                    tree.set_int(list, tokenize("max_columns"), COLUMNS);
                }
            }
        }
        for element in placed {
            keep(tree, element);
        }
        for frame in [drawer.top_frame, drawer.pack_area].into_iter().flatten() {
            tree.clear_image(frame);
        }
        // The panel and the inventory are laid out by the display's edges, not saved windows.
        tree.set_overlay_root(panel);
        keep(tree, panel);
        let template = drawer.rail;
        let part = |tree: &mut UiTree, id: &str, image: u32, nine: Option<i32>, z: i32| {
            make_part(tree, field, template, id, image, nine, z)
        };
        drawer.glass = part(
            tree,
            "ModernDrawerGlass",
            art::BEVEL,
            Some(art::BEVEL_BORDER),
            BACKING_Z,
        );
        drawer.header = part(
            tree,
            "ModernDrawerHeader",
            art::DRAWER_HEADER,
            Some(4),
            BACKING_Z - 10,
        );
        drawer.backings = [
            part(
                tree,
                "ModernDrawerHelpBacking",
                art::HEADER_BUTTON,
                None,
                PART_Z,
            ),
            part(
                tree,
                "ModernDrawerCloseBacking",
                art::HEADER_BUTTON,
                None,
                PART_Z,
            ),
        ];
        drawer.money_row = part(
            tree,
            "ModernDrawerMoneyRow",
            art::MONEY_ROW,
            Some(8),
            PART_Z,
        );
        drawer.doll = part(tree, "ModernDrawerDoll", art::DOLL_FRAME, None, PART_Z);
        drawer.ring = Some(crate::modern_item_ring::ItemRing::make(
            tree, field, template,
        ));
        if let Some(area) = drawer.pack_area {
            drawer.all = Some(crate::modern_all_view::AllView::make(
                tree,
                layouts,
                area,
                drawer.pages.len(),
                template,
            ));
        }
        if let (Some(gameplay), Some(area)) = (gameplay, drawer.pack_area) {
            drawer.bar = Some(crate::modern_drawer_bar::DrawerBar::make(
                tree,
                gameplay,
                layouts,
                area,
                drawer.pages.len(),
                template,
            ));
        }
        drawer.arrange(tree);
        Some(drawer)
    }

    /// [`Self::arrange`] when the display changed (run every frame).
    pub(crate) fn follow(&self, tree: &mut UiTree) {
        let (ox, oy) = tree.hud_offset("drawer");
        let key = (tree.display.0 + ox * 100_000, tree.display.1 + oy * 100_000);
        if self.laid_out.get() != Some(key) {
            self.arrange(tree);
        }
    }

    /// Every part put where the design has it, for the display as it is.
    pub(crate) fn arrange(&self, tree: &mut UiTree) {
        let (width, height) = tree.display;
        let (ox, oy) = tree.hud_offset("drawer");
        self.laid_out
            .set(Some((width + ox * 100_000, height + oy * 100_000)));
        let h = (height - TOP - BOTTOM).max(400);
        let at = (width - WIDTH + ox, TOP + oy);
        let now = tree.screen_box(self.panel);
        if (now.x0, now.y0, now.width(), now.height()) != (at.0, at.1, WIDTH, h) {
            tree.resize_to(self.panel, WIDTH, h);
            tree.move_to(self.panel, at.0, at.1);
        }
        let whole = sized(0, 0, (WIDTH, h));
        put(tree, Some(self.field), whole);
        put(tree, self.glass, whole);
        put(tree, self.header, sized(0, 0, (WIDTH, HEADER)));
        // The top frame holds the title, the buttons, the gold and the equipment.
        let equipment_y = HEADER + GAP + MONEY + GAP;
        put(
            tree,
            self.top_frame,
            sized(0, 0, (WIDTH, equipment_y + DOLL.1)),
        );
        put(tree, self.title, sized(0, 0, (WIDTH, HEADER)));
        // Help at the header's left, close at its right, the title between.
        let close_x = WIDTH - PAD - HEADER_BUTTON;
        let help_x = PAD;
        let by = (HEADER - HEADER_BUTTON) / 2;
        let inset = (HEADER_BUTTON - HEADER_ICON) / 2;
        for (i, (button, x)) in [(self.help, help_x), (self.close, close_x)]
            .into_iter()
            .enumerate()
        {
            put(
                tree,
                self.backings[i],
                sized(x, by, (HEADER_BUTTON, HEADER_BUTTON)),
            );
            put(
                tree,
                button,
                sized(x + inset, by + inset, (HEADER_ICON, HEADER_ICON)),
            );
        }
        let money = sized(PAD, HEADER + GAP, (WIDTH - 2 * PAD, MONEY));
        put(tree, self.money_row, money);
        put(tree, self.money, money);
        put(tree, self.coin_icon, sized(5, 5, (25, 23)));
        put(tree, self.gold_label, sized(36, 4, (120, 24)));
        put(
            tree,
            self.amount,
            sized(36, 4, (money.width() - 36 - 8, 24)),
        );
        // The equipment: a column of five, the doll, a column of five, centred.
        let block = 2 * SLOT.0 + 2 * GAP + DOLL.0;
        let x0 = (WIDTH - block) / 2;
        put(tree, self.equipment, sized(0, equipment_y, (WIDTH, DOLL.1)));
        let doll_x = x0 + SLOT.0 + GAP;
        put(tree, self.doll, sized(doll_x, equipment_y, DOLL));
        for (i, &slot) in self.slots.iter().enumerate() {
            let (column, row) = ((i / 5) as i32, (i % 5) as i32);
            let x = if column == 0 {
                x0
            } else {
                doll_x + DOLL.0 + GAP
            };
            put(tree, Some(slot), sized(x, row * (SLOT.1 + SLOT_GAP), SLOT));
        }
        // The rail, then the tabs over its lower part, then the page's grid.
        let rail_y = equipment_y + DOLL.1 + 2;
        put(tree, self.rail, sized(0, rail_y, (WIDTH, RAIL)));
        let area_y = rail_y + RAIL - TAB_OVERLAP;
        let area_h = h - area_y - PAD;
        put(tree, self.pack_area, sized(0, area_y, (WIDTH, area_h)));
        let mut tab_x = PAD;
        // The filter chips under the tabs, then the grid.
        let chips_y = MAIN_TAB.1 + 3;
        let grid_y = chips_y + CHIPS;
        // The footer under the grid: the page's name and count.
        let foot = sized(PAD, area_h - FOOT, (WIDTH - 2 * PAD, FOOT));
        let grid = sized(PAD, grid_y, (WIDTH - 2 * PAD, area_h - grid_y - FOOT - 2));
        let mut tabs = Vec::new();
        for (i, (container, button)) in self.pages.iter().enumerate() {
            let size = if i == 0 { MAIN_TAB } else { TAB };
            let tab = sized(tab_x, MAIN_TAB.1 - size.1, size);
            put(tree, *button, tab);
            tabs.push(tab);
            tab_x += size.0 + TAB_GAP;
            // In the All view the pages are the view's (`modern_all_view`).
            if self.all.as_ref().is_some_and(|a| a.is_on()) {
                continue;
            }
            put(tree, *container, grid);
            if let Some(container) = container {
                self.place_page(tree, *container, grid);
            }
        }
        if let Some(all) = &self.all {
            all.place_tab(tree, (tab_x + TAB_GAP, MAIN_TAB.1 - 26));
            all.lay_out(tree, &self.page_sizes(), grid);
        }
        if let Some(bar) = &self.bar {
            // The search field at the tabs' right, level with their lower edge.
            let search = sized(WIDTH - PAD - SEARCH.0, MAIN_TAB.1 - SEARCH.1 - 2, SEARCH);
            bar.place(tree, &tabs, search, (PAD, chips_y), foot);
        }
    }

    /// The pages' containers with their slot counts.
    fn page_sizes(&self) -> Vec<(Option<usize>, u32)> {
        let sizes = self.sizes.get();
        self.pages
            .iter()
            .enumerate()
            .map(|(i, (container, _))| (*container, sizes.get(i).copied().unwrap_or(0)))
            .collect()
    }

    fn containers(&self) -> Vec<Option<usize>> {
        self.pages.iter().map(|(c, _)| *c).collect()
    }

    /// A press on one of the drawer's own parts (a filter chip, the All tab); whether it took it.
    pub(crate) fn press(&self, tree: &mut UiTree, element: usize) -> bool {
        if self.bar.as_ref().is_some_and(|bar| bar.press(element)) {
            return true;
        }
        if let Some(all) = &self.all {
            if all.press(tree, element, &self.containers()) {
                self.arrange(tree);
                return true;
            }
        }
        false
    }

    /// A wheel turn over the All view scrolls it; whether it took the turn.
    pub(crate) fn wheel(&self, tree: &mut UiTree, action: u32) -> bool {
        let Some(all) = &self.all else {
            return false;
        };
        if all.wheel(tree, action) {
            all.lay_out(tree, &self.page_sizes(), self.grid(tree));
            return true;
        }
        false
    }

    /// The grid's box in the pack area, as the last layout put it.
    fn grid(&self, tree: &UiTree) -> UiBox {
        let (_, height) = tree.display;
        let h = (height - TOP - BOTTOM).max(400);
        let equipment_y = HEADER + GAP + MONEY + GAP;
        let rail_y = equipment_y + DOLL.1 + 2;
        let area_h = h - (rail_y + RAIL - TAB_OVERLAP) - PAD;
        let grid_y = MAIN_TAB.1 + 3 + CHIPS;
        sized(PAD, grid_y, (WIDTH - 2 * PAD, area_h - grid_y - FOOT - 2))
    }

    /// The drawer's own parts set for the backpack as it is (every frame).
    pub(crate) fn tick(
        &self,
        tree: &mut UiTree,
        backpack: &crate::BackpackUi,
        doll: Option<&crate::PaperDollUi>,
        source: &dyn crate::item_display::InventorySource,
    ) {
        let mut sizes = [0; 4];
        for (size, page) in sizes.iter_mut().zip(&backpack.packs) {
            *size = page.max_size();
        }
        let all_on = self.all.as_ref().is_some_and(|a| a.is_on());
        let left = self
            .all
            .as_ref()
            .is_some_and(|a| a.follow(tree, &self.containers()));
        if left || sizes != self.sizes.get() {
            self.sizes.set(sizes);
            self.arrange(tree);
        }
        if let Some(bar) = &self.bar {
            bar.tick(tree, backpack, source, all_on && !left);
        }
        if let Some(ring) = &self.ring {
            let target = self.ring_target(tree, backpack, doll, source);
            ring.show(tree, target);
        }
    }

    /// Where the item ring opens: the selected item's slot in a page shown (the grid's box its
    /// bounds), or its worn slot (the drawer's), while the drawer shows and nothing is dragged.
    fn ring_target(
        &self,
        tree: &UiTree,
        backpack: &crate::BackpackUi,
        doll: Option<&crate::PaperDollUi>,
        source: &dyn crate::item_display::InventorySource,
    ) -> Option<crate::modern_item_ring::RingTarget> {
        let item = source.selection();
        if item == 0 || tree.dragged().is_some() || !shown(tree, self.field) {
            return None;
        }
        let stack = source.item(item)?;
        let mut target = crate::modern_item_ring::RingTarget {
            item,
            slot: UiBox::default(),
            bounds: UiBox::default(),
            towards: -90.0,
            worn: false,
            split: stack.stackable && 1 < stack.quantity,
            melt: true,
            equip: source.preferred_location(item) != 0,
        };
        if let Some(slot) = doll.and_then(|d| d.worn_slot(item)) {
            // A worn item: sideways, towards the doll.
            let at = tree.screen_box(slot);
            let frame = self.doll.map(|f| tree.screen_box(f))?;
            target.slot = at;
            target.bounds = tree.screen_box(self.field);
            target.towards = if at.x1 < frame.x0 { 0.0 } else { 180.0 };
            target.worn = true;
            return Some(target);
        }
        let display = backpack
            .packs
            .iter()
            .filter(|p| p.list.is_some_and(|l| shown(tree, l)))
            .find_map(|p| Some((p.list?, p.items.iter().find(|d| d.item == item)?)))?;
        let (list, display) = display;
        let at = tree.screen_box(display.background);
        // The grid's box: the All view's, else the page list's.
        let bounds = match self.all.as_ref().filter(|a| a.is_on()) {
            Some(_) => tree.screen_box(tree.el(tree.el(list).parent_index()?).parent_index()?),
            None => tree.screen_box(list),
        };
        if at.y0 < bounds.y0 || at.y1 > bounds.y1 {
            return None;
        }
        let half = crate::modern_art::RING_ART / 2;
        let up = (at.y0 + at.y1) / 2 - bounds.y0;
        let down = bounds.y1 - (at.y0 + at.y1) / 2;
        target.towards = if up >= 40 + half || up >= down {
            -90.0
        } else {
            90.0
        };
        target.slot = at;
        target.bounds = bounds;
        Some(target)
    }

    /// A press on one of the item ring's buttons: its action and the item.
    pub(crate) fn ring_press(
        &self,
        element: usize,
    ) -> Option<(crate::modern_item_ring::RingAction, u64)> {
        self.ring.as_ref()?.press(element)
    }

    /// A page's list filling its box but for the scroll bar at its right: the bar's arrows at its
    /// ends and its track between them.
    fn place_page(&self, tree: &mut UiTree, container: usize, grid: UiBox) {
        let (w, h) = (grid.width(), grid.height());
        let list_w = w - SCROLL - 1;
        for child in tree.children(container).to_vec() {
            if tree.el(child).id() == tokenize("ContainerListBox") {
                put(tree, Some(child), sized(0, 0, (list_w, h)));
                continue;
            }
            // The bar's parts by where retail put them: the top arrow, the bottom one, the track.
            let r = tree.el(child).rect;
            let arrow = r.height() <= ARROW_MAX;
            let at = if arrow && r.y0 <= 1 {
                sized(w - SCROLL, 0, (SCROLL, SCROLL))
            } else if arrow {
                sized(w - SCROLL, h - SCROLL, (SCROLL, SCROLL))
            } else {
                sized(w - SCROLL, SCROLL, (SCROLL, h - 2 * SCROLL))
            };
            put(tree, Some(child), at);
            // The track's pieces and the thumb's as narrow as the bar.
            let mut under = tree.children(child).to_vec();
            while let Some(part) = under.pop() {
                under.extend(tree.children(part).iter().copied());
                let r = tree.el(part).rect;
                if r.width() != SCROLL {
                    tree.resize_to(part, SCROLL, r.height());
                }
            }
        }
    }
}

/// Whether `element` and every element over it show.
fn shown(tree: &UiTree, element: usize) -> bool {
    let mut at = Some(element);
    while let Some(e) = at {
        let Some(el) = tree.get(e) else {
            return false;
        };
        if !el.visible {
            return false;
        }
        at = el.parent_index();
    }
    true
}

/// The modern drawer's doll frame, which the paper doll's viewport fills; `None` in the classic
/// style.
pub(crate) fn doll_frame(tree: &UiTree) -> Option<usize> {
    let id = tokenize("ModernDrawerDoll");
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, id))
}

/// A part of the design under `field`: a copy of `template` (a plain field of the layout) with no
/// states, named `id`, its art `image` (in nine pieces with corners `nine` when given), taking no
/// pointer, at depth `z`.
pub(crate) fn make_part(
    tree: &mut UiTree,
    field: usize,
    template: Option<usize>,
    id: &str,
    image: u32,
    nine: Option<i32>,
    z: i32,
) -> Option<usize> {
    let template = template?;
    let layout = Arc::clone(&tree.el(field).layout);
    let mut desc = kept_in_place(&tree.el(template).reference);
    desc.id = tokenize(id);
    desc.states.clear();
    desc.z_level = z as u16;
    let part = tree.create_element(layout, Arc::new(desc), false)?;
    tree.set_parent(part, Some(field));
    // The part's art draws whole: not at the template's alpha value (an attribute `set_image`
    // reads while the element has none); a window's opacity sets one later, as on retail's parts.
    tree.el_mut(part)
        .attributes
        .remove(&crate::token::ALPHA_VALUE);
    tree.el_mut(part).alpha_value = -1.0;
    tree.set_image(part, image, DRAW_OVER);
    if let Some(border) = nine {
        tree.set_int(part, NINE_SLICE, border);
    }
    tree.set_bool(part, MOUSE_INVISIBLE, true);
    tree.set_z_level(part, z);
    Some(part)
}
