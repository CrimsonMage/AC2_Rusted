//! Typing into text elements: activation (`UIElement::Activate` / `Deactivate`,
//! `UIElementManager::ActivationAlert`: one active element; an editable text element turns on the
//! input manager's text mode while active), the text element's handlers (`UIElement_Text::
//! CharacterHandler` for typed characters and virtual keys, the `ActionHandler` navigation keys),
//! the cursor (`MoveCursor`, `SetCursorPosition`), the selection (`SetSelecting`,
//! `SetSelectionStart` / `End`, `GetSelection`, `SelectAll`, `Deselect`, `DeleteSelection`),
//! the copy, cut and paste keys, and clicking to place the cursor (`UIElement_Text::MouseDown`).
//! The element broadcasts `activated` / `deactivated`, `character_input` (with the character) and
//! `key_input`.
//!
//! The clipboard keys and the pointer drag selection are in `text_clip.rs`; the clipboard itself
//! is `UiTree::set_text_clipboard`.
//!
//! Not modelled yet: the IME and the cursor's flashing.

use crate::chat::Modifiers;
use crate::element::UiTree;
use crate::mouse::{UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::token::tokenize;

/// `UIElement_Text::MoveCursor`'s directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Home,
    End,
    LineHome,
    LineEnd,
    WordLeft,
    WordRight,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
}

/// `LEFT_MOUSE_BUTTON` (`0x40000004`): the press that places the cursor.
const LEFT_BUTTON: u32 = 0x4000_0004;

fn message(name: &str) -> u32 {
    tokenize(name)
}

impl UiTree {
    /// The active element (`m_activeElement`).
    pub fn active(&self) -> Option<usize> {
        self.active
    }

    /// The text element typed keys go to: the active one when it is text, else the read-only
    /// text that holds the copy focus.
    fn text_target(&self) -> Option<usize> {
        self.active
            .filter(|&index| self.get(index).is_some_and(|e| e.text.is_some()))
            .or_else(|| self.copy_focus.filter(|&index| self.get(index).is_some()))
    }

    /// Whether the active element takes typing (the input manager's text mode).
    pub fn text_mode(&self) -> bool {
        self.active.is_some_and(|index| {
            self.get(index)
                .and_then(|e| e.text.as_ref())
                .is_some_and(|t| t.editable)
        })
    }

    /// `UIElement::Activate` (and `UIElement_Text::Activate`): a visible element whose parent is
    /// shown (`m_visible & UIRegion::IsVisible(m_parent)`, admin 0x5e9489) becomes the active
    /// one, the previous one deactivating.
    pub fn activate(&mut self, index: usize) -> bool {
        let el = self.el(index);
        let shown = el.parent.is_none_or(|parent| self.region_visible(parent));
        if !(el.visible && shown) {
            return false;
        }
        if self.active == Some(index) {
            return true;
        }
        if let Some(previous) = self.active {
            self.deactivate(previous);
        }
        self.active = Some(index);
        // `UIElement_Button::Activate`: the state is refreshed.
        self.button_update_state(index);
        if self.el(index).text.as_ref().is_some_and(|t| t.editable) {
            self.broadcast(index, message("activated"), 0);
            self.changed = true;
        }
        true
    }

    /// `UIRegion::IsVisible`: the region and every parent visible.
    fn region_visible(&self, index: usize) -> bool {
        let el = self.el(index);
        el.visible && el.parent.is_none_or(|parent| self.region_visible(parent))
    }

    /// `UIElement::Deactivate` (and `UIElement_Text::Deactivate`).
    pub fn deactivate(&mut self, index: usize) {
        if self.active != Some(index) {
            return;
        }
        self.active = None;
        // `UIElement_Button::Deactivate`: the state is refreshed.
        self.button_update_state(index);
        if self.el(index).text.as_ref().is_some_and(|t| t.editable) {
            self.broadcast(index, message("deactivated"), 0);
            self.changed = true;
        }
    }

    /// Whether the element can hold a selection: a `selectable` one, or any editable one (the
    /// modern selection exception in AGENTS.md).
    pub(crate) fn selectable(&self, index: usize) -> bool {
        self.bool_attribute(index, tokenize("selectable"))
            || self.el(index).text.as_ref().is_some_and(|t| t.editable)
    }

    /// `UIElement_Text::SetSelecting`: turning the selection on resets its ends to 0.
    fn set_selecting(&mut self, index: usize, on: bool) {
        let Some(text) = self.el_mut(index).text.as_mut() else {
            return;
        };
        if text.selecting == on {
            return;
        }
        if on {
            text.selection = (0, 0);
        }
        text.selecting = on;
        self.changed = true;
    }

