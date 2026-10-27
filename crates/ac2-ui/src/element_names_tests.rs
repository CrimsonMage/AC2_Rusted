//! The element names the panels look up exist in the real layouts (`AC2_DAT_DIR`).
//!
//! A panel finds its elements by `Tokenize` of a name: `child_recursive(root, tokenize("..."))`,
//! `id("...")`, or a name built with `format!`. A wrong name never matches a press or a child, and
//! a test that builds its messages from the same wrong name still passes. So this reads the names
//! the production sources spell out and checks each one against every layout of the local DAT, and
//! the format-built families against the same layouts.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ac2_db::ResourceStore;

use super::*;

/// Element-looking names the code keeps that the DVD's layouts do not carry. Each is a retail name;
/// the reason says what the code does with it. The test checks each one stays absent, so a layout
/// that gains the element is noticed.
const ABSENT_ELEMENTS: &[(&str, &str)] = &[
    (
        "SecureTradeCancelButton",
        "gmSecureTradeUI's second close button (admin 7148 registers it beside SecureTradeCloseButton); the DVD layouts have no such element",
    ),
    (
        "ZoomButton",
        "gmAvatarModificationUI's zoom toggle (admin 7148 ListenToMessage); the DVD layouts have no such element",
    ),
    (
        "ChatButtonToggle1",
        "chat window 1's options toggle (gmChatUI, the m_toggleButton of ChatWindow); the DVD layouts have no such element for window 1",
    ),
];

/// Lookups whose name ends in one of these (and starts with a capital, as layout elements do) are
/// element names even when the call is not one of the lookup functions.
const ELEMENT_SUFFIXES: &[&str] = &[
    "Button", "Field", "Label", "Slider", "Panel", "Close", "List", "ListBox", "Viewport", "Icon",
    "Text", "Meter", "Bar", "Tree", "Page", "Frame", "Template", "Row", "Entry", "Window", "Help",
    "Item", "Scroll", "Tab", "Header", "Name", "Level", "Spinner", "Portrait", "Picture", "Menu",
    "Toggle", "Image", "Pad", "Overlay", "Stack", "Options",
];

/// The calls whose argument is an element's name (`tree.child_recursive(root, tokenize("..."))`).
const LOOKUP_CALLS: &[&str] = &[
    "child_recursive(",
    "create_root(",
    "create_from(",
    "create_nested_element(",
    "GetChildRecursive(",
    "find_named(",
    "named_scrollbar(",
];

/// Toolbar children (`ToolbarChild::name`): each has a field, a button and a close button.
const TOOLBAR_CHILDREN: &[&str] = &[
    "Examination",
    "Navigation",
    "Option",
    "Inventory",
    "Quest",
    "Skill",
    "Social",
];
const TOOLBAR_PARTS: &[&str] = &["Field", "Button", "CloseButton"];
/// The avatar modification's physique sliders (`PHYS_TYPES`), each with its parts.
const AVATAR_SLIDERS: &[&str] = &[
    "Height",
    "Build",
    "SkinTone",
    "SkinDetail",
    "FaceDetail",
    "HeadDetail",
    "HeadFrill",
    "FrillColor",
    "Special",
    "ShirtClothingColor",
    "PantsClothingColor",
    "BootsClothingColor",
];
const AVATAR_SLIDER_PARTS: &[&str] = &["Field", "Label", "Slider", "RightButton", "LeftButton"];

/// Every element id in every local layout, nested children included.
fn layout_element_ids(dataset: Arc<ac2_dat::DatDataset>) -> BTreeSet<u32> {
    fn walk(element: &ElementDesc, ids: &mut BTreeSet<u32>) {
        ids.insert(element.id);
        for state in &element.states {
            for child in &state.children {
                walk(child, ids);
            }
        }
    }
    let store = ResourceStore::new(Arc::clone(&dataset));
    let mut ids = BTreeSet::new();
    let layouts: Vec<u32> = dataset
        .ids_in_range(ac2_dat::DatFileKind::Local, LAYOUT_FIRST, LAYOUT_LAST)
        .collect();
    assert!(!layouts.is_empty(), "the local DAT has UI layouts");
    for did in layouts {
        let layout = load_layout(&store, ac2_core::DataId::new(did))
            .unwrap_or_else(|e| panic!("{did:08x}: {e}"))
            .unwrap_or_else(|| panic!("{did:08x} missing"));
        for element in &layout.elements {
            walk(element, &mut ids);
        }
    }
    ids
}

