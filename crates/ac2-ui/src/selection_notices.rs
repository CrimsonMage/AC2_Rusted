//! `gmSelectionUI`'s notice handlers and the selection keys beyond the cycling ones:
//! `RecvNotice_E_LastAttacker`, `RecvNotice_UpdateSelectability`, `RecvNotice_BeingDeleted`,
//! `RecvNotice_SetTargetingPointer`, and `Select`'s reselect (`0x41000018`), last selection
//! (`0x41000028`), last attacker (`0x4100002A`) and clear (`0x4000001E`) keys.
//!
//! The decisions here are pure: each returns the [`SelectionAction`]s retail makes, in order.
//! The client carries them out (the selection commands, the auto-target key through `Select`).
//! The pointer is the tree's: `SetTargetingPointer` swaps the `ItemSelectionField` root's cursor
//! for its `usage_cursor` and restores the saved one.
//!
//! `CWPhysicsObject::LookAtSelection` follows each `Select` action in the client (`CLookAt`).
//! `StopAttacking` is returned as an action; the client runs `gmCWInterface::StopAttacking`.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::tokenize;

/// `gmSelectionUI::Select(0x41000024)`: the auto-target, the next foe.
pub const AUTO_TARGET_KEY: u32 = 0x4100_0024;
/// `gmSelectionUI::Select(0x41000018)`: the player itself, or back to the last selection.
pub const RESELECT_KEY: u32 = 0x4100_0018;
/// `gmSelectionUI::Select(0x41000028)`: the last selection.
pub const LAST_SELECTION_KEY: u32 = 0x4100_0028;
/// `gmSelectionUI::Select(0x4100002A)`: the last attacker.
pub const LAST_ATTACKER_KEY: u32 = 0x4100_002A;
/// `gmSelectionUI::Select(0x4000001E)`: clear the selection, unless the input takes typing.
pub const CLEAR_KEY: u32 = 0x4000_001E;

/// The layout element `gmSelectionUI` makes its `_itemSelectionField`.
const ITEM_SELECTION_FIELD: &str = "ItemSelectionField";

/// What a selection notice asks for, in retail's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionAction {
    /// `gmCWInterface::SetSelection` (`None`: the invalid object, the selection clears).
    Select(Option<u64>),
    /// `gmCWInterface::StopAttacking` on this object.
    StopAttacking(u64),
    /// `gmSelectionUI::Select` with this key (a cycling key, or the auto-target).
    Key(u32),
    /// The second check of `RecvNotice_BeingDeleted` and `RecvNotice_UpdateSelectability`'s
    /// first: the selection clears when it is still this object (after a key that may have
    /// moved it).
    ClearIfSelected(u64),
}

/// The player's selection, as the notices read it (`gmCWInterface::GetSelection` and
/// `GetLastSelection`, `gmSelectionUI::_lastAttackerID`). Both selection values are `None`
/// when nothing is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionSnapshot {
    pub player: u64,
    pub selection: Option<u64>,
    pub last_selection: Option<u64>,
    pub last_attacker: Option<u64>,
}

/// `RecvNotice_E_LastAttacker`'s tail, once the attacker is stored: with the auto-target option
/// on and nothing selected, the attacker is selected. `auto_target` is
/// `gmCWInterface::GetAutoTarget`; `has_selection` is `HasSomethingSelected`.
pub fn last_attacker_notice(
    attacker: u64,
    auto_target: bool,
    has_selection: bool,
) -> Vec<SelectionAction> {
    if auto_target && !has_selection {
        vec![SelectionAction::Select(Some(attacker))]
    } else {
        Vec::new()
    }
}

/// `RecvNotice_UpdateSelectability(obj, isSelectable)`: an object that is the selection (not the
/// player) and no longer selectable clears the selection, stops attacking it, and with the
/// auto-target option on picks the next foe.
pub fn selectability_notice(
    object: u64,
    selectable: bool,
    snapshot: &SelectionSnapshot,
    auto_target: bool,
) -> Vec<SelectionAction> {
    if selectable || snapshot.selection != Some(object) || object == snapshot.player {
        return Vec::new();
    }
    let mut out = vec![
        SelectionAction::Select(None),
        SelectionAction::StopAttacking(object),
    ];
    if auto_target {
        out.push(SelectionAction::Key(AUTO_TARGET_KEY));
    }
    out
}

