//! Inventory, equipment, the world drop and corpse looting against a running AC2e (`AC2E_LIVE`,
//! see `tests/live/mod.rs`; `AC2E_LIVE_TEST=ac2e_live_inventory scripts/ac2e-live-test.sh`),
//! driven through the HUD with the pointer the way the window feeds it.
//!
//! AC2e grants a character its racial starter gear (three worn pieces and three items in the pack)
//! only on the Legions DAT profile: run the server with `AC2E_DATA=legions` on the DVD DATs
//! (`AC2_DAT_DIR`) and log on as `AC2E_LIVE_VERSION=7148`, which that profile accepts. On the
//! 7230 profile a new character owns nothing, so these tests fail on their first assertion there.
//! AC2e names no item (the name, icon and equip locations of its objects are not sent), so the
//! tests look items up by id.

#![cfg(feature = "window")]

mod live;

use ac2_net::protocol::{ObjectAction, ScriptMessage};
use ac2_ui::ToolbarChild;
use ac2_weenie::SessionEvent;
use live::LiveRig;

const MAIN: &str = "PackMainContainer";
const EXTERNAL: &str = "ExternalContainer";

/// A character in the world with the inventory drawer open.
fn rig(name: &str) -> Option<LiveRig> {
    let mut rig = LiveRig::new(name)?;
    rig.panel(ToolbarChild::Inventory, true);
    rig.run(2.0);
    Some(rig)
}

/// The slot of the main page that shows `item`.
fn slot_of(rig: &LiveRig, item: u64) -> Option<usize> {
    let at = rig.view.item_icon_center(MAIN, item)?;
    (0..42).find(|&i| rig.view.container_slot_center(MAIN, i) == Some(at))
}

/// The items on the paper doll: the drawer's icons that are not in the pack.
fn worn(rig: &LiveRig) -> Vec<u64> {
    let pack = rig.view.item_icons(MAIN);
    let mut all = rig.view.item_icons("RightSidePanel");
    all.retain(|i| !pack.contains(i));
    all
}

/// Whether a chat line containing `text` arrived since event `from`.
fn said(rig: &LiveRig, from: usize, text: &str) -> bool {
    rig.seen[from..].iter().any(|e| match e {
        SessionEvent::Interp {
            function,
            arguments,
            ..
        } => matches!(
            ScriptMessage::decode(*function, arguments),
            Some(Ok(ScriptMessage::DisplayStringInfo { text: t, .. }))
                if t.literal.as_deref().is_some_and(|l| l.contains(text))
        ),
        _ => false,
    })
}

/// The creatures this run has fought (their bodies stay in view).
static FOUGHT: std::sync::Mutex<Vec<u64>> = std::sync::Mutex::new(Vec::new());

/// A creature that has attacked the player and has not been fought yet: the monsters beside the
/// start come to it (the props in view have no names either, and never attack).
fn attacker(rig: &LiveRig, fought: &[u64]) -> Option<u64> {
    rig.seen.iter().find_map(|e| match e {
        SessionEvent::ObjectAction {
            object,
            action: ObjectAction::DoBehavior(_),
        } if object >> 48 == 0x8140 && !fought.contains(object) => Some(*object),
        _ => None,
    })
}

/// Kills the next creature that attacks (select it, start the attack, wait for the defeat) and
/// returns it.
fn kill_a_creature(rig: &mut LiveRig) -> u64 {
    assert!(
        rig.until(60.0, |r| attacker(r, &FOUGHT.lock().unwrap()).is_some()),
        "a creature attacks"
    );
    let target = attacker(rig, &FOUGHT.lock().unwrap()).unwrap();
    FOUGHT.lock().unwrap().push(target);
    let before = rig.seen.len();
    rig.command(ac2_client::Command::Select(Some(target)));
    rig.run(0.5);
    rig.command(ac2_client::Command::SetAttacking(true));
    assert!(
        rig.until(90.0, |r| said(r, before, "Creature defeated")),
        "the creature dies"
    );
    rig.command(ac2_client::Command::SetAttacking(false));
    rig.run(0.5);
    target
}

/// Kills creatures until a body holds loot, opens it (a use of the body) and returns it with the
/// items its window lists.
fn open_a_body(rig: &mut LiveRig) -> (u64, Vec<u64>) {
    for _ in 0..4 {
        let corpse = kill_a_creature(rig);
        rig.command(ac2_client::Command::Use(corpse));
        rig.run(2.0);
        let items = rig.view.item_icons(EXTERNAL);
        if !items.is_empty() {
            return (corpse, items);
        }
    }
    panic!("no body held any loot");
}

/// The character's starter gear is listed: three worn pieces on the doll and three items in the
/// pack, each in its own slot.
#[test]
fn the_inventory_shows_the_starting_items() {
    let Some(rig) = rig("Invstart") else {
        return;
    };
    let pack = rig.view.item_icons(MAIN);
    assert_eq!(pack.len(), 3, "the pack's items: {pack:x?}");
    assert_eq!(worn(&rig).len(), 3, "the worn items: {:x?}", worn(&rig));
    let mut slots: Vec<_> = pack.iter().map(|i| slot_of(&rig, *i)).collect();
    assert!(slots.iter().all(Option::is_some), "{slots:?}");
    slots.sort();
    slots.dedup();
    assert_eq!(slots.len(), 3, "distinct slots");
}

