//! The modern style's windows (AGENTS.md: the HUD's two styles; the design's "Other windows",
//! its shared component set): every window of the gameplay and toolbar layouts drawn with one set
//! of parts. Retail's windows share a small set of images: the stone frame and its header
//! (`0x41000970`, `0x410002CD`, `0x410011FC`), the help and close buttons (`0x410027DC`,
//! `0x410002B9`), the tabs' caps (`0x41001E5A`..`0x41001E5F`), four families of blue text
//! buttons, the check box, the expander and drop-down arrows, the rule (`0x41001E71`), the
//! wells' eight pieces, the dialogs' fill (`0x4100225F`, now the glass in the bevel) and bronze border
//! (`0x41002259`..`0x41002262`), the secure trade's frame (`0x410023AD`, redrawn with a header), the whole-window frames
//! of the store, catalog, new sale, ingredients and book (redrawn at their own sizes with their
//! wells where retail's were) and the blue scroll bar; the header bar's image (`0x410002CD`,
//! `0x410011FC`) only filled the room under a side panel's window. Each is swapped in the layout descriptions, before
//! the frameworks make their elements, for the design's part (`modern_art_windows`): teal glass
//! under a dark stone header (the frame's top) in a stone bevel, square header buttons, bronze-lined tabs (gold when
//! chosen), bronze pills, dark wells, a well with a gold tick, gold chevrons, a fine rule and the
//! thin scroll bar. Every element, state and message stays retail's; only the images change.
//! The windows with their own design (the inventory drawer, the loot window, the stack dialog)
//! are left to it. The room under a side panel's window is left empty. A window's title (the text
//! in its header) is drawn in the design's gold.

use std::collections::HashSet;
use std::sync::Arc;

use crate::compose::{nine_edges, DRAW_OVER, NINE_SLICE, NINE_SLICE_EDGES};
use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc};
use crate::modern_art as art;
use crate::text_title::MODERN_TITLE;
use crate::token::{tokenize, REFERENCE, REFERENT, STATELESS};

/// A retail image, the design's part for it and that part's nine-slice border (the attribute
/// and its value).
type Swap = (u32, u32, Option<(u32, i32)>);

/// A window without a header: the bevel all round.
const PANEL: Option<(u32, i32)> = Some((NINE_SLICE, art::WIN_GLASS_BORDER as i32));

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

