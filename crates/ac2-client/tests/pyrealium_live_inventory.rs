//! Inventory, equipment, the world drop and corpse looting against a running Pyrealium (`PYREALIUM_LIVE`,
//! see `tests/live/mod.rs`; `PYREALIUM_LIVE_TEST=pyrealium_live_inventory scripts/pyrealium-live-test.sh`),
//! driven through the HUD with the pointer the way the window feeds it.
//!
//! Runs on Pyrealium's 7230 profile with its development training kit (the script's defaults): the
//! character starts with its racial starter gear worn and the training supplies in the pack, and
//! the training creatures beside the start (`PYREALIUM_TEST_CORPSE_LOOT=1`) leave bodies with loot.

#![cfg(feature = "window")]

mod live;

use ac2_net::protocol::ScriptMessage;
use ac2_ui::ToolbarChild;
use ac2_weenie::{GameplayNotice, SessionEvent};
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

/// The server's chat lines since event `from` (what the HUD's chat shows).
fn said_lines(rig: &LiveRig, from: usize) -> Vec<String> {
    rig.seen[from..]
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Notice(GameplayNotice::DisplayStringInfo { text, .. }) => {
                text.literal.clone()
            }
            SessionEvent::Interp {
                function,
                arguments,
                ..
            } => match ScriptMessage::decode(*function, arguments) {
                Some(Ok(ScriptMessage::DisplayStringInfo { text, .. })) => text.literal,
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// Whether a chat line containing `text` arrived since event `from`.
fn said(rig: &LiveRig, from: usize, text: &str) -> bool {
    said_lines(rig, from).iter().any(|l| l.contains(text))
}

/// The commands the HUD asked for since the rig was made, the polling queries left out.
fn asked(rig: &LiveRig) -> Vec<&ac2_client::Command> {
    rig.asked
        .iter()
        .filter(|c| !matches!(c, ac2_client::Command::Social(_)))
        .collect()
}

/// The nearest monster in the world that is alive and has not been fought yet (the weenie type's
/// monster bits, `gmSelectionUI::TargetTypeVsKey`; the class defaults of their entity DIDs give
/// them).
fn next_monster(rig: &LiveRig, fought: &[u64]) -> Option<u64> {
    let frame = rig.client.frame();
    let player = frame.player.as_ref()?;
    frame
        .objects
        .iter()
        .filter(|o| o.id != player.id && !fought.contains(&o.id))
        .filter(|o| {
            let q = &rig
                .client
                .session()
                .objects()
                .get(o.id)
                .map(|x| x.qualities.clone());
            q.as_ref().is_some_and(|q| {
                q.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F)
                    && q.inq_int(0x102).is_none_or(|h| h > 0)
            })
        })
        .map(|o| {
            let d = (o.origin[0] - player.origin[0]).powi(2)
                + (o.origin[1] - player.origin[1]).powi(2)
                + (o.origin[2] - player.origin[2]).powi(2);
            (o.id, d)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Kills the next monster that can be attacked (select it, start the attack, wait for the
/// defeat; the server refuses the training area's non-combatants) and returns it.
fn kill_a_creature(rig: &mut LiveRig, fought: &mut Vec<u64>) -> u64 {
    for _ in 0..16 {
        assert!(
            rig.until(60.0, |r| next_monster(r, fought).is_some()),
            "a creature comes into the world"
        );
        let target = next_monster(rig, fought).unwrap();
        fought.push(target);
        let before = rig.seen.len();
        rig.command(ac2_client::Command::Select(Some(target)));
        rig.run(0.5);
        rig.command(ac2_client::Command::SetAttacking(true));
        let over = rig.until(90.0, |r| {
            said(r, before, "Creature defeated")
                || said(r, before, "not an attackable")
                || said(r, before, "out of attack range")
        });
        let dead = said(rig, before, "Creature defeated");
        rig.command(ac2_client::Command::SetAttacking(false));
        rig.run(0.5);
        if dead {
            return target;
        }
        let health = rig
            .client
            .session()
            .objects()
            .get(target)
            .and_then(|o| o.qualities.inq_int(0x102));
        let mut lines = said_lines(rig, before);
        lines.dedup();
        assert!(
            over,
            "{target:x} (health {health:?}) neither dies nor is refused: {lines:?}"
        );
    }
    panic!("no creature could be attacked");
}

/// Kills creatures until a body holds loot, opens it (a use of the body) and returns it with the
/// items its window lists.
fn open_a_body(rig: &mut LiveRig, fought: &mut Vec<u64>) -> (u64, Vec<u64>) {
    // The hostile creatures beside the start are the three of Pyrealium's training encounter.
    for _ in 0..3 {
        let corpse = kill_a_creature(rig, fought);
        rig.command(ac2_client::Command::Use(corpse));
        rig.run(2.0);

        let items = rig.view.item_icons(EXTERNAL);
        if !items.is_empty() {
            return (corpse, items);
        }
    }
    panic!("no body held any loot");
}

/// The pack's items that are not stacks, in the pack list's order, with their equip stats.
fn single_items(rig: &LiveRig) -> Vec<ac2_client::ItemFrame> {
    let frame = rig.client.frame();
    let pack = rig.view.item_icons(MAIN);
    frame
        .inventory
        .items
        .iter()
        .filter(|i| pack.contains(&i.id) && !i.stats.stackable)
        .cloned()
        .collect()
}

/// The character's starter gear and training supplies are listed, with the names, icons and equip
/// locations their entity DIDs give: the worn pieces on the doll, the rest in the pack, each in
/// its own slot.
#[test]
fn the_inventory_shows_the_starting_items() {
    let Some(rig) = rig("Invstart") else {
        return;
    };
    let pack = rig.view.item_icons(MAIN);
    assert!(pack.len() >= 3, "the pack's items: {pack:x?}");
    assert_eq!(worn(&rig).len(), 3, "the worn items: {:x?}", worn(&rig));
    let frame = rig.client.frame();
    for item in frame.inventory.items.iter() {
        assert_ne!(item.stats.icon, 0, "{:x} has an icon", item.id);
        assert!(item.name.string_id != 0 || item.name.literal.is_some());
    }
    assert!(
        frame
            .inventory
            .items
            .iter()
            .filter(|i| worn(&rig).contains(&i.id))
            .all(|i| i.stats.valid_locations != 0),
        "the worn items name their locations"
    );
    let mut slots: Vec<_> = pack.iter().map(|i| slot_of(&rig, *i)).collect();
    assert!(slots.iter().all(Option::is_some), "{slots:?}");
    slots.sort();
    slots.dedup();
    assert_eq!(slots.len(), pack.len(), "distinct slots");
}

/// Dragging an item to another slot of the pack reorganizes it (`ReorganizeContents`, packed the
/// way Pyrealium checks it against the retail client's packets) and the server's answer puts it there.
#[test]
fn an_item_dragged_to_another_slot_moves_there() {
    let Some(mut rig) = rig("Invmove") else {
        return;
    };
    let item = single_items(&rig)
        .into_iter()
        .map(|i| i.id)
        .find(|i| slot_of(&rig, *i) != Some(0))
        .expect("an item beyond the first slot");
    let from_slot = slot_of(&rig, item).unwrap();
    let from = rig.view.container_slot_center(MAIN, from_slot).unwrap();
    let to = rig.view.container_slot_center(MAIN, 0).unwrap();
    let before = rig.seen.len();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| slot_of(r, item) == Some(0)),
        "the item is in the first slot, not {:?}: {:?} {:?}",
        slot_of(&rig, item),
        said_lines(&rig, before),
        asked(&rig)
    );
}

/// An empty slot past the last item takes the item to the end of the list: the retail UI sends
/// the empty slot's index (`ContainerDisplay::GetNewIndex`), which the server clamps.
#[test]
fn an_item_dropped_on_an_empty_slot_goes_to_the_end() {
    let Some(mut rig) = rig("Invend") else {
        return;
    };
    let count = rig.view.item_icons(MAIN).len();
    let item = single_items(&rig)
        .into_iter()
        .map(|i| i.id)
        .find(|i| slot_of(&rig, *i) == Some(0))
        .expect("an item in the first slot");
    let from = rig.view.container_slot_center(MAIN, 0).unwrap();
    let to = rig.view.container_slot_center(MAIN, count + 3).unwrap();
    let before = rig.seen.len();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| slot_of(r, item) == Some(count - 1)),
        "the item is last, not {:?}: {:?}",
        slot_of(&rig, item),
        said_lines(&rig, before)
    );
}

/// A weapon dropped on the paper doll's figure is worn at its preferred location (the entity's
/// `PreferredLocation`), and dragged from its slot into the pack it comes off.
#[test]
fn an_item_equips_and_unequips() {
    let Some(mut rig) = rig("Invequip") else {
        return;
    };
    let item = single_items(&rig)
        .into_iter()
        .find(|i| i.stats.preferred_location != 0)
        .expect("an item with a preferred location");
    let from = rig.view.item_icon_center(MAIN, item.id).unwrap();
    let to = rig.view.ui_element_center("PaperDollViewport").unwrap();
    let before = rig.seen.len();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| worn(r).contains(&item.id)),
        "worn: {:?}",
        said_lines(&rig, before)
    );
    assert!(!rig.view.item_icons(MAIN).contains(&item.id));
    let from = rig
        .view
        .item_icons("RightSidePanel")
        .contains(&item.id)
        .then(|| rig.view.item_icon_center("", item.id))
        .flatten()
        .expect("the worn icon");
    let to = rig.view.container_slot_center(MAIN, 0).unwrap();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| r.view.item_icons(MAIN).contains(&item.id)),
        "unequipped: {:?} {:?}",
        rig.client.take_script_problems(),
        asked(&rig)
    );
    assert!(!worn(&rig).contains(&item.id));
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
    let item = single_items(&rig)[0].id;
    let from = rig.view.item_icon_center(MAIN, item).unwrap();
    let before = rig.seen.len();
    rig.drag(from, (300, 250));
    assert!(
        rig.until(5.0, |r| !r.view.item_icons(MAIN).contains(&item)),
        "dropped: {:?}",
        said_lines(&rig, before)
    );
    // The ground object is a new object (Pyrealium names it in its own range).
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
        rig.until(5.0, |r| r.view.item_icons(MAIN).len() == before_pack.len()),
        "picked up"
    );
    assert!(rig
        .view
        .item_icons(MAIN)
        .iter()
        .any(|i| !before_pack.contains(i)));
}

