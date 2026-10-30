//! `UIElement_Menu`: a drop-down. The element holds its `popup` (a layout element made when the
//! menu is initialised, not a child of the menu) and the popup's list box. A left press on the
//! menu (not ghosted) opens the popup under it (`UIElement_Menu::Open`): the popup becomes a root
//! while open, sized to the list's items, and centred under the menu (or above it with
//! `open_up`). A press anywhere else closes it (`ListenToMessage` on the global mouse messages),
//! and so does a release over the list. The list's selection is the menu's: a new selection in
//! the list broadcasts the menu's `new_selection` with the selected item's element id, and the
//! display element's index (`display_selection`). The menu's state follows `ghosted`, the open
//! popup and the pointer over its top (`UIElement_Menu::UpdateState`).
//!
//! Items are added at runtime (`InsertTextItem`: a text element made from the layout's
//! `text_item` template and inserted in the list). The menu defines no key handling in retail.
//!
//! Not modelled yet: the popup's z order against the other roots (it keeps the z of its layout
//! element). A menu is never a window's drag bar or grip, so
//! `MouseDown`'s `m_moving` / `m_resizing` test (`UiTree::movement`, `resizing`) never fails.

use crate::element::UiTree;
use crate::layout::{AttributeValue, ElementDesc, LayoutDesc};
use crate::list_box_select::LIST_BOX;
use crate::mouse::{GLOBAL_MOUSE_DOWN, GLOBAL_MOUSE_UP};
use crate::token::{tokenize, TEXT};

/// The element class `UIElement_Menu` (`Token_menu`).
pub(crate) const MENU: u32 = tokenize("menu");
/// `MenuMsg_new_selection`: the new selection's element id (0 for none).
pub const MSG_MENU_NEW_SELECTION: u32 = tokenize("new_selection");
/// `MenuMsg_opened` and `MenuMsg_closed`.
pub const MSG_MENU_OPENED: u32 = tokenize("opened");
pub const MSG_MENU_CLOSED: u32 = tokenize("closed");

const GHOSTED: u32 = tokenize("ghosted");
const OPEN_UP: u32 = tokenize("open_up");
const POPUP: u32 = tokenize("popup");
const LISTBOX: u32 = tokenize("listbox");
const TEXT_ITEM: u32 = tokenize("text_item");
const SELECTION_DISPLAY: u32 = tokenize("selection_display");
const DEFAULT_SELECTED_ITEM: u32 = tokenize("default_selected_item");
/// `MenuState_closed`, `_open`, `_rollover`, `_ghosted`.
const STATE_CLOSED: u32 = tokenize("closed");
const STATE_OPEN: u32 = tokenize("open");
const STATE_ROLLOVER: u32 = tokenize("rollover");
const STATE_GHOSTED: u32 = tokenize("ghosted");
/// The left button (`UIElement_Menu::MouseDown` takes only the press, not the double click).
const LEFT_PRESS: u32 = 0x4000_0004;

/// A menu's own state (`m_popup`, `m_listBox`, `m_open`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MenuState {
    /// `m_popup`: the drop-down made from the layout's `popup`. A root while the menu is open.
    pub popup: Option<usize>,
    /// `m_listBox`: the popup's `listbox`.
    pub listbox: Option<usize>,
    /// `m_open`.
    pub open: bool,
}

/// The layout's element `id`: a top-level element, else one nested in the layout.
fn layout_element(layout: &LayoutDesc, id: u32) -> Option<std::sync::Arc<ElementDesc>> {
    layout
        .element(id)
        .cloned()
        .or_else(|| layout.find_element(id))
}

impl UiTree {
    pub(crate) fn is_menu(&self, index: usize) -> bool {
        self.el(index).desc.kind == MENU
    }

    /// `GetAttribute_ID` of the element, when it is set.
    fn menu_id(&self, index: usize, id: u32) -> Option<u32> {
        match self.el(index).attributes.get(&id) {
            Some(AttributeValue::Id(value)) | Some(AttributeValue::StringId(value)) => Some(*value),
            _ => None,
        }
    }

    fn menu_state(&self, menu: usize) -> Option<&MenuState> {
        self.el(menu).menu.as_deref()
    }

    fn menu_state_mut(&mut self, menu: usize) -> &mut MenuState {
        self.el_mut(menu).menu.get_or_insert_with(Default::default)
    }

