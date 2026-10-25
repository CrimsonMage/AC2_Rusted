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
    fn float_attribute(&self, index: usize, id: u32) -> Option<f32> {
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
            if let Some(text) = self.el_mut(index).text.as_mut() {
                text.add_text(&string);
            }
        }
    }

    /// `GetText` / `SetText`: the text again in the current font.
    fn readd_text(&mut self, index: usize) {
        if let Some(text) = self.el_mut(index).text.as_mut() {
            let units = text.text();
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
