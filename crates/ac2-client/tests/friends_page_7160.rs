//! The friends page on the real social layout (`AC2_DAT_DIR`): the GUN list fills its rows with
//! the online friends first, then the offline ones, and its add and remove buttons ask the
//! client. The page's element names are the admin 7148 PDB's (`FriendsPage`, `FriendsListBox`,
//! `FriendsAddButton`, `FriendsRemoveButton`, `FriendTemplate`).

use std::sync::Arc;

use ac2_ui::{
    build_gameplay_hud, status_request, tokenize, FriendsPanel, FriendsRequest, UiMessage, UiTree,
    MSG_BUTTON_RELEASE_ON,
};
use ac2_weenie::FriendInfo;

fn friend(name: &str, location: &str, mode: u32) -> FriendInfo {
    FriendInfo {
        name: name.to_owned(),
        location: location.to_owned(),
        mode,
    }
}

fn release(tree: &UiTree, page: usize, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: tree.child_recursive(page, tokenize(name)),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

/// The `FriendsPage` element, the root the buttons hang under.
fn page_root(tree: &UiTree) -> usize {
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("FriendsPage")))
        .expect("the friends page")
}

#[test]
fn the_list_lists_the_online_friends_then_the_offline_ones() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut page: FriendsPanel = hud.friends.take().expect("the friends page");
    assert!(page.has_page());
    page.recv_list(
        &mut hud.tree,
        &[friend("Ilse", "", 0), friend("Tomas", "Holtburg", 1)],
    );
    assert_eq!(page.listed(), ["Tomas", "Ilse"]);

    // A status with a location moves Ilse into the online section, in the GUN list's order.
    page.recv_status(&mut hud.tree, &friend("Ilse", "Yaraq", 2));
    assert_eq!(page.listed(), ["Ilse", "Tomas"]);

    // The remove button asks for nothing without a selected friend.
    let root = page_root(&hud.tree);
    let remove = release(&hud.tree, root, "FriendsRemoveButton");
    let requests = page.message(&mut hud.tree, &remove, None);
    assert!(requests.is_empty());

    // The add button asks for the target when there is one.
    let add = release(&hud.tree, root, "FriendsAddButton");
    let requests = page.message(&mut hud.tree, &add, Some("Maria"));
    assert_eq!(requests, vec![FriendsRequest::Add("Maria".to_owned())]);
    assert!(page.message(&mut hud.tree, &add, None).is_empty());

    // A removed friend leaves the rows; an added one asks for the list again.
    page.recv_removed(&mut hud.tree, "Tomas");
    assert_eq!(page.listed(), ["Ilse"]);
    assert_eq!(
        page.recv_added(),
        FriendsRequest::GetList {
            user_requested: true
        }
    );
    assert_eq!(
        status_request(2),
        Some(FriendsRequest::Status {
            flags: 2,
            emote: Some("ID_EmoteAFKCmd")
        })
    );
}

#[test]
fn the_status_drop_down_sends_the_chosen_status() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut page: FriendsPanel = hud.friends.take().expect("the friends page");
    let root = page_root(&hud.tree);
    let menu = hud
        .tree
        .child_recursive(root, tokenize("StatusDropDownMenu"))
        .expect("the status drop-down");
    // The page's own selection when it was made sends nothing; the first status is selected.
    assert_eq!(hud.tree.menu_selected_index(menu), Some(0));
    assert!(!hud
        .tree
        .take_messages()
        .iter()
        .any(|message| message.element == Some(menu)));

    // A choice in the drop-down (the menu's new selection) is the status request.
    hud.tree.menu_set_selected_index(menu, Some(2));
    let requests: Vec<FriendsRequest> = hud
        .tree
        .take_messages()
        .iter()
        .flat_map(|message| page.message(&mut hud.tree, message, None))
        .collect();
    assert_eq!(
        requests,
        vec![FriendsRequest::Status {
            flags: 2,
            emote: Some("ID_EmoteAFKCmd")
        }]
    );
}
