//! Dropping an item from the pack into the world (`gm3DItemsUI`), through the HUD with the
//! pointer the way the window feeds it, against the Legions server double (`AC2_DAT_DIR`, a GPU
//! adapter). The drop finds its target with the selection render (the object under the pointer,
//! none here: the ground), a stack asks how many first (`gmStackUI`), and
//! `gmCWInterface::MoveItem` (`CContainer` 0x7 -> `CInventorySystem` 0x12) goes out.
#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_net::protocol::{InvMoveDesc, ScriptRequest};
use hud::Rig;
use legions_server_double::{PACK, PLAYER, POTIONS, RATIONS, SWORD};

/// The `MoveItem` requests the double heard, with the bytes they came as.
fn moves(rig: &Rig) -> Vec<(InvMoveDesc, Vec<u8>)> {
    let server = rig.server.borrow();
    let wire = |request: &ScriptRequest| match request.to_event() {
        ac2_net::protocol::ClientEvent::Interp { arguments, .. } => arguments,
        other => panic!("{other:?}"),
    };
    server
        .script_requests
        .iter()
        .filter_map(|request| match request {
            ScriptRequest::MoveItem(desc) => Some((desc.clone(), wire(request))),
            _ => None,
        })
        .collect()
}

#[test]
fn an_item_dropped_on_the_world_moves_to_the_ground() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.open_inventory();
    rig.wait("the sword in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", SWORD)
            .is_some()
    });
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    rig.press_and_move(from, (300, 250));
    assert!(
        rig.view.is_dragging(),
        "the sword's icon follows the pointer"
    );
    rig.release();
    // The sword was in slot 1 of the pack; the target none, the slot 0, one of it.
    let expected = InvMoveDesc::new(SWORD, PLAYER, 1, 0, 0, 1);
    let sent = moves(&rig);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, expected);
    let wire = match ScriptRequest::MoveItem(expected).to_event() {
        ac2_net::protocol::ClientEvent::Interp { arguments, .. } => arguments,
        other => panic!("{other:?}"),
    };
    assert_eq!(sent[0].1, wire);
}

/// A stack asks how many (`gmStackUI`, the default the whole stack up to its maximum) before the
/// move goes out.
#[test]
fn a_stack_dropped_on_the_world_asks_how_many_first() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.open_inventory();
    rig.wait("the potions in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", POTIONS)
            .is_some()
    });
    let from = rig
        .view
        .item_icon_center("PackMainContainer", POTIONS)
        .unwrap();
    rig.drag(from, (300, 250));
    assert!(moves(&rig).is_empty(), "nothing moves before the answer");
    let ok = rig
        .view
        .ui_element_center("StackOkButton")
        .expect("the stack dialog asks how many");
    rig.click(ok);
    // Five potions in slot 0 of the pack.
    let sent = moves(&rig);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, InvMoveDesc::new(POTIONS, PLAYER, 0, 0, 0, 5));
}

/// `gm3DItemsUI::StartItemDrop` takes only items in the player's own pack: one in a side pack
/// (its `FromID` the pack) does not start a drop.
#[test]
fn an_item_from_a_side_pack_is_not_dropped_on_the_world() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.open_inventory();
    rig.command(Command::SetContainerOpen {
        container: PACK,
        open: true,
    });
    rig.idle(8);
    rig.wait("the rations in the open side pack", |r| {
        r.view.item_icon_center("", RATIONS).is_some()
    });
    let from = rig.view.item_icon_center("", RATIONS).unwrap();
    rig.drag(from, (300, 250));
    assert!(moves(&rig).is_empty(), "{:?}", moves(&rig));
}

/// A creature in the world under the pointer is the drop's target (`FinishFind3DTarget`: the
/// object the selection render found): the item goes to it, a give, not to the ground.
#[test]
fn an_item_dropped_on_a_creature_is_given_to_it() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    let creatures = [
        (0x8000_0000_0000_0B01_u64, [0.0, 10.0]),
        (0x8000_0000_0000_0B02, [4.0, 8.0]),
        (0x8000_0000_0000_0B03, [-4.0, 8.0]),
        (0x8000_0000_0000_0B04, [8.0, 8.0]),
        (0x8000_0000_0000_0B05, [-8.0, 8.0]),
        (0x8000_0000_0000_0B06, [0.0, -10.0]),
    ];
    for (id, offset) in creatures {
        // The player's own model, as a creature's.
        rig.server
            .borrow_mut()
            .spawn_visible_creature(id, offset, 0x1F00_0023);
    }
    rig.open_inventory();
    // Frames until the creature models load and one of them is where the selection render
    // finds it (the model's middle at some height, on the screen).
    let mut found = None;
    for _ in 0..600 {
        rig.idle(1);
        let frame = rig.client.frame();
        let aspect = hud::SIZE.0 as f32 / hud::SIZE.1 as f32;
        rig.view.settle(&rig.gpu, &frame, aspect);
        found = creatures.iter().find_map(|&(id, _)| {
            [0.5, 1.0, 1.5].iter().find_map(|&height| {
                let pixel = rig.view.object_pixel(&frame, id, height)?;
                let on_screen = (0..hud::SIZE.0 as i32).contains(&pixel.0)
                    && (0..hud::SIZE.1 as i32).contains(&pixel.1);
                (on_screen && rig.view.pick(&frame, hud::SIZE, pixel) == Some(id))
                    .then_some((id, pixel))
            })
        });
        if found.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let (target, pixel) = found.expect("a creature the selection render finds");
    rig.wait("the sword in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", SWORD)
            .is_some()
    });
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    rig.drag(from, pixel);
    let sent = moves(&rig);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, InvMoveDesc::new(SWORD, PLAYER, 1, target, 0, 1));
}
