//! `gmFellowshipUI` and its `FellowshipHelper`: the social panel's fellowship page. Without a
//! fellowship the page is in `nofellowship` (name entry, XP sharing toggle, looting menu and
//! Create); with one it is in `fellowship`: the name, the XP sharing and looting indicators, the
//! leader's buttons (Recruit, Dismiss and Disband ghosted for the others), Leave, and one
//! [`FellowInfo`] row per member in `FellowshipListBox`, the leader first (the list itself is a
//! [`FellowRows`]). A recruitment invitation is a confirmation dialog.
//!
//! The panel turns the player's buttons and clicks into [`FellowshipRequest`]s; the client runs
//! them (`gmCWInterface::DoFellowship*`, `SetSelection`, `DoAssistedTell`).
//!
//! Not modelled yet: the looting drop-down (`menu.rs` ports `UIElement_Menu`, but it is not
//! wired here; Create sends retail's default, method 1), tooltips and the map markers (`SendNotice_AddMarker` / `RemoveMarker` /
//! `ClearMarkers`). The radar's mini panel is `mini_fellow.rs`.

use crate::dialog::{DialogFactory, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::fellow_info::{Fellow, FellowStrings};
use crate::fellow_rows::{FellowNotice, FellowRows};
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

/// `gmFellowshipUI`'s state.
#[derive(Debug, Clone)]
pub struct FellowshipUi {
    layout: Arc<LayoutDesc>,
    page: Option<usize>,
    /// `_fellowshipHelper`: the members, `None` without a fellowship.
    helper: Option<FellowRows>,
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
        self.helper.as_ref().map_or_else(Vec::new, |h| h.rows(tree))
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
        let mut rows = FellowRows::new(list, Arc::clone(&self.layout), false);
        rows.populate(tree, source, &data.fellows, data.leader);
        self.helper = Some(rows);
    }

    /// `RecvNotice_ClearFellowship`.
    pub fn clear_fellowship(&mut self, tree: &mut UiTree) {
        self.remove_all_fellows(tree);
        self.helper = None;
        self.set_no_fellowship_state(tree);
    }

    /// `FellowshipHelper::RemoveAllFellows`.
    fn remove_all_fellows(&mut self, tree: &mut UiTree) {
        if let Some(helper) = &mut self.helper {
            helper.remove_all(tree);
        }
    }

    /// A member-level notice (the helper's `RecvNotice_*`); a leader change also sets the
    /// buttons (`gmFellowshipUI::RecvNotice_UpdateLeader`).
    pub fn fellow_notice(
        &mut self,
        tree: &mut UiTree,
        source: &dyn FellowshipSource,
        notice: &FellowNotice,
    ) {
        if let FellowNotice::Leader(leader) = notice {
            if self.helper.is_some() {
                self.set_leader_state(tree, *leader == source.player());
            }
        }
        if let Some(helper) = &mut self.helper {
            helper.notice(tree, source, notice);
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
                if let Some(info) = helper.fellow_at_item(row) {
                    self.requests.push(FellowshipRequest::Select(info.id));
                }
            }
            return;
        }
        if message.message == id("mouse_press") && message.element == Some(helper.list) {
            let row = tree.list_item_under_mouse(helper.list);
            if let Some(info) = helper.fellow_at_item(row) {
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
