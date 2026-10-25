//! `gmKeyboardUI`: the controls screen from character selection's options button.
//! `KeyboardField` holds the key binding list (`list_box`) and its buttons; clear is ghosted
//! without a selected binding, save and undo without a change (`UpdateButtons`). Quit returns to
//! character selection behind a please-wait dialog (`Close`; with changes it would ask first).
//!
//! The binding list's model (rows, set, clear, clear all, the key hit gate, the saved bindings)
//! is [`KeyboardList`](super::KeyboardList). The page does not show it yet: the row elements and
//! the key names are not made, so nothing is ever modified here.

use crate::dialog::DIALOG_WAIT;
use crate::mouse::UiMessage;
use crate::pregame::flow::MODE_CHARACTER_SELECTION;
use crate::pregame::{Ctx, LAYOUT_KEYBOARD};
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};

/// `gmKeyboardUI`'s state.
#[derive(Debug, Clone, Default)]
pub(crate) struct KeyboardUi {
    field: Option<usize>,
    /// `m_clearButton`, `m_applyButton`, `m_undoButton`.
    buttons: [Option<usize>; 3],
    /// `m_selected`, `m_modified`.
    selected: Option<usize>,
    modified: bool,
    wait_context: u32,
}

impl KeyboardUi {
    /// `gmKeyboardUI::gmKeyboardUI`: the field and its buttons, then `UpdateButtons`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let field = ctx.assets.layout(LAYOUT_KEYBOARD).and_then(|layout| {
            ctx.tree
                .create_root(layout.clone(), tokenize("KeyboardField"))
        });
        let child = |ctx: &Ctx<'_>, name: &str| ctx.tree.child_recursive(field?, tokenize(name));
        let ui = Self {
            field,
            buttons: [
                child(ctx, "clear_button"),
                child(ctx, "save_button"),
                child(ctx, "undo_button"),
            ],
            ..Self::default()
        };
        ui.update_buttons(ctx);
        ui
    }

    /// The framework's destructor: its dialogs closed (`CloseAllDialogs`), its root removed.
    pub(crate) fn destroy(self, ctx: &mut Ctx<'_>) {
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
        if let Some(field) = self.field {
            ctx.tree.remove_root(field);
        }
    }

    /// `gmKeyboardUI::UpdateButtons`.
    fn update_buttons(&self, ctx: &mut Ctx<'_>) {
        let [Some(clear), Some(apply), Some(undo)] = self.buttons else {
            return;
        };
        ctx.tree
            .set_bool(clear, BUTTON_GHOSTED, self.selected.is_none());
        ctx.tree.set_bool(apply, BUTTON_GHOSTED, !self.modified);
        ctx.tree.set_bool(undo, BUTTON_GHOSTED, !self.modified);
    }

    /// `gmKeyboardUI::Close`: back to character selection behind a please-wait dialog.
    fn close(&mut self, ctx: &mut Ctx<'_>) {
        let text = ctx.strings.get("ID_Wait_PleaseWait");
        self.wait_context = ctx.dialogs.make_dialog(ctx.tree, &text, DIALOG_WAIT, false);
        ctx.flow.queue_mode(MODE_CHARACTER_SELECTION, "");
    }

    /// `gmKeyboardUI::ListenToMessage`: quit (unchanged, so without asking).
    pub(crate) fn message(&mut self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        if message.message == MSG_BUTTON_RELEASE_ON
            && message.element_id == tokenize("quit_button")
            && !self.modified
        {
            self.close(ctx);
        }
    }
}
