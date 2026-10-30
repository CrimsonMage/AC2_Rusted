//! `FellowshipHelper`: the members' list over one list box. The social page's `FellowshipListBox`
//! (`FellowTemplate` rows from the toolbar layout) and the mini panel's `FellowMiniPanelListBox`
//! (`FellowMiniPanelTemplate` rows from the gameplay layout, `FellowshipHelper` mode 1) are both
//! this helper. A member is a [`FellowInfo`] row; the leader's row goes to the top (`SetLeader`).
//! The member-level notices (`FellowshipHelper::RecvNotice_*`) come in as a [`FellowNotice`].
//!
//! The map markers the helper sends are `fellow_markers.rs`, sent once for both helpers.

use std::sync::Arc;

use crate::element::UiTree;
use crate::fellow_info::{Fellow, FellowInfo, FellowStrings};
use crate::fellowship::FellowshipSource;
use crate::layout::LayoutDesc;
use crate::token::tokenize;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// A member-level notice as the server describes it (the rows' `RecvNotice_*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FellowNotice {
    /// `FellowshipHelper::RecvNotice_UpdateFellow`: a new member's row, or the row updated.
    Update(Fellow),
    /// `FellowshipHelper::RecvNotice_ClearFellow`.
    Clear(u64),
    /// `FellowshipHelper::RecvNotice_UpdateLeader` (`SetLeader`).
    Leader(u64),
    /// `RecvNotice_UpdateFellowHealth`: health and PK damage.
    Health { fellow: u64, value: u32, pk: u32 },
    /// `RecvNotice_UpdateFellowMaxHealth`.
    MaxHealth { fellow: u64, value: u32 },
    /// `RecvNotice_UpdateFellowVigor`: vigor and PK vigor loss.
    Vigor { fellow: u64, value: u32, pk: u32 },
    /// `RecvNotice_UpdateFellowMaxVigor`.
    MaxVigor { fellow: u64, value: u32 },
    /// `RecvNotice_UpdateFellowLevel`.
    Level { fellow: u64, level: u32 },
    /// `RecvNotice_UpdateFellowCellID`.
    Cell { fellow: u64, cell: u32 },
    /// `FellowshipHelper::RecvNotice_SelectionChanged`: the selected member, if any.
    Selection(Option<u64>),
}

/// `FellowshipHelper`'s state: the list box, the rows in the order added, the leader and the
/// selection.
#[derive(Debug, Clone)]
pub struct FellowRows {
    layout: Arc<LayoutDesc>,
    /// `_miniPanel`: the mini panel's rows (`FellowMiniPanelTemplate`, no level or XP share).
    mini: bool,
    /// `_listBox`.
    pub(crate) list: usize,
    /// `_fellowUIHash`, in the order added.
    pub(crate) fellows: Vec<FellowInfo>,
    leader: Option<u64>,
    selection: Option<u64>,
}

impl FellowRows {
    /// `FellowshipHelper::FellowshipHelper`: an empty list over `list`, its rows from `layout`
    /// (the toolbar's for the page, the gameplay layout's for the mini panel).
    pub fn new(list: usize, layout: Arc<LayoutDesc>, mini: bool) -> Self {
        Self {
            layout,
            mini,
            list,
            fellows: Vec::new(),
            leader: None,
            selection: None,
        }
    }

    /// `FellowshipHelper::AddAllFellows`, the leader moved up, then `UpdateXP`.
    pub fn populate(
        &mut self,
        tree: &mut UiTree,
        source: &dyn FellowshipSource,
        fellows: &[Fellow],
        leader: u64,
    ) {
        for fellow in fellows {
            self.add_fellow(tree, source, fellow);
        }
        self.set_leader(tree, leader);
        self.update_xp(tree, source);
    }

    /// The members' rows, top first, and their ids.
    pub fn rows(&self, tree: &UiTree) -> Vec<(u64, usize)> {
        tree.list_items(self.list)
            .iter()
            .filter_map(|&item| {
                self.fellows
                    .iter()
                    .find(|f| f.root == item)
                    .map(|f| (f.id, item))
            })
            .collect()
    }

    /// The members: id, name and cell, in the order added.
    pub(crate) fn members(&self) -> Vec<(u64, String, u32)> {
        self.fellows
            .iter()
            .map(|f| (f.id, f.name.clone(), f.cell))
            .collect()
    }

    /// The member's row, if it is listed.
    pub fn fellow_at_item(&self, item: Option<usize>) -> Option<&FellowInfo> {
        self.fellows.iter().find(|f| Some(f.root) == item)
    }

    /// `FellowshipHelper::RemoveAllFellows`.
    pub fn remove_all(&mut self, tree: &mut UiTree) {
        tree.list_flush(self.list);
        self.fellows.clear();
    }

