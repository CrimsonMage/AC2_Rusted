//! `gmChatUI`'s spew box: the error-type strings that float in a list over the world.
//!
//! `gmChatUI::RecvNotice_DisplayStringInfo` queues each error-type line (`type & 1`, the
//! `TEXT_TYPE_ERROR` bit) in the pending list. `gmChatUI::Update`, run on each global loop
//! (`ListenToMessage`'s `0xffff0005`), first drops the lines whose removal time has come, then
//! makes an element from `SpewBoxMessage` for each pending line, trimmed of trailing space,
//! adds it to the `SpewBox` list and sets its removal time to `decay_time` seconds from now.
//! `decay_time` is read from the element (`UIElement::GetAttribute_Float`, 5.0 when it has
//! none); the sum is taken in f32, as the decompile's `(float)` casts do.
//!
//! Retail makes no root for the spew box. Its holder is a top-level element of the spew layout
//! (`0x4100000f`) that only the status bar's layout reaches: the status bar root (`StatusbarField`)
//! has a `reference` child whose `referent` is that holder, so the box is drawn inside the status
//! bar at the reference's rectangle. `gmChatUI::gmChatUI` finds the list by id over every element
//! (`UIElementManager::GetElement`), so the lookup here is over the whole tree too. The holder and
//! its list are `mouse_invisible`, so presses over the box reach what lies beneath.
//!
//! Not modelled: `gmChatUI::Show` runs `Update` when the chat first shows; the chat here is
//! always shown, so the loop alone drives the box.

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::{ElementDesc, LayoutDesc};
use crate::token::tokenize;

/// `SpewBox`: the list of lines, in the spew layout.
pub const SPEW_BOX: u32 = tokenize("SpewBox");
/// `SpewBoxMessage`: the template of one line.
const SPEW_BOX_MESSAGE: u32 = tokenize("SpewBoxMessage");
/// `decay_time`: how long a line stays, read from the line's element.
const DECAY_TIME: u32 = tokenize("decay_time");
/// The decay when the line has no `decay_time` (`gmChatUI::Update`'s `0x40a00000`, 5.0f).
const DEFAULT_DECAY_TIME: f32 = 5.0;

/// One shown line: its element and the `Timer::cur_time` at which it leaves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SpewLine {
    pub element: usize,
    pub removal_time: f64,
}

/// `gmChatUI::_spewBoxPending` and `_spewBoxItems`, with the list and its layout.
#[derive(Debug, Clone)]
pub(crate) struct SpewBox {
    list: usize,
    layout: Arc<LayoutDesc>,
    message: Arc<ElementDesc>,
    pending: Vec<String>,
    lines: Vec<SpewLine>,
}

impl SpewBox {
    /// `gmChatUI::gmChatUI`: the `SpewBox` list (`GetElement`, the first with that id in the
    /// tree; it sits under the status bar root), and the `SpewBoxMessage` template from `layout`.
    /// `None` when the tree has no spew box or the layout no message template.
    pub(crate) fn create(tree: &UiTree, layout: &Arc<LayoutDesc>) -> Option<Self> {
        let list = tree
            .roots()
            .iter()
            .find_map(|&root| tree.child_recursive(root, SPEW_BOX))?;
        let message = layout.find_element(SPEW_BOX_MESSAGE)?;
        Some(Self {
            list,
            layout: Arc::clone(layout),
            message,
            pending: Vec::new(),
            lines: Vec::new(),
        })
    }

    /// `RecvNotice_DisplayStringInfo` for the error bit: the line joins the pending list.
    pub(crate) fn queue(&mut self, text: &str) {
        self.pending.push(text.to_owned());
    }

    /// The lines shown, in the list's order (tests).
    #[cfg(test)]
    pub(crate) fn shown(&self) -> &[SpewLine] {
        &self.lines
    }

    /// `gmChatUI::Update` at `Timer::cur_time` `now`: expired lines leave, the pending ones
    /// are shown and the pending list is emptied.
    pub(crate) fn update(&mut self, tree: &mut UiTree, now: f64) {
        let queued = std::mem::take(&mut self.pending);
        let mut changed = false;
        let mut i = 0;
        while i < self.lines.len() {
            if self.lines[i].removal_time <= now {
                let line = self.lines.remove(i);
                tree.list_delete_item(self.list, line.element);
                changed = true;
            } else {
                i += 1;
            }
        }
        if queued.is_empty() && !changed {
            return;
        }
        tree.list_multi_op_start(self.list);
        for text in &queued {
            self.show_line(tree, text, now);
        }
        tree.list_multi_op_complete(self.list);
        tree.list_end(self.list);
    }

    /// One pending line: its element from the template, text and height set, added to the
    /// list, its removal time from `decay_time`.
    fn show_line(&mut self, tree: &mut UiTree, text: &str, now: f64) {
        let Some(element) =
            tree.create_element(Arc::clone(&self.layout), Arc::clone(&self.message), false)
        else {
            return;
        };
        let width = tree.get(self.list).map_or(0, |e| e.rect().width());
        tree.set_text(element, text.trim_end());
        let height = tree.text_paper_height(element).unwrap_or(0);
        tree.resize_to(element, width, height);
        tree.list_add_item(self.list, element);
        let decay = tree
            .float_attribute(element, DECAY_TIME)
            .unwrap_or(DEFAULT_DECAY_TIME);
        // `(float)decay + (float)cur_time`, stored as a double.
        let removal_time = f64::from(decay + now as f32);
        self.lines.push(SpewLine {
            element,
            removal_time,
        });
    }
}
