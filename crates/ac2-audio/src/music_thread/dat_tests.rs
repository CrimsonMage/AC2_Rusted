//! The music queue on the DATs (`AC2_DAT_DIR`): a styled segment queued, its notes sounding measure
//! by measure, and the groove changing the patterns of the measures after the change. The checks
//! are the queue's own invariants; nothing from the DATs is stored.

use std::sync::Arc;
use std::time::Duration;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};

use super::{MusicHandler, MusicThread, SegmentInfo, SegmentKind};
use crate::data::{DatSounds, MusicInfo};
use crate::music::clock::MusicClock;
use crate::music::harmony::Harmony;
use crate::music::library::{LoadedMusic, MusicLibrary};
use crate::music::performance::{MusicEvent, TimedEvent};
use crate::music::segment::TrackData;
use crate::music::style::Style;
use crate::music::style_performance::PatternChoice;
use crate::music::style_play::{covers, pattern_measure_notes};
use crate::music::MusicObject;
use crate::{PlaySoundData, RecordingSink, SoundManager};

/// The first segment with a style reference in the DATs (no chord track).
const STYLED_SEGMENT: u32 = 0x2400_0061;
const WAIT: Duration = Duration::from_secs(10);

fn store() -> Option<Arc<ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ResourceStore::new(dataset)))
}

/// The style the segment's styles track references, from the segment's closure.
fn style_of(music: &LoadedMusic) -> Arc<Style> {
    let MusicObject::Segment(segment) = &music.segment else {
        panic!("a music segment");
    };
    let reference = segment
        .tracks
        .iter()
        .find_map(|track| match &track.data {
            TrackData::Styles(styles) => styles.first().map(|(_, r)| r.clone()),
            _ => None,
        })
        .expect("a styles track");
    match music.resolve(&reference) {
        Some(MusicObject::Style(style)) => Arc::clone(style),
        _ => panic!("the referenced style is in the closure"),
    }
}

/// The length of the first measure, in seconds.
fn measure_seconds(music: &LoadedMusic) -> f64 {
    let MusicObject::Segment(segment) = &music.segment else {
        panic!("a music segment");
    };
    let clock = MusicClock::from_segment(segment);
    clock.seconds_at(MusicClock::measure_ticks(&clock.signature_at(0)))
}

/// Ignores the callbacks (the queue is driven for its events here).
struct Quiet;

impl MusicHandler for Quiet {
    fn on_segment_start(&mut self, _handle: u32) {}
    fn on_segment_ended(&mut self, _handle: u32) {}
}

