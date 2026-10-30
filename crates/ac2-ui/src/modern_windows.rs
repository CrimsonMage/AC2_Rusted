//! The modern style's windows (AGENTS.md: the HUD's two styles; the design's "Other windows",
//! its shared component set): every window of the gameplay and toolbar layouts drawn with one set
//! of parts. Retail's windows share a small set of images: the stone frame and its header
//! (`0x41000970`, `0x410002CD`, `0x410011FC`), the help and close buttons (`0x410027DC`,
//! `0x410002B9`), the tabs' caps (`0x41001E5A`..`0x41001E5F`), four families of blue text
//! buttons, the check box, the expander and drop-down arrows, the rule (`0x41001E71`), the
//! wells' eight pieces and the blue scroll bar; the header bar's image (`0x410002CD`,
//! `0x410011FC`) only filled the room under a side panel's window. Each is swapped in the layout descriptions, before
//! the frameworks make their elements, for the design's part (`modern_art_windows`): teal glass
//! under a dark stone header (the frame's top) in a stone bevel, square header buttons, bronze-lined tabs (gold when
//! chosen), bronze pills, dark wells, a well with a gold tick, gold chevrons, a fine rule and the
//! thin scroll bar. Every element, state and message stays retail's; only the images change.
//! The windows with their own design (the inventory drawer, the loot window, the stack dialog)
//! are left to it. The room under a side panel's window is left empty.

use std::sync::Arc;

use crate::compose::{nine_edges, DRAW_OVER, NINE_SLICE, NINE_SLICE_EDGES};
use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc};
use crate::modern_art as art;
use crate::token::tokenize;

/// A retail image, the design's part for it and that part's nine-slice border (the attribute
/// and its value).
type Swap = (u32, u32, Option<(u32, i32)>);

/// The window's borders: its header at the top, `WIN_GLASS_BORDER` on the other sides.
const FRAME: Option<(u32, i32)> = Some((
    NINE_SLICE_EDGES,
    nine_edges(
        art::WIN_GLASS_BORDER,
        art::WIN_HEADER_HEIGHT,
        art::WIN_GLASS_BORDER,
        art::WIN_GLASS_BORDER,
    ),
));

const fn text_buttons(normal: u32, lit: u32, pressed: u32, ghosted: u32) -> [Swap; 4] {
    let b = Some((NINE_SLICE, art::WIN_BUTTON_BORDER));
    [
        (normal, art::WIN_BUTTON[0], b),
        (lit, art::WIN_BUTTON[1], b),
        (pressed, art::WIN_BUTTON[2], b),
        (ghosted, art::WIN_BUTTON[3], b),
    ]
}

/// A well's eight retail pieces (top left, top, top right, right, bottom right, bottom, bottom
/// left, left) and the design's.
const fn well(first: u32) -> [Swap; 8] {
    let w = art::WIN_WELL;
    [
        (first, w[0], None),
        (first + 1, w[1], None),
        (first + 2, w[2], None),
        (first + 3, w[5], None),
        (first + 4, w[8], None),
        (first + 5, w[7], None),
        (first + 6, w[6], None),
        (first + 7, w[3], None),
    ]
}

/// The frame, its header, its buttons, the tabs, the check box, the arrows and the rule.
const PARTS: [Swap; 21] = [
    (0x4100_0970, art::WIN_GLASS, FRAME),
    (0x4100_11FB, art::WIN_GLASS, FRAME),
    (0x4100_02B9, art::WIN_CLOSE, None),
    (0x4100_02BA, art::WIN_CLOSE_LIT, None),
    (0x4100_27DC, art::WIN_HELP, None),
    (0x4100_27DD, art::WIN_HELP_LIT, None),
    // The tabs' caps: the chosen tab's (open at its foot) and the others'.
    (0x4100_1E5A, art::WIN_TAB[3], None),
    (0x4100_1E5C, art::WIN_TAB[4], None),
    (0x4100_1E5E, art::WIN_TAB[5], None),
    (0x4100_1E5B, art::WIN_TAB[0], None),
    (0x4100_1E5D, art::WIN_TAB[1], None),
    (0x4100_1E5F, art::WIN_TAB[2], None),
    (0x4100_2528, art::WIN_CHECK, None),
    (0x4100_2529, art::WIN_CHECK_ON, None),
    (0x4100_1E56, art::WIN_CHEVRON_RIGHT, None),
    (0x4100_1E57, art::WIN_CHEVRON_RIGHT_LIT, None),
    (0x4100_1E58, art::WIN_CHEVRON_DOWN, None),
    (0x4100_1E59, art::WIN_CHEVRON_DOWN_LIT, None),
    (0x4100_664A, art::WIN_CHEVRON_DOWN, None),
    (0x4100_65A4, art::WIN_CHEVRON_DOWN, None),
    (
        0x4100_1E71,
        art::WIN_RULE,
        Some((NINE_SLICE, art::WIN_RULE_BORDER)),
    ),
];

