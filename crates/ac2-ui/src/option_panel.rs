//! `gmOptionUI` (Legions 0x00434FB0..0x004357B0): the option panel over its pages. Closing the
//! panel (`Show(0)`) while a page has changes asks `ID_Option_ConfirmText`; yes applies the
//! pages (`DoConfirm`), no recalls them. The Leave button asks `ID_Option_LogoutText` and the
//! Quit button `ID_Option_QuitText`; while one of those is up the three buttons are ghosted
//! (`UpdateButtons`). A yes to the quit question is `CM_UI::SendNotice_Logoff` (upload the game
//! options, close the client), a yes to the leave question
//! `CM_UI::SendNotice_EndCharacterSession` (the gameplay UI ends the session). Retail's Assistance
//! button opened the `AssistanceTab` page, the `CommandTab`'s messages close both questions, and
//! the drop-down menus of the pages open up or down by their place in the panel
//! (`DropDownMenu` `mouse_over`).
//!
//! The client's change: the Assistance button is the In-Game Wiki button. Its label says so and a
//! press opens the quest guide ([`OptionPanel::take_wiki`]). The assistance page it opened has no
//! tab of its own (`AssistanceTab` sits off the panel), so it is no longer reached: the servers
//! the client plays on do not answer assistance calls.

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_OVER};
use crate::ui_string::{UiString, UiStrings};

/// What an answered question asks the HUD to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionAction {
    /// `DoConfirm`: apply every page (yes) or recall them (no).
    Confirm(bool),
    /// `CM_UI::SendNotice_Logoff`.
    Logoff,
    /// `CM_UI::SendNotice_EndCharacterSession`.
    EndCharacterSession,
}

/// `gmOptionUI`'s elements and question contexts (0 for none).
#[derive(Debug, Clone)]
pub struct OptionPanel {
    field: usize,
    leave: usize,
    quit: usize,
    assistance: usize,
    /// The In-Game Wiki button was pressed since the HUD last asked.
    wiki: bool,
    /// The modern Edit HUD layout button was pressed.
    edit: bool,
    confirm: u32,
    quit_context: u32,
    logout_context: u32,
    shown: bool,
}

/// The button's label (its first text descendant) set to `text`.
fn set_label(tree: &mut UiTree, button: usize, text: &str) {
    let mut stack = vec![button];
    while let Some(element) = stack.pop() {
        if tree.get(element).is_some_and(|e| e.text().is_some()) {
            tree.set_text(element, text);
            return;
        }
        stack.extend(tree.children(element).iter().copied());
    }
}

fn find(tree: &UiTree, name: &str) -> Option<usize> {
    let id = tokenize(name);
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, id))
}

impl OptionPanel {
    /// `gmOptionUI::gmOptionUI`: the field and the three buttons (all or nothing); hidden.
    pub fn create(tree: &UiTree) -> Option<Self> {
        let field = find(tree, "OptionField")?;
        Some(Self {
            field,
            leave: tree.child_recursive(field, tokenize("LeaveButton"))?,
            quit: tree.child_recursive(field, tokenize("QuitButton"))?,
            assistance: tree.child_recursive(field, tokenize("AssistanceButton"))?,
            wiki: false,
            edit: false,
            confirm: 0,
            quit_context: 0,
            logout_context: 0,
            shown: false,
        })
    }

    /// The Assistance button relabelled as the In-Game Wiki button.
    pub fn label_wiki(&self, tree: &mut UiTree) {
        set_label(tree, self.assistance, "In-Game Wiki");
    }

    /// The modern style's Edit HUD layout button (`modern_layout_edit`): a copy of the In-Game
    /// Wiki button under it.
    pub fn add_edit_layout_button(&self, tree: &mut UiTree) {
        let Some(page) = tree.el(self.assistance).parent_index() else {
            return;
        };
        let layout = std::sync::Arc::clone(&tree.el(self.assistance).layout);
        let mut desc = (*tree.el(self.assistance).reference).clone();
        desc.id = tokenize("ModernEditLayoutButton");
        let Some(button) = tree.create_element(layout, std::sync::Arc::new(desc), false) else {
            return;
        };
        tree.set_parent(button, Some(page));
        let at = tree.el(self.assistance).rect;
        tree.move_to(
            button,
            at.x0,
            at.y1 + 1 + (at.y0 - tree.el(self.quit).rect.y1 - 1),
        );
        tree.resize_to(button, at.width(), at.height());
        set_label(tree, button, "Edit HUD layout");
    }

