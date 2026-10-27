//! `gmClientUI`: the client options page's own state (`ac2_client::client_options` holds the
//! preference values it applies). The page's expanders are keyed by section ([`ClientPage::lookup_expander`]),
//! its Apply and Recall buttons follow whether a value changed ([`ClientPage::buttons`]), and its
//! questions are dialogs kept by their contexts: Recall and Default ask first, and a graphics
//! change asks for verification (`VerifyRenderOptions`, a timed dialog) before anything is saved.
//!
//! The question texts are the caller's: retail's string IDs in table `0x41000018` are not named in
//! this port, so [`ClientPageText`] carries the rendered lines.
//!
//! Legions 7160 has no `RecvNotice_CloseConfirmationDialog` for this page; the answer below is the
//! admin 7148 one, and [`ClientPage::recv_close_confirmation`] says which actions it asks for.

use crate::dialog::{DialogFactory, DIALOG_CONFIRMATION, DIALOG_TIMED};
use crate::element::UiTree;

/// The sections of `gmClientUI::Construct`, in the order it builds their expanders (the
/// Rendering section first, then Graphics, Camera, Avatar, Input, FX, Sound and Misc).
pub const SECTION_ORDER: [u32; 8] = [
    0x4000_000B,
    0x4000_0004,
    0x4000_0008,
    0x4000_000D,
    0x4000_000E,
    0x4000_0009,
    0x4000_0007,
    0x4000_000C,
];
/// `gmClientUI::_indexRendering`'s section: the display section the graphics verification concerns.
pub const RENDERING_SECTION: u32 = 0x4000_000B;

/// The question texts (`ConfirmRecall`, `ConfirmDefault` and `VerifyRenderOptions` render their
/// message from a string of table `0x41000018`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientPageText {
    pub recall: String,
    pub default: String,
    pub render: String,
}

/// What the page's Apply and Recall buttons look like (`gmClientUI::UpdateButtons`): both are
/// enabled while a value differs from the applied one, and disabled otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonStates {
    pub apply: bool,
    pub recall: bool,
}

/// What a question's answer asks the caller to do, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageAction {
    /// The display section's pending values go back to the applied ones (a render verification
    /// answered no), then the preferences are saved.
    RecallDisplay,
    /// The display section's values become the applied ones (a render verification answered yes),
    /// then the preferences are saved.
    SynchronizeDisplay,
    /// `gmClientUI::Recall`: every section goes back to its applied values.
    Recall,
    /// `gmClientUI::Default`: every section goes to its defaults and is applied.
    Default,
}

/// The page's dialogs and expanders (`_sections`, `_contextRender`, `_contextRecall`,
/// `_contextDefault`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPage {
    text: ClientPageText,
    /// The expanders `Construct` made, in [`SECTION_ORDER`] (0 once dismantled).
    sections: usize,
    context_render: u32,
    context_recall: u32,
    context_default: u32,
}

impl ClientPage {
    /// `gmClientUI::gmClientUI` and `Construct`: the page's expanders in [`SECTION_ORDER`] and no
    /// question open.
    pub fn new(text: ClientPageText) -> Self {
        Self {
            text,
            sections: SECTION_ORDER.len(),
            context_render: 0,
            context_recall: 0,
            context_default: 0,
        }
    }

    /// `gmClientUI::LookupExpander(section)`: the index into [`SECTION_ORDER`] of the section's
    /// expander, or `None` for a section the page has none for (and after `Dismantle`).
    pub fn lookup_expander(&self, section: u32) -> Option<usize> {
        let index = match section {
            0x4000_0004 | 0x4000_0007 | 0x4000_0008 | 0x4000_0009 | 0x4000_000B | 0x4000_000C
            | 0x4000_000D | 0x4000_000E => SECTION_ORDER.iter().position(|&s| s == section)?,
            _ => return None,
        };
        (index < self.sections).then_some(index)
    }

