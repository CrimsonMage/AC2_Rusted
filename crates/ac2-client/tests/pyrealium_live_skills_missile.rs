//! A spell's missile against a running Pyrealium server (see `tests/live/mod.rs`): the human starter
//! character trains Flame Bolt, wields its stave and casts the bolt at the Banderling Thrasher
//! `scripts/pyrealium-live-test.sh` places at the west end of the start room. The bolt is the client's own story missile: the server's `DoStory` on the caster
//! names the skill and a fire-missile hook (`StoryHookData` type 3) at the target, the player's
//! script names the missile (`gmCWInterface::GetMissileInformationFromWSL`: Flame Bolt's attack
//! hook 1 fires `fx_proj_energybolt_base_fire`, entity `0x47000D04`), and
//! `StoryManager::FireMissile` makes, places and fires it; its missile manager flies it to the
//! creature, where it detonates and is destroyed (`CObjectMaint::FadeAndDestroyObject`). Writes `target/ac2-snapshots/live_flame_bolt_*.png`.
//!
//! Run with `PYREALIUM_LIVE_TEST=pyrealium_live_skills_missile scripts/pyrealium-live-test.sh` (the skills
//! profile: the development kit's skill rules and experience to train with).

#![cfg(feature = "window")]

mod live;

use std::path::PathBuf;

use ac2_client::Command;
use ac2_net::protocol::ShortcutInfo;
use ac2_weenie::{
    ActiveAction, GameplayNotice, SessionEvent, SkillRequest, ACTION_ABOUT_FACE,
    ACTION_CLASS_MOVEMENT, ACTION_FORWARD,
};
use live::{LiveRig, SIZE};

/// Flame Bolt (`hum_ma_flamebolt`), after the starter magic skill 0x60.
const FLAME_BOLT: u32 = 0x59;
/// The missile Flame Bolt's script names (`fx_proj_energybolt_base_fire.entity`).
const BOLT_ENTITY: u32 = 0x4700_0D04;
/// The human starter stave (Flame Bolt's record asks for implement 0x40000 in the right hand).
const STAVE_ENTITY: u32 = 0x4700_07E0;
/// `ShortcutInfo::_type` of a skill.
const SKILL_SHORTCUT: u32 = 0x4000_0001;
const PRIMARY_HAND: u32 = 0x40;
/// The Banderling Thrasher `scripts/pyrealium-live-test.sh` places for this file.
const THRASHER: u64 = 0x8140_0000_0003_0061;
/// The weenie type bits of a monster (`gmSelectionUI::TargetTypeVsKey`'s `0x4F`).
const MONSTER_BITS: i32 = 0x4F;

fn rig_for(name: &str) -> Option<LiveRig> {
    if std::env::var_os("PYREALIUM_LIVE_TRAINING_KIT").is_none() {
        eprintln!("the server has no skill rules: run through scripts/pyrealium-live-test.sh with PYREALIUM_LIVE_TEST=pyrealium_live_skills_missile");
        return None;
    }
    // A sound device: the client starts the FX particle systems (the bolt's) only with one.
    LiveRig::new_with_sound(name)
}

fn write_png(name: &str, rgba: &[u8]) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).expect("snapshot directory");
    let file = std::fs::File::create(dir.join(name)).expect("png file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), SIZE.0, SIZE.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(rgba)
        .unwrap();
}

/// The world as the window draws it now, to `name`.
fn snapshot(rig: &mut LiveRig, target: &ac2_gpu::FrameTarget, name: &str) {
    let frame = rig.client.frame();
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    rig.view.settle(&rig.gpu, &frame, aspect);
    if rig
        .view
        .prepare(&rig.gpu, &frame, aspect, frame.server_time)
    {
        rig.view
            .render(&rig.gpu, &target.color_view, &target.depth_view, true);
        let rgba = target.read_rgba(&rig.gpu.device, &rig.gpu.queue);
        write_png(name, &rgba);
    }
}

/// The monsters in the player's world.
fn monsters(rig: &LiveRig) -> Vec<u64> {
    let player = rig.client.session().player();
    rig.client
        .session()
        .objects()
        .iter()
        .filter(|o| Some(o.id) != player)
        .filter(|o| {
            o.qualities
                .inq_int(0x14)
                .is_some_and(|t| t & MONSTER_BITS == MONSTER_BITS)
        })
        .map(|o| o.id)
        .collect()
}

