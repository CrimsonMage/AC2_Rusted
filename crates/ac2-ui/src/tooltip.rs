//! Tooltips (`UIElementManager::StartTooltip` / `ClearTooltip`, `UIElement::SetTooltip`). The
//! innermost element under the pointer with `tooltips_on` owns the tooltip (`NotifyMouseMove`).
//! A quarter second after the pointer last moved, its `tooltip_ID` layout element is made as its
//! own root, its `tooltip_text_ID` child given the owner's text (`SetTooltip`, else its
//! `tooltip_entry` from the owner's string table), sized to the text and placed below and right
//! of the pointer, kept on the screen. It goes after ten seconds, or when the owner changes.
//!
//! Not modelled yet: the right-button check (`ActionDown(0x40000004)` keeps it away), the
//! tooltip options (`HandleApply`), and the cursor image's own size (32 x 32 is assumed for
//! `RenderUI::cursorWidth` / `cursorHeight`).

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::tokenize;

const TOOLTIPS_ON: u32 = tokenize("tooltips_on");
const TOOLTIP_ID: u32 = tokenize("tooltip_ID");
const TOOLTIP_TEXT_ID: u32 = tokenize("tooltip_text_ID");
const TOOLTIP_ENTRY: u32 = tokenize("tooltip_entry");
/// `RenderUI::cursorWidth` / `cursorHeight`.
const CURSOR_SIZE: (i32, i32) = (32, 32);

/// `UIElementManager`'s tooltip state.
#[derive(Debug, Clone)]
pub(crate) struct TooltipState {
    pub owner: Option<usize>,
    pub element: Option<usize>,
    pub last_mouse_move: f64,
    pub start: f64,
    pub enabled: bool,
    pub delay: f64,
    pub duration: f64,
}

impl Default for TooltipState {
    /// `UIElementManager::UIElementManager`.
    fn default() -> Self {
        Self {
            owner: None,
            element: None,
            last_mouse_move: -1.0,
            start: -1.0,
            enabled: true,
            delay: 0.25,
            duration: 10.0,
        }
    }
}

/// Reads every `tooltip_entry` string of `layout` through `data` while it may still read
/// files (the tree's text is frozen once the HUD is built; the frame thread reads none).
pub(crate) fn prefetch_tooltip_strings(
    data: &dyn crate::text::UiTextData,
    layout: &crate::layout::LayoutDesc,
) {
    fn walk(data: &dyn crate::text::UiTextData, table: u32, desc: &crate::layout::ElementDesc) {
        for state in &desc.states {
            for attribute in &state.attributes {
                if let (TOOLTIP_ENTRY, AttributeValue::StringId(id)) =
                    (attribute.id, &attribute.value)
                {
                    let _ = data.string(table, *id);
                }
            }
            for child in &state.children {
                walk(data, table, child);
            }
        }
    }
    for element in &layout.elements {
        walk(data, layout.string_table, element);
    }
}

impl UiTree {
    /// `UIElement::IsTooltipOn`.
    pub(crate) fn is_tooltip_on(&self, index: usize) -> bool {
        self.bool_attribute(index, TOOLTIPS_ON)
    }

    /// `GetAttribute_ID` / `GetAttribute_StringID`.
    fn tooltip_id_attribute(&self, index: usize, id: u32) -> Option<u32> {
        match self.el(index).attributes.get(&id) {
            Some(AttributeValue::Id(v)) | Some(AttributeValue::StringId(v)) => Some(*v),
            _ => None,
        }
    }

    /// `UIElement::SetTooltip`: the element's tooltip text; a shown tooltip of this owner is made
    /// again (`ResetTooltip`).
    pub fn set_tooltip(&mut self, index: usize, text: Option<String>) {
        self.el_mut(index).tooltip = text;
        if self.tooltip.owner == Some(index) && self.tooltip.element.is_some() {
            self.clear_tooltip();
            self.start_tooltip();
        }
    }

