//! The shortcut bar's look: modern action-bar slots over the same elements.
//!
//! Retail's bar (`ShortcutField` of the toolbar layout) drew one grid image (`0x41008D29`, its
//! first row's cells painted with the digits 1 to 0) behind 32-pixel slots (`ShortcutTemplate`)
//! that had no frame of their own. Here the toolbar layout's two descriptions are rewritten
//! before the bar is made ([`restyle`]), keeping every element, state, message and attribute
//! `gmShortcutUI` reads:
//!
//! - each slot is 38 pixels: a framed inset of its own (the recipe button's art, `0x410080E1`,
//!   a dark well with a bevelled rim) with the icon at its native 32 pixels inside it, so the
//!   slots read apart;
//! - its overlay frame (`0x41000B8B`, pressed `0x41000A64`, ghosted `0x41000A65`) is drawn in
//!   nine pieces at the slot's size and gains a hover look: the slot button takes a rollover
//!   state that frames the slot in the selection corners (`0x4100219D`). The icon and the frame
//!   let the pointer through, so the button is the top region and rolls over (retail's never
//!   did: its icon took the pointer);
//! - the key's digit stands in the slot's top-left corner (`ShortcutKeyLabel`, the number
//!   display's font), where retail's grid painted it; the number display (`NumberDisplay`, empty
//!   slots while numbers are on) stays at the bottom right;
//! - the time-left meter's fill is a dark veil (a plain black square, `0x41005979`, at 0.62
//!   alpha) in place of retail's checkered fill (`0x410016EA`), so a skill on reset darkens
//!   while it waits;
//! - the bar's backing is the toolbar cluster's plate (`0x41007F71`, nine pieces), sized to the
//!   slots, with the same `row_height` and `border_size` arithmetic
//!   (`gmShortcutUI::RefreshShortcutHeight`).

use std::sync::Arc;

use crate::compose::NINE_SLICE;
use crate::layout::{
    AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc, Rect, StateDesc,
};
use crate::token::{
    tokenize, ALPHA_VALUE, BUTTON_ROLLOVER, MOUSE_INVISIBLE, SIZE_IMAGE_TO_OWNER, STATELESS,
    STATE_NORMAL_ROLLOVER,
};

/// A slot's size (the list's pitch) and the icon's inset in it.
pub(crate) const SLOT: i32 = 38;
const ICON_INSET: i32 = 3;
/// The slot's well, its overlay's hover frame, the meter's veil and the bar's plate.
pub(crate) const SLOT_WELL: u32 = 0x4100_80E1;
pub(crate) const SLOT_HOVER: u32 = 0x4100_219D;
const BAR_PLATE: u32 = 0x4100_7F71;
/// The bar's padding around its slots (the `border_size` it keeps is twice it).
const BAR_PAD: i32 = 6;
const SLOTS_PER_ROW: i32 = 10;
/// The veil: a plain black square of the radar's (`0x41005979`), at this alpha.
const VEIL: u32 = 0x4100_5979;
const VEIL_ALPHA: f32 = 0.62;

fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
    Rect {
        x0: x0 as i16,
        y0: y0 as i16,
        x1: x1 as i16,
        y1: y1 as i16,
    }
}

/// The icon's box in the slot, `inset` pixels in from the well's rim.
fn inner(inset: i32) -> Rect {
    rect(inset, inset, SLOT - 1 - inset, SLOT - 1 - inset)
}

fn attribute(id: u32, value: AttributeValue) -> AttributeDesc {
    AttributeDesc { id, value }
}

/// `desc`'s attribute `id` set to `value` in its stateless state.
fn set_attribute(desc: &mut ElementDesc, id: u32, value: AttributeValue) {
    if let Some(state) = desc.states.iter_mut().find(|s| s.id == STATELESS) {
        state.attributes.retain(|a| a.id != id);
        state.attributes.push(attribute(id, value));
    }
}

/// Each image of `desc`'s states replaced by `map` (file, draw mode).
fn map_images(desc: &mut ElementDesc, map: impl Fn(u32, u32) -> (u32, u32)) {
    for state in &mut desc.states {
        for media in &mut state.media {
            if let MediaDesc::Image {
                file, draw_mode, ..
            } = media
            {
                (*file, *draw_mode) = map(*file, *draw_mode);
            }
        }
    }
}

