//! The classic HUD driven by a player's mouse and keyboard at a 2560 x 1440 window: winit-equivalent
//! events (cursor moves, buttons, keys) enter `ac2_ui::Controls` as the window program feeds them,
//! and every frame is the window's world frame (`ac2_client::world_input`), over the Legions
//! server double (`AC2_DAT_DIR`, a GPU adapter). Attacks (the toolbar's attack buttons and the attack
//! key), skills (a shortcut slot and its number key), the right button's camera control and
//! dragging windows by their headers.

#![cfg(feature = "window")]

mod hud;

use ac2_client::Command;
use ac2_net::protocol::SpecialAttackRequest;
use ac2_ui::{tokenize, HudStyle, ToolbarChild, UiBox, UiScaleSetting, UiTree};
use hud::Rig;
use legions_server_double::{CREATURE_ONE, MELEE_AUTO_ATTACK};
use winit::keyboard::KeyCode;

const BIG: (u32, u32) = (2560, 1440);
/// `WM_Combat` 0x16 / 0x14: the attack request and its stop.
const ATTACK: u32 = 0x8204_0008;
const STOP_ATTACKING: u32 = 0x8204_0010;

fn dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("ac2-real-input-{}-{name}", std::process::id()))
}

/// A rig on a 2560 x 1440 window in `style` at `setting`, the bar built from the starter items.
fn rig(style: HudStyle, setting: UiScaleSetting, name: &str) -> Option<Rig> {
    let mut rig = Rig::new_in(Some(dir(name)))?;
    if style == HudStyle::Classic {
        rig.view
            .set_hud_style(&rig.gpu, ac2_gpu::COLOR_FORMAT, style);
    }
    rig.wait("the HUD", |r| r.view.hud_tree().is_some());
    rig.view.set_ui_scale(setting);
    rig.win_resize(BIG);
    {
        let mut server = rig.server.borrow_mut();
        server.give_starter_items();
        server.send_init_player();
        server.spawn_radar_objects();
    }
    rig.win_idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    rig.win_idle(4);
    Some(rig)
}

/// The creature selected through the gameplay command (the selection is not what is under test).
fn select_creature(rig: &mut Rig) {
    rig.command(Command::Select(Some(CREATURE_ONE)));
    rig.win_idle(8);
}

fn attack_calls(rig: &Rig, function: u32) -> usize {
    rig.server
        .borrow()
        .script_bytes
        .iter()
        .filter(|(f, _)| *f == function)
        .count()
}

fn clean(name: &str) {
    let _ = std::fs::remove_dir_all(dir(name));
}

#[test]
fn the_attack_button_attacks_the_selection() {
    let Some(mut rig) = rig(HudStyle::Classic, UiScaleSetting::Retail, "attack") else {
        return;
    };
    select_creature(&mut rig);
    let at = rig
        .view
        .ui_element_center("StartAttackButton")
        .expect("the attack button");
    eprintln!("attack button at {at:?}");
    rig.win_click(at);
    rig.win_idle(8);
    assert_eq!(attack_calls(&rig, ATTACK), 1, "the server saw one attack");
    clean("attack");
}

/// The root of the window holding `name`.
fn window_root(rig: &Rig, name: &str) -> usize {
    let tree: &UiTree = rig.view.ui_tree().unwrap();
    let id = tokenize(name);
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().visible() && tree.child_recursive(r, id).is_some())
        .unwrap_or_else(|| panic!("{name} is shown"))
}

/// Alt + left drag of the window holding `name` by `by`, pressed where the window is topmost.
fn alt_drag_window(rig: &mut Rig, name: &str, by: (i32, i32)) -> (UiBox, UiBox) {
    let root = window_root(rig, name);
    let before = rig.view.ui_tree().unwrap().screen_box(root);
    let mut from = None;
    'search: for y in (before.y0 + 4..before.y1 - 4).step_by(6) {
        for x in (before.x0 + 4..before.x1 - 4).step_by(6) {
            rig.win_point((x, y));
            let t = rig.view.ui_tree().unwrap();
            if t.mouse_top().is_some_and(|top| t.root_of(top) == root) {
                from = Some((x, y));
                break 'search;
            }
        }
    }
    let from = from.expect("a point of the window is on top");
    rig.win_alt(true);
    rig.win_drag(from, (from.0 + by.0, from.1 + by.1));
    rig.win_alt(false);
    (before, rig.view.ui_tree().unwrap().screen_box(root))
}

