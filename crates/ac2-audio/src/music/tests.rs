//! The music records against real data (`AC2_DAT_DIR`): invariants, no stored outputs.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};

use super::library::MusicRecord;
use super::segment::TrackData;
use super::{MusicLibrary, MusicObject, MusicSource};
use crate::data::MusicInfo;

fn store() -> Option<Arc<ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ResourceStore::new(dataset)))
}

#[test]
fn every_music_record_reads() {
    let Some(store) = store() else {
        return;
    };
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .collect();
    assert_eq!(ids.len(), 1111);
    let mut kinds: HashMap<&str, usize> = HashMap::new();
    let mut notes = 0usize;
    for did in ids {
        let record = store
            .load_cached_packed::<MusicRecord>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap_or_else(|e| panic!("{did:08X}: {e}"))
            .expect("listed record");
        let kind = match &record.0 {
            MusicObject::Segment(s) => {
                assert!(s.length >= 0, "{did:08X}");
                for track in &s.tracks {
                    if let TrackData::Tempo(t) = &track.data {
                        assert!(t.iter().all(|(_, bpm)| *bpm > 0.0 && *bpm < 1000.0));
                    }
                }
                "segment"
            }
            MusicObject::Style(s) => {
                notes += s.parts.iter().map(|p| p.notes.len()).sum::<usize>();
                assert!(!s.patterns.is_empty(), "{did:08X} has patterns");
                "style"
            }
            MusicObject::Band(_) => "band",
            MusicObject::Wave(_) => "wave",
            MusicObject::Collection(c) => {
                assert!(!c.instruments.is_empty());
                for i in &c.instruments {
                    for r in &i.regions {
                        let index = r.link.table_index as usize;
                        assert!(
                            c.waves.get(index).is_some_and(|w| w.is_some()),
                            "{did:08X}: region wave {index}"
                        );
                    }
                }
                "dls"
            }
            MusicObject::Script(s) => {
                assert_eq!(s.language, "AudioVBScript");
                assert!(s.container.is_some());
                "script"
            }
            MusicObject::Container(_) => "container",
            MusicObject::Other(_) => "other",
        };
        *kinds.entry(kind).or_default() += 1;
    }
    eprintln!("{kinds:?}, {notes} style notes");
    assert_eq!(kinds["segment"], 630);
    assert_eq!(kinds["style"], 396);
    assert_eq!(kinds["dls"], 61);
    assert_eq!(kinds["script"], 14);
}

#[test]
fn every_music_info_loads_its_music() {
    let Some(store) = store() else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2C00_0000, 0x2CFF_FFFE)
        .collect();
    let (mut segments, mut waves, mut missing) = (0, 0, 0);
    for did in ids {
        let info = store
            .load_cached_packed::<MusicInfo>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .unwrap();
        if info.segment >> 24 == 0x61 {
            // Decoding every MP3 is slow; one is enough here.
            if waves > 0 {
                continue;
            }
        }
        match library.load_music(&info) {
            Some(MusicSource::DirectMusic(music)) => {
                segments += 1;
                missing += music.missing;
                assert!(matches!(music.segment, MusicObject::Segment(_)));
            }
            Some(MusicSource::Wave(pcm)) => {
                waves += 1;
                assert!(pcm.rate >= 8000 && pcm.frames() > 44_100, "{did:08X}");
            }
            None => panic!("{did:08X}: no music"),
        }
    }
    eprintln!("{segments} segments ({missing} unresolved references), {waves} waves");
    assert_eq!(segments, 530);
    assert_eq!(waves, 1);
}

#[test]
fn every_segment_is_a_whole_number_of_measures() {
    use super::clock::{Boundary, MusicClock};
    let Some(store) = store() else {
        return;
    };
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .collect();
    let mut segments = 0;
    for did in ids {
        let record = store
            .load_cached_packed::<MusicRecord>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .expect("listed record");
        let MusicObject::Segment(segment) = &record.0 else {
            continue;
        };
        segments += 1;
        let clock = MusicClock::from_segment(segment);
        let sig = clock.signature_at(0);
        let measure = MusicClock::measure_ticks(&sig);
        assert_eq!(segment.length % measure, 0, "{did:08X}: {sig:?}");
        assert_eq!(
            clock.next_boundary(segment.length, Boundary::Measure),
            segment.length
        );
        let seconds = clock.seconds_at(segment.length);
        assert!(seconds.is_finite() && seconds > 0.0, "{did:08X}: {seconds}");
    }
    assert_eq!(segments, 630);
}

