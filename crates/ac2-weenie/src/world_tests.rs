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

/// `CEventQueue::FlushQueue`: the blobs held while entering the world replay in arrival order
/// once `FinishEnteringWorld` sets the state in-world, after the player is created.
#[test]
fn entering_queue_replays_in_arrival_order_after_create_player() {
    let mut world = World::new(Build::Legions7160);
    world.start_entering_world();
    let (first, second) = (0x7000_0000_0000_0001, 0x7000_0000_0000_0002);
    assert!(world
        .receive(blob(0x0300_0001, 1, create_object(first, 1, 0)), 0.0)
        .is_empty());
    assert!(world
        .receive(blob(0x0300_0002, 2, create_object(second, 1, 0)), 0.0)
        .is_empty());
    let events = world.receive(blob(0x0300_0003, 3, create_player()), 0.0);
    let created: Vec<u64> = events
        .iter()
        .filter_map(|event| match event {
            WorldEvent::Dispatched(ServerEvent::CreateObject(object)) => Some(object.id),
            _ => None,
        })
        .collect();
    assert_eq!(created, [first, second]);
    assert_eq!(world.state(), WorldState::InWorld);
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

#[test]
fn the_player_description_replaces_qualities_and_updates_change_them() {
    let mut world = World::new(Build::Legions7160);
    world.start_entering_world();
    world.receive(blob(0x0300_0001, 1, create_player()), 0.0);
    world.receive(blob(0x0300_0002, 2, create_object(PLAYER, 1, 0)), 0.0);
    // `CBaseQualities::UnPack`: header (int table), DID, one int.
    let mut desc = 0x0001_0068u32.to_le_bytes().to_vec();
    for word in [2u32, 0x8100_110F, 1, 0x19, 7] {
        desc.extend(word.to_le_bytes());
    }
    world.receive(blob(0x0300_0003, 3, desc), 0.0);
    let q = &world.objects.get(PLAYER).unwrap().qualities;
    assert_eq!((q.did, q.ints.get(&0x19)), (0x8100_110F, Some(&7)));
    // Private int update: the player.
    let mut update = 0x0001_0069u32.to_le_bytes().to_vec();
    for word in [0x19u32, 8] {
        update.extend(word.to_le_bytes());
    }
    world.receive(blob(0x0300_0004, 4, update), 0.0);
    assert_eq!(
        world.objects.get(PLAYER).unwrap().qualities.ints.get(&0x19),
        Some(&8)
    );
    // A visual quantity update for an object not yet created waits for it, then lands in
    // its weenie description.
    let npc: u64 = 0x7000_0000_0000_0042;
    let mut visual = 0x0001_006Au32.to_le_bytes().to_vec();
    visual.extend(npc.to_le_bytes());
    visual.extend([4, 0, 0, 0]);
    for word in [0x106u32, 3] {
        visual.extend(word.to_le_bytes());
    }
    assert!(world.receive(blob(0x0300_0005, 5, visual), 0.0).is_empty());
    world.receive(blob(0x0300_0006, 6, create_object(npc, 4, 0)), 0.0);
    let w = world
        .objects
        .get(npc)
        .unwrap()
        .qualities
        .weenie
        .clone()
        .unwrap();
    assert_eq!((w.quantity, w.data_mask & 0x800), (Some(3), 0x800));
}

#[test]
fn held_events_give_up_after_25_seconds_and_ask_for_their_object() {
    let mut world = World::new(Build::Legions7160);
    world.start_entering_world();
    world.receive(blob(0x0300_0001, 1, create_player()), 0.0);
    world.receive(blob(0x0300_0002, 2, create_object(PLAYER, 1, 0)), 0.0);
    let npc = 0x7000_0000_0000_0042;
    assert!(world
        .receive(blob(0x0200_0000, 10, set_mode(npc, 4, 1, 0x11)), 100.0)
        .is_empty());
    assert!(world.use_time(125.0).is_empty());
    assert_eq!(world.use_time(125.5), [WorldEvent::ForceCreate(npc)]);
    // Dropped: the create no longer replays the event.
    let events = world.receive(blob(0x0300_0003, 3, create_object(npc, 4, 0)), 126.0);
    assert_eq!(opcodes(&events), [0x10058]);
}

/// A per-object event with one word or stamp argument (`DestroyObject` has none).
fn object_event(opcode: u32, id: u64, instance: u16, args: &[u8]) -> Vec<u8> {
    let mut b = opcode.to_le_bytes().to_vec();
    b.extend(id.to_le_bytes());
    b.extend(instance.to_le_bytes());
    b.extend(0u16.to_le_bytes());
    b.extend(args);
    while !b.len().is_multiple_of(4) {
        b.push(0);
    }
    b
}

fn world_with(npc: u64) -> World {
    let mut world = World::new(Build::Legions7160);
    world.start_entering_world();
    world.receive(blob(0x0300_0001, 1, create_player()), 0.0);
    world.receive(blob(0x0300_0002, 2, create_object(PLAYER, 1, 0)), 0.0);
    world.receive(blob(0x0300_0003, 3, create_object(npc, 4, 0)), 0.0);
    world
}

#[test]
fn a_destroyed_object_goes_at_the_next_object_maintenance() {
    let npc = 0x7000_0000_0000_0042;
    let mut world = world_with(npc);
    // `RecvEvt_DestroyObject` makes it a pending deletion; it is still there until UseTime.
    world.receive(
        blob(0x0300_0004, 4, object_event(0x0001_00C2, npc, 4, &[])),
        10.0,
    );
    assert!(world.objects.get(npc).is_some());
    assert_eq!(world.use_time(10.0), [WorldEvent::Destroyed(npc)]);
    assert!(world.objects.get(npc).is_none());
    // The player is never destroyed this way.
    world.receive(
        blob(0x0300_0005, 5, object_event(0x0001_00C2, PLAYER, 1, &[])),
        11.0,
    );
    assert!(world.use_time(11.0).is_empty());
}

/// `CObjectMaint::FadeAndDestroyObject`: the object is a pending deletion due `destruction`
/// seconds on, removed by the `UseTime` that finds it due; an unknown id queues nothing.
#[test]
fn a_faded_object_is_destroyed_once_its_destruction_time_has_passed() {
    let npc = 0x7000_0000_0000_0042;
    let mut world = world_with(npc);
    world.use_time(10.0);
    assert!(!world.fade_and_destroy_object(0x7000_0000_0000_0099, 0.5));
    assert!(world.fade_and_destroy_object(npc, 0.5));
    assert!(world.use_time(10.4).is_empty());
    assert!(world.objects.get(npc).is_some());
    assert_eq!(world.use_time(10.5), [WorldEvent::Destroyed(npc)]);
    assert!(world.objects.get(npc).is_none());
}

/// `CPhysicsObj::StickySetParent`: the child hangs from the parent's bone as a bone-index child and
/// comes off the destruction queue (`CBookKeep::RemoveObjectToBeDestroyed`).
#[test]
fn a_stuck_object_is_a_bone_index_child_and_is_not_destroyed() {
    let npc = 0x7000_0000_0000_0042;
    let mut world = world_with(npc);
    world.use_time(100.0);
    world.add_object_to_be_destroyed(npc);
    assert!(world.is_to_be_destroyed(npc));
    let offset = ac2_physics::ChildFrame::IDENTITY;
    assert!(
        !world.stick_object(npc, 0x7000_0000_0000_0099, 3, offset),
        "unknown parent"
    );
    assert!(!world.stick_object(npc, npc, 3, offset), "not to itself");
    assert!(world.stick_object(npc, PLAYER, 3, offset));
    assert!(!world.is_to_be_destroyed(npc));
    let links = world.objects.links();
    assert_eq!(links.parent_of(npc), Some(PLAYER));
    let info = links.info_of(npc).unwrap();
    assert_eq!(info.kind, ac2_physics::ChildType::BoneIndex);
    assert_eq!(info.location, 3);
}

#[test]
fn an_object_out_of_view_is_destroyed_after_25_seconds_unless_it_comes_back() {
    let npc = 0x7000_0000_0000_0042;
    let mut world = world_with(npc);
    world.use_time(100.0);
    world.add_object_to_be_destroyed(npc);
    world.add_object_to_be_destroyed(PLAYER);
    assert!(world.is_to_be_destroyed(npc) && !world.is_to_be_destroyed(PLAYER));
    assert!(world.use_time(124.9).is_empty());
    world.remove_object_to_be_destroyed(npc);
    assert!(world.use_time(126.0).is_empty());
    world.add_object_to_be_destroyed(npc);
    assert_eq!(world.use_time(151.0), [WorldEvent::Destroyed(npc)]);
}

/// `CBookKeep::ObjectBeingDestructed`: a child held by the destroyed object is queued for
/// destruction 25 s after the parent goes, and stays if it comes back.
#[test]
fn a_destroyed_objects_children_are_queued_for_destruction() {
    let npc = 0x7000_0000_0000_0042;
    let child = 0x7000_0000_0000_0043;
    let mut world = world_with(npc);
    world.receive(blob(0x0300_0004, 4, create_object(child, 1, 0)), 10.0);
    // `RecvEvt_Parent`: the child (instance 1) is held by the npc (instance 4) at location 0.
    let mut parent = npc.to_le_bytes().to_vec();
    parent.extend(4u16.to_le_bytes());
    parent.extend(1u16.to_le_bytes());
    parent.extend([0u8; 8]);
    world.receive(
        blob(0x0300_0005, 5, object_event(0x0001_0087, child, 1, &parent)),
        10.0,
    );
    assert_eq!(
        world.objects.get(child).and_then(|o| o.parent),
        Some((npc, 0))
    );
    world.receive(
        blob(0x0300_0006, 6, object_event(0x0001_00C2, npc, 4, &[])),
        10.0,
    );
    assert_eq!(world.use_time(10.0), [WorldEvent::Destroyed(npc)]);
    assert!(world.is_to_be_destroyed(child));
    assert!(world.use_time(34.9).is_empty());
    world.remove_object_to_be_destroyed(child);
    assert!(world.use_time(40.0).is_empty());
    assert!(world.objects.get(child).is_some());
    world.add_object_to_be_destroyed(child);
    assert_eq!(world.use_time(65.0), [WorldEvent::Destroyed(child)]);
}

#[test]
fn leave_world_on_a_newer_stamp_queues_the_object_for_destruction() {
    let npc = 0x7000_0000_0000_0042;
    let mut world = world_with(npc);
    world.use_time(50.0);
    // An old position stamp (0) is ignored; stamp 3 takes the object out of the world.
    world.receive(
        blob(
            0x0300_0004,
            4,
            object_event(0x0001_00BF, npc, 4, &0u16.to_le_bytes()),
        ),
        50.0,
    );
    assert!(!world.is_to_be_destroyed(npc));
    world.receive(
        blob(
            0x0300_0005,
            5,
            object_event(0x0001_00BF, npc, 4, &3u16.to_le_bytes()),
        ),
        50.0,
    );
    assert!(world.is_to_be_destroyed(npc));
    assert_eq!(world.objects.get(npc).unwrap().update_times[0], 3);
    // `RecvEvt_MoveTo` is the player's only: an object keeps its move-to stamp.
    assert_eq!(world.objects.get(npc).unwrap().update_times[3], 0);
}
