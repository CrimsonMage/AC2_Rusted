//! A relog against a running Pyrealium server (`PYREALIUM_LIVE`, see `tests/live/mod.rs`): the player logs
//! off to character selection (`gmGamePlayUI::EndSession`) and enters the world again; the second
//! entry fills the world as the first did (the creatures and objects around the start, the
//! logon's chat lines). And uses that the server refuses are sent again by the same input.
//!
//! Run with `PYREALIUM_LIVE_TEST=pyrealium_live_relog scripts/pyrealium-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_net::protocol::ScriptMessage;
use ac2_ui::ToolbarChild;
use ac2_weenie::{GameplayNotice, SessionEvent};
use live::LiveRig;

const PACK: &str = "PackMainContainer";
/// The second press of a double click as the input layer sends it.
const DOUBLE_CLICK: u32 = 0x4000_0007;
/// The start's lifestone (`wsl_lifestone`'s entity), 4 m from the start: out of use range.
const LIFESTONE: u32 = 1191184350;

/// The server's chat lines since event `from` (the notice and the script call that carries it
/// both count).
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

/// The objects the session holds other than the player.
fn others(rig: &LiveRig) -> usize {
    let me = rig.client.session().player();
    rig.client
        .session()
        .objects()
        .iter()
        .filter(|o| Some(o.id) != me)
        .count()
}

/// The objects the frame draws in the world.
fn in_world(rig: &LiveRig) -> usize {
    rig.client
        .frame()
        .objects
        .iter()
        .filter(|o| o.in_world)
        .count()
}

/// The world object with entity `entity`.
fn object_of(rig: &LiveRig, entity: u32) -> Option<u64> {
    rig.client
        .session()
        .objects()
        .iter()
        .find(|o| o.entity == Some(entity))
        .map(|o| o.id)
}

/// The count of `UseSelection` commands the HUD and the test asked for.
fn uses(rig: &LiveRig) -> usize {
    rig.asked
        .iter()
        .filter(|c| matches!(c, Command::UseSelection))
        .count()
}

/// After a log off to character selection and the second `EnterWorld` the world is filled again:
/// the objects around the start are in the object list and drawn, and the logon's chat lines
/// arrive. Pyrealium's transport kept the first entry's ordered-stamp counter, so the client's
/// restarted stamps (`ClientNet::EnterWorld`: `Proto_UI::SetOrderingStamp(0)`) were dropped as
/// old, `SceneRenderingComplete` among them, and the server never sent the scene.
#[test]
fn the_second_world_entry_is_filled_like_the_first() {
    let Some(mut rig) = LiveRig::new("Relogger") else {
        return;
    };
    rig.run(5.0);
    let first = (others(&rig), in_world(&rig), said_lines(&rig, 0).len());
    eprintln!(
        "first entry: objects {} drawn {} chat {}",
        first.0, first.1, first.2
    );
    assert!(
        first.0 > 5,
        "the first entry holds the start's objects: {first:?}"
    );
    assert!(first.2 > 0, "the first entry brings logon chat lines");
    let mark = rig.seen.len();
    rig.relog();
    rig.run(5.0);
    let second = (others(&rig), in_world(&rig), said_lines(&rig, mark));
    eprintln!(
        "second entry: objects {} drawn {} chat {}",
        second.0,
        second.1,
        second.2.len()
    );
    assert!(
        second.0 + 1 >= first.0,
        "the second entry holds the objects: {second:?} vs {first:?}"
    );
    assert!(
        second.1 + 1 >= first.1,
        "the objects are drawn: {second:?} vs {first:?}"
    );
    assert!(
        second.2.iter().any(|l| l.contains("AGPL")),
        "the licence line arrives again: {:?}",
        second.2
    );
    assert!(
        object_of(&rig, LIFESTONE).is_some(),
        "the lifestone is in the object list"
    );
}

/// An object out of use range is refused ("Move within four meters of the lifestone") and used
/// again by the same selection and use, as often as asked: the player's usage script keeps no
/// state of a refused use.
#[test]
fn a_use_out_of_range_is_sent_again_each_time() {
    let Some(mut rig) = LiveRig::new("Userefused") else {
        return;
    };
    rig.wait("the lifestone", |r| object_of(r, LIFESTONE).is_some());
    let stone = object_of(&rig, LIFESTONE).unwrap();
    let refusals = |rig: &LiveRig| {
        said_lines(rig, 0)
            .iter()
            .filter(|l| l.contains("Move within four meters"))
            .count()
    };
    for n in 0..3 {
        let told = refusals(&rig);
        rig.command(Command::Select(Some(stone)));
        rig.command(Command::UseSelection);
        assert!(
            rig.until(5.0, |r| refusals(r) > told),
            "use {n}: the server was not asked again ({told} refusals)"
        );
    }
}

/// A double click on each pack item sends its use, even right after its first press selected it:
/// the selection's item ring (modern HUD) fans out of the slot and must not cover it, or the
/// second press lands on a ring button and no use is sent. Pack items are consumed, equipped or
/// refused by the server; whatever it answers, each double click reaches `UseSelection`.
#[test]
fn a_double_click_on_a_selected_pack_item_always_sends_the_use() {
    let Some(mut rig) = LiveRig::new("Useagain") else {
        return;
    };
    rig.panel(ToolbarChild::Inventory, true);
    rig.run(2.0);
    let items = rig.view.item_icons(PACK);
    assert!(items.len() >= 4, "the training kit's items: {items:?}");
    for item in items {
        // The previous item's ring (it fans over the neighbouring slots) goes with its selection.
        rig.command(Command::Select(None));
        rig.run(0.4);
        let Some(at) = rig.view.item_icon_center(PACK, item) else {
            continue;
        };
        rig.point(at);
        rig.button(true);
        rig.button(false);
        // The selection, then the ring that opens on its slot.
        rig.run(0.4);
        let before = uses(&rig);
        rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
        rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
        rig.run(0.4);
        assert!(
            uses(&rig) > before,
            "the double click on {item:#x} at {at:?} sent no use"
        );
    }
}