#[test]
fn a_segment_plays_its_sequence_notes_for_its_length() {
    use super::clock::MusicClock;
    use super::performance::{MusicEvent, MusicPerformance, TimedEvent};
    let Some(store) = store() else {
        return;
    };
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .collect();
    let mut checked = 0;
    for did in ids {
        let record = store
            .load_cached_packed::<MusicRecord>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .expect("listed record");
        let MusicObject::Segment(segment) = &record.0 else {
            continue;
        };
        let items: Vec<_> = segment
            .tracks
            .iter()
            .filter_map(|t| match &t.data {
                TrackData::Sequence { events, .. } => Some(events.clone()),
                _ => None,
            })
            .flatten()
            .filter(|e| e.status & 0xF0 == 0x90)
            .collect();
        if items.is_empty() {
            continue;
        }
        checked += 1;
        let clock = MusicClock::from_segment(segment);
        let mut perf = MusicPerformance::new(segment, None, false);
        let mut sink: Vec<TimedEvent> = Vec::new();
        perf.advance(clock.seconds_at(segment.length) + 1.0, &mut sink);
        assert!(perf.finished(), "{did:08X}");
        // Every note-on is played, and every one is released by the segment's end.
        let ons = sink
            .iter()
            .filter(|e| matches!(e.event, MusicEvent::NoteOn { .. }))
            .count();
        assert_eq!(ons, items.len(), "{did:08X}: note-ons");
        assert_eq!(sink.len(), 2 * items.len(), "{did:08X}: note-offs");
        assert!(sink.windows(2).all(|w| w[0].seconds <= w[1].seconds));
        // The first event is the earliest note at its clock time.
        let first_tick = items.iter().map(|e| e.time + i32::from(e.offset)).min();
        let first = sink[0];
        assert_eq!(
            first.seconds,
            clock.seconds_at(first_tick.unwrap()),
            "{did:08X}"
        );
        assert!(matches!(first.event, MusicEvent::NoteOn { .. }));
        if checked == 1 {
            eprintln!(
                "{did:08X}: {} notes, {:.3} s",
                items.len(),
                clock.seconds_at(segment.length)
            );
        }
        break;
    }
    assert_eq!(checked, 1, "a segment with a sequence track");
}

