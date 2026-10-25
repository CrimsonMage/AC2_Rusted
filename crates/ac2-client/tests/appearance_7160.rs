//! Appearance, qualities and containment against the Legions 7160 server double: a create with
//! a full `VisualDesc`, `UpdateVisualDesc`, the quality updates but string info (private for
//! the player, visual for an object), and wield / contain.

mod common;

use ac2_client::{ClientFrame, FrameInput, Holder, ObjectFrame};
use ac2_net::protocol::{IconLayer, Opcode, PartGroupDesc, VisualDesc};
use ac2_weenie::SessionEvent;
use common::*;
use legions_server_double::{
    contain_bytes, create_object_with_visual, object_event_bytes, quality_bytes,
    update_visual_desc_bytes, ObjectSpawn, Spawn, PLAYER,
};

const NPC: u64 = 0x8000_0000_0000_0042;
const SWORD: u64 = 0x8000_0000_0000_0050;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

fn object(frame: &ClientFrame, id: u64) -> Option<ObjectFrame> {
    frame.objects.iter().find(|o| o.id == id).cloned()
}

fn at(id: u64, origin: [f32; 3]) -> ObjectSpawn {
    ObjectSpawn {
        id,
        instance: 3,
        visual: 0,
        cell: CELL,
        origin,
        rotation: [1.0, 0.0, 0.0, 0.0],
        stamps: [10, 1, 1, 0],
        name: "Birch",
    }
}

fn group(key: u32, parent_key: u32, connection_point: u32, setup: u32) -> (u32, PartGroupDesc) {
    (
        key,
        PartGroupDesc {
            key,
            parent_key,
            connection_point,
            setup,
            ..PartGroupDesc::default()
        },
    )
}

/// A human with a body Setup, a helmet on connection point 4 tinted by an appearance table,
/// and a global skin-colour modifier.
fn dressed() -> VisualDesc {
    let mut helmet = group(0x20, 0x10, 4, 0x0200_0BEE);
    helmet.1.appearance = vec![(0x2000_0011, vec![(0x4000_0001, 0.5)])];
    VisualDesc {
        parent: Some(0x1F00_0023),
        scale: Some([1.0, 1.0, 1.05]),
        icon: vec![IconLayer {
            layer: 1,
            image: 0x0600_1234,
            color: 0xFF80_8080,
        }],
        global_mods: Some(PartGroupDesc {
            appearance: vec![(0x2000_0002, vec![(0x4000_0007, 0.25)])],
            ..PartGroupDesc::default()
        }),
        part_groups: vec![group(0x10, 0, 0, 0x0200_0001), helmet],
        ..VisualDesc::default()
    }
}

fn frames(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    count: usize,
) -> Vec<SessionEvent> {
    let idle = FrameInput::default();
    (0..count)
        .flat_map(|_| step(client, server, &idle))
        .collect()
}

#[test]
fn the_create_and_update_visual_descriptions_reach_the_object() {
    let (mut client, server) = entered(spawn());
    server.borrow_mut().send_event(&create_object_with_visual(
        &at(NPC, [80.0, 70.0, GROUND]),
        &dressed(),
    ));
    frames(&mut client, &server, 4);
    let npc = object(&client.frame(), NPC).expect("the object");
    let a = &npc.appearance;
    assert_eq!(npc.visual, Some(0x1F00_0023));
    assert_eq!(a.parent, 0x1F00_0023);
    assert_eq!(a.scale, [1.0, 1.0, 1.05]);
    assert_eq!(a.icon, vec![(1, 0x0600_1234, 0xFF80_8080)]);
    assert_eq!(a.root, Some(0x10));
    assert_eq!(a.children, vec![(0x10, vec![0x20])]);
    let helmet = a.group(0x20).unwrap();
    assert_eq!((helmet.setup, helmet.connection_point), (0x0200_0BEE, 4));
    assert_eq!(
        helmet.appearance,
        vec![(0x2000_0011, vec![(0x4000_0001, 0.5)])]
    );
    assert_eq!(
        a.global.appearance,
        vec![(0x2000_0002, vec![(0x4000_0007, 0.25)])]
    );

    // `UpdateVisualDesc` replaces the local description: the helmet comes off.
    let undressed = VisualDesc {
        parent: Some(0x1F00_0023),
        part_groups: vec![group(0x10, 0, 0, 0x0200_0001)],
        ..VisualDesc::default()
    };
    server
        .borrow_mut()
        .send_event(&update_visual_desc_bytes(NPC, 3, &undressed));
    frames(&mut client, &server, 2);
    let a = object(&client.frame(), NPC).unwrap().appearance;
    assert!(a.group(0x20).is_none());
    assert_eq!(a.scale, [1.0; 3]);
    assert!(a.global.appearance.is_empty());
}

