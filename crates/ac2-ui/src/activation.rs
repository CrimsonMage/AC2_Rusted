//! Keyboard use of the active element: Enter goes to it (`UIElementManager::KeyPressEvent` /
//! `KeyReleaseEvent`: `StartUseActive` / `EndUseActive`, which a button takes as a press and a
//! release on it), and `UIElement::SetVisible` activates an `activate_on_show` element as it
//! shows (the dialogs' `OkButton`, `YesButton` and `ResurrectButton`) and deactivates an active
//! one as it hides.
//!
//! Admin 7148's `KeyPressEvent` also stepped through the activatable elements on Tab
//! (`ActivateNext`, over `RegisterActivatable`'s list); Legions 7160 (0x005A4440) dropped that
//! case, so the DVD client has no Tab field navigation and neither does this one.

use crate::element::UiTree;
use crate::mouse::{GLOBAL_KEY_PRESS, GLOBAL_KEY_RELEASE};
use crate::token::tokenize;

/// `Token_Element_activate_on_show`.
const ACTIVATE_ON_SHOW: u32 = tokenize("activate_on_show");
/// The UI key map's use-active action (Enter).
pub(crate) const UI_USE_ACTIVE: u32 = 0x4000_0018;

impl UiTree {
    /// `UIElement::SetVisible` (`UIRegion::SetVisible`). When whether the element is shown (its
    /// own flag and its parent's `UIRegion::IsVisible`) changes, a hidden active element
    /// deactivates and a shown `activate_on_show` one activates, unless the use-active key is
    /// held (the Enter that brought it up does not press it).
    pub fn set_visible(&mut self, index: usize, visible: bool) {
        let before = self.el(index).visible && self.parent_shown(index);
        self.el_mut(index).visible = visible;
        self.changed = true;
        self.menu_visibility(index);
        let shown = visible && self.parent_shown(index);
        // `UIElementManager::ChangingElementVisibility`: a tooltip owner that is no longer shown
        // loses its tooltip.
        if !shown && self.tooltip.owner == Some(index) {
            self.tooltip.owner = None;
            self.clear_tooltip();
        }
        if shown == before {
            return;
        }
        if !shown {
            if self.active == Some(index) {
                self.deactivate(index);
            }
        } else if self.bool_attribute(index, ACTIVATE_ON_SHOW) && !self.use_held {
            self.activate(index);
        }
    }

    /// `UIRegion::IsVisible` of the element's parent (shown when it has none).
    fn parent_shown(&self, index: usize) -> bool {
        self.el(index)
            .parent
            .is_none_or(|parent| self.get(parent).is_some() && self.region_visible(parent))
    }

    /// `UIElementManager::KeyPressEvent`: Enter goes to the active element's `StartUseActive`,
    /// and a key it takes is not broadcast.
    pub(crate) fn key_press_event(&mut self, action: u32) {
        if action == UI_USE_ACTIVE {
            self.use_held = true;
            if let Some(active) = self.active {
                if self.start_use_active_element(active) {
                    return;
                }
            }
        }
        self.broadcast_global(GLOBAL_KEY_PRESS, action);
    }

    /// `UIElementManager::KeyReleaseEvent`: Enter's release goes to the active button's
    /// `EndUseActive`, and a release it takes is not broadcast.
    pub(crate) fn key_release_event(&mut self, action: u32) {
        if action == UI_USE_ACTIVE {
            self.use_held = false;
            if let Some(active) = self.active.filter(|&a| self.is_button(a)) {
                self.button_end_use_active(active);
                return;
            }
        }
        self.broadcast_global(GLOBAL_KEY_RELEASE, action);
    }

    /// The active element's `StartUseActive` (vtable +0xd8): a button presses, a panel page's
    /// tab opens; other elements leave the key alone.
    fn start_use_active_element(&mut self, index: usize) -> bool {
        if self.is_button(index) {
            self.button_start_use_active(index);
            true
        } else if self.is_panel_page(index) {
            self.start_use_active(index);
            true
        } else {
            false
        }
    }
}
