//! `gmShopCatalogUI` on the real gameplay layout (`AC2_DAT_DIR`): the list of stores, the
//! selection and the enter, the consignment and the close buttons, the server's leave and the
//! global loop's destruction.

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::StringInfo;
use ac2_weenie::Shop;

use crate::catalog::{CatalogRequest, CatalogUi};
use crate::mouse::UiMessage;
use crate::shop::ShopSource;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS};
use crate::{build_gameplay_hud, GameplayHud, UiTree};

const SHEILA: u32 = 0x6F00_321C;
const BOB: u32 = 0x6F00_3300;
const ROW_SELECT: i32 = 0x4000_0004;
const ROW_ENTER: i32 = 0x4000_0007;

struct Wire;

impl crate::ui_string::UiStrings for Wire {
    fn render_string(&self, string: &crate::ui_string::UiString) -> String {
        format!("{:08x}", string.string_id)
    }
}

impl ShopSource for Wire {
    fn render_wire(&self, info: &StringInfo) -> String {
        info.literal.clone().unwrap_or_default()
    }
}

fn literal(text: &str) -> StringInfo {
    StringInfo {
        literal: Some(text.into()),
        ..StringInfo::default()
    }
}

/// Two stores: the first with a size of 5 (its hash names it), the second with none.
fn shops() -> Vec<Shop> {
    let shop = |id: u32, name: &str, size: i32| Shop {
        id,
        name: literal(name),
        desc: literal("A store"),
        sorter: StringInfo::default(),
        portrait: 0,
        sales: Vec::new(),
        position: 0,
        store_size: size,
    };
    vec![
        shop(SHEILA, "Sheila's Wares", 5),
        shop(BOB, "Bob's Bits", 0),
    ]
}

/// The gameplay HUD with its catalog taken out, so the catalog and the HUD can both be borrowed.
fn setup() -> Option<(GameplayHud, CatalogUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let catalog = hud.catalog.take().expect("the catalog panel");
    Some((hud, catalog))
}

