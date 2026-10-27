//! The inventory against the Legions server double: contents and equipment arrive as
//! `WM_Inventory` 0x53 / 0x5F, moves and equips go out as the `CContainer` / `CInventorySystem`
//! scripts build their `InvMoveDesc` / `InvEquipDesc` (byte for byte), and the double's answers
//! update the stand-in's lists.

use std::time::Duration;

use ac2_net::protocol::{InvEquipDesc, InvMoveDesc, ScriptRequest};
use ac2_net::Credential;
use ac2_weenie::{
    CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent, INVENTORY_REFRESH,
};
use legions_server_double::{
    addr, create_object_bytes, double, words, ObjectSpawn, Spawn, CELL, PLAYER, SERVER,
};

const PACK: u64 = 0x8000_0002_0000_0100;
const A: u64 = 0x8000_0002_0000_0001;
const B: u64 = 0x8000_0002_0000_0002;
const C: u64 = 0x8000_0002_0000_0003;
const D: u64 = 0x8000_0002_0000_0004;
const E: u64 = 0x8000_0002_0000_0005;
const LANDBLOCK: u64 = 0x0299_0000;

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

#[test]
fn contents_moves_and_equipment_round_trip_through_the_double() {
    let (transport, state) = double(Spawn::default());
    {
        let mut s = state.borrow_mut();
        s.script_world
            .contents
            .insert(PLAYER, (vec![PACK], vec![A, B, C]));
        s.script_world.contents.insert(PACK, (vec![], vec![D]));
        s.script_world.equipped = vec![(0x10, E)];
    }
    let mut session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    session.wait_for_world(Duration::from_secs(10)).unwrap();
    session.poll();
    {
        let mut s = state.borrow_mut();
        // The login message carries the player's contents and equipment (`CPlayer` 0x65).
        s.send_init_player();
        // The side pack exists as an object before its contents come to it.
        s.send_event(&create_object_bytes(&ObjectSpawn {
            id: PACK,
            instance: 1,
            visual: 0x1F00_0023,
            cell: CELL,
            origin: [60.0, 240.0, -44.5],
            rotation: [1.0, 0.0, 0.0, 0.0],
            stamps: [1, 1, 1, 1],
            name: "Side Pack",
        }));
        s.send_contents(PACK);
    }
    let events = settle(&mut session, &state);
    let refresh = |container| GameplayNotice::UpdateContainer {
        container,
        code: INVENTORY_REFRESH,
        item: 0,
        slot: 0,
    };
    let raised = notices(&events);
    assert!(
        raised.contains(&refresh(PLAYER)) && raised.contains(&refresh(PACK)),
        "{raised:?}"
    );
    assert!(raised.contains(&GameplayNotice::UpdateInventory {
        code: INVENTORY_REFRESH,
        item: 0,
        location: 0
    }));
    assert_eq!(items(&session, PLAYER), [A, B, C]);
    assert_eq!(
        session.inventory().contents(PLAYER).unwrap().packs[..],
        [PACK]
    );
    assert_eq!(session.inventory().equipped(), [(0x10, E)]);

    // B into the side pack's first slot, A to the second slot of the player's own pack (a
    // reorganization), C onto its own slot (nothing), then C dropped on the landblock.
    session.move_item(B, PLAYER, 1, PACK, 0, 1);
    settle(&mut session, &state);
    assert_eq!(
        (items(&session, PLAYER), items(&session, PACK)),
        (vec![A, C], vec![B, D])
    );
    session.move_item(A, PLAYER, 0, PLAYER, 1, 1);
    settle(&mut session, &state);
    assert_eq!(items(&session, PLAYER), [C, A]);
    session.move_item(C, PLAYER, 0, PLAYER, 0, 1);
    session.move_item(C, PLAYER, 0, LANDBLOCK, -1, 1);
    settle(&mut session, &state);
    assert_eq!(items(&session, PLAYER), [A]);

    // Equip A at 0x20, take E off into slot 0; an unknown item is refused before sending.
    session.equip_item(A, 0x20).unwrap();
    settle(&mut session, &state);
    session.unequip_item(E, 0).unwrap();
    settle(&mut session, &state);
    assert_eq!(session.inventory().equipped(), [(0x20, A)]);
    assert_eq!(items(&session, PLAYER), [E]);
    assert!(session.equip_item(0x99, 0x20).is_err());
    assert!(session.unequip_item(0x99, 0).is_err());

    // Opening the side pack lists it again and marks it open; closing drops it.
    session.set_container_open(PACK, true);
    let events = settle(&mut session, &state);
    assert_eq!(
        notices(&events),
        [
            GameplayNotice::ContainerStateUpdate {
                container: PACK,
                open: true
            },
            refresh(PACK)
        ]
    );
    assert_eq!(session.inventory().open_containers(), [PACK]);
    // Everything in it into the player's pack.
    session.loot_all(PACK);
    settle(&mut session, &state);
    assert_eq!(items(&session, PLAYER), [E, B, D]);
    assert!(items(&session, PACK).is_empty());
    session.set_container_open(PACK, false);
    let events = settle(&mut session, &state);
    assert!(
        notices(&events).contains(&GameplayNotice::ContainerStateUpdate {
            container: PACK,
            open: false
        })
    );
    assert!(session.inventory().open_containers().is_empty());
    assert!(session.inventory().contents(PACK).is_none());
    // An item's quantity changing raises its item update (`CItem` 0x3).
    state
        .borrow_mut()
        .send_quality(&ac2_net::protocol::QualityUpdate {
            object: Some(ac2_net::protocol::InstanceIdWithStamp {
                id: PACK,
                instance: 1,
                other: 0,
            }),
            quality: 0x106,
            value: ac2_net::protocol::QualityValue::Int(3),
        });
    let events = settle(&mut session, &state);
    assert!(notices(&events).contains(&GameplayNotice::ItemUpdate(PACK)));

    let s = state.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    assert_eq!(
        s.script_requests,
        vec![
            ScriptRequest::MoveItem(InvMoveDesc::new(B, PLAYER, 1, PACK, 0, 1)),
            ScriptRequest::ReorganizeContents(InvMoveDesc::new(A, PLAYER, 0, PLAYER, 1, 1)),
            ScriptRequest::MoveItem(InvMoveDesc::new(C, PLAYER, 0, LANDBLOCK, -1, 1)),
            ScriptRequest::EquipItem(InvEquipDesc::equip(A, PLAYER, 0x20, PLAYER, 0)),
            ScriptRequest::UnequipItem(InvEquipDesc::unequip(E, PLAYER, 0x10, 0)),
            ScriptRequest::OpenContainer(PACK),
            ScriptRequest::LootAll(ac2_net::protocol::InvTakeAllDesc::new(PACK, PLAYER)),
            ScriptRequest::CloseContainer(PACK),
        ]
    );
    // Byte for byte: the first move's `InvMoveDesc` (one loaded package instance, 51 slots).
    let blob = s
        .blobs
        .iter()
        .find(|(_, q, d)| *q == 3 && d[4..8] == 0x820E_0036u32.to_le_bytes())
        .map(|(_, _, d)| d.clone())
        .unwrap();
    let lo = |v: u64| v as u32;
    let hi = |v: u64| (v >> 32) as u32;
    let mut expected = Vec::new();
    words(
        &mut expected,
        &[0x0001_0053, 0x820E_0036, 4 * 7 + 51 * 4 + 52],
    );
    words(&mut expected, &[5, 1, 1, 0x4400_0100, 0, 0x01F8_0000, 51]);
    #[rustfmt::skip]
    let slots = [
        1, 1, lo(PACK), hi(PACK), 0, 0, 0, 0, lo(PACK), hi(PACK),
        1, 0, 0, 1, 0, 0, lo(PLAYER), hi(PLAYER), 0, 0,
        0, 0, 0, 1, 0, 1, 1, 0, lo(B), hi(B),
        u32::MAX, 0, lo(PACK), hi(PACK), 0, 1, lo(PLAYER), hi(PLAYER), 0, 1,
        0, 0, 0, 0, lo(B), hi(B), 0, 1, 0, 1,
        0,
    ];
    words(&mut expected, &slots);
    #[rustfmt::skip]
    expected.extend([
        2, 2, 0x82, 0xCD, 2, 2, 2, 2, 0x82, 0xCD,
        2, 2, 2, 2, 2, 2, 0x82, 0xCD, 0x82, 0xCD,
        2, 2, 2, 2, 2, 2, 2, 2, 0x82, 0xCD,
        1, 2, 0x82, 0xCD, 2, 2, 0x82, 0xCD, 2, 2,
        2, 2, 2, 2, 0x82, 0xCD, 2, 2, 2, 2,
        2, 0,
    ]);
    assert_eq!(blob, expected);
}
