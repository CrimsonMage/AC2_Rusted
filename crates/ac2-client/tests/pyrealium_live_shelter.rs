//! Rescue the Shelter Hold against a running Pyrealium server (`PYREALIUM_LIVE`, see `tests/live/mod.rs`) set
//! up as the user's server is: the training kit's placements stripped, the captured 0x0298 Shelter
//! mirrored into the human start instance 0x0299 and Pyrealium's `human-shelter-world.tsv` added
//! (`scripts/pyrealium-server.sh` with `PYREALIUM_TRAINING_WORLD=0`). A new human character plays the whole
//! chain as a player does: it walks the Shelter's halls (keys only: forward and the two turns),
//! speaks to the Shelter Guard, Lieutenant and Sentry, fights Drudge Diggers until one leaves the
//! Imbued Digging Tool, trades it to the Shelter Scout for three potions, fights Drudge Sentinels
//! until one leaves the Menhir Glyph, climbs the rotunda stairs and uses the glyph on the Menhir
//! Portal, speaks to the Advance Shelter Scout in the Menhir Shard room, walks down the passage
//! and uses the Menhir Shard, and arrives in Dereth with the quest on the journal's Finished page.
//! Each NPC's line is checked by its DAT string (table and key of `guardshelter.stt` and the
//! others); the rewards by the experience the client shows.
//!
//! Run with `PYREALIUM_LIVE_TEST=pyrealium_live_shelter PYREALIUM_TRAINING_WORLD=0 scripts/pyrealium-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_ui::ToolbarChild;
use ac2_weenie::{
    ActiveAction, GameplayNotice, SessionEvent, ACTION_CLASS_MOVEMENT, ACTION_FORWARD,
    ACTION_TURN_NEGATIVE, ACTION_TURN_POSITIVE,
};
use live::LiveRig;

/// The Shelter's people and things (entity DIDs: `wsl_guardshelter.entity` and the others).
const GUARD: u32 = 0x4700_157D;
const LIEUTENANT: u32 = 0x4700_157E;
const ADVANCE_SCOUT: u32 = 0x4700_157F;
const SCOUT: u32 = 0x4700_1580;
const SENTRY: u32 = 0x4700_1581;
const STONE_7: u32 = 0x4700_1579;
const GLYPH: u32 = 0x4700_1582;
const TOOL: u32 = 0x4700_1587;
const POTIONS: [u32; 3] = [0x4700_1583, 0x4700_1584, 0x4700_1585];
const DIGGER: u32 = 0x4700_158B;
const SENTINEL: u32 = 0x4700_158C;
const PINK_PORTAL: u32 = 0x4700_1451;
const SHARD: u32 = 0x4700_079A;

/// The lines each step must show: (string table, key) of the DAT's `*shelter.stt`.
const GUARD_START: (u32, u32) = (0x2500_1648, 0x2824_FD4C);
const LIEUTENANT_START: (u32, u32) = (0x2500_1649, 0x8B6B_186F);
const SENTRY_START: (u32, u32) = (0x2500_164C, 0x8B6B_1869);
const SCOUT_TOOL: (u32, u32) = (0x2500_164B, 0x11DE_D2D4);
const ADVANCE_END: (u32, u32) = (0x2500_164A, 0x8B6B_1863);
const STONE_7_TEXT: (u32, u32) = (0x2500_1644, 0x6C45_9364);

/// The human start instance's Menhir Shard room and Dereth's Shelter exit (Pyrealium `shelter.rs`).
const SHARD_ROOM: u32 = 0x0299_014D;
const DERETH: u32 = 0x74BA_0008;

