//! The panels the HUD's build makes (the mini fellow, the pets and the string tables) are still
//! there once the HUD is built, and again after a rebuild (a relog). Driven over the Legions
//! server double. Needs `AC2_DAT_DIR` and a GPU adapter.
#![cfg(feature = "window")]

mod hud;

use hud::Rig;

const BUILT: [&str; 3] = ["mini_fellow", "pets", "strings"];

#[test]
fn built_panels_survive_the_hud_build() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    rig.idle(4);
    let made = rig.view.built_panels();
    for name in BUILT {
        assert!(
            made.contains(&name),
            "{name} dropped after the build: {made:?}"
        );
    }
}

#[test]
fn built_panels_survive_a_hud_rebuild() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.rebuild_hud();
    rig.idle(4);
    let made = rig.view.built_panels();
    for name in BUILT {
        assert!(
            made.contains(&name),
            "{name} dropped after the rebuild: {made:?}"
        );
    }
}
