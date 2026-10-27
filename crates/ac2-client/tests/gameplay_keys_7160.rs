//! The gameplay HUD's keys (`gmGamePlayUI::ListenToMessage`, `0xffff0003`): the child keys
//! `0x41000008` (toolbar), `0x41000009` (status bar) and `0x4100000B` (radar) toggle their child
//! through `gmGamePlayUI::ChildKeyPress`, and only while the gameplay UI is shown; the key
//! `0x4100000C` shows or hides the whole HUD (its status bar, radar and toolbar bar), and does
//! nothing while a logout is requested; `0x4100000D` asks for `UseSelection`. Driven through the HUD rig over the Legions server double
//! (`AC2_DAT_DIR`, a GPU adapter).
#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_ui::GameplayChild;
use hud::Rig;

const TOOLBAR_KEY: u32 = 0x4100_0008;
const STATUSBAR_KEY: u32 = 0x4100_0009;
const RADAR_KEY: u32 = 0x4100_000B;
const HUD_KEY: u32 = 0x4100_000C;
const USE_KEY: u32 = 0x4100_000D;

/// A key goes down and comes up: the press is the action.
fn press(rig: &mut Rig, key: u32) {
    rig.frame(&[(key, 1.0, true)]);
    rig.frame(&[(key, 1.0, false)]);
}

#[test]
fn each_child_key_toggles_its_child() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    for (key, child) in [
        (STATUSBAR_KEY, GameplayChild::Statusbar),
        (RADAR_KEY, GameplayChild::Radar),
        (TOOLBAR_KEY, GameplayChild::Toolbar),
    ] {
        let before = rig.view.child_visible(child);
        press(&mut rig, key);
        assert_eq!(rig.view.child_visible(child), !before, "key {key:#x}");
        press(&mut rig, key);
        assert_eq!(rig.view.child_visible(child), before, "key {key:#x} again");
    }
}

#[test]
fn the_hud_key_hides_and_shows_the_children_and_gates_the_child_keys() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    assert!(rig.view.gameplay_shown());
    assert!(rig.view.child_visible(GameplayChild::Statusbar));
    // The HUD hides: its status bar goes with it.
    press(&mut rig, HUD_KEY);
    assert!(!rig.view.gameplay_shown());
    assert!(!rig.view.child_visible(GameplayChild::Statusbar));
    // While hidden the child keys do nothing (`ListenToMessage` checks `Shown()`): the status
    // bar would otherwise show.
    press(&mut rig, STATUSBAR_KEY);
    assert!(!rig.view.child_visible(GameplayChild::Statusbar));
    // The HUD shows again, with its status bar.
    press(&mut rig, HUD_KEY);
    assert!(rig.view.gameplay_shown());
    assert!(rig.view.child_visible(GameplayChild::Statusbar));
}

#[test]
fn the_use_key_asks_for_use_selection_once_per_press() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    rig.asked.clear();
    // `gmGamePlayUI::ListenToMessage` case 0x4100000D: `gmCWInterface::UseSelection`, whether or
    // not the HUD is shown.
    press(&mut rig, USE_KEY);
    assert_eq!(
        rig.asked
            .iter()
            .filter(|c| **c == Command::UseSelection)
            .count(),
        1,
        "{:?}",
        rig.asked
    );
    rig.asked.clear();
    rig.frame(&[]);
    assert!(
        !rig.asked.contains(&Command::UseSelection),
        "{:?}",
        rig.asked
    );
}
