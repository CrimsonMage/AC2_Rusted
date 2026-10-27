//! The `.keymap` text: the `PFileParser` tree, `CKeyMap::ToFileNode`'s records and their read
//! back by `CKeyMap::FromFileNode`'s rules, and the files `Controls::load_with` reads.

use std::collections::HashMap;

use crate::keymap_file::{
    action_table, format_key_map, format_tree, key_map_from_tree, key_map_tree, parse_key_map,
    parse_tree, read_key_map, read_key_map_upgraded, save_set, update_version, version_of,
    FileNode, KeymapFiles, MAIN_KEY_UPDATES,
};
use crate::maps::{ActionMapping, KeyMap};
use crate::pregame::KeyMapSet;

const JUMP: u32 = 0x4000_0100;
const FORWARD: u32 = 0x4000_0101;
const W: u32 = 0x8100_0411;
const SPACE: u32 = 0x8100_0439;

fn sample() -> KeyMap {
    KeyMap {
        name: "mydefault".to_owned(),
        guid: [1, 0, 0, 0, 2, 0, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11],
        mappings: vec![
            ActionMapping {
                action: FORWARD,
                semantic: W,
                name: "Forward".to_owned(),
            },
            ActionMapping {
                action: JUMP,
                semantic: SPACE,
                name: "Jump".to_owned(),
            },
        ],
    }
}

fn actions() -> HashMap<String, u32> {
    HashMap::from([("Forward".to_owned(), FORWARD), ("Jump".to_owned(), JUMP)])
}

#[test]
fn the_tree_reads_names_brackets_and_comments() {
    let root = parse_tree("# a comment\nVersionHeader [ 1 ] \"\" [ \"my map\" ]\nleaf\n").unwrap();
    assert_eq!(root.children.len(), 3);
    assert_eq!(root.children[0].name, "VersionHeader");
    assert_eq!(root.children[0].children[0].name, "1");
    assert_eq!(root.children[1].name, "");
    assert_eq!(root.children[1].children[0].name, "my map");
    assert!(root.children[2].children.is_empty());
}

#[test]
fn an_unclosed_bracket_or_quote_is_an_error() {
    assert!(parse_tree("name [ a b").is_err());
    assert!(parse_tree("name ] ").is_err());
    assert!(parse_tree("\"open").is_err());
}

#[test]
fn names_with_delimiters_are_quoted_and_read_back() {
    let tree = FileNode {
        name: String::new(),
        children: vec![FileNode {
            name: "a b".to_owned(),
            children: vec![FileNode {
                name: String::new(),
                children: vec![FileNode {
                    name: "x#(y)".to_owned(),
                    children: Vec::new(),
                }],
            }],
        }],
    };
    let text = format_tree(&tree);
    assert!(text.contains("\"a b\""), "{text}");
    assert!(text.contains("\"\""), "{text}");
    assert!(text.contains("\"x#(y)\""), "{text}");
    assert_eq!(parse_tree(&text).unwrap(), tree);
}

#[test]
fn a_key_map_is_saved_in_the_to_file_node_layout() {
    let text = format_key_map(&sample());
    let root = parse_tree(&text).unwrap();
    assert_eq!(root.children[0].name, "VersionHeader");
    assert_eq!(root.children[0].children[0].name, "1");
    assert_eq!(root.children[1].children[0].name, "mydefault");
    // Data1 (little endian bytes 1,0,0,0), Data2, Data3, then eight Data4 bytes.
    let guid: Vec<&str> = root.children[2]
        .children
        .iter()
        .map(|n| n.name.as_str())
        .collect();
    assert_eq!(
        guid,
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"]
    );
    assert_eq!(root.children[3].children[0].name, "Forward");
    assert_eq!(root.children[3].children[1].name, "W");
}

#[test]
fn a_saved_key_map_reads_back_equal() {
    let map = sample();
    let text = format_key_map(&map);
    assert_eq!(parse_key_map(&text, &actions()).unwrap(), map);
}

#[test]
fn a_binding_with_an_unknown_action_or_key_is_dropped() {
    let text = "VersionHeader [ 1 ]\n\"\" [ m ]\n\"\" [ 1 2 3 0 0 0 0 0 0 0 0 ]\n\
                \"\" [ Forward W ]\n\"\" [ Gone W ]\n\"\" [ Jump Nonsense ]\n\"\" [ Jump \"Key 0x81000439\" ]\n";
    let map = parse_key_map(text, &actions()).unwrap();
    let bound: Vec<(u32, u32)> = map
        .mappings
        .iter()
        .map(|m| (m.action, m.semantic))
        .collect();
    assert_eq!(bound, [(FORWARD, W), (JUMP, SPACE)]);
}