/// Every swap: the parts, the four text button families (105 by 20, 80 by 30, 110 by 25, 212
/// by 30), the two wells and the scroll bar.
fn swaps() -> Vec<Swap> {
    let mut all = PARTS.to_vec();
    for family in [
        text_buttons(0x4100_23D2, 0x4100_23D3, 0x4100_2467, 0x4100_23D6),
        text_buttons(0x4100_1A86, 0x4100_1A87, 0x4100_1A88, 0x4100_1A89),
        text_buttons(0x4100_1EBB, 0x4100_1EBC, 0x4100_1EBD, 0x4100_1EBE),
        text_buttons(0x4100_25E7, 0x4100_25E8, 0x4100_25E9, 0x4100_25EA),
    ] {
        all.extend(family);
    }
    all.extend(well(0x4100_2251));
    all.extend(well(0x4100_2B33));
    // The scroll bar (not its `0x410002BA`, the close button's lit image above).
    all.extend(
        crate::modern_inventory::SCROLL_ART
            .iter()
            .filter(|(from, _)| *from != 0x4100_02BA)
            .map(|&(from, to)| (from, to, None)),
    );
    all
}

/// The header bar images, which retail tiled down the filler under a side panel's window (the
/// window's own header is its frame's top).
const FILLERS: [u32; 2] = [0x4100_02CD, 0x4100_11FC];

/// The windows the design draws on their own (`modern_inventory`, `modern_loot`,
/// `modern_stack`), left as they are.
fn own_design(id: u32) -> bool {
    ["InventoryField", "ExternalContainerField", "StackableField"]
        .iter()
        .any(|name| tokenize(name) == id)
}

/// `layout` with the windows' images swapped for the design's (every element id and state kept).
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    let swaps = swaps();
    fn walk(d: &ElementDesc, swaps: &[Swap]) -> ElementDesc {
        let mut d = d.clone();
        if own_design(d.id) {
            return d;
        }
        // The part's border goes in every state (the element reads it from its base state).
        let mut border = None;
        for state in &mut d.states {
            state
                .media
                .retain(|m| !matches!(m, MediaDesc::Image { file, .. } if FILLERS.contains(file)));
            for media in &mut state.media {
                if let MediaDesc::Image {
                    file, draw_mode, ..
                } = media
                {
                    if let Some(&(_, to, nine)) = swaps.iter().find(|(from, ..)| from == file) {
                        *file = to;
                        *draw_mode = DRAW_OVER;
                        border = border.or(nine);
                    }
                }
            }
            for child in &mut state.children {
                *child = Arc::new(walk(child, swaps));
            }
        }
        if let Some((id, value)) = border {
            for state in &mut d.states {
                state.attributes.retain(|a| a.id != id);
                state.attributes.push(AttributeDesc {
                    id,
                    value: AttributeValue::Int(value),
                });
            }
        }
        d
    }
    let mut layout = layout.clone();
    for element in &mut layout.elements {
        *element = Arc::new(walk(element, &swaps));
    }
    layout
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(d: &ElementDesc, out: &mut Vec<(u32, usize)>) {
        out.push((d.id, d.states.len()));
        for state in &d.states {
            for child in &state.children {
                ids(child, out);
            }
        }
    }

    fn images(d: &ElementDesc, inside_own: bool, out: &mut Vec<(u32, bool)>) {
        let own = inside_own || own_design(d.id);
        for state in &d.states {
            for media in &state.media {
                if let MediaDesc::Image { file, .. } = media {
                    out.push((*file, own));
                }
            }
            for child in &state.children {
                images(child, own, out);
            }
        }
    }

    /// The toolbar and gameplay layouts restyled keep every element and state, show none of the
    /// swapped retail images outside the windows with their own design, and drop the fillers.
    #[test]
    fn the_restyle_swaps_only_images() {
        let Some(dataset) = ac2_dat::test_support::env_dataset() else {
            return;
        };
        let store = ac2_db::ResourceStore::new(dataset);
        let swapped: Vec<u32> = swaps().iter().map(|s| s.0).collect();
        for layout_enum in [crate::hud::LAYOUT_TOOLBAR, crate::hud::LAYOUT_GAMEPLAY] {
            let layout = crate::hud::load_layout_by_enum(&store, layout_enum)
                .unwrap()
                .unwrap();
            let modern = restyle(&layout);
            let (mut before, mut after) = (Vec::new(), Vec::new());
            for (a, b) in layout.elements.iter().zip(&modern.elements) {
                ids(a, &mut before);
                ids(b, &mut after);
            }
            assert_eq!(before, after);
            let mut shown = Vec::new();
            for element in &modern.elements {
                images(element, false, &mut shown);
            }
            for (file, own) in shown {
                assert!(own || !swapped.contains(&file), "{file:08X}");
                assert!(own || !FILLERS.contains(&file), "{file:08X}");
            }
            assert!(swapped.contains(&0x4100_0970));
        }
    }
}
