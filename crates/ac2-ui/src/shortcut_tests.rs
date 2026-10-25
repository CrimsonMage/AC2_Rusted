//! The shortcut bar against the real layout (`AC2_DAT_DIR`), over a stub player.

use std::sync::Arc;

use ac2_db::ResourceStore;

use super::*;

const SKILL: u32 = 0x55;

struct Stub;

impl ShortcutSource for Stub {
    fn shortcuts(&self) -> Vec<Shortcut> {
        let mut all = vec![Shortcut::default(); 100];
        all[0] = Shortcut {
            kind: SHORTCUT_SKILL,
            skill: SKILL,
            ..Shortcut::default()
        };
        all[1] = Shortcut {
            kind: SHORTCUT_ALIAS,
            text: "/wave".into(),
            ..Shortcut::default()
        };
        // An item the player no longer has.
        all[2] = Shortcut {
            kind: SHORTCUT_ITEM,
            item: 0x77,
            ..Shortcut::default()
        };
        all
    }
    fn icon(&self, _: usize, shortcut: &Shortcut) -> u32 {
        if shortcut.kind == SHORTCUT_SKILL {
            0x0600_1234
        } else {
            0
        }
    }
    fn options(&self) -> ShortcutOptions {
        ShortcutOptions {
            which_set: 1,
            show_numbers: true,
            insta_scroll: true,
            rows: 1,
        }
    }
    fn skill_trained(&self, skill: u32) -> bool {
        skill == SKILL
    }
    fn has_item(&self, _: u64) -> bool {
        false
    }
}

fn key(tree: &mut UiTree, bar: &mut ShortcutUi, action: u32) {
    tree.start_input(action, 1.0);
    tree.stop_input(action);
    // Until the bar's own broadcasts (`new_view`) are answered.
    for _ in 0..4 {
        for message in tree.take_messages() {
            bar.message(tree, &Stub, &message);
        }
    }
}

#[test]
fn the_bar_shows_its_shortcuts_and_uses_the_set_shown() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut tree = hud.tree;
    let mut bar = hud
        .shortcuts
        .expect("the toolbar layout has the shortcut field");
    bar.reload(&mut tree, &Stub);
    let (_, list) = bar.elements();
    let list = list.expect("shortcut list box");
    let items = tree.list_items(list).to_vec();
    assert_eq!(items.len(), 100, "padded to the bar's slots");
    // The item that is gone was removed (and the player asked to forget it).
    let requests = bar.take_requests();
    assert!(
        requests.contains(&ShortcutRequest::Set(2, Shortcut::default())),
        "{requests:?}"
    );
    let items = tree.list_items(list).to_vec();
    let icon = tree
        .child_recursive(items[0], tokenize("ShortcutIcon"))
        .unwrap();
    assert_eq!(
        tree.get(icon).unwrap().image().map(|i| i.file),
        Some(0x0600_1234)
    );
    let alias = tree
        .child_recursive(items[1], tokenize("AliasText"))
        .unwrap();
    assert_eq!(tree.get(alias).unwrap().text().as_deref(), Some("/wave"));
    // Empty slots show their key's number.
    let number = tree
        .child_recursive(items[4], tokenize("NumberDisplay"))
        .unwrap();
    assert_eq!(tree.get(number).unwrap().text().as_deref(), Some("5"));
    // The first shortcut key uses the first slot of the set shown.
    key(&mut tree, &mut bar, 0x4100_002B);
    assert_eq!(bar.take_requests(), vec![ShortcutRequest::Use(0)]);
    // The second set's key shows it; its keys then use slots 10..19.
    key(&mut tree, &mut bar, 0x4100_0036);
    assert_eq!(bar.offset(), 10);
    assert!(bar.take_requests().contains(&ShortcutRequest::WhichSet(2)));
    key(&mut tree, &mut bar, 0x4100_002D);
    assert_eq!(bar.take_requests(), vec![ShortcutRequest::Use(12)]);
}
