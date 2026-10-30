//! The modern style's vitals (AGENTS.md: the HUD's two styles): `gmStatusbarUI`'s
//! `StatusbarField` laid out as the design's panel. Retail drew the vitals in a stone frame
//! (`0x41003A7D`..`0x41003A80`, the end cap `0x41006018`) with the indicators in a block at their
//! left. Here, on teal glass, the health, vigor and focus bars sit in dark wells, either side by
//! side over the experience bar (the default) or stacked over it, with the effects under them and
//! the indicators (net, PK, vitae) in a column at the right. A switch at the panel's corner, shown
//! while the pointer is over it, changes the layout. The fields are retail's (`gmVitalsUI`'s
//! meters and texts, `gmEffectsUI`'s list, the indicators' fields), so what they show and their
//! presses stay retail's; only their places change.

use std::cell::Cell;

use crate::element::{UiBox, UiTree};
use crate::modern_art as art;
use crate::modern_inventory::{keep, make_part};
use crate::mouse::UiMessage;
use crate::token::{tokenize, MOUSE_INVISIBLE, MSG_MOUSE_PRESS_TOP};

/// The panel's padding (left and right, top, bottom) and the gaps between its parts.
const PAD: (i32, i32, i32) = (7, 5, 6);
const GAP: i32 = 3;
/// The bars' row width side by side and the column's stacked; the bars' heights (health, the
/// others, experience).
const SIDE_WIDTH: i32 = 272;
const STACKED_WIDTH: i32 = 212;
const HEALTH: i32 = 11;
const BAR: i32 = 9;
const EXPERIENCE: i32 = 5;
/// The indicators' column: its gap from the bars and each field's size.
const INDICATOR_GAP: i32 = 8;
const INDICATOR: (i32, i32) = (20, 32);
/// An effect's room in the effects' rows when its list has none to measure, and the gap between
/// rows.
const EFFECT_PITCH: (i32, i32) = (29, 29);

/// `StatusbarField`'s unnamed children: the frame's end cap, the indicators' block, the vitals'
/// frame and its overlay, the effects' area.
const END_CAP: u32 = 0xE11E_3A97;
const INDICATORS: u32 = 0x17F7_D5C2;
const FRAME: u32 = 0x9570_5B41;
const FRAME_OVERLAY: u32 = 0x25C8_20CE;
const EFFECTS_AREA: u32 = 0x61C4_B08F;
/// The vitals' block holding the frame (`0x8264CFD6`).
const VITALS_BLOCK: u32 = 0x8264_CFD6;

/// The vitals' layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VitalsLayout {
    #[default]
    SideBySide,
    Stacked,
}

/// The panel's parts and state.
#[derive(Debug, Clone)]
pub struct ModernVitals {
    root: usize,
    /// Health, vigor, focus, experience.
    fields: [Option<usize>; 4],
    wells: [Option<usize>; 4],
    glass: Option<usize>,
    switch: Option<usize>,
    indicators: Option<usize>,
    /// The indicators' fields' holders, top to bottom.
    holders: Vec<usize>,
    effects: Option<usize>,
    /// `gmEffectsUI`'s list.
    effect_list: Option<usize>,
    /// What the panel was last laid out for: the effects' count and the indicators shown.
    content: std::cell::RefCell<Option<(usize, Vec<bool>)>>,
    layout: Cell<VitalsLayout>,
    laid_out: Cell<Option<VitalsLayout>>,
    lit: Cell<Option<bool>>,
}

/// Whether `element` shows and draws something: its own image, or a shown child that does.
fn draws(tree: &UiTree, element: usize) -> bool {
    let Some(el) = tree.get(element) else {
        return false;
    };
    el.visible && (el.image().is_some() || tree.children(element).iter().any(|&c| draws(tree, c)))
}

fn place(tree: &mut UiTree, element: Option<usize>, (x, y, w, h): (i32, i32, i32, i32)) {
    let Some(element) = element else {
        return;
    };
    let at = UiBox::new(x, y, x + w - 1, y + h - 1);
    if tree.el(element).rect != at {
        tree.resize_to(element, w, h);
        tree.move_to(element, x, y);
    }
}