#[test]
fn a_malformed_record_fails_the_file() {
    // A binding with three parts, a GUID short of 11 parts, a bad version.
    let three = "VersionHeader [ 1 ]\n\"\" [ m ]\n\"\" [ 1 2 3 0 0 0 0 0 0 0 0 ]\n\"\" [ a b c ]\n";
    assert!(parse_key_map(three, &actions()).is_err());
    let short = "\"\" [ m ]\n\"\" [ 1 2 3 ]\n";
    assert!(parse_key_map(short, &actions()).is_err());
    let version = "VersionHeader [ x ]\n\"\" [ m ]\n\"\" [ 1 2 3 0 0 0 0 0 0 0 0 ]\n";
    assert!(parse_key_map(version, &actions()).is_err());
}

#[test]
fn an_action_no_map_names_is_saved_by_its_enum_value_and_read_back() {
    let unnamed = KeyMap {
        name: "mydefault".to_owned(),
        guid: [0; 16],
        mappings: vec![ActionMapping {
            action: 0x4000_0017,
            semantic: W,
            name: "0x40000017".to_owned(),
        }],
    };
    let text = format_key_map(&unnamed);
    assert!(text.contains("0x40000017"), "{text}");
    assert_eq!(parse_key_map(&text, &actions()).unwrap(), unnamed);
}

