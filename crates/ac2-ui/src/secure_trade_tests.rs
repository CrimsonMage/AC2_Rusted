//! `gmSecureTradeUI` on the real gameplay and toolbar layouts (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::drag::MSG_STOP_DRAG_ITEM;
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};
use crate::ui_string::{UiString, UiStrings};
use crate::{
    build_gameplay_hud, AttributeValue, BackpackUi, GameplayHud, InventorySource, ItemSlot,
    SecureTradeRequest, SecureTradeSplit, UiTree, DIALOG_CONFIRMATION,
};

const PLAYER: u64 = 0x2000_0000_0000_0001;
const PARTNER: u64 = 0x2000_0000_0000_0002;
const POTIONS: u64 = 0x8000_0000_0000_0010;
const SWORD: u64 = 0x8000_0000_0000_0011;
const GEM: u64 = 0x8000_0000_0000_0012;

/// The player's pack (five potions, a sword) and the partner's gem.
struct Pack;

impl InventorySource for Pack {
    fn item(&self, item: u64) -> Option<ItemSlot> {
        let (icon, name, quantity, stackable) = match item {
            POTIONS => (0x0600_1001, "Healing Potions", 5, true),
            SWORD => (0x0600_1002, "Practice Sword", 1, false),
            GEM => (0x0600_1004, "Gem", 1, false),
            _ => return None,
        };
        Some(ItemSlot {
            icon,
            name: name.into(),
            quantity,
            stackable,
            max_stack: if stackable { 10 } else { 0 },
            ..ItemSlot::default()
        })
    }

    fn contents_info(&self, container: u64, _: u32) -> (Vec<u64>, u32, u32) {
        match container {
            PLAYER => (vec![POTIONS, SWORD], 0, 24),
            _ => (Vec::new(), 0, 0),
        }
    }

    fn max_container_size(&self, _: u64) -> u32 {
        24
    }

    fn selection(&self) -> u64 {
        0
    }

    fn player(&self) -> u64 {
        PLAYER
    }

    fn item_name(&self, item: u64, quantity: u32) -> Option<String> {
        (item == POTIONS).then(|| format!("{quantity} Healing Potions"))
    }
}

/// `ID(name=value)` for each rendered string.
struct Strings;

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        let vars: Vec<String> = string
            .vars
            .iter()
            .map(|(name, value)| match value {
                SkillVar::UInt(n) => format!("{name}={n}"),
                SkillVar::Text(t) => format!("{name}={t}"),
                _ => format!("{name}=?"),
            })
            .collect();
        format!("{}({})", string.string_id, vars.join(","))
    }
}

fn button(name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

/// A side's element (`Self` or `Other`) under the panel, then the named child of it.
fn under_side(tree: &UiTree, field: usize, side: &str, name: &str) -> usize {
    let side = tree
        .child_recursive(field, tokenize(side))
        .expect("the side's element");
    tree.child_recursive(side, tokenize(name))
        .unwrap_or_else(|| panic!("{name} under {side:?}"))
}

/// The gameplay HUD with the player's backpack made over it.
fn setup() -> Option<(GameplayHud, BackpackUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let layout = hud.toolbar_layout.clone().unwrap();
    let backpack = BackpackUi::create(&mut hud.tree, &layout, &Pack, PLAYER);
    hud.secure_trade
        .as_mut()
        .expect("the trade panel")
        .set_player(PLAYER);
    Some((hud, backpack))
}

/// The backpack's icon of `item`: the copy a drag from the pack starts from.
fn pack_copy(backpack: &BackpackUi, item: u64) -> usize {
    backpack.packs[0]
        .items
        .iter()
        .find(|slot| slot.item == item)
        .expect("the item in the pack")
        .icon
}

/// A drop of `copy` on a trade list (`stop_drag_item` with `data` 1, the drop).
fn list_drop(tree: &UiTree, list: usize, copy: usize) -> UiMessage {
    UiMessage {
        element_id: tree.get(list).unwrap().id(),
        element: Some(list),
        message: MSG_STOP_DRAG_ITEM,
        data: 1,
        item: Some(copy),
    }
}

/// A trade item dragged out of its list (`TradeItemTemplate`, `data` 0 for the source).
fn item_drag_out(tree: &UiTree, copy: usize) -> UiMessage {
    UiMessage {
        element_id: tokenize("TradeItemTemplate"),
        element: tree.get(copy).and_then(|e| e.parent_index()),
        message: MSG_STOP_DRAG_ITEM,
        data: 0,
        item: Some(copy),
    }
}

/// The icon of the first item in a trade list.
fn first_icon(tree: &UiTree, list: usize) -> usize {
    let background = tree.list_items(list)[0];
    tree.child_recursive(background, tokenize("ItemIcon"))
        .expect("the item's icon")
}

/// The shown confirmation dialog's root.
fn confirmation_root(tree: &UiTree) -> usize {
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == tokenize("ConfirmationField"))
        .expect("the confirmation dialog")
}

