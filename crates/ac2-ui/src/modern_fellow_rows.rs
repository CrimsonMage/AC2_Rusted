//! The modern style's fellow mini panel rows (AGENTS.md: the HUD's two styles; `FellowRows::
//! set_modern`): each row 24 pixels tall with its health and vigor bars 10 tall, so the numbers
//! the bars carry fit in them, and a fine rule along the foot of each row, so the members' bars
//! part at a glance. Retail's rows were 15 tall with 5-pixel bars, edge to edge.

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::{ElementDesc, LayoutDesc, Rect};
use crate::modern_art as art;
use crate::modern_inventory::{keep, make_part};
use crate::token::MOUSE_INVISIBLE;

/// `FellowMiniPanelTemplate`'s parts: the row, the bars' holder, the health bar's and the vigor
/// bar's groups, the name.
const ROW: u32 = 0xA66F_644C;
const BARS: u32 = 0xD81F_CD7E;
const HEALTH: u32 = 0x9D3E_044E;
const VIGOR: u32 = 0x281F_EB46;
const NAME: u32 = 0x7225_DB19;
/// The row's height and a bar's.
const ROW_HEIGHT: i16 = 24;
const BAR_HEIGHT: i16 = 10;

fn rect(x0: i16, y0: i16, x1: i16, y1: i16) -> Rect {
    Rect { x0, y0, x1, y1 }
}

/// The gameplay layout with the mini panel's row template made taller (every id, state and edge
/// kept): the row 24 tall, the bars 10 tall each, the name centred beside them.
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    /// `d` and everything under it as tall as a bar (the bar's own parts fill its group).
    fn bar(d: &ElementDesc, y0: i16) -> ElementDesc {
        let mut d = d.clone();
        d.rect = rect(d.rect.x0, y0, d.rect.x1, y0 + BAR_HEIGHT - 1);
        for state in &mut d.states {
            for child in &mut state.children {
                *child = Arc::new(bar(child, 0));
            }
        }
        d
    }
    fn walk(d: &ElementDesc) -> ElementDesc {
        let mut d = d.clone();
        match d.id {
            HEALTH => return bar(&d, 0),
            VIGOR => return bar(&d, BAR_HEIGHT + 1),
            ROW => d.rect = rect(0, 0, d.rect.x1, ROW_HEIGHT - 1),
            BARS => d.rect = rect(d.rect.x0, 1, d.rect.x1, 2 * BAR_HEIGHT + 1),
            NAME => d.rect = rect(d.rect.x0, 5, d.rect.x1, ROW_HEIGHT - 6),
            _ => {}
        }
        for state in &mut d.states {
            for child in &mut state.children {
                *child = Arc::new(walk(child));
            }
        }
        d
    }
    let mut layout = layout.clone();
    for element in &mut layout.elements {
        if element.id == crate::token::tokenize("FellowMiniPanelTemplate") {
            *element = Arc::new(walk(element));
        }
    }
    layout
}

/// A rule along the foot of the row `root` (a copy of one of its plain parts), taking no pointer.
pub(crate) fn add_rule(tree: &mut UiTree, root: usize) {
    let mut all = vec![root];
    let mut i = 0;
    while i < all.len() {
        let kids = tree.children(all[i]).to_vec();
        all.extend(kids);
        i += 1;
    }
    let template = all
        .into_iter()
        .skip(1)
        .find(|&e| tree.el(e).reference.kind == tree.el(e).desc.kind);
    let Some(rule) = make_part(
        tree,
        root,
        template,
        "ModernFellowRule",
        art::FELLOW_RULE,
        Some(art::FELLOW_RULE_BORDER),
        -1,
    ) else {
        return;
    };
    keep(tree, rule);
    tree.set_bool(rule, MOUSE_INVISIBLE, true);
    let r = tree.el(root).rect;
    tree.resize_to(rule, r.width() - 4, 1);
    tree.move_to(rule, 2, r.height() - 1);
}
