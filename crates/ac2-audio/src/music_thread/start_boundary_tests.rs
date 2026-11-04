//! A secondary's start boundary (`DirectXPlayingSound::PlayInternal`'s `dwFlags` to
//! `PlaySegmentEx` with a start time of 0), on synthetic segments: DirectMusic resolves it against
//! the primary playing (`DMUS_SEGF_FLAGS`), and starts at once with no primary, or with no marker
//! left for a marker start (retail passes no other resolution flag).

use std::sync::Arc;

use super::tests::{run, seq, sequence, Calls, QUARTER};
use super::{MusicThread, SegmentInfo, SegmentKind};
use crate::music::library::LoadedMusic;
use crate::music::performance::MusicEvent;
use crate::sink::Boundary;

/// Two measures (4/4 at 120) of quarter notes on channel 0: a measure is two seconds.
fn primary() -> Arc<LoadedMusic> {
    let notes = (0..8)
        .map(|q| seq(q * QUARTER, QUARTER, 0, 60 + q as u8))
        .collect();
    sequence(8 * QUARTER, notes)
}

/// One quarter note on channel 1 at its start.
fn motif() -> Arc<LoadedMusic> {
    sequence(4 * QUARTER, vec![seq(0, QUARTER, 1, 90)])
}

fn secondary(handle: u32, start: Boundary) -> SegmentInfo {
    SegmentInfo {
        start,
        ..super::tests::info(SegmentKind::Secondary, handle, &motif())
    }
}

/// When the motif's note sounds, cued at 0.3 s with `start` (over the primary when `with_primary`).
fn cued_at(start: Boundary, with_primary: bool) -> (f64, Calls) {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    if with_primary {
        assert!(thread.play(super::tests::info(SegmentKind::Primary, 1, &primary())));
    }
    let _ = run(&mut thread, 0.3, &mut calls);
    assert!(thread.play(secondary(2, start)));
    let played = run(&mut thread, 6.0, &mut calls);
    let at = played
        .iter()
        .find(|(_, e)| matches!(e, MusicEvent::NoteOn { channel: 1, .. }))
        .map(|(t, _)| *t)
        .expect("the motif plays");
    (at, calls)
}

#[test]
fn a_secondary_starts_on_the_primarys_next_beat_measure_or_end() {
    assert!((cued_at(Boundary::Beat, true).0 - 0.5).abs() < 1e-9);
    assert!((cued_at(Boundary::Measure, true).0 - 2.0).abs() < 1e-9);
    assert!((cued_at(Boundary::End, true).0 - 4.0).abs() < 1e-9);
    assert!((cued_at(Boundary::Immediate, true).0 - 0.3).abs() < 1e-9);
}

#[test]
fn with_no_primary_or_no_marker_a_secondary_starts_at_once() {
    assert!((cued_at(Boundary::Measure, false).0 - 0.3).abs() < 1e-9);
    assert!((cued_at(Boundary::Marker, true).0 - 0.3).abs() < 1e-9);
}

#[test]
fn a_cued_secondary_is_not_playing_until_its_boundary_and_a_stop_drops_it() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    assert!(thread.play(super::tests::info(SegmentKind::Primary, 1, &primary())));
    let _ = run(&mut thread, 0.3, &mut calls);
    assert!(thread.play(secondary(2, Boundary::Measure)));
    let _ = run(&mut thread, 1.0, &mut calls);
    assert!(!thread.is_playing(2));
    assert!(!calls.0.contains(&("start", 2)));
    let _ = run(&mut thread, 2.1, &mut calls);
    assert!(thread.is_playing(2));
    assert!(calls.0.contains(&("start", 2)));
    // Another, stopped before its measure: it never plays.
    assert!(thread.play(secondary(3, Boundary::Measure)));
    thread.stop(3, Boundary::Immediate);
    let played = run(&mut thread, 5.0, &mut calls);
    assert_eq!(
        played
            .iter()
            .filter(|(_, e)| matches!(e, MusicEvent::NoteOn { channel: 1, .. }))
            .count(),
        0
    );
    assert!(calls.0.contains(&("ended", 3)));
    assert!(!calls.0.contains(&("start", 3)));
}
