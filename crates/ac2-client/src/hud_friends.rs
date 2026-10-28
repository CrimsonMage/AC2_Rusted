//! `gmFriendsUI` on the social page: the GUN list and status events and the friend results reach
//! the page, and its buttons and status choice become the client's friend requests
//! ([`crate::gameplay::Command::Friends`]).

use crate::gameplay::SelectionFrame;

/// `gmCWInterface::GetSelection` as the friends page takes it: the name of the selected player
/// when it is another player (not the local one), the add button's target. `None` otherwise.
pub(crate) fn friend_target(selection: Option<&SelectionFrame>) -> Option<&str> {
    selection
        .filter(|s| s.other_player)
        .and_then(|s| s.name.as_deref())
}

#[cfg(test)]
mod tests {
    use super::friend_target;
    use crate::gameplay::SelectionFrame;

    fn selection(name: Option<&str>, other_player: bool) -> SelectionFrame {
        SelectionFrame {
            id: 7,
            name: name.map(Into::into),
            name_info: None,
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