#[test]
fn an_opened_trade_shows_the_panel_with_both_lists_empty() {
    let Some((mut hud, _)) = setup() else {
        return;
    };
    let GameplayHud {
        tree,
        secure_trade,
        dialogs,
        ..
    } = &mut hud;
    let panel = secure_trade.as_mut().unwrap();
    // The player asked for the trade: the panel shows, waiting for the partner.
    panel.recv_trade_be_registered(tree, dialogs, &Strings, PLAYER, PARTNER, "Bob");
    assert!(panel.is_shown());
    assert_eq!(panel.partner(), PARTNER);
    assert!(
        dialogs.shown().is_none(),
        "no confirmation for our own request"
    );
    let field = panel.field().expect("SecureTradeField");
    assert!(tree.get(field).unwrap().visible());
    assert_eq!(tree.get(field).unwrap().state(), tokenize("acceptpending"));

    // The partner opens it: the trade's state, and the partner's name on the label.
    panel.recv_trade_be_opened(tree);
    assert_eq!(tree.get(field).unwrap().state(), tokenize("trade"));
    let label = tree
        .child_recursive(field, tokenize("PartnerLabel"))
        .expect("PartnerLabel");
    assert_eq!(tree.get(label).unwrap().text().as_deref(), Some("Bob"));
    assert!(panel.own_items().is_empty());
    assert!(panel.partner_items().is_empty());
    assert!(panel.take_requests().is_empty());
}

#[test]
fn a_pack_item_dropped_on_the_list_asks_offer_and_a_stack_asks_the_stack_dialog_first() {
    let Some((mut hud, backpack)) = setup() else {
        return;
    };
    let GameplayHud {
        tree,
        secure_trade,
        stack,
        ..
    } = &mut hud;
    let panel = secure_trade.as_mut().unwrap();
    panel.recv_trade_be_opened(tree);
    let field = panel.field().unwrap();
    let own = under_side(tree, field, "Self", "TradeListBoxSelf");

    // The sword (one, not a stack) on the player's list asks Offer at once.
    let drop = list_drop(tree, own, pack_copy(&backpack, SWORD));
    panel.message(tree, &Pack, &drop);
    assert_eq!(
        panel.take_requests(),
        [SecureTradeRequest::Offer {
            item: SWORD,
            quantity: 1
        }]
    );
    assert!(panel.take_splits().is_empty());
    // The notice that it is offered fills the player's list.
    panel.recv_trade_be_offered(tree, &Pack, PLAYER, SWORD, 1);
    assert_eq!(panel.own_items(), [SWORD]);
    assert_eq!(tree.list_items(own).len(), 1);

    // The potions (a stack of five) ask the stack dialog first, five the default and the most.
    let drop = list_drop(tree, own, pack_copy(&backpack, POTIONS));
    panel.message(tree, &Pack, &drop);
    assert!(panel.take_requests().is_empty());
    let splits = panel.take_splits();
    assert_eq!(splits.len(), 1);
    let SecureTradeSplit {
        context,
        item,
        amount,
    } = splits[0];
    assert_eq!((item, amount), (POTIONS, 5));
    assert_ne!(context, 0);
    let stack = stack.as_mut().expect("the stack dialog");
    stack.init(tree, context, "Healing Potions", amount, amount, PLAYER, 0);
    assert_eq!(stack.shown(), Some(context));

    // OK answers with the entry's five: the offer of five potions.
    let done = stack
        .message(tree, &button("StackOkButton"), &Strings)
        .expect("the stack's answer");
    assert_eq!((done.context, done.amount), (context, 5));
    panel.recv_stack_ui_done(done.context, done.amount);
    assert_eq!(
        panel.take_requests(),
        [SecureTradeRequest::Offer {
            item: POTIONS,
            quantity: 5
        }]
    );
    // An answer to another stack offers nothing.
    panel.recv_stack_ui_done(context + 1, 3);
    assert!(panel.take_requests().is_empty());

    // Three of the five offered: the slot names the potions at the offered three
    // (`ItemDisplay::SetQuantity`).
    panel.recv_trade_be_offered(tree, &Pack, PLAYER, POTIONS, 3);
    let slot = tree.list_items(own)[1];
    let icon = tree.child(slot, tokenize("ItemIcon")).expect("the icon");
    assert_eq!(
        tree.tooltip_text(icon).as_deref(),
        Some("3 Healing Potions")
    );
}

