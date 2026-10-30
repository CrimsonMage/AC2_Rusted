//! The modern HUD's shortcut bar (AGENTS.md: the HUD's two styles; the design "variant D"): rows
//! of 32-pixel glass wells 3 pixels apart, an empty slot only a faint line until the pointer is
//! over the strip or something is dragged. Retail's bar (`ShortcutField`) drew one grid image
//! (`0x41008D29`) behind 32-pixel slots that sat edge to edge. The toolbar layout's two
//! descriptions are rewritten before the bar is made ([`restyle`]), keeping every element, state,
//! message and attribute `gmShortcutUI` reads: the bar loses its grid and is ten slots of 35
//! pixels wide and one row of 35 tall under a 4-pixel band where the top edge's drag reaches the
//! resize grip (`row_height` 35, `border_size` 4, so `gmShortcutUI::RefreshShortcutHeight` makes
//! it whole rows); the slot (`ShortcutTemplate`) is 35
//! pixels square and draws its well (`modern_art`) in its top-left 32, its parts unchanged there.
//! The strip itself (its fade, the attack buttons and the set chip beside the rows) is the
//! toolbar's (`modern_toolbar.rs`).

use std::sync::Arc;

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc, Rect};
use crate::modern_art::{SLOT, SLOT_PITCH, SLOT_WELL};
use crate::token::{tokenize, STATELESS};

const SLOTS_PER_ROW: i32 = 10;
/// The band over the rows where a press reaches the resize grip (retail's bar kept 12 pixels
/// over its slots for it; `border_size`, so the rows still snap whole).
const GRIP_BAND: i32 = 4;
/// The list's holder in `ShortcutField` (unnamed).
const LIST_HOLDER: u32 = 0xEA2A_0B6D;

fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
    Rect {
        x0: x0 as i16,
        y0: y0 as i16,
        x1: x1 as i16,
        y1: y1 as i16,
    }
}

/// `desc`'s attribute `name` set to `value` in its stateless state.
fn set_attribute(desc: &mut ElementDesc, name: &str, value: AttributeValue) {
    let id = tokenize(name);
    if let Some(state) = desc.states.iter_mut().find(|s| s.id == STATELESS) {
        state.attributes.retain(|a| a.id != id);
        state.attributes.push(AttributeDesc { id, value });
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
        } else if element.id == tokenize("ShortcutTemplate") {
            *element = Arc::new(template(element));
        }
    }
    layout
}

/// The bar: ten slots wide and one row tall, without its grid, kept where the toolbar puts it.
fn bar(desc: &ElementDesc) -> ElementDesc {
    let width = SLOT_PITCH * SLOTS_PER_ROW;
    let edit = |d: &mut ElementDesc| {
        if d.id == tokenize("ShortcutField") {
            let y1 = i32::from(d.rect.y1);
            d.rect = rect(
                512 - width / 2,
                y1 + 1 - SLOT_PITCH - GRIP_BAND,
                512 - width / 2 + width - 1,
                y1,
            );
            d.edges = [0; 4];
            for state in &mut d.states {
                state
                    .media
                    .retain(|m| !matches!(m, MediaDesc::Image { .. }));
            }
            set_attribute(d, "border_size", AttributeValue::Int(GRIP_BAND));
            set_attribute(d, "row_height", AttributeValue::Int(SLOT_PITCH));
            set_attribute(d, "min_height", AttributeValue::Int(GRIP_BAND + SLOT_PITCH));
            set_attribute(
                d,
                "max_height",
                AttributeValue::Int(GRIP_BAND + 5 * SLOT_PITCH),
            );
        } else if d.edges == [1, 1, 1, 2] {
            // The resize grip (`UIElement_Resizebar`, behind the list): the bar's first row and
            // the band over it, where only the grip takes a press.
            d.rect = rect(0, 0, width - 1, GRIP_BAND + SLOT_PITCH - 1);
        } else if d.id == LIST_HOLDER {
            // The list's holder: the slots' area, under the band.
            d.rect = rect(0, GRIP_BAND, width - 1, GRIP_BAND + SLOT_PITCH - 1);
        } else {
            // The list: its holder's area.
            d.rect = rect(0, 0, width - 1, SLOT_PITCH - 1);
        }
    };
    edit_tree(desc, &edit)
}

/// The slot: 35 pixels square with its well; its parts stay the 32 pixels in its top-left.
fn template(desc: &ElementDesc) -> ElementDesc {
    let mut desc = desc.clone();
    desc.rect = rect(0, 0, SLOT_PITCH - 1, SLOT_PITCH - 1);
    for state in &mut desc.states {
        if state.id == STATELESS {
            state
                .media
                .retain(|m| !matches!(m, MediaDesc::Image { .. }));
            state.media.push(MediaDesc::Image {
                file: SLOT_WELL,
                draw_mode: crate::compose::DRAW_OVER,
                offset: (0, 0),
            });
        }
        for child in &mut state.children {
            // Kept 32 pixels square at the slot's corner, where it no longer follows the slot.
            if child.edges == [1, 1, 1, 1] {
                let mut c = (**child).clone();
                c.edges = [0; 4];
                c.rect = rect(0, 0, SLOT - 1, SLOT - 1);
                *child = Arc::new(c);
            }
        }
    }
    desc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slot_is_its_pitch_square_with_its_well_and_its_parts_in_the_corner() {
        let state = |children| crate::layout::StateDesc {
            id: STATELESS,
            flags: 0,
            rect: rect(-1, -1, -1, -1),
            z_level: 0,
            attributes: Vec::new(),
            media: Vec::new(),
            transitions: Vec::new(),
            children,
        };
        let element = |name: &str, edges, states| ElementDesc {
            id: tokenize(name),
            kind: 0,
            default_state: STATELESS,
            rect: rect(0, 0, 31, 31),
            z_level: 0,
            edges,
            states,
        };
        let child = element("ShortcutButton", [1, 1, 1, 1], vec![state(Vec::new())]);
        let desc = element(
            "ShortcutTemplate",
            [0; 4],
            vec![state(vec![Arc::new(child)])],
        );
        let t = template(&desc);
        assert_eq!(t.rect, rect(0, 0, SLOT_PITCH - 1, SLOT_PITCH - 1));
        let state = &t.states[0];
        assert!(state
            .media
            .iter()
            .any(|m| matches!(m, MediaDesc::Image { file, .. } if *file == SLOT_WELL)));
        assert_eq!(state.children[0].rect, rect(0, 0, SLOT - 1, SLOT - 1));
        assert_eq!(state.children[0].edges, [0; 4]);
    }
}