    /// `UIElement_Menu::Initialized`: the popup is made from the layout, its list box found and
    /// the default item selected. Runs once the menu's own children exist.
    pub(crate) fn menu_initialize(&mut self, menu: usize) {
        let layout = std::sync::Arc::clone(&self.el(menu).layout);
        let Some(popup_id) = self.menu_id(menu, POPUP) else {
            return;
        };
        let Some(desc) = layout_element(&layout, popup_id) else {
            return;
        };
        let Some(popup) = self.create_element(layout, desc, true) else {
            return;
        };
        self.el_mut(menu).menu = Some(Box::new(MenuState {
            popup: Some(popup),
            listbox: None,
            open: false,
        }));
        let listbox = self
            .menu_id(menu, LISTBOX)
            .and_then(|id| self.child_recursive(popup, id))
            .filter(|&child| self.el(child).desc.kind == LIST_BOX);
        let Some(list) = listbox else {
            return;
        };
        self.menu_state_mut(menu).listbox = Some(list);
        self.list(list).menu = Some(menu);
        if let Some(default) = self.menu_id(menu, DEFAULT_SELECTED_ITEM) {
            let item = self
                .list_items(list)
                .iter()
                .copied()
                .find(|&item| self.el(item).id() == default);
            if let Some(item) = item {
                self.list_set_selected_item(list, Some(item), true);
            }
        }
    }

    /// `UIElement_Menu::GetItem` (`GetNumItems` is the length of [`UiTree::menu_items`]).
    pub fn menu_items(&self, menu: usize) -> &[usize] {
        match self.menu_state(menu).and_then(|state| state.listbox) {
            Some(list) => self.list_items(list),
            None => &[],
        }
    }

    /// `UIElement_Menu::GetItem`.
    pub fn menu_item(&self, menu: usize, index: usize) -> Option<usize> {
        self.menu_items(menu).get(index).copied()
    }

    /// `UIElement_Menu::GetSelectedIndex` (`None` for -1).
    pub fn menu_selected_index(&self, menu: usize) -> Option<usize> {
        let list = self.menu_state(menu)?.listbox?;
        self.list_selected_index(list)
    }

    /// `UIElement_Menu::GetSelectedItem`.
    pub fn menu_selected_item(&self, menu: usize) -> Option<usize> {
        let list = self.menu_state(menu)?.listbox?;
        self.list_selected_item(list)
    }

    /// `UIElement_Menu::SetSelectedIndex`: broadcasts the new selection.
    pub fn menu_set_selected_index(&mut self, menu: usize, index: Option<usize>) {
        if let Some(list) = self.menu_state(menu).and_then(|state| state.listbox) {
            match index {
                Some(index) => self.list_set_selected_index(list, Some(index), true),
                None => self.list_set_selected_item(list, None, true),
            }
        }
    }

    /// `UIElement_Menu::SetSelectedItem`: broadcasts the new selection.
    pub fn menu_set_selected_item(&mut self, menu: usize, item: Option<usize>) {
        if let Some(list) = self.menu_state(menu).and_then(|state| state.listbox) {
            self.list_set_selected_item(list, item, true);
        }
    }

    /// `UIElement_Menu::Flush`: every item deleted.
    pub fn menu_flush(&mut self, menu: usize) {
        if let Some(list) = self.menu_state(menu).and_then(|state| state.listbox) {
            self.list_flush(list);
        }
    }

    /// `UIElement_Menu::InsertTextItem` with a plain string: a text element made from the
    /// layout's `text_item` template, its text set and inserted at `here`. `None` when the
    /// template is missing or is not a text element.
    pub fn menu_insert_text(&mut self, menu: usize, text: &str, here: usize) -> Option<usize> {
        let item = self.menu_text_item(menu)?;
        self.set_text(item, text);
        self.menu_insert_item(menu, item, here)
    }

    /// `UIElement_Menu::InsertTextItem` with a string-table entry (`StringInfo`): the text
    /// element's text is string `id` of `table`.
    pub fn menu_insert_string(
        &mut self,
        menu: usize,
        table: u32,
        id: u32,
        here: usize,
    ) -> Option<usize> {
        let string = self.text_data.as_ref()?.string(table, id)?;
        let item = self.menu_text_item(menu)?;
        self.set_text(item, &String::from_utf16_lossy(&string));
        self.menu_insert_item(menu, item, here)
    }

    /// The text element made for an item (deleted again when it is not a text element).
    fn menu_text_item(&mut self, menu: usize) -> Option<usize> {
        let id = self.menu_id(menu, TEXT_ITEM)?;
        let layout = std::sync::Arc::clone(&self.el(menu).layout);
        let desc = layout_element(&layout, id)?;
        let item = self.create_element(layout, desc, false)?;
        if self.el(item).desc.kind == TEXT {
            Some(item)
        } else {
            self.delete(item);
            None
        }
    }

