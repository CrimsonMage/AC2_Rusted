//! The friends panel (`gmFriendsUI`): the list of friends the GUN service reports, its rows, the
//! selection, and the requests the panel makes. The panel keeps the friends as the GUN list gives
//! them (`BuildUIList` rebuilds the rows from them); a status change updates one friend in place;
//! a removed friend leaves the list; an added friend asks the GUN service for the list again
//! (`gmFriendsUI::RecvNotice_GUNResult_AddFriend` calls `RequestGUN_GetFriendsList(1)`).
//!
//! The rows are drawn in the panel's `FRIEND_ROWS` visible window; the row order is the list's
//! (`BuildUIList`'s sort is not traced).

use ac2_weenie::FriendInfo;

/// The rows the panel shows at once (the list box's visible window).
pub const FRIEND_ROWS: usize = 15;

/// What the panel asks of the GUN service (`gmCWInterface`'s friend requests).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendsRequest {
    /// `RequestGUN_GetFriendsList(userRequested)`.
    GetList { user_requested: bool },
    /// `RequestGUN_FriendAdd(name)` (`gmFriendsUI::HandleAddButton`).
    Add(String),
    /// `RequestGUN_FriendDelete(name)` (`gmFriendsUI::HandleRemoveButton`).
    Delete(String),
    /// `RequestGUN_SetLocation(flags, "")` and the status's emote (`gmFriendsUI::HandleStatusChange`:
    /// `DoEmote` with the emote's string id, named as the UI table names it, `ID_EmoteAFKCmd`).
    Status {
        flags: u32,
        emote: Option<&'static str>,
    },
}

/// The friends panel's state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendsUi {
    friends: Vec<FriendInfo>,
    selected: Option<usize>,
}

impl FriendsUi {
    /// The friends, in the order the list gave them.
    pub fn friends(&self) -> &[FriendInfo] {
        &self.friends
    }

    /// The rows the panel shows (at most [`FRIEND_ROWS`], from the top of the list).
    pub fn rows(&self) -> &[FriendInfo] {
        &self.friends[..self.friends.len().min(FRIEND_ROWS)]
    }

    /// The selected row, if any (`UIElement_ListBox::SetSelectedIndex`).
    pub fn selected(&self) -> Option<&FriendInfo> {
        self.selected.and_then(|i| self.friends.get(i))
    }

    /// Selects row `index` (or clears the selection past the list's end).
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index.filter(|i| *i < self.friends.len());
    }

    /// `gmFriendsUI::BuildUIList`: the list replaces the friends, and the selection keeps the
    /// same friend when it is still listed.
    pub fn recv_list(&mut self, list: &[FriendInfo]) {
        let kept = self.selected().map(|f| f.name.clone());
        self.friends = list.to_vec();
        self.selected = kept.and_then(|name| self.friends.iter().position(|f| f.name == name));
    }

    /// `gmFriendsUI::RecvNotice_GUNResult_FriendStatus`: the friend's status replaces the listed
    /// one. A friend not on the list is not added (the panel shows only the GUN list).
    pub fn recv_status(&mut self, status: &FriendInfo) {
        if let Some(friend) = self.friends.iter_mut().find(|f| f.name == status.name) {
            *friend = status.clone();
        }
    }

    /// `gmFriendsUI::RecvNotice_GUNResult_AddFriend`: the list is requested again, for the
    /// added friend to appear.
    pub fn recv_added(&mut self) -> FriendsRequest {
        FriendsRequest::GetList {
            user_requested: true,
        }
    }

    /// `gmFriendsUI::RecvNotice_GUNResult_RemoveFriend`: the friend leaves the list (the selection
    /// clears when it was on that friend).
    pub fn recv_removed(&mut self, name: &str) {
        if let Some(index) = self.friends.iter().position(|f| f.name == name) {
            self.friends.remove(index);
            self.selected = match self.selected {
                Some(s) if s == index => None,
                Some(s) if s > index => Some(s - 1),
                other => other,
            };
        }
    }

    /// `gmFriendsUI::HandleRemoveButton`: the selected friend's name, to be deleted at the GUN
    /// service (the remove button is enabled only with a selection).
    pub fn remove_request(&self) -> Option<String> {
        self.selected().map(|f| f.name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn friend(name: &str, mode: u32) -> FriendInfo {
        FriendInfo {
            name: name.to_owned(),
            location: String::new(),
            mode,
        }
    }

    #[test]
    fn the_list_fills_the_rows_and_a_status_replaces_one_friend() {
        let mut ui = FriendsUi::default();
        ui.recv_list(&[friend("Tomas", 1), friend("Ilse", 0)]);
        assert_eq!(ui.rows().len(), 2);
        ui.recv_status(&friend("Ilse", 2));
        assert_eq!(ui.friends()[1].mode, 2);
        // A status for a friend not listed is not added.
        ui.recv_status(&friend("Nobody", 1));
        assert_eq!(ui.friends().len(), 2);
    }

    #[test]
    fn the_rows_stop_at_the_visible_window() {
        let mut ui = FriendsUi::default();
        let list: Vec<FriendInfo> = (0..20).map(|i| friend(&format!("F{i}"), 0)).collect();
        ui.recv_list(&list);
        assert_eq!(ui.rows().len(), FRIEND_ROWS);
        assert_eq!(ui.friends().len(), 20);
    }

    #[test]
    fn remove_takes_the_selected_friend_and_keeps_the_others_selected() {
        let mut ui = FriendsUi::default();
        ui.recv_list(&[friend("Tomas", 1), friend("Ilse", 0), friend("Maria", 0)]);
        ui.select(Some(2));
        assert_eq!(ui.remove_request().as_deref(), Some("Maria"));
        ui.recv_removed("Tomas");
        assert_eq!(ui.selected().map(|f| f.name.as_str()), Some("Maria"));
        ui.recv_removed("Maria");
        assert!(ui.selected().is_none());
    }

    #[test]
    fn an_add_asks_for_the_list_again() {
        let mut ui = FriendsUi::default();
        assert_eq!(
            ui.recv_added(),
            FriendsRequest::GetList {
                user_requested: true
            }
        );
    }
}
