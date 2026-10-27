//! `gmGamePlayUI`'s dialogs besides the death panel ([`crate::DeathPanel`], which also keeps the
//! resurrection offer): the please-wait dialog (`MakePleaseWaitDialog`, `RecvNotice_CloseWaitDialog`,
//! `DoPleaseWaitCancel`) and the message boxes (`RecvNotice_DisplayMessage`,
//! `RecvNotice_CloseMessageDialog`). The dialogs themselves are made and closed by
//! [`crate::DialogFactory`]; this keeps the contexts the gameplay UI keeps for them
//! (`_waitContext`, `_messageContexts`).
//!
//! Not modelled: the please-wait dialog that `EndSession` shows while the character logs off (it
//! is raised by `EndSession(requested)`, which no HUD control reaches yet), and the flow's
//! `QueueUIMode` (character selection) that a cancelled wait sends the player to.

use crate::chat_window::TEXT_TYPE_ERROR;
use crate::dialog::{DialogFactory, DIALOG_MESSAGE, DIALOG_WAIT};
use crate::element::UiTree;
use crate::token::tokenize;
use crate::ui_string::{UiString, UiStrings};

/// The chat text type of `SetErrorMsg`'s line (`TEXT_TYPE_ERROR`, as the chat's error type).
pub const ERROR_TEXT_TYPE: u32 = TEXT_TYPE_ERROR;

/// `gmGamePlayUI`'s wait and message contexts (`_waitContext`, `_messageContexts`, 0 for none).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameplayDialogs {
    wait_context: u32,
    message_contexts: Vec<u32>,
}

impl GameplayDialogs {
    /// `_waitContext`: the please-wait dialog's context, 0 for none.
    pub fn wait_context(&self) -> u32 {
        self.wait_context
    }

    /// `_messageContexts`: the message boxes not yet answered, oldest first.
    pub fn message_contexts(&self) -> &[u32] {
        &self.message_contexts
    }

    /// `gmGamePlayUI::MakePleaseWaitDialog`: one wait dialog at a time (`ID_Wait_PleaseWait`,
    /// wait type, not topmost).
    pub fn make_please_wait(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
    ) {
        if self.wait_context != 0 {
            return;
        }
        let text = strings.render_string(&UiString::ui(tokenize("ID_Wait_PleaseWait"), Vec::new()));
        self.wait_context = dialogs.make_dialog(tree, &text, DIALOG_WAIT, false);
    }

    /// `gmGamePlayUI::RecvNotice_CloseWaitDialog`: the wait context's close runs
    /// `DoPleaseWaitCancel`. Returns whether it ran; the caller then queues the character
    /// selection mode with no message (`UIFlow::QueueUIMode`), which the HUD does not reach yet.
    pub fn recv_close_wait_dialog(&mut self, context: u32) -> bool {
        if context != self.wait_context {
            return false;
        }
        self.do_please_wait_cancel();
        true
    }

    /// `gmGamePlayUI::DoPleaseWaitCancel`: the wait is over. The dialog that closed it is already
    /// gone, so only its context is forgotten.
    fn do_please_wait_cancel(&mut self) {
        self.wait_context = 0;
    }

    /// `gmGamePlayUI::RecvNotice_DisplayMessage`: a message box (`DIALOG_MESSAGE`, at the top when
    /// `topmost`), kept in `_messageContexts` until it is answered. Returns its context.
    pub fn display_message(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        text: &str,
        topmost: bool,
    ) -> u32 {
        let context = dialogs.make_dialog(tree, text, DIALOG_MESSAGE, topmost);
        self.message_contexts.push(context);
        context
    }

    /// `gmGamePlayUI::RecvNotice_CloseMessageDialog`: the answered message's context leaves the
    /// list (the last one with that context, as the list walk finds it).
    pub fn close_message(&mut self, context: u32) {
        if let Some(at) = self.message_contexts.iter().rposition(|&c| c == context) {
            self.message_contexts.remove(at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::{DIALOG_MESSAGE, DIALOG_WAIT};

    struct Text;

    impl UiStrings for Text {
        fn render_string(&self, string: &UiString) -> String {
            format!("{:08x}/{:08x}", string.table_enum, string.string_id)
        }
    }

    #[test]
    fn the_wait_dialog_is_made_once_and_its_close_cancels_the_wait() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut game = GameplayDialogs::default();
        game.make_please_wait(&mut tree, &mut dialogs, &Text);
        let context = game.wait_context();
        assert_ne!(context, 0);
        assert_eq!(dialogs.shown(), Some((context, DIALOG_WAIT)));
        game.make_please_wait(&mut tree, &mut dialogs, &Text);
        assert_eq!(game.wait_context(), context);
        assert!(dialogs.queued().is_empty());

        assert!(!game.recv_close_wait_dialog(context + 1));
        assert_eq!(game.wait_context(), context);
        assert!(game.recv_close_wait_dialog(context));
        assert_eq!(game.wait_context(), 0);
        assert!(!game.recv_close_wait_dialog(context));
    }

    #[test]
    fn messages_are_listed_until_they_are_answered() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut game = GameplayDialogs::default();
        let first = game.display_message(&mut tree, &mut dialogs, "one", false);
        let second = game.display_message(&mut tree, &mut dialogs, "two", true);
        assert_eq!(game.message_contexts(), &[first, second]);
        assert_eq!(dialogs.shown(), Some((second, DIALOG_MESSAGE)));

        game.close_message(first);
        assert_eq!(game.message_contexts(), &[second]);
        game.close_message(first);
        assert_eq!(game.message_contexts(), &[second]);
        game.close_message(second);
        assert!(game.message_contexts().is_empty());
    }

    #[test]
    fn the_error_line_is_the_chat_error_type() {
        assert_eq!(ERROR_TEXT_TYPE, 0x1);
    }
}
