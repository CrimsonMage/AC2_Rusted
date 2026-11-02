//! The classic inventory resizes vertically (client exception, `AGENTS.md`). Retail's
//! `InventoryField` (in `RightSidePanel`) has no resize grip and no `allow_resize`: its layout
//! was drawn at one size. Here its top edge carries a resize grip (a `UIElement_Resizebar`
//! with `border_top`, cloned from the shortcut bar's top grip; the window sits on the bottom
//! edge of the display, so it grows upward) and the window grows by whole pack rows: the pack
//! page's list gets the rows (`UIElement_ListBox::ResizeTo`, `ContainerDisplay`), the bottom
//! border stays on the bottom edge, and the paper doll, the money bag and the page tabs keep their
//! size (`UIElement::UpdateSizeAndPosition` anchors, edited in the layout copy below). The other
//! right-side panels keep their size in the taller window.
//!
//! Resizing, the limits and the saved size are retail's own machinery (`resize.rs`,
//! `UIElement::MouseResizeElement`, `ApplyLocation`'s `allow_resize`); the row step is the one
//! client addition ([`RESIZE_HEIGHT_STEP`]).

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::token::{tokenize, ALLOW_RESIZE, MAX_HEIGHT, MIN_HEIGHT, RESIZEBAR};

/// The root the panels live in.
const RIGHT_PANEL: u32 = tokenize("RightSidePanel");
/// The inventory page, a child of the root.
const INVENTORY: u32 = tokenize("InventoryField");
/// The shortcut bar's top grip, whose description the new grip is made from.
const GRIP_TEMPLATE: u32 = 0xC9A6_F0D2;
/// The grip's height in pixels (the header band's resize margin is the grip's 6 pixels).
const GRIP_HEIGHT: i16 = 8;
/// The new grip's id, `Tokenize("InventoryResizeBar")` (a client element: no layout has it, so the
/// name is not spelled where the layout-name scan reads it).
pub(crate) const GRIP: u32 = 0xB761BA58;
/// The inventory page's three parts by layout id: the paper doll half, the bottom border and the
/// pack half.
const DOLL_HALF: u32 = 0x27ED_4815;
const BORDER_STRIP: u32 = 0x523D_41C6;
const PACK_HALF: u32 = 0x1BB9_C51C;
/// The page tabs beside the pack list (`PackMainButton` and its three siblings).
const PACK_BUTTONS: [&str; 4] = [
    "PackMainButton",
    "PackOneButton",
    "PackTwoButton",
    "PackThreeButton",
];
/// A window's height moves in whole steps of this many pixels (`resize_height_step`): one pack
/// row of the list (`ContainerDisplay`'s 36-pixel cells).
pub const RESIZE_HEIGHT_STEP: i32 = 36;
/// The window attribute naming the step (client-only; no retail layout has it).
pub(crate) const RESIZE_HEIGHT_STEP_ATTRIBUTE: u32 = tokenize("resize_height_step");
/// How many rows the window may gain.
const MAX_EXTRA_ROWS: i32 = 24;

fn attribute(id: u32, value: AttributeValue) -> AttributeDesc {
    AttributeDesc { id, value }
}

fn with_attribute(state: &mut StateDesc, id: u32, value: AttributeValue) {
    state.attributes.retain(|a| a.id != id);
    state.attributes.push(attribute(id, value));
}

/// `desc` with the bottom edge kept (0: the side stays where it is when the parent grows).
fn fixed_bottom(desc: &Arc<ElementDesc>) -> Arc<ElementDesc> {
    let mut d = ElementDesc::clone(desc);
    d.edges[3] = 0;
    Arc::new(d)
}

/// `desc` and every element below it with the inventory page's parts anchored for a taller
/// window: they are references in the page's description (`UIElement::UpdateSizeAndPosition`
/// reads the reference's edges), found by their layout ids.
fn retune(desc: &Arc<ElementDesc>) -> Arc<ElementDesc> {
    let mut d = ElementDesc::clone(desc);
    match d.id {
        // The doll keeps its height; the border follows the bottom edge down; the pack half
        // takes the growth (its top stays); the tabs keep their size.
        DOLL_HALF => d.edges[3] = 0,
        BORDER_STRIP => {
            d.edges[1] = 2;
            d.edges[3] = 1;
        }
        PACK_HALF => d.edges[1] = 0,
        id if PACK_BUTTONS.iter().any(|name| tokenize(name) == id) => d.edges[3] = 0,
        _ => {}
    }
    for state in &mut d.states {
        for child in &mut state.children {
            *child = retune(child);
        }
    }
    Arc::new(d)
}

/// `RightSidePanel`'s description with the grip and the resize attributes added.
fn restyle_panel(panel: &ElementDesc, grip: &ElementDesc) -> ElementDesc {
    let mut d = panel.clone();
    let rect = d.rect;
    let height = i32::from(rect.y1) - i32::from(rect.y0) + 1;
    let default_state = d.default_state;
    for state in &mut d.states {
        for child in &mut state.children {
            // The other pages keep their size in the taller window.
            if child.id != INVENTORY {
                *child = fixed_bottom(child);
            }
        }
        if state.id == default_state {
            with_attribute(state, ALLOW_RESIZE, AttributeValue::Bool(true));
            with_attribute(state, MIN_HEIGHT, AttributeValue::Int(height));
            with_attribute(
                state,
                MAX_HEIGHT,
                AttributeValue::Int(height + MAX_EXTRA_ROWS * RESIZE_HEIGHT_STEP),
            );
            with_attribute(
                state,
                RESIZE_HEIGHT_STEP_ATTRIBUTE,
                AttributeValue::Int(RESIZE_HEIGHT_STEP),
            );
            // The grip along the top edge, as wide as the window.
            let mut grip = grip.clone();
            grip.id = GRIP;
            // (In the window's own coordinates; `rect` is its place in the layout.)
            grip.rect = Rect {
                x0: 0,
                y0: 0,
                x1: rect.x1 - rect.x0,
                y1: GRIP_HEIGHT - 1,
            };
            grip.edges = [1, 1, 1, 0];
            // Over the pages (a lower z is nearer the pointer).
            grip.z_level = 0;
            for grip_state in &mut grip.states {
                grip_state.rect = grip.rect;
                grip_state.z_level = 0;
            }
            state.children.push(Arc::new(grip));
        }
    }
    d
}

/// The toolbar layout with the inventory made resizable (the classic style; none of the layout
/// is changed when it lacks the shortcut bar's grip or the panel).
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    let Some(grip) = layout
        .find_element(GRIP_TEMPLATE)
        .filter(|g| g.kind == RESIZEBAR)
    else {
        return layout.clone();
    };
    let mut out = layout.clone();
    for element in &mut out.elements {
        *element = if element.id == RIGHT_PANEL {
            retune(&Arc::new(restyle_panel(element, &grip)))
        } else {
            retune(element)
        };
    }
    out
}

impl UiTree {
    /// The grip lives along the inventory page's bottom: it is there and takes presses only while
    /// that page is shown.
    pub(crate) fn sync_inventory_grip(&mut self) {
        let Some(root) = self
            .roots()
            .iter()
            .copied()
            .find(|&r| self.el(r).id() == RIGHT_PANEL)
        else {
            return;
        };
        let (Some(grip), Some(page)) = (
            self.child_recursive(root, GRIP),
            self.child_recursive(root, INVENTORY),
        ) else {
            return;
        };
        let shown = self.region_visible(page);
        if self.el(grip).visible != shown {
            self.set_visible(grip, shown);
        }
    }
}
