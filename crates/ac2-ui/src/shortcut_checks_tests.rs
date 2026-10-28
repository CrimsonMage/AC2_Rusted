//! The shortcut bar's notice checks (`CheckAllItems`, `CheckAllSkills`, `CheckSkill`,
//! `CheckItem`, `RecvNotice_CreateObject`) on the real toolbar layout (`AC2_DAT_DIR`), over a stub player.

use std::cell::RefCell;

use ac2_db::ResourceStore;

use crate::layout::AttributeValue;
use crate::shortcut::{
    Shortcut, ShortcutOptions, ShortcutRequest, ShortcutSource, SHORTCUT_ITEM, SHORTCUT_SKILL,
};
use crate::token::tokenize;
use crate::{build_gameplay_hud, GameplayHud, UiTree};

const SKILL: u32 = 0x03;
const ITEM: u64 = 0x5000_0001;

/// The stub player: its skill's reset, usability and held items can change between checks.
#[derive(Default)]
struct Player {
    reset: RefCell<Option<(f64, f64)>>,
    usable: RefCell<bool>,
    held: RefCell<Vec<u64>>,
    /// The items the client has objects for (`GetWeenieObject`).
    known: RefCell<Vec<u64>>,
}

impl Player {
    fn new() -> Self {
        Self {
            usable: RefCell::new(true),
            held: RefCell::new(vec![ITEM]),
            ..Self::default()
        }
    }
}

/// Slot 0 a skill, slot 1 an item.
fn bar_slots() -> Vec<Shortcut> {
    vec![
        Shortcut {
            kind: SHORTCUT_SKILL,
            skill: SKILL,
            ..Shortcut::default()
        },
        Shortcut {
            kind: SHORTCUT_ITEM,
            item: ITEM,
            ..Shortcut::default()
        },
    ]
}

struct Bar<'a> {
    slots: RefCell<Vec<Shortcut>>,
    player: &'a Player,
}

impl ShortcutSource for Bar<'_> {
    fn shortcuts(&self) -> Vec<Shortcut> {
        let mut all = self.slots.borrow().clone();
        all.resize(100, Shortcut::default());
        all
    }
    fn icon(&self, _: usize, _: &Shortcut) -> u32 {
        0
    }
    fn options(&self) -> ShortcutOptions {
        ShortcutOptions {
            which_set: 1,
            rows: 1,
            ..ShortcutOptions::default()
        }
    }
    fn skill_trained(&self, _: u32) -> bool {
        true
    }
    fn skill_reset(&self, _: u32) -> Option<(f64, f64)> {
        *self.player.reset.borrow()
    }
    fn skill_usable(&self, _: u32) -> bool {
        *self.player.usable.borrow()
    }
    fn has_item(&self, item: u64) -> bool {
        self.player.held.borrow().contains(&item)
    }
    fn item(&self, item: u64) -> Option<crate::ItemSlot> {
        self.player
            .known
            .borrow()
            .contains(&item)
            .then(|| crate::ItemSlot {
                icon: 0x0600_1234,
                name: "Health Potions (3)".into(),
                quantity: 3,
                ..crate::ItemSlot::default()
            })
    }
}

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

fn state(tree: &UiTree, element: usize) -> u32 {
    tree.get(element).unwrap().state()
}

fn meter_position(tree: &UiTree, slot: usize) -> Option<f32> {
    let meter = tree.child_recursive(slot, tokenize("TimeLeftMeter"))?;
    match tree.get(meter)?.attribute(tokenize("position")) {
        Some(AttributeValue::Float(v)) => Some(*v),
        _ => None,
    }
}

