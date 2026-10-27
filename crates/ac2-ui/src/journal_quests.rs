//! `gmJournalQuestsUI`: the quest journal's two pages, `UnderwayPage` and `FinishedPage`, each a
//! list of `QuestTemplate` rows. A row shows the quest's name and time left (`journal_time`);
//! clicked, it expands to the quest's description, journal entry and challenge level in a
//! `QuestExtraText`. The player's quests arrive as `CM_Quest::SendNotice_UpdateQuests`
//! (`RecvNotice_UpdateQuests`): a quest with underway status is kept on the underway list and one
//! failed or completed on the finished list, each newest bestowal first (`insert_sort`). Changes
//! wait for the global loop, which lays the rows out (`Update`); the quest the last notice
//! changed opens and scrolls into view.
//!
//! The Quest panel (`gmQuestsUI`, `quests_panel`) owns this journal: it asks the script for the
//! list when the login notice comes (code 2), makes the tab the notice names active
//! ([`QuestTab`] requests from [`JournalQuestsUi::take_tabs`]) and reports the panel's `Show`.
//! Not modelled yet: the relayout after an extra text downloads.

use std::sync::Arc;

use ac2_net::protocol::{GmQuestInfo, StringInfo};

use crate::element::UiTree;
use crate::journal_time::QuestTimeDisplay;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};
use crate::ui_string::{UiString, UiStrings};

/// `gmQuestInfo::_questStatus`: underway (`GetStatus` 0x10000000, the status the time left and
/// the extra text read), failed (0x20000000) and completed (0x30000000).
pub const STATUS_UNDERWAY: u32 = 0x1000_0000;
pub const STATUS_FAILED: u32 = 0x2000_0000;
pub const STATUS_COMPLETED: u32 = 0x3000_0000;

/// `RecvNotice_UpdateQuests`' codes: one quest updated, the whole list, one added, one removed.
pub const CODE_UPDATE: u32 = 0x4000_0001;
pub const CODE_LIST: u32 = 0x4000_0002;
pub const CODE_ADD: u32 = 0x4000_0003;
pub const CODE_REMOVE: u32 = 0x4000_0004;

/// `gmJournalQuestsUI::m_monthTimeMin`, `m_dayTimeMin`, `m_minuteTimeMin` (the constructor's 1, 1,
/// 60).
const TIME_THRESHOLDS: (i64, i64, i64) = (1, 1, 60);

/// The Quest panel's pages (`gmQuestsUI::s_UnderwayTab`, `s_FinishedTab`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestTab {
    Underway,
    Finished,
    /// The Story page (`gmStoryQuestsUI`, `story_quests`).
    Story,
}

/// The source of the strings the journal shows.
pub trait JournalSource: UiStrings {
    /// A wire `StringInfo` (a quest's name, description or journal entry) rendered.
    fn render_info(&self, info: &StringInfo) -> String;
}

/// `gmJournalQuestsUI::QuestDescriptor`: a quest, its row, extra text, time display and flags.
#[derive(Debug, Clone)]
pub(crate) struct QuestDescriptor {
    pub quest: GmQuestInfo,
    /// `m_uiElement`: the `QuestTemplate` row.
    row: Option<usize>,
    /// `m_uiExtraText`: the `QuestExtraText` under an expanded row.
    extra: Option<usize>,
    /// `m_timeDisplay`.
    time: Option<QuestTimeDisplay>,
    /// `m_updated`: changed since the page last laid it out (its name and time flash).
    pub updated: bool,
    /// `m_expanded`.
    pub expanded: bool,
}

impl QuestDescriptor {
    /// `QuestDescriptor::QuestDescriptor` of a quest with no row yet.
    pub(crate) fn new(quest: GmQuestInfo) -> Self {
        Self {
            quest,
            row: None,
            extra: None,
            time: None,
            updated: false,
            expanded: false,
        }
    }
}

/// `gmJournalQuestsUI`'s state.
#[derive(Debug, Clone)]
pub struct JournalQuestsUi {
    layout: Arc<LayoutDesc>,
    underway_list: Option<usize>,
    finished_list: Option<usize>,
    underway: Vec<QuestDescriptor>,
    finished: Vec<QuestDescriptor>,
    /// `m_update`: the lists wait for the global loop to lay them out.
    update: bool,
    /// `m_updatedQuestID`: the quest the last notice changed.
    updated_quest: u32,
    tabs: Vec<QuestTab>,
    /// The journal's own shown state (`UIFramework::IsShown`).
    shown: bool,
}

