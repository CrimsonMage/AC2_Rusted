//! `gmGamePlayUI`'s dialogs besides the death panel ([`crate::DeathPanel`], which also keeps the
//! resurrection offer): the please-wait dialog (`MakePleaseWaitDialog`, `RecvNotice_CloseWaitDialog`,
//! `DoPleaseWaitCancel`) and the message boxes (`RecvNotice_DisplayMessage`,
//! `RecvNotice_CloseMessageDialog`). The dialogs themselves are made and closed by
//! [`crate::DialogFactory`]; this keeps the contexts the gameplay UI keeps for them
//! (`_waitContext`, `_messageContexts`).
//!
//! The logout: the logout key (`LOGOUT_KEY`, Shift+Escape, action `0x4100000e`) asks for the end
//! of the session (`ListenToMessage`'s `_doEndSession`); the next UI loop runs `EndSession(1)`,
//! which closes the death dialog, raises the please-wait dialog and marks the session ended
//! (`_requestedEndSession`). The caller uploads the game options and sends `LogOffCharacter`.
//!
//! `gmGamePlayUI::Update` queues the character selection mode and sets `_endSession`
//! ([`GameplayDialogs::update`]); `gmGamePlayUI::Init` closes the please-wait dialog
//! ([`GameplayDialogs::init`]); the destructor's end of session and dialog closes are
//! [`GameplayDialogs::destroy`].
//!
//! Not modelled: `EndSession`'s story UI close, its notice-handler unregistering, `CleanupChildren`
//! (the HUD stays up until the server's `CharExitGame` moves the app on), `CWAllocator::ResetShell`
//! and the gameplay UI's `Show(1)` after a death dialog. The flow's `QueueUIMode` (character
//! selection) that a cancelled wait sends the player to is [`GameplayDialogs::recv_close_wait_dialog`]'s
//! caller's business.

use crate::chat_window::TEXT_TYPE_ERROR;
use crate::death::DeathPanel;
use crate::dialog::{DialogFactory, DIALOG_MESSAGE, DIALOG_WAIT};
use crate::element::UiTree;
use crate::token::tokenize;
use crate::ui_string::{UiString, UiStrings};

/// The chat text type of `SetErrorMsg`'s line (`TEXT_TYPE_ERROR`, as the chat's error type).
pub const ERROR_TEXT_TYPE: u32 = TEXT_TYPE_ERROR;

/// `LOGOUT_KEY` (Shift+Escape in the shift key map): the UI action `gmGamePlayUI::ListenToMessage`
/// ends the session on.
pub const LOGOUT_ACTION: u32 = 0x4100_000E;

