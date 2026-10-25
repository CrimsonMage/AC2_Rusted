//! `UIElement_Panel` and its tabs, `UIElement_PanelPage`. A page tab names its content by the
//! `page` attribute: `Initialized` makes that element of the tab's layout a child of the tab's
//! parent, shown only while the tab is `open`. A left press opens a closed tab
//! (`StartUseActive`); the pointer over it turns a closed tab to `rollover` and back
//! (`MouseOverTop`). Opening or closing broadcasts `opened` / `closed` (`SetState`), and the panel
//! closes its other open tabs when one opens (`UIElement_Panel::ListenToMessage`).
//!
//! Not modelled yet: keyboard activation (`Activate` / `Deactivate`'s rollover).

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::tokenize;

pub(crate) const PANEL: u32 = tokenize("panel");
pub(crate) const PANEL_PAGE: u32 = tokenize("panel_page");
pub(crate) const PAGE_OPEN: u32 = tokenize("open");
pub(crate) const PAGE_CLOSED: u32 = tokenize("closed");
const PAGE_ROLLOVER: u32 = tokenize("rollover");
const PAGE_GHOSTED: u32 = tokenize("ghosted");
pub(crate) const MSG_OPENED: u32 = tokenize("opened");
const MSG_CLOSED: u32 = tokenize("closed");
const LEFT: u32 = 0x4000_0004;

impl UiTree {
    pub(crate) fn is_panel_page(&self, index: usize) -> bool {
        self.el(index).desc.kind == PANEL_PAGE
    }

    /// `UIElement_PanelPage::Initialized` for every tab under `index` (the content is a top-level
    /// element of the tab's layout).
    pub(crate) fn initialize_panel_pages(&mut self, index: usize) {
        for child in self.el(index).children.clone() {
            self.initialize_panel_pages(child);
        }
        if !self.is_panel_page(index) || self.el(index).page.is_some() {
            return;
        }
        let id = match self.el(index).attributes.get(&tokenize("page")) {
            Some(AttributeValue::Id(id)) => *id,
            _ => return,
        };
        let layout = Arc::clone(&self.el(index).layout);
        let Some(reference) = layout.element(id).cloned() else {
            return;
        };
        let Some(parent) = self.el(index).parent else {
            return;
        };
        let Some(page) = self.create_element(layout, reference, false) else {
            return;
        };
        self.set_parent(page, Some(parent));
        self.update_size_and_position(page);
        self.initialize_panel_pages(page);
        let open = self.el(index).state == PAGE_OPEN;
        self.set_visible(page, open);
        self.el_mut(index).page = Some(page);
    }

    /// `UIElement_PanelPage::SetState` after the state changed: the content shows when open; the
    /// panel closes the other open tabs.
    pub(crate) fn panel_page_state(&mut self, index: usize, state: u32) {
        let Some(page) = self.el(index).page else {
            return;
        };
        self.set_visible(page, state == PAGE_OPEN);
        if state == PAGE_OPEN {
            self.broadcast(index, MSG_OPENED, 0);
            let Some(panel) = self
                .el(index)
                .parent
                .filter(|&p| self.el(p).desc.kind == PANEL)
            else {
                return;
            };
            for tab in self.el(panel).children.clone() {
                if tab != index && self.is_panel_page(tab) && self.el(tab).state == PAGE_OPEN {
                    self.set_state(tab, PAGE_CLOSED);
                }
            }
        } else if state == PAGE_CLOSED {
            self.broadcast(index, MSG_CLOSED, 0);
        }
    }

    /// `UIElement_PanelPage::MouseDown`: a left press opens the tab (`StartUseActive`).
    pub(crate) fn panel_page_mouse_down(&mut self, index: usize, action: u32) {
        if !self.is_panel_page(index) || action != LEFT {
            return;
        }
        let state = self.el(index).state;
        if state != PAGE_OPEN && state != PAGE_GHOSTED {
            self.set_state(index, PAGE_OPEN);
        }
    }

    /// `UIElement_PanelPage::MouseOverTop`.
    pub(crate) fn panel_page_mouse_over_top(&mut self, index: usize, over: bool) {
        if !self.is_panel_page(index) {
            return;
        }
        let state = self.el(index).state;
        if !over && state == PAGE_ROLLOVER {
            self.set_state(index, PAGE_CLOSED);
        } else if over && state == PAGE_CLOSED {
            self.set_state(index, PAGE_ROLLOVER);
        }
    }
}