impl JournalQuestsUi {
    /// `gmJournalQuestsUI::gmJournalQuestsUI`: the two lists, found under their pages in the
    /// toolbar's tree. A missing page leaves its list out.
    pub fn create(tree: &UiTree, layout: &Arc<LayoutDesc>) -> Self {
        Self {
            layout: Arc::clone(layout),
            underway_list: page_list(tree, "UnderwayPage", "UnderwayQuestListBox"),
            finished_list: page_list(tree, "FinishedPage", "FinishedQuestListBox"),
            underway: Vec::new(),
            finished: Vec::new(),
            update: false,
            updated_quest: 0,
            tabs: Vec::new(),
            shown: false,
        }
    }

    /// The journal's shown state (`UIFramework::IsShown`).
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// The Quest panel's tab requests since the last call (`gmQuestsUI::SetActivePanel`).
    pub fn take_tabs(&mut self) -> Vec<QuestTab> {
        std::mem::take(&mut self.tabs)
    }

    /// The underway quests' ids, newest bestowal first.
    pub fn underway_ids(&self) -> Vec<u32> {
        self.underway.iter().map(|q| q.quest.id).collect()
    }

    /// The finished quests' ids, newest bestowal first.
    pub fn finished_ids(&self) -> Vec<u32> {
        self.finished.iter().map(|q| q.quest.id).collect()
    }

    fn find_mut(&mut self, id: u32) -> Option<&mut QuestDescriptor> {
        self.underway
            .iter_mut()
            .chain(self.finished.iter_mut())
            .find(|q| q.quest.id == id)
    }

    /// `gmJournalQuestsUI::RecvNotice_UpdateQuests`. `quest` is the notice's quest; `quests` is the
    /// list code 2 (`LIST`) fills the pages with, as `InqQuestList` answers it.
    pub fn recv_notice_update_quests(
        &mut self,
        tree: &mut UiTree,
        code: u32,
        quest: &GmQuestInfo,
        quests: &[GmQuestInfo],
    ) {
        // The previous updated quest closes; this one is the one to open.
        if let Some(previous) = self.find_mut(self.updated_quest) {
            previous.expanded = false;
        }
        self.updated_quest = quest.id;
        self.update = false;
        let mut flash = false;
        match code {
            CODE_UPDATE => {
                flash = true;
                match quest.status {
                    STATUS_UNDERWAY => {
                        if let Some(desc) =
                            self.underway.iter_mut().find(|q| q.quest.id == quest.id)
                        {
                            desc.updated = true;
                            desc.quest = quest.clone();
                        } else {
                            self.remove_quest_from(tree, QuestTab::Finished, quest.id);
                            self.add_new(QuestTab::Underway, quest);
                        }
                        self.tabs.push(QuestTab::Underway);
                    }
                    STATUS_FAILED | STATUS_COMPLETED => {
                        self.remove_quest_from(tree, QuestTab::Underway, quest.id);
                        self.add_new(QuestTab::Finished, quest);
                        self.tabs.push(QuestTab::Finished);
                    }
                    _ => {}
                }
            }
            CODE_LIST => {
                for entry in quests {
                    match entry.status {
                        STATUS_UNDERWAY => {
                            insert_unique_sort(
                                &mut self.underway,
                                QuestDescriptor::new(entry.clone()),
                            );
                        }
                        STATUS_FAILED | STATUS_COMPLETED => {
                            insert_unique_sort(
                                &mut self.finished,
                                QuestDescriptor::new(entry.clone()),
                            );
                        }
                        _ => {}
                    }
                }
                self.tabs.push(QuestTab::Underway);
            }
            CODE_ADD => {
                flash = true;
                self.add_new(QuestTab::Underway, quest);
                self.tabs.push(QuestTab::Underway);
            }
            CODE_REMOVE => {
                let removed = self.remove_quest_from(tree, QuestTab::Underway, quest.id)
                    || self.remove_quest_from(tree, QuestTab::Finished, quest.id);
                if removed && self.updated_quest == quest.id {
                    self.updated_quest = 0;
                }
            }
            _ => {}
        }
        if matches!(code, CODE_UPDATE | CODE_LIST | CODE_ADD) {
            self.update = true;
        }
        // `m_update` and the quest button's flash: the rows wait for the global loop.
        if flash && !self.shown {
            if let Some(button) = find_named(tree, "QuestButton") {
                tree.set_state(button, tokenize("changed"));
            }
        }
    }