/// The production sources: `ac2-ui/src` and `ac2-client/src`, minus the test modules.
fn production_sources() -> Vec<PathBuf> {
    fn visit(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if !name.contains("tests") {
                    visit(&path, out);
                }
            } else if name.ends_with(".rs") && !name.ends_with("tests.rs") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    for root in [
        concat!(env!("CARGO_MANIFEST_DIR"), "/src"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../ac2-client/src"),
    ] {
        visit(Path::new(root), &mut files);
    }
    files.sort();
    files
}

/// The string literal that starts at `text[start..]` (`text[start]` is the opening quote), if it is
/// a plain identifier.
fn literal_at(text: &str, start: usize) -> Option<&str> {
    let rest = &text[start + 1..];
    let end = rest.find('"')?;
    let name = &rest[..end];
    (!name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .then_some(name)
}

/// The argument text of the call whose `(` is at `open`, up to its matching `)`.
fn call_arguments(text: &str, open: usize) -> &str {
    let mut depth = 0_usize;
    for (offset, byte) in text.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open + 1..open + offset];
                }
            }
            _ => {}
        }
    }
    &text[open + 1..]
}

/// The element names one source spells out: the literals of `tokenize("..")`, `id("..")` and
/// `name("..")` that carry an element suffix, and every `tokenize("..")` inside a lookup call.
/// `ID_*` string keys are the string tables' and stay out of it.
fn lookup_names(text: &str, out: &mut BTreeSet<String>) {
    for call in ["tokenize(\"", "id(\"", "name(\""] {
        for (start, _) in text.match_indices(call) {
            // `string_id(` and `ID_` are not element lookups; `call` must start an identifier.
            let before = text[..start].chars().next_back();
            if before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
                continue;
            }
            let quote = start + call.len() - 1;
            let Some(name) = literal_at(text, quote) else {
                continue;
            };
            // `ID_*` names are string-table keys, not elements.
            let capital =
                name.starts_with(|c: char| c.is_ascii_uppercase()) && !name.starts_with("ID_");
            if capital && ELEMENT_SUFFIXES.iter().any(|s| name.ends_with(s)) {
                out.insert(name.to_owned());
            }
        }
    }
    for call in LOOKUP_CALLS {
        for (start, _) in text.match_indices(call) {
            let open = start + call.len() - 1;
            let args = call_arguments(text, open);
            for (arg, _) in args.match_indices("tokenize(\"") {
                if let Some(name) = literal_at(args, arg + "tokenize(".len()) {
                    out.insert(name.to_owned());
                }
            }
        }
    }
}

/// Every element name the production code looks up, from its sources.
fn scanned_lookup_names() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for path in production_sources() {
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        lookup_names(&text, &mut names);
    }
    names
}

#[test]
fn every_element_name_the_panels_look_up_is_in_a_layout() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids = layout_element_ids(dataset);
    let names = scanned_lookup_names();
    // The scan must reach the panels it is for; an empty scan would pass for nothing.
    for known in [
        "NewConsignmentButton",
        "CollectConsignmentButton",
        "ConsignmentCloseButton",
        "NewSaleCloseButton",
        "SecureTradeCloseButton",
        "ToolbarField",
    ] {
        assert!(names.contains(known), "the scan finds {known}");
    }
    assert!(names.len() > 100, "{} names scanned", names.len());
    let absent: BTreeSet<&str> = ABSENT_ELEMENTS.iter().map(|(name, _)| *name).collect();
    let mut missing = Vec::new();
    for name in &names {
        let found = ids.contains(&tokenize(name));
        if absent.contains(name.as_str()) {
            assert!(
                !found,
                "{name} is in a layout now; drop it from ABSENT_ELEMENTS"
            );
        } else if !found {
            missing.push(name.as_str());
        }
    }
    assert!(
        missing.is_empty(),
        "looked up but in no layout: {missing:?}"
    );
    eprintln!("{} element names looked up, all in a layout", names.len());
}

#[test]
fn the_format_built_element_names_are_in_a_layout() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids = layout_element_ids(dataset);
    let mut missing = Vec::new();
    let families = [
        (TOOLBAR_CHILDREN, TOOLBAR_PARTS),
        (AVATAR_SLIDERS, AVATAR_SLIDER_PARTS),
    ];
    for (bases, parts) in families {
        for base in bases {
            for part in parts {
                let name = format!("{base}{part}");
                if !ids.contains(&tokenize(&name)) {
                    missing.push(name);
                }
            }
        }
    }
    // The chat windows: `ChatWindow{n}` and `ChatButtonToggle{n}` (`chat_window.rs`).
    for n in 1..=4 {
        let window = format!("ChatWindow{n}");
        assert!(ids.contains(&tokenize(&window)), "{window}");
        let toggle = format!("ChatButtonToggle{n}");
        if n == 1 {
            assert!(!ids.contains(&tokenize(&toggle)), "{toggle} is absent");
        } else if !ids.contains(&tokenize(&toggle)) {
            missing.push(toggle);
        }
    }
    assert!(
        missing.is_empty(),
        "format-built names in no layout: {missing:?}"
    );
}