    fn menu_insert_item(&mut self, menu: usize, item: usize, here: usize) -> Option<usize> {
        match self.menu_state(menu).and_then(|state| state.listbox) {
            Some(list) => {
                self.list_insert_item(list, item, here);
                Some(item)
            }
            None => {
                self.delete(item);
                None
            }
        }
    }

    /// `UIElement_Menu::Open`: the popup becomes a root, sized to its items and placed under the
    /// menu (above it with `open_up`), then the menu is open and `opened` is broadcast.
    pub fn menu_open(&mut self, menu: usize) {
        let Some(state) = self.menu_state(menu).cloned() else {
            return;
        };
        let (Some(popup), Some(list)) = (state.popup, state.listbox) else {
            return;
        };
        if state.open || self.get(popup).is_none() || self.get(list).is_none() {
            return;
        }
        let origin = self.screen_box(menu);
        self.add_root(popup);
        // The popup keeps the list's margins: its size is the widest item and the items' total
        // height, plus whatever the popup has beyond the list (never less than zero).
        let p = self.el(popup).rect;
        let l = self.el(list).rect;
        let width_extra = ((p.x1 - p.x0) - (l.x1 - l.x0)).max(0);
        let height_extra = ((p.y1 - p.y0) - (l.y1 - l.y0)).max(0);
        let (mut widest, mut total) = (0, 0);
        for &item in self.list_items(list) {
            let r = self.el(item).rect;
            widest = widest.max(r.x1 - r.x0 + 1);
            total += r.y1 - r.y0 + 1;
        }
        self.resize_to(popup, widest + width_extra, total + height_extra);
        // Centred under the menu, by the difference of the widths (rounded towards zero, as
        // retail's integer division does).
        let m = self.el(menu).rect;
        let p = self.el(popup).rect;
        let half = ((p.x1 - p.x0) - (m.x1 - m.x0)) / 2;
        let x = origin.x0 - half;
        let y = if self.bool_attribute(menu, OPEN_UP) {
            origin.y0 - (p.y1 - p.y0) - 1
        } else {
            origin.y0 + (m.y1 - m.y0)
        };
        self.move_to(popup, x, y);
        self.changed = true;
        self.moved = true;
        self.menu_state_mut(menu).open = true;
        self.menu_update_state(menu);
        self.broadcast(menu, MSG_MENU_OPENED, 0);
    }

    /// `UIElement_Menu::Close`: the popup leaves the roots (it stays for the next open).
    pub fn menu_close(&mut self, menu: usize) {
        let Some(state) = self.menu_state(menu).cloned() else {
            return;
        };
        if !state.open {
            return;
        }
        if let Some(popup) = state.popup {
            self.roots.retain(|&root| root != popup);
        }
        self.changed = true;
        self.moved = true;
        self.menu_state_mut(menu).open = false;
        self.menu_update_state(menu);
        self.broadcast(menu, MSG_MENU_CLOSED, 0);
    }

    /// The menu's popup element (`m_popup`).
    pub fn menu_popup(&self, menu: usize) -> Option<usize> {
        self.menu_state(menu).and_then(|state| state.popup)
    }

    /// Whether the menu's popup is open.
    pub fn menu_is_open(&self, menu: usize) -> bool {
        self.menu_state(menu).is_some_and(|state| state.open)
    }

    /// Drops the menu's `new_selection` messages not yet taken: a selection made while the page
    /// was built, before its listener was registered (`gmFriendsUI::gmFriendsUI`).
    pub fn menu_drop_selection_messages(&mut self, menu: usize) {
        self.messages
            .retain(|m| !(m.element == Some(menu) && m.message == MSG_MENU_NEW_SELECTION));
    }

    /// `UIElement_Menu::UpdateState`: ghosted, else open, else rolled over (the pointer over the
    /// menu's top), else closed.
    pub(crate) fn menu_update_state(&mut self, menu: usize) {
        if !self.is_menu(menu) {
            return;
        }
        let state = if self.bool_attribute(menu, GHOSTED) {
            STATE_GHOSTED
        } else if self.menu_is_open(menu) {
            STATE_OPEN
        } else if self.el(menu).mouse_over_top {
            STATE_ROLLOVER
        } else {
            STATE_CLOSED
        };
        self.set_state(menu, state);
    }