/// A creature killed through the HUD leaves gold, which the money shows, and a body: a use of it
/// opens its window on its loot; an item dragged out of it lands in the pack, and the Loot All
/// button of the next body takes everything it holds. (Needs the server's
/// `PYREALIUM_TEST_CORPSE_LOOT=1`: its training creatures pay into the pack and leave no body.)
#[test]
fn a_corpse_is_looted_by_drag_and_by_loot_all() {
    let Some(mut rig) = rig("Invloot") else {
        return;
    };
    let gold = rig.client.frame().inventory.money;
    let mut fought = Vec::new();
    let (_, items) = open_a_body(&mut rig, &mut fought);
    assert!(rig.client.frame().inventory.money > gold, "the gold shows");
    let pack = rig.view.item_icons(MAIN);
    let from = rig.view.item_icon_center(EXTERNAL, items[0]).unwrap();
    let to = rig.view.container_slot_center(MAIN, pack.len()).unwrap();
    let before = rig.seen.len();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| r.view.item_icons(MAIN).len() == pack.len() + 1),
        "one looted: {:?}",
        said_lines(&rig, before)
    );
    assert_eq!(rig.view.item_icons(EXTERNAL).len(), items.len() - 1);
    // The next body, all at once.
    let (_, items) = open_a_body(&mut rig, &mut fought);
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

