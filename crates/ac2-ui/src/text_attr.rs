//! The attribute side of `UIElement_Text`: `SetAttribute_Int` (margins, `max_chars`),
//! `SetAttribute_Bool` (`font_reset`, `one_line`, `center_vertically`, `center`,
//! `right_justify`), `SetAttribute_StringID` (`entry`: `SetString`) and `SetAttribute_DataID`
//! (`font`: `SetFontNum(0)` then the text again in the new font).

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::text::Justification;
use crate::token::{
    tokenize, TEXT_BOTTOM_MARGIN, TEXT_CENTER, TEXT_CENTER_VERTICALLY, TEXT_ENTRY, TEXT_FONT,
    TEXT_FONT_RESET, TEXT_LEFT_MARGIN, TEXT_MAX_CHARS, TEXT_ONE_LINE, TEXT_RIGHT_JUSTIFY,
    TEXT_RIGHT_MARGIN, TEXT_TOP_MARGIN,
};

impl UiTree {
    pub(crate) fn float_attribute(&self, index: usize, id: u32) -> Option<f32> {
        match self.el(index).attributes.get(&id) {
            Some(AttributeValue::Float(value)) => Some(*value),
            _ => None,
        }
    }

    /// `UIElement_Text::SetFontNumAndColor(font_num, color_num)` (`SetFontNum` is the case where
    /// both are the same): font `font{n}`, colour `font_color{m}_red/green/blue` (white where
    /// unset), outline `font{n}_outline`. Without a font attribute that loads, the font and
    /// colour stay as they are.
    pub(crate) fn set_font_num_and_color(&mut self, index: usize, font_num: u32, color_num: u32) {
        let number = |n: u32| if n == 0 { String::new() } else { n.to_string() };
        let font_name = format!("font{}", number(font_num));
        let color_name = format!("font_color{}", number(color_num));
        let font_did = match self.el(index).attributes.get(&tokenize(&font_name)) {
            Some(AttributeValue::DataId(did)) => *did,
            _ => 0,
        };
        let channel = |tree: &Self, suffix: &str| {
            tree.float_attribute(index, tokenize(&format!("{color_name}_{suffix}")))
                .unwrap_or(1.0)
        };
        let color = [
            channel(self, "red"),
            channel(self, "green"),
            channel(self, "blue"),
            1.0,
        ];
        let outline = matches!(
            self.el(index)
                .attributes
                .get(&tokenize(&format!("{font_name}_outline"))),
            Some(AttributeValue::Bool(true))
        );
        if font_did == 0 {
            return;
        }
        let Some(font) = self.text_data.as_ref().and_then(|data| data.font(font_did)) else {
            return;
        };
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.font = Some(font);
            text.color = color;
            text.outline = outline;
        }
    }

    /// `UIElement_Text::SetString`: string `id` of the layout's string table, in font 0.
    pub fn set_string(&mut self, index: usize, id: u32) {
        let table = self.el(index).layout.string_table;
        let string = self
            .text_data
            .as_ref()
            .and_then(|data| data.string(table, id));
        let Some(text) = self.el_mut(index).text.as_mut() else {
            return;
        };
        text.clear();
        // The string download: font and colour, then the text.
        if let Some(string) = string {
            self.set_font_num_and_color(index, 0, 0);
            self.determine_font(index, &string);
            if let Some(text) = self.el_mut(index).text.as_mut() {
                text.add_text(&string);
            }
        }
    }

    /// `UIElement_Text::SetText(text)` and `SetStringInfo(info)`: the element's text replaced, in
    /// font 0 and colour 0
    /// (`ClearAllText`, then `AddStringInfowFont(si, 0)`, whose string download applies
    /// `SetFontNumAndColor(0, 0)` before the text). The vertical bar follows the new paper
    /// (retail updates it when the glyphs are recalculated, `RecalculateGlyphList` ->
    /// `UpdateScrollbars`).
    pub fn set_text(&mut self, index: usize, text: &str) {
        self.set_text_with_font(index, text, 0, 0);
    }

    /// `UIElement_Text::SetText(text, fontNumber, colorNumber)` and `SetStringInfo(info,
    /// fontNumber, colorNumber)`: the element's text replaced, in that font and colour.
    pub fn set_text_with_font(&mut self, index: usize, text: &str, font_num: u32, color_num: u32) {
        if self.el(index).text.is_none() {
            return;
        }
        self.set_font_num_and_color(index, font_num, color_num);
        let units: Vec<u16> = text.encode_utf16().collect();
        self.determine_font(index, &units);
        if let Some(state) = self.el_mut(index).text.as_mut() {
            state.clear();
            state.add_text(&units);
            self.changed = true;
            self.text_update_scrollbar(index);
        }
    }

    /// `UIElement_Text::InqSizewMargins`: the size `text` would take in the element's font,
    /// laid out no wider than `max_width` (else the display) less the margins, plus the margins.
    pub fn text_size_with_margins(&self, index: usize, text: &str) -> Option<(i32, i32)> {
        let el = self.el(index);
        let state = el.text.as_ref()?;
        let font = state.font.clone()?;
        let glyphs: Vec<_> = text
            .encode_utf16()
            .map(|ch| crate::text::Glyph::new(ch, state.color, Some(font.clone()), state.outline))
            .collect();
        let max_width = match el.attributes.get(&crate::token::MAX_WIDTH) {
            Some(AttributeValue::Int(width)) => *width,
            _ => self.display.0,
        };
        let [up, down, left, right] = state.margins;
        let (_, width, height) =
            crate::text::recalculate_lines(&glyphs, max_width - left - right, state.one_line);
        Some((width + right + left, height + down + up))
    }

    /// `UIElement_Text::InqSizewMargins` with the text on one line: the width `text` takes in
    /// the element's font, margins included (`m_paperWidth` of an unwrapped line).
    pub fn text_line_width(&self, index: usize, text: &str) -> Option<i32> {
        let state = self.el(index).text.as_ref()?;
        let font = state.font.clone()?;
        let glyphs: Vec<_> = text
            .encode_utf16()
            .map(|ch| crate::text::Glyph::new(ch, state.color, Some(font.clone()), state.outline))
            .collect();
        let [_, _, left, right] = state.margins;
        let (_, width, _) = crate::text::recalculate_lines(&glyphs, i32::MAX / 2, true);
        Some(width + left + right)
    }

    /// `GetText` / `SetText`: the text again in the current font.
    fn readd_text(&mut self, index: usize) {
        let Some(units) = self.el(index).text.as_ref().map(|t| t.text()) else {
            return;
        };
        self.determine_font(index, &units);
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.clear();
            text.add_text(&units);
        }
    }

    /// The `UIElement_Text` overrides of the attribute setters, after the base setter stored
    /// `value` (`changed` when it differed).
    pub(crate) fn text_attribute(
        &mut self,
        index: usize,
        id: u32,
        value: &AttributeValue,
        changed: bool,
    ) {
        if self.el(index).text.is_none() {
            return;
        }
        if id == TEXT_FONT_RESET {
            self.set_font_num_and_color(index, 0, 0);
            self.readd_text(index);
        }
        if !changed {
            return;
        }
        let (width, height) = {
            let rect = self.el(index).rect;
            (rect.width(), rect.height())
        };
        match (id, value) {
            (TEXT_ENTRY, AttributeValue::StringId(string)) => self.set_string(index, *string),
            (TEXT_FONT, AttributeValue::DataId(_)) => {
                self.set_font_num_and_color(index, 0, 0);
                self.readd_text(index);
            }
            (_, AttributeValue::Int(v)) => {
                let Some(text) = self.el_mut(index).text.as_mut() else {
                    return;
                };
                let slot = match id {
                    TEXT_TOP_MARGIN => 0,
                    TEXT_BOTTOM_MARGIN => 1,
                    TEXT_LEFT_MARGIN => 2,
                    TEXT_RIGHT_MARGIN => 3,
                    TEXT_MAX_CHARS => {
                        text.max_characters = (*v).max(0) as usize;
                        return;
                    }
                    _ => return,
                };
                if *v != -1 {
                    text.requested[slot] = *v;
                }
                text.determine_margins(width, height);
                text.dirty = true;
            }
            (_, AttributeValue::Bool(v)) => {
                let Some(text) = self.el_mut(index).text.as_mut() else {
                    return;
                };
                match id {
                    // `SetAtrb_Editable`.
                    id if id == tokenize("editable") => text.editable = *v,
                    // A password field (a modern flag, not a retail attribute): its text never
                    // goes to the clipboard (`text_clip.rs`).
                    id if id == tokenize("password") => text.password = *v,
                    TEXT_ONE_LINE => text.one_line = *v,
                    TEXT_CENTER_VERTICALLY => text.center_vertically = *v,
                    TEXT_CENTER => text.justification = Justification::Center,
                    TEXT_RIGHT_JUSTIFY => text.justification = Justification::Right,
                    _ => return,
                }
                text.dirty = true;
            }
            _ => {}
        }
    }
}

