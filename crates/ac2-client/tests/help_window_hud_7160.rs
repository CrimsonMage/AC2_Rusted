//! The help window through the HUD rig (`KeyStone::OpenHelp`): a panel's help button opens its
//! page in the window with text, the key stone key (`0x400000BB`, `gmToolbarUI::ShowKeyStone`)
//! opens the default page and closes it again, and the window's close button hides it. Driven
//! over the Legions server double. Needs `AC2_DAT_DIR`, `AC2_CONTENT_DIR` (the DVD's
//! `ProgramFiles/Content`, which holds the KSML pages) and a GPU adapter.
#![cfg(feature = "window")]

mod hud;

use ac2_ui::{ToolbarChild, UiBox};
use hud::Rig;

/// `gmToolbarUI::ListenToMessage`'s key stone key.
const KEYSTONE_KEY: u32 = 0x4000_00BB;

/// A key goes down and comes up: the press is the action.
fn press(rig: &mut Rig, key: u32) {
    rig.frame(&[(key, 1.0, true)]);
    rig.frame(&[(key, 1.0, false)]);
}

fn centre(b: UiBox) -> (i32, i32) {
    ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
}

/// The pages come from the content directory the HUD worker reads at build time.
fn content_configured() -> bool {
    std::env::var_os("AC2_CONTENT_DIR").is_some()
}

#[test]
fn a_panel_help_button_opens_its_page_in_the_window() {
    if !content_configured() {
        return;
    }
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    assert!(rig.view.help_window_text().is_none());
    rig.panel(ToolbarChild::Skill, true);
    let button = rig
        .view
        .element_box("SkillHelpButton")
        .expect("the skill panel has its help button");
    rig.click(centre(button));
    let text = rig
        .view
        .help_window_text()
        .expect("the help button shows the window");
    assert!(!text.trim().is_empty(), "the skill page has text");
    assert!(!text.contains("not installed"), "{text}");
}

#[test]
fn the_key_stone_key_toggles_the_default_page() {
    if !content_configured() {
        return;
    }
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    press(&mut rig, KEYSTONE_KEY);
    let text = rig
        .view
        .help_window_text()
        .expect("the key stone key opens the window");
    assert!(!text.trim().is_empty());
    press(&mut rig, KEYSTONE_KEY);
    assert!(rig.view.help_window_text().is_none(), "the key closes it");
}

#[test]
fn the_window_close_button_hides_the_window() {
    if !content_configured() {
        return;
    }
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.wait("the HUD", |r| r.view.shortcut_slots() > 0);
    press(&mut rig, KEYSTONE_KEY);
    assert!(rig.view.help_window_text().is_some());
    let close = rig
        .view
        .help_element_box("BookCloseButton")
        .expect("the help window has its close button");
    rig.click(centre(close));
    assert!(
        rig.view.help_window_text().is_none(),
        "the close button hides it"
    );
}