/// The monsters within 50 m, nearest first.
fn foes(rig: &LiveRig) -> Vec<(f32, u64)> {
    let frame = rig.client.frame();
    let Some(player) = frame.player.as_ref().map(|p| p.origin) else {
        return Vec::new();
    };
    let monsters = monsters(rig);
    let mut foes: Vec<(f32, u64)> = frame
        .objects
        .iter()
        .filter(|o| monsters.contains(&o.id))
        .map(|o| (distance(o.origin, player), o.id))
        .filter(|&(d, _)| d < 50.0)
        .collect();
    foes.sort_by(|a, b| a.0.total_cmp(&b.0));
    // The Thrasher the script placed at the room's west end first: the creatures that come to the
    // player stand too close for the bolt to fly.
    foes.sort_by_key(|&(_, id)| id != THRASHER);
    foes
}

/// Frames until `ready` holds (it may ask the script queries, which need `&mut`), at most 60 s.
fn wait_mut(rig: &mut LiveRig, what: &str, mut ready: impl FnMut(&mut LiveRig) -> bool) {
    for _ in 0..60 * 60 {
        if ready(rig) {
            return;
        }
        rig.frame(&[]);
    }
    panic!("{what}");
}

/// A press of the about-face key, then time for the turn.
fn about_face(rig: &mut LiveRig) {
    rig.held = vec![ActiveAction {
        class: ACTION_CLASS_MOVEMENT,
        enum_value: ACTION_ABOUT_FACE,
        map_extent: 1.0,
        extent: 1.0,
    }];
    rig.run(0.1);
    rig.held.clear();
    rig.run(2.0);
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn ready(rig: &mut LiveRig) {
    wait_mut(rig, "the player's script", |r| {
        r.client.skill_credits().is_ok()
    });
    rig.wait("experience and skills", |r| {
        let progress = r.client.frame().progress;
        progress.experience.available_xp > 0 && !progress.skills.skills.is_empty()
    });
}

/// Trains Flame Bolt, puts it on the bar and wields the stave; the bar slot.
fn flame_bolt_ready(rig: &mut LiveRig) -> u32 {
    ready(rig);
    rig.command(Command::Skill(SkillRequest::Train(FLAME_BOLT)));
    wait_mut(rig, "Flame Bolt trained", |r| {
        r.client.is_skill_trained(FLAME_BOLT) == Ok(true)
    });
    let bar = rig.client.shortcut_array().unwrap();
    let slot = bar
        .iter()
        .position(|s| matches!(s, ShortcutInfo::Empty(_)))
        .expect("an empty shortcut slot") as u32;
    rig.client
        .set_shortcut(
            slot,
            ShortcutInfo::Enum {
                kind: SKILL_SHORTCUT,
                value: FLAME_BOLT,
            },
        )
        .unwrap();
    rig.wait("the stave in the pack", |r| {
        r.client
            .session()
            .objects()
            .iter()
            .any(|o| o.entity == Some(STAVE_ENTITY))
    });
    let stave = rig
        .client
        .session()
        .objects()
        .iter()
        .find(|o| o.entity == Some(STAVE_ENTITY))
        .map(|o| o.id)
        .unwrap();
    rig.command(Command::EquipItem {
        item: stave,
        location: PRIMARY_HAND,
    });
    rig.wait("the stave wielded", |r| {
        r.client
            .frame()
            .inventory
            .equipped
            .iter()
            .any(|&(location, item)| location == PRIMARY_HAND && item == stave)
    });
    slot
}

#[test]
fn flame_bolt_flies_to_the_creature_and_detonates() {
    let Some(mut rig) = rig_for("Boltcast") else {
        return;
    };
    let slot = flame_bolt_ready(&mut rig);
    rig.wait("a creature in the world", |r| !foes(r).is_empty());
    // The Thrasher stands at the west end of the start room, which the character faces. The
    // bolt starts a few metres in front of the caster (the script's source offset) and toward
    // the target only within 45 degrees of the caster's heading
    // (`CMissileManager::InitHeadingTowardTarget`), so the character turns about, walks to the
    // room's east end and turns back, leaving the bolt room to fly.
    about_face(&mut rig);
    let here = rig.origin();
    rig.walk_towards(
        &[ACTION_FORWARD],
        [here[0] + 5.0, here[1], here[2]],
        0.5,
        6.0,
    );
    about_face(&mut rig);
    let target = ac2_gpu::FrameTarget::new(&rig.gpu.device, SIZE.0, SIZE.1);

    let mut seen_missiles = Vec::new();
    let mut missile_entity = None;
    let mut flights: Vec<(u64, [f32; 3])> = Vec::new();
    let mut drawn = false;
    let mut shots = 0;
    let mut creature = 0;
    // The nearest creature first; one the server refuses as no creature (a guard) is dropped, one
    // out of sight behind a wall is tried again once the others have been, as the creatures move
    // (the training creatures come to the hurt player).
    let mut refused: Vec<u64> = Vec::new();
    'cast: for attempt in 0..16 {
        let candidates: Vec<(f32, u64)> = foes(&rig)
            .into_iter()
            .filter(|(_, id)| !refused.contains(id))
            .collect();
        eprintln!("[{attempt}] foes {candidates:x?}");
        let Some(&(_, foe)) = candidates.get(attempt % candidates.len().max(1)) else {
            rig.run(2.0);
            continue;
        };
        creature = foe;
        rig.command(Command::Select(Some(creature)));
        rig.run(0.5);
        let from = rig.seen.len();
        assert_eq!(rig.client.use_shortcut(slot), Ok(1), "the cast is sent");
        for _ in 0..60 * 4 {
            rig.frame(&[]);
            for (id, holder, at, flying) in rig.client.story_missiles() {
                // A story missile is made from its entity's DID (`CreateMissile`).
                let entity = rig
                    .client
                    .frame()
                    .objects
                    .iter()
                    .find(|o| o.id == id)
                    .and_then(|o| o.visual);
                missile_entity = missile_entity.or(entity);
                if !seen_missiles.contains(&id) {
                    seen_missiles.push(id);
                    let target_at = rig
                        .client
                        .frame()
                        .objects
                        .iter()
                        .find(|o| o.id == creature)
                        .map(|o| (o.cell, o.origin));
                    eprintln!(
                        "story missile {id:#x} (entity {entity:x?}) holder {holder:?}, target at {target_at:x?}"
                    );
                }
                if let (Some(at), true) = (at, flying) {
                    let o = at.frame.origin;
                    let moved = flights
                        .last()
                        .is_none_or(|&(_, last)| last != [o.x, o.y, o.z]);
                    flights.push((id, [o.x, o.y, o.z]));
                    // Mid-flight: the bolt moved since the last frame.
                    if holder.is_none() && shots < 4 && moved {
                        snapshot(&mut rig, &target, &format!("live_flame_bolt_{shots}.png"));
                        shots += 1;
                        drawn = rig
                            .client
                            .frame()
                            .objects
                            .iter()
                            .any(|object| object.id == id);
                    }
                }
            }
        }
        for e in &rig.seen[from..] {
            match e {
                SessionEvent::ObjectAction { object, action } => {
                    let text = format!("{action:?}");
                    eprintln!("[{attempt}] {object:#x} {}", &text[..text.len().min(400)]);
                }
                SessionEvent::ObjectAdded(id) | SessionEvent::ObjectRemoved(id)
                    if id >> 56 == 0x81 =>
                {
                    eprintln!("[{attempt}] {e:?}");
                }
                SessionEvent::Notice(GameplayNotice::DisplayStringInfo { text, .. }) => {
                    eprintln!("[{attempt}] said {:?}", text.literal);
                    if text
                        .literal
                        .as_deref()
                        .is_some_and(|t| t.contains("Select a creature"))
                    {
                        refused.push(creature);
                    }
                }
                _ => {}
            }
        }
        if !flights.is_empty() {
            break 'cast;
        }
        // The cast was refused (a guard, the target out of reach) or made no missile: wait out
        // the skill's reset and try the next creature.
        rig.run(3.0);
    }
    assert!(!seen_missiles.is_empty(), "the cast made a story missile");
    assert_eq!(
        missile_entity,
        Some(BOLT_ENTITY),
        "the bolt is Flame Bolt's missile"
    );
    assert!(!flights.is_empty(), "the bolt flies");
    assert!(drawn, "the flying bolt is in the drawn objects");
    // It flies from the caster to the creature: the bolt's path closes on it.
    let creature_at = rig
        .client
        .frame()
        .objects
        .iter()
        .find(|o| o.id == creature)
        .map(|o| o.origin)
        .expect("the creature is drawn");
    let path: Vec<f32> = flights
        .iter()
        .map(|&(_, at)| distance(at, creature_at))
        .collect();
    eprintln!("bolt distances to the creature {path:?}");
    let (first, nearest) = (path[0], path.iter().copied().fold(f32::MAX, f32::min));
    assert!(
        nearest < first && nearest < 3.5,
        "the bolt closes on the creature: {path:?}"
    );
    // It detonates there and is destroyed after its fade (`CObjectMaint::FadeAndDestroyObject`),
    // well before its `MISSILE_TIMEOUT_TIME` of 8 s.
    assert!(
        rig.until(3.0, |r| r.client.story_missiles().is_empty()),
        "the bolt is gone after it detonates: {:?}",
        rig.client.story_missiles()
    );
}
