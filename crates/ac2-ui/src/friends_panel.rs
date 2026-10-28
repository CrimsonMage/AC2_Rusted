//! `gmFriendsUI` on the social toolbar: the friends page (`FriendsPage`) over the friends model
//! ([`FriendsUi`]). `BuildUIList` rebuilds the `FriendsListBox` rows from the model: an
//! `OnlineFriendsMarker` row and the online friends, then an `OfflineFriendsMarker` row and the
//! offline ones, each from `FriendTemplate` with its `FriendName` text. The add and remove buttons
//! ask the client (`gmFriendsUI::HandleAddButton`, `HandleRemoveButton`); the status drop-down
//! asks for the GUN status and its emote (`gmFriendsUI::HandleStatusChange`).
//!
//! Online and offline come from the friend's mode: `gmFriendsUI::RecvNotice_GUNResult_FriendStatus`
//! and `RecvNotice_GUNResult_GetFriendsList` list a friend with `m_Mode == 0` offline and any other
//! mode online. A row's state and text follow `BuildUIList`: an away friend (`MODE_AWAY`) shows
//! the AFK message after the name and the `token_awayfromkeyboard` state, a do-not-disturb friend
//! (`MODE_DO_NOT_DISTURB`, when not away) the DND message and `token_donotdisturb`, an online
//! friend `token_online`, an offline one `token_offline`.
//!
//! The status drop-down (`StatusDropDownMenu`, a `UIElement_Menu`) is filled with the status names
//! when the page is made; a new selection in it is the status request ([`status_request`]).
//!
//! The server's own status (`CM_Friends::SendNotice_LocalStatus`) selects the drop-down entry
//! ([`FriendsPanel::recv_local_status`]). Not modelled yet: the IsMoving reset of the drop-down
//! (`ListenToMessage`'s global 0xFFFF0005), and the row tooltips.

use std::sync::Arc;

use ac2_weenie::{FriendInfo, GameplayNotice, MODE_AWAY, MODE_DO_NOT_DISTURB};

use crate::element::UiTree;
use crate::friends::{FriendsRequest, FriendsUi};
use crate::layout::LayoutDesc;
use crate::menu::MSG_MENU_NEW_SELECTION;
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `gmFriendsUI::StatusTable` (admin 7148's `_E435` initializer): the GUN mode each status sets
/// (`m_gunMode`, the first from Legions 7160's data), and the emote's string id name it plays
/// (`m_emoteStringID`: `ID_EmoteYawnCmd`, or `ID_EmoteAFKCmd` for away).
pub const STATUS_TABLE: [(u32, &str); 3] = [
    (1, "ID_EmoteYawnCmd"),
    (4, "ID_EmoteYawnCmd"),
    (2, "ID_EmoteAFKCmd"),
];

/// `gmFriendsUI::HandleStatusChange` for drop-down entry `index`: the status flags and the emote
/// word (`None` past the table).
pub fn status_request(index: usize) -> Option<FriendsRequest> {
    let (flags, emote) = *STATUS_TABLE.get(index)?;
    Some(FriendsRequest::Status {
        flags,
        emote: Some(emote),
    })
}

/// The status drop-down's items, in [`STATUS_TABLE`]'s order (`gmFriendsUI::StatusTable`'s names).
const STATUS_NAMES: [&str; 3] = [
    "ID_FriendStatus_Available",
    "ID_FriendStatus_DoNotDisturb",
    "ID_FriendStatus_AwayFromKeyboard",
];

/// `gmFriendsUI::BuildUIList`'s row states (`token_online`, `token_donotdisturb`,
/// `token_awayfromkeyboard`, `token_offline`).
const STATE_ONLINE: &str = "token_online";
const STATE_DO_NOT_DISTURB: &str = "token_donotdisturb";
const STATE_AWAY: &str = "token_awayfromkeyboard";
const STATE_OFFLINE: &str = "token_offline";

/// The status messages a row appends to its name (`ID_FriendStatus_*Message`, the UI table).
const AWAY_MESSAGE: &str = "ID_FriendStatus_AwayFromKeyboardMessage";
const DO_NOT_DISTURB_MESSAGE: &str = "ID_FriendStatus_DoNotDisturbMessage";

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
    /// `m_statusMenu` (`StatusDropDownMenu`).
    status: Option<usize>,
    /// The friend rows as listed: the row element and the friend's name, in list order.
    rows: Vec<(usize, String)>,
    /// The away and do-not-disturb messages, read while the tree is built (the text is frozen
    /// once the HUD is made).
    away_text: Option<String>,
    do_not_disturb_text: Option<String>,
}