/// `gmGamePlayUI`'s wait and message contexts (`_waitContext`, `_messageContexts`, 0 for none),
/// and its logout state (`_doEndSession`, `_requestedEndSession`).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameplayDialogs {
    wait_context: u32,
    message_contexts: Vec<u32>,
    do_end_session: bool,
    requested_end_session: bool,
    end_session_pending: bool,
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

    /// `gmGamePlayUI::_requestedEndSession`: the session's end has been asked for.
    pub fn end_session_requested(&self) -> bool {
        self.requested_end_session
    }

    /// `gmGamePlayUI::_endSession`: set by [`GameplayDialogs::update`].
    pub fn end_session_pending(&self) -> bool {
        self.end_session_pending
    }

    /// `gmGamePlayUI::Update`: with the app present (`UIFlow`, the vtable call at `+0x18`), the
    /// character selection mode is queued (`QueueUIMode("characterselection")`, the caller's) and
    /// `_endSession` is set. Returns whether the mode is queued; without the app, nothing happens.
    pub fn update(&mut self, app: bool) -> bool {
        if !app {
            return false;
        }
        self.end_session_pending = true;
        true
    }

    /// `gmGamePlayUI::Init`'s dialog step: the please-wait dialog closes (`DialogFactory::CloseDialog`
    /// on `_waitContext`, a no-op for 0) and its context is forgotten. The HUD's children are built
    /// by the caller.
    pub fn init(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        dialogs.close_dialog(tree, self.wait_context);
        self.wait_context = 0;
    }

    /// `gmGamePlayUI::EndSession(requested = 0)`: the death dialog closes and the session is marked
    /// ended, with no please-wait dialog. Returns whether it ran (once); the caller logs the
    /// character off as [`GameplayDialogs::end_session`] does.
    pub fn end_session_quiet(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        death: &mut DeathPanel,
    ) -> bool {
        if self.requested_end_session {
            return false;
        }
        death.close(tree, dialogs);
        self.requested_end_session = true;
        true
    }

    /// `gmGamePlayUI::~gmGamePlayUI`'s dialog part: `EndSession(0)` runs first when the session has
    /// not ended (returned, as [`GameplayDialogs::end_session_quiet`]), then the please-wait dialog
    /// and every message box close and their contexts are forgotten.
    pub fn destroy(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        death: &mut DeathPanel,
    ) -> bool {
        let ended = self.end_session_quiet(tree, dialogs, death);
        dialogs.close_dialog(tree, self.wait_context);
        self.wait_context = 0;
        for context in std::mem::take(&mut self.message_contexts) {
            dialogs.close_dialog(tree, context);
        }
        ended
    }

    /// `gmGamePlayUI::ListenToMessage` for the global key press `0x4100000e` (`LOGOUT_KEY`):
    /// `_doEndSession` is set, and the next UI loop ends the session.
    pub fn logout_key(&mut self) {
        self.do_end_session = true;
    }

    /// `gmGamePlayUI::ListenToMessage` for the global loop (`0xffff0005`): a pending logout that
    /// was not yet answered runs `EndSession(1)`. Returns whether it ran (see
    /// [`GameplayDialogs::end_session`]).
    pub fn end_session_loop(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
        death: &mut DeathPanel,
    ) -> bool {
        if self.requested_end_session || !self.do_end_session {
            return false;
        }
        self.do_end_session = false;
        self.end_session(tree, dialogs, strings, death)
    }

    /// `gmGamePlayUI::EndSession(requested = 1)`: the death dialog closes, the please-wait dialog
    /// is raised (`MakePleaseWaitDialog`) and the session is marked ended. Returns whether it ran
    /// (it runs once); the caller then sends `UploadGameOptions` and `LogOffCharacter`
    /// (`Proto_UI::LogOffCharacter`), in that order.
    pub fn end_session(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
        death: &mut DeathPanel,
    ) -> bool {
        if self.requested_end_session {
            return false;
        }
        death.close(tree, dialogs);
        self.make_please_wait(tree, dialogs, strings);
        self.requested_end_session = true;
        true
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

    #[test]
    fn the_logout_key_ends_the_session_once_on_the_next_loop() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut game = GameplayDialogs::default();
        let mut death = DeathPanel::default();
        assert_eq!(LOGOUT_ACTION, 0x4100_000E);
        assert!(!game.end_session_loop(&mut tree, &mut dialogs, &Text, &mut death));
        assert!(!game.end_session_requested());

        game.logout_key();
        assert!(!game.end_session_requested(), "the key only asks");
        assert!(game.end_session_loop(&mut tree, &mut dialogs, &Text, &mut death));
        assert!(game.end_session_requested());
        let wait = game.wait_context();
        assert_ne!(wait, 0);
        assert_eq!(dialogs.shown(), Some((wait, DIALOG_WAIT)));

        // Ended once: a second key or loop asks nothing more.
        game.logout_key();
        assert!(!game.end_session_loop(&mut tree, &mut dialogs, &Text, &mut death));
        assert!(!game.end_session(&mut tree, &mut dialogs, &Text, &mut death));
        assert_eq!(game.wait_context(), wait);
    }

    #[test]
    fn ending_the_session_closes_the_death_dialog_first() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut game = GameplayDialogs::default();
        let mut death = DeathPanel::default();
        death.notice(&mut tree, &mut dialogs, &Text, true);
        let died = death.context();
        assert_ne!(died, 0);
        assert_eq!(dialogs.shown().map(|(context, _)| context), Some(died));

        assert!(game.end_session(&mut tree, &mut dialogs, &Text, &mut death));
        assert_eq!(death.context(), 0);
        let wait = game.wait_context();
        assert_ne!(wait, died);
        assert_eq!(dialogs.shown(), Some((wait, DIALOG_WAIT)));
    }

    #[test]
    fn update_queues_the_mode_only_with_an_app_and_sets_end_session() {
        let mut game = GameplayDialogs::default();
        assert!(!game.update(false));
        assert!(!game.end_session_pending());
        assert!(game.update(true));
        assert!(game.end_session_pending());
        // The flag is kept; the mode is queued again on each update the app runs.
        assert!(game.update(true));
    }

    #[test]
    fn init_closes_the_wait_dialog_it_was_left_with() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut game = GameplayDialogs::default();
        game.make_please_wait(&mut tree, &mut dialogs, &Text);
        assert_ne!(game.wait_context(), 0);
        game.init(&mut tree, &mut dialogs);
        assert_eq!(game.wait_context(), 0);
        assert_eq!(dialogs.shown(), None);
        // With no wait dialog, Init closes nothing.
        game.init(&mut tree, &mut dialogs);
        assert_eq!(game.wait_context(), 0);
    }

    #[test]
    fn the_destructor_ends_an_open_session_quietly_and_closes_its_dialogs() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut game = GameplayDialogs::default();
        let mut death = DeathPanel::default();
        let first = game.display_message(&mut tree, &mut dialogs, "one", false);
        game.make_please_wait(&mut tree, &mut dialogs, &Text);
        death.notice(&mut tree, &mut dialogs, &Text, true);
        let died = death.context();

        assert!(game.destroy(&mut tree, &mut dialogs, &mut death));
        assert_eq!(death.context(), 0, "EndSession(0) closes the death dialog");
        assert!(game.end_session_requested());
        assert_eq!(game.wait_context(), 0);
        assert!(game.message_contexts().is_empty());
        assert_eq!(dialogs.shown(), None);
        assert!(dialogs.queued().is_empty(), "the queued message closed too");
        assert_ne!(first, died);

        // A session that already ended is not ended again.
        assert!(!game.destroy(&mut tree, &mut dialogs, &mut death));
    }

    #[test]
    fn the_quiet_end_raises_no_wait_dialog() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut game = GameplayDialogs::default();
        let mut death = DeathPanel::default();
        assert!(game.end_session_quiet(&mut tree, &mut dialogs, &mut death));
        assert!(game.end_session_requested());
        assert_eq!(game.wait_context(), 0);
        assert_eq!(dialogs.shown(), None);
        assert!(!game.end_session_quiet(&mut tree, &mut dialogs, &mut death));
    }
}
