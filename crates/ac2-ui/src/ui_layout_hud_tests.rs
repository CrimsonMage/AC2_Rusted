//! `/saveui` and `/loadui` on the real gameplay HUD in each style (`AC2_DAT_DIR`): a save, a
//! change of the window places and (modern) the HUD layout, and a load that puts everything back
//! and keeps it; a file loaded in the other style; a bare 0930 file.

use std::path::PathBuf;
use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::ui_layout_text::UiLayoutFile;
use crate::{build_styled_hud, GameplayHud, Hud, HudStyle, UiTree};

/// The HUD's panels and tree in `style`, as the client holds them.
fn rig(style: HudStyle) -> Option<(Hud, UiTree)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    let GameplayHud {
        tree,
        vitals,
        chat,
        pets,
        modern_edit,
        ..
    } = build_styled_hud(&store, 1365, 768, style).unwrap();
    let mut hud = Hud::default();
    hud.style = style;
    hud.vitals = Some(vitals);
    hud.chat = chat;
    hud.pets = pets;
    hud.modern_edit = modern_edit;
    Some((hud, tree))
}

fn temp_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ac2-ui-layout-{tag}-{}", std::process::id()))
}

fn set(file: &mut UiLayoutFile, key: &str, value: &str) {
    match file.settings.iter_mut().find(|(k, _)| k == key) {
        Some(entry) => entry.1 = value.to_string(),
        None => file.settings.push((key.to_string(), value.to_string())),
    }
}

fn get<'a>(file: &'a UiLayoutFile, key: &str) -> Option<&'a str> {
    file.settings
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// Writes `file` as `name` in `dir` and loads it.
fn load(hud: &mut Hud, tree: &mut UiTree, dir: &std::path::Path, name: &str, file: &UiLayoutFile) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(crate::ui_layout_file(dir, name), file.to_text()).unwrap();
    hud.load_ui_layout(tree, dir, name).unwrap();
}

/// Everything the file would write, as the HUD stands.
fn state(hud: &Hud, tree: &UiTree) -> UiLayoutFile {
    hud.ui_layout_file(tree)
}

fn shifted(file: &UiLayoutFile) -> UiLayoutFile {
    let mut b = file.clone();
    for l in &mut b.nine {
        l.x += 13;
        l.y += 9;
    }
    for w in &mut b.windows {
        w.x += 11;
        w.y += 7;
    }
    b
}

