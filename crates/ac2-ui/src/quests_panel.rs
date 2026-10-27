//! `gmQuestsUI`: the Quest panel on the toolbar. It owns the quest journal (`gmJournalQuestsUI`,
//! [`JournalQuestsUi`]) and the two tabs over its pages: `UnderwayTab` and `FinishedTab` are
//! panel pages (`UIElement_PanelPage`), and a quest notice makes the tab of the page it lands on
//! active (`SetActivePanel`), which closes the other tab. The panel's `Show` reaches the journal
//! (`UIFramework::Show` on its children) and clears the quest button's flash once the panel is
//! open. The toolbar shows and hides the panel's field (`ToolbarUi`); the client calls
//! [`QuestsUi::sync`] with the toolbar's answer, so the journal sees each change of the Quest
//! child (`gmToolbarUI`'s `ManageChildren` and key or button presses).
//!
//! The panel's `QuestHelpButton` (`KeyStone::OpenHelp`) is not modelled; its close button is the
//! toolbar's (`QuestCloseButton`).

use std::sync::Arc;

use ac2_net::protocol::{GmActInfo, GmQuestInfo};

use crate::element::UiTree;
use crate::journal_quests::{find_named, JournalQuestsUi, JournalSource, QuestTab};
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::story_quests::{StoryQuestAction, StoryQuestsUi};
use crate::token::tokenize;

/// `gmQuestsUI`'s state: the journal and the panel's shown state.
#[derive(Debug, Clone)]
pub struct QuestsUi {
    journal: JournalQuestsUi,
    /// `m_storyQuestsUI`: the Story page's act list.
    story: StoryQuestsUi,
    /// The panel's own shown state (`UIFramework::IsShown`).
    shown: bool,
}

impl QuestsUi {
    /// `gmQuestsUI::gmQuestsUI`: the journal made over the toolbar's pages; the panel starts
    /// hidden (`Show(0)`).
    pub fn create(tree: &UiTree, toolbar: &Arc<LayoutDesc>) -> Self {
        Self {
            journal: JournalQuestsUi::create(tree, toolbar),
            story: StoryQuestsUi::create(tree, toolbar),
            shown: false,
        }
    }

    /// The Story page (`gmStoryQuestsUI`).
    pub fn story(&self) -> &StoryQuestsUi {
        &self.story
    }

    /// The Story page's notice (`gmStoryQuestsUI::RecvNotice_UpdateStoryQuests`): the Story tab
    /// becomes active when the panel was hidden.
    pub fn recv_story_notice(&mut self, tree: &mut UiTree, code: u32, act: u32) {
        if self.story.recv_notice(tree, code, act, self.shown) {
            self.set_active_panel(tree, QuestTab::Story);
        }
    }

    /// The global loop's Story page reload (`gmStoryQuestsUI::ListenToMessage` 0xffff0005): when
    /// one is pending, `acts` (the script's list) lays the page out.
    pub fn story_global_loop(
        &mut self,
        tree: &mut UiTree,
        source: &dyn JournalSource,
        acts: impl FnOnce() -> Vec<GmActInfo>,
    ) {
        if self.story.take_reload() {
            let acts = acts();
            self.story.reload(tree, source, acts);
        }
    }

    /// A press on the Story page (see [`StoryQuestsUi::message`]).
    pub fn story_message(
        &mut self,
        tree: &UiTree,
        message: &UiMessage,
    ) -> Option<StoryQuestAction> {
        self.story.message(tree, message)
    }

    /// The journal (`gmJournalQuestsUI`).
    pub fn journal(&self) -> &JournalQuestsUi {
        &self.journal
    }

    /// The journal, for the notices, the clicks and the global loop.
    pub fn journal_mut(&mut self) -> &mut JournalQuestsUi {
        &mut self.journal
    }

    /// Whether the panel is shown.
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// `gmQuestsUI::Show`: `UIFramework::Show` reaches the journal (its updated quest closes when
    /// the panel closes), and the quest button's flash clears when the panel opens (the toolbar
    /// shows the panel's field).
    pub fn show(&mut self, tree: &mut UiTree, shown: bool) {
        self.shown = shown;
        self.journal.show(shown);
        self.story.show(shown);
        if shown {
            if let Some(button) = find_named(tree, "QuestButton") {
                tree.set_state(button, tokenize("unchanged"));
            }
        }
    }

    /// The toolbar's answer for the Quest child, each frame: a change calls [`QuestsUi::show`].
    pub fn sync(&mut self, tree: &mut UiTree, toolbar_shown: bool) {
        if toolbar_shown != self.shown {
            self.show(tree, toolbar_shown);
        }
    }

    /// `gmJournalQuestsUI::RecvNotice_UpdateQuests` and the tabs it makes active
    /// (`gmQuestsUI::SetActivePanel`). `quests` is the list code 2 (`LIST`) fills the pages with,
    /// as the script's `InqQuestList` answers it.
    pub fn recv_notice(
        &mut self,
        tree: &mut UiTree,
        code: u32,
        quest: &GmQuestInfo,
        quests: &[GmQuestInfo],
    ) {
        self.journal
            .recv_notice_update_quests(tree, code, quest, quests);
        for tab in self.journal.take_tabs() {
            self.set_active_panel(tree, tab);
        }
    }

    /// `gmQuestsUI::SetActivePanel`: the tab's element (found by name, as `IUIManager::GetElement`
    /// does) starts its use, so it opens and the other tab closes.
    pub fn set_active_panel(&self, tree: &mut UiTree, tab: QuestTab) {
        let name = match tab {
            QuestTab::Underway => "UnderwayTab",
            QuestTab::Finished => "FinishedTab",
            QuestTab::Story => "StoryTab",
        };
        if let Some(element) = find_named(tree, name) {
            tree.start_use_active(element);
        }
    }
}