#[test]
fn the_quality_updates_reach_the_player_and_objects() {
    let (mut client, server) = entered(spawn());
    server.borrow_mut().send_event(&create_object_with_visual(
        &at(NPC, [80.0, 70.0, GROUND]),
        &dressed(),
    ));
    frames(&mut client, &server, 4);
    let mut string = Vec::new();
    ac2_net::pack_string(&mut string, 0, b"Gorvath");
    let mut position = 0x7F7F_001Cu32.to_le_bytes().to_vec();
    for v in [1.0f32, 2.0, 3.0, 1.0, 0.0, 0.0, 0.0] {
        position.extend(v.to_le_bytes());
    }
    let kinds: Vec<(u32, u32, Vec<u8>, u32)> = vec![
        (
            Opcode::UPDATE_INT_PRIVATE,
            0x19,
            (-7i32).to_le_bytes().to_vec(),
            4,
        ),
        (
            Opcode::UPDATE_BOOL_PRIVATE,
            0x30,
            1u32.to_le_bytes().to_vec(),
            1,
        ),
        (
            Opcode::UPDATE_FLOAT_PRIVATE,
            0x40,
            2.5f32.to_le_bytes().to_vec(),
            3,
        ),
        (Opcode::UPDATE_STRING_PRIVATE, 0x2000, string.clone(), 7),
        (
            Opcode::UPDATE_DATA_ID_PRIVATE,
            0x50,
            0x0600_0001u32.to_le_bytes().to_vec(),
            2,
        ),
        (
            Opcode::UPDATE_INSTANCE_ID_PRIVATE,
            0x60,
            NPC.to_le_bytes().to_vec(),
            5,
        ),
        (Opcode::UPDATE_POSITION_PRIVATE, 0x70, position.clone(), 6),
        (
            Opcode::UPDATE_TIMESTAMP_PRIVATE,
            0x80,
            99.5f64.to_le_bytes().to_vec(),
            8,
        ),
        (
            Opcode::UPDATE_LONG_INT_PRIVATE,
            0x90,
            (-3i64).to_le_bytes().to_vec(),
            12,
        ),
    ];
    for (opcode, quality, value, _) in &kinds {
        server
            .borrow_mut()
            .send_event(&quality_bytes(*opcode, None, *quality, value));
        // The visual form names the object first.
        server
            .borrow_mut()
            .send_event(&quality_bytes(opcode + 1, Some((NPC, 3)), *quality, value));
    }
    let events = frames(&mut client, &server, 4);
    let changed: Vec<(u64, u32, u32)> = events
        .iter()
        .filter_map(|e| match e {
            SessionEvent::QualityChanged {
                object,
                table,
                quality,
            } => Some((*object, *table, *quality)),
            _ => None,
        })
        .collect();
    let mut expected = Vec::new();
    for (_, quality, _, table) in &kinds {
        expected.push((PLAYER, *table, *quality));
        expected.push((NPC, *table, *quality));
    }
    assert_eq!(changed, expected);
    let objects = client.session().objects();
    for id in [PLAYER, NPC] {
        let q = &objects.get(id).unwrap().qualities;
        assert_eq!(q.ints.get(&0x19), Some(&-7));
        assert_eq!(q.bools.get(&0x30), Some(&1));
        assert_eq!(q.floats.get(&0x40), Some(&2.5));
        assert_eq!(
            q.strings.get(&0x2000).map(Vec::as_slice),
            Some(&b"Gorvath"[..])
        );
        assert_eq!(q.dids.get(&0x50), Some(&0x0600_0001));
        assert_eq!(q.iids.get(&0x60), Some(&NPC));
        assert_eq!(
            q.positions.get(&0x70).map(|p| p.origin),
            Some([1.0, 2.0, 3.0])
        );
        assert_eq!(q.timestamps.get(&0x80), Some(&99.5));
        assert_eq!(q.long_ints.get(&0x90), Some(&-3));
    }
}

#[test]
fn wielding_containing_and_the_weenie_container_follow_the_events() {
    let (mut client, server) = entered(spawn());
    server.borrow_mut().send_event(&create_object_with_visual(
        &at(NPC, [80.0, 70.0, GROUND]),
        &dressed(),
    ));
    server.borrow_mut().send_event(&create_object_with_visual(
        &at(SWORD, [75.0, 70.0, GROUND]),
        &dressed(),
    ));
    frames(&mut client, &server, 4);
    // `Parent` (newer position stamp 11): the NPC wields the sword in holding location 2 with
    // placement frame 5.
    let mut args = NPC.to_le_bytes().to_vec();
    args.extend(3u16.to_le_bytes());
    args.extend(11u16.to_le_bytes());
    args.extend([2u32, 5].iter().flat_map(|v| v.to_le_bytes()));
    let words: Vec<u32> = args
        .chunks(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    server
        .borrow_mut()
        .send_event(&object_event_bytes(0x0001_0087, SWORD, 3, &words));
    // The weenie side: the sword's wielder quality (0x105).
    server.borrow_mut().send_event(&quality_bytes(
        Opcode::UPDATE_INSTANCE_ID_VISUAL,
        Some((SWORD, 3)),
        0x105,
        &NPC.to_le_bytes(),
    ));
    frames(&mut client, &server, 2);
    let sword = object(&client.frame(), SWORD).expect("held objects are listed");
    assert_eq!(
        sword.holder,
        Some(Holder {
            id: NPC,
            location: 2,
            orientation: Some(5)
        })
    );
    assert!(!sword.in_world);
    assert_eq!(sword.wielder, Some(NPC));

    // `Contain` (stamp 12): into the player's pack, out of the frame, and not destroyed.
    server.borrow_mut().send_event(&contain_bytes(SWORD, 3, 12));
    server.borrow_mut().send_event(&quality_bytes(
        Opcode::UPDATE_INSTANCE_ID_VISUAL,
        Some((SWORD, 3)),
        0x102,
        &PLAYER.to_le_bytes(),
    ));
    let events = frames(&mut client, &server, 2);
    assert!(
        events.contains(&SessionEvent::ObjectContained(SWORD)),
        "{events:?}"
    );
    assert!(object(&client.frame(), SWORD).is_none());
    let table = client.session().objects();
    let sword = table.get(SWORD).unwrap();
    assert!(sword.contained && sword.parent.is_none());
    assert_eq!(
        sword.qualities.weenie.as_ref().unwrap().container,
        Some(PLAYER)
    );
    assert!(!client.session().is_to_be_destroyed(SWORD));
    // A stale contain changes nothing.
    server.borrow_mut().send_event(&contain_bytes(NPC, 3, 1));
    let events = frames(&mut client, &server, 2);
    assert!(!events.contains(&SessionEvent::ObjectContained(NPC)));
}