    /// Whether the Edit HUD layout button was pressed since the last call.
    pub fn take_edit(&mut self) -> bool {
        std::mem::take(&mut self.edit)
    }

    /// Whether the In-Game Wiki button was pressed since the last call.
    pub fn take_wiki(&mut self) -> bool {
        std::mem::take(&mut self.wiki)
    }

    pub fn confirm_context(&self) -> u32 {
        self.confirm
    }

    pub fn quit_context(&self) -> u32 {
        self.quit_context
    }

    pub fn logout_context(&self) -> u32 {
        self.logout_context
    }

    /// Follows the field's visibility (the toolbar shows and hides it); true when it was shown
    /// and is hidden now, which is `Show(0)`: the caller then runs [`OptionPanel::close`].
    pub fn closing(&mut self, tree: &UiTree) -> bool {
        let shown = tree.get(self.field).is_some_and(|e| e.visible);
        let closing = self.shown && !shown;
        self.shown = shown;
        closing
    }

    fn ask(
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
        id: &str,
    ) -> u32 {
        let text = strings.render_string(&UiString::ui(tokenize(id), Vec::new()));
        dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false)
    }

    /// `gmOptionUI::Show(0)` after the field hid: changes in a page ask the confirmation
    /// (`MakeConfirmDialog`), then the leave and quit questions close.
    pub fn close(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
        changed: bool,
    ) {
        if changed {
            self.confirm = Self::ask(tree, dialogs, strings, "ID_Option_ConfirmText");
        }
        dialogs.close_dialog(tree, self.quit_context);
        self.quit_context = 0;
        dialogs.close_dialog(tree, self.logout_context);
        self.logout_context = 0;
        self.update_buttons(tree);
    }

    /// `gmOptionUI::UpdateButtons`: the three buttons are ghosted while a quit or leave question is up.
    fn update_buttons(&self, tree: &mut UiTree) {
        let ghosted = self.quit_context != 0 || self.logout_context != 0;
        for button in [self.leave, self.quit, self.assistance] {
            tree.set_bool(button, BUTTON_GHOSTED, ghosted);
        }
    }

    /// `gmOptionUI::ListenToMessage` (the toolbar handles the close, help and option buttons).
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
        message: &UiMessage,
    ) {
        let id = message.element_id;
        if id == tokenize("DropDownMenu") && message.message == MSG_MOUSE_OVER {
            if let Some(menu) = message.element {
                let field = tree.screen_box(self.field);
                let menu_box = tree.screen_box(menu);
                let down = (menu_box.y0 + menu_box.y1) / 2 < (field.y0 + field.y1) / 2;
                tree.set_bool(menu, tokenize("open_down"), down);
                tree.set_bool(menu, tokenize("open_up"), !down);
            }
            return;
        }
        if message.message == MSG_BUTTON_RELEASE_ON {
            if id == tokenize("LeaveButton") {
                self.logout_context = Self::ask(tree, dialogs, strings, "ID_Option_LogoutText");
                self.update_buttons(tree);
            } else if id == tokenize("QuitButton") {
                self.quit_context = Self::ask(tree, dialogs, strings, "ID_Option_QuitText");
                self.update_buttons(tree);
            } else if id == tokenize("AssistanceButton") {
                self.wiki = true;
            } else if id == tokenize("ModernEditLayoutButton") {
                self.edit = true;
            }
        }
        if id == tokenize("CommandTab") {
            dialogs.close_dialog(tree, self.quit_context);
            self.quit_context = 0;
            dialogs.close_dialog(tree, self.logout_context);
            self.logout_context = 0;
            self.update_buttons(tree);
        }
    }

    /// `gmOptionUI::RecvNotice_CloseConfirmationDialog`: the answers to this panel's questions.
    pub fn dialog_outcomes(
        &mut self,
        tree: &mut UiTree,
        outcomes: &[DialogOutcome],
    ) -> Vec<OptionAction> {
        let mut actions = Vec::new();
        for outcome in outcomes {
            let DialogOutcome::Confirmation { context, yes } = *outcome else {
                continue;
            };
            if context == 0 {
                continue;
            }
            if context == self.confirm {
                self.confirm = 0;
                actions.push(OptionAction::Confirm(yes));
            } else if context == self.quit_context {
                self.quit_context = 0;
                self.update_buttons(tree);
                if yes {
                    actions.push(OptionAction::Logoff);
                }
            } else if context == self.logout_context {
                self.logout_context = 0;
                self.update_buttons(tree);
                if yes {
                    actions.push(OptionAction::EndCharacterSession);
                }
            }
        }
        actions
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_db::ResourceStore;

    use super::*;
    use crate::dialog::DIALOG_CONFIRMATION;
    use crate::{build_gameplay_hud, ToolbarChild};

    struct Text;

    impl UiStrings for Text {
        fn render_string(&self, string: &UiString) -> String {
            format!("{:08x}", string.string_id)
        }
    }

    fn release(name: &str) -> UiMessage {
        UiMessage {
            element_id: tokenize(name),
            element: None,
            message: MSG_BUTTON_RELEASE_ON,
            data: 0,
            item: None,
        }
    }

    /// Leave and Quit ask their questions and ghost the buttons; a yes is the notice; closing
    /// with changes asks to confirm and its answer is `DoConfirm`.
    #[test]
    fn the_questions_and_their_answers() {
        let Some(dataset) = ac2_dat::test_support::env_dataset() else {
            return;
        };
        let store = Arc::new(ResourceStore::new(dataset));
        let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
        let tree = &mut hud.tree;
        let mut dialogs = DialogFactory::new(None);
        let mut panel = hud.option_panel.take().expect("the option panel");

        panel.message(tree, &mut dialogs, &Text, &release("LeaveButton"));
        let leave = panel.logout_context();
        assert_ne!(leave, 0);
        assert_eq!(dialogs.shown(), Some((leave, DIALOG_CONFIRMATION)));
        assert!(tree.bool_attribute(panel.leave, BUTTON_GHOSTED));
        assert!(tree.bool_attribute(panel.quit, BUTTON_GHOSTED));
        // The command tab closes the questions and unghosts the buttons.
        panel.message(tree, &mut dialogs, &Text, &release("CommandTab"));
        assert_eq!(panel.logout_context(), 0);
        assert!(!tree.bool_attribute(panel.leave, BUTTON_GHOSTED));

        panel.message(tree, &mut dialogs, &Text, &release("QuitButton"));
        let quit = panel.quit_context();
        dialogs.answer(tree, quit, true);
        let outcomes = dialogs.take_outcomes();
        assert_eq!(
            panel.dialog_outcomes(tree, &outcomes),
            vec![OptionAction::Logoff]
        );
        assert!(!tree.bool_attribute(panel.quit, BUTTON_GHOSTED));

        panel.message(tree, &mut dialogs, &Text, &release("LeaveButton"));
        let leave = panel.logout_context();
        dialogs.answer(tree, leave, false);
        let outcomes = dialogs.take_outcomes();
        assert!(panel.dialog_outcomes(tree, &outcomes).is_empty());

        panel.message(tree, &mut dialogs, &Text, &release("LeaveButton"));
        let leave = panel.logout_context();
        dialogs.answer(tree, leave, true);
        let outcomes = dialogs.take_outcomes();
        assert_eq!(
            panel.dialog_outcomes(tree, &outcomes),
            vec![OptionAction::EndCharacterSession]
        );

        // Shown then hidden: with changes the confirmation asks, without it does not.
        hud.toolbar.show(tree, ToolbarChild::Option, true);
        assert!(!panel.closing(tree));
        hud.toolbar.show(tree, ToolbarChild::Option, false);
        assert!(panel.closing(tree));
        panel.close(tree, &mut dialogs, &Text, false);
        assert_eq!(panel.confirm_context(), 0);
        hud.toolbar.show(tree, ToolbarChild::Option, true);
        panel.closing(tree);
        hud.toolbar.show(tree, ToolbarChild::Option, false);
        assert!(panel.closing(tree));
        panel.close(tree, &mut dialogs, &Text, true);
        let confirm = panel.confirm_context();
        assert_ne!(confirm, 0);
        dialogs.answer(tree, confirm, true);
        let outcomes = dialogs.take_outcomes();
        assert_eq!(
            panel.dialog_outcomes(tree, &outcomes),
            vec![OptionAction::Confirm(true)]
        );
        assert_eq!(panel.confirm_context(), 0);
    }
}
