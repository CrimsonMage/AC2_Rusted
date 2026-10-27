//! The external container against the Legions server double: a chest on the landblock, not in the
//! player's containers, opens as the side pack does (`ContainerStateUpdate`, its two items listed
//! in order), gives one item to the player's pack when looted from, and closes.

use std::time::Duration;

use ac2_net::protocol::{InvMoveDesc, ScriptRequest};
use ac2_net::Credential;
use ac2_weenie::{
    CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent, INVENTORY_REFRESH,
};
use legions_server_double::{
    addr, double, Spawn, CHEST, CHEST_GEM, CHEST_SCROLL, PLAYER, POTIONS, SERVER, SWORD,
};

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

fn settle(session: &mut OnlineSession, state: &Double) -> Vec<SessionEvent> {
    let mut events = Vec::new();
    for _ in 0..8 {
        events.extend(session.poll());
        state.borrow_mut().elapsed += 0.05;
    }
    events
}

fn notices(events: &[SessionEvent]) -> Vec<GameplayNotice> {
    events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(n) => Some(n.clone()),
            _ => None,
        })
        .collect()
}

fn items(session: &OnlineSession, container: u64) -> Vec<u64> {
    session
        .inventory()
        .contents(container)
        .unwrap()
        .items
        .to_vec()
}

fn refresh(container: u64) -> GameplayNotice {
    GameplayNotice::UpdateContainer {
        container,
        code: INVENTORY_REFRESH,
        item: 0,
        slot: 0,
    }
}

#[test]
fn the_chest_opens_lists_its_items_and_gives_one_to_the_player() {
    let (transport, state) = double(Spawn::default());
    state
        .borrow_mut()
        .script_world
        .contents
        .insert(PLAYER, (vec![], vec![POTIONS, SWORD]));
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    {
        let mut s = state.borrow_mut();
        s.send_init_player();
        s.spawn_chest();
    }
    settle(&mut session, &state);
    // The chest is in the world, not among the player's containers.
    assert_eq!(items(&session, PLAYER), [POTIONS, SWORD]);
    assert!(session.inventory().contents(CHEST).is_none());

    // Opening it: the open notice, then its contents (the gem, then the scroll).
    session.set_container_open(CHEST, true);
    let events = settle(&mut session, &state);
    assert_eq!(
        notices(&events),
        [
            GameplayNotice::ContainerStateUpdate {
                container: CHEST,
                open: true
            },
            refresh(CHEST)
        ]
    );
    assert_eq!(session.inventory().open_containers(), [CHEST]);
    assert_eq!(items(&session, CHEST), [CHEST_GEM, CHEST_SCROLL]);
    assert!(session
        .inventory()
        .contents(CHEST)
        .unwrap()
        .packs
        .is_empty());

    // The gem into the player's pack (slot -1: the end): the chest shrinks, the pack grows.
    session.move_item(CHEST_GEM, CHEST, 0, PLAYER, -1, 1);
    let events = settle(&mut session, &state);
    assert_eq!(notices(&events), [refresh(CHEST), refresh(PLAYER)]);
    assert_eq!(items(&session, CHEST), [CHEST_SCROLL]);
    assert_eq!(items(&session, PLAYER), [POTIONS, SWORD, CHEST_GEM]);

    // Closing it: the close notice, and no container open.
    session.set_container_open(CHEST, false);
    let events = settle(&mut session, &state);
    assert_eq!(
        notices(&events),
        [GameplayNotice::ContainerStateUpdate {
            container: CHEST,
            open: false
        }]
    );
    assert!(session.inventory().open_containers().is_empty());
    assert!(session.inventory().contents(CHEST).is_none());

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    assert_eq!(
        s.script_requests,
        vec![
            ScriptRequest::OpenContainer(CHEST),
            ScriptRequest::MoveItem(InvMoveDesc::new(CHEST_GEM, CHEST, 0, PLAYER, -1, 1)),
            ScriptRequest::CloseContainer(CHEST),
        ]
    );
}
