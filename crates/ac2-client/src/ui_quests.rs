//! The quest journal (`gmJournalQuestsUI`) driven from the client: the player's quest notices
//! reach it (`CM_Quest::SendNotice_UpdateQuests`, `RecvNotice_UpdateQuests`), and its words are
//! rendered from the player's tables. Its clicks and global loop run from `HudView::frameworks`.

use ac2_net::protocol::StringInfo;
use ac2_ui::{JournalQuestsUi, JournalSource, UiTree};
use ac2_weenie::GameplayNotice;

use crate::ui_sources::ClientPlayer;

impl JournalSource for ClientPlayer<'_> {
    fn render_info(&self, info: &StringInfo) -> String {
        ClientPlayer::render_info(self, info)
    }
}

/// One notice for the journal. The quest list (code 2) is not fetched yet (`InqQuestList` is not
/// asked of the player's script), so it arrives empty.
pub(crate) fn quest_notice(
    journal: &mut JournalQuestsUi,
    tree: &mut UiTree,
    notice: &GameplayNotice,
) {
    if let GameplayNotice::UpdateQuests { code, quest } = notice {
        journal.recv_notice_update_quests(tree, *code, quest, &[]);
        // The Quest panel's tabs (`gmQuestsUI::SetActivePanel`) are not modelled yet.
        journal.take_tabs();
    }
}
