//! A killed creature's body through the pointer, against a running Pyrealium server (`PYREALIUM_LIVE`, see
//! `tests/live/mod.rs`): a click on the body in the world selects it (`gmSelectionUI` ->
//! `Finish3DSelection` over the selection render's pick list) and a double click uses it, which
//! opens the corpse's loot (the container opens).
//!
//! Run with `PYREALIUM_LIVE_TEST=pyrealium_live_corpse_click scripts/pyrealium-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use live::{LiveRig, SIZE};

/// `gmSelectionUI`'s double click action.
const DOUBLE_CLICK: u32 = 0x4000_0007;

fn point_on(rig: &LiveRig, id: u64) -> Option<(i32, i32)> {
    let frame = rig.client.frame();
    (0..SIZE.1 as i32).step_by(4).find_map(|y| {
        (0..SIZE.0 as i32).step_by(4).find_map(|x| {
            (rig.view.pick_list(&frame, SIZE, (x, y)).first() == Some(&id)).then_some((x, y))
        })
    })
}

fn monsters(rig: &LiveRig) -> Vec<u64> {
    let player = rig.client.session().player();
    rig.client
        .session()
        .objects()
        .iter()
        .filter(|o| Some(o.id) != player)
        .filter(|o| o.qualities.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F))
        .filter(|o| o.qualities.inq_int(0x102).is_none_or(|h| h > 0))
        .map(|o| o.id)
        .collect()
}

#[test]
fn a_click_selects_a_corpse_and_a_double_click_opens_it() {
    let Some(mut rig) = LiveRig::new("Corpseclick") else {
        return;
    };
    rig.wait("no creature", |r| !monsters(r).is_empty());
    let mut corpse = None;
    for _ in 0..8 {
        rig.wait("no creature", |r| !monsters(r).is_empty());
        let target = monsters(&rig)[0];
        rig.command(Command::Select(Some(target)));
        rig.run(0.5);
        rig.command(Command::SetAttacking(true));
        let dead = rig.until(90.0, |r| {
            r.client
                .session()
                .objects()
                .get(target)
                .is_some_and(|o| o.qualities.inq_int(0x102) == Some(0))
        });
        rig.command(Command::SetAttacking(false));
        rig.run(1.0);
        if dead {
            corpse = Some(target);
            break;
        }
    }
    let corpse = corpse.expect("a creature killed");
    // Let the death play and the body settle.
    rig.command(Command::Select(None));
    rig.run(3.0);
    rig.settle_view();
    {
        let q = &rig
            .client
            .session()
            .objects()
            .get(corpse)
            .unwrap()
            .qualities;
        eprintln!(
            "corpse {corpse:#x}: selectable 0x270 {:?}, type 0x14 {:?}, health {:?}",
            q.inq_bool(0x270),
            q.inq_int(0x14),
            q.inq_int(0x102)
        );
    }
    let frame = rig.client.frame();
    let player = frame.player.as_ref().map(|p| p.origin);
    let body = frame
        .objects
        .iter()
        .find(|o| o.id == corpse)
        .map(|o| o.origin);
    eprintln!("player {player:?}, body {body:?}");
    if let Some(o) = frame.objects.iter().find(|o| o.id == corpse) {
        eprintln!(
            "body: in_world {} holder {:?} hidden by fx {} visual {:?}",
            o.in_world,
            o.holder.map(|h| h.id),
            frame.fx_looks.hidden.contains(&corpse),
            o.visual
        );
    }
    let mut anywhere = Vec::new();
    let mut firsts = std::collections::BTreeSet::new();
    for y in (0..SIZE.1 as i32).step_by(4) {
        for x in (0..SIZE.0 as i32).step_by(4) {
            let list = rig.view.pick_list(&frame, SIZE, (x, y));
            if let Some(&f) = list.first() {
                firsts.insert(f);
            }
            if list.contains(&corpse) {
                anywhere.push(((x, y), list));
            }
        }
    }
    eprintln!("objects picked first anywhere: {firsts:x?}");
    eprintln!(
        "the body is in {} pick lists, e.g. {:x?}",
        anywhere.len(),
        anywhere.first()
    );
    let at = point_on(&rig, corpse).expect("the pick list finds the body somewhere on screen");
    rig.click(at);
    assert_eq!(
        rig.client.frame().selection.map(|s| s.id),
        Some(corpse),
        "a click on the body selects it"
    );
    // The second press of a double click, as the input layer sends it.
    rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
    rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
    assert!(
        rig.until(10.0, |r| r
            .client
            .frame()
            .inventory
            .containers
            .iter()
            .any(|c| c.id == corpse)),
        "the double click opens the body"
    );
}
