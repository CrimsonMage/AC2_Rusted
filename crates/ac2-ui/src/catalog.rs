//! `gmShopCatalogUI`: the storekeeper's catalog of stores (`Catalog` of the gameplay layout), a
//! child framework made when the catalog opens and destroyed after it has been hidden for two
//! minutes (`ListenToMessage`'s global loop). `RecvNotice_ViewCatalog` opens it with the group's
//! header and one row per store (`PostShop`: the portrait, the name and, for a store the hash
//! sizes, the size); the player selects a row and enters its store (`EnterStore`, which asks
//! `RequestEnterStore` with the row's store id), goes to the consignment (`ConsignmentButton`), or
//! closes the catalog (`CatalogCloseButton`, `RequestLeaveCatalog`). `RecvNotice_LeaveCatalog`
//! closes it from the server's side.
//!
//! The presses come as [`CatalogUi::message`], the notices as the `recv_*` methods, and the global
//! loop as [`CatalogUi::tick`]. What the panel asks of the player is queued for
//! [`CatalogUi::take_requests`].
//!
//! Not modelled: the help page (`KeyStone::OpenHelp(ID_Help_ShopCatalog)`).

use std::sync::Arc;

use ac2_net::protocol::StringInfo;
use ac2_weenie::Shop;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::shop::{id, ShopSource, SELECTED};
use crate::token::{BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS, STATELESS};

/// `ListenToMessage`'s global loop: a hidden catalog waits this long before it is destroyed.
const DORMANT_LIMIT: f64 = 120.0;
/// The selection field's press data that selects (0x40000004, 0x40000005) and that also enters
/// the store (0x40000007, 0x40000008), as the catalog's `ListenToMessage` reads them.
const SELECT_PRESSES: [i32; 4] = [0x4000_0004, 0x4000_0005, 0x4000_0007, 0x4000_0008];
const ENTER_PRESSES: [i32; 2] = [0x4000_0007, 0x4000_0008];

/// What the catalog asks of the player: the `gmCWInterface` calls it makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogRequest {
    /// `gmCWInterface::RequestLeaveCatalog`.
    LeaveCatalog,
    /// `gmCWInterface::RequestEnterStore(didStore)`: the selected row's store.
    EnterStore(u32),
    /// `gmCWInterface::RequestEnterConsignment`.
    EnterConsignment,
}

/// `gmShopCatalogUI`'s state.
#[derive(Debug, Clone)]
pub struct CatalogUi {
    pub(crate) layout: Arc<LayoutDesc>,
    /// The catalog's root, while its managed elements exist (`m_pCatalogField`).
    pub(crate) field: Option<usize>,
    /// `IsShown`.
    pub(crate) shown: bool,
    /// `m_selectedElement`: the selected row's selection field.
    pub(crate) selected: Option<usize>,
    /// Each row's store id and selection field, in the list's order.
    pub(crate) rows: Vec<(u32, usize)>,
    /// `m_timeDormant`, on the global loop's clock.
    pub(crate) dormant_since: f64,
    requests: Vec<CatalogRequest>,
}

impl CatalogUi {
    /// `gmShopCatalogUI::gmShopCatalogUI`: hidden, with no managed elements; the gameplay layout is
    /// kept for them.
    pub fn create(gameplay: &Arc<LayoutDesc>) -> Self {
        Self {
            layout: Arc::clone(gameplay),
            field: None,
            shown: false,
            selected: None,
            rows: Vec::new(),
            dormant_since: 0.0,
            requests: Vec::new(),
        }
    }

    /// The catalog's root, while its managed elements exist.
    pub fn field(&self) -> Option<usize> {
        self.field
    }

    /// `IsShown`.
    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// The selected row's store id.
    pub fn selected_store(&self) -> Option<u32> {
        let selected = self.selected?;
        self.rows
            .iter()
            .find(|(_, row)| *row == selected)
            .map(|(store, _)| *store)
    }

    /// The store ids the list shows, in the list's order.
    pub fn store_ids(&self) -> Vec<u32> {
        self.rows.iter().map(|(store, _)| *store).collect()
    }

    /// The selection field of store `store`, while the list shows it.
    pub fn row_of(&self, store: u32) -> Option<usize> {
        self.rows
            .iter()
            .find(|(s, _)| *s == store)
            .map(|(_, row)| *row)
    }

    /// The requests the catalog made since the last call, for the client to send.
    pub fn take_requests(&mut self) -> Vec<CatalogRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The element `name` under the catalog.
    fn child(&self, tree: &UiTree, name: &str) -> Option<usize> {
        tree.child_recursive(self.field?, id(name))
    }

    /// `gmShopCatalogUI::Show`'s managed elements: the `Catalog` root, once.
    fn create_managed(&mut self, tree: &mut UiTree) -> bool {
        if self.field.is_none() {
            self.field = tree.create_root(Arc::clone(&self.layout), id("Catalog"));
        }
        self.field.is_some()
    }

    /// `gmShopCatalogUI::DestroyManagedElements`: the root and the rows go.
    fn destroy_managed(&mut self, tree: &mut UiTree) {
        if let Some(field) = self.field.take() {
            tree.remove_root(field);
        }
        self.selected = None;
        self.rows.clear();
    }