impl ModernVitals {
    /// The status bar laid out as the panel: its frame art cleared, its fields kept where they
    /// are put, the glass, the wells and the switch made. `None` without the status bar.
    pub fn adopt(tree: &mut UiTree) -> Option<Self> {
        let root = tree
            .roots()
            .iter()
            .copied()
            .find(|&r| tree.el(r).id() == tokenize("StatusbarField"))?;
        let find = |tree: &UiTree, id: u32| tree.child_recursive(root, id);
        let named = |tree: &UiTree, name: &str| find(tree, tokenize(name));
        let fields = [
            named(tree, "HealthField"),
            named(tree, "VigorField"),
            named(tree, "FocusField"),
            named(tree, "ExperienceField"),
        ];
        let indicators = find(tree, INDICATORS);
        let holders: Vec<usize> = indicators
            .map(|i| {
                let mut h = tree.children(i).to_vec();
                h.sort_by_key(|&c| tree.el(c).rect.x0);
                h
            })
            .unwrap_or_default();
        let effects = find(tree, EFFECTS_AREA);
        for frame in [END_CAP, INDICATORS, FRAME, FRAME_OVERLAY] {
            if let Some(frame) = find(tree, frame) {
                tree.clear_image(frame);
            }
        }
        // The fields, the indicators and the effects move up to the panel itself (retail nests
        // them in the frame's fields, which would clip them); the frame's emptied fields hide.
        for &part in fields.iter().flatten().chain(&indicators).chain(&effects) {
            if tree.el(part).parent_index() != Some(root) {
                tree.set_parent(part, Some(root));
            }
        }
        for frame in [FRAME, VITALS_BLOCK] {
            if let Some(frame) = find(tree, frame) {
                tree.set_visible(frame, false);
            }
        }
        let mut placed: Vec<usize> = fields.iter().flatten().copied().collect();
        placed.extend(holders.iter().copied());
        placed.extend([root].into_iter().chain(indicators).chain(effects));
        placed.extend(find(tree, FRAME));
        placed.extend(find(tree, FRAME_OVERLAY));
        placed.extend(find(tree, END_CAP));
        for element in placed {
            keep(tree, element);
        }
        // The glass, the wells and the switch, copies of the end cap (a plain field).
        let template = find(tree, END_CAP);
        let part = |tree: &mut UiTree, id: &str, image: u32, nine: Option<i32>, z: i32| {
            let part = make_part(tree, root, template, id, image, nine, z)?;
            keep(tree, part);
            Some(part)
        };
        let glass = part(
            tree,
            "ModernVitalsGlass",
            art::VITALS_GLASS,
            Some(art::VITALS_BORDER),
            900,
        );
        let wells = [0, 1, 2, 3].map(|i| {
            part(
                tree,
                &format!("ModernVitalsWell{i}"),
                art::BAR_WELL,
                Some(art::WELL_BORDER),
                800,
            )
        });
        let switch = part(tree, "ModernVitalsLayout", art::LAYOUT_SWITCH, None, 0);
        if let Some(switch) = switch {
            tree.set_bool(switch, MOUSE_INVISIBLE, false);
            tree.set_visible(switch, false);
        }
        // The frame's own fields hide (their art is gone; the fields are moved out of them).
        let vitals = Self {
            root,
            fields,
            wells,
            glass,
            switch,
            indicators,
            holders,
            effects,
            effect_list: find(tree, tokenize("EffectListBox")),
            content: Default::default(),
            layout: Cell::new(VitalsLayout::default()),
            laid_out: Cell::new(None),
            lit: Cell::new(None),
        };
        vitals.arrange(tree);
        Some(vitals)
    }

    #[cfg(test)]
    pub fn layout(&self) -> VitalsLayout {
        self.layout.get()
    }

    /// The layout set (laid out at once).
    pub fn set_layout(&self, tree: &mut UiTree, layout: VitalsLayout) {
        self.layout.set(layout);
        self.arrange(tree);
    }

