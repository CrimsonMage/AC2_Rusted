//! Typing into text elements: activation (`UIElement::Activate` / `Deactivate`,
//! `UIElementManager::ActivationAlert`: one active element; an editable text element turns on the
//! input manager's text mode while active), the text element's handlers (`UIElement_Text::
//! CharacterHandler` for typed characters and virtual keys, `ActionHandler` for the editing
//! actions), the cursor (`MoveCursor`, `SetCursorPosition`) and clicking to place it
//! (`UIElement_Text::MouseDown`). The element broadcasts `activated` / `deactivated`,
//! `character_input` (with the character) and `key_input`.
//!
//! Not modelled yet: selection, copy / cut / paste, the IME, page up / down, the cursor's
//! flashing and scrolling the view to the cursor.

use crate::element::UiTree;
use crate::layout::AttributeValue;
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
}

fn message(name: &str) -> u32 {
    tokenize(name)
}

impl UiTree {
    /// The active element (`m_activeElement`).
    pub fn active(&self) -> Option<usize> {
        self.active
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
        if self.el(index).text.as_ref().is_some_and(|t| t.editable) {
            self.broadcast(index, message("deactivated"), 0);
            self.changed = true;
        }
    }

    /// `UIElement_Text::SetCursorPosition`, clamped to the text.
    fn set_cursor(&mut self, index: usize, position: isize) -> bool {
        let Some(text) = self.el_mut(index).text.as_mut() else {
            return false;
        };
        let position = position.clamp(0, text.glyphs.len() as isize) as usize;
        let moved = text.cursor != position;
        text.cursor = position;
        self.changed |= moved;
        moved
    }

    /// `UIElement_Text::MoveCursor`.
    pub(crate) fn move_cursor(&mut self, index: usize, direction: Direction) -> bool {
        let Some(text) = self.el(index).text.as_ref() else {
            return false;
        };
        let cursor = text.cursor as isize;
        let units: Vec<u16> = text.glyphs.iter().map(|g| g.ch).collect();
        let white = |c: u16| matches!(c, 9 | 13 | 32);
        let line = text
            .lines
            .iter()
            .rposition(|l| l.index as isize <= cursor)
            .unwrap_or(0);
        let line_start = |i: usize| text.lines.get(i).map_or(0, |l| l.index) as isize;
        let line_end = |i: usize| {
            text.lines
                .get(i + 1)
                .map_or(units.len() as isize, |l| l.index as isize)
        };
        let target = match direction {
            Direction::Home => 0,
            Direction::End => units.len() as isize,
            Direction::LineHome => line_start(line),
            Direction::LineEnd => line_end(line),
            Direction::Left => cursor - 1,
            Direction::Right => cursor + 1,
            Direction::WordLeft => {
                let mut i = cursor;
                while i > 0 && white(units[i as usize - 1]) {
                    i -= 1;
                }
                while i > 0 && !white(units[i as usize - 1]) {
                    i -= 1;
                }
                i
            }
            Direction::WordRight => {
                let mut i = cursor;
                while (i as usize) < units.len() && !white(units[i as usize]) {
                    i += 1;
                }
                while (i as usize) < units.len() && white(units[i as usize]) {
                    i += 1;
                }
                i
            }
            Direction::Up | Direction::Down => {
                let to = if direction == Direction::Up {
                    line.checked_sub(1)
                } else {
                    (line + 1 < text.lines.len()).then_some(line + 1)
                };
                let Some(to) = to else {
                    return false;
                };
                let column = cursor - line_start(line);
                (line_start(to) + column).min(line_end(to))
            }
        };
        self.set_cursor(index, target)
    }

    /// `UIElement_Text::ActionHandler` on the active element: the editing actions while held
    /// (`ActionInterpreter::CtrlKeyDown` turns a step into a word or the line into the text).
    pub fn text_action(&mut self, action: u32, extent: f32, ctrl: bool) {
        let Some(index) = self.active.filter(|_| self.text_mode()) else {
            return;
        };
        if extent == 0.0 {
            return;
        }
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
        self.move_cursor(index, direction);
    }

    /// `UIElement_Text::CharacterHandler` on the active element: `ch` a typed character, or `key`
    /// the virtual key of a key that typed none.
    pub fn character(&mut self, ch: u16, key: u32, ctrl: bool) {
        let Some(index) = self.active else {
            return;
        };
        if self.el(index).text.is_none() {
            return;
        }
        let direction = match key {
            0x21 | 0x22 => None,
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
            self.move_cursor(index, direction);
        }
        let (editable, one_line, cursor, filter) = {
            let text = self.el(index).text.as_ref().expect("text element");
            (text.editable, text.one_line, text.cursor, text.filter)
        };
        if !editable || key != 0 {
            if editable && key == 0x2E {
                self.delete_at(index, cursor);
            }
            if key != 0 {
                self.broadcast(index, message("key_input"), key as i32);
                return;
            }
        } else {
            match ch {
                8 if cursor > 0 => {
                    self.set_cursor(index, cursor as isize - 1);
                    self.delete_at(index, cursor - 1);
                }
                10 | 13 if !one_line => self.insert(index, &[10]),
                c if c >= 0x20 && c != 0x7F && filter.is_none_or(|f| f(c)) => {
                    self.insert(index, &[c])
                }
                _ => {}
            }
        }
        self.broadcast(index, message("character_input"), i32::from(ch));
    }

    /// `m_filter` of a text element: typed printable characters `filter` refuses are dropped.
    pub fn set_text_filter(&mut self, index: usize, filter: Option<fn(u16) -> bool>) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.filter = filter;
        }
    }

    /// `UIElement_Text::AddText_Internal` at the cursor.
    fn insert(&mut self, index: usize, units: &[u16]) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.insert_at_cursor(units);
            self.changed = true;
        }
    }

    /// `UIElement_Text::DeleteChar` at `position`.
    fn delete_at(&mut self, index: usize, position: usize) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            if position < text.glyphs.len() {
                text.glyphs.remove(position);
                text.dirty = true;
                self.changed = true;
            }
        }
    }

    /// `UIElement_Text::MouseDown`: a left press on an editable (or selectable) text element
    /// places the cursor at the pointer and activates an editable one.
    pub(crate) fn text_mouse_down(&mut self, index: usize, action: u32) {
        if action != 0x4000_0004 {
            return;
        }
        let Some(text) = self.el(index).text.as_ref() else {
            return;
        };
        let selectable = matches!(
            self.el(index).attributes.get(&tokenize("selectable")),
            Some(AttributeValue::Bool(true))
        );
        if !(text.editable || selectable) {
            return;
        }
        let editable = text.editable;
        let local_x = self.pointer.0 - self.screen_box(index).x0;
        let position = self
            .el(index)
            .text
            .as_ref()
            .map_or(0, |t| t.position_at(local_x));
        self.set_cursor(index, position as isize);
        if editable {
            self.activate(index);
        }
    }
}
