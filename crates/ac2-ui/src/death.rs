//! `gmGamePlayUI`'s death panel: while the player is dead, `RecvNotice_UpdateDeathState` raises
//! the resurrect dialog (`MakeResurrectDialog` with no resurrecter: the `ID_Death_YouDied` entry
//! of the UI table, dialog type `0x40000003`, topmost). Its button closes it through
//! `CM_UI::SendNotice_CloseResurrectDialog`, and `RecvNotice_CloseResurrectDialog` then asks for the
//! respawn (`gmCWInterface::RequestResurrect`). Alive again closes the dialog (`UpdateDeathState`
//! with 0 closes `_deathContext`).

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_RESURRECT};
use crate::element::UiTree;
use crate::token::tokenize;
use crate::ui_string::{UiString, UiStrings};

/// The death dialog's text: the `ID_Death_YouDied` entry of the UI table (`MakeResurrectDialog`).
pub fn you_died_string(strings: &dyn UiStrings) -> String {
    strings.render_string(&UiString::ui(tokenize("ID_Death_YouDied"), Vec::new()))
}

/// `gmGamePlayUI::_deathContext`: the death dialog's context (0 while none is up).
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
        self.close(tree, dialogs);
        if dead {
            let text = you_died_string(strings);
            self.context = dialogs.make_dialog(tree, &text, DIALOG_RESURRECT, true);
        }
    }

    /// `gmGamePlayUI::RecvNotice_CloseResurrectDialog` for the answers the dialogs gave: when the
    /// death dialog's button closed it, the respawn is asked (`true`).
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

    /// `DialogFactory::CloseDialog` on the death dialog, if one is up.
    fn close(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        if self.context != 0 {
            dialogs.close_dialog(tree, self.context);
            self.context = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
