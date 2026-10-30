//! The modern dialogs' header (AGENTS.md: the HUD's two styles; the design's dialog): a dark stone
//! band across the top of each of `DialogFactory`'s dialogs (`ConfirmationField`,
//! `MessageField`, `WaitField`, `ResurrectField`) with its title centred in gold. Retail's dialogs
//! had no title. The band and title are the client's own parts over the dialog's root; the text
//! and buttons are retail's elements, the text moved down clear of the band, so the dialog's
//! messages and answers stay retail's.

use std::sync::Arc;

use crate::dialog::{
    DIALOG_CONFIRMATION, DIALOG_MESSAGE, DIALOG_RESURRECT, DIALOG_TIMED, DIALOG_WAIT,
};
use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_inventory::{keep, make_part};
use crate::token::MOUSE_INVISIBLE;
use crate::window_parts::{set_button_label, text_button};

/// The band's inset from the dialog's edge (inside its bevel) and its height.
const INSET: i32 = 2;
const HEIGHT: i32 = 26;
/// The top the body's text keeps clear under the band.
const BODY_TOP: i32 = 31;

/// A dialog's title.
fn title(kind: u32) -> Option<&'static str> {
    Some(match kind {
        DIALOG_CONFIRMATION | DIALOG_TIMED => "CONFIRM",
        DIALOG_MESSAGE => "MESSAGE",
        DIALOG_WAIT => "PLEASE WAIT",
        DIALOG_RESURRECT => "RESURRECTION",
        _ => return None,
    })
}

/// The header band and gold title put on a dialog's `root` (its `text` element moved down clear
/// of the band when it sits under it), the title's text button from `gameplay`.
pub(crate) fn dress(
    tree: &mut UiTree,
    gameplay: &Arc<LayoutDesc>,
    root: usize,
    text: Option<usize>,
    kind: u32,
) {
    let Some(name) = title(kind) else {
        return;
    };
    let width = tree.el(root).rect.width();
    let mut all = vec![root];
    let mut i = 0;
    while i < all.len() {
        let kids = tree.children(all[i]).to_vec();
        all.extend(kids);
        i += 1;
    }
    let template = all
        .into_iter()
        .find(|&e| tree.el(e).reference.kind == tree.el(e).desc.kind);
    if let Some(head) = make_part(
        tree,
        root,
        template,
        "ModernDialogHead",
        art::DIALOG_HEAD,
        Some(art::DIALOG_HEAD_BORDER),
        -2,
    ) {
        keep(tree, head);
        tree.move_to(head, INSET, INSET);
        tree.resize_to(head, width - 2 * INSET, HEIGHT);
    }
    if let Some(label) = text_button(
        tree,
        gameplay,
        root,
        "ModernDialogTitle",
        "",
        (INSET, INSET, width - 2 * INSET),
    ) {
        keep(tree, label);
        tree.clear_image(label);
        tree.set_bool(label, MOUSE_INVISIBLE, true);
        tree.set_z_level(label, -3);
        for child in tree.children(label).to_vec() {
            if let Some(t) = tree.el_mut(child).text.as_mut() {
                t.justification = crate::text::Justification::Center;
            }
        }
        // The button's font is taller than its 19-pixel label: the label takes the band's height.
        tree.resize_to(label, width - 2 * INSET, HEIGHT);
        set_button_label(tree, label, name);
        for child in tree.children(label).to_vec() {
            tree.resize_to(child, width - 2 * INSET, HEIGHT);
            tree.set_bool(child, MOUSE_INVISIBLE, true);
            if tree.el(child).text.is_some() {
                // Gold small caps as the windows' titles (`text_title.rs`).
                crate::modern_windows::mark_title(tree, child);
            }
        }
    }
    // The body's text: the highest of its ancestors that is smaller than the root (the frame
    // field that holds everything is not moved), moved down if it starts under the band.
    if let Some(mut e) = text {
        while let Some(parent) = tree.el(e).parent_index() {
            if parent == root || tree.el(parent).rect.width() >= width {
                break;
            }
            e = parent;
        }
        let (x, top) = (tree.el(e).rect.x0, tree.el(e).rect.y0);
        if (1..BODY_TOP).contains(&top) {
            tree.move_to(e, x, BODY_TOP);
        }
    }
}