#[test]
fn the_examination_panel_moves_by_alt_drag() {
    let Some(mut rig) = rig(HudStyle::Classic, UiScaleSetting::Retail, "exam") else {
        return;
    };
    rig.view.open_examination();
    rig.win_idle(8);
    let (before, after) = alt_drag_window(&mut rig, "ExaminationField", (120, -60));
    assert_eq!((after.x0 - before.x0, after.y0 - before.y0), (120, -60));
    clean("exam");
}

#[test]
fn a_dragbar_window_moves_by_its_header() {
    let Some(mut rig) = rig(HudStyle::Classic, UiScaleSetting::Retail, "bar") else {
        return;
    };
    // The radar (RadarField, root 0x6433c3c7) has drag bars.
    let tree = rig.view.ui_tree().unwrap();
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == 0x6433_c3c7)
        .expect("the radar window");
    let mut bars = vec![];
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        if tree.get(i).unwrap().kind() == tokenize("dragbar") {
            bars.push(i);
        }
        stack.extend(tree.children(i).iter().copied());
    }
    let bar = *bars.first().expect("a drag bar");
    let before = tree.screen_box(root);
    let at = rig.view.ui_index_center(bar).unwrap();
    rig.win_drag(at, (at.0 - 100, at.1 + 50));
    let after = rig.view.ui_tree().unwrap().screen_box(root);
    assert_eq!((after.x0 - before.x0, after.y0 - before.y0), (-100, 50));
    clean("bar");
}

fn special_attacks(rig: &Rig) -> Vec<SpecialAttackRequest> {
    rig.server
        .borrow()
        .script_calls
        .iter()
        .filter_map(SpecialAttackRequest::from_call)
        .collect()
}

#[test]
fn a_shortcut_slot_and_its_number_key_use_the_skill() {
    let Some(mut rig) = rig(HudStyle::Classic, UiScaleSetting::Retail, "skill") else {
        return;
    };
    select_creature(&mut rig);
    let want = SpecialAttackRequest {
        attack: 1,
        maneuver: MELEE_AUTO_ATTACK,
        target: CREATURE_ONE,
    };
    let at = rig.view.shortcut_slot_center(0).expect("slot 0");
    eprintln!("slot 0 at {at:?}");
    rig.win_click(at);
    rig.win_idle(8);
    assert_eq!(special_attacks(&rig), vec![want], "the slot's click");
    // Move off the bar first, then the number key.
    rig.win_point((1280, 600));
    rig.win_tap(KeyCode::Digit1);
    rig.win_idle(8);
    assert_eq!(special_attacks(&rig).len(), 2, "the number key");
    clean("skill");
}

#[test]
fn the_middle_button_toggles_mouse_look() {
    let Some(mut rig) = rig(HudStyle::Classic, UiScaleSetting::Retail, "camera") else {
        return;
    };
    rig.win_point((1280, 500));
    let rotation = |rig: &Rig| rig.client.frame().camera.expect("a camera").rotation;
    let start = rotation(&rig);
    // `CameraToggleMouseLook` (the main key map binds the middle button): on at the press.
    rig.win_press(2, true);
    rig.win_press(2, false);
    rig.win_idle(2);
    assert!(rig.client.camera().mouse_look, "mouse look is on");
    assert!(rig.win.grabbed, "the window grabs the pointer");
    for _ in 0..8 {
        rig.win_motion(25, 0);
        rig.win_frame();
    }
    assert_ne!(start, rotation(&rig), "the camera turned");
    rig.win_press(2, true);
    rig.win_press(2, false);
    rig.win_idle(2);
    assert!(!rig.client.camera().mouse_look, "the second press ends it");
    clean("camera");
}

#[test]
fn the_backquote_key_toggles_attacking() {
    let Some(mut rig) = rig(HudStyle::Classic, UiScaleSetting::Retail, "tick") else {
        return;
    };
    select_creature(&mut rig);
    rig.win_point((1280, 500));
    // `AUTOATTACK_KEY` (0x41000013): `gmShortcutUI::ListenToMessage` asks the attack state change.
    rig.win_tap(KeyCode::Backquote);
    rig.win_idle(8);
    assert_eq!(attack_calls(&rig, ATTACK), 1, "the first press attacks");
    // The bar learns the attack state from the server's answer (`UpdateAttackState`).
    rig.win_idle(64);
    assert!(
        rig.win.seen.iter().any(|e| matches!(
            e,
            ac2_weenie::SessionEvent::Notice(ac2_weenie::GameplayNotice::UpdateAttackState(true))
        )),
        "the server answered the attack with its state"
    );
    // The bar shows combat: the start button is ghosted, the stop button is not.
    let ghosted = |rig: &Rig, name: &str| {
        let tree = rig.view.ui_tree().unwrap();
        let id = tokenize(name);
        tree.roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id))
            .map(|i| {
                tree.get(i).unwrap().attribute(tokenize("ghosted"))
                    == Some(&ac2_ui::AttributeValue::Bool(true))
            })
    };
    assert_eq!(
        ghosted(&rig, "StartAttackButton"),
        Some(true),
        "start ghosted"
    );
    assert_eq!(ghosted(&rig, "StopAttackButton"), Some(false), "stop ready");
    rig.win_tap(KeyCode::Backquote);
    rig.win_idle(8);
    assert_eq!(attack_calls(&rig, STOP_ATTACKING), 1, "the second stops");
    clean("tick");
}

