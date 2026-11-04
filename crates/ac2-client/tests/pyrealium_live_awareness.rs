//! Creature awareness against a running Pyrealium server (see `tests/live/mod.rs`): the Banderling
//! Thrasher `scripts/pyrealium-live-test.sh` places 8 m north of the start notices the new
//! character standing at the start and closes in (Pyrealium's perception scan and pursuit), and the
//! client draws it where the server put it: the creature's drawn body reaches the server's
//! position beside the player (its interpolation, `InterpolationManager`, walks or blips it there;
//! a server position under the body's resting height made every blip fail and froze it short).
//!
//! Run with `PYREALIUM_LIVE_TEST=pyrealium_live_awareness scripts/pyrealium-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use live::LiveRig;

/// The Banderling Thrasher the script places for this file.
const THRASHER: u64 = 0x8140_0000_0003_0062;

/// The creature's and the player's origins (block-local, with their cells) as the session has them.
fn places(rig: &LiveRig) -> Option<((u32, [f32; 3]), (u32, [f32; 3]))> {
    let session = rig.client.session();
    let player = session.player()?;
    let at = |id: u64| {
        session
            .objects()
            .iter()
            .find(|o| o.id == id)
            .and_then(|o| o.position.map(|p| (p.cell, p.origin)))
    };
    Some((at(THRASHER)?, at(player)?))
}

fn distance(a: (u32, [f32; 3]), b: (u32, [f32; 3])) -> f32 {
    let [ax, ay, _] = a.1;
    let [bx, by, _] = b.1;
    (ax - bx).hypot(ay - by)
}

#[test]
fn a_creature_notices_the_player_closes_in_and_is_drawn_there() {
    let Some(mut rig) = LiveRig::new("awareness") else {
        return;
    };
    rig.wait("the thrasher in view", |r| places(r).is_some());
    let (creature, player) = places(&rig).unwrap();
    let start = distance(creature, player);
    eprintln!("start: creature {creature:?} player {player:?} distance {start:.2}");
    let (mut nearest, mut drawn_nearest) = (start, start);
    for second in 0..12 {
        rig.run(1.0);
        if let Some((c, p)) = places(&rig) {
            let d = distance(c, p);
            nearest = nearest.min(d);
            let drawn = rig
                .client
                .frame()
                .objects
                .iter()
                .find(|o| o.id == THRASHER)
                .map(|o| (o.cell, o.origin));
            if let Some(drawn) = drawn {
                drawn_nearest = drawn_nearest.min(distance(drawn, p));
            }
            eprintln!("{second}s: creature {c:?} distance {d:.2} drawn {drawn:?}");
        }
    }
    eprintln!("nearest {nearest:.2}, drawn nearest {drawn_nearest:.2}");
    assert!(
        nearest < start - 1.0,
        "the thrasher did not close in ({start:.2} -> {nearest:.2})"
    );
    assert!(
        drawn_nearest < nearest + 1.0,
        "the client drew the thrasher no nearer than {drawn_nearest:.2} m while the server had it at {nearest:.2} m"
    );
}