#[test]
fn offered_notices_fill_both_lists_and_dragging_an_item_out_revokes_it() {
    let Some((mut hud, _)) = setup() else {
        return;
    };
    let GameplayHud {
        tree, secure_trade, ..
    } = &mut hud;
    let panel = secure_trade.as_mut().unwrap();
    panel.recv_trade_be_opened(tree);
    let field = panel.field().unwrap();
    let own = under_side(tree, field, "Self", "TradeListBoxSelf");
    let other = under_side(tree, field, "Other", "TradeListBoxOther");

    panel.recv_trade_be_offered(tree, &Pack, PLAYER, SWORD, 1);
    panel.recv_trade_be_offered(tree, &Pack, PARTNER, GEM, 1);
    assert_eq!(panel.own_items(), [SWORD]);
    assert_eq!(panel.partner_items(), [GEM]);
    assert_eq!(tree.list_items(own).len(), 1);
    assert_eq!(tree.list_items(other).len(), 1);
    // The player's item is dragable; the partner's is not, and neither is a shortcut.
    let own_icon = first_icon(tree, own);
    let other_icon = first_icon(tree, other);
    assert!(tree.bool_attribute(own_icon, tokenize("dragable")));
    assert!(!tree.bool_attribute(other_icon, tokenize("dragable")));
    assert!(matches!(
        tree.get(own_icon)
            .unwrap()
            .attribute(tokenize("Token_ShortcutType")),
        Some(AttributeValue::Id(0))
    ));

    // An item already listed is not listed twice.
    panel.recv_trade_be_offered(tree, &Pack, PLAYER, SWORD, 1);
    assert_eq!(panel.own_items(), [SWORD]);

    // Dragging the player's item out of its list revokes it; the notice empties the list.
    let drag = item_drag_out(tree, own_icon);
    panel.message(tree, &Pack, &drag);
    assert_eq!(
        panel.take_requests(),
        [SecureTradeRequest::Revoke { item: SWORD }]
    );
    panel.recv_trade_be_revoked(tree, PLAYER, SWORD);
    assert!(panel.own_items().is_empty());
    assert!(tree.list_items(own).is_empty());
    assert_eq!(panel.partner_items(), [GEM]);
}

#[test]
fn accept_asks_at_once_and_the_partners_accept_shows_on_its_button() {
    let Some((mut hud, _)) = setup() else {
        return;
    };
    let GameplayHud {
        tree, secure_trade, ..
    } = &mut hud;
    let panel = secure_trade.as_mut().unwrap();
    panel.recv_trade_be_opened(tree);
    let field = panel.field().unwrap();
    let accept = under_side(tree, field, "Self", "AcceptTradeButton");
    let partner_inner = under_side(tree, field, "Other", "TradeButtonInner");
    let own_inner = under_side(tree, field, "Self", "TradeButtonInner");

    // Retail's `AcceptTrade` is sent at once (no confirmation), and the button is then ghosted.
    let press = UiMessage {
        element: Some(accept),
        ..button("AcceptTradeButton")
    };
    panel.message(tree, &Pack, &press);
    assert_eq!(panel.take_requests(), [SecureTradeRequest::Accept]);
    assert!(tree.bool_attribute(accept, BUTTON_GHOSTED));

    // The partner accepts: its side reads `accepted`; the acceptance cleared resets both sides.
    panel.recv_trade_be_accepted(tree, PARTNER);
    assert_eq!(
        tree.get(partner_inner).unwrap().state(),
        tokenize("accepted")
    );
    panel.recv_trade_be_acceptance_cleared(tree);
    assert_eq!(
        tree.get(partner_inner).unwrap().state(),
        tokenize("notaccepted")
    );
    assert_eq!(
        tree.get(own_inner).unwrap().state(),
        tokenize("notaccepted")
    );

    // A decline by the partner shows on its side and leaves the panel up.
    panel.recv_trade_be_declined(tree, PARTNER);
    assert!(panel.is_shown());
    assert_eq!(
        tree.get(partner_inner).unwrap().state(),
        tokenize("notaccepted")
    );
}

