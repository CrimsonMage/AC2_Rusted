//! The quest journal (`gmJournalQuestsUI`) and the Quest panel (`gmQuestsUI`) driven from the
//! client: the player's quest notices reach them (`CM_Quest::SendNotice_UpdateQuests`,
//! `RecvNotice_UpdateQuests`), the list notice (code 2) asks the player's script for the quests
//! (`gmCWInterface::InqQuestList`), and their words are rendered from the player's tables. Their
//! clicks and global loop run from `HudView::frameworks`.

use ac2_net::protocol::{GmQuestInfo, NativeObject, StringInfo};
use ac2_script::host::{IFunArg, IFunReturn, OutValue};
use ac2_script::native::ntype;
use ac2_script::native::NativeObject as ScriptNative;
use ac2_ui::{JournalSource, QuestsUi, UiTree, QUEST_CODE_LIST};
use ac2_weenie::GameplayNotice;

use crate::client::Client;
use crate::ui_sources::ClientPlayer;

/// `ggmCWInterface::CallIFun_InqQuestInfoList` (`gmCWInterface::InqQuestList`): the player's
/// quests, filled into a `ggmQuestInfoList` the call makes.
pub const IFUN_INQ_QUEST_INFO_LIST: u32 = 0x00B4_008E;

impl JournalSource for ClientPlayer<'_> {
    fn render_info(&self, info: &StringInfo) -> String {
        ClientPlayer::render_info(self, info)
    }
}

impl Client {
    /// `gmCWInterface::InqQuestList`: the player's quests from the script. The list native is
    /// filled by the call; a call that fails (no player, or the VM stopped for data) gives none.
    pub(crate) fn inq_quest_list(&mut self) -> Vec<GmQuestInfo> {
        let args = [IFunArg::Filled(ntype::GM_QUEST_INFO_LIST)];
        let Ok(result) = self.query(IFUN_INQ_QUEST_INFO_LIST, &args, IFunReturn::Word) else {
            return Vec::new();
        };
        match result.outs.first() {
            Some(Some(OutValue::Vm(ScriptNative::Wire(list)))) => match &**list {
                NativeObject::GmQuestInfoList(quests) => quests.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }
}

/// One notice for the Quest panel and its journal. The list notice (code 2) asks the script for
/// the quests first, as `RecvNotice_UpdateQuests` case 2 calls `gmCWInterface::InqQuestList`.
pub(crate) fn quest_notice(
    quests: &mut QuestsUi,
    tree: &mut UiTree,
    player: &ClientPlayer,
    notice: &GameplayNotice,
) {
    if let GameplayNotice::UpdateQuests { code, quest } = notice {
        let list = if *code == QUEST_CODE_LIST {
            player.client.borrow_mut().inq_quest_list()
        } else {
            Vec::new()
        };
        quests.recv_notice(tree, *code, quest, &list);
    }
}