/// The text calls the chat makes on its log and entry line.
impl UiTree {
    /// `UIElement_Text::ClearAllText`.
    pub(crate) fn clear_text(&mut self, index: usize) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.clear();
            self.changed = true;
        }
    }

    /// `UIElement_Text::AddStringInfowFont` once its string arrived (`StringDownload`):
    /// `SetFontNumAndColor(font_num, color_num)`, then the text at the end (`AddStringInfo` is font
    /// 0, colour 0). The cursor goes to the end first, as `MoveCursorToEnd` does, without its
    /// selection handling or `ScrollHorizontalView`.
    pub(crate) fn add_text_with_font(
        &mut self,
        index: usize,
        units: &[u16],
        font_num: u32,
        color_num: u32,
    ) {
        if self.el(index).text.is_none() {
            return;
        }
        self.set_font_num_and_color(index, font_num, color_num);
        self.determine_font(index, units);
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.cursor = text.glyphs.len();
            text.insert_at_cursor(units);
            self.changed = true;
        }
    }

    /// `UIElement_Text::RecalculateGlyphList` at the element's size: the laid-out text's height
    /// (`m_paperHeight`).
    pub(crate) fn text_paper_height(&mut self, index: usize) -> Option<i32> {
        let rect = self.el(index).rect;
        let text = self.el_mut(index).text.as_mut()?;
        text.dirty = true;
        text.recalculate(rect.width(), rect.height());
        Some(text.paper.1)
    }

    /// `UIElement_Text::ScrollToEnd` (the view is kept in bounds when the text is laid out; retail
    /// gets there through `ScrollToPosition`, which this does not model for other positions).
    pub(crate) fn scroll_to_end(&mut self, index: usize) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.view.1 = i32::MAX / 2;
            text.dirty = true;
            self.changed = true;
        }
    }

    /// `UIElement_Text::ScrollToHome`.
    pub(crate) fn scroll_to_home(&mut self, index: usize) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.view = (0, 0);
            text.dirty = true;
            self.changed = true;
        }
    }

    /// `UIElement_Text::VerticalPageDecrement` / `VerticalPageIncrement`: the view one element
    /// height up or down (`DetermineAndSetNewPage`).
    pub(crate) fn page_text(&mut self, index: usize, down: bool) {
        self.text_new_page(index, true, down);
    }

    /// `UIElement_Text::DetermineAndSetNewPage`: the view moves one element height
    /// (`vertical`) or width forward (`increment`) or back, is kept in bounds, and the bars
    /// follow.
    pub(crate) fn text_new_page(&mut self, index: usize, vertical: bool, increment: bool) {
        let rect = self.el(index).rect;
        let extent = if vertical {
            rect.height()
        } else {
            rect.width()
        };
        let delta = if increment { extent } else { -extent };
        let Some(text) = self.el_mut(index).text.as_mut() else {
            return;
        };
        if vertical {
            text.view.1 += delta;
        } else {
            text.view.0 += delta;
        }
        text.dirty = true;
        self.changed = true;
        // The layout keeps the view in bounds (`KeepInBounds`).
        self.text_paper_height(index);
        self.text_update_scrollbar(index);
    }
}

