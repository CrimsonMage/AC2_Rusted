//! `UIElement_Text`'s vertical scroll bar.
//!
//! A text names its bar in `vertical_scrollbar` (`GetScrollbar`). The bar's position is the
//! view's share of the paper: `UIElement_Text::UpdateScrollbar` sets `y_position` to
//! `GetPercentageV` (the view over the scrollable range), and `disabled` when the paper fits the
//! element. A bar's new position moves the view (`HandleScrollBarMessage`): `y_position` times the
//! scrollable range, which is the paper less the element's own inclusive height,
//! `m_paperHeight - (y1 - y0 + 1)`. The admin 7148 machine code of `GetPercentageV` (0x005EDC40)
//! and `HandleScrollBarMessage` (0x005F0790) reads `m_box.y1 - m_box.y0 + 1` in both, and the
//! Legions functions (0x005AE830, 0x005B12C0) match them byte for byte. The Legions C export drops
//! the x87 multiply in `HandleScrollBarMessage`; the `fmuls` in the admin code is that multiply.
//! A wheel turn over the text goes to its bar (`UIElement_Text::MouseDown` calls the bar's
//! `MouseDown`), which broadcasts a line message the text takes as one line up or down
//! (`DetermineAndSetNewLine`): the line at the view's top.
//!
//! The bar's widget is sized to the element's share of the paper, and a page message moves the
//! view by the element's height (`DetermineAndSetNewPage`).
//!
//! Not modelled yet: the horizontal bar.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::{UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::token::{tokenize, TEXT};

const VERTICAL_SCROLLBAR: &str = "vertical_scrollbar";

impl UiTree {
    /// The vertical scroll bar a text names (`GetScrollbar(Token_Text_vertical_scrollbar)`).
    pub(crate) fn text_scrollbar(&self, text: usize) -> Option<usize> {
        self.named_scrollbar(text, VERTICAL_SCROLLBAR)
    }

    /// The text elements that name `scrollbar` as their vertical scroll bar (`GetScrollbar`
    /// resolves it across the UI, see `named_scrollbar`).
    pub(crate) fn texts_bound_to(&self, scrollbar: usize) -> Vec<usize> {
        let id = self.el(scrollbar).id();
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let el = slot.as_ref()?;
                let names_it = el.desc.kind == TEXT
                    && matches!(
                        el.attributes.get(&tokenize(VERTICAL_SCROLLBAR)),
                        Some(AttributeValue::Id(v)) if *v == id
                    );
                (names_it && self.text_scrollbar(index) == Some(scrollbar)).then_some(index)
            })
            .collect()
    }

    /// `UIElement_Text::HandleScrollBarMessage` for `new_position`: the text's view follows the
    /// bar's position over the scrollable range.
    pub(crate) fn texts_follow_scrollbar(&mut self, scrollbar: usize) {
        let position = f64::from(self.scrollbar_position(scrollbar));
        for text in self.texts_bound_to(scrollbar) {
            let range = self.text_scroll_range(text);
            let y = (position * f64::from(range)) as i32;
            self.text_set_view_y(text, y);
        }
    }

    /// The paper's height the view can scroll over: `m_paperHeight - (y1 - y0 + 1)`, the paper
    /// less the element's inclusive height (zero when the paper fits).
    fn text_scroll_range(&mut self, text: usize) -> i32 {
        let height = self.el(text).rect.height();
        let paper = self.text_paper_height(text).unwrap_or(0);
        (paper - height).max(0)
    }

    /// Sets the view's y (kept in bounds), then the bar's position to match
    /// (`UIElement_Text::UpdateScrollbar`).
    fn text_set_view_y(&mut self, text: usize, y: i32) {
        self.set_text_view_y(text, y);
        self.text_paper_height(text);
        self.text_update_scrollbar(text);
    }

    /// `UIElement_Text::UpdateScrollbar` (admin 0x005EE550) for the vertical bar: with the bar's
    /// layout held (`allow_updating`) and the bar enabled, its widget size is the element's share
    /// of the paper (at most one) and its position the view's share of the scrollable range; the
    /// layout then runs, and the bar is disabled when the paper fits the element (so a
    /// `hide_when_disabled` bar hides).
    pub(crate) fn text_update_scrollbar(&mut self, text: usize) {
        let Some(bar) = self.text_scrollbar(text) else {
            return;
        };
        let range = self.text_scroll_range(text);
        let height = self.el(text).rect.height();
        let paper = self.text_paper_height(text).unwrap_or(0);
        let view = self.el(text).text.as_ref().map_or(0, |t| t.view.1);
        let share = if range <= 0 {
            0.0
        } else {
            (f64::from(view) / f64::from(range)).clamp(0.0, 1.0)
        };
        let size = if paper > 0 {
            (height as f32 / paper as f32).min(1.0)
        } else {
            1.0
        };
        let attrs = &mut self.el_mut(bar).attributes;
        attrs.insert(tokenize("allow_updating"), AttributeValue::Bool(false));
        attrs.insert(tokenize("disabled"), AttributeValue::Bool(false));
        attrs.insert(tokenize("y_widget_size"), AttributeValue::Float(size));
        self.scrollbar_set_position(bar, share as f32);
        self.el_mut(bar)
            .attributes
            .insert(tokenize("allow_updating"), AttributeValue::Bool(true));
        self.scrollbar_update_layout(bar);
        if paper <= height {
            self.el_mut(bar)
                .attributes
                .insert(tokenize("disabled"), AttributeValue::Bool(true));
            self.scrollbar_update_layout(bar);
        }
    }

    /// `UIElement_Text::DetermineAndSetNewPage` (vertical, admin 0x005F04F0): the view moves the
    /// element's own height, down (`forward`) or up.
    pub(crate) fn text_page_step(&mut self, text: usize, forward: bool) {
        let height = self.el(text).rect.height();
        let Some(view) = self.el(text).text.as_ref().map(|t| t.view.1) else {
            return;
        };
        let step = if forward { height } else { -height };
        self.text_set_view_y(text, view + step);
    }

    /// `UIElement_Text::DetermineAndSetNewLine` (vertical): the view moves one line, the height
    /// of the line at its top, down (`steps` 1) or up (-1).
    pub(crate) fn text_line_step(&mut self, text: usize, steps: i32) {
        self.text_paper_height(text);
        let Some(view) = self.el(text).text.as_ref().map(|t| t.view.1) else {
            return;
        };
        let line = self.el(text).text.as_ref().and_then(|t| {
            let mut top = t.margins[0] - view;
            t.lines.iter().find_map(|l| {
                let found = (top <= 0 && top + l.height > 0).then_some(l.height);
                top += l.height;
                found
            })
        });
        let Some(line) = line.filter(|&h| h > 0) else {
            return;
        };
        self.text_set_view_y(text, view + steps * line);
    }

    /// A wheel turn over text `text`: its vertical bar takes it (`UIElement_Text::MouseDown`).
    pub(crate) fn text_wheel(&mut self, text: usize, action: u32) {
        if !matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT) {
            return;
        }
        let Some(bar) = self.text_scrollbar(text) else {
            return;
        };
        self.wheel_scrolled = true;
        self.text_update_scrollbar(text);
        self.scrollbar_mouse_down(bar, action);
    }
}