/// `edit` applied to each element of `desc`'s tree, children first.
fn edit_tree(desc: &ElementDesc, edit: &dyn Fn(&mut ElementDesc)) -> ElementDesc {
    let mut desc = desc.clone();
    for state in &mut desc.states {
        for child in &mut state.children {
            *child = Arc::new(edit_tree(child, edit));
        }
    }
    edit(&mut desc);
    desc
}

/// The toolbar layout with the bar and its slot restyled; every other element is shared.
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    let mut layout = layout.clone();
    for element in &mut layout.elements {
        if element.id == tokenize("ShortcutField") {
            *element = Arc::new(bar(element));
        } else if let Some(template) = template(element) {
            *element = Arc::new(template);
        }
    }
    layout
}

/// The bar: as wide as ten slots and its padding, one row tall, on the plate.
fn bar(desc: &ElementDesc) -> ElementDesc {
    let width = SLOT * SLOTS_PER_ROW + 2 * BAR_PAD;
    let height = SLOT + 2 * BAR_PAD;
    let edit = |d: &mut ElementDesc| {
        if d.id == tokenize("ShortcutField") {
            // Centred on the bottom edge as retail's, one pixel up from it.
            let (x0, y1) = (512 - width / 2, i32::from(d.rect.y1));
            d.rect = rect(x0, y1 + 1 - height, x0 + width - 1, y1);
            map_images(d, |_, _| (BAR_PLATE, 2));
            set_attribute(d, NINE_SLICE, AttributeValue::Int(8));
            set_attribute(d, tokenize("border_size"), AttributeValue::Int(2 * BAR_PAD));
            set_attribute(d, tokenize("row_height"), AttributeValue::Int(SLOT));
            set_attribute(d, tokenize("min_height"), AttributeValue::Int(height));
            set_attribute(
                d,
                tokenize("max_height"),
                AttributeValue::Int(5 * SLOT + 2 * BAR_PAD),
            );
        } else if d.edges == [1, 1, 1, 1] {
            // The list's holder: the slots' area inside the padding.
            d.rect = rect(
                BAR_PAD,
                BAR_PAD,
                BAR_PAD + SLOT * SLOTS_PER_ROW - 1,
                BAR_PAD + SLOT - 1,
            );
        } else if d.edges == [1, 1, 1, 2] {
            // The resize line's cursor region: the whole bar.
            d.rect = rect(0, 0, width - 1, height - 1);
        }
    };
    edit_tree(desc, &edit)
}

/// Whether one of `desc`'s states shows image `file`.
fn has_image(desc: &ElementDesc, file: u32) -> bool {
    desc.states.iter().any(|state| {
        state
            .media
            .iter()
            .any(|m| matches!(m, MediaDesc::Image { file: f, .. } if *f == file))
    })
}

/// A state `id` added to `desc` (once): showing `image` (blended by its alpha), and passed on to
/// the children when `to_children` (a button's states reach its parts that way).
fn add_state(desc: &mut ElementDesc, id: u32, image: Option<u32>, to_children: bool) {
    if desc.states.iter().any(|s| s.id == id) {
        return;
    }
    desc.states.push(StateDesc {
        id,
        flags: u8::from(to_children),
        rect: rect(-1, -1, -1, -1),
        z_level: 0,
        attributes: Vec::new(),
        media: image
            .map(|file| MediaDesc::Image {
                file,
                draw_mode: 2,
                offset: (0, 0),
            })
            .into_iter()
            .collect(),
        transitions: Vec::new(),
        children: Vec::new(),
    });
}

