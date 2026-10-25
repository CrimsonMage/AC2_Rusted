//! The scripts on the music queue (`AC2_DAT_DIR`): a Cue's script runs at the Cue's start and
//! queues the segment its routine plays behind the Cue (`AtFinish`); that segment plays when the
//! Cue's pass ends, and a copy of the Cue requested behind it does not abort it (the performance's
//! queue, not the sound manager's one waiting copy). The checks are the queue's; nothing from the
//! DAT is stored.

use std::sync::Arc;

use ac2_db::ResourceStore;

use super::tests::{run, Calls};
use super::{MusicThread, ScriptedPlay, SegmentInfo, SegmentKind};
use crate::music::clock::MusicClock;
use crate::music::library::MusicLibrary;
use crate::music::script::bind_segment;
use crate::music::MusicObject;
use crate::synth::performance_bands;

/// `Cue_Tum_Reeds_A_DPent` (the town's): its script queues `MI_Tum_Reeds_A_DPent`.
const TOWN_CUE: u32 = 0x2400_0476;
const TOWN_SEGMENT: &str = "MI_Tum_Reeds_A_DPent";
const FIRST: u32 = 1;
const COPY: u32 = 2;

fn library() -> Option<MusicLibrary> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(MusicLibrary::new(Arc::new(ResourceStore::new(dataset))))
}

/// The Cue's play: its closure, bands and cues, as a sound manager gives them.
fn cue(library: &MusicLibrary, handle: u32, kind: SegmentKind) -> SegmentInfo {
    let music = Arc::new(library.closure(library.record(TOWN_CUE).expect("the cue")));
    SegmentInfo {
        kind,
        handle,
        bands: performance_bands(&music).into(),
        cues: bind_segment(&music),
        segment: music,
    }
}

/// The Cue's pass, in seconds (its length on its own clock).
fn cue_seconds(info: &SegmentInfo) -> f64 {
    let MusicObject::Segment(segment) = &info.segment.segment else {
        panic!("a segment");
    };
    MusicClock::from_segment(segment).seconds_at(segment.length)
}

/// The name of a segment a script started.
fn name_of(play: &ScriptedPlay) -> Option<String> {
    match &play.segment.segment {
        MusicObject::Segment(segment) => segment.name.clone(),
        _ => None,
    }
}

#[test]
fn the_cue_queues_its_segment_behind_it_at_every_groove_level() {
    let Some(library) = library() else {
        return;
    };
    for groove in [0, 9, 30] {
        let mut thread = MusicThread::new();
        thread.set_groove_level(groove);
        let mut calls = Calls::default();
        thread.use_time(0.0, Some(&mut calls));
        assert!(thread.play(cue(&library, FIRST, SegmentKind::Primary)));
        run(&mut thread, 0.5, &mut calls);
        let started = thread.take_scripted();
        let primary: Vec<&ScriptedPlay> = started
            .iter()
            .filter(|p| p.kind == SegmentKind::Primary)
            .collect();
        assert_eq!(primary.len(), 1, "one primary at groove {groove}");
        assert_eq!(
            name_of(primary[0]).as_deref(),
            Some(TOWN_SEGMENT),
            "the routine's branch at groove {groove}"
        );
        assert!(thread.holds(primary[0].handle), "queued behind the cue");
        assert!(
            !thread.is_playing(primary[0].handle),
            "not before the cue ends"
        );
    }
}

#[test]
fn the_queued_segment_plays_at_the_cue_end_and_a_copy_behind_it_does_not_abort_it() {
    let Some(library) = library() else {
        return;
    };
    let mut thread = MusicThread::new();
    thread.set_groove_level(30);
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    let first = cue(&library, FIRST, SegmentKind::Primary);
    let end = cue_seconds(&first);
    assert!(thread.play(first));
    run(&mut thread, 0.5, &mut calls);
    let scripted = thread
        .take_scripted()
        .into_iter()
        .find(|p| p.kind == SegmentKind::Primary)
        .expect("the script queued its segment");
    // The sound manager asks for another copy of the Cue while the scripted one waits.
    assert!(thread.play(cue(&library, COPY, SegmentKind::Primary)));
    assert!(
        thread.holds(scripted.handle),
        "the copy does not abort the scripted segment"
    );
    run(&mut thread, end - 0.1, &mut calls);
    assert!(thread.is_playing(FIRST), "the cue plays its pass");
    run(&mut thread, end + 0.5, &mut calls);
    assert!(!thread.is_playing(FIRST), "the cue's pass has ended");
    assert!(
        thread.is_playing(scripted.handle),
        "the scripted segment plays next, before the copy"
    );
    assert!(!thread.is_playing(COPY), "the copy waits behind it");
    assert!(thread.holds(COPY));
}

#[test]
fn the_queued_segments_triggers_start_their_segments_as_secondaries_after_the_cue() {
    let Some(library) = library() else {
        return;
    };
    let mut thread = MusicThread::new();
    thread.set_groove_level(30);
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    let first = cue(&library, FIRST, SegmentKind::Primary);
    let end = cue_seconds(&first);
    assert!(thread.play(first));
    // The Cue's pass: its script queues the segment, which does not start before it ends.
    run(&mut thread, end - 0.1, &mut calls);
    assert!(
        thread
            .take_scripted()
            .iter()
            .all(|p| p.kind == SegmentKind::Primary),
        "no trigger starts during the Cue's pass"
    );
    // The segment starts at the Cue's end; its own triggers start the arrangement and the band.
    run(&mut thread, end + 1.0, &mut calls);
    let triggered = thread.take_scripted();
    assert!(
        triggered.len() >= 2,
        "the segment's triggers start its segments: {}",
        triggered.len()
    );
    assert!(
        triggered.iter().all(|p| p.kind == SegmentKind::Secondary),
        "a trigger starts a secondary segment"
    );
}