    /// `gmShopCatalogUI::Show`: shown makes the managed elements and shows them; hidden hides the
    /// root, which waits to be destroyed.
    pub(crate) fn show(&mut self, tree: &mut UiTree, shown: bool, now: f64) {
        self.shown = shown;
        if shown {
            if self.create_managed(tree) {
                if let Some(field) = self.field {
                    tree.set_visible(field, true);
                }
            }
            return;
        }
        if let Some(field) = self.field {
            tree.set_visible(field, false);
        }
        self.dormant_since = now;
    }

    /// `gmShopCatalogUI::RecvNotice_ViewCatalog`: the catalog opens with the group's header and
    /// its stores. A store with id 0 is selected when the list is built, as retail does.
    pub fn recv_view_catalog(
        &mut self,
        tree: &mut UiTree,
        s: &dyn ShopSource,
        header: &StringInfo,
        shops: &[Shop],
        now: f64,
    ) {
        self.show(tree, true, now);
        if self.field.is_none() {
            return;
        }
        if let Some(text) = self.child(tree, "StoreHeader") {
            tree.set_text(text, &s.render_wire(header));
        }
        if self.selected.is_some() {
            self.select(tree, None);
        }
        let Some(list) = self.child(tree, "StoreList") else {
            return;
        };
        tree.list_flush(list);
        self.rows.clear();
        tree.list_multi_op_start(list);
        for shop in shops {
            let Some(desc) = self.layout.element(id("StoreElement")).cloned() else {
                break;
            };
            let Some(row) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
                continue;
            };
            self.post_store(tree, s, shop, row);
            tree.list_add_item(list, row);
            let Some(field) = tree.child_recursive(row, id("SelectionField")) else {
                continue;
            };
            self.rows.push((shop.id, field));
            if shop.id == 0 {
                self.select(tree, Some(field));
            }
        }
        tree.list_multi_op_complete(list);
        self.ghost_enter(tree);
    }

    /// `gmShopCatalogUI::PostShop`: a row's portrait (with the store's description as its tooltip),
    /// its name, and its size when the hash gives one.
    fn post_store(&self, tree: &mut UiTree, s: &dyn ShopSource, shop: &Shop, row: usize) {
        if let Some(icon) = tree.child_recursive(row, id("StoreIcon")) {
            if shop.portrait != 0 {
                tree.set_image(icon, shop.portrait, 2);
                tree.set_visible(icon, true);
                tree.set_tooltip(icon, Some(s.render_wire(&shop.desc)));
            }
        }
        if let Some(name) = tree.child_recursive(row, id("StoreName")) {
            tree.set_text(name, &s.render_wire(&shop.name));
        }
        if shop.store_size > 0 {
            if let Some(size) = tree.child_recursive(row, id("StoreSize")) {
                tree.set_text(size, &shop.store_size.to_string());
            }
        }
    }

    /// `gmShopCatalogUI::RecvNotice_LeaveCatalog`: the selection is cleared and the catalog closes.
    pub fn recv_leave_catalog(&mut self, tree: &mut UiTree, now: f64) {
        if self.selected.is_some() {
            self.select(tree, None);
        }
        self.ghost_enter(tree);
        self.show(tree, false, now);
    }

    /// `gmShopCatalogUI::Select`: the row's selection state moves, and the enter button is live
    /// while a row is selected.
    pub(crate) fn select(&mut self, tree: &mut UiTree, row: Option<usize>) {
        if row != self.selected {
            if let Some(old) = self.selected {
                tree.set_state(old, STATELESS);
            }
            if let Some(new) = row {
                tree.set_state(new, id(SELECTED));
            }
            self.selected = row;
        }
        self.ghost_enter(tree);
    }

    /// The enter button is ghosted while no row is selected.
    fn ghost_enter(&self, tree: &mut UiTree) {
        if let Some(enter) = self.child(tree, "EnterStoreButton") {
            tree.set_bool(enter, BUTTON_GHOSTED, self.selected.is_none());
        }
    }

    /// `gmShopCatalogUI::EnterStore`: the selected row's store is entered.
    fn enter_store(&mut self) {
        if let Some(store) = self.selected_store() {
            self.requests.push(CatalogRequest::EnterStore(store));
        }
    }

    /// `gmShopCatalogUI::ListenToMessage` for the global loop (`now`): a hidden catalog is
    /// destroyed after its time.
    pub fn tick(&mut self, tree: &mut UiTree, now: f64) {
        if !self.shown && self.field.is_some() && DORMANT_LIMIT < now - self.dormant_since {
            self.destroy_managed(tree);
        }
    }

    /// `gmShopCatalogUI::ListenToMessage`: the close, enter and consignment buttons, and a press on
    /// a row (a press that selects it, and one that enters its store).
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage, now: f64) {
        if !self.shown {
            return;
        }
        let element = message.element_id;
        let release = message.message == MSG_BUTTON_RELEASE_ON;
        if element == id("CatalogCloseButton") {
            if release {
                self.requests.push(CatalogRequest::LeaveCatalog);
                self.recv_leave_catalog(tree, now);
            }
        } else if element == id("EnterStoreButton") {
            if release {
                self.enter_store();
            }
        } else if element == id("ConsignmentButton") {
            if release {
                self.requests.push(CatalogRequest::EnterConsignment);
            }
        } else if element == id("SelectionField")
            && message.message == MSG_MOUSE_PRESS
            && SELECT_PRESSES.contains(&message.data)
        {
            if let Some(row) = message.element {
                self.select(tree, Some(row));
            }
            if ENTER_PRESSES.contains(&message.data) {
                self.enter_store();
            }
        }
    }
}
