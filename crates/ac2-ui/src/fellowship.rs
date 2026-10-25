//! `gmFellowshipUI` and its `FellowshipHelper`: the social panel's fellowship page. Without a
//! fellowship the page is in `nofellowship` (name entry, XP sharing toggle, looting menu and
//! Create); with one it is in `fellowship`: the name, the XP sharing and looting indicators, the
//! leader's buttons (Recruit, Dismiss and Disband ghosted for the others), Leave, and one
//! [`FellowInfo`] row per member in `FellowshipListBox`, the leader first. A recruitment
//! invitation is a confirmation dialog.
//!
//! The panel turns the player's buttons and clicks into [`FellowshipRequest`]s; the client runs
//! them (`gmCWInterface::DoFellowship*`, `SetSelection`, `DoAssistedTell`).
//!
//! Not modelled yet: the looting drop-down (`UIElement_Menu`; Create sends retail's default,
//! method 1), tooltips, the radar's mini panel (`FellowMiniPanelTemplate`) and the map markers
//! (`SendNotice_AddMarker` / `RemoveMarker` / `ClearMarkers`).

use crate::dialog::{DialogFactory, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::fellow_info::{Fellow, FellowInfo, FellowStrings};
use crate::layout::LayoutDesc;
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, BUTTON_GHOSTED, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON};

use std::sync::Arc;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// The fellowship as the server describes it (`FellowshipAdaptor`), members in its table's order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FellowshipData {
    pub name: String,
    pub leader: u64,
    /// Experience shared (`m_social`).
    pub social: bool,
    pub looting: i32,
    pub fellows: Vec<Fellow>,
}

/// What the panel asks of the player.
#[derive(Debug, Clone, PartialEq)]
pub enum FellowshipRequest {
    /// `DoFellowshipCreate(name, social, looting)`.
    Create {
        name: String,
        social: bool,
        looting: i32,
    },
    Leave,
    /// `DoFellowshipRecruit` / `DoFellowshipDismiss` of the current selection.
    Recruit,
    Dismiss,
    Disband,
    /// `DoFellowshipRecruitAccept`, `DoFellowshipRecruitDecline(0x138D)`.
    RecruitAccept,
    RecruitDecline,
    /// `SetSelection`.
    Select(u64),
    /// `DoAssistedTell(name)`.
    AssistedTell(String),
}

/// What the panel reads from the player.
pub trait FellowshipSource: FellowStrings {
    fn player(&self) -> u64;
    /// `gmCWInterface::GetFellowXPHash`: each member's share of the experience, if answered.
    fn fellow_xp(&self) -> Option<Vec<(u64, f32)>>;
}

/// `FellowshipHelper`'s state.
#[derive(Debug, Clone)]
struct Helper {
    list: usize,
    /// `_fellowUIHash`, in the order added.
    fellows: Vec<FellowInfo>,
    leader: Option<u64>,
    selection: Option<u64>,
}

/// `gmFellowshipUI`'s state.
#[derive(Debug, Clone)]
pub struct FellowshipUi {
    layout: Arc<LayoutDesc>,
    page: Option<usize>,
    helper: Option<Helper>,
    name: String,
    social: bool,
    looting: i32,
    /// `_recruitContext`.
    recruit_context: u32,
    requests: Vec<FellowshipRequest>,
}

const LOOTING_FIELD_STATES: [&str; 5] = ["method0", "method1", "method2", "method3", "method4"];

