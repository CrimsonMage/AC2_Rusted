use super::{World, WorldEvent, WorldState};
use ac2_net::protocol::{Build, ServerEvent};
use ac2_net::{pack_string, Blob};

const PLAYER: u64 = 0x5000_0000_0000_0001;

fn blob(high: u32, low: u32, data: Vec<u8>) -> Blob {
    Blob {
        id: (u64::from(high) << 32) | u64::from(low),
        queue: 1,
        data,
    }
}

fn character_error() -> Vec<u8> {
    let mut b = 0x0001_0056u32.to_le_bytes().to_vec();
    b.extend(7u32.to_le_bytes());
    b
}

fn create_player() -> Vec<u8> {
    let mut b = 0x0001_0060u32.to_le_bytes().to_vec();
    b.extend(PLAYER.to_le_bytes());
    b.extend(1u32.to_le_bytes());
    b
}

/// `CreateObject` with no visual or weenie fields and a physics instance stamp.
fn create_object(id: u64, instance: u16, visual_stamp: u16) -> Vec<u8> {
    let mut b = 0x0001_0058u32.to_le_bytes().to_vec();
    b.extend(id.to_le_bytes());
    b.extend(0u32.to_le_bytes()); // VisualDesc bitfield
    b.extend(0u32.to_le_bytes()); // PhysicsDesc bitfield
    b.extend(instance.to_le_bytes());
    b.extend(visual_stamp.to_le_bytes());
    b.extend([0u8; 8]); // timestamps
    b.extend(0u32.to_le_bytes()); // WeenieDesc mask
    b
}

/// `SetMode` from `id` at `instance`, visual stamp `stamp`.
fn set_mode(id: u64, instance: u16, stamp: u16, mode: u32) -> Vec<u8> {
    let mut b = 0x0001_00A9u32.to_le_bytes().to_vec();
    b.extend(id.to_le_bytes());
    b.extend(instance.to_le_bytes());
    b.extend(stamp.to_le_bytes());
    b.extend(mode.to_le_bytes());
    b
}

fn opcodes(events: &[WorldEvent]) -> Vec<u32> {
    events
        .iter()
        .map(|e| match e {
            WorldEvent::Dispatched(ServerEvent::CharacterError(_)) => 0x10056,
            WorldEvent::Dispatched(ServerEvent::CreatePlayer { .. }) => 0x10060,
            WorldEvent::Dispatched(ServerEvent::CreateObject(_)) => 0x10058,
            WorldEvent::Dispatched(ServerEvent::Object(_)) => 0x100A9,
            WorldEvent::Dispatched(ServerEvent::CharacterSet(_)) => 0x10055,
            other => panic!("unexpected {other:?}"),
        })
        .collect()
}

#[test]
fn nothing_but_out_of_world_events_arrives_before_world_entry() {
    let mut world = World::new(Build::Admin7148);
    // Not out of world: dropped.
    assert!(world
        .receive(blob(0x0300_0001, 1, character_error()), 0.0)
        .is_empty());
    let mut set = 0x0001_0055u32.to_le_bytes().to_vec();
    set.extend([0u32, 0, 0, 5].iter().flat_map(|v| v.to_le_bytes()));
    pack_string(&mut set, 0, b"local");
    set.extend(0u32.to_le_bytes());
    let events = world.receive(blob(0x2300_0002, 2, set), 0.0);
    assert_eq!(opcodes(&events), [0x10055]);
}

#[test]
fn entering_holds_everything_but_create_player_until_it_arrives() {
    let mut world = World::new(Build::Legions7160);
    world.start_entering_world();
    let object = 0x7000_0000_0000_0009;
    assert!(world
        .receive(blob(0x0300_0001, 1, create_object(object, 1, 0)), 0.0)
        .is_empty());
    let events = world.receive(blob(0x0300_0002, 2, create_player()), 0.0);
    // CreatePlayer, then the held create replays.
    assert_eq!(opcodes(&events), [0x10060, 0x10058]);
    assert_eq!(world.state(), WorldState::InWorld);
    assert!(world.objects.get(object).is_some());
}

#[test]
fn events_for_unknown_objects_wait_and_visual_order_is_kept() {
    let mut world = World::new(Build::Legions7160);
    world.start_entering_world();
    world.receive(blob(0x0300_0001, 1, create_player()), 0.0);
    world.receive(blob(0x0300_0002, 2, create_object(PLAYER, 1, 0)), 0.0);
    let npc = 0x7000_0000_0000_0042;
    // Visual-ordered events for an object not created yet: held for it, stamps 2 then 1.
    let second = blob(0x0200_0000, 10, set_mode(npc, 4, 2, 0x22));
    let first = blob(0x0200_0000, 11, set_mode(npc, 4, 1, 0x11));
    assert!(world.receive(second, 0.0).is_empty());
    assert!(world.receive(first, 0.0).is_empty());
    // Its create (visual stamp 0) releases them in stamp order.
    let events = world.receive(blob(0x0300_0003, 3, create_object(npc, 4, 0)), 0.0);
    assert_eq!(opcodes(&events), [0x10058, 0x100A9, 0x100A9]);
    assert_eq!(world.objects.get(npc).unwrap().mode, Some(0x22));
    // An event for an older instance is dropped; a newer instance waits.
    assert!(world
        .receive(blob(0x0200_0000, 12, set_mode(npc, 3, 3, 0x33)), 0.0)
        .is_empty());
    assert!(world
        .receive(blob(0x0200_0000, 13, set_mode(npc, 5, 1, 0x55)), 0.0)
        .is_empty());
    let events = world.receive(blob(0x0300_0004, 4, create_object(npc, 5, 0)), 0.0);
    assert_eq!(opcodes(&events), [0x10058, 0x100A9]);
    assert_eq!(world.objects.get(npc).unwrap().mode, Some(0x55));
}
