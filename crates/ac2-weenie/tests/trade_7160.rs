//! The secure trade against the Legions server double: the trade window's calls go out as the
//! `WM_Trade` stubs send them (`ScriptRequest`s, the double's `script_requests`), and the
//! double's answers come back as the `CM_Trade` notices the player's script raises.

use std::time::Duration;

use ac2_net::protocol::{PackedReference, ScriptRequest, StringInfo};
use ac2_net::Credential;
use ac2_weenie::{
    CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent, INVENTORY_REFRESH,
};
use legions_server_double::{
    addr, double, Spawn, FELLOW, PLAYER, SERVER, TRADE_CANCELLED, TRADE_COMPLETED,
};

const SWORD: u64 = 0x8000_0002_0000_0021;
const ROBE: u64 = 0x8000_0002_0000_0022;
const SHIELD: u64 = 0x8000_0002_0000_0031;
const HELM: u64 = 0x8000_0002_0000_0032;

fn config() -> OnlineConfig {
    OnlineConfig {
        server: addr(SERVER),
        account: "local".into(),
        credential: Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::First,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    }
}

type Double = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

fn settle(session: &mut OnlineSession, state: &Double) -> Vec<GameplayNotice> {
    let mut notices = Vec::new();
    for _ in 0..8 {
        for event in session.poll() {
            if let SessionEvent::Notice(notice) = event {
                notices.push(notice);
            }
        }
        state.borrow_mut().elapsed += 0.05;
    }
    notices
}

fn partner_name() -> StringInfo {
    StringInfo {
        literal: Some("Birch".into()),
        ..StringInfo::default()
    }
}

fn no_reference() -> ScriptRequest {
    ScriptRequest::AcceptTrade(PackedReference {
        ids: Vec::new(),
        entries: Vec::new(),
    })
}

fn opened() -> Vec<GameplayNotice> {
    vec![
        GameplayNotice::TradeRegistered {
            initiator: PLAYER,
            partner: FELLOW,
            partner_name: partner_name(),
        },
        GameplayNotice::TradeOpened,
    ]
}

#[test]
fn the_secure_trade_runs_through_the_double() {
    if std::env::var_os("AC2_DAT_DIR").is_none() {
        eprintln!("AC2_DAT_DIR is unset: skipped");
        return;
    }
    let (transport, state) = double(Spawn::default());
    {
        let mut s = state.borrow_mut();
        s.script_world
            .contents
            .insert(PLAYER, (Vec::new(), vec![SWORD, ROBE]));
        s.script_world
            .contents
            .insert(FELLOW, (Vec::new(), vec![SHIELD, HELM]));
    }
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    state.borrow_mut().send_init_player();
    settle(&mut session, &state);

    // A deal: the player offers the sword, the partner the shield; the player takes the sword
    // back and offers it again; both accept.
    session.open_trade(FELLOW);
    assert_eq!(settle(&mut session, &state), opened());
    session.offer_item_for_trade(SWORD, 1);
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::TradeOffered {
            source: PLAYER,
            item: SWORD,
            amount: 1
        }]
    );
    state.borrow_mut().partner_offer(SHIELD, 1);
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::TradeOffered {
            source: FELLOW,
            item: SHIELD,
            amount: 1
        }]
    );
    session.revoke_item_from_trade(SWORD);
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::TradeRevoked {
            source: PLAYER,
            item: SWORD
        }]
    );
    session.offer_item_for_trade(SWORD, 1);
    settle(&mut session, &state);
    session.accept_trade();
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::TradeAccepted(PLAYER)]
    );
    state.borrow_mut().partner_accept();
    assert_eq!(
        settle(&mut session, &state),
        [
            GameplayNotice::TradeAccepted(FELLOW),
            // The player's container lists again (the items swapped).
            GameplayNotice::UpdateContainer {
                container: PLAYER,
                code: INVENTORY_REFRESH,
                item: 0,
                slot: 0
            },
            GameplayNotice::TradeClosed {
                source: PLAYER,
                kind: TRADE_COMPLETED
            }
        ]
    );
    {
        let s = state.borrow();
        // The sword went to the partner's container, the shield to the player's.
        assert_eq!(s.script_world.contents[&PLAYER].1, [ROBE, SHIELD]);
        assert_eq!(s.script_world.contents[&FELLOW].1, [HELM, SWORD]);
    }
    let items = session.inventory().contents(PLAYER).unwrap().items.clone();
    assert_eq!(items, [ROBE, SHIELD]);

    // Declined: the player's offer stands until the partner's side declines.
    session.open_trade(FELLOW);
    settle(&mut session, &state);
    session.offer_item_for_trade(ROBE, 2);
    settle(&mut session, &state);
    session.decline_trade();
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::TradeDeclined(PLAYER)]
    );

    // Closed by the player, with no exchange.
    session.open_trade(FELLOW);
    settle(&mut session, &state);
    session.close_trade();
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::TradeClosed {
            source: PLAYER,
            kind: TRADE_CANCELLED
        }]
    );

    // A change after an accept: the acceptances lapse before the change is raised.
    session.open_trade(FELLOW);
    settle(&mut session, &state);
    session.offer_item_for_trade(ROBE, 1);
    settle(&mut session, &state);
    session.accept_trade();
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::TradeAccepted(PLAYER)]
    );
    session.revoke_item_from_trade(ROBE);
    assert_eq!(
        settle(&mut session, &state),
        [
            GameplayNotice::TradeAcceptanceCleared,
            GameplayNotice::TradeRevoked {
                source: PLAYER,
                item: ROBE
            }
        ]
    );
    session.decline_trade();
    assert_eq!(
        settle(&mut session, &state),
        [GameplayNotice::TradeDeclined(PLAYER)]
    );

    // The calls the trade window made, in order, as the stubs send them.
    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    assert_eq!(
        s.script_requests,
        vec![
            ScriptRequest::OpenTrade(FELLOW),
            ScriptRequest::OfferItem {
                item: SWORD,
                amount: 1
            },
            ScriptRequest::RevokeItem(SWORD),
            ScriptRequest::OfferItem {
                item: SWORD,
                amount: 1
            },
            no_reference(),
            ScriptRequest::OpenTrade(FELLOW),
            ScriptRequest::OfferItem {
                item: ROBE,
                amount: 2
            },
            ScriptRequest::DeclineTrade,
            ScriptRequest::OpenTrade(FELLOW),
            ScriptRequest::CloseTrade,
            ScriptRequest::OpenTrade(FELLOW),
            ScriptRequest::OfferItem {
                item: ROBE,
                amount: 1
            },
            no_reference(),
            ScriptRequest::RevokeItem(ROBE),
            ScriptRequest::DeclineTrade,
        ]
    );
}