impl FellowshipUi {
    /// `gmFellowshipUI::gmFellowshipUI`: the page (`IUIManager` lookup of `FellowshipPage`),
    /// the looting menu, then `SetNoFellowshipState`.
    pub fn create(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> Self {
        let page = tree.roots().iter().find_map(|&r| {
            if tree.get(r).is_some_and(|e| e.id() == id("FellowshipPage")) {
                Some(r)
            } else {
                tree.child_recursive(r, id("FellowshipPage"))
            }
        });
        let mut ui = Self {
            layout: Arc::clone(layout),
            page,
            helper: None,
            name: String::new(),
            social: false,
            looting: 0,
            recruit_context: 0,
            requests: Vec::new(),
        };
        ui.set_no_fellowship_state(tree);
        ui
    }

    pub fn take_requests(&mut self) -> Vec<FellowshipRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The members' rows, top first, and their ids.
    pub fn rows(&self, tree: &UiTree) -> Vec<(u64, usize)> {
        let Some(helper) = &self.helper else {
            return Vec::new();
        };
        tree.list_items(helper.list)
            .iter()
            .filter_map(|&item| {
                helper
                    .fellows
                    .iter()
                    .find(|f| f.root == item)
                    .map(|f| (f.id, item))
            })
            .collect()
    }

    fn page_child(&self, tree: &UiTree, name: &str) -> Option<usize> {
        self.page.and_then(|p| tree.child_recursive(p, id(name)))
    }

    /// `gmFellowshipUI::SetNoFellowshipState`.
    fn set_no_fellowship_state(&mut self, tree: &mut UiTree) {
        let Some(page) = self.page else {
            return;
        };
        tree.set_state(page, id("nofellowship"));
        if let Some(text) = self.page_child(tree, "FellowshipNameText") {
            tree.set_text(text, "");
        }
    }

    /// `gmFellowshipUI::SetFellowshipState`: the page in `fellowship`, the name, the XP
    /// sharing indicator (highlighted when shared, always ghosted) and the looting method's
    /// state.
    fn set_fellowship_state(&mut self, tree: &mut UiTree) {
        let Some(page) = self.page else {
            return;
        };
        if tree.get(page).map(|e| e.state()) != Some(id("fellowship")) {
            tree.set_state(page, id("fellowship"));
        }
        let Some(text) = self.page_child(tree, "FellowshipNameText") else {
            return;
        };
        tree.set_text(text, &self.name);
        let Some(xp) = self.page_child(tree, "FellowshipXPButton") else {
            return;
        };
        tree.set_bool(xp, BUTTON_HIGHLIGHTED, self.social);
        tree.set_bool(xp, BUTTON_GHOSTED, true);
        if let Some(loot) = self.page_child(tree, "FellowshipLootField") {
            if let Some(state) = usize::try_from(self.looting)
                .ok()
                .and_then(|i| LOOTING_FIELD_STATES.get(i))
            {
                tree.set_state(loot, id(state));
            }
        }
    }

    /// `gmFellowshipUI::SetLeaderState`: Disband, Recruit and Dismiss ghosted unless leading.
    fn set_leader_state(&self, tree: &mut UiTree, leader: bool) {
        for button in [
            "FellowshipDisbandButton",
            "FellowshipRecruitButton",
            "FellowshipDismissButton",
        ] {
            let Some(button) = self.page_child(tree, button) else {
                return;
            };
            tree.set_bool(button, BUTTON_GHOSTED, !leader);
        }
    }

    /// `RecvNotice_UpdateFellowship`: the page filled, then a new helper over the list box.
    pub fn update_fellowship(
        &mut self,
        tree: &mut UiTree,
        source: &dyn FellowshipSource,
        data: &FellowshipData,
    ) {
        if self.page.is_none() {
            return;
        }
        self.name = data.name.clone();
        self.social = data.social;
        self.looting = data.looting;
        self.set_fellowship_state(tree);
        self.set_leader_state(tree, data.leader == source.player());
        self.remove_all_fellows(tree);
        self.helper = None;
        let Some(list) = self.page_child(tree, "FellowshipListBox") else {
            return;
        };
        self.helper = Some(Helper {
            list,
            fellows: Vec::new(),
            leader: None,
            selection: None,
        });
        // `FellowshipHelper::AddAllFellows`.
        for fellow in &data.fellows {
            self.add_fellow(tree, source, fellow);
        }
        self.set_leader(tree, data.leader);
        self.update_xp(tree, source);
    }

    /// `RecvNotice_ClearFellowship`.
    pub fn clear_fellowship(&mut self, tree: &mut UiTree) {
        self.remove_all_fellows(tree);
        self.helper = None;
        self.set_no_fellowship_state(tree);
    }

    /// `FellowshipHelper::AddFellow`: a `FellowTemplate` row added to the list.
    fn add_fellow(&mut self, tree: &mut UiTree, strings: &dyn FellowStrings, fellow: &Fellow) {
        let Some(helper) = &mut self.helper else {
            return;
        };
        let Some(desc) = self.layout.element(id("FellowTemplate")).cloned() else {
            return;
        };
        let Some(root) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
            return;
        };
        let info = FellowInfo::new(tree, root, fellow, strings);
        tree.list_add_item(helper.list, root);
        helper.fellows.push(info);
    }

    /// `FellowshipHelper::RemoveAllFellows`.
    fn remove_all_fellows(&mut self, tree: &mut UiTree) {
        if let Some(helper) = &mut self.helper {
            tree.list_flush(helper.list);
            helper.fellows.clear();
        }
    }

    /// `FellowshipHelper::SetLeader`: the leader's row moved to the top.
    fn set_leader(&mut self, tree: &mut UiTree, leader: u64) {
        let Some(helper) = &mut self.helper else {
            return;
        };
        let old = helper.leader.replace(leader);
        let find = |id: Option<u64>| helper.fellows.iter().find(|f| Some(f.id) == id);
        // Retail moves the old leader back up when the new one has no row.
        let Some(row) = find(Some(leader)).or_else(|| find(old)).map(|f| f.root) else {
            return;
        };
        let list = helper.list;
        tree.list_multi_op_start(list);
        if let Some(at) = tree.list_what_num(list, row) {
            tree.list_remove_item(list, at);
        }
        tree.list_insert_item(list, row, 0);
        tree.list_multi_op_complete(list);
    }

