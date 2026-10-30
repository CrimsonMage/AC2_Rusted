//! The fellows' map notes over the Legions server double (`AC2_DAT_DIR`): a created fellowship's
//! other member gets a type 4 note at its cell on the map (`FellowshipHelper::AddFellow`'s
//! `SendNotice_AddMarker`), as the player's options show fellows on the map, and leaving the
//! fellowship clears it (`RemoveAllFellows`). Needs a GPU adapter.
#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_ui::fellow_markers::FELLOW_MARKER;
use hud::Rig;

#[test]
fn a_fellowship_member_is_a_note_on_the_map() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.server.borrow_mut().send_init_player();
    rig.idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    rig.command(Command::Social(
        ac2_weenie::SocialRequest::FellowshipCreate {
            name: "Wanderers".into(),
            social: 0,
            looting: 0,
        },
    ));
    rig.idle(32);
    let player = rig.client.frame().player.as_ref().map(|p| p.id).unwrap();
    let notes: Vec<(u64, u32)> = rig
        .client
        .frame()
        .markers
        .iter()
        .filter(|m| m.kind == FELLOW_MARKER)
        .map(|m| (m.id, m.cell))
        .collect();
    assert!(!notes.is_empty(), "the other member has a note");
    assert!(notes.iter().all(|&(id, _)| id != player), "{notes:?}");

    rig.command(Command::Social(ac2_weenie::SocialRequest::FellowshipLeave));
    rig.idle(32);
    assert!(
        rig.client
            .frame()
            .markers
            .iter()
            .all(|m| m.kind != FELLOW_MARKER),
        "leaving clears the notes"
    );
}