    /// A new quest on `page`, marked updated (`insert_unique_sort`).
    fn add_new(&mut self, page: QuestTab, quest: &GmQuestInfo) {
        let mut desc = QuestDescriptor::new(quest.clone());
        desc.updated = true;
        match page {
            QuestTab::Underway => insert_unique_sort(&mut self.underway, desc),
            QuestTab::Finished => insert_unique_sort(&mut self.finished, desc),
            // The Story page holds acts, not quests.
            QuestTab::Story => false,
        };
    }

    /// `RemoveQuestElement` and the list removal for the quest `id` on `page`; the quest's id is
    /// cleared as the updated one when it was that one (`RecvNotice_UpdateQuests` case 4 only).
    fn remove_quest_from(&mut self, tree: &mut UiTree, page: QuestTab, id: u32) -> bool {
        let (list, quests) = match page {
            QuestTab::Underway => (self.underway_list, &mut self.underway),
            QuestTab::Finished => (self.finished_list, &mut self.finished),
            QuestTab::Story => return false,
        };
        let Some(at) = position(quests, id) else {
            return false;
        };
        let mut desc = quests.remove(at);
        drop_row(tree, list, &mut desc);
        true
    }

    /// `Update` (on the global loop after a notice): both pages laid out.
    fn update_pages(&mut self, tree: &mut UiTree, source: &dyn JournalSource, now: f64) {
        let (Some(underway), Some(finished)) = (self.underway_list, self.finished_list) else {
            return;
        };
        let updated = self.updated_quest;
        let mut rows = Rows {
            tree,
            source,
            layout: Arc::clone(&self.layout),
            now,
        };
        rows.lay_out(underway, &mut self.underway, updated);
        rows.lay_out(finished, &mut self.finished, updated);
    }

    /// `gmJournalQuestsUI::ListenToMessage` for the global loop: the pages laid out when changed,
    /// and each time display counted down.
    pub fn global_loop(&mut self, tree: &mut UiTree, source: &dyn JournalSource, now: f64) {
        if self.update {
            self.update = false;
            self.update_pages(tree, source, now);
        }
        for desc in self.underway.iter_mut().chain(self.finished.iter_mut()) {
            if let Some(time) = desc.time.as_mut() {
                time.global_loop(tree, source, now);
            }
        }
    }

    /// `gmJournalQuestsUI::ListenToMessage` for a button or element: a `QuestTemplate` clicked
    /// opens or closes its quest.
    pub fn message(&mut self, tree: &UiTree, message: &UiMessage) {
        if message.element_id != tokenize("QuestTemplate")
            || message.message != MSG_BUTTON_RELEASE_ON
        {
            return;
        }
        let Some(element) = message.element else {
            return;
        };
        self.updated_quest = 0;
        let id = match tree
            .get(element)
            .and_then(|e| e.attribute(tokenize("QuestIDToken")))
        {
            Some(AttributeValue::Id(id)) => *id,
            _ => 0,
        };
        if let Some(desc) = self.find_mut(id) {
            desc.updated = false;
            desc.expanded = !desc.expanded;
            self.update = true;
        }
    }

    /// `gmJournalQuestsUI::Show` (`UIFramework::Show` from the Quest panel): closing the journal
    /// with an updated quest pending clears it. Retail tests the underway list's visibility
    /// before the change; the journal's own shown state is that visibility, since the panel
    /// shows and hides the list only through this call.
    pub fn show(&mut self, shown: bool) {
        let was_visible = self.underway_list.is_some() && self.shown;
        self.shown = shown;
        if was_visible && !shown && self.updated_quest != 0 {
            if let Some(desc) = self.find_mut(self.updated_quest) {
                desc.expanded = false;
                desc.updated = false;
            }
            self.updated_quest = 0;
            self.update = true;
        }
    }
}