    /// `UIElement_Menu::SetAttribute_Bool`, after a change: ghosting closes an open menu.
    pub(crate) fn menu_attribute(&mut self, menu: usize, id: u32, value: bool) {
        if !self.is_menu(menu) || id != GHOSTED {
            return;
        }
        if value && self.menu_is_open(menu) {
            self.menu_close(menu);
        }
        self.menu_update_state(menu);
    }

    /// `UIElement_Menu::SetVisible`: a menu that is hidden, or whose parent is, closes.
    pub(crate) fn menu_visibility(&mut self, menu: usize) {
        if !self.is_menu(menu) || !self.menu_is_open(menu) {
            return;
        }
        let parent_visible = self
            .el(menu)
            .parent
            .is_none_or(|parent| self.el(parent).visible);
        if !(self.el(menu).visible && parent_visible) {
            self.menu_close(menu);
        }
    }

    /// `UIElement_Menu::MouseDown`, after `UIElement::MouseDown`: a left press on a menu that is
    /// not ghosted opens it, or closes it when open; the press is taken.
    pub(crate) fn menu_mouse_down(&mut self, menu: usize, action: u32, processed: &mut bool) {
        if !self.is_menu(menu) || action != LEFT_PRESS || self.bool_attribute(menu, GHOSTED) {
            return;
        }
        if self.menu_is_open(menu) {
            self.menu_close(menu);
        } else {
            self.menu_open(menu);
        }
        *processed = true;
    }

    /// `UIElement_Menu::ListenToMessage` on the global mouse messages: a press outside the menu
    /// and its popup closes it; a release closes it unless the pointer is over the menu or over
    /// the popup but not its list.
    pub(crate) fn menu_global_mouse(&mut self, message: u32) {
        let open: Vec<usize> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let menu = slot.as_ref()?.menu.as_deref()?;
                menu.open.then_some(index)
            })
            .collect();
        for menu in open {
            let Some(state) = self.menu_state(menu).cloned() else {
                continue;
            };
            let (Some(popup), Some(list)) = (state.popup, state.listbox) else {
                continue;
            };
            let over = |index: usize| self.el(index).mouse_over;
            let close = match message {
                GLOBAL_MOUSE_DOWN => !over(menu) && !over(popup),
                GLOBAL_MOUSE_UP => !over(menu) && (!over(popup) || over(list)),
                _ => false,
            };
            if close {
                self.menu_close(menu);
            }
        }
    }

    /// `UIElement_Menu::NewSelection`, on the list's `new_selection`: the display text element
    /// (`selection_display`, a child of the menu) shows the selected item's text
    /// (`UIElement_Text::ListenToMessage`'s `display_selection`), then `new_selection` is
    /// broadcast with the selected item's element id (0 for none).
    pub(crate) fn menu_new_selection(&mut self, menu: usize) {
        let Some(list) = self.menu_state(menu).and_then(|state| state.listbox) else {
            return;
        };
        let display = self
            .menu_id(menu, SELECTION_DISPLAY)
            .filter(|&id| id != 0)
            .and_then(|id| self.child_recursive(menu, id))
            .filter(|&child| self.el(child).desc.kind == TEXT);
        if let Some(display) = display {
            match self.list_selected_item(list) {
                Some(item) if self.el(item).desc.kind == TEXT => {
                    let text = self.el(item).text().unwrap_or_default();
                    self.set_text(display, &text);
                }
                Some(_) => {}
                None => self.set_text(display, ""),
            }
        }
        let id = self
            .list_selected_item(list)
            .map_or(0, |item| self.el(item).id());
        self.broadcast(menu, MSG_MENU_NEW_SELECTION, id as i32);
    }

    /// The list's `new_selection` reaches its menu (the menu listens to its list).
    pub(crate) fn menu_list_selection(&mut self, list: usize) {
        let Some(menu) = self.el(list).list.as_ref().and_then(|state| state.menu) else {
            return;
        };
        if self.get(menu).is_some() && self.menu_state(menu).and_then(|s| s.listbox) == Some(list) {
            self.menu_new_selection(menu);
        }
    }

    /// `UIElement_Menu::~UIElement_Menu`: the popup is deleted with the menu (it is not a child).
    pub(crate) fn menu_delete(&mut self, menu: usize) {
        let Some(popup) = self.menu_state(menu).and_then(|state| state.popup) else {
            return;
        };
        self.roots.retain(|&root| root != popup);
        if self.get(popup).is_some() {
            self.delete(popup);
        }
    }
}
