//! Dragging items between the pack's slots, into a side pack, onto the paper doll and back, through
//! the HUD with the pointer the way the window feeds it, against the Legions server double
//! (`AC2_DAT_DIR`, a GPU adapter). Each drop is the request `gmCWInterface::MoveItem`
//! (`CContainer` 0x7, or the reorganization `CInventorySystem` 0x19 inside the player's own pack),
//! `EquipItem` (`Inventory` 0x21) or `UnEquipItem` (`Inventory` 0x9) sends.
#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_net::protocol::{ClientEvent, InvMoveDesc, ScriptRequest};
use ac2_ui::ToolbarChild;
use hud::Rig;
use legions_server_double::{PACK, PLAYER, POTIONS, PRIMARY_HAND, RATIONS, SECONDARY_HAND, SWORD};

/// The inventory requests the double heard from the `since`th on (a press also selects:
/// `SetTarget`, which is left out).
fn requests(rig: &Rig, since: usize) -> Vec<ScriptRequest> {
    rig.server.borrow().script_requests[since..]
        .iter()
        .filter(|r| !matches!(r, ScriptRequest::SetTarget(_)))
        .cloned()
        .collect()
}

fn count(rig: &Rig) -> usize {
    rig.server.borrow().script_requests.len()
}

/// The bytes a request goes out as.
fn wire(request: &ScriptRequest) -> Vec<u8> {
    match request.to_event() {
        ClientEvent::Interp { arguments, .. } => arguments,
        other => panic!("{other:?}"),
    }
}

fn with_pack() -> Option<Rig> {
    let mut rig = Rig::new()?;
    rig.open_inventory();
    rig.wait("the sword in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", SWORD)
            .is_some()
    });
    Some(rig)
}

#[test]
fn an_item_dragged_to_another_slot_of_the_pack_reorganizes_it() {
    let Some(mut rig) = with_pack() else {
        return;
    };
    let before = count(&rig);
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    let to = rig
        .view
        .container_slot_center("PackMainContainer", 0)
        .unwrap();
    rig.drag(from, to);
    // The sword from slot 1 to slot 0 of the player's own pack: `ReorganizeContents`.
    let sent = requests(&rig, before);
    let expected =
        ScriptRequest::ReorganizeContents(InvMoveDesc::new(SWORD, PLAYER, 1, PLAYER, 0, 1));
    assert_eq!(sent, std::slice::from_ref(&expected));
    assert_eq!(wire(&sent[0]), wire(&expected));
}

#[test]
fn an_item_dragged_onto_a_slot_of_the_doll_is_equipped_there() {
    let Some(mut rig) = with_pack() else {
        return;
    };
    let before = count(&rig);
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    let to = rig.view.ui_element_center("SecondaryHand_Slot").unwrap();
    rig.drag(from, to);
    // `HandlePaperDollDrop`: `EquipItem(sword, SecondaryHand)`.
    let sent = requests(&rig, before);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let ScriptRequest::EquipItem(desc) = &sent[0] else {
        panic!("{sent:?}");
    };
    assert_eq!((desc.item(), desc.location()), (SWORD, 0x80));
}

/// `ContainerDisplay::GetNewIndex`: a drop past the last item lands on the slot after it, and the
/// move of the item that is already last stays where it is (`FinishListBoxDrop`), nothing sent.
#[test]
fn an_item_dropped_past_the_end_of_the_pack_stays_last() {
    let Some(mut rig) = with_pack() else {
        return;
    };
    let before = count(&rig);
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    let to = rig
        .view
        .container_slot_center("PackMainContainer", 5)
        .unwrap();
    rig.press_and_move(from, to);
    assert!(rig.view.is_dragging());
    rig.release();
    assert!(requests(&rig, before).is_empty());
}

/// The side pack opened beside the inventory (`gmExternalContainerUI`).
fn open_side_pack(rig: &mut Rig) {
    rig.command(Command::SetContainerOpen {
        container: PACK,
        open: true,
    });
    rig.wait("the rations in the side pack", |r| {
        r.view
            .item_icon_center("ExternalContainer", RATIONS)
            .is_some()
    });
}

#[test]
fn an_item_dragged_into_the_open_side_pack_is_moved_there() {
    let Some(mut rig) = with_pack() else {
        return;
    };
    open_side_pack(&mut rig);
    let before = count(&rig);
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    // The side pack holds the rations: a drop on its second slot lands after them.
    let to = rig
        .view
        .container_slot_center("ExternalContainer", 1)
        .unwrap();
    rig.drag(from, to);
    let sent = requests(&rig, before);
    let expected = ScriptRequest::MoveItem(InvMoveDesc::new(SWORD, PLAYER, 1, PACK, 1, 1));
    assert_eq!(sent, std::slice::from_ref(&expected));
    assert_eq!(wire(&sent[0]), wire(&expected));
}

