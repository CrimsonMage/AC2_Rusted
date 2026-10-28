//! `gmFriendsUI`'s add and remove buttons on the real social layout (`AC2_DAT_DIR`): the game
//! selection's target (`UpdateButtonStates`), the add request and the list selection.

use ac2_db::ResourceStore;
use ac2_weenie::{FriendInfo, MODE_AWAY, MODE_DO_NOT_DISTURB};

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

fn with_mode(name: &str, mode: u32) -> FriendInfo {
    FriendInfo {
        name: name.to_owned(),
        location: String::new(),
        mode,
    }
}

/// A listed row's `FriendName` text and state.
fn row_text_and_state(tree: &UiTree, row: usize) -> (String, u32) {
    let name = tree
        .child_recursive(row, tokenize("FriendName"))
        .expect("the friend's name");
    let text = tree.get(name).and_then(|e| e.text()).unwrap_or_default();
    (text, tree.get(row).unwrap().state())
}

#[test]
fn a_friends_mode_splits_online_and_sets_its_row_text_and_state() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut panel: FriendsPanel = hud.friends.take().expect("FriendsPage");
    let tree = &mut hud.tree;
    // The toolbar's string table, as the page's drop-down entries are read from.
    let table = hud
        .toolbar_layout
        .as_ref()
        .expect("the toolbar layout")
        .string_table;
    let text = |id: &str| {
        let units = tree
            .text_data
            .as_ref()
            .and_then(|data| data.string(table, tokenize(id)));
        String::from_utf16_lossy(&units.expect(id))
    };
    let away = text("ID_FriendStatus_AwayFromKeyboardMessage");
    let dnd = text("ID_FriendStatus_DoNotDisturbMessage");
    panel.recv_list(
        tree,
        &[
            with_mode("Away", MODE_AWAY),
            with_mode("Busy", MODE_DO_NOT_DISTURB),
            with_mode("Here", 1),
            with_mode("Gone", 0),
            // Away wins over do-not-disturb, as BuildUIList tests the away bit first.
            with_mode("Both", MODE_AWAY | MODE_DO_NOT_DISTURB),
        ],
    );
    assert_eq!(panel.listed(), ["Away", "Busy", "Here", "Both", "Gone"]);
    let rows = panel.row_roots();
    let check = |tree: &UiTree, row: usize, name: &str, suffix: &str, state: &str| {
        let (got_text, got_state) = row_text_and_state(tree, row);
        assert_eq!(got_text, format!("{name}{suffix}"), "{name}");
        assert_eq!(got_state, tokenize(state), "{name}");
    };
    check(tree, rows[0], "Away", &away, "token_awayfromkeyboard");
    check(tree, rows[1], "Busy", &dnd, "token_donotdisturb");
    check(tree, rows[2], "Here", "", "token_online");
    check(tree, rows[3], "Both", &away, "token_awayfromkeyboard");
    check(tree, rows[4], "Gone", "", "token_offline");
}

#[test]
fn the_server_local_status_selects_the_drop_down_and_sends_its_status() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut panel: FriendsPanel = hud.friends.take().expect("FriendsPage");
    let tree = &mut hud.tree;
    let menu = find(tree, "StatusDropDownMenu");
    tree.take_messages();
    // Do-not-disturb (4) selects entry 1; the broadcast asks the client for the status change.
    panel.recv_local_status(tree, MODE_DO_NOT_DISTURB);
    assert_eq!(tree.menu_selected_index(menu), Some(1));
    let requests: Vec<FriendsRequest> = tree
        .take_messages()
        .iter()
        .flat_map(|m| panel.message(tree, m, None))
        .collect();
    assert_eq!(
        requests,
        [FriendsRequest::Status {
            flags: MODE_DO_NOT_DISTURB,
            emote: Some("ID_EmoteYawnCmd")
        }]
    );
    // A status the table does not hold selects nothing.
    panel.recv_local_status(tree, 99);
    assert_eq!(tree.menu_selected_index(menu), Some(1));
}