/// A button's release, with its element when the panel has one.
fn release(tree: &UiTree, root: Option<usize>, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: root.and_then(|r| tree.child_recursive(r, tokenize(name))),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

/// A press on the selection field of `row` (`data` is the press's action).
fn press_row(row: usize, data: i32) -> UiMessage {
    UiMessage {
        element_id: tokenize("SelectionField"),
        element: Some(row),
        message: MSG_MOUSE_PRESS,
        data,
        ..UiMessage::default()
    }
}

/// The row a selection field sits in (the list's element for the store).
fn row_of_field(tree: &UiTree, field: usize) -> usize {
    tree.get(field)
        .and_then(|e| e.parent_index())
        .expect("the row")
}

fn text_of(tree: &UiTree, parent: usize, name: &str) -> Option<String> {
    let element = tree.child_recursive(parent, tokenize(name))?;
    tree.get(element)?.text()
}

fn opened(hud: &mut GameplayHud, catalog: &mut CatalogUi) {
    catalog.recv_view_catalog(
        &mut hud.tree,
        &Wire,
        &literal("Stores of Ghan"),
        &shops(),
        0.0,
    );
}

#[test]
fn view_catalog_lists_the_stores_with_their_names_and_sizes() {
    let Some((mut hud, mut catalog)) = setup() else {
        return;
    };
    opened(&mut hud, &mut catalog);
    assert!(catalog.is_shown());
    let field = catalog.field().expect("the catalog's root");
    assert_eq!(
        catalog.store_ids(),
        vec![SHEILA, BOB],
        "the stores in order"
    );
    assert_eq!(
        text_of(&hud.tree, field, "StoreHeader").as_deref(),
        Some("Stores of Ghan")
    );
    let sheila = row_of_field(&hud.tree, catalog.row_of(SHEILA).expect("Sheila's row"));
    assert_eq!(
        text_of(&hud.tree, sheila, "StoreName").as_deref(),
        Some("Sheila's Wares")
    );
    assert_eq!(
        text_of(&hud.tree, sheila, "StoreSize").as_deref(),
        Some("5")
    );
    let bob = row_of_field(&hud.tree, catalog.row_of(BOB).expect("Bob's row"));
    assert_eq!(
        text_of(&hud.tree, bob, "StoreSize").as_deref(),
        Some(""),
        "a store the hash does not size keeps the size text empty"
    );
    assert!(catalog.take_requests().is_empty());
}

#[test]
fn selecting_a_store_enables_the_enter_button_and_enters_it() {
    let Some((mut hud, mut catalog)) = setup() else {
        return;
    };
    opened(&mut hud, &mut catalog);
    let field = catalog.field().unwrap();
    let enter = hud
        .tree
        .child_recursive(field, tokenize("EnterStoreButton"))
        .unwrap();
    assert!(
        hud.tree.bool_attribute(enter, BUTTON_GHOSTED),
        "no row selected"
    );
    let bob = catalog.row_of(BOB).unwrap();
    let mut message = press_row(bob, ROW_SELECT);
    catalog.message(&mut hud.tree, &message, 1.0);
    assert_eq!(catalog.selected_store(), Some(BOB));
    assert!(!hud.tree.bool_attribute(enter, BUTTON_GHOSTED));
    message = release(&hud.tree, Some(field), "EnterStoreButton");
    catalog.message(&mut hud.tree, &message, 1.0);
    assert_eq!(
        catalog.take_requests(),
        vec![CatalogRequest::EnterStore(BOB)]
    );
}

#[test]
fn a_double_press_on_a_row_enters_its_store() {
    let Some((mut hud, mut catalog)) = setup() else {
        return;
    };
    opened(&mut hud, &mut catalog);
    let sheila = catalog.row_of(SHEILA).unwrap();
    catalog.message(&mut hud.tree, &press_row(sheila, ROW_ENTER), 1.0);
    assert_eq!(
        catalog.take_requests(),
        vec![CatalogRequest::EnterStore(SHEILA)]
    );
}

#[test]
fn consignment_and_close_buttons_send_their_requests() {
    let Some((mut hud, mut catalog)) = setup() else {
        return;
    };
    opened(&mut hud, &mut catalog);
    let field = catalog.field().unwrap();
    let consignment = release(&hud.tree, Some(field), "ConsignmentButton");
    catalog.message(&mut hud.tree, &consignment, 2.0);
    assert_eq!(
        catalog.take_requests(),
        vec![CatalogRequest::EnterConsignment]
    );
    assert!(
        catalog.is_shown(),
        "the consignment button keeps the catalog open"
    );
    let close = release(&hud.tree, Some(field), "CatalogCloseButton");
    catalog.message(&mut hud.tree, &close, 3.0);
    assert_eq!(catalog.take_requests(), vec![CatalogRequest::LeaveCatalog]);
    assert!(!catalog.is_shown());
}

#[test]
fn the_servers_leave_hides_the_catalog_then_the_loop_destroys_it() {
    let Some((mut hud, mut catalog)) = setup() else {
        return;
    };
    opened(&mut hud, &mut catalog);
    let field = catalog.field().unwrap();
    let bob = catalog.row_of(BOB).unwrap();
    catalog.message(&mut hud.tree, &press_row(bob, ROW_SELECT), 1.0);
    catalog.recv_leave_catalog(&mut hud.tree, 10.0);
    assert!(!catalog.is_shown());
    assert_eq!(catalog.selected_store(), None, "the selection is cleared");
    assert!(
        catalog.take_requests().is_empty(),
        "the server closed it; nothing is sent"
    );
    catalog.tick(&mut hud.tree, 100.0);
    assert_eq!(
        catalog.field(),
        Some(field),
        "still waiting to be destroyed"
    );
    catalog.tick(&mut hud.tree, 131.0);
    assert_eq!(catalog.field(), None, "destroyed after two minutes hidden");
    assert!(catalog.store_ids().is_empty());
}
