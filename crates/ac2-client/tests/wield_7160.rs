//! A character wields an item, against the Legions server double over the real DATs
//! (`AC2_DAT_DIR`). The double sends the item's Parent (0x10087) naming the character and its
//! hand's holding location; the client hangs the item from that holding point
//! (`CPhysicsObj::RecvEvt_Parent`, `CPartArray::GetHoldingLocation`), so the item is drawn at the
//! hand and moves with the character's posed hand bone each frame (`view/held.rs`).

mod common;

use std::sync::Arc;

use ac2_anim::Ac2Setup;
use ac2_client::{DatWorld, FrameInput};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use common::*;
use legions_server_double::{Spawn, GROUND_ITEM, PLAYER};

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// The right hand's holding location: `CPhysicsObj::UpdateChild` treats locations 1 and 2 as
/// the hands (its water fade), and the human Setup lists 1 on `rweaponholder` and 2 on
/// `lweaponholder` (`gmbonetype` 0x81000002 / 0x81000003).
const RIGHT_HAND: u32 = 1;

/// `gmbonetype` `rweaponholder`.
const RIGHT_WEAPON_HOLDER: u32 = 0x8100_0002;

/// The bone the character's root Setup hangs `location` from, and that bone's name.
fn holding_bone(store: &ResourceStore, visual: u32, location: u32) -> (usize, u32) {
    // The character's visual is a VisualDesc: its root part group's Setup lists the holding points
    // (`CPartArray::GetHoldingLocation` on the root group).
    let groups = ac2_render::object::resolve_drawn_groups(store, DataId::new(visual), None)
        .expect("the character's part groups read")
        .expect("the character's part groups");
    let setup = store
        .load_cached_packed::<Ac2Setup>(TypedResourceRef::new(
            DatFileKind::Portal,
            groups[0].setup,
            0,
        ))
        .expect("the character's Setup reads")
        .expect("the character's Setup");
    let point = setup
        .holding_points
        .iter()
        .find(|point| point.id == location)
        .expect("the character's Setup lists the location");
    (point.bone, setup.bones[point.bone].name)
}

#[test]
fn a_character_wields_an_item_from_its_hand() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let world = Arc::new(DatWorld::new(Arc::clone(&store)).unwrap());
    let (mut client, server) = entered_with(spawn(), world);
    {
        let mut state = server.borrow_mut();
        state.send_init_player();
        state.spawn_world_objects();
    }
    step(&mut client, &server, &FrameInput::default());
    let visual = client
        .frame()
        .player
        .as_ref()
        .and_then(|player| player.visual)
        .expect("the character's visual");
    let location = RIGHT_HAND;
    let (bone, name) = holding_bone(&store, visual, location);
    assert_eq!(name, RIGHT_WEAPON_HOLDER, "location 1 is on the right weapon holder");

    server.borrow_mut().wield(GROUND_ITEM, PLAYER, location, 0);
    // The item's holder point needs the character's body in (the loader thread); wait on it with
    // a frame budget.
    let mut point = None;
    let mut parented = false;
    for _ in 0..600 {
        // `CPhysicsObj::RecvEvt_Parent` takes the parent once (`set_parent`).
        parented |= step(&mut client, &server, &FrameInput::default()).contains(
            &ac2_weenie::SessionEvent::ObjectParented {
                child: GROUND_ITEM,
                holder: PLAYER,
            },
        );
        point = client
            .frame()
            .objects
            .iter()
            .find(|object| object.id == GROUND_ITEM)
            .and_then(|object| object.holder)
            .and_then(|holder| holder.point);
        if point.is_some() {
            break;
        }
    }
    let frame = client.frame();
    let item = frame
        .objects
        .iter()
        .find(|object| object.id == GROUND_ITEM)
        .expect("the item is in the world");
    let holder = item.holder.expect("the item hangs from the character");
    assert_eq!(holder.id, PLAYER);
    assert_eq!(holder.location, location);
    let point = point.expect("the character's hand is a holding point of its body");
    assert!(parented, "the Parent was taken");
    assert_eq!(point.bone, bone, "the item hangs from the hand bone");
    assert_eq!(point.group, 0, "the hand is the root Setup's");
    assert_eq!(
        point.transform,
        ac2_anim::Ac2Transform::IDENTITY,
        "location 1 sits on the bone itself"
    );
}