/// `RecvNotice_BeingDeleted(obj)`: a deleted object that is the selection stops being attacked
/// and, with the auto-target option on, the next foe is picked; the selection then clears when
/// it is still this object.
pub fn being_deleted(
    object: u64,
    snapshot: &SelectionSnapshot,
    auto_target: bool,
) -> Vec<SelectionAction> {
    if snapshot.selection != Some(object) {
        return Vec::new();
    }
    let mut out = Vec::new();
    if auto_target {
        out.push(SelectionAction::StopAttacking(object));
        out.push(SelectionAction::Key(AUTO_TARGET_KEY));
    }
    out.push(SelectionAction::ClearIfSelected(object));
    out
}

/// The keys `gmSelectionUI::Select` answers without cycling: reselect (the player, or the last
/// selection when the player is already selected), the last selection, the last attacker, and
/// the clear key (`typing`: the input takes text, `Select`'s `ICIDM` check, so nothing happens).
/// `None` for the cycling keys.
pub fn direct_key(
    key: u32,
    snapshot: &SelectionSnapshot,
    typing: bool,
) -> Option<Vec<SelectionAction>> {
    let action = match key {
        RESELECT_KEY if snapshot.selection != Some(snapshot.player) => {
            SelectionAction::Select(Some(snapshot.player))
        }
        RESELECT_KEY | LAST_SELECTION_KEY => SelectionAction::Select(snapshot.last_selection),
        LAST_ATTACKER_KEY => SelectionAction::Select(snapshot.last_attacker),
        CLEAR_KEY if typing => return Some(Vec::new()),
        CLEAR_KEY => SelectionAction::Select(None),
        _ => return None,
    };
    Some(vec![action])
}

/// The `ItemSelectionField` root's index, when the HUD has it.
fn item_selection_field(tree: &UiTree) -> Option<usize> {
    let field = tokenize(ITEM_SELECTION_FIELD);
    tree.roots()
        .iter()
        .copied()
        .find(|&root| tree.el(root).id() == field)
}

/// `gmSelectionUI::RecvNotice_SetTargetingPointer(obj, usagePointerOn)`: with the pointer on, the
/// field's `usage_cursor` (its `usage_cursor_x` and `usage_cursor_y` hotspot) replaces the
/// cursor and the old one is saved (`UIElement::SaveCursor`); off, the saved cursor comes back
/// (`UIElement::RestoreSavedCursor`). Nothing happens when the field or its attributes are
/// missing.
pub fn set_targeting_pointer(tree: &mut UiTree, on: bool) {
    if let Some(index) = item_selection_field(tree) {
        swap_usage_cursor(tree, index, on);
    }
}

/// The body `gmSelectionUI` and `ItemDisplay` share (`RecvNotice_SetTargetingPointer`): on, the
/// element's `usage_cursor` with its hotspot replaces its cursor, saved (`SaveCursor`,
/// `SetCursor`), when all three attributes are there and the cursor is not 0; off, the saved
/// cursor comes back (`RestoreSavedCursor`).
pub(crate) fn swap_usage_cursor(tree: &mut UiTree, index: usize, on: bool) {
    if !on {
        let (file, x, y) = tree.el(index).saved_cursor;
        tree.set_element_cursor(index, file, x, y);
        if file == 0 && tree.el(index).mouse_over() {
            tree.use_default_pointer_cursor();
        }
        return;
    }
    let element = tree.el(index);
    let x = int_attribute(element.attribute(tokenize("usage_cursor_x")));
    let y = int_attribute(element.attribute(tokenize("usage_cursor_y")));
    let file = match element.attribute(tokenize("usage_cursor")) {
        Some(AttributeValue::DataId(file)) => Some(*file),
        _ => None,
    };
    if let (Some(x), Some(y), Some(file)) = (x, y, file) {
        if file != 0 {
            let cursor = tree.el(index).cursor;
            tree.el_mut(index).saved_cursor = cursor;
            tree.set_element_cursor(index, file, x, y);
        }
    }
}

fn int_attribute(value: Option<&AttributeValue>) -> Option<i32> {
    match value {
        Some(AttributeValue::Int(v)) => Some(*v),
        _ => None,
    }
}
