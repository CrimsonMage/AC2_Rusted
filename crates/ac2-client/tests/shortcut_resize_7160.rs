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

/// The bar's one-row height, then its height and rows after its top edge is dragged 60 window
/// pixels up, in `style`; the uploaded heights checked.
fn drag_up(style: ac2_ui::HudStyle) -> Option<(i32, i32, u32)> {
    let mut rig = Rig::new()?;
    rig.view
        .set_hud_style(&rig.gpu, ac2_gpu::COLOR_FORMAT, style);
    {
        let mut server = rig.server.borrow_mut();
        server.give_starter_items();
        server.send_init_player();
    }
    rig.idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    let (top, one_row) = rig.view.shortcut_bar_top().expect("the bar's field");
    let before = rig.server.borrow().script_calls.len();
    rig.drag(top, (top.0, top.1 - 60));
    rig.idle(16);
    let (_, height) = rig.view.shortcut_bar_top().unwrap();
    let rows = rig.client.shortcut_height().expect("the height");
    assert_eq!(
        uploaded_heights(&rig, before).last(),
        Some(&rows),
        "the profile uploaded"
    );
    assert!(rig.client.take_script_problems().is_empty());
    Some((one_row, height, rows))
}

#[test]
fn dragging_the_bars_top_edge_up_snaps_it_to_rows_and_uploads_the_height() {
    // 60 window pixels up at the 1024 x 768 rig's scale: 104 tall, snapped to three rows.
    if let Some(dragged) = drag_up(ac2_ui::HudStyle::Classic) {
        assert_eq!(dragged, (44, 108, 3));
    }
}

/// The modern strip's rows (32-pixel wells 3 apart, under the grip's 4-pixel band) snap the same
/// way and upload the same height.
#[test]
fn the_modern_strip_snaps_to_its_rows_and_uploads_the_height() {
    if let Some(dragged) = drag_up(ac2_ui::HudStyle::Modern) {
        // 39 + 60 = 99 tall: (99 - 4) / 35 + 0.5 rounds to three rows of 35 under the band.
        assert_eq!(dragged, (39, 109, 3));
    }
}
