//! `CPhysicsObj::ApplyVisualDesc`'s part rebuild, on the real DATs (`AC2_DAT_DIR`) against the
//! Legions 7160 server double: an object whose local description adds a part group to its
//! visual has that group in its body's part tree, and a plain one does not.

mod common;

use std::sync::Arc;

use ac2_client::{DatWorld, FrameInput};
use ac2_db::ResourceStore;
use ac2_net::protocol::{PartGroupDesc, VisualDesc};
use common::*;
use legions_server_double::{create_object_with_visual, ObjectSpawn, Spawn};

const PLAIN: u64 = 0x8000_0000_0000_0042;
const ARMED: u64 = 0x8000_0000_0000_0043;
const HUMAN: u32 = 0x1F00_0023;
/// The human's root group, a connection point of it nothing hangs from, and a Setup.
const ROOT: u32 = 0x8100_0001;
const FREE_POINT: u32 = 0x8100_0015;
const SETUP: u32 = 0x0400_0029;

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

#[test]
fn a_description_that_adds_a_group_adds_it_to_the_part_tree() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let world = Arc::new(DatWorld::new(Arc::new(ResourceStore::new(dataset))).unwrap());
    let spawn = Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (mut client, server) = entered_with(spawn, world);
    server.borrow_mut().send_init_player();
    let plain = VisualDesc {
        parent: Some(HUMAN),
        ..VisualDesc::default()
    };
    let armed = VisualDesc {
        part_groups: vec![(
            0x7777,
            PartGroupDesc {
                key: 0x7777,
                parent_key: ROOT,
                connection_point: FREE_POINT,
                setup: SETUP,
                ..PartGroupDesc::default()
            },
        )],
        ..plain.clone()
    };
    for (id, desc) in [(PLAIN, &plain), (ARMED, &armed)] {
        server.borrow_mut().send_event(&create_object_with_visual(
            &at(id, [80.0, 70.0, GROUND]),
            desc,
        ));
    }
    let mut counts = (None, None);
    for _ in 0..200 {
        step(&mut client, &server, &FrameInput::default());
        counts = (
            client.part_group_count(PLAIN),
            client.part_group_count(ARMED),
        );
        if let (Some(plain), Some(armed)) = counts {
            if armed > plain {
                break;
            }
        }
    }
    let (Some(plain), Some(armed)) = counts else {
        panic!("both bodies build: {counts:?}");
    };
    assert_eq!(armed, plain + 1, "the added group follows into the tree");
}
