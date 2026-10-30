//! Resizing the shortcut bar by its top grip, through the HUD with the pointer, against the
//! Legions server double (`AC2_DAT_DIR`, a GPU adapter). `gmShortcutUI`'s global loop snaps the
//! field to whole rows and sends `CM_Character::SendNotice_SetShortcutHeight`, which
//! `gmCharacterUI::RecvNotice_SetShortcutHeight` turns into `gmCWInterface::SetShortcutHeight`
//! on the player's script and `UploadGameOptions` (the profile to the server).
#![cfg(feature = "window")]

mod hud;

use ac2_net::protocol::PackValue;
use ac2_script::host::interface_calls::InterfaceCalls;
use hud::Rig;

/// `gmCWInterface::UploadGameOptions`'s `WM_Player` call: the option bits, the radar mask, the
/// shortcut height and the damage text range.
const UPLOAD_GAME_OPTIONS: u32 = 0x8212_000C;

/// The shortcut heights the player uploaded from the `since`th call on.
fn uploaded_heights(rig: &Rig, since: usize) -> Vec<u32> {
    rig.server.borrow().script_calls[since..]
        .iter()
        .filter(|call| call.function == UPLOAD_GAME_OPTIONS)
        .map(|call| match call.arguments[..] {
            [_, _, PackValue::Int(height), _] => height,
            _ => panic!("{call:?}"),
        })
        .collect()
}

#[test]
fn dragging_the_bars_top_edge_up_snaps_it_to_rows_and_uploads_the_height() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    {
        let mut server = rig.server.borrow_mut();
        server.give_starter_items();
        server.send_init_player();
    }
    rig.idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    let (top, height) = rig.view.shortcut_bar_top().expect("the bar's field");
    assert_eq!(height, 44, "one row");
    let before = rig.server.borrow().script_calls.len();
    // 60 window pixels up at the 1024 x 768 rig's scale: 104 tall, snapped to three rows.
    rig.drag(top, (top.0, top.1 - 60));
    rig.idle(16);
    let (_, height) = rig.view.shortcut_bar_top().unwrap();
    assert_eq!(height, 108);
    assert_eq!(rig.client.shortcut_height(), Ok(3));
    assert_eq!(
        uploaded_heights(&rig, before).last(),
        Some(&3),
        "the profile uploaded"
    );
    assert!(rig.client.take_script_problems().is_empty());
}
