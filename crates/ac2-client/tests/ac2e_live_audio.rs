//! What a player hears against a running AC2e server (`AC2E_LIVE`, see `tests/live/mod.rs`): a new
//! character in the Menhir Vault with a sound manager on a recording sink, attached before the
//! login as `bin/ac2/main.rs` attaches the speakers'. Every `SoundManager::Play` request is
//! logged (`SoundManager::take_played`, test-support) and sorted by its emitter: the player's
//! (its animations' FX: the steps), a creature's (the Idol Spawns' FX as they move and fight),
//! an ambient emitter's (`WorldRender::UpdateAmbientSounds`' environments), or none (the UI,
//! music and the portal space).
//!
//! Run with `AC2E_LIVE_TEST=ac2e_live_audio scripts/ac2e-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use std::collections::BTreeMap;

use ac2_audio::{PlaySoundData, RecordingSink, SinkCommand};
use ac2_weenie::{
    ActiveAction, ACTION_CLASS_MOVEMENT, ACTION_STRAFE_NEGATIVE, ACTION_STRAFE_POSITIVE,
};
use live::LiveRig;

/// The weenie type bits of a monster (`gmSelectionUI::TargetTypeVsKey`'s `0x4F`).
const MONSTER_BITS: i32 = 0x4F;

/// Who asked for a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Source {
    Player,
    Creature,
    Ambient,
    OtherObject,
    NoEmitter,
}

type Heard = BTreeMap<Source, BTreeMap<u32, usize>>;

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

fn source(rig: &LiveRig, data: &PlaySoundData) -> Source {
    match data.emitter {
        None => Source::NoEmitter,
        Some(id) if ac2_audio::is_ambient_emitter(id) => Source::Ambient,
        Some(id) if Some(id) == rig.client.session().player() => Source::Player,
        Some(id) if monsters(rig).contains(&id) => Source::Creature,
        Some(_) => Source::OtherObject,
    }
}

/// The requests since the last call, by source, each with its sound DIDs and how often each
/// was asked for; printed under `phase`.
fn heard(rig: &mut LiveRig, phase: &str) -> Heard {
    let played = rig.client.audio_mut().expect("audio").take_played();
    let mut out = Heard::new();
    for data in &played {
        *out.entry(source(rig, data))
            .or_default()
            .entry(data.sound)
            .or_default() += 1;
    }
    for (source, dids) in &out {
        let list: Vec<String> = dids
            .iter()
            .map(|(did, n)| format!("{did:08X}x{n}"))
            .collect();
        eprintln!("[{phase}] {source:?}: {}", list.join(" "));
    }
    out
}

fn count(sounds: &Heard, source: Source) -> usize {
    sounds.get(&source).map_or(0, |d| d.values().sum())
}

fn planar(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// A run across the vault and a stand among its creatures: the player's steps, the creatures'
/// sounds and the vault's ambience are all asked for, and the world's sounds reach the sink in
/// 3D.
#[test]
fn a_run_and_the_creatures_play_steps_creatures_and_ambience() {
    let sink = RecordingSink::new();
    let Some(mut rig) = LiveRig::new_with_audio("Soundwalk", Box::new(sink.clone())) else {
        return;
    };
    rig.run(1.0);
    let idle = heard(&mut rig, "idle");

    // A run across the vault: strafing one way, then back (the lifestone west of the start
    // stops a run forward within a second, before the run cycle reaches its first step).
    let before = rig.origin();
    let mut ran = 0.0;
    for action in [ACTION_STRAFE_NEGATIVE, ACTION_STRAFE_POSITIVE] {
        rig.held = vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: action,
            map_extent: 1.0,
            extent: 1.0,
        }];
        for _ in 0..12 {
            let from = rig.origin();
            rig.run(0.2);
            ran += planar(from, rig.origin());
            eprintln!("run at {:?}", rig.origin());
        }
    }
    rig.held.clear();
    rig.run(0.5);
    eprintln!("from {before:?}");
    let run = heard(&mut rig, "run");
    eprintln!("ran {ran:.2} m");

    rig.wait("no creature came into the world", |r| {
        !monsters(r).is_empty()
    });
    // The vault's creatures attack at once (AC2e's encounter rules); stand among them.
    rig.run(12.0);
    let attacked = heard(&mut rig, "attacked");

    let starts: Vec<bool> = sink
        .commands()
        .iter()
        .filter_map(|c| match c {
            SinkCommand::Start { spatial, .. } => Some(spatial.is_some()),
            _ => None,
        })
        .collect();
    let spatial = starts.iter().filter(|&&s| s).count();
    eprintln!("sink: {} waves started, {spatial} in 3D", starts.len());

    assert!(ran > 3.0, "the run crossed the vault ({ran} m)");
    assert!(count(&run, Source::Player) >= 2, "the player's steps");
    assert!(
        count(&attacked, Source::Creature) > 0,
        "the creatures' sounds"
    );
    let ambient: usize = [&idle, &run, &attacked]
        .iter()
        .map(|s| count(s, Source::Ambient))
        .sum();
    assert!(ambient > 0, "the vault's ambience");
    assert!(spatial > 0, "the world's sounds reach the sink in 3D");
}
