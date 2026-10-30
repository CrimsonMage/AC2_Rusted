//! `gmGamePlayUI`'s death panel and the resurrection offer. While the player is dead,
//! `RecvNotice_UpdateDeathState` raises the resurrect dialog (`MakeResurrectDialog` with no
//! resurrecter: the `ID_Death_YouDied` entry of the UI table, dialog type `0x40000003`, topmost).
//! Its button closes it through `CM_UI::SendNotice_CloseResurrectDialog`, and
//! `RecvNotice_CloseResurrectDialog` then asks for the respawn (`gmCWInterface::RequestResurrect`).
//! Alive again closes the dialog (`UpdateDeathState` with 0 closes `_deathContext`).
//!
//! Another player's offer (`RecvNotice_ResurrectionRequestedByOther`) is the same dialog with a
//! resurrecter's name: `MakeResurrectDialog` makes a confirmation (`0x40000001`, the
//! `ID_ResurrectionPrompt` entry of table `0x4100000a`, topmost) in the same `_deathContext`. Its
//! answer comes back as `CM_UI::SendNotice_CloseConfirmationDialog`: yes asks the respawn
//! (`gmCWInterface::RequestResurrect`), no clears the offer (`gmCWInterface::ClearResurrectionRequest`)
//! and makes the death dialog again.

use ac2_net::protocol::StringInfo;
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION, DIALOG_RESURRECT};
use crate::element::UiTree;
use crate::skill_tree::SkillVar;
use crate::token::tokenize;
use crate::ui_string::{UiString, UiStrings};

/// The string table of the resurrection prompt (`MakeResurrectDialog`'s `0x4100000a`).
pub const RESURRECT_PROMPT_TABLE: u32 = 0x4100_000a;

/// The death dialog's text: the `ID_Death_YouDied` entry of the UI table (`MakeResurrectDialog`).
pub fn you_died_string(strings: &dyn UiStrings) -> String {
    strings.render_string(&UiString::ui(tokenize("ID_Death_YouDied"), Vec::new()))
}

/// The offer's prompt as `MakeResurrectDialog` builds it: `ID_ResurrectionPrompt` of table
/// `0x4100000a` with the resurrecter's name as the `player` variable (`token_player`).
pub fn resurrection_prompt(resurrecter: &StringInfo) -> UiString {
    UiString::new(
        tokenize("ID_ResurrectionPrompt"),
        RESURRECT_PROMPT_TABLE,
        vec![(tokenize("player"), SkillVar::Wire(resurrecter.clone()))],
    )
}

/// What the offer's confirmation asked for (`RecvNotice_CloseConfirmationDialog`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfferAnswer {
    /// Not the offer's confirmation.
    None,
    /// No: the offer is cleared (`gmCWInterface::ClearResurrectionRequest`) and the death dialog
    /// is made again (`MakeResurrectDialog` with no resurrecter).
    Declined,
    /// Yes: the offer is cleared and the respawn is asked (`gmCWInterface::RequestResurrect`).
    Accepted,
}

/// `gmGamePlayUI::_deathContext`: the death dialog's or the resurrection offer's context (0 while
/// none is up).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DeathPanel {
    context: u32,
}

impl DeathPanel {
    /// The death dialog's context, or 0.
    pub fn context(&self) -> u32 {
        self.context
    }

    /// `RecvNotice_UpdateDeathState`: dead raises the dialog, alive closes it.
    pub fn notice(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
        dead: bool,
    ) {
        if dead {
            self.make_you_died(tree, dialogs, strings);
        } else {
            self.close(tree, dialogs);
        }
    }

    /// `MakeResurrectDialog` with no resurrecter: the death dialog (closes the one up first).
    pub fn make_you_died(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn UiStrings,
    ) {
        let text = you_died_string(strings);
        self.make(tree, dialogs, &text, DIALOG_RESURRECT);
    }

    /// `RecvNotice_ResurrectionRequestedByOther`: `MakeResurrectDialog` with the resurrecter's
    /// prompt as a confirmation, which replaces the death dialog up in `_deathContext`.
    pub fn recv_resurrection_requested(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        prompt: &str,
    ) {
        self.make(tree, dialogs, prompt, DIALOG_CONFIRMATION);
    }