#[test]
fn the_first_styled_segment_chooses_its_patterns_by_groove() {
    use super::clock::MusicClock;
    use super::harmony::{Harmony, DEFAULT_SUBCHORD};
    use super::performance::{MusicEvent, TimedEvent};
    use super::segment::SubChord;
    use super::style::Pattern;
    use super::style_performance::StylePerformance;
    use super::style_play::pattern_measure_notes;
    let Some(store) = store() else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .collect();
    // The first segment with a style reference (a styles track), in DID order.
    let mut found = None;
    for did in ids {
        let record = store
            .load_cached_packed::<MusicRecord>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .expect("listed record");
        let MusicObject::Segment(segment) = &record.0 else {
            continue;
        };
        let reference = segment.tracks.iter().find_map(|t| match &t.data {
            TrackData::Styles(styles) => styles.first().map(|(_, r)| r.clone()),
            _ => None,
        });
        if let Some(reference) = reference {
            found = Some((did, segment.clone(), reference));
            break;
        }
    }
    let (did, segment, reference) = found.expect("a segment with a style reference");
    assert_eq!(did, 0x2400_0061, "the first styled segment");
    // The segment has no chord track, so it plays over C major.
    let harmony = Harmony::from_segment(&segment);
    assert_eq!(harmony, Harmony::default());
    let guid = reference.guid.expect("a style reference names its style");
    let Some(MusicObject::Style(style)) = library.by_guid(&guid) else {
        panic!("{did:08X}: the referenced style is not in the library");
    };
    let clock = MusicClock::from_segment(&segment);
    let measure = MusicClock::measure_ticks(&clock.signature_at(0));
    let measure_seconds = clock.seconds_at(measure);
    let covers = |p: &Pattern, level: i32| {
        p.embellishment == 0
            && i32::from(p.groove_bottom) <= level
            && level <= i32::from(p.groove_top)
    };
    // One measure at each level. Playback reaching a measure's start chooses its pattern, so the
    // advance also chooses measure 1 (its start is exactly one measure in); only the events
    // before that start are measure 0's.
    let mut first_choice = Vec::new();
    for level in [1i8, 100] {
        let mut perf = StylePerformance::new(&segment, Arc::clone(&style), level, 1);
        let mut sink: Vec<TimedEvent> = Vec::new();
        perf.advance(measure_seconds, &mut sink);
        let chosen = perf.choices();
        for c in chosen {
            assert!(
                covers(&style.patterns[c.pattern], i32::from(level)),
                "{did:08X}: {c:?}"
            );
        }
        let in_measure: Vec<&TimedEvent> = sink
            .iter()
            .filter(|e| e.seconds < measure_seconds)
            .collect();
        assert!(in_measure.iter().all(|e| e.seconds >= 0.0));
        // The measure's note-ons are the chosen pattern's notes for it.
        if let Some(c) = chosen.first() {
            let pattern = &style.patterns[c.pattern];
            let band = pattern.band.as_ref().or(style.bands.first());
            let notes = pattern_measure_notes(&style, pattern, band, 0, &harmony, 0).len();
            let ons = in_measure
                .iter()
                .filter(|e| matches!(e.event, MusicEvent::NoteOn { .. }))
                .count();
            assert_eq!(ons, notes, "{did:08X}: groove {level}");
        }
        let covering = style
            .patterns
            .iter()
            .filter(|p| covers(p, i32::from(level)))
            .count();
        eprintln!(
            "{did:08X} groove {level}: {covering} covering patterns, first measure chose {:?}, {} events in it",
            chosen.first().map(|c| c.pattern),
            in_measure.len()
        );
        first_choice.push((level, chosen.first().copied(), in_measure.len()));
    }
    // Level 1 is below every pattern's range in this style (the lowest bottom is 2): nothing
    // covers it, so its measure is silent. Level 100 is covered by eleven patterns.
    assert_eq!(first_choice[0], (1, None, 0));
    let (_, chosen, _) = first_choice[1];
    assert_eq!(chosen.map(|c| c.measure), Some(0));
    assert!(style.patterns.iter().all(|p| !covers(p, 1)));
    // Played through the whole segment at groove 100, every measure chooses a covering pattern,
    // and the note-ons are the chosen patterns' notes. The choice is random among the eleven
    // covering patterns, seven of which are blank (their parts hold no notes), so one seed can
    // choose blanks throughout: the segment must sound for some seed among a few.
    let mut sounded = false;
    for seed in 1..=8 {
        let mut perf = StylePerformance::new(&segment, Arc::clone(&style), 100, seed);
        let mut sink: Vec<TimedEvent> = Vec::new();
        perf.advance(clock.seconds_at(segment.length) + 1.0, &mut sink);
        assert!(perf.finished());
        assert_eq!(perf.choices().len(), (segment.length / measure) as usize);
        assert!(perf
            .choices()
            .iter()
            .all(|c| covers(&style.patterns[c.pattern], 100)));
        let expected: usize = perf
            .choices()
            .iter()
            .map(|c| {
                let pattern = &style.patterns[c.pattern];
                let band = pattern.band.as_ref().or(style.bands.first());
                let start = c.measure as i32 * measure;
                pattern_measure_notes(&style, pattern, band, c.measure, &harmony, start).len()
            })
            .sum();
        let ons = sink
            .iter()
            .filter(|e| matches!(e.event, MusicEvent::NoteOn { .. }))
            .count();
        assert_eq!(ons, expected, "seed {seed}");
        for event in &sink {
            if let MusicEvent::NoteOn { note, .. } = event.event {
                assert!(note <= 127, "{did:08X}: note {note}");
            }
        }
        sounded |= ons > 0;
    }
    assert!(sounded, "groove 100 sounds for some seed");
    // Every part here that holds notes is FIXED (play mode 0): its pitches are the MIDI values,
    // so a chord of another root plays the same notes. The blank parts hold no notes.
    let other = Harmony::new(
        0,
        0xAB5AB5,
        vec![(
            0,
            vec![SubChord {
                chord_root: 5,
                ..DEFAULT_SUBCHORD
            }],
        )],
    );
    for pattern in &style.patterns {
        let band = pattern.band.as_ref().or(style.bands.first());
        for m in 0..usize::from(pattern.measures) {
            let start = m as i32 * measure;
            let here = pattern_measure_notes(&style, pattern, band, m, &harmony, start);
            let there = pattern_measure_notes(&style, pattern, band, m, &other, start);
            assert_eq!(here, there, "{did:08X}: measure {m} of {:?}", pattern.name);
            assert!(here.iter().all(|n| n.note <= 127));
        }
        for pref in &pattern.parts {
            if let Some(part) = style.part(&pref.part) {
                assert!(part.notes.is_empty() || part.play_mode == 0, "{did:08X}");
            }
        }
    }
}

