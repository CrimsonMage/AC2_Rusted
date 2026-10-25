//! The controls screen's maps: the meta keys, the save's partition and the presets.

use std::collections::{BTreeMap, HashMap};

use crate::maps::{ActionMap, ActionMapping, KeyMap};
use crate::pregame::keyboard_list::{
    KeyBinding, KeyboardList, META_CONTROL, META_NONE, META_SHIFT,
};
use crate::pregame::keyboard_maps::{
    map_kind, meta_key_of, saved_set, MapKind, Preset, LEFT_CONTROL, LEFT_SHIFT, RIGHT_CONTROL,
    RIGHT_SHIFT,
};

#[test]
fn meta_modes_are_the_meta_keys_semantics() {
    assert_eq!(meta_key_of(0), META_NONE);
    assert_eq!(meta_key_of(1), META_SHIFT);
    assert_eq!(meta_key_of(4), META_CONTROL);
    assert_eq!(META_SHIFT, LEFT_SHIFT);
    assert_eq!(META_CONTROL, LEFT_CONTROL);
}

#[test]
fn either_side_of_a_meta_key_files_its_binding_under_that_map() {
    assert_eq!(map_kind(META_NONE), Some(MapKind::Main));
    assert_eq!(map_kind(LEFT_SHIFT), Some(MapKind::Shift));
    assert_eq!(map_kind(RIGHT_SHIFT), Some(MapKind::Shift));
    assert_eq!(map_kind(LEFT_CONTROL), Some(MapKind::Control));
    assert_eq!(map_kind(RIGHT_CONTROL), Some(MapKind::Control));
    assert_eq!(map_kind(0x8100_0439), None);
}

#[test]
fn the_presets_use_three_consecutive_map_enums_each() {
    assert_eq!(
        Preset::Default.maps(),
        [0x4100_0001, 0x4100_0002, 0x4100_0003]
    );
    assert_eq!(Preset::Eq.maps(), [0x4100_0004, 0x4100_0005, 0x4100_0006]);
    assert_eq!(Preset::Daoc.maps(), [0x4100_0007, 0x4100_0008, 0x4100_0009]);
}

/// Save files each binding by its meta key, and names the three maps as retail does.
#[test]
fn save_partitions_the_bindings_by_meta_key() {
    const FIRST: u32 = 0x4000_0100;
    const SECOND: u32 = 0x4000_0101;
    let actions = ActionMap {
        string_table: 1,
        descriptions: HashMap::from([(FIRST, (1, 2)), (SECOND, (3, 4))]),
        actions: BTreeMap::new(),
        toggles: HashMap::new(),
    };
    let base = KeyMap {
        name: "base".to_owned(),
        guid: [0; 16],
        mappings: vec![
            ActionMapping {
                action: FIRST,
                semantic: 0x8100_0411,
                name: String::new(),
            },
            ActionMapping {
                action: SECOND,
                semantic: 0x8100_0414,
                name: String::new(),
            },
        ],
    };
    let mut list = KeyboardList::from_maps(&actions, &[(&base, META_NONE)]);
    // FIRST's second slot (index 2) gets a shift binding; SECOND's second slot (index 5) a
    // control binding.
    list.click(2);
    list.start_set(KeyBinding {
        semantic: 0x8100_0412,
        meta: LEFT_SHIFT,
    });
    list.click(5);
    list.start_set(KeyBinding {
        semantic: 0x8100_0413,
        meta: LEFT_CONTROL,
    });
    let set = saved_set(&list);
    assert_eq!(set.main.name, "mydefault");
    assert_eq!(set.shift.name, "mydefaults");
    assert_eq!(set.control.name, "mydefaultc");
    let semantics = |map: &KeyMap| map.mappings.iter().map(|m| m.semantic).collect::<Vec<_>>();
    assert_eq!(semantics(&set.main), [0x8100_0411, 0x8100_0414]);
    assert_eq!(semantics(&set.shift), [0x8100_0412]);
    assert_eq!(semantics(&set.control), [0x8100_0413]);
}

/// `KeyHitHandler`'s presses: each keyboard press of a non-meta key, with the meta mode it went
/// down in (the shift key itself is not a press).
#[test]
fn the_input_manager_reports_each_key_press_with_its_meta_mode() {
    use crate::input::{InputEvent, InputManager, KeyPress};
    let mut input = InputManager::new();
    let mut out = Vec::new();
    input.process(
        InputEvent::Key {
            scan: 0x2A,
            pressed: true,
        },
        0,
        &mut out,
    );
    input.process(
        InputEvent::Key {
            scan: 0x39,
            pressed: true,
        },
        1,
        &mut out,
    );
    assert_eq!(
        input.take_key_presses(),
        vec![KeyPress {
            semantic: 0x8100_0439,
            meta_mode: 1,
        }]
    );
    assert!(input.take_key_presses().is_empty());
}

/// `SaveKeyboardList`'s reload: the edited maps replace the three the input manager pushed.
#[test]
fn replacing_the_key_maps_changes_what_the_input_manager_resolves() {
    use crate::controls::Controls;
    use crate::input::InputManager;
    use crate::interpreter::ActionInterpreter;
    use crate::maps::ActionMap;
    use crate::pregame::KeyMapSet;
    const ACTION: u32 = 0x4000_0100;
    const APPS: u32 = 0x8100_04DD;
    let mut controls = Controls::new(
        InputManager::new(),
        ActionInterpreter::new(ActionMap::default()),
    );
    assert_eq!(controls.input.action_from_key(APPS, 0), None);
    let main = KeyMap {
        name: "mydefault".to_owned(),
        guid: [0; 16],
        mappings: vec![ActionMapping {
            action: ACTION,
            semantic: APPS,
            name: String::new(),
        }],
    };
    let empty = |name: &str| KeyMap {
        name: name.to_owned(),
        guid: [0; 16],
        mappings: Vec::new(),
    };
    let set = KeyMapSet {
        main,
        shift: empty("mydefaults"),
        control: empty("mydefaultc"),
    };
    controls.replace_key_maps(&set);
    assert_eq!(controls.input.action_from_key(APPS, 0), Some(ACTION));
}