#[test]
fn the_action_table_names_each_action_once() {
    let table = action_table([&sample()]);
    assert_eq!(table.get("Forward"), Some(&FORWARD));
    assert_eq!(table.get("Jump"), Some(&JUMP));
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ac2-keymap-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_saved_set_is_written_and_read_from_its_three_files() {
    let dir = temp_dir("set");
    let files = KeymapFiles::in_dir(&dir);
    let set = KeyMapSet {
        main: sample(),
        shift: KeyMap {
            name: "mydefaults".to_owned(),
            guid: [0; 16],
            mappings: Vec::new(),
        },
        control: KeyMap {
            name: "mydefaultc".to_owned(),
            guid: [0; 16],
            mappings: Vec::new(),
        },
    };
    save_set(&files, &set).unwrap();
    let main = read_key_map(&files.main, &actions())
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(main, set.main);
    let shift = read_key_map(&files.shift, &actions())
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(shift.name, "mydefaults");
    // A missing file is not an error: the DAT's map is used.
    let missing = KeymapFiles::in_dir(&dir.join("none"));
    assert!(read_key_map(&missing.main, &actions()).unwrap().is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_user_files_are_the_three_retail_names() {
    let files = KeymapFiles::user();
    for (path, name) in [
        (&files.main, "Main.keymap"),
        (&files.shift, "Shift.keymap"),
        (&files.control, "Control.keymap"),
    ] {
        assert!(path.ends_with(name), "{path:?}");
    }
}

/// The DATs' maps read back through the file format: each saved binding returns as it was.
#[test]
fn the_dat_key_maps_round_trip_through_the_file() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let maps: Vec<KeyMap> = [
        crate::maps::KEYMAP_MAIN,
        crate::maps::KEYMAP_SHIFT,
        crate::maps::KEYMAP_CONTROL,
    ]
    .into_iter()
    .map(|enum_value| {
        (*crate::maps::load_key_map(&store, enum_value)
            .unwrap()
            .unwrap())
        .clone()
    })
    .collect();
    let table = action_table(&maps);
    for map in &maps {
        let text = format_key_map(map);
        let back = parse_key_map(&text, &table).unwrap();
        assert_eq!(back.name, map.name);
        assert_eq!(back.guid, map.guid);
        // Every binding has a name in the DATs and so reads back.
        assert_eq!(back.mappings, map.mappings, "{}", map.name);
    }
}

/// A rebinding in the player's file takes effect at load: `Controls::load_with` pushes it over the
/// DAT's main map.
#[test]
fn a_saved_file_changes_the_controls_it_loads() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let dir = temp_dir("controls");
    let files = KeymapFiles::in_dir(&dir);
    let main = (*crate::maps::load_key_map(&store, crate::maps::KEYMAP_MAIN)
        .unwrap()
        .unwrap())
    .clone();
    // Bind the jump action to the B key (a free key in the default map) as well.
    let mut edited = main.clone();
    let jump = edited
        .mappings
        .iter()
        .map(|m| m.action)
        .next()
        .expect("a bound action");
    let name = action_table(std::slice::from_ref(&main))
        .into_iter()
        .find(|(_, action)| *action == jump)
        .map(|(name, _)| name)
        .unwrap_or_default();
    edited.mappings.push(ActionMapping {
        action: jump,
        semantic: crate::pregame::key_names::semantic_from_name("B").unwrap(),
        name,
    });
    std::fs::write(&files.main, format_key_map(&edited)).unwrap();
    let controls = crate::Controls::load_with(&store, Some(&files))
        .unwrap()
        .unwrap();
    assert_eq!(
        controls.input.action_from_key(
            crate::pregame::key_names::semantic_from_name("B").unwrap(),
            0
        ),
        Some(jump)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// `REPLY_KEY`'s action (the chat's reply key) and the R key the Main table binds it to.
const REPLY: u32 = 0x4100_0049;
const R: u32 = 0x8100_0413;

/// The actions the DATs hold, with `REPLY_KEY` among them.
fn actions_with_reply() -> HashMap<String, u32> {
    let mut table = actions();
    table.insert("REPLY_KEY".to_owned(), REPLY);
    table
}

/// A map's text as a version 0 file writes it: the `VersionHeader` record left out.
fn without_header(map: &KeyMap) -> String {
    let mut tree = key_map_tree(map);
    assert_eq!(tree.children[0].name, "VersionHeader");
    tree.children.remove(0);
    format_tree(&tree)
}

fn reply_binding() -> ActionMapping {
    ActionMapping {
        action: REPLY,
        semantic: R,
        name: "REPLY_KEY".to_owned(),
    }
}

#[test]
fn the_version_header_is_read_and_a_headerless_file_is_version_zero() {
    let current = parse_tree(&format_key_map(&sample())).unwrap();
    assert_eq!(version_of(&current), Ok(1));
    let old = parse_tree(&without_header(&sample())).unwrap();
    assert_eq!(version_of(&old), Ok(0));
}

/// `CKeyMap::UpdateVersion`: a version 0 file gains the R key's `REPLY_KEY` binding; the upgrade
/// does not run a second time.
#[test]
fn a_file_older_than_the_table_gains_its_binding() {
    let root = parse_tree(&without_header(&sample())).unwrap();
    let mut map = key_map_from_tree(&root, &actions_with_reply()).unwrap();
    assert!(update_version(
        &mut map,
        0,
        MAIN_KEY_UPDATES,
        &actions_with_reply()
    ));
    assert!(map.mappings.contains(&reply_binding()));
    assert!(!update_version(
        &mut map,
        0,
        MAIN_KEY_UPDATES,
        &actions_with_reply()
    ));
}

#[test]
fn a_current_file_is_not_upgraded() {
    let mut map = sample();
    let before = map.clone();
    assert!(!update_version(
        &mut map,
        1,
        MAIN_KEY_UPDATES,
        &actions_with_reply()
    ));
    assert_eq!(map, before);
}

/// The file's own binding of the R key stays: the table adds only where the key is unbound.
#[test]
fn a_key_the_file_binds_is_kept() {
    let mut map = sample();
    map.mappings.push(ActionMapping {
        action: JUMP,
        semantic: R,
        name: "Jump".to_owned(),
    });
    let before = map.clone();
    assert!(!update_version(
        &mut map,
        0,
        MAIN_KEY_UPDATES,
        &actions_with_reply()
    ));
    assert_eq!(map, before);
}

/// A row whose action the DATs do not name is skipped, and so reports no change.
#[test]
fn a_row_with_an_unknown_action_is_skipped() {
    let mut map = sample();
    let before = map.clone();
    assert!(!update_version(&mut map, 0, MAIN_KEY_UPDATES, &actions()));
    assert_eq!(map, before);
}

/// `CInputManager_WIN32::PushKeyMap`: an upgraded file is read upgraded and written back at the
/// current version, and a file already at it is left as it is.
#[test]
fn an_upgraded_file_is_rewritten_at_the_current_version() {
    let dir = temp_dir("upgrade");
    let files = KeymapFiles::in_dir(&dir);
    std::fs::write(&files.main, without_header(&sample())).unwrap();
    let loaded = read_key_map_upgraded(&files.main, &actions_with_reply(), MAIN_KEY_UPDATES)
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(loaded.mappings.contains(&reply_binding()));
    let written = std::fs::read_to_string(&files.main).unwrap();
    assert_eq!(version_of(&parse_tree(&written).unwrap()), Ok(1));
    let again = read_key_map_upgraded(&files.main, &actions_with_reply(), MAIN_KEY_UPDATES)
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(again, loaded);
    assert_eq!(std::fs::read_to_string(&files.main).unwrap(), written);
    assert!(read_key_map_upgraded(&files.shift, &actions(), &[])
        .unwrap()
        .is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

/// The Main map of the DATs, with its R bindings removed and the file written without a header:
/// `Controls::load_with` loads it upgraded, so R takes `REPLY_KEY`.
#[test]
fn an_old_main_file_loads_with_the_reply_key_on_r() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let mut main = (*crate::maps::load_key_map(&store, crate::maps::KEYMAP_MAIN)
        .unwrap()
        .unwrap())
    .clone();
    main.mappings.retain(|m| m.semantic != R);
    let dir = temp_dir("old-main");
    let files = KeymapFiles::in_dir(&dir);
    std::fs::write(&files.main, without_header(&main)).unwrap();
    let controls = crate::Controls::load_with(&store, Some(&files))
        .unwrap()
        .unwrap();
    assert_eq!(controls.input.action_from_key(R, 0), Some(REPLY));
    let _ = std::fs::remove_dir_all(&dir);
}