/// The dialogs' (and the pet and fellow panels') bronze border's eight pieces (left, bottom
/// right, bottom, bottom left, right, top left, top, top right) and the design's.
fn dialog_border() -> [Swap; 8] {
    let b = art::WIN_BORDER;
    [
        (0x4100_2259, b[3], None),
        (0x4100_225A, b[8], None),
        (0x4100_225B, b[7], None),
        (0x4100_225C, b[6], None),
        (0x4100_225D, b[5], None),
        (0x4100_2260, b[0], None),
        (0x4100_2261, b[1], None),
        (0x4100_2262, b[2], None),
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
        // The gold, green and grey families (no ghosted image of their own: the pressed one
        // twice, harmless).
        text_buttons(0x4100_1A8A, 0x4100_1A8B, 0x4100_1A8C, 0x4100_1A8D),
        text_buttons(0x4100_1A8E, 0x4100_1A8F, 0x4100_1A90, 0x4100_1A90),
        text_buttons(0x4100_23D4, 0x4100_23D5, 0x4100_2468, 0x4100_2468),
        text_buttons(0x4100_3AD1, 0x4100_3AD2, 0x4100_3AD3, 0x4100_3AD3),
        text_buttons(0x4100_631E, 0x4100_631F, 0x4100_6320, 0x4100_6321),
    ] {
        all.extend(family);
    }
    let a = art::WIN_ARROW;
    // Drawn to the arrow's whole box (retail's arrows tile otherwise).
    const STRETCH: Option<(u32, i32)> = Some((NINE_SLICE, 0));
    let row = Some((NINE_SLICE, art::WIN_ROW_BORDER));
    all.extend([
        // The page arrows (right, left), the pets' mode arrows (left, right), the spinners' (up,
        // down) and the fold arrows (down, up): gold chevrons.
        (0x4100_25DF, a[2], STRETCH),
        (0x4100_25E0, a[3], STRETCH),
        (0x4100_25E1, a[3], STRETCH),
        (0x4100_25E2, a[2], STRETCH),
        (0x4100_25E3, a[0], STRETCH),
        (0x4100_25E4, a[1], STRETCH),
        (0x4100_25E5, a[1], STRETCH),
        (0x4100_25E6, a[0], STRETCH),
        (0x4100_9F1C, a[0], STRETCH),
        (0x4100_9F1D, a[1], STRETCH),
        (0x4100_9F1E, a[2], STRETCH),
        (0x4100_9F1F, a[3], STRETCH),
        (0x4100_7FDF, a[4], STRETCH),
        (0x4100_7FE0, a[5], STRETCH),
        (0x4100_7FE1, a[4], STRETCH),
        (0x4100_7FE2, a[6], STRETCH),
        (0x4100_7FE3, a[7], STRETCH),
        (0x4100_7FE4, a[6], STRETCH),
        (0x4100_1D4E, a[6], STRETCH),
        (0x4100_1D4F, a[7], STRETCH),
        (0x4100_1D50, a[4], STRETCH),
        (0x4100_1D51, a[5], STRETCH),
        // The store's and consignments' page arrows (left, right: normal, lit, pressed, ghosted).
        (0x4100_877C, a[0], STRETCH),
        (0x4100_877D, a[1], STRETCH),
        (0x4100_877E, a[1], STRETCH),
        (0x4100_877F, a[0], STRETCH),
        (0x4100_8780, a[2], STRETCH),
        (0x4100_8781, a[3], STRETCH),
        (0x4100_8782, a[3], STRETCH),
        (0x4100_8783, a[2], STRETCH),
        // The fellows' meters (health, vigor, focus; the panel's and the mini panel's).
        (0x4100_0978, art::WIN_METER[0], None),
        (0x4100_0977, art::WIN_METER[1], None),
        (0x4100_7FEA, art::WIN_METER[2], None),
        (0x4100_2023, art::WIN_METER[3], None),
        (0x4100_2024, art::WIN_METER[4], None),
        (0x4100_7FE9, art::WIN_METER[5], None),
        // The secure trade's title strip and divider.
        (0x4100_6643, art::WIN_STRIP, None),
        (0x4100_23D7, art::WIN_DIVIDER, None),
        // The small check box (clear, ticked).
        (0x4100_6649, art::WIN_CHECK, None),
        (0x4100_6648, art::WIN_CHECK_ON, None),
        // List rows: plain, chosen.
        (0x4100_2B31, art::WIN_ROW, row),
        (0x4100_2B32, art::WIN_ROW_SEL, row),
        (0x4100_70AB, art::WIN_ROW, row),
        (0x4100_70AC, art::WIN_ROW_SEL, row),
    ]);
    all.extend(dialog_border());
    all.extend([
        (0x4100_225F, art::WIN_PANEL, PANEL),
        (0x4100_23AD, art::WIN_TRADE, None),
        (0x4100_8468, art::WIN_STORE, None),
        (0x4100_87C5, art::WIN_CATALOG, None),
        (0x4100_8C0C, art::WIN_NEW_SALE, None),
        (0x4100_7FDA, art::WIN_INGREDIENTS_TOP, None),
        (0x4100_7FDB, art::WIN_INGREDIENTS_FOOT, None),
        (0x4100_6F0B, art::WIN_BOOK_TOP, None),
        (0x4100_7F70, art::WIN_BOOK_PAGE, None),
        (0x4100_7F71, art::WIN_BOOK_FOOT, None),
        (0x4100_866A, art::WIN_BOOK_DARK, None),
        (0x4100_8C80, art::WIN_PRICE, None),
    ]);
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

/// The frames with the design's header, whose title is drawn in gold.
const HEADED: [u32; 7] = [
    art::WIN_GLASS,
    art::WIN_TRADE,
    art::WIN_STORE,
    art::WIN_CATALOG,
    art::WIN_NEW_SALE,
    art::WIN_INGREDIENTS_TOP,
    art::WIN_BOOK_DARK,
];
/// The design's gold (`#d9c27a`), the titles' colour.
pub(crate) const TITLE_GOLD: [f32; 3] = [217.0 / 255.0, 194.0 / 255.0, 122.0 / 255.0];

/// A frame's title: a text in its header (its top `WIN_HEADER_HEIGHT` rows), or a reference
/// there (retail's titles refer to a shared text; a colour on a reference to anything else is
/// read by nothing).
fn is_title(d: &ElementDesc) -> bool {
    (d.kind == tokenize("text") || d.kind == REFERENCE)
        && d.rect.y0 <= 4
        && i32::from(d.rect.y1) <= i32::from(art::WIN_HEADER_HEIGHT)
        && d.rect.x1 - d.rect.x0 >= 40
}

/// `d` coloured the design's gold (`font_color_red/green/blue` in every state).
fn gilded(d: &ElementDesc) -> ElementDesc {
    let mut d = d.clone();
    for state in &mut d.states {
        for (channel, value) in ["red", "green", "blue"].into_iter().zip(TITLE_GOLD) {
            let id = tokenize(&format!("font_color_{channel}"));
            state.attributes.retain(|a| a.id != id);
            state.attributes.push(AttributeDesc {
                id,
                value: AttributeValue::Float(value),
            });
        }
        // Small caps, spaced and fitted to the box (`text_title.rs`).
        state.attributes.retain(|a| a.id != MODERN_TITLE);
        state.attributes.push(AttributeDesc {
            id: MODERN_TITLE,
            value: AttributeValue::Bool(true),
        });
    }
    d
}

/// `layout` with the windows' images swapped for the design's (every element id and state kept).
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    let swaps = swaps();
    fn walk(d: &ElementDesc, swaps: &[Swap], headed_ids: &HashSet<u32>) -> ElementDesc {
        let mut d = d.clone();
        if own_design(d.id) {
            return d;
        }
        // The part's border goes in every state (the element reads it from its base state).
        let mut border = None;
        // A reference takes its referent's frame (or gives its own to its referent's title).
        let mut headed =
            headed_ids.contains(&d.id) || referent(&d).is_some_and(|r| headed_ids.contains(&r));
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
                        headed |= HEADED.contains(&to);
                    }
                }
            }
            for child in &mut state.children {
                *child = Arc::new(walk(child, swaps, headed_ids));
            }
        }
        if headed {
            for state in &mut d.states {
                for child in &mut state.children {
                    if is_title(child) {
                        *child = Arc::new(gilded(child));
                    }
                }
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
    // The elements whose frame has the design's header, and the referents of references that
    // draw one: their titles are gilded.
    let mut headed_ids = HashSet::new();
    fn collect(d: &ElementDesc, swaps: &[Swap], out: &mut HashSet<u32>) {
        let headed = d.states.iter().any(|state| {
            state.media.iter().any(|m| {
                matches!(m, MediaDesc::Image { file, .. }
                    if swaps.iter().any(|(from, to, _)| from == file && HEADED.contains(to)))
            })
        });
        if headed {
            out.insert(d.id);
            out.extend(referent(d));
        }
        for state in &d.states {
            for child in &state.children {
                collect(child, swaps, out);
            }
        }
    }
    for element in &layout.elements {
        collect(element, &swaps, &mut headed_ids);
    }
    let mut layout = layout.clone();
    for element in &mut layout.elements {
        *element = Arc::new(walk(element, &swaps, &headed_ids));
    }
    layout
}

/// The element a reference stands for (its `referent`).
fn referent(d: &ElementDesc) -> Option<u32> {
    if d.kind != REFERENCE {
        return None;
    }
    d.states.iter().find(|s| s.id == STATELESS).and_then(|s| {
        s.attributes.iter().find_map(|a| match (a.id, &a.value) {
            (REFERENT, AttributeValue::Id(id)) => Some(*id),
            _ => None,
        })
    })
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
            let (mut gold, mut marked) = (0, 0);
            fn count(d: &ElementDesc, gold: &mut usize, marked: &mut usize) {
                for state in &d.states {
                    *gold += state
                        .attributes
                        .iter()
                        .filter(|a| a.id == tokenize("font_color_red"))
                        .filter(|a| a.value == AttributeValue::Float(TITLE_GOLD[0]))
                        .count();
                    // Every gilded title is also marked for small caps.
                    *marked += state
                        .attributes
                        .iter()
                        .filter(|a| a.id == MODERN_TITLE && a.value == AttributeValue::Bool(true))
                        .count();
                    for child in &state.children {
                        count(child, gold, marked);
                    }
                }
            }
            for element in &modern.elements {
                count(element, &mut gold, &mut marked);
            }
            assert!(gold > 0, "a title in gold");
            assert_eq!(gold, marked, "each gold title in small caps");
        }
    }
}