/// The slot template restyled, when `desc` is it.
fn template(desc: &ElementDesc) -> Option<ElementDesc> {
    if desc.id != tokenize("ShortcutTemplate") {
        return None;
    }
    let edit = |d: &mut ElementDesc| {
        let id = d.id;
        if id == tokenize("ShortcutTemplate") || id == tokenize("ShortcutButton") {
            d.rect = inner(0);
        }
        if id == tokenize("ShortcutButton") {
            set_attribute(d, BUTTON_ROLLOVER, AttributeValue::Bool(true));
            add_state(d, STATE_NORMAL_ROLLOVER, None, true);
        } else if id == tokenize("ShortcutIcon") {
            // Native size inside the rim, one pixel further in while pressed; the pointer passes
            // to the button under it, which takes its rollover as the top region
            // (`UIElement::MouseOverTop`).
            d.rect = inner(ICON_INSET);
            set_attribute(d, MOUSE_INVISIBLE, AttributeValue::Bool(true));
            for state in &mut d.states {
                if state.rect.x0 != -1 {
                    let pressed = state.rect.x0 > 0;
                    state.rect = inner(ICON_INSET + i32::from(pressed));
                }
            }
        } else if id == tokenize("NumberDisplay") {
            d.rect = rect(SLOT - 15, SLOT - 15, SLOT - 5, SLOT - 5);
        } else if id == tokenize("TimeLeftMeter") || id == tokenize("AliasText") {
            d.rect = inner(ICON_INSET);
        }
        // The overlay frame over the icon (normal, pressed and ghosted art), and the hover.
        if has_image(d, 0x4100_0B8B) {
            d.rect = inner(0);
            set_attribute(d, NINE_SLICE, AttributeValue::Int(6));
            set_attribute(d, MOUSE_INVISIBLE, AttributeValue::Bool(true));
            add_state(d, STATE_NORMAL_ROLLOVER, Some(SLOT_HOVER), false);
        }
        // The meter's fill: a dark veil.
        if has_image(d, 0x4100_16EA) {
            map_images(d, |_, _| (VEIL, 3));
            set_attribute(d, ALPHA_VALUE, AttributeValue::Float(VEIL_ALPHA));
            set_attribute(d, SIZE_IMAGE_TO_OWNER, AttributeValue::Bool(true));
        }
        // The active frame (34 pixels), the toggled frame and the pending hourglass (32) at
        // their art's size, centred.
        if has_image(d, 0x4100_5B1B) {
            d.rect = inner(2);
        }
        if has_image(d, 0x4100_7F72) || has_image(d, 0x4100_1710) {
            d.rect = inner(ICON_INSET);
        }
    };
    let mut template = edit_tree(desc, &edit);
    let stateless = template.states.iter_mut().find(|s| s.id == STATELESS)?;
    let number = stateless
        .children
        .iter()
        .find(|c| c.id == tokenize("NumberDisplay"))
        .cloned()?;
    // The well, behind everything (the highest z), and not under the pointer.
    stateless.children.push(Arc::new(ElementDesc {
        id: tokenize("ShortcutSlotWell"),
        kind: desc.kind,
        default_state: STATELESS,
        rect: inner(0),
        z_level: 9,
        edges: [0; 4],
        states: vec![StateDesc {
            id: STATELESS,
            flags: 0,
            rect: rect(-1, -1, -1, -1),
            z_level: 0,
            attributes: vec![attribute(MOUSE_INVISIBLE, AttributeValue::Bool(true))],
            media: vec![MediaDesc::Image {
                file: SLOT_WELL,
                draw_mode: 2,
                offset: (0, 0),
            }],
            transitions: Vec::new(),
            children: Vec::new(),
        }],
    }));
    // The key's digit: the number display's text, left-justified in the top-left corner, in a
    // muted gold.
    let mut key = (*number).clone();
    key.id = tokenize(KEY_LABEL);
    key.rect = rect(4, 3, 15, 13);
    for state in &mut key.states {
        state
            .attributes
            .retain(|a| a.id != tokenize("right_justify"));
    }
    for (channel, value) in [("red", 0.92), ("green", 0.80), ("blue", 0.52)] {
        set_attribute(
            &mut key,
            tokenize(&format!("font_color_{channel}")),
            AttributeValue::Float(value),
        );
    }
    stateless.children.push(Arc::new(key));
    Some(template)
}

/// The key label's element name (the client's own).
pub(crate) const KEY_LABEL: &str = "ShortcutKeyLabel";