    /// `RecvNotice_CloseConfirmationDialog`: only the context the panel holds answers. Yes
    /// clears it; no keeps it, and the caller makes the death dialog again.
    pub fn recv_close_confirmation(&mut self, context: u32, yes: bool) -> OfferAnswer {
        // No dialog answers with context 0, so 0 never matches a dialog that is not up.
        if context == 0 || context != self.context {
            return OfferAnswer::None;
        }
        if yes {
            self.context = 0;
            OfferAnswer::Accepted
        } else {
            OfferAnswer::Declined
        }
    }

    /// `RecvNotice_CloseResurrectDialog` for the answers the dialogs gave: when the death
    /// dialog's button closed it, the respawn is asked (`true`).
    pub fn take_resurrect(&mut self, outcomes: &[DialogOutcome]) -> bool {
        let mut asked = false;
        for outcome in outcomes {
            if let DialogOutcome::Resurrect { context } = *outcome {
                if context != 0 && context == self.context {
                    self.context = 0;
                    asked = true;
                }
            }
        }
        asked
    }

    /// `MakeResurrectDialog`: the dialog up in `_deathContext` is closed, then the new one is made
    /// topmost and kept.
    fn make(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory, text: &str, kind: u32) {
        self.close(tree, dialogs);
        self.context = dialogs.make_dialog(tree, text, kind, true);
    }

    /// `DialogFactory::CloseDialog` on the dialog up, if one is (`gmGamePlayUI::EndSession` closes
    /// the death dialog this way too).
    pub fn close(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context != 0 {
            dialogs.close_dialog(tree, self.context);
            self.context = 0;
        }
    }
}

