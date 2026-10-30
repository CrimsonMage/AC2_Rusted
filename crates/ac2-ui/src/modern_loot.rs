//! The modern style's loot window (AGENTS.md: the HUD's two styles): `gmExternalContainerUI`'s
//! `ExternalContainerField` as the design's (its variant A, the refined classic). Retail drew the
//! panel at its frame's size (`0x41000D41`, 138 by 165) with the container's list three slots
//! wide, a small close button, and the Loot All and Transmute All buttons as their art alone.
//! Here the same frame is drawn in nine pieces at 214 by 262 (its corners kept) over teal glass,
//! the title centred under its top, a bronze close button at its corner, the list four slots wide
//! with the thin scroll bar, the two buttons as pills holding their art and their names, and a note
//! under them. The elements are retail's, so the panel's opening, slots, drops, Loot All,
//! Transmute All and Close stay retail's.

use std::sync::Arc;

use crate::compose::{nine_edges, NINE_SLICE, NINE_SLICE_EDGES};
use crate::element::{UiBox, UiTree};
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_inventory::{keep, make_part, remap_art};
use crate::token::{tokenize, SIZE_IMAGE_TO_OWNER};

/// The window's size and the frame's borders (left, top, right, bottom).
const SIZE: (i32, i32) = (214, 262);
const FRAME_EDGES: (u8, u8, u8, u8) = (10, 40, 10, 14);
/// The grid's place, its columns and slot size; the buttons' row and the note's.
const GRID: (i32, i32) = (35, 46);
const COLUMNS: i32 = 4;
const SLOT: i32 = 36;
const BUTTONS_Y: i32 = 198;
const NOTE_Y: i32 = 230;
const PILL_HEIGHT: i32 = 24;
/// The pills' widths (the design's, Loot All then Transmute).
const PILL_WIDTHS: [i32; 2] = [86, 104];
/// The buttons' names and the art's size at the pill's left.
const BUTTON_NAMES: [&str; 2] = ["Loot All", "Transmute"];
const ICON: (i32, i32) = (20, 10);
/// The buttons' art and the design's (`modern_art`).
const BUTTON_ART: [(u32, u32); 6] = [
    (0x4100_6F50, art::LOOT_PILL),
    (0x4100_6F51, art::LOOT_PILL_LIT),
    (0x4100_6F52, art::LOOT_PILL_LIT),
    (0x4100_6F53, art::LOOT_PILL),
    (0x4100_6F54, art::LOOT_PILL_LIT),
    (0x4100_6F55, art::LOOT_PILL_LIT),
];
const CLOSE_ART: [(u32, u32); 2] = [
    (0x4100_02B9, art::LOOT_CLOSE),
    (0x4100_02BA, art::LOOT_CLOSE_LIT),
];
/// The panel's unnamed parts: its drag bars (top, bottom) and its list.
const TOP_BAR: u32 = 0xA9CB_0E76;
const BOTTOM_BAR: u32 = 0xC9CB_0E76;
const LIST: u32 = 0x397F_2C39;

fn place(tree: &mut UiTree, element: Option<usize>, (x, y, w, h): (i32, i32, i32, i32)) {
    let Some(element) = element else {
        return;
    };
    keep(tree, element);
    let at = UiBox::new(x, y, x + w - 1, y + h - 1);
    if tree.el(element).rect != at {
        tree.resize_to(element, w, h);
        tree.move_to(element, x, y);
    }
}