/// The walk through the Shelter's halls: floor points of AC2_Rusted's physics, joined by a search
/// over the dungeon's floor samples that keeps 1.9 m from every object standing still (the
/// creatures move) (`ac2-physics`
/// probe of block 0x0299 on the 7230 DATs). From the start room west past the Guard to the
/// Lieutenant, north through the training halls and the tunnel, through the large room and up the
/// rotunda stairs to the Menhir Portal's arch.
const ROUTE: [[f32; 2]; 48] = [
    [59.0, 240.0],
    [50.0, 240.0],
    [45.0, 242.0],
    [42.0, 242.0],
    [28.0, 241.0],
    [-8.0, 241.0],
    [-10.0, 240.0],
    [-4.6, 243.2],
    [-0.9, 247.0],
    [-0.6, 263.2],
    [1.2, 275.0],
    [2.0, 305.9],
    [-0.8, 316.0],
    [-0.8, 323.0],
    [0.9, 326.0],
    [0.9, 330.5],
    [0.9, 336.0],
    [0.4, 341.0],
    [0.4, 353.0],
    [1.2, 358.0],
    [1.2, 368.0],
    [0.0, 377.0],
    [0.0, 381.0],
    [0.6, 386.0],
    [1.5, 388.6],
    [4.8, 390.0],
    [13.4, 389.7],
    [17.1, 392.0],
    [20.0, 398.0],
    [21.0, 412.0],
    [22.0, 423.1],
    [20.5, 425.5],
    [22.5, 430.0],
    [23.6, 434.9],
    [34.0, 445.0],
    [43.9, 454.5],
    [45.0, 459.5],
    [44.8, 466.0],
    [44.8, 474.0],
    [45.0, 503.0],
    [45.0, 540.0],
    [80.1, 556.9],
    [82.5, 558.7],
    [85.0, 571.0],
    [85.0, 614.8],
    [96.5, 630.0],
    [132.0, 631.0],
    [143.0, 631.0],
];
/// Where along `ROUTE` each NPC is spoken to (the index of the route point next to it).
const AT_GUARD: usize = 1;
const AT_LIEUTENANT: usize = 6;
const AT_STONE_7: usize = 23;
const AT_SENTRY: usize = 29;
const AT_DIGGERS: usize = 34;
const AT_SCOUT: usize = 39;
const AT_PORTAL: usize = 47;
/// The shard room: the arrival, the Advance Shelter Scout, and the passage to the shard
/// (its middle runs at x 1 from the room to the shard's alcove).
const PASSAGE: [[f32; 2]; 4] = [[1.0, 10.0], [1.0, 60.0], [1.0, 110.0], [1.0, 131.0]];

fn hold(actions: &[u32]) -> Vec<ActiveAction> {
    actions
        .iter()
        .map(|&enum_value| ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value,
            map_extent: 1.0,
            extent: 1.0,
        })
        .collect()
}

/// The player's facing on the ground: the body's +y axis turned by its rotation (w, x, y, z).
fn facing(rig: &LiveRig) -> [f32; 2] {
    let [w, x, y, z] = rig.client.frame().player.as_ref().unwrap().rotation;
    [2.0 * (x * y - w * z), 1.0 - 2.0 * (x * x + z * z)]
}

fn cell(rig: &LiveRig) -> u32 {
    rig.client.frame().player.as_ref().map_or(0, |p| p.cell)
}

/// The signed angle (radians, counter-clockwise positive) from the facing to the way to `to`.
fn bearing(rig: &LiveRig, to: [f32; 2]) -> f32 {
    let me = rig.origin();
    let f = facing(rig);
    let (dx, dy) = (to[0] - me[0], to[1] - me[1]);
    (f[0] * dy - f[1] * dx).atan2(f[0] * dx + f[1] * dy)
}

/// Which turn key turns the body counter-clockwise: the sign of the turn `TURN_POSITIVE` makes.
fn calibrate(rig: &mut LiveRig) -> bool {
    let before = facing(rig);
    rig.held = hold(&[ACTION_TURN_POSITIVE]);
    rig.run(0.4);
    rig.held.clear();
    rig.run(0.3);
    let after = facing(rig);
    before[0] * after[1] - before[1] * after[0] > 0.0
}