    /// The tooltip root shown now, if any.
    pub fn tooltip_element(&self) -> Option<usize> {
        self.tooltip.element
    }

    /// `UIElementManager::ClearTooltip`.
    pub(crate) fn clear_tooltip(&mut self) {
        if let Some(element) = self.tooltip.element.take() {
            self.remove_root(element);
        }
    }

    /// The owner chosen by the last mouse pass (`UpdateMouse`): a new owner clears the tooltip.
    pub(crate) fn set_tooltip_owner(&mut self, owner: Option<usize>) {
        if self.tooltip.owner != owner {
            self.tooltip.owner = owner;
            self.clear_tooltip();
        }
    }

    /// `UIElementManager::UseTime`'s tooltip part: shown once the pointer rested for the delay,
    /// gone after the duration.
    pub(crate) fn tooltip_use_time(&mut self, now: f64) {
        match self.tooltip.element {
            None => {
                if self.tooltip.delay <= now - self.tooltip.last_mouse_move {
                    self.start_tooltip();
                }
            }
            Some(_) => {
                if self.tooltip.duration <= now - self.tooltip.start {
                    self.clear_tooltip();
                }
            }
        }
    }

    /// `UIElementManager::StartTooltip`.
    pub(crate) fn start_tooltip(&mut self) {
        self.clear_tooltip();
        if !self.tooltip.enabled {
            return;
        }
        let Some(owner) = self.tooltip.owner.filter(|&o| self.get(o).is_some()) else {
            return;
        };
        let Some(tooltip_id) = self.tooltip_id_attribute(owner, TOOLTIP_ID) else {
            return;
        };
        let layout = Arc::clone(&self.el(owner).layout);
        let Some(desc) = layout.element(tooltip_id).cloned() else {
            return;
        };
        let text = match &self.el(owner).tooltip {
            Some(text) => Some(text.clone()),
            None => {
                let entry = self.tooltip_id_attribute(owner, TOOLTIP_ENTRY);
                let table = layout.string_table;
                entry.and_then(|id| {
                    let data = self.text_data.as_ref()?;
                    data.string(table, id).map(|t| String::from_utf16_lossy(&t))
                })
            }
        };
        let Some(text) = text.filter(|t| !t.is_empty()) else {
            return;
        };
        let Some(tip) = self.create_element(layout, desc, true) else {
            return;
        };
        let text_element = self
            .tooltip_id_attribute(tip, TOOLTIP_TEXT_ID)
            .and_then(|id| self.child_recursive(tip, id));
        let Some(text_element) = text_element else {
            self.delete(tip);
            return;
        };
        let (tw, th) = self
            .text_size_with_margins(text_element, &text)
            .unwrap_or((0, 0));
        let tip_rect = self.el(tip).rect;
        let text_rect = self.el(text_element).rect;
        let mut width = tw + tip_rect.width() - text_rect.width();
        let mut height = th + tip_rect.height() - text_rect.height();
        self.resize_to(tip, width, height);
        self.set_text(text_element, &text);
        if let Some(paper) = self.text_paper_height(text_element) {
            if th < paper {
                let tip_rect = self.el(tip).rect;
                let text_rect = self.el(text_element).rect;
                height = tip_rect.height() - text_rect.height() + paper;
                self.resize_to(tip, tip_rect.width(), height);
            }
        }
        let rect = self.el(tip).rect;
        (width, height) = (rect.width(), rect.height());
        let x = (self.pointer.0 + CURSOR_SIZE.0)
            .max(0)
            .min(self.display.0 - width);
        let y = (self.pointer.1 + CURSOR_SIZE.1)
            .max(0)
            .min(self.display.1 - height);
        self.el_mut(tip).object = Some((x, y));
        self.el_mut(tip).z = 0xFFFF;
        self.add_root(tip);
        self.set_visible(tip, true);
        self.tooltip.element = Some(tip);
        self.tooltip.start = self.time;
        self.moved = true;
    }
}