#[test]
fn a_dat_segment_queued_plays_its_notes_and_the_next_measure_follows_the_groove() {
    let Some(store) = store() else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let record = library.record(STYLED_SEGMENT).expect("the segment record");
    let music = Arc::new(library.closure(record));
    let style = style_of(&music);
    let MusicObject::Segment(segment) = &music.segment else {
        panic!("a music segment");
    };
    let harmony = Harmony::from_segment(segment);
    let ticks = MusicClock::measure_ticks(&MusicClock::from_segment(segment).signature_at(0));
    let measure = measure_seconds(&music);

    // Two groove levels: each covered by patterns, the share of which sound notes in the
    // segment is the best available, and the second covered by a different set of patterns.
    let measures = (segment.length / ticks).max(0) as usize;
    let sounds = |index: usize| -> bool {
        let pattern = &style.patterns[index];
        let band = pattern.band.as_ref().or(style.bands.first());
        (0..measures).any(|m| {
            let start = m as i32 * ticks;
            !pattern_measure_notes(&style, pattern, band, m, &harmony, start).is_empty()
        })
    };
    let covering = |level: i8| -> Vec<usize> {
        style
            .patterns
            .iter()
            .enumerate()
            .filter(|(_, p)| covers(p, level))
            .map(|(i, _)| i)
            .collect()
    };
    let candidates: Vec<(i8, Vec<usize>, f64)> = (0..=127i8)
        .filter_map(|level| {
            let set = covering(level);
            if set.is_empty() {
                return None;
            }
            let noted = set.iter().filter(|&&i| sounds(i)).count();
            let share = noted as f64 / set.len() as f64;
            Some((level, set, share))
        })
        .collect();
    let best = |candidates: &[(i8, Vec<usize>, f64)]| {
        candidates
            .iter()
            .max_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(level, set, _)| (*level, set.clone()))
    };
    let (level_a, set_a) = best(&candidates).expect("a level some pattern covers");
    let others: Vec<_> = candidates
        .iter()
        .filter(|c| c.1 != set_a)
        .cloned()
        .collect();
    let (level_b, _) = best(&others).expect("a level other patterns cover");

    // Copies of the segment, each choosing its patterns on its own (the choice is seeded by the
    // playback): a silent pattern is one of the covering ones at most levels.
    const COPIES: u32 = 4;
    let mut thread = MusicThread::new();
    thread.set_groove_level(i32::from(level_a));
    thread.use_time(0.0, None);
    for handle in 1..=COPIES {
        assert!(thread.play(SegmentInfo {
            kind: SegmentKind::Secondary,
            handle,
            segment: Arc::clone(&music),
        }));
    }
    // Measure 0 of each copy chooses its pattern at level A.
    thread.use_time(0.1, Some(&mut Quiet));
    // The groove changes after measure 0 has started: every later measure chooses at level B.
    thread.set_groove_level(i32::from(level_b));
    let length = MusicClock::from_segment(segment).seconds_at(segment.length);
    // The choices are read before the end: a segment that ends leaves the queue.
    thread.use_time(length - 0.01, Some(&mut Quiet));
    let choices: Vec<Vec<PatternChoice>> = (1..=COPIES)
        .map(|handle| thread.choices(handle).expect("a style plays").to_vec())
        .collect();
    thread.use_time(length + 0.1, Some(&mut Quiet));
    let mut events: Vec<TimedEvent> = Vec::new();
    thread.send_events(&mut events);
    assert!(
        (1..=COPIES).all(|handle| !thread.is_playing(handle)),
        "each ends at its length"
    );

    for copy in &choices {
        assert!(copy.len() >= 2, "the segment has more than one measure");
        assert_eq!(copy[0].groove, level_a);
        assert!(copy[1..].iter().all(|c| c.groove == level_b));
        assert!(covers(&style.patterns[copy[1].pattern], level_b));
    }

    // Each measure sounds exactly the notes of the patterns the copies chose for it (no band or
    // chord change in this segment's measures).
    let measures = choices
        .iter()
        .flatten()
        .map(|c| c.measure)
        .max()
        .unwrap_or(0)
        + 1;
    let mut sounded = 0;
    for measure_index in 0..measures {
        let expected: usize = choices
            .iter()
            .flatten()
            .filter(|c| c.measure == measure_index)
            .map(|c| {
                let pattern = &style.patterns[c.pattern];
                let band = pattern.band.as_ref().or(style.bands.first());
                let start = c.measure as i32 * ticks;
                pattern_measure_notes(&style, pattern, band, c.measure, &harmony, start).len()
            })
            .sum();
        let from = measure_index as f64 * measure;
        let got = events
            .iter()
            .filter(|e| matches!(e.event, MusicEvent::NoteOn { .. }))
            .filter(|e| (from..from + measure).contains(&e.seconds))
            .count();
        assert_eq!(
            got, expected,
            "measure {measure_index} sounds its patterns' notes"
        );
        sounded += got;
    }
    assert!(sounded > 0, "the segment sounds notes");
}

#[test]
fn a_dat_music_info_of_the_segment_plays_through_the_sound_manager() {
    let Some(store) = store() else {
        return;
    };
    // The MusicInfo that names the styled segment.
    let did = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2C00_0000, 0x2CFF_FFFE)
        .find(|&did| {
            store
                .load_cached_packed::<MusicInfo>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    DataId::new(did),
                    0,
                ))
                .ok()
                .flatten()
                .is_some_and(|info| info.segment == STYLED_SEGMENT)
        });
    let Some(did) = did else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let record = library.record(STYLED_SEGMENT).expect("the segment record");
    let style = style_of(&library.closure(record));
    let level = style
        .patterns
        .iter()
        .find(|p| p.embellishment == 0)
        .expect("a groove pattern")
        .groove_bottom as i8;

    let mut manager = SoundManager::new(
        Box::new(RecordingSink::new()),
        Arc::new(DatSounds::new(Arc::clone(&store))),
    );
    manager.set_groove_level(i32::from(level));
    let handle = manager
        .play(&PlaySoundData::new(did))
        .expect("the music is requested");
    assert!(manager.wait_for_loads(WAIT));
    manager.use_time(0.0);
    manager.use_time(0.1);
    // Loaded on the loader thread, the segment plays from the next use_time, at the groove.
    assert!(manager.music().is_playing(handle));
    let choice = manager.music().choices(handle).expect("a style plays")[0];
    assert_eq!(choice.groove, level);
}