impl FriendsPanel {
    /// `gmFriendsUI::gmFriendsUI`: the page is the one `FriendsPage` element in the social
    /// layout's roots. Both buttons start ghosted (`UpdateButtonStates`: no target, no selection).
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
        for button in [add, remove].into_iter().flatten() {
            tree.set_bool(button, BUTTON_GHOSTED, true);
        }
        // `gmFriendsUI::gmFriendsUI`: the status items inserted, the first selected (before the
        // page listens to the drop-down, so that selection is not its message).
        let status = child(tree, "StatusDropDownMenu");
        if let Some(menu) = status {
            for (index, name) in STATUS_NAMES.iter().enumerate() {
                tree.menu_insert_string(menu, layout.string_table, id(name), index);
            }
            tree.menu_set_selected_index(menu, Some(0));
            tree.menu_drop_selection_messages(menu);
        }
        // The toolbar's string table (the UI table, `0x41000018`, as its drop-down entries use).
        let message = |name: &str| {
            tree.text_data
                .as_ref()
                .and_then(|data| data.string(layout.string_table, id(name)))
                .map(|units| String::from_utf16_lossy(&units))
        };
        let away_text = message(AWAY_MESSAGE);
        let do_not_disturb_text = message(DO_NOT_DISTURB_MESSAGE);
        Self {
            layout: Arc::clone(layout),
            model: FriendsUi::default(),
            page,
            list,
            add,
            remove,
            status,
            rows: Vec::new(),
            away_text,
            do_not_disturb_text,
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

    /// The friend rows' elements, top first (the marker rows are not among them).
    pub fn row_roots(&self) -> Vec<usize> {
        self.rows.iter().map(|(root, _)| *root).collect()
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

    /// `gmFriendsUI::RecvNotice_LocalStatus`: the server's status (a [`STATUS_TABLE`] mode) selects
    /// its drop-down entry. The selection is broadcast, as retail's `SetSelectedIndex` is, so the
    /// page's status change (`HandleStatusChange`) follows through [`FriendsPanel::message`].
    pub fn recv_local_status(&mut self, tree: &mut UiTree, status: u32) {
        let Some(index) = STATUS_TABLE.iter().position(|(mode, _)| *mode == status) else {
            return;
        };
        if let Some(menu) = self.status {
            tree.menu_set_selected_index(menu, Some(index));
        }
    }

    /// The page's notices: an added friend asks for the list again (the returned request); a
    /// removed one leaves the rows; a status or the list is shown; the server's own status
    /// selects the drop-down. Other notices are not the page's.
    pub fn notice(&mut self, tree: &mut UiTree, notice: &GameplayNotice) -> Option<FriendsRequest> {
        match notice {
            GameplayNotice::FriendAdded(_) => return Some(self.recv_added()),
            GameplayNotice::FriendRemoved(name) => self.recv_removed(tree, name),
            GameplayNotice::FriendStatus(friend) => self.recv_status(tree, friend),
            GameplayNotice::FriendsList(list) => self.recv_list(tree, list),
            GameplayNotice::LocalStatus(status) => self.recv_local_status(tree, *status),
            _ => {}
        }
        None
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

    /// `gmFriendsUI::UpdateButtonStates`: the add button and the remove button's states for the
    /// game selection's name `target` (a player's name, see `hud_friends::friend_target`).
    pub fn set_target(&mut self, tree: &mut UiTree, target: Option<&str>) {
        self.update_add(tree, target);
        self.update_remove(tree);
    }

    /// The add button is enabled for a player who is not a friend yet, and only while no list
    /// row is selected: a selected row (a friend or a marker) keeps it ghosted, as retail's
    /// `UpdateButtonStates` leaves it.
    fn update_add(&self, tree: &mut UiTree, target: Option<&str>) {
        let row_selected = self
            .list
            .is_some_and(|list| tree.list_selected_item(list).is_some());
        let addable = !row_selected
            && target.is_some_and(|name| {
                !name.is_empty() && !self.model.friends().iter().any(|f| f.name == name)
            });
        if let Some(add) = self.add {
            tree.set_bool(add, BUTTON_GHOSTED, !addable);
        }
    }

    /// The panel's messages. A press on a friend row selects it. The add button sends the
    /// target's name (`HandleAddButton`, `target` as [`FriendsPanel::set_target`] has it); the
    /// remove button sends the selected friend (`HandleRemoveButton`). A new selection in the
    /// status drop-down sends its status (`HandleStatusChange`).
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        message: &UiMessage,
        target: Option<&str>,
    ) -> Vec<FriendsRequest> {
        let mut requests = Vec::new();
        if message.message == MSG_MENU_NEW_SELECTION
            && message.element.is_some()
            && message.element == self.status
        {
            if let Some(index) = self.status.and_then(|menu| tree.menu_selected_index(menu)) {
                requests.extend(status_request(index));
            }
        }
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
        self.update_add(tree, target);
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
            .filter(|f| f.is_online())
            .cloned()
            .collect();
        let offline: Vec<FriendInfo> = self
            .model
            .friends()
            .iter()
            .filter(|f| !f.is_online())
            .cloned()
            .collect();
        self.add_marker(tree, list, "OnlineFriendsMarker");
        for friend in &online {
            self.add_online_row(tree, list, friend);
        }
        self.add_marker(tree, list, "OfflineFriendsMarker");
        for friend in &offline {
            self.add_row(tree, list, friend, STATE_OFFLINE, None);
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

    /// An online friend's row (`BuildUIList`'s online branch): away before do-not-disturb, the
    /// message appended to the name.
    fn add_online_row(&mut self, tree: &mut UiTree, list: usize, friend: &FriendInfo) {
        let (state, suffix) = if friend.mode & MODE_AWAY != 0 {
            (STATE_AWAY, self.away_text.clone())
        } else if friend.mode & MODE_DO_NOT_DISTURB != 0 {
            (STATE_DO_NOT_DISTURB, self.do_not_disturb_text.clone())
        } else {
            (STATE_ONLINE, None)
        };
        self.add_row(tree, list, friend, state, suffix);
    }

    fn add_row(
        &mut self,
        tree: &mut UiTree,
        list: usize,
        friend: &FriendInfo,
        state: &str,
        suffix: Option<String>,
    ) {
        let Some(desc) = self.layout.find_element(id("FriendTemplate")) else {
            return;
        };
        let Some(root) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
            return;
        };
        if let Some(name) = tree.child_recursive(root, id("FriendName")) {
            let text = format!("{}{}", friend.name, suffix.unwrap_or_default());
            tree.set_text(name, &text);
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
                emote: Some("ID_EmoteYawnCmd")
            })
        );
        assert_eq!(
            status_request(2),
            Some(FriendsRequest::Status {
                flags: 2,
                emote: Some("ID_EmoteAFKCmd")
            })
        );
        assert_eq!(status_request(3), None);
    }
}
