//! `gmSocialUI`: the social panel (`SocialField`) of the toolbar. Its buttons show and hide the
//! panel (`SocialButton` toggles it, `SocialCloseButton` hides it) and `SocialHelpButton` opens the
//! social help page. The panel's three pages (allegiance, fellowship, friends) are the toolbar's
//! own panels; this keeps the panel's shown state and the field's visibility.
//!
//! `gmSocialUI::Show(shown)` sets the framework's shown flag (`UIFramework::Show`) and the field's
//! visibility to it. The constructor shows it hidden. Not modelled: the framework's child list
//! (`UIFramework::Show` passes the flag to the child frameworks, which the toolbar's pages do not
//! follow here), and the help page itself (`KeyStone::OpenHelp(ID_Help_Social)`, returned as
//! [`SocialAction::OpenHelp`] for the caller).

use crate::element::UiTree;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};

/// `KeyStone::OpenHelp`'s topic for the social page (the id `gmSocialUI::ListenToMessage` passes).
pub const SOCIAL_HELP_TOPIC: u32 = 0x4100_004E;

/// What the social panel's buttons ask beyond the panel's own state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialAction {
    /// `KeyStone::OpenHelp(topic)`.
    OpenHelp(u32),
}

/// `gmSocialUI`'s state: the field and whether the panel is shown.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SocialPanel {
    /// `_socialField`.
    field: Option<usize>,
    /// `gmSocialUI::Shown` (the framework's shown flag).
    shown: bool,
}

impl SocialPanel {
    /// `gmSocialUI::gmSocialUI`: the field found in the toolbar's roots, then `Show(0)`.
    pub fn create(tree: &mut UiTree) -> Self {
        let field = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, tokenize("SocialField")));
        let mut panel = Self {
            field,
            shown: false,
        };
        panel.show(tree, false);
        panel
    }

    /// `gmSocialUI::Shown`.
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// `gmSocialUI::Show`: the shown flag, and the field's visibility with it.
    pub fn show(&mut self, tree: &mut UiTree, shown: bool) {
        self.shown = shown;
        if let Some(field) = self.field {
            tree.set_visible(field, shown);
        }
    }

    /// `gmSocialUI::ListenToMessage`: the toggle, the close and the help button, on a release.
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage) -> Option<SocialAction> {
        if message.message != MSG_BUTTON_RELEASE_ON {
            return None;
        }
        if message.element_id == tokenize("SocialButton") {
            let shown = !self.shown;
            self.show(tree, shown);
        } else if message.element_id == tokenize("SocialCloseButton") {
            self.show(tree, false);
        } else if message.element_id == tokenize("SocialHelpButton") {
            return Some(SocialAction::OpenHelp(SOCIAL_HELP_TOPIC));
        }
        None
    }
}