    /// Every part put where the layout has it.
    fn arrange(&self, tree: &mut UiTree) {
        let layout = self.layout.get();
        self.laid_out.set(Some(layout));
        let (x0, y0) = (PAD.0, PAD.1);
        let (width, bars): (i32, [(i32, i32, i32, i32); 4]) = match layout {
            VitalsLayout::SideBySide => {
                let w = (SIDE_WIDTH - 2 * 4) / 3;
                let row = HEALTH + GAP;
                (
                    SIDE_WIDTH,
                    [
                        (x0, y0, w, HEALTH),
                        (x0 + w + 4, y0, w, BAR),
                        (x0 + 2 * (w + 4), y0, w, BAR),
                        (x0, y0 + row, SIDE_WIDTH, EXPERIENCE),
                    ],
                )
            }
            VitalsLayout::Stacked => {
                let w = STACKED_WIDTH;
                let v = y0 + HEALTH + GAP;
                let f = v + BAR + GAP;
                (
                    w,
                    [
                        (x0, y0, w, HEALTH),
                        (x0, v, w, BAR),
                        (x0, f, w, BAR),
                        (x0, f + BAR + GAP, w, EXPERIENCE),
                    ],
                )
            }
        };
        for (i, &(x, y, w, h)) in bars.iter().enumerate() {
            place(tree, self.wells[i], (x, y, w, h));
            // The field inside its well's line.
            place(tree, self.fields[i], (x + 1, y + 1, w - 2, h - 2));
        }
        let bottom = bars[3].1 + EXPERIENCE + GAP;
        // The effects in as many rows as they take (none: no room).
        let (count, shown) = self.content(tree);
        let pitch = self.effect_pitch(tree);
        let per_row = (width / pitch.0).max(1) as usize;
        let rows = count.div_ceil(per_row) as i32;
        let effects = rows * pitch.1;
        place(tree, self.effects, (x0, bottom, width, effects.max(1)));
        // The indicators shown in a column at the right, packed from the top.
        let ix = x0 + width + INDICATOR_GAP;
        let mut y = 0;
        for (&holder, &on) in self.holders.iter().zip(&shown) {
            place(tree, Some(holder), (0, y, INDICATOR.0, INDICATOR.1));
            if on {
                y += INDICATOR.1 + GAP;
            }
        }
        let column = (y - GAP).max(0);
        place(tree, self.indicators, (ix, y0, INDICATOR.0, column.max(1)));
        let w = ix + INDICATOR.0 + PAD.0;
        let h = (bottom + effects).max(y0 + column) + PAD.2;
        let root = tree.el(self.root).rect;
        if (root.width(), root.height()) != (w, h) {
            tree.resize_to(self.root, w, h);
        }
        place(tree, self.glass, (0, 0, w, h));
        place(
            tree,
            self.switch,
            (w - art::SWITCH - 2, 2, art::SWITCH, art::SWITCH),
        );
    }

    /// The effects' count and whether each indicator holder's field shows.
    fn content(&self, tree: &UiTree) -> (usize, Vec<bool>) {
        let count = self.effect_list.map_or(0, |l| tree.list_items(l).len());
        let shown = self.holders.iter().map(|&h| draws(tree, h)).collect();
        (count, shown)
    }

    /// An effect's room (its item's size and the gap), measured from the list's first item.
    fn effect_pitch(&self, tree: &UiTree) -> (i32, i32) {
        self.effect_list
            .and_then(|l| tree.list_items(l).first().copied())
            .map(|item| {
                let r = tree.el(item).rect;
                (r.width() + GAP, r.height() + GAP)
            })
            .unwrap_or(EFFECT_PITCH)
    }

    /// The switch shown while the pointer is over the panel, lit while it is over the switch;
    /// the panel laid out again when its effects or indicators change.
    pub fn tick(&self, tree: &mut UiTree) {
        let content = self.content(tree);
        if self.content.borrow().as_ref() != Some(&content) {
            *self.content.borrow_mut() = Some(content);
            self.arrange(tree);
        }
        let Some(switch) = self.switch else {
            return;
        };
        let over = |tree: &UiTree, e: usize| {
            let at = tree.screen_box(e);
            let (x, y) = tree.pointer();
            x >= at.x0 && x <= at.x1 && y >= at.y0 && y <= at.y1
        };
        let shown = tree.el(self.root).visible && over(tree, self.root);
        if tree.el(switch).visible != shown {
            tree.set_visible(switch, shown);
        }
        let lit = shown && over(tree, switch);
        if self.lit.get() != Some(lit) {
            self.lit.set(Some(lit));
            let image = if lit {
                art::LAYOUT_SWITCH_LIT
            } else {
                art::LAYOUT_SWITCH
            };
            tree.set_image(switch, image, crate::compose::DRAW_OVER);
        }
    }

    /// A press on the switch changes the layout. Whether it took the press.
    pub fn message(&self, tree: &mut UiTree, message: &UiMessage) -> bool {
        if message.message != MSG_MOUSE_PRESS_TOP
            || message.element != self.switch
            || self.switch.is_none()
        {
            return false;
        }
        let next = match self.layout.get() {
            VitalsLayout::SideBySide => VitalsLayout::Stacked,
            VitalsLayout::Stacked => VitalsLayout::SideBySide,
        };
        self.set_layout(tree, next);
        true
    }
}
