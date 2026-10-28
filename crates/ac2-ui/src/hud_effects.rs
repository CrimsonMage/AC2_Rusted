//! What the client's mapping of the frame's [`crate::HudRequest`]s leaves for the HUD to do to
//! itself: the examination panel opens (`CM_UI::SendNotice_OpenExaminationPanel`), help topics
//! open (`KeyStone::OpenHelp`) and the chat line becomes a tell (`ChatWindow::DoAssistedTell`).

use crate::{Hud, ToolbarChild, UiTree};

/// The effects of a frame's requests on the HUD, applied by [`Hud::apply_effects`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HudEffects {
    /// The examination panel opens.
    pub open_examination: bool,
    /// The help topics to open (`KeyStone::OpenHelp` tokens), in order.
    pub help: Vec<u32>,
    /// The names the chat line becomes an assisted tell to, in order.
    pub tells: Vec<String>,
}

impl Hud {
    /// `CM_UI::SendNotice_OpenExaminationPanel` (`gmExaminationUI::RecvNotice_OpenExaminationPanel`):
    /// the toolbar shows its examination child.
    pub fn open_examination(&self, tree: &mut UiTree, dirty: &mut bool) {
        if let Some(toolbar) = &self.toolbar {
            toolbar.show(tree, ToolbarChild::Examination, true);
            if tree.take_changed() {
                *dirty = true;
            }
        }
    }

    /// `KeyStone::OpenHelp(token)`: the topic's page in the help window (token 0 is the default
    /// page). The window reads the pages it already holds; no file is read here.
    pub fn open_help(&mut self, tree: &mut UiTree, dirty: &mut bool, token: u32) {
        let Some(help) = self.help.as_mut() else {
            return;
        };
        help.open(tree, self.help_pages.as_ref(), token);
        *dirty = true;
    }

    /// `ChatWindow::DoAssistedTell`: the entry line becomes a tell to `target`.
    pub fn assisted_tell(&mut self, tree: &mut UiTree, dirty: &mut bool, target: &str) {
        if let Some(chat) = self.chat.as_mut() {
            chat.assisted_tell(tree, target);
            if tree.take_changed() {
                *dirty = true;
            }
        }
    }

    /// Applies the effects of a frame's requests: the examination panel, the help topics, then
    /// the tells (`FellowshipHelper::ListenToMessage`: a member's name into the chat line).
    pub fn apply_effects(&mut self, tree: &mut UiTree, dirty: &mut bool, effects: HudEffects) {
        if effects.open_examination {
            self.open_examination(tree, dirty);
        }
        for token in effects.help {
            self.open_help(tree, dirty, token);
        }
        for name in effects.tells {
            self.assisted_tell(tree, dirty, &name);
        }
    }
}
