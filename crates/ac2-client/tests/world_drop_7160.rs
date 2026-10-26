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
