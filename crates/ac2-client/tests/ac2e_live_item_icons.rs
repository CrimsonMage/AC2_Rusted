//! The items' icons and the paper doll's slots against a running AC2e
//! (`AC2E_LIVE_TEST=ac2e_live_item_icons scripts/ac2e-live-test.sh`), with the frames not waiting
//! for the loader threads as the window's do: an item's class defaults (its icon among them,
//! `CBaseQualities::InitDefaults`) arrive some frames after its `CreateObject`, and the slots
//! must still draw the icon and the doll still show what is worn.

#![cfg(feature = "window")]

mod live;

use ac2_ui::ToolbarChild;
use live::LiveRig;

/// `default_icon.tga`, the icon of an item whose icon id is not set (the "?" slot).
const DEFAULT_ICON: u32 = 0x4100_034b;
const MAIN: &str = "PackMainContainer";

fn rig(name: &str) -> Option<LiveRig> {
    let mut rig = LiveRig::new_unsettled(name)?;
    rig.panel(ToolbarChild::Inventory, true);
    rig.run(2.0);
    Some(rig)
}

/// The items on the paper doll: the drawer's icons that are not in the pack.
fn worn(rig: &LiveRig) -> Vec<u64> {
    let pack = rig.view.item_icons(MAIN);
    let mut all = rig.view.item_icons("RightSidePanel");
    all.retain(|i| !pack.contains(i));
    all
}

/// Every pack and worn item draws its icon (the item's icon id from its class defaults), none is
/// the "?" of an icon id of 0.
fn assert_icons(rig: &LiveRig) {
    let frame = rig.client.frame();
    let mut shown = rig.view.item_icon_images(MAIN);
    shown.extend(rig.view.item_icon_images("RightSidePanel"));
    assert!(shown.len() >= 6, "the icons shown: {shown:x?}");
    for (item, file) in shown {
        let stats = &frame.inventory.items.iter().find(|i| i.id == item).expect("a listed item").stats;
        assert_ne!(stats.icon, 0, "{item:x} has an icon id");
        let layers = &frame.inventory.items.iter().find(|i| i.id == item).unwrap().icon_layers;
        let expect = layers.first().map_or(stats.icon, |l| l.1);
        assert_eq!(file, expect, "{item:x} draws its icon {layers:x?}");
        assert_ne!(file, DEFAULT_ICON, "{item:x} draws the default '?' icon");
    }
}

#[test]
fn the_pack_and_the_doll_draw_the_items_icons() {
    let Some(mut rig) = rig("Iconshow") else {
        return;
    };
    assert_icons(&rig);
    assert_eq!(worn(&rig).len(), 3, "the worn items: {:x?}", worn(&rig));
    // A moment later they still do (nothing blanks them).
    rig.run(1.0);
    assert_icons(&rig);
}

/// A weapon dropped on the doll's figure is worn in its slot with its icon, at once and not only
/// once its class defaults have been read; the worn armour keeps its slots.
#[test]
fn an_equipped_bow_shows_in_its_slot() {
    let Some(mut rig) = rig("Iconbow") else {
        return;
    };
    let before = worn(&rig);
    let item = rig
        .client
        .frame()
        .inventory
        .items
        .iter()
        .find(|i| {
            i.stats.preferred_location != 0
                && rig.view.item_icons(MAIN).contains(&i.id)
                && !i.stats.stackable
        })
        .cloned()
        .expect("a pack item with a preferred location");
    let from = rig.view.item_icon_center(MAIN, item.id).unwrap();
    let to = rig.view.ui_element_center("PaperDollViewport").unwrap();
    rig.drag(from, to);
    assert!(
        rig.until(5.0, |r| worn(r).contains(&item.id)),
        "worn: {:x?} asked {:?}",
        worn(&rig),
        rig.asked
    );
    assert_eq!(worn(&rig).len(), before.len() + 1);
    assert_icons(&rig);
    rig.run(1.0);
    assert_icons(&rig);
}