/// The panel laid out as the design's (its parts made once, kept where they are put): `root`
/// its `ExternalContainerField`.
pub(crate) fn adopt(tree: &mut UiTree, root: usize, fonts: &[&Arc<LayoutDesc>]) {
    let find = |tree: &UiTree, id: u32| tree.child_recursive(root, id);
    let (w, h) = SIZE;
    tree.resize_to(root, w, h);
    let (l, t, r, b) = FRAME_EDGES;
    tree.set_int(root, NINE_SLICE_EDGES, nine_edges(l, t, r, b));
    let template = find(tree, TOP_BAR);
    let glass = make_part(
        tree,
        root,
        template,
        "ModernLootGlass",
        art::LOOT_GLASS,
        Some(art::LOOT_GLASS_BORDER),
        60,
    );
    if let Some(g) = glass {
        keep(tree, g);
    }
    place(tree, glass, (5, 34, w - 10, h - 39));
    place(tree, find(tree, TOP_BAR), (0, 0, w, 34));
    place(tree, find(tree, BOTTOM_BAR), (0, h - 6, w, 6));
    place(
        tree,
        find(tree, tokenize("ExternalContainerLabel")),
        (34, 7, w - 68, 26),
    );
    if let Some(text) =
        find(tree, tokenize("ExternalContainerLabel")).and_then(|t| tree.el_mut(t).text.as_mut())
    {
        text.justification = crate::text::Justification::Center;
    }
    let close = find(tree, tokenize("ExternalContainerCloseButton"));
    if let Some(c) = close {
        remap_art(tree, c, &CLOSE_ART, false);
    }
    place(tree, close, (w - 12 - 19, 7, 19, 19));
    // The grid four slots wide, the thin bar at its right.
    let container = find(tree, tokenize("ExternalContainer"));
    let grid_w = COLUMNS * SLOT;
    let grid_h = 4 * SLOT;
    let bar = art::SCROLL_WIDTH;
    place(tree, container, (GRID.0, GRID.1, grid_w + 2 + bar, grid_h));
    if let Some(container) = container {
        for part in tree.children(container).to_vec() {
            if tree.el(part).id() == LIST {
                place(tree, Some(part), (0, 0, grid_w, grid_h));
                tree.set_int(part, tokenize("max_columns"), COLUMNS);
                continue;
            }
            remap_art(tree, part, &crate::modern_inventory::SCROLL_ART, true);
            let r = tree.el(part).rect;
            let at = if r.height() <= 16 && r.y0 <= 1 {
                (grid_w + 2, 0, bar, bar)
            } else if r.height() <= 16 {
                (grid_w + 2, grid_h - bar, bar, bar)
            } else {
                (grid_w + 2, bar, bar, grid_h - 2 * bar)
            };
            place(tree, Some(part), at);
            let mut under = tree.children(part).to_vec();
            while let Some(p) = under.pop() {
                under.extend(tree.children(p).iter().copied());
                let pr = tree.el(p).rect;
                if pr.width() != bar {
                    tree.resize_to(p, bar, pr.height());
                }
            }
        }
    }
    // The buttons as pills: their art at the left, their names after it.
    let buttons = [
        (
            find(tree, tokenize("LootAllButton")),
            0x4100_6F50,
            "Loot All",
        ),
        (
            find(tree, tokenize("TransmuteAllButton")),
            0x4100_6F53,
            "Transmute",
        ),
    ];
    let mut icons = [None, None];
    let mut names = [None, None];
    let mut widths = [0; 2];
    for (i, &(button, image, name)) in buttons.iter().enumerate() {
        let Some(button) = button else {
            continue;
        };
        remap_art(tree, button, &BUTTON_ART, false);
        tree.set_int(button, NINE_SLICE, art::LOOT_PILL_BORDER);
        let icon = make_part(
            tree,
            button,
            template,
            &format!("ModernLootIcon{i}"),
            image,
            None,
            1,
        );
        if let Some(icon) = icon {
            keep(tree, icon);
            tree.set_bool(icon, SIZE_IMAGE_TO_OWNER, true);
        }
        icons[i] = icon;
        names[i] = crate::modern_drawer_bar::count_label(
            tree,
            fonts,
            button,
            &format!("ModernLootName{i}"),
            crate::text::Justification::Left,
        );
        if let Some(l) = names[i] {
            keep(tree, l);
            for (channel, value) in [("red", 0xEC), ("green", 0xE3), ("blue", 0xC4)] {
                let _ = tree.set_float(
                    l,
                    tokenize(&format!("font_color1_{channel}")),
                    value as f32 / 255.0,
                );
            }
            tree.set_text_with_font(l, name, 0, 1);
        }
        // As wide as the art, the name and their padding (the design's widths at the least).
        let text = names[i]
            .and_then(|l| tree.text_line_width(l, BUTTON_NAMES[i]))
            .unwrap_or(60);
        widths[i] = (ICON.0 + 10 + text + 8).max(PILL_WIDTHS[i]);
    }
    let total = widths[0] + 6 + widths[1];
    let mut x = ((w - total) / 2).max(2);
    for (i, &(button, _, _)) in buttons.iter().enumerate() {
        place(tree, button, (x, BUTTONS_Y, widths[i], PILL_HEIGHT));
        place(
            tree,
            icons[i],
            (5, (PILL_HEIGHT - ICON.1) / 2, ICON.0, ICON.1),
        );
        crate::modern_drawer_bar::place_label(
            tree,
            names[i],
            UiBox::new(ICON.0 + 10, 5, widths[i] - 2, 18),
        );
        x += widths[i] + 6;
    }
    // The note in the small font, the design's grey green (`#8fa79a`).
    let note = crate::modern_drawer_bar::count_label(
        tree,
        fonts,
        root,
        "ModernLootNote",
        crate::text::Justification::Center,
    );
    if let Some(n) = note {
        keep(tree, n);
        for (channel, value) in [("red", 0x8F), ("green", 0xA7), ("blue", 0x9A)] {
            let _ = tree.set_float(
                n,
                tokenize(&format!("font_color1_{channel}")),
                value as f32 / 255.0,
            );
        }
        tree.set_text_with_font(n, "Transmute: all items to gold", 0, 1);
    }
    crate::modern_drawer_bar::place_label(tree, note, UiBox::new(14, NOTE_Y, w - 15, NOTE_Y + 13));
}
