//! Dragging onto the shortcut bar, through the HUD with the pointer the way the window feeds it
//! (`UI_POINTER_X` / `UI_POINTER_Y` and the left button), against the Legions server double
//! (`AC2_DAT_DIR`, a GPU adapter). A drop on a slot is `gmShortcutUI::SetShortcut`, which asks
//! the player's script (`gmCWInterface::SetShortcut`, IFun 0xB4006F) and so sends the `WM_Player`
//! call 0x8212_0018: the `ShortcutInfo` native (`ShortcutInfo::StreamPack`: its type, then the
//! datum) and the slot, as a `PackBuffer`.
#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_net::protocol::{pack_values, NativeObject, PackValue, ScriptCall, ShortcutInfo};
use ac2_ui::ToolbarChild;
use hud::Rig;
use legions_server_double::{SkillCost, SWORD};

/// A trainable, active skill of the melee panel.
const SKILL: u32 = 0xB0;
/// `gmCWInterface::SetShortcut`'s `WM_Player` call.
const SET_SHORTCUT: u32 = 0x8212_0018;
/// `ShortcutInfo::_type` of a skill and an item.
const SKILL_TYPE: u32 = 0x4000_0001;

fn skill() -> ShortcutInfo {
    ShortcutInfo::Enum {
        kind: SKILL_TYPE,
        value: SKILL,
    }
}

/// The player with `SKILL` trained, the bar built and the skill panel open on the melee page.
fn with_skill_panel() -> Option<Rig> {
    let mut rig = Rig::new()?;
    {
        let mut server = rig.server.borrow_mut();
        server.give_starter_items();
        server.send_init_player();
        server.grant_xp(100);
        server.set_skill_cost(SKILL, SkillCost { credits: 2, xp: 7 });
    }
    rig.idle(16);
    rig.command(Command::Skill(ac2_weenie::SkillRequest::Train(SKILL)));
    rig.idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    rig.panel(ToolbarChild::Skill, true);
    let melee = rig
        .client
        .root_panel_from_race(ac2_ui::ROOT_MELEE_PANEL)
        .unwrap()
        .unwrap();
    rig.command(Command::Examine(ac2_net::protocol::ExaminationRequest {
        kind: ac2_ui::EXAMINE_PANEL,
        data_enum: melee,
        ..Default::default()
    }));
    rig.wait("the skill node", |r| {
        r.view.skill_node_center(SKILL).is_some()
    });
    Some(rig)
}

/// The `SetShortcut` calls the player sent from the `since`th call on: (slot, shortcut).
fn set_shortcuts(rig: &Rig, since: usize) -> Vec<(u32, ShortcutInfo)> {
    rig.server.borrow().script_calls[since..]
        .iter()
        .filter(|call| call.function == SET_SHORTCUT)
        .map(|call| {
            let [shortcut, PackValue::Int(slot)] = &call.arguments[..] else {
                panic!("{call:?}");
            };
            let Some(NativeObject::ShortcutInfo(shortcut)) = shortcut.as_native() else {
                panic!("{call:?}");
            };
            (*slot, shortcut.clone())
        })
        .collect()
}

fn sorted(mut calls: Vec<(u32, ShortcutInfo)>) -> Vec<(u32, ShortcutInfo)> {
    calls.sort_by_key(|(slot, _)| *slot);
    calls
}

fn calls(rig: &Rig) -> usize {
    rig.server.borrow().script_calls.len()
}

fn slot(rig: &mut Rig, index: usize) -> ShortcutInfo {
    rig.client.shortcut_array().unwrap()[index].clone()
}

#[test]
fn a_skill_dropped_on_a_slot_sets_the_shortcut_and_sends_it() {
    let Some(mut rig) = with_skill_panel() else {
        return;
    };
    assert_eq!(slot(&mut rig, 5), ShortcutInfo::Empty(0));
    let (from, to) = (
        rig.view.skill_node_center(SKILL).unwrap(),
        rig.view.shortcut_slot_center(5).unwrap(),
    );
    let (before, bytes_before) = (calls(&rig), rig.server.borrow().script_bytes.len());
    rig.press_and_move(from, to);
    assert!(rig.view.is_dragging(), "the skill icon follows the pointer");
    rig.release();
    assert!(!rig.view.is_dragging());
    assert_eq!(set_shortcuts(&rig, before), [(5, skill())]);
    assert_eq!(slot(&mut rig, 5), skill());
    // The bytes: the native `ShortcutInfo` (0xAC) as `PackPackage`, then the slot as `PackInt`.
    let sent: Vec<Vec<u8>> = rig.server.borrow().script_bytes[bytes_before..]
        .iter()
        .filter(|(function, _)| *function == SET_SHORTCUT)
        .map(|(_, bytes)| bytes.clone())
        .collect();
    let call = ScriptCall::new(
        SET_SHORTCUT,
        vec![
            PackValue::native(NativeObject::ShortcutInfo(skill())),
            PackValue::Int(5),
        ],
    )
    .unwrap();
    // The VM numbers the reference (its third word); the rest is the codec's `PackBuffer`.
    let words = |bytes: &[u8]| -> Vec<u32> {
        bytes
            .chunks(4)
            .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
            .collect()
    };
    let (mut sent, mut expected) = (words(&sent[0]), words(&pack_values(&call.arguments)));
    assert_eq!(sent.len(), 11);
    sent[2] = 0;
    expected[2] = 0;
    assert_eq!(sent, expected);
    // `PackPackage` (5); the native 0xAC; `StreamPack`'s type 0x40000001 and the skill; `PackInt`
    // (1) and the slot.
    assert_eq!(sent[0], 5);
    assert_eq!(sent[5..9], [0xFFFF_00AC, SKILL_TYPE, SKILL, 0]);
    assert_eq!(sent[9..], [1, 5]);
    assert!(rig.client.take_script_problems().is_empty());
}

