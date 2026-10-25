//! The input manager and action interpreter on synthetic maps.

use super::*;

const W: u32 = 0x11;
const SHIFT: u32 = 0x2A;
const R: u32 = 0x13;
const FORWARD: u32 = 0x4000_0020;
const RUN_LOCK: u32 = 0x4000_0027;
const SHIFT_ACTION: u32 = 0x4000_000E;
const ZOOM_IN: u32 = 0x4000_002D;
const WAVE: u32 = 0x4100_0001;

fn mapping(action: u32, scan: u32) -> ActionMapping {
    ActionMapping {
        action,
        semantic: 0x8100_0400 | scan,
        name: Vec::new(),
    }
}

fn controls() -> Controls {
    let mut input = InputManager::new();
    let preproc = KeyMap {
        name: Vec::new(),
        guid: [0; 16],
        mappings: vec![
            mapping(SHIFT_ACTION, SHIFT),
            ActionMapping {
                action: ZOOM_IN,
                semantic: 0xFF00_4704,
                name: Vec::new(),
            },
        ],
    };
    let main = KeyMap {
        name: Vec::new(),
        guid: [0; 16],
        mappings: vec![mapping(FORWARD, W), mapping(RUN_LOCK, R)],
    };
    let shift = KeyMap {
        name: Vec::new(),
        guid: [0; 16],
        mappings: vec![mapping(WAVE, W)],
    };
    input.push_key_map(&preproc, 0);
    input.push_key_map(&main, 0);
    input.push_key_map(&shift, 0x8100_042A);
    let mut map = ActionMap::default();
    let value = |class, enum_value| ActionValue {
        class,
        enum_value,
        extent: 1.0,
    };
    map.toggles.insert(FORWARD, TOGGLE_HOLD);
    map.toggles.insert(RUN_LOCK, TOGGLE_TOGGLE);
    map.toggles.insert(SHIFT_ACTION, TOGGLE_HOLD);
    map.toggles.insert(ZOOM_IN, TOGGLE_ONE_SHOT);
    map.toggles.insert(WAVE, TOGGLE_ONE_SHOT);
    map.actions
        .insert((FORWARD, 0), value(CLASS_MOVEMENT, 0x4000_0001));
    map.actions
        .insert((RUN_LOCK, 0), value(CLASS_MOVEMENT, 0x4000_0008));
    map.actions
        .insert((SHIFT_ACTION, 0), value(CLASS_MODIFIER, 1));
    map.actions
        .insert((ZOOM_IN, 0), value(CLASS_CAMERA, ZOOM_IN));
    map.actions
        .insert((WAVE, 0), value(CLASS_BEHAVIOR, 0x8000_0001));
    Controls::new(input, ActionInterpreter::new(map))
}

fn key(controls: &mut Controls, scan: u32, pressed: bool, now: f64) {
    controls.event(
        InputEvent::Key { scan, pressed },
        (now * 1000.0) as u32,
        now,
    );
}

#[test]
fn a_held_key_moves_until_released() {
    let mut c = controls();
    key(&mut c, W, true, 0.0);
    let frame = c.frame();
    assert!(frame.interrupt_move_to);
    assert_eq!(frame.actions.len(), 1);
    assert_eq!(frame.actions[0].enum_value, 0x4000_0001);
    key(&mut c, W, false, 0.1);
    assert!(c.frame().actions.is_empty());
}

#[test]
fn shift_selects_the_shift_map_and_the_release_resolves_like_the_press() {
    let mut c = controls();
    key(&mut c, SHIFT, true, 0.0);
    // Shift held: W is the shift map's one-shot behaviour, not forward.
    key(&mut c, W, true, 0.5);
    // Shift released first: W's release still resolves through the shift binding, so forward
    // never starts; the one-shot wave lasts the frame.
    key(&mut c, SHIFT, false, 0.6);
    key(&mut c, W, false, 0.7);
    let frame = c.frame();
    assert_eq!(frame.behaviors, vec![(0x8000_0001, true)]);
    assert_eq!(frame.actions.len(), 1);
    assert_eq!(frame.actions[0].class, CLASS_BEHAVIOR);
    assert!(c.frame().actions.is_empty());
}

#[test]
fn run_lock_toggles_and_forward_turns_it_off() {
    let mut c = controls();
    key(&mut c, R, true, 0.0);
    key(&mut c, R, false, 0.1);
    assert_eq!(
        c.frame().actions.len(),
        1,
        "run lock stays on after its key goes up"
    );
    key(&mut c, W, true, 1.0);
    let held: Vec<_> = c.interpreter.current().iter().map(|a| a.action).collect();
    assert_eq!(held, vec![FORWARD]);
    key(&mut c, W, false, 1.1);
    key(&mut c, R, true, 2.0);
    key(&mut c, R, true, 3.0);
    assert!(
        c.frame().actions.is_empty(),
        "a second press toggles it off"
    );
}

#[test]
fn the_wheel_zooms_for_one_frame_and_taps_double() {
    let mut c = controls();
    c.event(InputEvent::Wheel { delta: 240 }, 0, 0.0);
    assert_eq!(c.interpreter.current()[0].extent, 2.0);
    let frame = c.frame();
    assert_eq!(frame.camera_actions, vec![(ZOOM_IN, 1.0, true)]);
    assert_eq!(frame.actions.len(), 1);
    assert!(
        c.frame().actions.is_empty(),
        "one-shot actions last one frame"
    );
    key(&mut c, W, true, 5.0);
    key(&mut c, W, false, 5.05);
    key(&mut c, W, true, 5.1);
    assert!(c.interpreter.is_double_tap());
}

#[test]
fn tokenize_matches_the_retail_token_values() {
    // `Tokenize("reference")`, `("stateless")`, `("referent")` as the layouts store them.
    assert_eq!(crate::tokenize("reference"), 0xBCE0_5594);
    assert_eq!(crate::tokenize("stateless"), 0x5CE1_FCAA);
    assert_eq!(crate::tokenize("referent"), 0xBCE0_425E);
    assert_eq!(crate::tokenize(""), 0);
}
