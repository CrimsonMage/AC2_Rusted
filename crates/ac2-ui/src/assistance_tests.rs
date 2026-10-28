//! `gmAssistanceUI` on the real toolbar layout (`AC2_DAT_DIR`): the call-type menu, the checks
//! before a send, and the answer to a sent call.

use crate::dialog::{DialogFactory, DIALOG_MESSAGE};
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};
use crate::{build_gameplay_hud, AssistancePanel, AssistanceRequest, UiTree};
use ac2_db::ResourceStore;

const ABUSE: u32 = 0xE000;
const LAG: u32 = 0x0004;

fn page(tree: &UiTree) -> usize {
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("AssistancePage")))
        .expect("the assistance page")
}

/// The call-type item whose `GUNCallType` is `call_type`.
fn item_for(tree: &UiTree, menu: usize, call_type: u32) -> usize {
    let items = tree.menu_items(menu).to_vec();
    items
        .into_iter()
        .find(|&item| {
            matches!(
                tree.get(item).and_then(|e| e.attribute(tokenize("GUNCallType"))),
                Some(crate::layout::AttributeValue::Int(v)) if *v as u32 == call_type
            )
        })
        .expect("the call type's item")
}

fn release(tree: &UiTree, page: usize, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: tree.child_recursive(page, tokenize(name)),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

/// Every message the tree queued, taken by the page (its requests in order).
fn pump(
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    panel: &mut AssistancePanel,
) -> Vec<AssistanceRequest> {
    let messages = tree.take_messages();
    messages
        .iter()
        .filter_map(|m| panel.message(tree, dialogs, m, ""))
        .collect()
}

#[test]
fn the_call_type_menu_holds_the_nine_types_and_reset_ghosts_send() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let page = page(&hud.tree);
    let menu = hud
        .tree
        .child_recursive(page, tokenize("CallTypeMenu"))
        .unwrap();
    let send = hud
        .tree
        .child_recursive(page, tokenize("AssistanceSendButton"))
        .unwrap();
    let mut panel = hud.assistance.take().expect("the assistance page");
    assert_eq!(hud.tree.menu_items(menu).len(), 9);
    assert_eq!(hud.tree.menu_selected_index(menu), None);
    assert!(hud.tree.bool_attribute(send, BUTTON_GHOSTED));
    // `InsertTextItem` at index 0 each time: the list shows the types in reverse of
    // `InitCallTypeMenu`'s order (Legions 0x00444130), so Abuse is first and Other last.
    let call_type = |index: usize| {
        let item = hud.tree.menu_item(menu, index).unwrap();
        hud.tree
            .get(item)
            .unwrap()
            .attribute(tokenize("GUNCallType"))
            .cloned()
    };
    assert_eq!(
        call_type(0),
        Some(crate::layout::AttributeValue::Int(0xE000))
    );
    assert_eq!(
        call_type(8),
        Some(crate::layout::AttributeValue::Int(0x8008))
    );
    // A choice unghosts the send button.
    hud.tree.menu_set_selected_index(menu, Some(0));
    let _ = pump(&mut hud.tree, &mut hud.dialogs, &mut panel);
    assert!(!hud.tree.bool_attribute(send, BUTTON_GHOSTED));
}

#[test]
fn send_checks_the_abuser_then_the_description_and_answers_the_call() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let root = page(&hud.tree);
    let menu = hud
        .tree
        .child_recursive(root, tokenize("CallTypeMenu"))
        .unwrap();
    let abuser = hud
        .tree
        .child_recursive(root, tokenize("AbuserNameEntryText"))
        .unwrap();
    let description = hud
        .tree
        .child_recursive(root, tokenize("AssistanceDescriptionEntryText"))
        .unwrap();
    let mut panel = hud.assistance.take().expect("the assistance page");
    let send = release(&hud.tree, root, "AssistanceSendButton");

    // Abuse (0xE000) needs an abuser name: none, so its message box and no call.
    let item = item_for(&hud.tree, menu, ABUSE);
    let index = hud
        .tree
        .menu_items(menu)
        .iter()
        .position(|&i| i == item)
        .unwrap();
    hud.tree.menu_set_selected_index(menu, Some(index));
    pump(&mut hud.tree, &mut hud.dialogs, &mut panel);
    let requests = panel.message(&mut hud.tree, &mut hud.dialogs, &send, "");
    assert!(requests.is_none());
    let (context, kind) = hud.dialogs.shown().expect("the abuser box");
    assert_eq!(kind, DIALOG_MESSAGE);
    assert!(context > 0);

    // With a name, the description is still missing (0xE000 has the description bit too).
    hud.tree.set_text(abuser, "Ilse");
    let requests = panel.message(&mut hud.tree, &mut hud.dialogs, &send, "");
    assert!(requests.is_none());
    let (second, _) = hud.dialogs.shown().expect("the description box");
    assert_ne!(second, context);

    // Complete: the call is the request, and the answer decides the page.
    hud.tree.set_text(description, "Speeding");
    let request = panel
        .message(&mut hud.tree, &mut hud.dialogs, &send, "Legions")
        .expect("the call");
    assert_eq!(
        request,
        AssistanceRequest::CallCreate {
            call_type: ABUSE,
            world: "Legions".into(),
            target: "Ilse".into(),
            description: "Speeding".into(),
        }
    );
    // Refused by the GUN service: the "can't connect" box, the entries kept.
    panel.send_answered(&mut hud.tree, &mut hud.dialogs, false);
    assert_eq!(
        hud.tree.get(abuser).and_then(|e| e.text()).as_deref(),
        Some("Ilse")
    );
    assert!(hud.dialogs.shown().is_some());
    // Queued: the page resets.
    panel.send_answered(&mut hud.tree, &mut hud.dialogs, true);
    assert_eq!(hud.tree.menu_selected_index(menu), None);
    assert!(hud.tree.bool_attribute(
        hud.tree
            .child_recursive(root, tokenize("AssistanceSendButton"))
            .unwrap(),
        BUTTON_GHOSTED
    ));
}

#[test]
fn a_call_type_without_requirements_sends_straight_away() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let root = page(&hud.tree);
    let menu = hud
        .tree
        .child_recursive(root, tokenize("CallTypeMenu"))
        .unwrap();
    let mut panel = hud.assistance.take().expect("the assistance page");
    let item = item_for(&hud.tree, menu, LAG);
    let index = hud
        .tree
        .menu_items(menu)
        .iter()
        .position(|&i| i == item)
        .unwrap();
    hud.tree.menu_set_selected_index(menu, Some(index));
    pump(&mut hud.tree, &mut hud.dialogs, &mut panel);
    let send = release(&hud.tree, root, "AssistanceSendButton");
    let request = panel.message(&mut hud.tree, &mut hud.dialogs, &send, "");
    assert_eq!(
        request,
        Some(AssistanceRequest::CallCreate {
            call_type: LAG,
            world: String::new(),
            target: String::new(),
            description: String::new(),
        })
    );
    assert!(hud.dialogs.shown().is_none());
}