/// `HandlePaperDollDrop`: an item from another container (`FromID` not the player) only comes
/// into the player's pack: retail returns after the move.
#[test]
fn an_item_from_a_side_pack_dropped_on_the_doll_only_comes_into_the_pack() {
    let Some(mut rig) = with_pack() else {
        return;
    };
    open_side_pack(&mut rig);
    let before = count(&rig);
    let from = rig
        .view
        .item_icon_center("ExternalContainer", RATIONS)
        .unwrap();
    let to = rig.view.ui_element_center("PrimaryHand_Slot").unwrap();
    rig.drag(from, to);
    let sent = requests(&rig, before);
    assert_eq!(
        sent,
        [ScriptRequest::MoveItem(InvMoveDesc::new(
            RATIONS, PACK, 0, PLAYER, 0, 1
        ))]
    );
}

/// Equipping, then taking it off by dragging it from its slot into the pack
/// (`gmPaperDollUI::HandlePaperDollDrag`, `ContainerDisplay::HandleListBoxDrop`'s unequip).
#[test]
fn a_worn_item_dragged_off_the_doll_into_the_pack_comes_off() {
    let Some(mut rig) = with_pack() else {
        return;
    };
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    rig.drag(
        from,
        rig.view.ui_element_center("SecondaryHand_Slot").unwrap(),
    );
    rig.wait("the sword worn", |r| {
        r.view
            .item_icon_center("SecondaryHand_Slot", SWORD)
            .is_some()
    });
    assert_eq!(rig.view.paper_doll_item(SECONDARY_HAND), Some(SWORD));
    // A worn item is not dropped on the world: it comes off first.
    let before = count(&rig);
    let worn = rig
        .view
        .item_icon_center("SecondaryHand_Slot", SWORD)
        .unwrap();
    rig.drag(worn, (300, 250));
    assert!(requests(&rig, before).is_empty());
    // Off the doll, onto the pack's first slot.
    let to = rig
        .view
        .container_slot_center("PackMainContainer", 0)
        .unwrap();
    rig.drag(worn, to);
    let sent = requests(&rig, before);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let ScriptRequest::UnequipItem(desc) = &sent[0] else {
        panic!("{sent:?}");
    };
    assert_eq!(desc.item(), SWORD);
    assert_eq!(desc.slot(), 0);
    rig.wait("the sword back in the pack", |r| {
        r.view.paper_doll_item(SECONDARY_HAND).is_none()
    });
}

/// A drop on the doll's figure equips at the item's preferred location (the sword's is the
/// primary hand).
#[test]
fn an_item_dropped_on_the_dolls_figure_goes_to_its_preferred_hand() {
    let Some(mut rig) = with_pack() else {
        return;
    };
    let before = count(&rig);
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    rig.view.show_panel(ToolbarChild::Inventory, true);
    let figure = rig.view.ui_element_center("PaperDollViewport").unwrap();
    rig.drag(from, figure);
    let sent = requests(&rig, before);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let ScriptRequest::EquipItem(desc) = &sent[0] else {
        panic!("{sent:?}");
    };
    assert_eq!((desc.item(), desc.location()), (SWORD, PRIMARY_HAND));
}

/// `gmBackpackUI::ListenToMessage`: an item dropped on another page's tab goes to the end of that
/// page (`FinishItemDrop`: a move inside the player's container, to the first slot after the
/// page's items).
#[test]
fn an_item_dropped_on_another_pages_tab_goes_to_the_end_of_that_page() {
    let Some(mut rig) = with_pack() else {
        return;
    };
    let before = count(&rig);
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    let tab = rig.view.ui_element_center("PackOneButton").unwrap();
    rig.drag(from, tab);
    let sent = requests(&rig, before);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let ScriptRequest::ReorganizeContents(desc) = &sent[0] else {
        panic!("{sent:?}");
    };
    // The sword from slot 1 to the first slot of the second page (the main page's layout has 42 slots: `m_firstSlotNumber + _count`).
    assert_eq!(
        (
            desc.item(),
            desc.from_slot(),
            desc.target(),
            desc.target_slot()
        ),
        (SWORD, 1, PLAYER, 42)
    );
    // The tab of the page shown takes nothing.
    let before = count(&rig);
    let shown = rig.view.ui_element_center("PackMainButton").unwrap();
    let from = rig
        .view
        .item_icon_center("PackMainContainer", POTIONS)
        .unwrap();
    rig.drag(from, shown);
    assert!(requests(&rig, before).is_empty());
}