#[test]
fn a_shortcut_dragged_between_slots_clears_the_old_one_then_sets_the_new() {
    let Some(mut rig) = with_skill_panel() else {
        return;
    };
    let (from, to) = (
        rig.view.skill_node_center(SKILL).unwrap(),
        rig.view.shortcut_slot_center(5).unwrap(),
    );
    rig.drag(from, to);
    let before = calls(&rig);
    let (from, to) = (
        rig.view.shortcut_slot_center(5).unwrap(),
        rig.view.shortcut_slot_center(7).unwrap(),
    );
    rig.drag(from, to);
    // `gmShortcutUI::ListenToMessage`: `RemoveShortcut` of the slot it came from, then
    // `SetShortcut` of the slot under the pointer (the unit test keeps that order out of the
    // bar; the two leave in one frame, so the net layer's priority queue may swap them).
    assert_eq!(
        sorted(set_shortcuts(&rig, before)),
        [(5, ShortcutInfo::Empty(0)), (7, skill())]
    );
    assert_eq!(slot(&mut rig, 5), ShortcutInfo::Empty(0));
    assert_eq!(slot(&mut rig, 7), skill());
}

#[test]
fn a_shortcut_dragged_off_the_bar_is_removed() {
    let Some(mut rig) = with_skill_panel() else {
        return;
    };
    let (from, to) = (
        rig.view.skill_node_center(SKILL).unwrap(),
        rig.view.shortcut_slot_center(5).unwrap(),
    );
    rig.drag(from, to);
    let before = calls(&rig);
    // Released over the world: `UIElement::DragAndDropComplete` tells the owner it was not
    // dropped, and the bar removes the shortcut.
    let from = rig.view.shortcut_slot_center(5).unwrap();
    rig.press_and_move(from, (400, 250));
    eprintln!(
        "DRAGGING {} field {:?} {:?}",
        rig.view.is_dragging(),
        rig.view.ui_element_rect("ItemSelectionField"),
        rig.view.ui_tree().unwrap().roots().len()
    );
    rig.release();
    assert_eq!(set_shortcuts(&rig, before), [(5, ShortcutInfo::Empty(0))]);
    assert_eq!(slot(&mut rig, 5), ShortcutInfo::Empty(0));
}

#[test]
fn a_skill_off_the_skill_panel_and_not_on_the_bar_changes_nothing() {
    let Some(mut rig) = with_skill_panel() else {
        return;
    };
    let before = calls(&rig);
    let from = rig.view.skill_node_center(SKILL).unwrap();
    rig.drag(from, (400, 250));
    assert!(set_shortcuts(&rig, before).is_empty());
}

/// `gmShortcutUI_DOCB`: with the bar locked (`GetShortcutLocked`) a shortcut leaves its slot
/// only while shift is held.
#[test]
fn a_locked_bar_lets_a_shortcut_move_only_with_shift() {
    let Some(mut rig) = with_skill_panel() else {
        return;
    };
    let (from, to) = (
        rig.view.skill_node_center(SKILL).unwrap(),
        rig.view.shortcut_slot_center(5).unwrap(),
    );
    rig.drag(from, to);
    // The options page: the lock ticked and applied; the bar reloads its options.
    rig.view.character_open();
    rig.view.character_tick("ID_Option_ShortcutLocked", true);
    rig.view.character_press("ApplyButton");
    rig.idle(16);
    let before = calls(&rig);
    let (from, to) = (
        rig.view.shortcut_slot_center(5).unwrap(),
        rig.view.shortcut_slot_center(7).unwrap(),
    );
    rig.press_and_move(from, to);
    assert!(!rig.view.is_dragging(), "the locked bar holds its slot");
    rig.release();
    assert!(set_shortcuts(&rig, before).is_empty());
    // With shift held at the pull, it moves.
    rig.shift = true;
    rig.drag(from, to);
    rig.shift = false;
    assert_eq!(
        sorted(set_shortcuts(&rig, before)),
        [(5, ShortcutInfo::Empty(0)), (7, skill())]
    );
    // Dropping a skill from the panel is not a drag out of the bar: the lock does not stop it.
    let panel = rig.view.skill_node_center(SKILL).unwrap();
    let before = calls(&rig);
    rig.drag(panel, rig.view.shortcut_slot_center(2).unwrap());
    assert_eq!(set_shortcuts(&rig, before), [(2, skill())]);
}

#[test]
fn an_item_dropped_on_a_slot_becomes_an_item_shortcut() {
    let Some(mut rig) = with_skill_panel() else {
        return;
    };
    rig.panel(ToolbarChild::Inventory, true);
    rig.wait("the sword in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", SWORD)
            .is_some()
    });
    let before = calls(&rig);
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    let to = rig.view.shortcut_slot_center(3).unwrap();
    rig.press_and_move(from, to);
    assert!(
        rig.view.is_dragging(),
        "the sword's icon follows the pointer"
    );
    rig.release();
    // `ShortcutInfo::StreamPack`: type 0x40000002 and the item id.
    assert_eq!(
        set_shortcuts(&rig, before),
        [(3, ShortcutInfo::Instance(SWORD))]
    );
    assert_eq!(slot(&mut rig, 3), ShortcutInfo::Instance(SWORD));
}