/// The pack's stack (the coin stack of the training kit): its item and quantity.
fn stack_in_pack(rig: &LiveRig) -> Option<(u64, i32)> {
    let pack = rig.view.item_icons(MAIN);
    rig.client
        .frame()
        .inventory
        .items
        .iter()
        .find(|i| pack.contains(&i.id) && i.stats.stackable && i.stats.quantity > 1)
        .map(|i| (i.id, i.stats.quantity))
}

/// A stack dragged to another slot asks how many first (`gmStackUI`); the default answer, all of
/// it, moves the whole stack.
#[test]
fn a_stack_dragged_to_another_slot_asks_how_many_and_moves() {
    let Some(mut rig) = rig("Invstack") else {
        return;
    };
    let (stack, quantity) = stack_in_pack(&rig).expect("a stack in the pack");
    let count = rig.view.item_icons(MAIN).len();
    let from = rig.view.item_icon_center(MAIN, stack).unwrap();
    let to = rig.view.container_slot_center(MAIN, 0).unwrap();
    let before = rig.seen.len();
    rig.drag(from, to);
    let ok = rig.view.ui_element_center("StackOkButton");
    assert!(ok.is_some(), "the stack dialog asks how many ({quantity})");
    rig.click(ok.unwrap());
    assert!(
        rig.until(5.0, |r| slot_of(r, stack) == Some(0)),
        "the stack is in the first slot, not {:?}: {:?} {:?}",
        slot_of(&rig, stack),
        said_lines(&rig, before),
        asked(&rig)
    );
    assert_eq!(rig.view.item_icons(MAIN).len(), count);
}

