//! The client's side of the Quest panel (`ac2_ui::QuestsUi`): the player's script answers the quest
//! and act lists (`gmCWInterface::InqQuestList`, `InqActInfoList`), and a Story press plays its
//! scene through the client. The panel's notices and clicks are in `ac2_ui`.

use ac2_net::protocol::{GmActInfo, GmQuestInfo, NativeObject, StringInfo};
use ac2_script::host::{IFunArg, IFunReturn, OutValue};
use ac2_script::native::ntype;
use ac2_script::native::NativeObject as ScriptNative;
use ac2_ui::{JournalSource, QuestSource};
use ac2_weenie::SceneInfo;

use crate::client::Client;
use crate::ui_sources::ClientPlayer;

/// `ggmCWInterface::CallIFun_InqQuestInfoList` (`gmCWInterface::InqQuestList`): the player's
/// quests, filled into a `ggmQuestInfoList` the call makes.
pub const IFUN_INQ_QUEST_INFO_LIST: u32 = 0x00B4_008E;

/// `ggmCWInterface::CallIFun_InqActInfoList` (`gmCWInterface::InqActInfoList`): the story's acts,
/// filled into a `List<gmActInfo>` the call makes.
pub const IFUN_INQ_ACT_INFO_LIST: u32 = 0x00B4_008F;

impl JournalSource for ClientPlayer<'_> {
    fn render_info(&self, info: &StringInfo) -> String {
        ClientPlayer::render_info(self, info)
    }
}

impl QuestSource for ClientPlayer<'_> {
    fn quest_list(&self) -> Vec<GmQuestInfo> {
        self.client.borrow_mut().inq_quest_list()
    }

    fn act_list(&self) -> Vec<GmActInfo> {
        self.client.borrow_mut().inq_act_info_list()
    }

    fn play_story_scene(&self, scene: SceneInfo) {
        self.client.borrow_mut().play_story_scene(scene);
    }
}

impl Client {
    /// `gmCWInterface::InqActInfoList`: the acts the Story page lists, from the script. A call that
    /// fails (no player, or the VM stopped for data) gives none.
    pub(crate) fn inq_act_info_list(&mut self) -> Vec<GmActInfo> {
        let args = [IFunArg::Filled(ntype::GM_ACT_INFO_LIST)];
        let Ok(result) = self.query(IFUN_INQ_ACT_INFO_LIST, &args, IFunReturn::Word) else {
            return Vec::new();
        };
        match result.outs.first() {
            Some(Some(OutValue::Vm(ScriptNative::Wire(list)))) => match &**list {
                NativeObject::GmActInfoList(acts) => acts.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

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