/// Dragging an item to another slot of the pack reorganizes it (`ReorganizeContents`, packed the
/// way AC2e checks it against the retail client's packets) and the server's answer puts it there.
#[test]
fn an_item_dragged_to_another_slot_moves_there() {
    let Some(mut rig) = rig("Invmove") else {
        return;
    };
    let item = rig
        .view
        .item_icons(MAIN)
        .into_iter()
        .find(|i| slot_of(&rig, *i) != Some(0))
        .expect("an item beyond the first slot");
    let from_slot = slot_of(&rig, item).unwrap();
    let from = rig.view.container_slot_center(MAIN, from_slot).unwrap();
    let to = rig.view.container_slot_center(MAIN, 0).unwrap();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| slot_of(r, item) == Some(0)),
        "the item is in the first slot, not {:?}",
        slot_of(&rig, item)
    );
    assert_eq!(rig.view.item_icons(MAIN).len(), 3);
}

/// An item dragged onto a hand slot of the paper doll is worn (`EquipItem`), and dragged from it
/// into the pack it comes off (`UnEquipItem`).
#[test]
fn an_item_equips_and_unequips() {
    let Some(mut rig) = rig("Invequip") else {
        return;
    };
    // AC2e does not send the items' equip locations, so each item is offered to each hand slot
    // until the server takes it.
    let mut equipped = None;
    'search: for item in rig.view.item_icons(MAIN) {
        for slot in ["PrimaryHand_Slot", "SecondaryHand_Slot"] {
            let from = rig.view.item_icon_center(MAIN, item).unwrap();
            let to = rig.view.ui_element_center(slot).unwrap();
            rig.drag(from, to);
            rig.run(1.5);
            if worn(&rig).contains(&item) {
                equipped = Some((item, slot));
                break 'search;
            }
        }
    }
    let (item, slot) = equipped.expect("an item equips");
    assert!(!rig.view.item_icons(MAIN).contains(&item));
    assert!(rig.view.item_icon_center(slot, item).is_some());
    let from = rig.view.item_icon_center(slot, item).unwrap();
    let to = rig.view.container_slot_center(MAIN, 0).unwrap();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| r.view.item_icons(MAIN).contains(&item)),
        "unequipped"
    );
    assert!(!worn(&rig).contains(&item));
}

/// Dropping an item on the ground and picking it up again: a drag out of the drawer into the
/// world (`gm3DItemsUI`) names the player's landblock, the server puts the item on the ground,
/// and a use of it brings it back.
#[test]
fn an_item_drops_on_the_ground_and_is_picked_up() {
    let Some(mut rig) = rig("Invdrop") else {
        return;
    };
    let before_pack = rig.view.item_icons(MAIN);
    let item = before_pack[0];
    let from = rig.view.item_icon_center(MAIN, item).unwrap();
    let before = rig.seen.len();
    rig.drag(from, (300, 250));
    assert!(
        rig.until(5.0, |r| !r.view.item_icons(MAIN).contains(&item)),
        "dropped"
    );
    // The ground object is a new object (AC2e names it in its own range).
    let ground = rig.seen[before..]
        .iter()
        .find_map(|e| match e {
            SessionEvent::ObjectAdded(id) => Some(*id),
            _ => None,
        })
        .expect("the item appears on the ground");
    assert!(rig.client.session().objects().get(ground).is_some());
    rig.command(ac2_client::Command::Use(ground));
    // The server numbers the item again when it comes back into the pack.
    assert!(
        rig.until(5.0, |r| r.view.item_icons(MAIN).len() == 3),
        "picked up"
    );
    assert!(rig
        .view
        .item_icons(MAIN)
        .iter()
        .any(|i| !before_pack.contains(i)));
}

/// A creature killed through the HUD leaves gold, which the money shows, and a body: a use of it
/// opens its window on its loot, and an item dragged out of it lands in the pack.
#[test]
fn an_item_is_looted_from_a_corpse_by_drag() {
    let Some(mut rig) = rig("Invloot") else {
        return;
    };
    let gold = rig.client.frame().inventory.money;
    let (_, items) = open_a_body(&mut rig);
    assert!(rig.client.frame().inventory.money > gold, "the gold shows");
    let pack = rig.view.item_icons(MAIN);
    let from = rig.view.item_icon_center(EXTERNAL, items[0]).unwrap();
    let to = rig.view.container_slot_center(MAIN, pack.len()).unwrap();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| r.view.item_icons(MAIN).len() == pack.len() + 1),
        "one looted"
    );
    assert_eq!(rig.view.item_icons(EXTERNAL).len(), items.len() - 1);
}

/// The Loot All button takes everything the body holds.
#[test]
fn a_corpse_is_looted_by_loot_all() {
    let Some(mut rig) = rig("Invlootall") else {
        return;
    };
    let (_, items) = open_a_body(&mut rig);
    let pack = rig.view.item_icons(MAIN);
    let button = rig.view.ui_element_center("LootAllButton").unwrap();
    rig.click(button);
    assert!(
        rig.until(5.0, |r| r.view.item_icons(MAIN).len()
            == pack.len() + items.len()),
        "all looted"
    );
    assert!(rig.view.item_icons(EXTERNAL).is_empty());
}