/// A live Drudge of `entities` within `range` metres, nearest first.
fn drudge_near(rig: &LiveRig, entities: &[u32], range: f32) -> Option<u64> {
    let me = rig.origin();
    let session = rig.client.session();
    rig.client
        .frame()
        .objects
        .iter()
        .filter(|o| o.in_world)
        .filter_map(|o| {
            let x = session.objects().get(o.id)?;
            let alive = x.qualities.inq_int(0x102).is_none_or(|h| h > 0);
            (alive && x.entity.is_some_and(|e| entities.contains(&e)))
                .then(|| (o.id, (o.origin[0] - me[0]).hypot(o.origin[1] - me[1])))
        })
        .filter(|(_, d)| *d <= range)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// The server's notices since event `from`.
fn notices(rig: &LiveRig, from: usize) -> impl Iterator<Item = &ac2_net::protocol::StringInfo> {
    rig.seen[from..].iter().filter_map(|e| match e {
        SessionEvent::Notice(GameplayNotice::DisplayStringInfo { text, .. })
        | SessionEvent::Notice(GameplayNotice::DisplayMessage { text, .. }) => Some(text),
        _ => None,
    })
}

fn said(rig: &LiveRig, from: usize, line: (u32, u32)) -> bool {
    notices(rig, from).any(|t| (t.table_id, t.string_id) == line)
}

fn said_text(rig: &LiveRig, from: usize, part: &str) -> bool {
    notices(rig, from).any(|t| t.literal.as_deref().is_some_and(|l| l.contains(part)))
}

fn lines(rig: &LiveRig, from: usize) -> Vec<String> {
    notices(rig, from)
        .map(|t| {
            t.literal
                .clone()
                .unwrap_or_else(|| format!("{:08x}/{:08x}", t.table_id, t.string_id))
        })
        .collect()
}

fn experience(rig: &LiveRig) -> u64 {
    rig.client.frame().progress.experience.total_xp
}

/// The id of an item in the player's containers (the session's inventory), by entity DID.
fn held_item(rig: &LiveRig, entity: u32) -> Option<u64> {
    let session = rig.client.session();
    session
        .inventory()
        .containers()
        .flat_map(|(_, c)| c.items.iter().copied().collect::<Vec<_>>())
        .find(|id| {
            session
                .objects()
                .get(*id)
                .is_some_and(|o| o.entity == Some(entity))
        })
}

struct Walker {
    ccw_positive: bool,
}

impl Walker {
    /// Runs to `to` with the keys (forward, turning on the way) until within `within` metres.
    /// Drudges that come within 6 m on the way are fought first.
    fn go(&self, rig: &mut LiveRig, to: [f32; 2], within: f32) {
        self.walk(rig, to, within, true);
    }

    /// `go`, fighting the Drudges met on the way only when `fights`.
    fn walk(&self, rig: &mut LiveRig, to: [f32; 2], within: f32, fights: bool) {
        let mut best = f32::MAX;
        let mut stalled = 0.0;
        loop {
            let me = rig.origin();
            let left = (to[0] - me[0]).hypot(to[1] - me[1]);
            // A route point a wall or a body keeps the player from is close enough within 2 m.
            if left <= within || (left <= 2.0 && stalled > 2.0) {
                break;
            }
            if fights && drudge_near(rig, &[DIGGER, SENTINEL], 6.0).is_some() {
                rig.held.clear();
                self.fight(rig, &[DIGGER, SENTINEL], 6.0);
                best = f32::MAX;
                continue;
            }
            let angle = bearing(rig, to);
            let turn = if (angle > 0.0) == self.ccw_positive {
                ACTION_TURN_POSITIVE
            } else {
                ACTION_TURN_NEGATIVE
            };
            rig.held = if angle.abs() > 0.5 {
                hold(&[turn])
            } else if angle.abs() > 0.08 {
                hold(&[ACTION_FORWARD, turn])
            } else {
                hold(&[ACTION_FORWARD])
            };
            rig.run(0.05);
            if left < best - 0.05 {
                best = left;
                stalled = 0.0;
            } else {
                stalled += 0.05;
            }
            assert!(
                stalled < 12.0,
                "stuck at {me:?} (cell {:08x}) on the way to {to:?}",
                cell(rig)
            );
        }
        rig.held.clear();
        rig.run(0.3);
    }

    /// Fights every live Drudge of `entities` within `range` metres until none is left there.
    fn fight(&self, rig: &mut LiveRig, entities: &[u32], range: f32) {
        let mut fought = 0;
        while let Some(target) = drudge_near(rig, entities, range) {
            fought += 1;
            assert!(fought <= 12, "too many fights");
            let before = rig.seen.len();
            let fallen = |r: &LiveRig| {
                said_text(r, before, "defeated")
                    || r.client
                        .session()
                        .objects()
                        .get(target)
                        .is_none_or(|x| x.qualities.inq_int(0x102).is_some_and(|h| h <= 0))
            };
            rig.command(Command::Select(Some(target)));
            rig.run(0.5);
            let mut dead = false;
            for _ in 0..12 {
                // Into reach (3 m), then the attack until it falls or is out of reach again.
                if let Some(at) = rig.place_of(target) {
                    self.walk(rig, [at[0], at[1]], 2.2, false);
                }
                let from = rig.seen.len();
                rig.command(Command::SetAttacking(true));
                rig.until(15.0, |r| {
                    fallen(r) || said_text(r, from, "out of attack range")
                });
                dead = fallen(rig);
                if dead {
                    break;
                }
                rig.command(Command::SetAttacking(false));
                rig.run(0.3);
            }
            rig.command(Command::SetAttacking(false));
            rig.run(1.0);
            let tail = lines(rig, before);
            eprintln!(
                "FIGHT {target:x} dead {dead}: {:?}",
                tail
            );
            assert!(dead, "the Drudge {target:x} did not fall");
        }
    }

    /// Walks `ROUTE` from point `from` to point `to` (inclusive).
    fn route(&self, rig: &mut LiveRig, from: usize, to: usize) {
        for point in &ROUTE[from..=to] {
            self.go(rig, *point, 0.7);
        }
    }
}

/// Uses the NPC created from `entity` (walking up to it first) and waits for `line`.
fn speak(rig: &mut LiveRig, walker: &Walker, entity: u32, line: (u32, u32)) {
    let npc = rig
        .object_of(entity)
        .unwrap_or_else(|| panic!("{entity:08x} in view"));
    let at = rig.place_of(npc).expect("the NPC's place");
    walker.go(rig, [at[0], at[1]], 2.6);
    let from = rig.seen.len();
    rig.command(Command::Use(npc));
    assert!(
        rig.until(10.0, |r| said(r, from, line)),
        "{entity:08x} did not say {line:x?}: {:?}",
        lines(rig, from)
    );
    rig.run(1.0);
    eprintln!("SPOKE {entity:08x}: {:?}", lines(rig, from));
}

#[test]
fn rescue_the_shelter_hold() {
    let Some(mut rig) = LiveRig::new("Sheltered") else {
        return;
    };
    rig.wait("the Shelter Guard", |r| r.object_of(GUARD).is_some());
    assert_eq!(cell(&rig) >> 16, 0x0299, "the human start instance");
    rig.panel(ToolbarChild::Quest, true);
    let walker = Walker {
        ccw_positive: calibrate(&mut rig),
    };

    // The Guard starts the quest; the Lieutenant and the Sentry move it on.
    walker.route(&mut rig, 0, AT_GUARD);
    speak(&mut rig, &walker, GUARD, GUARD_START);
    walker.route(&mut rig, AT_GUARD, AT_LIEUTENANT);
    speak(&mut rig, &walker, LIEUTENANT, LIEUTENANT_START);
    walker.route(&mut rig, AT_LIEUTENANT, AT_STONE_7);
    speak(&mut rig, &walker, STONE_7, STONE_7_TEXT);
    walker.route(&mut rig, AT_STONE_7, AT_SENTRY);
    speak(&mut rig, &walker, SENTRY, SENTRY_START);

    // A Drudge Digger leaves the Imbued Digging Tool.
    walker.route(&mut rig, AT_SENTRY, AT_DIGGERS);
    for point in &ROUTE[AT_DIGGERS..=AT_SCOUT] {
        if held_item(&rig, TOOL).is_some() {
            break;
        }
        walker.fight(&mut rig, &[DIGGER], 8.0);
        walker.go(&mut rig, *point, 1.2);
    }
    walker.fight(&mut rig, &[DIGGER], 8.0);
    assert!(
        rig.until(5.0, |r| held_item(r, TOOL).is_some()),
        "a Drudge Digger left the Imbued Digging Tool"
    );

    // The Shelter Scout takes it: three potions and 40 experience.
    walker.route(&mut rig, AT_DIGGERS, AT_SCOUT);
    let before = experience(&rig);
    speak(&mut rig, &walker, SCOUT, SCOUT_TOOL);
    assert!(
        rig.until(10.0, |r| POTIONS.iter().all(|p| held_item(r, *p).is_some())),
        "the Scout's three potions"
    );
    assert!(held_item(&rig, TOOL).is_none(), "the tool was given");
    assert!(rig.until(5.0, |r| experience(r) >= before + 40), "40 XP");
    let scout_xp = experience(&rig) - before;

    // A Drudge Sentinel in the large room leaves the Menhir Glyph.
    for point in &ROUTE[AT_SCOUT..=AT_PORTAL] {
        if held_item(&rig, GLYPH).is_none() {
            walker.fight(&mut rig, &[SENTINEL], 10.0);
        }
        walker.go(&mut rig, *point, 1.2);
    }
    assert!(
        held_item(&rig, GLYPH).is_some(),
        "a Drudge Sentinel left the Menhir Glyph"
    );

    // The glyph on the Menhir Portal: the Menhir Shard room.
    let portal = rig
        .client
        .session()
        .objects()
        .iter()
        .filter(|o| o.entity == Some(PINK_PORTAL))
        .map(|o| o.id)
        .min_by(|a, b| {
            let me = rig.origin();
            let d = |id: &u64| {
                rig.place_of(*id)
                    .map_or(f32::MAX, |p| (p[0] - me[0]).hypot(p[1] - me[1]))
            };
            d(a).total_cmp(&d(b))
        })
        .expect("the Menhir Portal");
    let at = rig.place_of(portal).unwrap();
    walker.go(&mut rig, [at[0], at[1]], 3.0);
    let glyph = held_item(&rig, GLYPH).unwrap();
    let from = rig.seen.len();
    rig.command(Command::UseWithTarget {
        item: glyph,
        target: portal,
    });
    assert!(
        rig.until(30.0, |r| cell(r) == SHARD_ROOM && r.client.frame().in_world),
        "the glyph's portal led to the Menhir Shard room; {:?}",
        lines(&rig, from)
    );
    rig.run(3.0);
    assert!(held_item(&rig, GLYPH).is_none(), "the glyph was used");

    // The Advance Shelter Scout ends the quest: the rest of its 105 experience.
    let before = experience(&rig);
    speak(&mut rig, &walker, ADVANCE_SCOUT, ADVANCE_END);
    assert!(rig.until(5.0, |r| experience(r) >= before + 65), "65 XP");
    rig.run(1.0);
    assert_eq!(
        scout_xp + experience(&rig) - before,
        105,
        "Rescue the Shelter Hold pays 105 XP"
    );
    let (_, finished) = rig.view.quest_journal_texts();
    assert!(
        finished
            .iter()
            .any(|t| t.contains("Rescue the Shelter Hold")),
        "the quest is finished: {finished:?}"
    );

    // Down the passage to the Menhir Shard, which leads out to Dereth.
    for point in PASSAGE {
        walker.go(&mut rig, point, 1.2);
    }
    let shard = rig.object_of(SHARD).expect("the Menhir Shard in view");
    let at = rig.place_of(shard).unwrap();
    walker.go(&mut rig, [at[0], at[1]], 5.5);
    let from = rig.seen.len();
    rig.command(Command::Use(shard));
    assert!(
        rig.until(30.0, |r| cell(r) == DERETH && r.client.frame().in_world),
        "the Menhir Shard led to Dereth; {:?}",
        lines(&rig, from)
    );
}