/// Half of a stack dragged to another slot splits it: a second stack stands in the new slot and
/// the first keeps the rest.
#[test]
fn half_of_a_stack_splits_off_a_new_stack() {
    let Some(mut rig) = rig("Invsplit") else {
        return;
    };
    let (stack, quantity) = stack_in_pack(&rig).expect("a stack in the pack");
    let count = rig.view.item_icons(MAIN).len();
    let from = rig.view.item_icon_center(MAIN, stack).unwrap();
    let to = rig.view.container_slot_center(MAIN, count + 2).unwrap();
    let before = rig.seen.len();
    rig.drag(from, to);
    let toggle = rig
        .view
        .ui_element_center("AmountToggleButton")
        .expect("the stack dialog has its all / half toggle");
    rig.click(toggle);
    let ok = rig.view.ui_element_center("StackOkButton").unwrap();
    rig.click(ok);
    assert!(
        rig.until(5.0, |r| r.view.item_icons(MAIN).len() == count + 1),
        "a second stack appears: {:?} {:?}",
        said_lines(&rig, before),
        asked(&rig)
    );
    // The original is created again with what is left, the new stack with what was split off.
    let stacks = |r: &LiveRig| -> Vec<i32> {
        let pack = r.view.item_icons(MAIN);
        r.client
            .frame()
            .inventory
            .items
            .iter()
            .filter(|i| i.stats.stackable && pack.contains(&i.id))
            .map(|i| i.stats.quantity)
            .collect()
    };
    assert!(
        rig.until(5.0, |r| {
            let q = stacks(r);
            q.len() == 2 && q.iter().sum::<i32>() == quantity
        }),
        "two stacks of {quantity} between them: {:?}",
        stacks(&rig)
    );
    assert!(stacks(&rig).iter().all(|q| *q < quantity));
}

/// The pack's stacks (stackable items): their ids and quantities, in the pack list's order.
fn stacks_in_pack(rig: &LiveRig) -> Vec<(u64, i32)> {
    let pack = rig.view.item_icons(MAIN);
    rig.client
        .frame()
        .inventory
        .items
        .iter()
        .filter(|i| i.stats.stackable && pack.contains(&i.id))
        .map(|i| (i.id, i.stats.quantity))
        .collect()
}