#[test]
fn a_skill_on_reset_gets_a_timer_only_on_its_own_slot() {
    let Some(mut hud) = hud() else {
        return;
    };
    let player = Player::new();
    let source = Bar {
        slots: RefCell::new(bar_slots()),
        player: &player,
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    bar.reload(&mut hud.tree, &source);
    *player.reset.borrow_mut() = Some((10.0, 4.0));
    bar.check_all_skills(&mut hud.tree, &source);
    assert_eq!(bar.timers.len(), 1);
    assert_eq!(bar.timers[0].index, 0);
    bar.update_time_left(&mut hud.tree, 0.0);
    let list = bar.elements().1.expect("the list");
    let slot = hud.tree.list_items(list)[0];
    assert!((meter_position(&hud.tree, slot).unwrap() - 0.6).abs() < 1e-6);

    // The skill is ready: its entry ends on the next update, its meter full.
    *player.reset.borrow_mut() = None;
    bar.check_skill(&mut hud.tree, &source, SKILL);
    bar.update_time_left(&mut hud.tree, 0.5);
    assert!(bar.timers.is_empty());
    assert!((meter_position(&hud.tree, slot).unwrap() - 1.0).abs() < 1e-6);
}

#[test]
fn an_unusable_skill_shows_a_long_timer_and_a_zero_total_is_a_tick() {
    let Some(mut hud) = hud() else {
        return;
    };
    let player = Player::new();
    *player.usable.borrow_mut() = false;
    let source = Bar {
        slots: RefCell::new(bar_slots()),
        player: &player,
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    bar.reload(&mut hud.tree, &source);
    bar.check_skill(&mut hud.tree, &source, SKILL);
    assert_eq!(bar.timers.len(), 1);
    assert_eq!(bar.timers[0].total, f64::from(1.0e6_f32));

    // A usable skill whose reset reports a zero total takes `1e-4` seconds.
    *player.usable.borrow_mut() = true;
    *player.reset.borrow_mut() = Some((0.0, 1.0));
    bar.check_skill(&mut hud.tree, &source, SKILL);
    assert_eq!(bar.timers[0].total, f64::from(1.0e-4_f32));
}

#[test]
fn an_item_check_removes_a_missing_item_and_marks_the_pending_one() {
    let Some(mut hud) = hud() else {
        return;
    };
    let player = Player::new();
    let source = Bar {
        slots: RefCell::new(bar_slots()),
        player: &player,
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    bar.reload(&mut hud.tree, &source);
    let list = bar.elements().1.expect("the list");
    let item_slot = hud.tree.list_items(list)[1];

    // Pending (0x40000005): the slot shows pending while it is held. The reload already queued
    // the slot, as the client has no object for the item (`FIXMESetupItemElement`).
    bar.check_item(&mut hud.tree, &source, ITEM, true);
    assert_eq!(state(&hud.tree, item_slot), tokenize("pending"));
    assert_eq!(bar.pending, vec![(ITEM, 1), (ITEM, 1)]);

    // The item is gone: the slot's shortcut is removed and asked of the client.
    player.held.borrow_mut().clear();
    bar.check_item(&mut hud.tree, &source, ITEM, false);
    let requests = bar.take_requests();
    assert!(requests.contains(&ShortcutRequest::Set(1, Shortcut::default())));
}

#[test]
fn a_skill_check_leaves_the_items_alone() {
    let Some(mut hud) = hud() else {
        return;
    };
    let player = Player::new();
    let source = Bar {
        slots: RefCell::new(bar_slots()),
        player: &player,
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    bar.reload(&mut hud.tree, &source);
    // The reload's own set number request.
    bar.take_requests();
    player.held.borrow_mut().clear();
    bar.check_all_skills(&mut hud.tree, &source);
    assert!(bar.take_requests().is_empty());
    bar.check_all_items(&mut hud.tree, &source);
    assert_eq!(
        bar.take_requests(),
        vec![ShortcutRequest::Set(1, Shortcut::default())]
    );
}

#[test]
fn an_unknown_item_waits_pending_until_its_object_is_created() {
    let Some(mut hud) = hud() else {
        return;
    };
    let player = Player::new();
    let source = Bar {
        slots: RefCell::new(bar_slots()),
        player: &player,
    };
    let bar = hud.shortcuts.as_mut().expect("the shortcut bar");
    bar.reload(&mut hud.tree, &source);
    let list = bar.elements().1.expect("the list");
    let item_slot = hud.tree.list_items(list)[1];
    // Queued by `FIXMESetupItemElement`; the reload's `CheckShortcut` (`CheckItem(index)`) then
    // makes the held item's slot stateless again, still waiting and without a tooltip.
    assert_eq!(state(&hud.tree, item_slot), tokenize("stateless"));
    assert_eq!(bar.pending, vec![(ITEM, 1)]);
    assert_eq!(hud.tree.tooltip_text(item_slot), None);

    // Another object, or this one before the client knows it, changes nothing.
    bar.create_object(&mut hud.tree, &source, ITEM + 1);
    bar.create_object(&mut hud.tree, &source, ITEM);
    assert_eq!(bar.pending, vec![(ITEM, 1)]);

    // Created: the slot shows the item's icon and name and no longer waits.
    player.known.borrow_mut().push(ITEM);
    crate::apply_shortcut_checks(
        bar,
        &mut hud.tree,
        &source,
        &crate::shortcut_checks_of(&ac2_weenie::GameplayNotice::CreateObject(ITEM)),
    );
    assert!(bar.pending.is_empty());
    assert_eq!(state(&hud.tree, item_slot), tokenize("stateless"));
    assert_eq!(
        hud.tree.tooltip_text(item_slot).as_deref(),
        Some("Health Potions (3)")
    );
    let icon = hud
        .tree
        .child_recursive(item_slot, tokenize("ShortcutIcon"))
        .expect("the icon");
    assert_eq!(
        hud.tree.get(icon).unwrap().image().map(|i| i.file),
        Some(0x0600_1234)
    );
}
