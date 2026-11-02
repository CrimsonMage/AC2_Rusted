//! The modern HUD's Edit layout mode through the client's real pointer path (the way the window
//! feeds the HUD: `UI_POINTER_X` / `UI_POINTER_Y` and the left button, then `Client::use_time` and
//! the HUD's frameworks every frame), against the Legions server double (`AC2_DAT_DIR`, a GPU
//! adapter): the option panel's button opens the mode, a drag on the shield moves a part, and the
//! bar's lock, reset, save as, load and Done buttons each take effect.
#![cfg(feature = "window")]

mod hud;

use ac2_ui::{Modifiers, ToolbarChild};
use hud::Rig;

fn shown(rig: &Rig, name: &str) -> bool {
    let tree = rig.view.ui_tree().unwrap();
    let id = ac2_ui::tokenize(name);
    tree.roots()
        .iter()
        .any(|&r| tree.get(r).is_some_and(|e| e.id() == id && e.visible()))
}

fn offset(rig: &Rig, part: &str) -> (i32, i32) {
    rig.view.ui_tree().unwrap().hud_offset(part)
}

fn click_named(rig: &mut Rig, name: &str) {
    let at = rig
        .view
        .ui_element_center(name)
        .unwrap_or_else(|| panic!("{name}"));
    rig.click(at);
}

/// The mode opened by the option panel's button.
fn open_mode(rig: &mut Rig) {
    rig.panel(ToolbarChild::Option, true);
    click_named(rig, "ModernEditLayoutButton");
    assert!(shown(rig, "ModernEditShield"), "the shield is not shown");
    assert!(shown(rig, "ModernEditBar"), "the bar is not shown");
}

#[test]
fn the_edit_layout_mode_can_be_used_through_the_pointer() {
    let dir = tempfile::tempdir().unwrap();
    let Some(mut rig) = Rig::new_in(Some(dir.path().to_path_buf())) else {
        return;
    };
    rig.wait("the HUD", |r| r.view.ui_tree().is_some());
    rig.idle(8);
    assert_eq!(rig.view.hud_style(), ac2_ui::HudStyle::Modern);
    let player = rig.client.session().player().unwrap();

    // A drag on the shield moves the vitals.
    open_mode(&mut rig);
    let (x0, y0, ..) = rig.view.ui_element_rect("StatusbarField").unwrap();
    rig.drag((x0 + 20, y0 + 8), (x0 + 70, y0 + 38));
    assert_eq!(
        offset(&rig, "vitals"),
        (50, 30),
        "the drag did not move the vitals"
    );
    let (nx, ny, ..) = rig.view.ui_element_rect("StatusbarField").unwrap();
    assert_eq!((nx - x0, ny - y0), (50, 30));

    // Save as: the name typed into the bar's field, then the button.
    click_named(&mut rig, "ModernEditPresetName");
    assert!(rig.view.text_mode(), "the name field takes the typing");
    let typed: Vec<(u16, u32)> = "Raid".encode_utf16().map(|c| (c, 0)).collect();
    rig.view.text_input(&[], &typed, Modifiers::default());
    rig.idle(2);
    click_named(&mut rig, "ModernEditSave");
    rig.idle(4);
    let saved = ac2_ui::hud_layout::read_hud_layout(dir.path(), player).unwrap();
    assert_eq!(saved.presets(), ["Raid"], "Save as saved nothing");

    // Reset puts the part back; Load brings the saved place.
    click_named(&mut rig, "ModernEditReset");
    assert_eq!(offset(&rig, "vitals"), (0, 0), "Reset did not reset");
    click_named(&mut rig, "ModernEditLoad");
    rig.idle(4);
    assert_eq!(offset(&rig, "vitals"), (50, 30), "Load did not load");

    // Done closes the mode.
    click_named(&mut rig, "ModernEditDone");
    assert!(!shown(&rig, "ModernEditShield") && !shown(&rig, "ModernEditBar"));
    assert!(!rig.view.ui_tree().unwrap().is_hud_locked());

    // Lock holds the HUD and closes the mode.
    open_mode(&mut rig);
    click_named(&mut rig, "ModernEditLock");
    assert!(
        rig.view.ui_tree().unwrap().is_hud_locked(),
        "Lock did not lock"
    );
    assert!(!shown(&rig, "ModernEditShield") && !shown(&rig, "ModernEditBar"));
}
