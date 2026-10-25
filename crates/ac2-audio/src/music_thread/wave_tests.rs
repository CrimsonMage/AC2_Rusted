//! A wave segment in the queue (an `EncodedWave` as the segment's wave track): it starts and ends
//! on the queue like any segment, repeats as the current primary does, is replaced like the other
//! primaries, and honours its stop boundary on the performance's default clock (120 quarter notes
//! a minute, 4/4: a measure is 2 s). Synthetic segments, so no DAT is needed.

use std::collections::HashMap;
use std::sync::Arc;

use super::tests::{info, run, Calls};
use super::{MusicThread, SegmentKind};
use crate::music::library::LoadedMusic;
use crate::music::{MusicObject, WaveSegment};
use crate::sink::Boundary;

fn wave(seconds: f64, looping: bool) -> Arc<LoadedMusic> {
    Arc::new(LoadedMusic {
        segment: MusicObject::Wave(WaveSegment { seconds, looping }),
        objects: HashMap::new(),
        missing: 0,
    })
}

fn count(calls: &Calls, name: &str, handle: u32) -> usize {
    calls
        .0
        .iter()
        .filter(|(n, h)| *n == name && *h == handle)
        .count()
}

#[test]
fn a_wave_segment_has_no_notes_and_starts_and_ends_on_the_queue() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    assert!(thread.play(info(SegmentKind::Primary, 7, &wave(1.0, false))));
    let events = run(&mut thread, 0.0, &mut calls);
    assert!(events.is_empty(), "a wave sends no notes");
    assert_eq!(count(&calls, "start", 7), 1, "the pass starts");
    assert!(thread.is_playing(7));
    let events = run(&mut thread, 0.5, &mut calls);
    assert!(events.is_empty());
    assert!(thread.is_playing(7), "still in its pass");
}

#[test]
fn a_repeated_wave_pass_keeps_its_handle_until_it_is_stopped() {
    // A non-looping primary repeats while nothing waits (`OnMusicStopping`'s `m_bLoopPrimary`):
    // its pass ends at one second, the repeat starts, and no end reaches the sound.
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.play(info(SegmentKind::Primary, 7, &wave(1.0, false)));
    run(&mut thread, 0.0, &mut calls);
    run(&mut thread, 1.5, &mut calls);
    assert_eq!(count(&calls, "start", 7), 2, "the pass repeats");
    assert_eq!(count(&calls, "ended", 7), 0, "the sound keeps the handle");
    assert!(thread.is_playing(7), "the repeat plays");
    assert!(thread.holds(7));
    // Retail's stop only clears the loop: the pass in progress ends at its end (2 s).
    thread.stop(7, Boundary::Immediate);
    run(&mut thread, 1.6, &mut calls);
    assert!(thread.is_playing(7), "the pass in progress plays out");
    run(&mut thread, 2.1, &mut calls);
    assert!(!thread.is_playing(7));
    assert_eq!(count(&calls, "ended", 7), 1, "the stop ends it once");
    assert!(!thread.holds(7));
}

#[test]
fn a_queued_wave_replaces_the_copy_waiting_behind_the_current_primary() {
    // Two copies asked for while A plays: B is replaced by C (B aborts), and C takes over when A's
    // pass ends (`QueuePrimaryInternal`, `PlayQueuedPrimarySegment`).
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.play(info(SegmentKind::Primary, 1, &wave(1.0, false)));
    run(&mut thread, 0.0, &mut calls);
    thread.play(info(SegmentKind::Primary, 2, &wave(0.5, false)));
    thread.play(info(SegmentKind::Primary, 3, &wave(0.5, false)));
    run(&mut thread, 0.2, &mut calls);
    assert_eq!(count(&calls, "ended", 2), 1, "the replaced copy aborts");
    assert_eq!(count(&calls, "start", 2), 0, "it never starts");
    assert!(thread.is_playing(1), "A plays out its pass");
    assert!(!thread.is_playing(3), "C waits");
    run(&mut thread, 1.1, &mut calls);
    assert_eq!(count(&calls, "start", 3), 1, "C takes over at A's end");
    assert!(thread.is_playing(3));
    assert_eq!(
        count(&calls, "ended", 1),
        1,
        "A's pass ended, and it is released"
    );
}

#[test]
fn a_looping_wave_stopped_at_a_measure_plays_its_pass_out() {
    // Looping, ten seconds; stopped at 0.3 s, the pass in progress runs to its end (10 s), not to
    // the measure: retail's stop does not read the boundary of a primary.
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.play(info(SegmentKind::Primary, 7, &wave(10.0, true)));
    run(&mut thread, 0.0, &mut calls);
    run(&mut thread, 0.3, &mut calls);
    thread.stop(7, Boundary::Measure);
    run(&mut thread, 9.9, &mut calls);
    assert!(thread.is_playing(7), "it plays out to the end of the pass");
    assert_eq!(count(&calls, "ended", 7), 0);
    run(&mut thread, 10.1, &mut calls);
    assert!(!thread.is_playing(7), "it ends at the end of the pass");
    assert_eq!(count(&calls, "ended", 7), 1);
}

#[test]
fn a_looping_wave_stopped_immediately_plays_its_pass_out() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.play(info(SegmentKind::Primary, 7, &wave(10.0, true)));
    run(&mut thread, 0.0, &mut calls);
    run(&mut thread, 0.3, &mut calls);
    thread.stop(7, Boundary::Immediate);
    run(&mut thread, 0.35, &mut calls);
    assert!(
        thread.is_playing(7),
        "an immediate stop of a primary is not a cut"
    );
    run(&mut thread, 10.1, &mut calls);
    assert!(!thread.is_playing(7));
    assert_eq!(count(&calls, "ended", 7), 1);
}