/// The row-making half of the journal: the elements, the list boxes and the strings.
struct Rows<'a> {
    tree: &'a mut UiTree,
    source: &'a dyn JournalSource,
    layout: Arc<LayoutDesc>,
    now: f64,
}

impl Rows<'_> {
    fn create(&mut self, name: &str) -> Option<usize> {
        let desc = self.layout.element(tokenize(name))?.clone();
        self.tree
            .create_element(Arc::clone(&self.layout), desc, false)
    }

    /// `gmJournalQuestsUI::Update` for one page: each quest's row made or updated, in order, and
    /// the updated quest opened and scrolled to.
    fn lay_out(&mut self, list: usize, quests: &mut [QuestDescriptor], updated: u32) {
        self.tree.list_multi_op_start(list);
        let mut scroll = None;
        for (index, desc) in quests.iter_mut().enumerate() {
            if desc.quest.id == updated {
                desc.expanded = true;
                scroll = Some(index);
            }
            if desc.row.is_none() {
                self.add(list, desc, index);
            } else {
                self.update(list, desc);
            }
        }
        self.tree.list_multi_op_complete(list);
        if let Some(row) = scroll {
            self.tree.list_scroll_to_row(list, row as i32);
        }
    }

    /// `gmJournalQuestsUI::AddQuestElement`: the row made at `location`, its time, its state, the
    /// extra text when expanded, and its name (flashing when updated).
    fn add(&mut self, list: usize, desc: &mut QuestDescriptor, location: usize) {
        let Some(row) = self.create("QuestTemplate") else {
            return;
        };
        desc.row = Some(row);
        self.tree
            .set_attribute_id(row, tokenize("QuestIDToken"), desc.quest.id);
        self.tree.list_insert_item(list, row, location);
        let time = self.tree.child_recursive(row, tokenize("QuestTime"));
        self.do_quest_time(desc, time);
        if desc.expanded {
            self.tree.set_state(row, tokenize("highlight"));
            if let Some(text) = extra_text(&desc.quest) {
                let Some(extra) = self.create("QuestExtraText") else {
                    return;
                };
                let rendered = self.source.render_string(&text);
                self.tree.set_text(extra, &rendered);
                self.tree.list_insert_item(list, extra, location + 1);
                desc.extra = Some(extra);
            }
        } else {
            self.tree.set_state(row, tokenize("normal"));
        }
        if let Some(name) = self.tree.child_recursive(row, tokenize("QuestName")) {
            if desc.updated {
                flash(self.tree, time);
                flash(self.tree, Some(name));
            }
            let text = self.source.render_info(&desc.quest.name);
            self.tree.set_text(name, &text);
        }
    }

    /// `gmJournalQuestsUI::UpdateQuestElement`: the row's state, its extra text made or dropped
    /// with the expansion, and the flash when updated.
    fn update(&mut self, list: usize, desc: &mut QuestDescriptor) {
        let Some(row) = desc.row else {
            return;
        };
        if desc.expanded {
            self.tree.set_state(row, tokenize("highlight"));
            if let Some(text) = extra_text(&desc.quest) {
                let extra = match desc.extra {
                    Some(extra) => extra,
                    None => {
                        let Some(extra) = self.create("QuestExtraText") else {
                            return;
                        };
                        let at = self
                            .tree
                            .list_what_num(list, row)
                            .map_or(0, |index| index + 1);
                        self.tree.list_insert_item(list, extra, at);
                        desc.extra = Some(extra);
                        extra
                    }
                };
                let rendered = self.source.render_string(&text);
                self.tree.set_text(extra, &rendered);
            }
        } else {
            self.tree.set_state(row, tokenize("normal"));
            if let Some(extra) = desc.extra.take() {
                remove_item(self.tree, Some(list), extra);
            }
        }
        if desc.updated {
            if let Some(name) = self.tree.child_recursive(row, tokenize("QuestName")) {
                let time = self.tree.child_recursive(row, tokenize("QuestTime"));
                flash(self.tree, time);
                flash(self.tree, Some(name));
            }
        }
    }

    /// `gmJournalQuestsUI::DoQuestTime`: the time left counted down when the quest has some,
    /// else the words for no time or for a maxed quest.
    fn do_quest_time(&mut self, desc: &mut QuestDescriptor, time: Option<usize>) {
        let Some(time) = time else {
            return;
        };
        let quest = &desc.quest;
        let maxed = quest.status != STATUS_UNDERWAY && quest.maxed_out;
        let word = if maxed {
            "ID_Quest_QuestMaxed"
        } else {
            // `GetSecondsRemaining`: the time left while underway, else the time until a retry.
            let seconds = if quest.status == STATUS_UNDERWAY {
                quest.seconds_remaining
            } else {
                quest.seconds_until_retry
            } as i64;
            if seconds > 0 {
                let end = self.now + seconds as f64;
                desc.time = Some(QuestTimeDisplay::init(
                    self.tree,
                    self.source,
                    time,
                    end,
                    TIME_THRESHOLDS,
                    self.now,
                ));
                return;
            }
            "ID_Quest_TimeNone"
        };
        let rendered = self
            .source
            .render_string(&UiString::ui(tokenize(word), Vec::new()));
        self.tree.set_text(time, &rendered);
    }
}