    /// `FellowshipHelper::UpdateXP`: each member's share, when the player answers.
    fn update_xp(&self, tree: &mut UiTree, source: &dyn FellowshipSource) {
        let Some(helper) = &self.helper else {
            return;
        };
        let Some(shares) = source.fellow_xp() else {
            return;
        };
        for (fellow, share) in shares {
            let Some(info) = helper.fellows.iter().find(|f| f.id == fellow) else {
                return;
            };
            info.update_percentage(tree, share, source);
        }
    }

    fn fellow_mut(&mut self, fellow: u64) -> Option<&mut FellowInfo> {
        self.helper
            .as_mut()?
            .fellows
            .iter_mut()
            .find(|f| f.id == fellow)
    }

    /// `FellowshipHelper::RecvNotice_UpdateFellow`: a new member's row, or the row updated.
    pub fn update_fellow(
        &mut self,
        tree: &mut UiTree,
        source: &dyn FellowshipSource,
        fellow: &Fellow,
    ) {
        if self.helper.is_none() {
            return;
        }
        match self.fellow_mut(fellow.id) {
            Some(info) => info.update_info(tree, fellow, source),
            None => self.add_fellow(tree, source, fellow),
        }
        self.update_xp(tree, source);
    }

    /// `FellowshipHelper::RecvNotice_ClearFellow` (`RemoveFellow`, then `UpdateXP`).
    pub fn clear_fellow(&mut self, tree: &mut UiTree, source: &dyn FellowshipSource, fellow: u64) {
        let Some(helper) = &mut self.helper else {
            return;
        };
        if let Some(at) = helper.fellows.iter().position(|f| f.id == fellow) {
            let info = helper.fellows.remove(at);
            tree.list_delete_item(helper.list, info.root);
        }
        self.update_xp(tree, source);
    }

    /// `gmFellowshipUI::RecvNotice_UpdateLeader` and the helper's (`SetLeader`).
    pub fn update_leader(&mut self, tree: &mut UiTree, source: &dyn FellowshipSource, leader: u64) {
        if self.helper.is_some() {
            self.set_leader_state(tree, leader == source.player());
        }
        self.set_leader(tree, leader);
    }

    /// `RecvNotice_UpdateFellowHealth` (health and PK damage).
    pub fn update_health(
        &mut self,
        tree: &mut UiTree,
        s: &dyn FellowStrings,
        fellow: u64,
        v: u32,
        pk: u32,
    ) {
        if let Some(info) = self.fellow_mut(fellow) {
            info.update_health(tree, v, pk, s);
        }
    }

    /// `RecvNotice_UpdateFellowMaxHealth`.
    pub fn update_max_health(
        &mut self,
        tree: &mut UiTree,
        s: &dyn FellowStrings,
        fellow: u64,
        v: u32,
    ) {
        if let Some(info) = self.fellow_mut(fellow) {
            info.update_max_health(tree, v, s);
        }
    }

    /// `RecvNotice_UpdateFellowVigor` (vigor and PK vigor loss).
    pub fn update_vigor(
        &mut self,
        tree: &mut UiTree,
        s: &dyn FellowStrings,
        fellow: u64,
        v: u32,
        pk: u32,
    ) {
        if let Some(info) = self.fellow_mut(fellow) {
            info.update_vigor(tree, v, pk, s);
        }
    }

    /// `RecvNotice_UpdateFellowMaxVigor`.
    pub fn update_max_vigor(
        &mut self,
        tree: &mut UiTree,
        s: &dyn FellowStrings,
        fellow: u64,
        v: u32,
    ) {
        if let Some(info) = self.fellow_mut(fellow) {
            info.update_max_vigor(tree, v, s);
        }
    }

    /// `RecvNotice_UpdateFellowLevel`: the row's level, then the shares.
    pub fn update_level(
        &mut self,
        tree: &mut UiTree,
        source: &dyn FellowshipSource,
        fellow: u64,
        level: u32,
    ) {
        if let Some(info) = self.fellow_mut(fellow) {
            info.update_level(tree, level, source);
        }
        self.update_xp(tree, source);
    }

    /// `RecvNotice_UpdateFellowCellID`.
    pub fn update_cell(&mut self, fellow: u64, cell: u32) {
        if let Some(info) = self.fellow_mut(fellow) {
            info.cell = cell;
        }
    }

