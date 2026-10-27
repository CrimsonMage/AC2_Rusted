//! `gmSocialUI` on the real toolbar layout (`AC2_DAT_DIR`): the field starts hidden, the toggle and
//! close buttons show and hide it, and the help button asks for the social page's help.

use crate::social_page::{SocialAction, SOCIAL_HELP_TOPIC};
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};
use crate::{build_gameplay_hud, UiMessage, UiTree};
use ac2_db::ResourceStore;

fn release(tree: &UiTree, name: &str) -> UiMessage {
    let root = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize(name)));
    UiMessage {
        element_id: tokenize(name),
        element: root,
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

fn field_visible(tree: &UiTree) -> bool {
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("SocialField")))
        .and_then(|field| tree.get(field))
        .is_some_and(|field| field.visible())
}

#[test]
fn the_social_panel_toggles_closes_and_asks_for_help() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.social.as_mut().expect("the social panel");
    // `gmSocialUI::gmSocialUI` shows the panel hidden.
    assert!(!panel.shown());
    assert!(!field_visible(&hud.tree));

    let toggle = release(&hud.tree, "SocialButton");
    assert_eq!(panel.message(&mut hud.tree, &toggle), None);
    assert!(panel.shown());
    assert!(field_visible(&hud.tree));

    let help = release(&hud.tree, "SocialHelpButton");
    assert_eq!(
        panel.message(&mut hud.tree, &help),
        Some(SocialAction::OpenHelp(SOCIAL_HELP_TOPIC))
    );
    assert!(panel.shown());

    let close = release(&hud.tree, "SocialCloseButton");
    panel.message(&mut hud.tree, &close);
    assert!(!panel.shown());
    assert!(!field_visible(&hud.tree));
}
