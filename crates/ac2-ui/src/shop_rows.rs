//! `gmShopUI`'s sales: the list rebuilt from a store's sales (`UpdateSales`), each sale's element
//! (`PostSale`), a sale's stock text (`RecvNotice_UpdateSale`) and the page buttons
//! (`UpdatePosition`). A sale's icon and texts come from the notice: the item's icon DID, its
//! description and the cost; the icon's items are not looked up (their names may be unknown).

use std::sync::Arc;

use ac2_weenie::{Shop, ShopSale};

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::shop::{id, ShopSource, ShopUi};
use crate::skill_tree::SkillVar;
use crate::token::BUTTON_GHOSTED;
use crate::ui_string::UiString;

/// `Token_unavailable`: a sale or button that cannot be bought from.
const UNAVAILABLE: &str = "unavailable";
/// `Token_SourceContext`: the drag's context (the shop's).
const SOURCE_CONTEXT: &str = "SourceContext";
/// `Token_ShortcutData`: the item a drag carries.
const SHORTCUT_DATA: &str = "Token_ShortcutData";

impl ShopUi {
    /// `gmShopUI::UpdateSales`: the page's position and size, then the list rebuilt from the
    /// sales in their order. The selected sale stays selected while it is listed.
    pub(crate) fn update_sales(&mut self, tree: &mut UiTree, s: &dyn ShopSource, shop: &Shop) {
        self.update_position(tree, shop.position, shop.store_size);
        let Some(list) = self.child(tree, "ItemList") else {
            return;
        };
        let kept = self.selected_item();
        if self.selected.is_some() {
            self.select(tree, None);
        }
        tree.list_flush(list);
        self.rows.clear();
        tree.list_multi_op_start(list);
        for sale in &shop.sales {
            let Some(desc) = self.layout.element(id("Sale")).cloned() else {
                break;
            };
            let Some(row) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
                continue;
            };
            self.post_sale(tree, s, sale, row);
            tree.list_add_item(list, row);
            self.rows.retain(|(item, _)| *item != sale.item);
            self.rows.push((sale.item, row));
            if kept == Some(sale.item) {
                self.select(tree, Some(row));
            }
        }
        tree.list_multi_op_complete(list);
        if self.selected.is_none() {
            if let Some(purchase) = self.child(tree, "PurchaseButton") {
                tree.set_bool(purchase, BUTTON_GHOSTED, true);
            }
        }
    }

    /// `gmShopUI::PostSale`: a sale's element shows its item (icon, description, tooltip), its
    /// cost (icon and quantity) and its stock. The item icon is a drag source for the shop's
    /// context; a restricted, sold-out sale is unavailable.
    fn post_sale(&mut self, tree: &mut UiTree, s: &dyn ShopSource, sale: &ShopSale, row: usize) {
        if let Some(product) = tree.child_recursive(row, id("ProductIcon")) {
            if let Some(icon) = tree.child_recursive(product, id("ItemIcon")) {
                if sale.icon_did != 0 {
                    tree.set_image(icon, sale.icon_did, 2);
                }
                tree.set_tooltip(icon, Some(s.render_wire(&sale.item_desc)));
                let attributes = &mut tree.el_mut(icon).attributes;
                attributes.insert(id(SHORTCUT_DATA), AttributeValue::InstanceId(sale.item));
                attributes.insert(id(SOURCE_CONTEXT), AttributeValue::Int(self.context as i32));
                tree.set_bool(icon, id("dragable"), true);
            }
        }
        if let Some(desc) = tree.child_recursive(row, id("ProductDesc")) {
            tree.set_text(desc, &s.render_wire(&sale.item_desc));
        }
        if let Some(cost_icon) = tree.child_recursive(row, id("CostIcon")) {
            if sale.cost_icon != 0 {
                tree.set_image(cost_icon, sale.cost_icon, 0);
                tree.set_visible(cost_icon, true);
                tree.set_tooltip(cost_icon, Some(s.render_wire(&sale.cost_desc)));
            }
        }
        if let Some(cost) = tree.child_recursive(row, id("CostQuantity")) {
            let quantity = if sale.cost_quantity.fract() == 0.0 {
                SkillVar::Int(i64::from(sale.cost_quantity as i32))
            } else {
                SkillVar::Float {
                    value: f64::from(sale.cost_quantity),
                    precision: 2,
                }
            };
            let text = UiString::ui(id("ID_Store_Quantity"), vec![(id("Quantity"), quantity)]);
            tree.set_text(cost, &s.render_string(&text));
        }
        if let Some(stock) = tree.child_recursive(row, id("StockQuantity")) {
            let text = if sale.status == 1 {
                tree.set_bool(row, id(UNAVAILABLE), true);
                UiString::ui(id("ID_Store_Restricted"), Vec::new())
            } else if sale.stock < 1 {
                if sale.stock == 0 {
                    tree.set_bool(row, id(UNAVAILABLE), true);
                    UiString::ui(id("ID_Store_SoldOut"), Vec::new())
                } else {
                    UiString::ui(id("ID_Store_Unlimited"), Vec::new())
                }
            } else {
                let quantity = SkillVar::Int(i64::from(sale.stock));
                UiString::ui(id("ID_Store_Quantity"), vec![(id("Quantity"), quantity)])
            };
            if sale.status == 1 || sale.stock == 0 {
                tree.set_state(stock, id(UNAVAILABLE));
            }
            tree.set_text(stock, &s.render_string(&text));
        }
    }

    /// `gmShopUI::RecvNotice_UpdateSale`: the sale's stock text (`Store_Quantity` of the stock, or
    /// sold out, which makes the sale unavailable).
    pub fn recv_update_sale(
        &mut self,
        tree: &mut UiTree,
        s: &dyn ShopSource,
        item: u64,
        stock: i32,
    ) {
        if !self.shown {
            return;
        }
        let Some(row) = self.sale_element(item) else {
            return;
        };
        let Some(text) = tree.child_recursive(row, id("StockQuantity")) else {
            return;
        };
        let quantity = if stock < 1 {
            tree.set_state(text, id(UNAVAILABLE));
            tree.set_bool(row, id(UNAVAILABLE), true);
            let sold_out = UiString::ui(id("ID_Store_SoldOut"), Vec::new());
            SkillVar::Info(Box::new(sold_out))
        } else {
            SkillVar::Int(i64::from(stock))
        };
        let string = UiString::ui(id("ID_Store_Quantity"), vec![(id("Quantity"), quantity)]);
        tree.set_text(text, &s.render_string(&string));
    }

    /// `gmShopUI::UpdatePosition`: the previous button is ghosted on the first page, the next one
    /// on the last (`size - 7 <= position`, seven sales a page); a page request holds both until
    /// its wait is over (the global loop releases them).
    pub(crate) fn update_position(&mut self, tree: &mut UiTree, position: i32, size: i32) {
        if let Some(prev) = self.child(tree, "ShopPrevButton") {
            tree.set_bool(prev, id(UNAVAILABLE), position == 0);
            if position == 0 {
                tree.set_bool(prev, BUTTON_GHOSTED, true);
            } else if self.next_update == 0.0 {
                tree.set_bool(prev, BUTTON_GHOSTED, false);
            }
        }
        if let Some(next) = self.child(tree, "ShopNextButton") {
            if size - 7 <= position {
                tree.set_bool(next, id(UNAVAILABLE), true);
                tree.set_bool(next, BUTTON_GHOSTED, true);
            } else {
                tree.set_bool(next, id(UNAVAILABLE), false);
                if self.next_update == 0.0 {
                    tree.set_bool(next, BUTTON_GHOSTED, false);
                }
            }
        }
    }

    /// The number of sales the list shows.
    pub fn sale_count(&self) -> usize {
        self.rows.len()
    }
}