    /// `gmClientUI::UpdateButtons`: with the page built, Apply and Recall are enabled exactly while
    /// some section changed (`changed` is `gmClientUI::Changed`). `None` once the page is gone.
    pub fn buttons(&self, changed: bool) -> Option<ButtonStates> {
        (self.sections > 0).then_some(ButtonStates {
            apply: changed,
            recall: changed,
        })
    }

    /// `gmClientUI::ConfirmRecall`: asks once while the question is open (`DIALOG_CONFIRMATION`).
    pub fn confirm_recall(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context_recall == 0 {
            self.context_recall =
                dialogs.make_dialog(tree, &self.text.recall, DIALOG_CONFIRMATION, false);
        }
    }

    /// `gmClientUI::ConfirmDefault`: asks once while the question is open.
    pub fn confirm_default(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context_default == 0 {
            self.context_default =
                dialogs.make_dialog(tree, &self.text.default, DIALOG_CONFIRMATION, false);
        }
    }

    /// `gmClientUI::VerifyRenderOptions`: the graphics change is asked about (a timed dialog,
    /// `DIALOG_TIMED`), once while it is open.
    pub fn verify_render_options(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context_render == 0 {
            self.context_render = dialogs.make_dialog(tree, &self.text.render, DIALOG_TIMED, false);
        }
    }

    /// `gmClientUI::ClearConfirmations`: every open question closes and its context is forgotten.
    pub fn clear_confirmations(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        for context in [
            &mut self.context_render,
            &mut self.context_recall,
            &mut self.context_default,
        ] {
            if *context != 0 {
                dialogs.close_dialog(tree, *context);
                *context = 0;
            }
        }
    }

    /// The question contexts that are open (render, recall, default), for the caller's answers.
    pub fn open_contexts(&self) -> [u32; 3] {
        [
            self.context_render,
            self.context_recall,
            self.context_default,
        ]
    }

    /// The page's end after an Apply: a graphics change that needs verifying asks for it, and the
    /// buttons are then updated (`gmClientUI::Apply`'s `VerifyRenderOptions` or the save, then
    /// `UpdateButtons`). Returns whether the verification was asked.
    pub fn after_apply(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        verify_render: bool,
    ) -> bool {
        if verify_render {
            self.verify_render_options(tree, dialogs);
        }
        verify_render
    }

    /// `gmClientUI::RecvNotice_CloseConfirmationDialog` (admin 7148). A render verification answered
    /// no recalls the display section and saves; answered yes, it synchronizes the display section
    /// and saves. A yes to Recall or Default asks for those actions. Every open question then
    /// closes. The answered context's own action is only taken on yes.
    pub fn recv_close_confirmation(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        context: u32,
        yes: bool,
    ) -> Vec<PageAction> {
        let mut actions = Vec::new();
        if context != 0 && context == self.context_render {
            actions.push(if yes {
                PageAction::SynchronizeDisplay
            } else {
                PageAction::RecallDisplay
            });
        }
        if yes && context != 0 {
            if context == self.context_recall {
                actions.push(PageAction::Recall);
            }
            if context == self.context_default {
                actions.push(PageAction::Default);
            }
        }
        self.clear_confirmations(tree, dialogs);
        actions
    }

