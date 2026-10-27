//! `gmToolbarUI`'s mount and sale buttons, and the toolbar's show, on the real gameplay layout
//! (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::token::tokenize;
use crate::{build_gameplay_hud, GameplayHud, UiTree};

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

/// The button named `name` under the toolbar field.
fn button(tree: &UiTree, name: &str) -> usize {
    let field = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("ToolbarField"))
        })
        .expect("the toolbar field");
    tree.child_recursive(field, tokenize(name))
        .unwrap_or_else(|| panic!("{name}"))
}

#[test]
fn the_sale_reminder_shows_the_consignments_button_in_its_state() {
    let Some(mut hud) = hud() else {
        return;
    };
    let consignments = button(&hud.tree, "ConsignmentsButton");
    // Hidden until a sale needs it.
    assert!(!hud.tree.get(consignments).unwrap().visible());
    hud.toolbar.sale_reminder(&mut hud.tree, true);
    let shown = hud.tree.get(consignments).unwrap();
    assert!(shown.visible());
    assert_eq!(shown.state(), tokenize("consignments"));
    hud.toolbar.sale_reminder(&mut hud.tree, false);
    let hidden = hud.tree.get(consignments).unwrap();
    assert!(!hidden.visible());
    assert_eq!(hidden.state(), tokenize("stateless"));
}

#[test]
fn the_mount_state_shows_the_dismount_button_in_its_state() {
    let Some(mut hud) = hud() else {
        return;
    };
    let dismount = button(&hud.tree, "DismountButton");
    assert!(!hud.tree.get(dismount).unwrap().visible());
    hud.toolbar.mount_state(&mut hud.tree, true);
    let shown = hud.tree.get(dismount).unwrap();
    assert!(shown.visible());
    assert_eq!(shown.state(), tokenize("dismount"));
    hud.toolbar.mount_state(&mut hud.tree, false);
    assert!(!hud.tree.get(dismount).unwrap().visible());
}

/// `gmToolbarUI::ListenToMessage`: the dismount button's release is the dismount press; the
/// other buttons' releases are not.
#[test]
fn the_dismount_button_release_is_the_dismount_press() {
    use crate::mouse::UiMessage;
    use crate::token::MSG_BUTTON_RELEASE_ON;
    let Some(hud) = hud() else {
        return;
    };
    let press = |element_id: &str, message: u32| UiMessage {
        element_id: tokenize(element_id),
        element: None,
        message,
        data: 0,
        item: None,
    };
    assert!(hud
        .toolbar
        .is_dismount_press(&press("DismountButton", MSG_BUTTON_RELEASE_ON)));
    assert!(!hud.toolbar.is_dismount_press(&press("DismountButton", 0)));
    assert!(!hud
        .toolbar
        .is_dismount_press(&press("ConsignmentsButton", MSG_BUTTON_RELEASE_ON)));
}

#[test]
fn showing_the_toolbar_precludes_its_height_from_the_bottom_until_it_hides() {
    let Some(mut hud) = hud() else {
        return;
    };
    let height = hud.toolbar.show_bar(&mut hud.tree, true);
    assert!(height > 0, "the toolbar's PrecludeHeight");
    assert_eq!(hud.toolbar.show_bar(&mut hud.tree, false), 0);
}