    /// `FellowshipHelper::RecvNotice_SelectionChanged`: the selected member's name highlighted.
    pub fn selection_changed(&mut self, tree: &mut UiTree, selection: Option<u64>) {
        let Some(helper) = &mut self.helper else {
            return;
        };
        if let Some(old) = helper.selection.take() {
            if let Some(info) = helper.fellows.iter().find(|f| f.id == old) {
                info.show_selection_highlight(tree, false);
            }
        }
        if let Some(info) = helper.fellows.iter().find(|f| Some(f.id) == selection) {
            info.show_selection_highlight(tree, true);
            helper.selection = selection;
        }
    }

    /// `gmFellowshipUI::RecvNotice_RequestRecruitment` (`MakeRecruitDialog`):
    /// `ID_Fellowship_RecruitText` with the leader and the fellowship as a confirmation dialog.
    pub fn request_recruitment(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        strings: &dyn FellowStrings,
        leader: &str,
        fellowship: &str,
    ) {
        if self.recruit_context != 0 {
            return;
        }
        let text = strings.render(
            id("ID_Fellowship_RecruitText"),
            &[
                (id("leader"), SkillVar::Text(leader.to_owned())),
                (id("name"), SkillVar::Text(fellowship.to_owned())),
            ],
        );
        self.recruit_context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
    }

    /// `gmFellowshipUI::RecvNotice_ExpireRecruitment`.
    pub fn expire_recruitment(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        dialogs.close_dialog(tree, self.recruit_context);
        self.recruit_context = 0;
    }

    /// `gmFellowshipUI::RecvNotice_CloseConfirmationDialog`: the invitation answered.
    pub fn close_confirmation(&mut self, context: u32, yes: bool) {
        if context != self.recruit_context || context == 0 {
            return;
        }
        self.requests.push(if yes {
            FellowshipRequest::RecruitAccept
        } else {
            FellowshipRequest::RecruitDecline
        });
        self.recruit_context = 0;
    }

    /// `gmFellowshipUI::ListenToMessage` and `FellowshipHelper::ListenToMessage`.
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage) {
        if message.message == MSG_BUTTON_RELEASE_ON {
            let request = match message.element_id {
                e if e == id("FellowshipCreateButton") => self.create_request(tree),
                e if e == id("FellowshipLeaveButton") => Some(FellowshipRequest::Leave),
                e if e == id("FellowshipRecruitButton") => Some(FellowshipRequest::Recruit),
                e if e == id("FellowshipDisbandButton") => Some(FellowshipRequest::Disband),
                e if e == id("FellowshipDismissButton") => Some(FellowshipRequest::Dismiss),
                e if e == id("FellowshipNumberButton") => {
                    // `FellowshipHelper::ToggleTextState`.
                    if let Some(helper) = &self.helper {
                        for info in &helper.fellows {
                            info.toggle_text_state(tree);
                        }
                    }
                    None
                }
                _ => None,
            };
            self.requests.extend(request);
            return;
        }
        let Some(helper) = &self.helper else {
            return;
        };
        if message.message == GLOBAL_KEY_PRESS {
            // `SelectFellowByIdx`: the select-fellow keys pick rows 0 to 8.
            let action = message.data as u32;
            if (0x4100_0053..=0x4100_005B).contains(&action) {
                let index = (action - 0x4100_0053) as usize;
                let list = helper.list;
                tree.list_set_selected_index(list, Some(index), true);
                let row = tree.list_items(list).get(index).copied();
                if let Some(info) = helper.fellows.iter().find(|f| Some(f.root) == row) {
                    self.requests.push(FellowshipRequest::Select(info.id));
                }
            }
            return;
        }
        if message.message == id("mouse_press") && message.element == Some(helper.list) {
            let row = tree.list_item_under_mouse(helper.list);
            if let Some(info) = helper.fellows.iter().find(|f| Some(f.root) == row) {
                self.requests.push(FellowshipRequest::Select(info.id));
                if message.data as u32 == 0x4000_0007 {
                    self.requests
                        .push(FellowshipRequest::AssistedTell(info.name.clone()));
                }
            }
        }
    }

    /// `gmFellowshipUI::HandleCreateButton`: the entered name; retail passes "shared" as the XP
    /// toggle not highlighted.
    fn create_request(&self, tree: &UiTree) -> Option<FellowshipRequest> {
        self.page?;
        let name = self.page_child(tree, "NoFellowshipNameEntryText")?;
        let toggle = self.page_child(tree, "NoFellowshipXPToggleButton")?;
        let name = tree.get(name)?.text().unwrap_or_default();
        Some(FellowshipRequest::Create {
            name,
            social: !tree.bool_attribute(toggle, BUTTON_HIGHLIGHTED),
            looting: 1,
        })
    }
}
