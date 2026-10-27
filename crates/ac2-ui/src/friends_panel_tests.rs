//! `gmFriendsUI`'s add and remove buttons on the real social layout (`AC2_DAT_DIR`): the game
//! selection's target (`UpdateButtonStates`), the add request and the list selection.

use ac2_db::ResourceStore;
use ac2_weenie::FriendInfo;

use crate::element::UiTree;
use crate::friends::FriendsRequest;
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};
use crate::{build_gameplay_hud, FriendsPanel, GameplayHud};

/// The friend list's rows: the online marker, the offline marker, then the friend `Alpha`.
const ALPHA_ROW: usize = 2;

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

fn friend(name: &str) -> FriendInfo {
    FriendInfo {
        name: name.to_owned(),
        location: String::new(),
        mode: 0,
    }
}

/// An element of the friends page by its id, found under the roots.
fn find(tree: &UiTree, name: &str) -> usize {
    tree.roots()
        .iter()
        .find_map(|&root| tree.child_recursive(root, tokenize(name)))
        .unwrap_or_else(|| panic!("{name}"))
}

fn ghosted(tree: &UiTree, element: usize) -> bool {
    matches!(
        tree.get(element).and_then(|e| e.attribute(BUTTON_GHOSTED)),
        Some(crate::layout::AttributeValue::Bool(true))
    )
}

fn message(element_id: &str, element: Option<usize>, message: u32) -> UiMessage {
    UiMessage {
        element_id: tokenize(element_id),
        element,
        message,
        data: 0,
        item: None,
    }
}

#[test]
fn the_add_button_takes_a_player_the_selection_names_who_is_not_a_friend() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut panel: FriendsPanel = hud.friends.take().expect("FriendsPage");
    let tree = &mut hud.tree;
    panel.recv_list(tree, &[friend("Alpha")]);
    let add = find(tree, "FriendsAddButton");
    assert!(ghosted(tree, add), "no target: add is ghosted");
    panel.set_target(tree, Some("Birch"));
    assert!(!ghosted(tree, add), "a player who is not a friend");
    // The press asks to add the target by name.
    let press = message("FriendsAddButton", Some(add), MSG_BUTTON_RELEASE_ON);
    assert_eq!(
        panel.message(tree, &press, Some("Birch")),
        [FriendsRequest::Add("Birch".to_owned())]
    );
    // A friend is not added again; no selection and an empty name ghost it.
    panel.set_target(tree, Some("Alpha"));
    assert!(ghosted(tree, add), "already a friend");
    panel.set_target(tree, None);
    assert!(ghosted(tree, add), "nothing selected");
    panel.set_target(tree, Some(""));
    assert!(ghosted(tree, add), "no name");
}

#[test]
fn a_selected_list_row_keeps_add_ghosted_and_selects_the_friend_for_remove() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut panel: FriendsPanel = hud.friends.take().expect("FriendsPage");
    let tree = &mut hud.tree;
    panel.recv_list(tree, &[friend("Alpha")]);
    let add = find(tree, "FriendsAddButton");
    let remove = find(tree, "FriendsRemoveButton");
    let list = find(tree, "FriendsListBox");
    panel.set_target(tree, Some("Birch"));
    assert!(!ghosted(tree, add));
    // Selecting the friend's row takes the selection: remove is enabled, add ghosts.
    tree.list_set_selected_index(list, Some(ALPHA_ROW), true);
    panel.message(tree, &message("", None, 0), Some("Birch"));
    assert!(!ghosted(tree, remove), "a friend is selected");
    assert!(ghosted(tree, add), "a row is selected");
    // Clearing the selection brings the target back to the add button.
    tree.list_set_selected_index(list, None, true);
    panel.message(tree, &message("", None, 0), Some("Birch"));
    assert!(ghosted(tree, remove));
    assert!(!ghosted(tree, add));
}
