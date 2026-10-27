//! The clipboard keys and the pointer drag selection of text elements. Ctrl+A selects all,
//! Ctrl+C copies, Ctrl+X cuts and Ctrl+V pastes through the tree's clipboard
//! (`UiTree::set_text_clipboard`). A left press starts a selection and a drag moves its moving
//! end until the release (`UIElement_Text::MouseDown` sets `m_mouseSelecting`,
//! `Global_MouseMove` moves the end with the pointer, `MouseUp` clears it).
//!
//! Retail's `UIElement_Text::Copy`, `Cut` and `Paste` (admin 7148) go through
//! `Device::SendStringToClipboard` and `Device::GetStringFromClipboard`; the Legions 7160 text
//! code has no copy, cut or paste of its own. Here a password field never copies or cuts,
//! and a paste keeps only what the field takes: no control characters, each printable one through
//! the field's `filter`, and no more than its `max_characters`.

use crate::element::UiTree;

/// `LEFT_MOUSE_BUTTON` (`0x40000004`).
const LEFT_BUTTON: u32 = 0x4000_0004;

/// The units a paste types into a field: CR is dropped and a line break is kept only on a
/// multi-line field (on a one-line one it becomes a space); other control characters are
/// dropped, and each remaining character must pass the field's `filter`.
pub(crate) fn paste_units(text: &str, one_line: bool, filter: Option<fn(u16) -> bool>) -> Vec<u16> {
    let accepted = |ch: u16| filter.is_none_or(|f| f(ch));
    let mut units = Vec::with_capacity(text.len());
    for ch in text.encode_utf16() {
        match ch {
            13 => {}
            10 if one_line => {
                if accepted(0x20) {
                    units.push(0x20);
                }
            }
            10 => units.push(10),
            c if c < 0x20 || c == 0x7F => {}
            c => {
                if accepted(c) {
                    units.push(c);
                }
            }
        }
    }
    units
}

impl UiTree {
    /// The clipboard key `ch` (a control code from `character`) on text element `index`.
    pub(crate) fn clipboard_key(&mut self, index: usize, ch: u16) {
        match ch {
            1 => self.select_all(index),
            3 => {
                self.copy_selection(index);
            }
            0x16 => self.paste_clipboard(index),
            0x18 => self.cut_selection(index),
            _ => {}
        }
    }

    /// `UIElement_Text::Copy`: the selected text goes to the clipboard. Nothing is copied from a
    /// password field or when nothing is selected; `false` then, and when the clipboard refuses.
    pub(crate) fn copy_selection(&mut self, index: usize) -> bool {
        let Some(text) = self.copyable_selection(index) else {
            return false;
        };
        self.text_clipboard.set_text(&text)
    }

    /// The selected text of element `index` as a string, when the element may give it out.
    fn copyable_selection(&self, index: usize) -> Option<String> {
        let text = self.get(index)?.text.as_ref()?;
        if text.password {
            return None;
        }
        let units = self.selected_text(index);
        (!units.is_empty()).then(|| String::from_utf16_lossy(&units))
    }

    /// `UIElement_Text::Cut`: the selection is copied and then deleted, on an editable element
    /// only. Nothing changes when the copy is refused (a password field, or the clipboard).
    pub(crate) fn cut_selection(&mut self, index: usize) {
        let editable = self
            .get(index)
            .and_then(|e| e.text.as_ref())
            .is_some_and(|t| t.editable);
        if editable && self.copy_selection(index) {
            self.delete_selection(index);
        }
    }

    /// `UIElement_Text::Paste`: the clipboard's text, filtered for the field, replaces the
    /// selection at the cursor. Passwords paste like any field.
    pub(crate) fn paste_clipboard(&mut self, index: usize) {
        let Some(text) = self.get(index).and_then(|e| e.text.as_ref()) else {
            return;
        };
        if !text.editable {
            return;
        }
        let (one_line, filter) = (text.one_line, text.filter);
        let Some(clip) = self.text_clipboard.text() else {
            return;
        };
        let units = paste_units(clip.trim_end_matches(['\r', '\n']), one_line, filter);
        if !units.is_empty() {
            self.insert(index, &units);
        }
    }

    /// The pointer relative to the top left of element `index` (`localX`, `localY`).
    pub(crate) fn text_local_pointer(&self, index: usize) -> (i32, i32) {
        let origin = self.screen_box(index);
        (self.pointer.0 - origin.x0, self.pointer.1 - origin.y0)
    }

    /// `UIElement_Text::Global_MouseMove` while a drag selection runs: the moving end follows
    /// the pointer. The drag ends when the button is no longer held.
    pub(crate) fn text_drag_move(&mut self) {
        let Some(index) = self.text_drag else {
            return;
        };
        let position = self
            .get(index)
            .and_then(|e| e.text.as_ref())
            .map(|_| self.text_local_pointer(index));
        match position {
            Some((x, y)) if self.drag.left_down => {
                let at = self
                    .el(index)
                    .text
                    .as_ref()
                    .map_or(0, |t| t.position_at(x, y));
                self.set_cursor(index, at as isize, true);
                self.scroll_to_cursor(index);
            }
            _ => self.text_drag = None,
        }
    }

    /// `UIElement_Text::MouseUp`: releasing the button ends the element's drag selection.
    pub(crate) fn text_mouse_up(&mut self, index: usize, action: u32) {
        if action == LEFT_BUTTON && self.text_drag == Some(index) {
            self.text_drag = None;
        }
    }
}
