//! `gmFriendsUI` on the social toolbar: the friends page (`FriendsPage`) over the friends model
//! ([`FriendsUi`]). `BuildUIList` rebuilds the `FriendsListBox` rows from the model: an
//! `OnlineFriendsMarker` row and the online friends, then an `OfflineFriendsMarker` row and the
//! offline ones, each from `FriendTemplate` with its `FriendName` text. The add and remove buttons
//! ask the client (`gmFriendsUI::HandleAddButton`, `HandleRemoveButton`); the status drop-down
//! asks for the GUN status and its emote (`gmFriendsUI::HandleStatusChange`).
//!
//! Online and offline come from the friend's location: retail's `m_onlineList` and `m_offlineList`
//! are filled by code that Legions 7160 does not carry in this page, so an empty location (the GUN
//! status's "offline or unknown", see [`FriendInfo`]) lists the friend offline.
//!
//! Not modelled yet: the status drop-down's choice (`UIElement_Menu` is not ported, so
//! [`FriendsPanel::message`] reads no choice; [`status_request`] is the request it would send), the
//! text of an away or do-not-disturb friend (the GUN mode bits are not traced), and the row tooltips.

use std::sync::Arc;

use ac2_weenie::FriendInfo;

use crate::element::UiTree;
use crate::friends::{FriendsRequest, FriendsUi};
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `gmFriendsUI::StatusTable` (admin 7148's `_E435` initializer): the GUN mode each status sets
/// (`m_gunMode`, the first from Legions 7160's data), and the emote it plays (`m_emoteStringID`:
/// `ID_EmoteYawnCmd`, or `ID_EmoteAFKCmd` for away).
pub const STATUS_TABLE: [(u32, &str); 3] = [(1, "yawn"), (4, "yawn"), (2, "afk")];

/// `gmFriendsUI::HandleStatusChange` for drop-down entry `index`: the status flags and the emote
/// word (`None` past the table).
pub fn status_request(index: usize) -> Option<FriendsRequest> {
    let (flags, emote) = *STATUS_TABLE.get(index)?;
    Some(FriendsRequest::Status {
        flags,
        emote: Some(emote),
    })
}

/// The friends page's state: the model, and the page's elements.
#[derive(Debug, Clone)]
pub struct FriendsPanel {
    layout: Arc<LayoutDesc>,
    model: FriendsUi,
    /// `m_friendsPage`.
    page: Option<usize>,
    /// `m_friendsListBox`.
    list: Option<usize>,
    /// `m_addButton`, `m_removeButton`.
    add: Option<usize>,
    remove: Option<usize>,
    /// The friend rows as listed: the row element and the friend's name, in list order.
    rows: Vec<(usize, String)>,
}

impl FriendsPanel {
    /// `gmFriendsUI::gmFriendsUI`: the page is the one `FriendsPage` element in the social
    /// layout's roots. The remove button starts ghosted (`UpdateButtonStates`, no selection).
    pub fn create(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> Self {
        let page = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("FriendsPage")));
        let child =
            |tree: &UiTree, name: &str| page.and_then(|p| tree.child_recursive(p, id(name)));
        let list = child(tree, "FriendsListBox");
        let add = child(tree, "FriendsAddButton");
        let remove = child(tree, "FriendsRemoveButton");
        if let Some(remove) = remove {
            tree.set_bool(remove, BUTTON_GHOSTED, true);
        }
        Self {
            layout: Arc::clone(layout),
            model: FriendsUi::default(),
            page,
            list,
            add,
            remove,
            rows: Vec::new(),
        }
    }

    /// Whether the page was found in the layout.
    pub fn has_page(&self) -> bool {
        self.page.is_some()
    }

    /// The friends model (the list in the order the GUN service gave it).
    pub fn model(&self) -> &FriendsUi {
        &self.model
    }

    /// The friend rows as listed, top first: each row's friend name.
    pub fn listed(&self) -> Vec<&str> {
        self.rows.iter().map(|(_, name)| name.as_str()).collect()
    }

    /// `gmFriendsUI::BuildUIList` on the GUN list: the friends replace the model and the rows
    /// are rebuilt.
    pub fn recv_list(&mut self, tree: &mut UiTree, list: &[FriendInfo]) {
        self.model.recv_list(list);
        self.build(tree);
    }

    /// `gmFriendsUI::RecvNotice_GUNResult_FriendStatus`: one friend's status, and the rows.
    pub fn recv_status(&mut self, tree: &mut UiTree, status: &FriendInfo) {
        self.model.recv_status(status);
        self.build(tree);
    }

    /// `gmFriendsUI::RecvNotice_GUNResult_AddFriend`: the request for the list again.
    pub fn recv_added(&mut self) -> FriendsRequest {
        self.model.recv_added()
    }

    /// `gmFriendsUI::RecvNotice_GUNResult_RemoveFriend`: the friend leaves the rows.
    pub fn recv_removed(&mut self, tree: &mut UiTree, name: &str) {
        self.model.recv_removed(name);
        self.build(tree);
    }

    /// `gmFriendsUI::UpdateButtonStates`: the add button is enabled for a player who is not a
    /// friend yet (`target`, the game selection's name, when it is a player), and the remove
    /// button for a selected friend.
    pub fn set_target(&mut self, tree: &mut UiTree, target: Option<&str>) {
        let addable = target.is_some_and(|name| {
            !name.is_empty() && !self.model.friends().iter().any(|f| f.name == name)
        });
        if let Some(add) = self.add {
            tree.set_bool(add, BUTTON_GHOSTED, !addable);
        }
        self.update_remove(tree);
    }

    /// The panel's messages. A press on a friend row selects it. The add button sends the
    /// target's name (`HandleAddButton`, `target` as [`FriendsPanel::set_target`] has it); the
    /// remove button sends the selected friend (`HandleRemoveButton`).
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        message: &UiMessage,
        target: Option<&str>,
    ) -> Vec<FriendsRequest> {
        let mut requests = Vec::new();
        if message.message == MSG_BUTTON_RELEASE_ON {
            if message.element.is_some() && message.element == self.add {
                if let Some(name) = target.filter(|n| !n.is_empty()) {
                    requests.push(FriendsRequest::Add(name.to_owned()));
                }
            } else if message.element.is_some() && message.element == self.remove {
                if let Some(name) = self.model.remove_request() {
                    requests.push(FriendsRequest::Delete(name));
                }
            }
        }
        self.sync_selection(tree);
        requests
    }

    /// The list's selected row, as the model's selection (a marker row selects nothing).
    fn sync_selection(&mut self, tree: &mut UiTree) {
        let Some(list) = self.list else {
            return;
        };
        let item = tree.list_selected_item(list);
        let index = item
            .and_then(|item| self.rows.iter().find(|(root, _)| *root == item))
            .and_then(|(_, name)| self.model.friends().iter().position(|f| &f.name == name));
        self.model.select(index);
        self.update_remove(tree);
    }

    fn update_remove(&self, tree: &mut UiTree) {
        if let Some(remove) = self.remove {
            let selected = self.model.selected().is_some();
            tree.set_bool(remove, BUTTON_GHOSTED, !selected);
        }
    }

    /// `BuildUIList`: the rows are flushed and listed again from the model.
    fn build(&mut self, tree: &mut UiTree) {
        let Some(list) = self.list else {
            return;
        };
        tree.list_flush(list);
        self.rows.clear();
        let online: Vec<FriendInfo> = self
            .model
            .friends()
            .iter()
            .filter(|f| !f.location.is_empty())
            .cloned()
            .collect();
        let offline: Vec<FriendInfo> = self
            .model
            .friends()
            .iter()
            .filter(|f| f.location.is_empty())
            .cloned()
            .collect();
        self.add_marker(tree, list, "OnlineFriendsMarker");
        for friend in &online {
            self.add_row(tree, list, friend, "normal");
        }
        self.add_marker(tree, list, "OfflineFriendsMarker");
        for friend in &offline {
            self.add_row(tree, list, friend, "offline");
        }
        self.update_remove(tree);
    }

    fn add_marker(&mut self, tree: &mut UiTree, list: usize, marker: &str) {
        let Some(desc) = self.layout.find_element(id(marker)) else {
            return;
        };
        if let Some(root) = tree.create_element(Arc::clone(&self.layout), desc, false) {
            tree.list_add_item(list, root);
        }
    }

    fn add_row(&mut self, tree: &mut UiTree, list: usize, friend: &FriendInfo, state: &str) {
        let Some(desc) = self.layout.find_element(id("FriendTemplate")) else {
            return;
        };
        let Some(root) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
            return;
        };
        if let Some(name) = tree.child_recursive(root, id("FriendName")) {
            tree.set_text(name, &friend.name);
        }
        tree.set_state(root, id(state));
        tree.list_add_item(list, root);
        self.rows.push((root, friend.name.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_table_is_available_dnd_and_away() {
        assert_eq!(
            status_request(1),
            Some(FriendsRequest::Status {
                flags: 4,
                emote: Some("yawn")
            })
        );
        assert_eq!(
            status_request(2),
            Some(FriendsRequest::Status {
                flags: 2,
                emote: Some("afk")
            })
        );
        assert_eq!(status_request(3), None);
    }
}