/// Half of a stack dropped on a slot that holds another item lands on that slot, not at the end:
/// the request names the slot under the pointer (`ContainerDisplay::GetNewIndex`).
#[test]
fn a_split_stack_lands_on_the_slot_it_is_dropped_on() {
    let Some(mut rig) = rig("Invslot") else {
        return;
    };
    let (stack, quantity) = stack_in_pack(&rig).expect("a stack in the pack");
    let count = rig.view.item_icons(MAIN).len();
    let from = rig.view.item_icon_center(MAIN, stack).unwrap();
    // A slot of another item, in front of the stack's own.
    let target = (0..count)
        .find(|&i| i != slot_of(&rig, stack).unwrap())
        .unwrap();
    let to = rig.view.container_slot_center(MAIN, target).unwrap();
    let before = rig.seen.len();
    rig.drag(from, to);
    let toggle = rig
        .view
        .ui_element_center("AmountToggleButton")
        .expect("the stack dialog has its all / half toggle");
    rig.click(toggle);
    let ok = rig.view.ui_element_center("StackOkButton").unwrap();
    rig.click(ok);
    assert!(
        rig.until(5.0, |r| r.view.item_icons(MAIN).len() == count + 1),
        "a second stack appears: {:?} {:?}",
        said_lines(&rig, before),
        asked(&rig)
    );
    assert!(
        rig.until(5.0, |r| stacks_in_pack(r).len() == 2),
        "two stacks: {:?}",
        stacks_in_pack(&rig)
    );
    let new_stack = stacks_in_pack(&rig)
        .into_iter()
        .map(|(id, _)| id)
        .find(|id| *id != stack)
        .expect("the new stack");
    assert!(
        rig.until(5.0, |r| slot_of(r, new_stack) == Some(target)),
        "the new stack is on slot {target}, not {:?} (the original had {quantity})",
        slot_of(&rig, new_stack)
    );
}

/// A stack dropped on another stack of the same kind merges into it: one stack with both
/// quantities stands where the target stood, and the dropped stack's slot closes.
#[test]
fn a_stack_dropped_on_another_stack_merges() {
    let Some(mut rig) = rig("Invmerge") else {
        return;
    };
    let (stack, quantity) = stack_in_pack(&rig).expect("a stack in the pack");
    let count = rig.view.item_icons(MAIN).len();
    // Split it first: half goes to a new stack at the end.
    let from = rig.view.item_icon_center(MAIN, stack).unwrap();
    let to = rig.view.container_slot_center(MAIN, count + 2).unwrap();
    rig.drag(from, to);
    let toggle = rig.view.ui_element_center("AmountToggleButton").unwrap();
    rig.click(toggle);
    let ok = rig.view.ui_element_center("StackOkButton").unwrap();
    rig.click(ok);
    assert!(
        rig.until(5.0, |r| {
            let s = stacks_in_pack(r);
            s.len() == 2 && s.iter().map(|(_, q)| q).sum::<i32>() == quantity
        }),
        "two stacks of {quantity}: {:?}",
        stacks_in_pack(&rig)
    );
    // The new stack onto the first one, all of it.
    let two = stacks_in_pack(&rig);
    let (target, moved) = (two[0].0, two[1].0);
    let from = rig.view.item_icon_center(MAIN, moved).unwrap();
    let to = rig.view.item_icon_center(MAIN, target).unwrap();
    let before = rig.seen.len();
    rig.drag(from, to);
    let ok = rig
        .view
        .ui_element_center("StackOkButton")
        .expect("the stack dialog asks how many");
    rig.click(ok);
    assert!(
        rig.until(5.0, |r| stacks_in_pack(r).len() == 1),
        "the stacks merged: {:?} {:?} {:?}",
        stacks_in_pack(&rig),
        said_lines(&rig, before),
        asked(&rig)
    );
    assert_eq!(stacks_in_pack(&rig)[0].1, quantity);
    assert_eq!(rig.view.item_icons(MAIN).len(), count);
}