#[test]
fn a_partners_request_asks_the_confirmation_then_opens_or_declines() {
    let Some((mut hud, _)) = setup() else {
        return;
    };
    let GameplayHud {
        tree,
        secure_trade,
        dialogs,
        ..
    } = &mut hud;
    let panel = secure_trade.as_mut().unwrap();
    panel.recv_trade_be_registered(tree, dialogs, &Strings, PARTNER, PLAYER, "Bob");
    assert!(!panel.is_shown(), "the request does not open the trade");
    let (_, kind) = dialogs.shown().expect("the confirmation");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    let root = confirmation_root(tree);
    let text = tree
        .child_recursive(root, tokenize("QuestionText"))
        .expect("QuestionText");
    assert_eq!(
        tree.get(text).unwrap().text().as_deref(),
        Some(
            format!(
                "{}({}=Bob)",
                tokenize("ID_SecureTrade_AcceptTradeText"),
                tokenize("value")
            )
            .as_str()
        )
    );

    // Yes opens the negotiations with the partner.
    let yes = UiMessage {
        element: tree.child_recursive(root, tokenize("YesButton")),
        ..button("YesButton")
    };
    dialogs.message(tree, &yes);
    panel.dialog_outcomes(tree, &dialogs.take_outcomes());
    assert_eq!(
        panel.take_requests(),
        [SecureTradeRequest::Open { partner: PARTNER }]
    );
    assert!(dialogs.shown().is_none());

    // Retail keeps `_acceptTradeContext` until the trade closes: while it is open, a new
    // request asks nothing.
    panel.recv_trade_be_opened(tree);
    panel.recv_trade_be_registered(tree, dialogs, &Strings, PARTNER, PLAYER, "Bob");
    assert!(dialogs.shown().is_none());
    // The partner closes the trade (the context goes with it); the next request asks again, and
    // No closes the trade's negotiations.
    panel.recv_trade_be_closed(tree, dialogs, &Strings, PARTNER, 0.0);
    panel.recv_trade_be_registered(tree, dialogs, &Strings, PARTNER, PLAYER, "Bob");
    let root = confirmation_root(tree);
    let no = UiMessage {
        element: tree.child_recursive(root, tokenize("NoButton")),
        ..button("NoButton")
    };
    dialogs.message(tree, &no);
    panel.dialog_outcomes(tree, &dialogs.take_outcomes());
    assert_eq!(panel.take_requests(), [SecureTradeRequest::Close]);
}

#[test]
fn a_closed_trade_empties_the_lists_and_closes_after_its_delay() {
    let Some((mut hud, _)) = setup() else {
        return;
    };
    let GameplayHud {
        tree,
        secure_trade,
        dialogs,
        ..
    } = &mut hud;
    let panel = secure_trade.as_mut().unwrap();
    panel.recv_trade_be_registered(tree, dialogs, &Strings, PLAYER, PARTNER, "Bob");
    panel.recv_trade_be_opened(tree);
    panel.recv_trade_be_offered(tree, &Pack, PLAYER, SWORD, 1);
    panel.recv_trade_be_offered(tree, &Pack, PARTNER, GEM, 1);
    let field = panel.field().unwrap();

    // The partner closes it: the lists go, the ended text shows, the panel stays up for now.
    panel.recv_trade_be_closed(tree, dialogs, &Strings, PARTNER, 10.0);
    assert!(panel.own_items().is_empty());
    assert!(panel.partner_items().is_empty());
    assert!(panel.trade_ended());
    assert_eq!(tree.get(field).unwrap().state(), tokenize("partnerclosed"));
    let ended = tree
        .child_recursive(field, tokenize("EndedText"))
        .expect("EndedText");
    assert_eq!(
        tree.get(ended).unwrap().text().as_deref(),
        Some(
            format!(
                "{}({}=Bob)",
                tokenize("ID_SecureTrade_Ended"),
                tokenize("name")
            )
            .as_str()
        )
    );

    // The field's `delay` sets when it closes (the global loop, after that time).
    let delay = match tree.get(field).unwrap().attribute(tokenize("delay")) {
        Some(AttributeValue::Float(seconds)) => f64::from(*seconds),
        _ => 0.0,
    };
    panel.tick(tree, 10.0 + delay - 0.5);
    assert!(panel.is_shown(), "open until its delay passes");
    panel.tick(tree, 10.0 + delay + 0.5);
    assert!(!panel.is_shown());
    assert!(panel.field().is_none());
    assert!(!panel.trade_ended());
}

#[test]
fn a_close_button_asks_the_close_and_our_own_close_hides_at_once() {
    let Some((mut hud, _)) = setup() else {
        return;
    };
    let GameplayHud {
        tree,
        secure_trade,
        dialogs,
        ..
    } = &mut hud;
    let panel = secure_trade.as_mut().unwrap();
    panel.recv_trade_be_opened(tree);
    panel.message(tree, &Pack, &button("SecureTradeCancelButton"));
    assert_eq!(panel.take_requests(), [SecureTradeRequest::Close]);
    assert!(panel.is_shown(), "the request waits for the server's close");

    panel.recv_trade_be_closed(tree, dialogs, &Strings, PLAYER, 5.0);
    assert!(!panel.is_shown());
    assert!(panel.field().is_none());
}
