//! The performance band on synthetic segments (no game data): a band segment starting reaches a
//! band-less segment already playing, a band track's item reaches the playing segments at its
//! instant, and a segment with its own band keeps it. Each band here maps channel 0 to one patch,
//! and its sources are a marker (`Source::Missing` on the patch's number), so the voices' changes
//! can be told apart by the patch.

use std::sync::Arc;

use super::tests::{info, run, seq, sequence, Calls, QUARTER};
use super::{MusicThread, SegmentInfo, SegmentKind};
use crate::music::band::{Band, BandInstrument};
use crate::music::performance::MusicEvent;
use crate::synth::{PerformanceBand, Source};

/// A band whose channel `pchannel` plays `patch`.
fn band(patch: u32, pchannel: u32) -> Arc<PerformanceBand> {
    Arc::new(PerformanceBand {
        band: Arc::new(Band {
            guid: None,
            name: None,
            instruments: vec![BandInstrument {
                patch,
                assign_patch: 0,
                note_ranges: [0; 4],
                pchannel,
                flags: 0,
                pan: 0,
                volume: 0,
                transpose: 0,
                channel_priority: 0,
                pitch_bend_range: 0,
                collection: None,
            }],
        }),
        sources: Arc::from(vec![(patch, Source::Missing)]),
    })
}

/// The band items as the queue takes them: `(seconds, band)`.
fn items(list: &[(f64, Arc<PerformanceBand>)]) -> Arc<[(f64, Arc<PerformanceBand>)]> {
    Arc::from(list.to_vec())
}

/// The note-ons of `events` as (seconds, instrument, note).
fn note_ons(events: &[(f64, MusicEvent)]) -> Vec<(f64, Option<u32>, u8)> {
    events
        .iter()
        .filter_map(|(t, e)| match e {
            MusicEvent::NoteOn {
                instrument, note, ..
            } => Some((*t, *instrument, *note)),
            _ => None,
        })
        .collect()
}

/// Four quarter notes on channel 0 (60, 62, 64, 65), on a band-less segment.
fn four_notes() -> Arc<crate::music::library::LoadedMusic> {
    sequence(
        4 * QUARTER,
        vec![
            seq(0, QUARTER, 0, 60),
            seq(QUARTER, QUARTER, 0, 62),
            seq(2 * QUARTER, QUARTER, 0, 64),
            seq(3 * QUARTER, QUARTER, 0, 65),
        ],
    )
}

#[test]
fn a_band_segment_starting_reaches_a_band_less_segment_already_playing() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    assert!(thread.play(info(SegmentKind::Secondary, 1, &four_notes())));
    // The first half second: the notes sound with no band (no instrument).
    let before = run(&mut thread, 0.6, &mut calls);
    assert_eq!(note_ons(&before), vec![(0.0, None, 60), (0.5, None, 62)]);
    // A band segment starts at 0.6 s: its band (patch 40 on channel 0) is the performance's.
    let band_segment = sequence(QUARTER, Vec::new());
    assert!(thread.play(SegmentInfo {
        kind: SegmentKind::Secondary,
        handle: 2,
        segment: band_segment,
        bands: items(&[(0.0, band(40, 0))]),
    }));
    let after = run(&mut thread, 2.0, &mut calls);
    // The first segment's later notes take the band's patch.
    assert_eq!(
        note_ons(&after),
        vec![(1.0, Some(40), 64), (1.5, Some(40), 65)]
    );
    // Its voice is told the band's sources (the band segment's voice starts with its own).
    let changes = thread.take_band_changes();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].handle, 1);
    assert_eq!(changes[0].sources.as_ref(), &[(40, Source::Missing)][..]);
}

#[test]
fn a_band_track_item_reaches_the_segments_playing_at_its_instant() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    // A band-less segment playing notes on channel 0 at 0.25, 0.75 and 1.25 s.
    let plain = sequence(
        4 * QUARTER,
        vec![
            seq(QUARTER / 2, QUARTER, 0, 60),
            seq(3 * QUARTER / 2, QUARTER, 0, 62),
            seq(5 * QUARTER / 2, QUARTER, 0, 64),
        ],
    );
    assert!(thread.play(info(SegmentKind::Secondary, 1, &plain)));
    // A band segment: patch 40 from its start, patch 41 from 0.5 s (its band track's second item).
    let banded = sequence(
        4 * QUARTER,
        vec![
            seq(0, QUARTER, 0, 36),
            seq(QUARTER, QUARTER, 0, 38),
            seq(2 * QUARTER, QUARTER, 0, 40),
        ],
    );
    assert!(thread.play(SegmentInfo {
        kind: SegmentKind::Secondary,
        handle: 2,
        segment: banded,
        bands: items(&[(0.0, band(40, 0)), (0.5, band(41, 0))]),
    }));
    let events = run(&mut thread, 3.0, &mut calls);
    // The band segment's own notes (36, 38, 40): 40 until its band track's item, 41 from it.
    let own: Vec<_> = note_ons(&events)
        .into_iter()
        .filter(|(_, _, note)| *note < 48)
        .collect();
    assert_eq!(
        own,
        vec![
            (0.0, Some(40), 36),
            (0.5, Some(41), 38),
            (1.0, Some(41), 40)
        ]
    );
    // The band-less segment (60, 62, 64): 40 from the band segment's start, 41 from its item.
    let plain_notes: Vec<_> = note_ons(&events)
        .into_iter()
        .filter(|(_, _, note)| *note >= 48)
        .collect();
    assert_eq!(
        plain_notes,
        vec![
            (0.25, Some(40), 60),
            (0.75, Some(41), 62),
            (1.25, Some(41), 64)
        ]
    );
    // The changes in order: the plain voice at the band segment's start (40); then the band
    // segment's own voice at its item (41), and the plain voice with it (41).
    let changes = thread.take_band_changes();
    let got: Vec<(u32, Vec<(u32, Source)>)> = changes
        .iter()
        .map(|c| (c.handle, c.sources.to_vec()))
        .collect();
    assert_eq!(
        got,
        vec![
            (1, vec![(40, Source::Missing)]),
            (2, vec![(41, Source::Missing)]),
            (1, vec![(41, Source::Missing)]),
        ]
    );
}

#[test]
fn a_segment_with_its_own_band_keeps_it_when_another_band_starts() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    let own = sequence(
        4 * QUARTER,
        vec![seq(0, QUARTER, 0, 60), seq(2 * QUARTER, QUARTER, 0, 62)],
    );
    assert!(thread.play(SegmentInfo {
        kind: SegmentKind::Secondary,
        handle: 1,
        segment: own,
        bands: items(&[(0.0, band(40, 0))]),
    }));
    let _ = run(&mut thread, 0.5, &mut calls);
    // Another band starts at 0.5 s: patch 50 becomes the performance's, not this segment's.
    assert!(thread.play(SegmentInfo {
        kind: SegmentKind::Secondary,
        handle: 2,
        segment: sequence(QUARTER, Vec::new()),
        bands: items(&[(0.0, band(50, 0))]),
    }));
    let events = run(&mut thread, 2.0, &mut calls);
    assert_eq!(note_ons(&events), vec![(1.0, Some(40), 62)]);
    assert!(
        thread.take_band_changes().iter().all(|c| c.handle != 1),
        "the segment with its own band is not told of the performance's band"
    );
}