    /// `gmClientUI::Dismantle` and the destructor's part of it: the expanders are gone, so the
    /// lookups find none. The questions close with [`ClientPage::clear_confirmations`], which the
    /// caller runs first.
    pub fn dismantle(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        self.clear_confirmations(tree, dialogs);
        self.sections = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sections_are_built_in_construct_order_and_looked_up_by_id() {
        let page = ClientPage::new(ClientPageText::default());
        assert_eq!(SECTION_ORDER[0], RENDERING_SECTION);
        assert_eq!(page.lookup_expander(0x4000_000B), Some(0));
        assert_eq!(page.lookup_expander(0x4000_0004), Some(1));
        assert_eq!(page.lookup_expander(0x4000_0007), Some(6));
        assert_eq!(page.lookup_expander(0x4000_000C), Some(7));
        // Not a section of the page: the device section is submitted, not expanded.
        assert_eq!(page.lookup_expander(0x4000_0001), None);
    }

    #[test]
    fn the_buttons_follow_the_changed_state_and_go_with_the_page() {
        let mut page = ClientPage::new(ClientPageText::default());
        assert_eq!(
            page.buttons(false),
            Some(ButtonStates {
                apply: false,
                recall: false
            })
        );
        assert_eq!(page.buttons(true).map(|b| b.apply), Some(true));

        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        page.dismantle(&mut tree, &mut dialogs);
        assert_eq!(page.buttons(true), None);
        assert_eq!(page.lookup_expander(0x4000_0004), None);
    }

    #[test]
    fn recall_and_default_ask_once_with_their_texts_and_close_together() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let text = ClientPageText {
            recall: "recall?".into(),
            default: "default?".into(),
            render: "verify?".into(),
        };
        let mut page = ClientPage::new(text);
        page.confirm_recall(&mut tree, &mut dialogs);
        let recall = page.open_contexts()[1];
        assert_ne!(recall, 0);
        assert_eq!(dialogs.shown(), Some((recall, DIALOG_CONFIRMATION)));
        page.confirm_recall(&mut tree, &mut dialogs);
        assert_eq!(page.open_contexts()[1], recall, "asked once");
        page.confirm_default(&mut tree, &mut dialogs);
        assert_eq!(dialogs.queued().len(), 1);

        page.clear_confirmations(&mut tree, &mut dialogs);
        assert_eq!(page.open_contexts(), [0, 0, 0]);
        assert_eq!(dialogs.shown(), None);
        assert!(dialogs.queued().is_empty());
    }

    #[test]
    fn a_render_change_asks_for_verification_as_a_timed_dialog() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut page = ClientPage::new(ClientPageText::default());
        assert!(!page.after_apply(&mut tree, &mut dialogs, false));
        assert_eq!(dialogs.shown(), None);

        assert!(page.after_apply(&mut tree, &mut dialogs, true));
        let render = page.open_contexts()[0];
        assert_eq!(dialogs.shown(), Some((render, DIALOG_TIMED)));
    }

    #[test]
    fn the_render_answer_recalls_or_synchronizes_the_display_section() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut page = ClientPage::new(ClientPageText::default());

        page.verify_render_options(&mut tree, &mut dialogs);
        let render = page.open_contexts()[0];
        assert_eq!(
            page.recv_close_confirmation(&mut tree, &mut dialogs, render, false),
            vec![PageAction::RecallDisplay]
        );
        assert_eq!(page.open_contexts(), [0, 0, 0]);

        page.verify_render_options(&mut tree, &mut dialogs);
        let render = page.open_contexts()[0];
        assert_eq!(
            page.recv_close_confirmation(&mut tree, &mut dialogs, render, true),
            vec![PageAction::SynchronizeDisplay]
        );
    }

    #[test]
    fn only_a_yes_to_recall_or_default_runs_it() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut page = ClientPage::new(ClientPageText::default());

        page.confirm_recall(&mut tree, &mut dialogs);
        let recall = page.open_contexts()[1];
        assert!(page
            .recv_close_confirmation(&mut tree, &mut dialogs, recall, false)
            .is_empty());

        page.confirm_default(&mut tree, &mut dialogs);
        let default = page.open_contexts()[2];
        assert_eq!(
            page.recv_close_confirmation(&mut tree, &mut dialogs, default, true),
            vec![PageAction::Default]
        );
        page.confirm_recall(&mut tree, &mut dialogs);
        let recall = page.open_contexts()[1];
        assert_eq!(
            page.recv_close_confirmation(&mut tree, &mut dialogs, recall, true),
            vec![PageAction::Recall]
        );
        // An unknown context answers nothing.
        assert!(page
            .recv_close_confirmation(&mut tree, &mut dialogs, 0x7777, true)
            .is_empty());
    }
}