    /// `UIElement_Text::SetSelectionStart`: the anchor at `position` (both ends), when selecting.
    fn set_selection_start(&mut self, index: usize, position: usize) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            if text.selecting {
                let position = position.min(text.glyphs.len());
                text.selection = (position, position);
            }
        }
    }

    /// `UIElement_Text::SetSelectionEnd`: the moving end at `position`, when selecting.
    fn set_selection_end(&mut self, index: usize, position: usize) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            if text.selecting {
                text.selection.1 = position.min(text.glyphs.len());
            }
        }
    }

    /// `UIElement_Text::GetSelection`: the selected range, lowest first; `None` when off.
    pub fn selection(&self, index: usize) -> Option<(usize, usize)> {
        let text = self.get(index)?.text.as_ref()?;
        text.selecting.then(|| {
            let (a, b) = text.selection;
            (a.min(b), a.max(b))
        })
    }

    /// `UIElement_Text::SelectAll`: the whole text selected, on a selectable element.
    pub(crate) fn select_all(&mut self, index: usize) {
        if !self.selectable(index) {
            return;
        }
        let Some(len) = self
            .get(index)
            .and_then(|e| e.text.as_ref())
            .map(|t| t.glyphs.len())
        else {
            return;
        };
        self.set_selecting(index, true);
        self.set_selection_start(index, 0);
        self.set_selection_end(index, len);
        self.changed = true;
    }

    /// `UIElement_Text::Deselect`.
    pub fn deselect(&mut self, index: usize) {
        self.set_selecting(index, false);
    }

    /// `UIElement_Text::GetSelectedText`: the selected characters (none when off).
    pub(crate) fn selected_text(&self, index: usize) -> Vec<u16> {
        let Some((start, end)) = self.selection(index) else {
            return Vec::new();
        };
        self.get(index)
            .and_then(|e| e.text.as_ref())
            .and_then(|t| t.glyphs.get(start..end))
            .map_or_else(Vec::new, |glyphs| glyphs.iter().map(|g| g.ch).collect())
    }

    /// `UIElement_Text::DeleteSelection`: the selected glyphs go and the selection ends.
    pub(crate) fn delete_selection(&mut self, index: usize) {
        let Some((start, end)) = self.selection(index) else {
            return;
        };
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.delete_range(start, end);
            text.selecting = false;
        }
        self.changed = true;
    }

    /// `UIElement_Text::SetCursorPosition`, clamped to the text. Shift extends a selection from
    /// the old cursor on a selectable element; a move without shift drops the selection
    /// (`m_mouseSelecting` is never set in the Legions 7160 text code, so it stays off here).
    pub(crate) fn set_cursor(&mut self, index: usize, position: isize, shift: bool) {
        let selecting = self
            .get(index)
            .and_then(|e| e.text.as_ref())
            .is_some_and(|t| t.selecting);
        if !shift {
            if selecting {
                self.set_selecting(index, false);
            }
        } else if !selecting && self.selectable(index) {
            let old = self
                .get(index)
                .and_then(|e| e.text.as_ref())
                .map_or(0, |t| t.cursor);
            self.set_selecting(index, true);
            self.set_selection_start(index, old);
        }
        let Some(text) = self.el_mut(index).text.as_mut() else {
            return;
        };
        let position = position.clamp(0, text.glyphs.len() as isize) as usize;
        text.cursor = position;
        self.set_selection_end(index, position);
        self.changed = true;
    }

    /// `UIElement_Text::ScrollHorizontalView` on a one-line element: the view moves the least
    /// to keep the cursor's column inside the element's margins.
    pub(crate) fn scroll_to_cursor(&mut self, index: usize) {
        let width = self.el(index).rect.width();
        let Some(text) = self.el_mut(index).text.as_mut() else {
            return;
        };
        if !text.one_line {
            return;
        }
        let x: i32 = text.glyphs[..text.cursor].iter().map(|g| g.width).sum();
        let [_, _, left, right] = text.margins;
        let drawable = (width - left - right).max(1);
        let view = text.view.0;
        text.view.0 = if x < view {
            x
        } else if x - view >= drawable {
            x - drawable + 1
        } else {
            view
        }
        .max(0);
        self.changed = true;
    }

    /// `UIElement_Text::MoveCursor`: the cursor moves in `direction`; `false` when it cannot.
    pub(crate) fn move_cursor(&mut self, index: usize, direction: Direction, shift: bool) -> bool {
        let Some(text) = self.el(index).text.as_ref() else {
            return false;
        };
        let cursor = text.cursor as isize;
        let len = text.glyphs.len() as isize;
        // The searches are `GlyphList`'s (`glyph_find.rs`); a move one of them refuses is no move.
        let target = match direction {
            Direction::Home => Some(0),
            Direction::End => Some(len),
            Direction::LineHome => text.move_begin_end_line(true, cursor).map(|p| p as isize),
            Direction::LineEnd => text.move_begin_end_line(false, cursor).map(|p| p as isize),
            Direction::Left => Some(cursor - 1),
            Direction::Right => Some(cursor + 1),
            Direction::WordLeft => text.find_prev_word(cursor).map(|p| p as isize),
            Direction::WordRight => text.find_next_word(cursor).map(|p| p as isize),
            Direction::Up => text.move_up_down(true, cursor).map(|p| p as isize),
            Direction::Down => text.move_up_down(false, cursor).map(|p| p as isize),
            Direction::PageUp | Direction::PageDown => None,
        };
        // `DetermineAndSetNewPage`, then the position at the new page's top left corner.
        let target = match direction {
            Direction::PageUp | Direction::PageDown => {
                self.text_new_page(index, true, direction == Direction::PageDown);
                self.el(index)
                    .text
                    .as_ref()
                    .map(|text| text.position_at(0, 0) as isize)
            }
            _ => target,
        };
        let Some(target) = target else {
            return false;
        };
        self.set_cursor(index, target, shift);
        self.scroll_to_cursor(index);
        true
    }

    /// `UIElement_Text::ActionHandler` on the active element: the navigation actions while held
    /// (`ActionInterpreter::CtrlKeyDown` turns a step into a word or the line into the text).
    pub fn text_action(&mut self, action: u32, extent: f32, modifiers: Modifiers) {
        let Some(index) = self.active.filter(|_| self.text_mode()) else {
            return;
        };
        if extent == 0.0 {
            return;
        }
        let ctrl = modifiers.ctrl;
        let direction = match (action, ctrl) {
            (0x4000_0013, false) => Direction::Left,
            (0x4000_0013, true) => Direction::WordLeft,
            (0x4000_0014, false) => Direction::Right,
            (0x4000_0014, true) => Direction::WordRight,
            (0x4000_0015, _) => Direction::Up,
            (0x4000_0016, _) => Direction::Down,
            (0x4000_001B, false) => Direction::LineHome,
            (0x4000_001B, true) => Direction::Home,
            (0x4000_001C, false) => Direction::LineEnd,
            (0x4000_001C, true) => Direction::End,
            _ => return,
        };
        self.move_cursor(index, direction, modifiers.shift);
    }

    /// `UIElement_Text::CharacterHandler` on the active element: `ch` a typed character, or `key`
    /// the virtual key of a key that typed none.
    pub fn character(&mut self, ch: u16, key: u32, modifiers: Modifiers) {
        let Some(index) = self.text_target() else {
            return;
        };
        let (shift, ctrl) = (modifiers.shift, modifiers.ctrl);
        let direction = match key {
            0x23 => Some(if ctrl {
                Direction::End
            } else {
                Direction::LineEnd
            }),
            0x24 => Some(if ctrl {
                Direction::Home
            } else {
                Direction::LineHome
            }),
            0x25 => Some(if ctrl {
                Direction::WordLeft
            } else {
                Direction::Left
            }),
            0x21 => Some(Direction::PageUp),
            0x22 => Some(Direction::PageDown),
            0x26 => Some(Direction::Up),
            0x27 => Some(if ctrl {
                Direction::WordRight
            } else {
                Direction::Right
            }),
            0x28 => Some(Direction::Down),
            _ => None,
        };
        if let Some(direction) = direction {
            self.move_cursor(index, direction, shift);
        }
        let (editable, filter) = {
            let text = self.el(index).text.as_ref().expect("text element");
            (text.editable, text.filter)
        };
        if !editable {
            // A read-only text takes the clipboard keys only (`text_clip.rs`): Ctrl+A and Ctrl+C
            // work, and cut and paste do nothing there.
            if key == 0 {
                let ch = control_code(ch, modifiers.ctrl);
                if matches!(ch, 1 | 3 | 0x16 | 0x18) {
                    self.clipboard_key(index, ch);
                }
            }
            return;
        }
        if key != 0 {
            if editable && key == 0x2E {
                self.delete_forward(index);
            }
            self.broadcast(index, message("key_input"), key as i32);
            return;
        }
        let ch = control_code(ch, modifiers.ctrl);
        match ch {
            // Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X (`text_clip.rs`).
            1 | 3 | 0x16 | 0x18 => self.clipboard_key(index, ch),
            _ if editable => self.edit_character(index, ch, filter),
            _ => {}
        }
        self.broadcast(index, message("character_input"), i32::from(ch));
    }

    /// The Delete key: the selection, or the character at the cursor (`DeleteChar`).
    fn delete_forward(&mut self, index: usize) {
        if self.selection(index).is_some() {
            self.delete_selection(index);
        } else {
            let cursor = self.el(index).text.as_ref().map_or(0, |t| t.cursor);
            self.delete_at(index, cursor);
        }
    }

    /// The editable branch of `CharacterHandler` for a typed character `ch`.
    fn edit_character(&mut self, index: usize, ch: u16, filter: Option<fn(u16) -> bool>) {
        let (cursor, one_line) = {
            let text = self.el(index).text.as_ref().expect("text element");
            (text.cursor, text.one_line)
        };
        match ch {
            8 => {
                if self.selection(index).is_some() {
                    self.delete_selection(index);
                } else if cursor > 0 {
                    self.set_cursor(index, cursor as isize - 1, false);
                    self.delete_at(index, cursor - 1);
                }
            }
            9 | 0x1B => {}
            10 | 13 => {
                if !one_line {
                    self.insert(index, &[10]);
                }
            }
            c if c < 0x20 || c == 0x7F => {}
            c if filter.is_none_or(|f| f(c)) => self.insert(index, &[c]),
            _ => {}
        }
        self.scroll_to_cursor(index);
    }

    /// `m_filter` of a text element: typed printable characters `filter` refuses are dropped.
    pub fn set_text_filter(&mut self, index: usize, filter: Option<fn(u16) -> bool>) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.filter = filter;
        }
    }

    /// `UIElement_Text::AddText_Internal` at the cursor: an editable element first replaces its
    /// selection, anything else drops it.
    pub(crate) fn insert(&mut self, index: usize, units: &[u16]) {
        if self.el(index).text.as_ref().is_some_and(|t| t.editable) {
            self.delete_selection(index);
        } else {
            self.set_selecting(index, false);
        }
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.insert_at_cursor(units);
            self.changed = true;
        }
    }

    /// `UIElement_Text::DeleteChar` at `position`.
    fn delete_at(&mut self, index: usize, position: usize) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            if position < text.glyphs.len() {
                text.delete_range(position, position + 1);
                self.changed = true;
            }
        }
    }

    /// `UIElement_Text::MouseDown`: a left press on an editable (or selectable) text element
    /// places the cursor at the pointer (extending a selection while shift is held), starts a
    /// drag selection (`text_clip.rs`) and activates an editable one.
    pub(crate) fn text_mouse_down(&mut self, index: usize, action: u32) {
        if matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT) {
            self.text_wheel(index, action);
            return;
        }
        if action != LEFT_BUTTON {
            return;
        }
        let Some(text) = self.el(index).text.as_ref() else {
            return;
        };
        let editable = text.editable;
        if !(editable || self.selectable(index)) {
            return;
        }
        // The press moves the copy focus to this text (or away from a read-only one).
        self.copy_focus = (!editable).then_some(index);
        let (local_x, local_y) = self.text_local_pointer(index);
        let position = self
            .el(index)
            .text
            .as_ref()
            .map_or(0, |t| t.position_at(local_x, local_y));
        // The pointer's shift state (`set_shift`, shared with the drag).
        let shift = self.drag.shift;
        self.set_cursor(index, position as isize, shift);
        self.text_drag = Some(index);
        if editable {
            self.activate(index);
        }
    }
}

/// The control code a typed Ctrl+letter stands for (Ctrl+A, C, V, X give 1, 3, 0x16, 0x18 where
/// the platform types the code itself, as `WM_CHAR` does); other characters are unchanged.
fn control_code(ch: u16, ctrl: bool) -> u16 {
    if !ctrl || ch > 0x7F {
        return ch;
    }
    match (ch as u8).to_ascii_lowercase() {
        b'a' => 1,
        b'c' => 3,
        b'v' => 0x16,
        b'x' => 0x18,
        _ => ch,
    }
}