fn round_trip(style: HudStyle) {
    let Some((mut hud, mut tree)) = rig(style) else {
        return;
    };
    let dir = temp_dir(&format!("{style:?}"));
    let modern = style == HudStyle::Modern;
    let start = state(&hud, &tree);
    assert_eq!(start.style, Some(style));
    assert!(!start.nine.is_empty(), "the HUD has some of the nine");
    assert!(!start.windows.is_empty(), "the HUD has other windows");
    assert_eq!(start.settings.is_empty(), !modern);

    // The player's layout: the settings chosen, then saved.
    let mut a = start.clone();
    if modern {
        set(&mut a, "offset.radar", "10,-20");
        set(&mut a, "chat.1.name", "Trade = gold");
        set(&mut a, "chat.1.lines", "200");
        set(&mut a, "chat.1.timestamps", "true");
        set(&mut a, "vitals.layout", "stacked");
        set(&mut a, "hud.locked", "true");
    }
    load(&mut hud, &mut tree, &dir, "a", &a);
    let saved = state(&hud, &tree);
    if modern {
        assert_eq!(get(&saved, "offset.radar"), Some("10,-20"));
        assert_eq!(get(&saved, "chat.1.name"), Some("Trade = gold"));
        assert_eq!(get(&saved, "chat.1.lines"), Some("200"));
        assert_eq!(get(&saved, "vitals.layout"), Some("stacked"));
        assert_eq!(get(&saved, "hud.locked"), Some("true"));
    }
    let path = hud.save_ui_layout(&tree, &dir, "saved").unwrap();
    assert_eq!(path, dir.join("saved.txt"));
    assert_eq!(
        UiLayoutFile::parse(&std::fs::read_to_string(&path).unwrap()),
        saved
    );

    // Changed by hand: windows moved, the settings changed.
    let mut b = shifted(&saved);
    if modern {
        set(&mut b, "offset.radar", "-30,40");
        set(&mut b, "chat.1.name", "Other");
        set(&mut b, "chat.1.lines", "500");
        set(&mut b, "chat.1.timestamps", "false");
        set(&mut b, "vitals.layout", "side");
        set(&mut b, "hud.locked", "false");
    }
    load(&mut hud, &mut tree, &dir, "b", &b);
    let changed = state(&hud, &tree);
    assert_ne!(changed.nine, saved.nine);
    assert_ne!(changed.windows, saved.windows);
    if modern {
        assert_eq!(get(&changed, "offset.radar"), Some("-30,40"));
        assert_eq!(get(&changed, "chat.1.name"), Some("Other"));
        assert_eq!(get(&changed, "vitals.layout"), Some("side"));
    }

    // Loaded back: everything is where it was and kept.
    tree.take_positions_changed();
    hud.layout_dirty = false;
    let load = hud.load_ui_layout(&mut tree, &dir, "saved").unwrap();
    assert_eq!(load.notice(), None);
    assert_eq!(state(&hud, &tree), saved);
    assert!(tree.take_positions_changed(), "the places are kept");
    assert_eq!(tree.saved_positions, tree.window_positions());
    assert_eq!(hud.layout_dirty, modern, "the settings are written");
    if modern {
        let kept: Vec<(String, String)> = hud
            .hud_layout
            .settings()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        assert_eq!(kept, saved.settings);
    }
    assert!(hud.load_ui_layout(&mut tree, &dir, "missing").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_modern_hud_saves_and_loads_its_whole_layout() {
    round_trip(HudStyle::Modern);
}

#[test]
fn the_classic_hud_saves_and_loads_its_whole_layout() {
    round_trip(HudStyle::Classic);
}

#[test]
fn a_modern_file_loaded_in_classic_applies_what_fits_and_says_so() {
    let (Some((mut modern, modern_tree)), Some((mut classic, mut classic_tree))) =
        (rig(HudStyle::Modern), rig(HudStyle::Classic))
    else {
        return;
    };
    let dir = temp_dir("cross");
    let mut file = state(&modern, &modern_tree);
    for l in &mut file.nine {
        l.x += 17;
        l.y += 5;
    }
    set(&mut file, "offset.radar", "10,-20");
    set(&mut file, "chat.1.name", "Modern only");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(crate::ui_layout_file(&dir, "m"), file.to_text()).unwrap();
    let classic_before = state(&classic, &classic_tree);
    classic_tree.take_positions_changed();
    let load = classic
        .load_ui_layout(&mut classic_tree, &dir, "m")
        .unwrap();
    assert_eq!(load.skipped_style, Some(HudStyle::Modern));
    let notice = load.notice().unwrap();
    assert!(
        notice.contains("modern") && notice.contains("m.txt"),
        "{notice}"
    );
    // The nine windows classic has are where the file puts them.
    let now = state(&classic, &classic_tree);
    assert_eq!(now.nine.len(), classic_before.nine.len());
    for l in &now.nine {
        let wanted = file.nine.iter().find(|w| w.tag == l.tag).unwrap();
        assert_eq!(l, wanted);
    }
    assert!(classic_tree.take_positions_changed());
    // The modern-only values are not taken.
    assert!(classic.hud_layout.settings().is_empty());
    assert!(!classic.layout_dirty);
    assert!(now.settings.is_empty());
    // A classic file loaded in modern: its windows that fit, the layout untouched.
    let classic_file = state(&classic, &classic_tree);
    std::fs::write(crate::ui_layout_file(&dir, "c"), classic_file.to_text()).unwrap();
    modern.hud_layout.set("offset.radar", "1,2");
    let load = modern
        .load_ui_layout(&mut { modern_tree }, &dir, "c")
        .unwrap();
    assert_eq!(load.skipped_style, Some(HudStyle::Classic));
    assert!(load.notice().unwrap().contains("classic"));
    assert_eq!(modern.hud_layout.get("offset.radar"), Some("1,2"));
    assert!(!modern.layout_dirty);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_bare_zero_nine_thirty_file_loads_silently_in_either_style() {
    for style in [HudStyle::Modern, HudStyle::Classic] {
        let Some((mut hud, mut tree)) = rig(style) else {
            return;
        };
        let dir = temp_dir(&format!("bare{style:?}"));
        std::fs::create_dir_all(&dir).unwrap();
        let before = state(&hud, &tree);
        let bare = UiLayoutFile {
            nine: before
                .nine
                .iter()
                .map(|l| crate::UiLayoutLine { x: l.x + 3, ..*l })
                .collect(),
            ..UiLayoutFile::default()
        };
        std::fs::write(crate::ui_layout_file(&dir, "bare"), bare.to_text()).unwrap();
        let load = hud.load_ui_layout(&mut tree, &dir, "bare").unwrap();
        assert_eq!(load.notice(), None);
        let after = state(&hud, &tree);
        assert_eq!(after.nine, bare.nine);
        assert_eq!(after.windows, before.windows);
        assert_eq!(after.settings, before.settings);
        assert!(!hud.layout_dirty);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