    /// `FellowshipHelper::AddFellow`: a row from the template (`FellowTemplate` or
    /// `FellowMiniPanelTemplate`) added to the list.
    fn add_fellow(&mut self, tree: &mut UiTree, strings: &dyn FellowStrings, fellow: &Fellow) {
        let template = if self.mini {
            "FellowMiniPanelTemplate"
        } else {
            "FellowTemplate"
        };
        let Some(desc) = self.layout.element(id(template)).cloned() else {
            return;
        };
        let Some(root) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
            return;
        };
        let info = FellowInfo::new(tree, root, fellow, strings, self.mini);
        tree.list_add_item(self.list, root);
        self.fellows.push(info);
    }

    /// `FellowshipHelper::SetLeader`: the leader's row moved to the top.
    fn set_leader(&mut self, tree: &mut UiTree, leader: u64) {
        let old = self.leader.replace(leader);
        let find = |id: Option<u64>| self.fellows.iter().find(|f| Some(f.id) == id);
        // Retail moves the old leader back up when the new one has no row.
        let Some(row) = find(Some(leader)).or_else(|| find(old)).map(|f| f.root) else {
            return;
        };
        let list = self.list;
        tree.list_multi_op_start(list);
        if let Some(at) = tree.list_what_num(list, row) {
            tree.list_remove_item(list, at);
        }
        tree.list_insert_item(list, row, 0);
        tree.list_multi_op_complete(list);
    }

    /// `FellowshipHelper::UpdateXP`: each member's share, when the player answers.
    fn update_xp(&self, tree: &mut UiTree, source: &dyn FellowshipSource) {
        let Some(shares) = source.fellow_xp() else {
            return;
        };
        for (fellow, share) in shares {
            let Some(info) = self.fellows.iter().find(|f| f.id == fellow) else {
                return;
            };
            info.update_percentage(tree, share, source);
        }
    }

    fn fellow_mut(&mut self, fellow: u64) -> Option<&mut FellowInfo> {
        self.fellows.iter_mut().find(|f| f.id == fellow)
    }

    /// `FellowshipHelper::RecvNotice_UpdateFellow`: a new member's row, or the row updated.
    fn update_fellow(&mut self, tree: &mut UiTree, source: &dyn FellowshipSource, fellow: &Fellow) {
        match self.fellow_mut(fellow.id) {
            Some(info) => info.update_info(tree, fellow, source),
            None => self.add_fellow(tree, source, fellow),
        }
        self.update_xp(tree, source);
    }

    /// `FellowshipHelper::RecvNotice_ClearFellow` (`RemoveFellow`, then `UpdateXP`).
    fn clear_fellow(&mut self, tree: &mut UiTree, source: &dyn FellowshipSource, fellow: u64) {
        if let Some(at) = self.fellows.iter().position(|f| f.id == fellow) {
            let info = self.fellows.remove(at);
            tree.list_delete_item(self.list, info.root);
        }
        self.update_xp(tree, source);
    }

    /// `FellowshipHelper::RecvNotice_SelectionChanged`: the selected member's name highlighted.
    fn selection_changed(&mut self, tree: &mut UiTree, selection: Option<u64>) {
        if let Some(old) = self.selection.take() {
            if let Some(info) = self.fellows.iter().find(|f| f.id == old) {
                info.show_selection_highlight(tree, false);
            }
        }
        if let Some(info) = self.fellows.iter().find(|f| Some(f.id) == selection) {
            info.show_selection_highlight(tree, true);
            self.selection = selection;
        }
    }

    /// One member-level notice.
    pub fn notice(
        &mut self,
        tree: &mut UiTree,
        source: &dyn FellowshipSource,
        notice: &FellowNotice,
    ) {
        match notice {
            FellowNotice::Update(fellow) => self.update_fellow(tree, source, fellow),
            FellowNotice::Clear(fellow) => self.clear_fellow(tree, source, *fellow),
            FellowNotice::Leader(leader) => self.set_leader(tree, *leader),
            FellowNotice::Health { fellow, value, pk } => {
                if let Some(info) = self.fellow_mut(*fellow) {
                    info.update_health(tree, *value, *pk, source);
                }
            }
            FellowNotice::MaxHealth { fellow, value } => {
                if let Some(info) = self.fellow_mut(*fellow) {
                    info.update_max_health(tree, *value, source);
                }
            }
            FellowNotice::Vigor { fellow, value, pk } => {
                if let Some(info) = self.fellow_mut(*fellow) {
                    info.update_vigor(tree, *value, *pk, source);
                }
            }
            FellowNotice::MaxVigor { fellow, value } => {
                if let Some(info) = self.fellow_mut(*fellow) {
                    info.update_max_vigor(tree, *value, source);
                }
            }
            FellowNotice::Level { fellow, level } => {
                if let Some(info) = self.fellow_mut(*fellow) {
                    info.update_level(tree, *level, source);
                }
                self.update_xp(tree, source);
            }
            FellowNotice::Cell { fellow, cell } => {
                if let Some(info) = self.fellow_mut(*fellow) {
                    info.cell = *cell;
                }
            }
            FellowNotice::Selection(selection) => self.selection_changed(tree, *selection),
        }
    }
}
