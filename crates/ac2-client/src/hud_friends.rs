//! `gmFriendsUI` on the social page: the GUN list and status events and the friend results reach
//! the page, and its buttons and status choice become the client's friend requests
//! ([`crate::gameplay::Command::Friends`]).

use ac2_ui::{FriendsPanel, UiMessage, UiTree};
use ac2_weenie::GameplayNotice;

use crate::friends::FriendEvent;
use crate::gameplay::{Command, SelectionFrame};

/// `gmCWInterface::GetSelection` as the friends page takes it: the name of the selected player
/// when it is another player (not the local one), the add button's target. `None` otherwise.
pub(crate) fn friend_target(selection: Option<&SelectionFrame>) -> Option<&str> {
    selection
        .filter(|s| s.other_player)
        .and_then(|s| s.name.as_deref())
}

/// The page's notice (`gmFriendsUI::RecvNotice_GUNResult_*`): an added friend asks for the list
/// again; a removed one leaves the rows; a status or the list is shown.
pub(crate) fn friends_notice(
    panel: &mut FriendsPanel,
    tree: &mut UiTree,
    notice: &GameplayNotice,
    commands: &mut Vec<Command>,
) {
    match notice {
        GameplayNotice::FriendAdded(_) => {
            commands.push(Command::Friends(panel.recv_added()));
        }
        GameplayNotice::FriendRemoved(name) => panel.recv_removed(tree, name),
        GameplayNotice::FriendStatus(friend) => panel.recv_status(tree, friend),
        GameplayNotice::FriendsList(list) => panel.recv_list(tree, list),
        _ => {}
    }
}

/// A GUN event the client took for the page: the `FriendStart` list (`BuildUIList`) or one
/// status.
pub(crate) fn friend_event(panel: &mut FriendsPanel, tree: &mut UiTree, event: FriendEvent) {
    match event {
        FriendEvent::Start(list) => panel.recv_list(tree, &list),
        FriendEvent::Status(friend) => panel.recv_status(tree, &friend),
    }
}

/// The page's messages: its buttons and the status choice, as requests in order.
pub(crate) fn friends_message(
    panel: &mut FriendsPanel,
    tree: &mut UiTree,
    message: &UiMessage,
    target: Option<&str>,
    commands: &mut Vec<Command>,
) {
    for request in panel.message(tree, message, target) {
        commands.push(Command::Friends(request));
    }
}

#[cfg(test)]
mod tests {
    use super::friend_target;
    use crate::gameplay::SelectionFrame;

    fn selection(name: Option<&str>, other_player: bool) -> SelectionFrame {
        SelectionFrame {
            id: 7,
            name: name.map(Into::into),
            info: None,
            other_player,
        }
    }

    #[test]
    fn only_another_players_name_is_the_friend_target() {
        assert_eq!(friend_target(None), None);
        assert_eq!(
            friend_target(Some(&selection(Some("Birch"), true))),
            Some("Birch")
        );
        // A creature, or the local player's own object, is not a friend target.
        assert_eq!(friend_target(Some(&selection(Some("Birch"), false))), None);
        assert_eq!(friend_target(Some(&selection(None, true))), None);
    }
}