#[test]
fn the_attack_button_works_at_2x_and_in_the_modern_style() {
    for (style, setting, name) in [
        (HudStyle::Classic, UiScaleSetting::Percent(200), "a2x"),
        (HudStyle::Modern, UiScaleSetting::Retail, "amod"),
    ] {
        let Some(mut rig) = rig(style, setting, name) else {
            return;
        };
        select_creature(&mut rig);
        let at = rig.view.ui_element_center("StartAttackButton");
        eprintln!("{name}: attack button {at:?}");
        if let Some(at) = at {
            rig.win_click(at);
        } else {
            rig.win_point((1280, 500));
            rig.win_tap(KeyCode::Backquote);
        }
        rig.win_idle(8);
        assert_eq!(attack_calls(&rig, ATTACK), 1, "{name}");
        clean(name);
    }
}

/// Drags the window holding `name` by its header band (no Alt) by `by`.
fn header_drag_window(rig: &mut Rig, name: &str, by: (i32, i32)) -> (UiBox, UiBox) {
    let root = window_root(rig, name);
    let before = rig.view.ui_tree().unwrap().screen_box(root);
    let y = before.y0 + 6;
    let mut from = None;
    for x in (before.x0 + 4..before.x1 - 4).step_by(3) {
        rig.win_point((x, y));
        let t = rig.view.ui_tree().unwrap();
        if t.mouse_top().is_some_and(|top| t.root_of(top) == root) {
            from = Some((x, y));
            break;
        }
    }
    let from = from.expect("a header point of the window is on top");
    rig.win_drag(from, (from.0 + by.0, from.1 + by.1));
    (before, rig.view.ui_tree().unwrap().screen_box(root))
}

#[test]
fn the_examination_and_inventory_windows_move_by_their_header() {
    let Some(mut rig) = rig(HudStyle::Classic, UiScaleSetting::Retail, "header") else {
        return;
    };
    rig.view.open_examination();
    rig.view.show_panel(ToolbarChild::Inventory, true);
    rig.win_idle(8);
    for (name, by) in [
        ("ExaminationField", (120, -60)),
        ("PackMainContainer", (-90, -80)),
    ] {
        let (before, after) = header_drag_window(&mut rig, name, by);
        assert_eq!(
            (after.x0 - before.x0, after.y0 - before.y0),
            by,
            "{name} follows its header"
        );
    }
    clean("header");
}

#[test]
fn holding_the_right_button_over_empty_world_turns_the_camera() {
    let Some(mut rig) = rig(HudStyle::Classic, UiScaleSetting::Retail, "rlook") else {
        return;
    };
    let rotation = |rig: &Rig| rig.client.frame().camera.expect("a camera").rotation;
    // A point of the world nothing is drawn on: no UI element and no object under it.
    let frame = rig.client.frame();
    let empty = (0..1440)
        .rev()
        .step_by(40)
        .flat_map(|y| (700..1900).step_by(40).map(move |x| (x, y)))
        .find(|&(x, y)| rig.view.pick_list(&frame, BIG, (x, y)).is_empty() && { true })
        .expect("an empty point");
    rig.win_point(empty);
    let t = rig.view.ui_tree().unwrap();
    assert_eq!(
        t.mouse_top().and_then(|i| t.get(i)).map(|e| e.id()),
        Some(tokenize("ItemSelectionField")),
        "the pointer is over the world element"
    );
    let start = rotation(&rig);
    rig.win_press(1, true);
    rig.win_idle(2);
    assert!(
        rig.client.camera().mouse_look,
        "the held right button looks"
    );
    assert!(rig.win.grabbed);
    for _ in 0..8 {
        rig.win_motion(25, 0);
        rig.win_frame();
    }
    assert_ne!(start, rotation(&rig), "the camera turned");
    rig.win_press(1, false);
    rig.win_idle(2);
    assert!(!rig.client.camera().mouse_look, "the release ends the look");
    clean("rlook");
}