/// `gmJournalQuestsUI::InqExtraText`: the description, journal entry and challenge level while
/// underway; the completed or failed words after; none for other statuses.
fn extra_text(quest: &GmQuestInfo) -> Option<UiString> {
    match quest.status {
        STATUS_UNDERWAY => {
            let level = if quest.challenge_level == -999 {
                SkillVar::Info(Box::new(UiString::ui(
                    tokenize("ID_Quest_ChallengeLevelAny"),
                    Vec::new(),
                )))
            } else {
                SkillVar::Int(i64::from(quest.challenge_level))
            };
            Some(UiString::ui(
                tokenize("ID_Quest_Description"),
                vec![
                    (tokenize("DESC"), SkillVar::Wire(quest.description.clone())),
                    (tokenize("JOURNAL"), SkillVar::Wire(quest.journal.clone())),
                    (tokenize("LEVEL"), level),
                ],
            ))
        }
        STATUS_FAILED => Some(UiString::ui(tokenize("ID_Quest_Failed"), Vec::new())),
        STATUS_COMPLETED => Some(UiString::ui(tokenize("ID_Quest_Completed"), Vec::new())),
        _ => None,
    }
}

/// `List<QuestDescriptor>::insert_unique_sort`: nothing when the id is listed, else the quest
/// before the first one bestowed earlier (`insert_sort`, newest first).
pub(crate) fn insert_unique_sort(list: &mut Vec<QuestDescriptor>, desc: QuestDescriptor) -> bool {
    if position(list, desc.quest.id).is_some() {
        return false;
    }
    let at = list
        .iter()
        .position(|q| desc.quest.bestowal > q.quest.bestowal)
        .unwrap_or(list.len());
    list.insert(at, desc);
    true
}

fn position(list: &[QuestDescriptor], id: u32) -> Option<usize> {
    list.iter().position(|q| q.quest.id == id)
}

/// `RemoveQuestElement`: the row, its extra text and the time display come out of the page.
fn drop_row(tree: &mut UiTree, list: Option<usize>, desc: &mut QuestDescriptor) {
    let Some(row) = desc.row.take() else {
        return;
    };
    if let Some(extra) = desc.extra.take() {
        remove_item(tree, list, extra);
    }
    desc.time = None;
    remove_item(tree, list, row);
}

/// `MultiOpRemoveItem` and the element's release (`(**vtable)(1)`).
fn remove_item(tree: &mut UiTree, list: Option<usize>, item: usize) {
    if !list.is_some_and(|list| tree.list_delete_item(list, item)) {
        tree.delete(item);
    }
}

/// `SetState(token_changed)` on an element, if there is one.
fn flash(tree: &mut UiTree, element: Option<usize>) {
    if let Some(element) = element {
        tree.set_state(element, tokenize("changed"));
    }
}

/// `IUIManager::GetElement`: the first root (or descendant of one) with this name.
pub(crate) fn find_named(tree: &UiTree, name: &str) -> Option<usize> {
    let name = tokenize(name);
    tree.roots().iter().find_map(|&root| {
        if tree.get(root).is_some_and(|e| e.id() == name) {
            Some(root)
        } else {
            tree.child_recursive(root, name)
        }
    })
}

fn page_list(tree: &UiTree, page: &str, list: &str) -> Option<usize> {
    let page = find_named(tree, page)?;
    tree.child_recursive(page, tokenize(list))
}
