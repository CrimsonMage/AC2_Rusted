//! The shortcut bar's look (`shortcut_look.rs`) on the real toolbar layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::mouse::{UI_POINTER_X, UI_POINTER_Y};
use crate::shortcut_look::{KEY_LABEL, SLOT, SLOT_HOVER, SLOT_WELL};
use crate::token::{tokenize, STATE_NORMAL_ROLLOVER};
use crate::{
    build_styled_hud, GameplayHud, HudStyle, Shortcut, ShortcutOptions, ShortcutSource, UiTree,
    SHORTCUT_SKILL,
};

/// A player whose first slot holds a skill and whose numbers are `numbers`.
struct Player {
    numbers: bool,
}

impl ShortcutSource for Player {
    fn shortcuts(&self) -> Vec<Shortcut> {
        let mut all = vec![Shortcut::default(); 100];
        all[0] = Shortcut {
            kind: SHORTCUT_SKILL,
            skill: 7,
            ..Shortcut::default()
        };
        all
    }
    fn icon(&self, _: usize, shortcut: &Shortcut) -> u32 {
        if shortcut.kind == SHORTCUT_SKILL {
            0x4100_A7A6
        } else {
            0
        }
    }
    fn options(&self) -> ShortcutOptions {
        ShortcutOptions {
            which_set: 1,
            show_numbers: self.numbers,
            rows: 1,
            ..ShortcutOptions::default()
        }
    }
    fn skill_trained(&self, _: u32) -> bool {
        true
    }
    fn has_item(&self, _: u64) -> bool {
        true
    }
}

fn bar_hud(numbers: bool) -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_styled_hud(&store, 1024, 768, HudStyle::Modern).unwrap();
    let bar = hud.shortcuts.as_mut()?;
    bar.reload(&mut hud.tree, &Player { numbers });
    Some(hud)
}

fn slot(hud: &GameplayHud, index: usize) -> usize {
    let (_, list) = hud.shortcuts.as_ref().unwrap().elements();
    hud.tree.list_items(list.unwrap())[index]
}

fn text(tree: &UiTree, slot: usize, name: &str) -> Option<String> {
    let element = tree.child_recursive(slot, tokenize(name))?;
    tree.get(element)?.text().filter(|t| !t.is_empty())
}

#[test]
fn the_slots_are_framed_wells_with_their_icon_at_its_native_size() {
    let Some(hud) = bar_hud(false) else {
        return;
    };
    let tree = &hud.tree;
    let (field, list) = hud.shortcuts.as_ref().unwrap().elements();
    let first = slot(&hud, 0);
    assert_eq!(tree.screen_box(first).width(), SLOT);
    assert_eq!(
        tree.screen_box(slot(&hud, 1)).x0 - tree.screen_box(first).x0,
        SLOT
    );
    let well = tree
        .child_recursive(first, tokenize("ShortcutSlotWell"))
        .unwrap();
    assert_eq!(
        tree.get(well).unwrap().image().map(|i| i.file),
        Some(SLOT_WELL)
    );
    let icon = tree
        .child_recursive(first, tokenize("ShortcutIcon"))
        .unwrap();
    let icon_box = tree.screen_box(icon);
    assert_eq!((icon_box.width(), icon_box.height()), (32, 32));
    // Ten slots fill the bar's row, inside the bar.
    let bar = tree.screen_box(field.unwrap());
    let row = tree.screen_box(list.unwrap());
    assert_eq!(row.width(), 10 * SLOT);
    assert!(row.x0 > bar.x0 && row.x1 < bar.x1 && row.y1 < bar.y1);
    assert_eq!((bar.x0 + bar.x1) / 2, 511, "centred on the bottom edge");
}

#[test]
fn each_slot_shows_its_key_unless_the_number_display_does() {
    let Some(hud) = bar_hud(false) else {
        return;
    };
    assert_eq!(
        text(&hud.tree, slot(&hud, 0), KEY_LABEL).as_deref(),
        Some("1")
    );
    assert_eq!(
        text(&hud.tree, slot(&hud, 9), KEY_LABEL).as_deref(),
        Some("0")
    );
    assert_eq!(text(&hud.tree, slot(&hud, 3), "NumberDisplay"), None);
    // With numbers on, an empty slot's number display shows the digit (retail) and the corner
    // label steps aside; a filled slot keeps its label.
    let Some(hud) = bar_hud(true) else {
        return;
    };
    assert_eq!(
        text(&hud.tree, slot(&hud, 3), "NumberDisplay").as_deref(),
        Some("4")
    );
    assert_eq!(text(&hud.tree, slot(&hud, 3), KEY_LABEL), None);
    assert_eq!(
        text(&hud.tree, slot(&hud, 0), KEY_LABEL).as_deref(),
        Some("1")
    );
}

#[test]
fn a_hovered_slot_takes_the_selection_corners() {
    let Some(mut hud) = bar_hud(false) else {
        return;
    };
    let first = slot(&hud, 0);
    let at = hud.tree.screen_box(first);
    hud.tree
        .start_input(UI_POINTER_X, (at.x0 + SLOT / 2) as f32);
    hud.tree
        .start_input(UI_POINTER_Y, (at.y0 + SLOT / 2) as f32);
    let button = hud
        .tree
        .child_recursive(first, tokenize("ShortcutButton"))
        .unwrap();
    assert_eq!(hud.tree.get(button).unwrap().state(), STATE_NORMAL_ROLLOVER);
    let hover = hud
        .tree
        .children(button)
        .iter()
        .copied()
        .find(|&c| hud.tree.get(c).unwrap().image().map(|i| i.file) == Some(SLOT_HOVER));
    assert!(hover.is_some(), "the hover frame shows");
}