#[test]
fn every_style_note_decodes_into_the_midi_range_and_chord_relative_notes_follow_the_chord() {
    use super::harmony::{ChordState, DEFAULT_SUBCHORD};
    use super::music_value::{effective_play_mode, midi_note};
    use super::segment::SubChord;
    let Some(store) = store() else {
        return;
    };
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .collect();
    // C major over key C, and the same with its chord root a fifth up.
    let c_major = ChordState {
        key: 0,
        subchord: DEFAULT_SUBCHORD,
    };
    let fifth_up = ChordState {
        key: 0,
        subchord: SubChord {
            chord_root: 7,
            ..DEFAULT_SUBCHORD
        },
    };
    let (mut relative, mut moved) = (0usize, 0usize);
    for did in ids {
        let record = store
            .load_cached_packed::<MusicRecord>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .expect("listed record");
        let MusicObject::Style(style) = &record.0 else {
            continue;
        };
        for part in &style.parts {
            for note in &part.notes {
                let mode = effective_play_mode(note.play_mode, part.play_mode);
                let at_c = midi_note(note.music_value, mode, &c_major);
                let at_g = midi_note(note.music_value, mode, &fifth_up);
                assert!(
                    at_c <= 127 && at_g <= 127,
                    "{did:08X}: {:#06x}",
                    note.music_value
                );
                if mode == 0 {
                    assert_eq!(at_c, at_g, "{did:08X}: a FIXED note moved");
                    assert_eq!(at_c, (note.music_value & 0x7F) as u8);
                } else {
                    relative += 1;
                    moved += usize::from(at_c != at_g);
                }
            }
        }
    }
    eprintln!("{relative} chord-relative style notes, {moved} moved by a fifth");
    assert!(relative > 0);
    assert!(moved > 0, "a chord-relative note follows its chord");
}

/// The band tracks of the segments (`RIFF 'DMBT'` holding the `lbdl` band items) are read as
/// bands; a band-class track that falls through to `Other` would leave its notes without an
/// instrument.
#[test]
fn every_band_track_reads_its_bands() {
    let Some(store) = store() else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .collect();
    let (mut band_tracks, mut instruments, mut unread) = (0usize, 0usize, 0usize);
    for did in ids {
        let Some(MusicObject::Segment(segment)) = library.record(did) else {
            continue;
        };
        for track in &segment.tracks {
            match &track.data {
                TrackData::Bands(bands) => {
                    band_tracks += 1;
                    instruments += bands
                        .iter()
                        .map(|(_, b)| b.instruments.len())
                        .sum::<usize>();
                }
                TrackData::Other if track.class == super::segment::class::BAND => unread += 1,
                _ => {}
            }
        }
    }
    assert_eq!(unread, 0, "every band track reads");
    assert_eq!(band_tracks, 44);
    assert_eq!(instruments, 93);
}