impl UiTree {
    /// `UIElement_Text::DetermineFont`, before text is added: when the current font lacks one of
    /// `text`'s characters (`Font::ContainsString`), the first fallback font (Korean, then
    /// Japanese) that has them all becomes the element's font; otherwise it stays.
    pub(crate) fn determine_font(&mut self, index: usize, text: &[u16]) {
        let Some(current) = self.el(index).text.as_ref().and_then(|t| t.font.clone()) else {
            return;
        };
        if crate::text::font_contains(&current, text) {
            return;
        }
        let Some(data) = self.text_data.as_ref() else {
            return;
        };
        let found = data
            .fallback_fonts()
            .into_iter()
            .find(|font| crate::text::font_contains(font, text));
        if let (Some(font), Some(state)) = (found, self.el_mut(index).text.as_mut()) {
            state.font = Some(font);
        }
    }
}

/// The text calls the book makes on its pages (`gmBookUI::ParseText`).
impl UiTree {
    /// `UIElement_Text::AddText` at the end, in the current font (no font change).
    pub(crate) fn append_text(&mut self, index: usize, text: &str) {
        let units: Vec<u16> = text.encode_utf16().collect();
        self.determine_font(index, &units);
        if let Some(state) = self.el_mut(index).text.as_mut() {
            state.add_text(&units);
            self.changed = true;
        }
    }

    /// Text at the end in the current font, in `rgb` (the client's own colour, for windows it
    /// makes) or, for `None`, the element's colour; the element's colour is kept for what follows.
    pub(crate) fn append_text_rgb(&mut self, index: usize, text: &str, rgb: Option<[f32; 3]>) {
        let units: Vec<u16> = text.encode_utf16().collect();
        self.determine_font(index, &units);
        if let Some(state) = self.el_mut(index).text.as_mut() {
            let own = state.color;
            if let Some([r, g, b]) = rgb {
                state.color = [r, g, b, own[3]];
            }
            state.add_text(&units);
            state.color = own;
            self.changed = true;
        }
    }

    /// `UIElement_Text::RecalculateGlyphList` then `GetTotalLines`: the display lines.
    pub(crate) fn text_line_count(&mut self, index: usize) -> usize {
        self.text_paper_height(index);
        self.el(index)
            .text
            .as_ref()
            .map_or(0, |text| text.lines.len())
    }

    /// `UIElement_Text::SetViewY`: the text's view moved to `y` pixels.
    pub(crate) fn set_text_view_y(&mut self, index: usize, y: i32) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            text.view.1 = y;
            text.dirty = true;
            self.changed = true;
        }
    }
}