/// `RecvNotice_UpdateDeathState` and `RecvNotice_ResurrectionRequestedByOther` for the frame's
/// notices, in order. The offer's prompt is rendered with the resurrecter's name. Dying calls
/// `gmGamePlayUI::Show(0)` before the dialog is made and coming back alive calls `Show(1)` after
/// it is closed; `Show` does nothing while a please-wait dialog (`wait_up`) or a death dialog
/// (`_deathContext`, checked at that moment) is up, so each call that takes effect pushes its
/// flag onto `shown`.
pub fn death_notices(
    death: &mut DeathPanel,
    tree: &mut UiTree,
    dialogs: &mut Option<DialogFactory>,
    strings: &dyn UiStrings,
    events: &[SessionEvent],
    wait_up: bool,
    shown: &mut Vec<bool>,
) {
    let Some(dialogs) = dialogs.as_mut() else {
        return;
    };
    for event in events {
        match event {
            SessionEvent::Notice(GameplayNotice::UpdateDeathState(dead)) => {
                if *dead {
                    if !wait_up && death.context() == 0 {
                        shown.push(false);
                    }
                    death.notice(tree, dialogs, strings, true);
                } else {
                    death.notice(tree, dialogs, strings, false);
                    if !wait_up {
                        shown.push(true);
                    }
                }
            }
            SessionEvent::Notice(GameplayNotice::ResurrectionRequestedByOther(name)) => {
                let prompt = strings.render_string(&resurrection_prompt(name));
                death.recv_resurrection_requested(tree, dialogs, &prompt);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::DIALOG_CONFIRMATION;

    struct Text;

    impl UiStrings for Text {
        fn render_string(&self, string: &UiString) -> String {
            format!("{:08x}/{:08x}", string.table_enum, string.string_id)
        }
    }

    #[test]
    fn dead_raises_the_resurrect_dialog_and_alive_closes_it() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut panel = DeathPanel::default();
        panel.notice(&mut tree, &mut dialogs, &Text, true);
        let context = panel.context();
        assert_ne!(context, 0);
        assert_eq!(dialogs.shown(), Some((context, DIALOG_RESURRECT)));
        panel.notice(&mut tree, &mut dialogs, &Text, false);
        assert_eq!(panel.context(), 0);
        assert_eq!(dialogs.shown(), None);
    }

    #[test]
    fn dying_hides_the_gameplay_panels_and_coming_back_shows_them() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = Some(DialogFactory::new(None));
        let mut panel = DeathPanel::default();
        let dead = [SessionEvent::Notice(GameplayNotice::UpdateDeathState(true))];
        let alive = [SessionEvent::Notice(GameplayNotice::UpdateDeathState(
            false,
        ))];
        let mut shown = Vec::new();
        death_notices(
            &mut panel,
            &mut tree,
            &mut dialogs,
            &Text,
            &dead,
            false,
            &mut shown,
        );
        assert_eq!(shown, [false]);
        // A second death notice finds the death dialog up: `Show` does nothing.
        death_notices(
            &mut panel,
            &mut tree,
            &mut dialogs,
            &Text,
            &dead,
            false,
            &mut shown,
        );
        assert_eq!(shown, [false]);
        death_notices(
            &mut panel,
            &mut tree,
            &mut dialogs,
            &Text,
            &alive,
            false,
            &mut shown,
        );
        assert_eq!(shown, [false, true]);
        // A please-wait dialog up holds the UI: no call takes effect.
        let mut held = Vec::new();
        death_notices(
            &mut panel,
            &mut tree,
            &mut dialogs,
            &Text,
            &dead,
            true,
            &mut held,
        );
        death_notices(
            &mut panel,
            &mut tree,
            &mut dialogs,
            &Text,
            &alive,
            true,
            &mut held,
        );
        assert!(held.is_empty());
    }

    #[test]
    fn the_button_asks_the_respawn_once_for_the_death_dialog_only() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut panel = DeathPanel::default();
        panel.notice(&mut tree, &mut dialogs, &Text, true);
        let context = panel.context();
        let other = [DialogOutcome::Resurrect {
            context: context + 1,
        }];
        assert!(!panel.take_resurrect(&other));
        let answer = [DialogOutcome::Resurrect { context }];
        assert!(panel.take_resurrect(&answer));
        assert!(!panel.take_resurrect(&answer));
        assert_eq!(panel.context(), 0);
    }

    #[test]
    fn an_offer_is_a_topmost_confirmation_that_replaces_the_death_dialog() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut panel = DeathPanel::default();
        panel.notice(&mut tree, &mut dialogs, &Text, true);
        let death = panel.context();
        panel.recv_resurrection_requested(&mut tree, &mut dialogs, "offer");
        let offer = panel.context();
        assert_ne!(offer, death);
        assert_eq!(dialogs.shown(), Some((offer, DIALOG_CONFIRMATION)));
        assert!(dialogs.queued().is_empty());
    }

    #[test]
    fn the_offer_confirmation_answers_only_its_own_context() {
        let mut tree = UiTree::new(1024, 768);
        let mut dialogs = DialogFactory::new(None);
        let mut panel = DeathPanel::default();
        assert_eq!(panel.recv_close_confirmation(0, true), OfferAnswer::None);
        panel.recv_resurrection_requested(&mut tree, &mut dialogs, "offer");
        let context = panel.context();
        assert_eq!(
            panel.recv_close_confirmation(context + 1, true),
            OfferAnswer::None
        );
        assert_eq!(
            panel.recv_close_confirmation(context, false),
            OfferAnswer::Declined
        );
        assert_eq!(panel.context(), context, "a no keeps the context");
        assert_eq!(
            panel.recv_close_confirmation(context, true),
            OfferAnswer::Accepted
        );
        assert_eq!(panel.context(), 0);
        assert_eq!(
            panel.recv_close_confirmation(context, true),
            OfferAnswer::None
        );
    }

    #[test]
    fn the_prompt_names_the_resurrecter_as_the_player_variable() {
        let name = StringInfo::default();
        let prompt = resurrection_prompt(&name);
        assert_eq!(prompt.string_id, tokenize("ID_ResurrectionPrompt"));
        assert_eq!(prompt.table_enum, RESURRECT_PROMPT_TABLE);
        assert_eq!(prompt.vars.len(), 1);
        assert_eq!(prompt.vars[0].0, tokenize("player"));
    }
}
