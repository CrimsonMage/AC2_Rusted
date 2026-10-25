//! Style playback on synthetic styles: pattern choice by groove level, changes at the next
//! measure, the looping of a pattern across the segment, the band and variation rules, and the
//! pitch of chord-relative notes against the segment's chords.

use std::sync::Arc;

use super::band::{Band, BandInstrument};
use super::performance::{MusicEvent, TimedEvent};
use super::riff::Guid;
use super::segment::{Chord, Segment, SubChord, TimeSignature, Track, TrackData};
use super::style::{Part, PartRef, Pattern, Style, StyleNote};
use super::style_performance::{PatternChoice, StylePerformance};

/// A quarter at 120 BPM is half a second; a 4/4 measure is two seconds.
const MEASURE: f64 = 2.0;

pub(crate) fn guid(byte: u8) -> Guid {
    Guid([byte; 16])
}

/// A note on the beat at `grid` (a grid is a sixteenth in 4/4) with `music_value`, one quarter
/// long, playing in every variation.
pub(crate) fn note(grid: i32, music_value: u16) -> StyleNote {
    StyleNote {
        grid_start: grid,
        variation: u32::MAX,
        duration: 768,
        time_offset: 0,
        music_value,
        velocity: 100,
        time_range: 0,
        duration_range: 0,
        velocity_range: 0,
        inversion_id: 0,
        play_mode: 16,
        note_flags: 0,
    }
}

pub(crate) fn part(byte: u8, measures: u16, notes: Vec<StyleNote>) -> Part {
    Part {
        time_sig: TimeSignature::default(),
        variation_choices: [0x7FFF_FFFF; 32],
        guid: guid(byte),
        measures,
        play_mode: 0,
        invert_upper: 0,
        invert_lower: 0,
        flags: 0,
        notes,
        curves: Vec::new(),
    }
}

pub(crate) fn pattern(bottom: u8, top: u8, measures: u16, parts: &[(u8, u32)]) -> Pattern {
    Pattern {
        name: None,
        time_sig: TimeSignature::default(),
        groove_bottom: bottom,
        groove_top: top,
        embellishment: 0,
        measures,
        dest_groove_bottom: 1,
        dest_groove_top: 100,
        flags: 0,
        rhythm: Vec::new(),
        parts: parts
            .iter()
            .map(|&(byte, pchannel)| PartRef {
                part: guid(byte),
                logical_part: 0,
                variation_lock: 0,
                subchord_level: 0,
                priority: 0,
                random_variation: 0,
                pchannel,
            })
            .collect(),
        band: None,
    }
}

pub(crate) fn style(parts: Vec<Part>, patterns: Vec<Pattern>) -> Style {
    Style {
        guid: None,
        name: None,
        time_sig: TimeSignature::default(),
        tempo: 120.0,
        parts,
        patterns,
        bands: Vec::new(),
    }
}

/// A segment of `measures` 4/4 measures and no tracks (the default tempo and meter).
pub(crate) fn segment(measures: i32) -> Segment {
    Segment {
        guid: None,
        name: None,
        repeats: 0,
        length: measures * 3072,
        play_start: 0,
        loop_start: 0,
        loop_end: 0,
        resolution: 0,
        flags: 0,
        tracks: Vec::new(),
    }
}

/// The two-pattern style of the spec: `low` (groove 1..=50, a note of 60 each measure) and
/// `high` (groove 51..=100, a note of 72).
pub(crate) fn two_pattern_style() -> Style {
    style(
        vec![part(1, 1, vec![note(0, 60)]), part(2, 1, vec![note(0, 72)])],
        vec![pattern(1, 50, 1, &[(1, 0)]), pattern(51, 100, 1, &[(2, 0)])],
    )
}

/// Advances `perf` by `seconds` and returns the events sent.
pub(crate) fn play(perf: &mut StylePerformance, seconds: f64) -> Vec<TimedEvent> {
    let mut sink: Vec<TimedEvent> = Vec::new();
    perf.advance(seconds, &mut sink);
    sink
}

pub(crate) fn on_notes(events: &[TimedEvent]) -> Vec<(f64, u8)> {
    events
        .iter()
        .filter_map(|e| match e.event {
            MusicEvent::NoteOn { note, .. } => Some((e.seconds, note)),
            _ => None,
        })
        .collect()
}

#[test]
fn level_20_picks_the_first_pattern_and_80_the_second() {
    let style = Arc::new(two_pattern_style());
    let seg = segment(4);

    let mut low = StylePerformance::new(&seg, Arc::clone(&style), 20, 1);
    let events = play(&mut low, 4.0 * MEASURE);
    assert_eq!(
        on_notes(&events),
        vec![(0.0, 60), (2.0, 60), (4.0, 60), (6.0, 60)]
    );
    assert!(low
        .choices()
        .iter()
        .all(|c| c.pattern == 0 && c.groove == 20));
    assert_eq!(low.choices().len(), 4);

    let mut high = StylePerformance::new(&seg, Arc::clone(&style), 80, 1);
    let events = play(&mut high, 4.0 * MEASURE);
    assert_eq!(
        on_notes(&events),
        vec![(0.0, 72), (2.0, 72), (4.0, 72), (6.0, 72)]
    );
    assert!(high.choices().iter().all(|c| c.pattern == 1));
    assert!(high.finished());
}

#[test]
fn a_change_mid_measure_applies_at_the_next_measure() {
    let style = Arc::new(two_pattern_style());
    let mut perf = StylePerformance::new(&segment(4), style, 20, 1);
    // Half of measure 0 at groove 20: its pattern is the first.
    let mut events = play(&mut perf, 1.0);
    perf.set_groove_level(80);
    assert_eq!(perf.groove_level(), 80);
    // Measure 0 keeps its pattern; measures 1 onward take the second.
    events.extend(play(&mut perf, 3.0));
    events.extend(play(&mut perf, 4.0));
    assert_eq!(
        perf.choices(),
        &[
            PatternChoice {
                measure: 0,
                pattern: 0,
                groove: 20
            },
            PatternChoice {
                measure: 1,
                pattern: 1,
                groove: 80
            },
            PatternChoice {
                measure: 2,
                pattern: 1,
                groove: 80
            },
            PatternChoice {
                measure: 3,
                pattern: 1,
                groove: 80
            },
        ]
    );
    assert_eq!(
        on_notes(&events),
        vec![(0.0, 60), (2.0, 72), (4.0, 72), (6.0, 72)]
    );
    assert!(perf.finished());
    assert_eq!(perf.elapsed(), 8.0);
}

#[test]
fn a_level_no_pattern_covers_plays_nothing() {
    let style = Arc::new(two_pattern_style());
    let mut perf = StylePerformance::new(&segment(2), style, 0, 1);
    assert!(play(&mut perf, 2.0 * MEASURE).is_empty());
    assert!(perf.choices().is_empty());
    assert!(perf.finished());
}

#[test]
fn a_pattern_loops_over_the_segment_by_its_length() {
    // A two-measure pattern: a note of 60 in its first measure and 62 in its second.
    let style = Arc::new(style(
        vec![part(1, 2, vec![note(0, 60), note(16, 62)])],
        vec![pattern(1, 100, 2, &[(1, 0)])],
    ));
    let mut perf = StylePerformance::new(&segment(4), style, 50, 1);
    let events = play(&mut perf, 4.0 * MEASURE);
    assert_eq!(
        on_notes(&events),
        vec![(0.0, 60), (2.0, 62), (4.0, 60), (6.0, 62)]
    );
}

#[test]
fn the_band_assigns_the_patch_and_transposes() {
    let mut style = two_pattern_style();
    style.bands.push(Band {
        instruments: vec![BandInstrument {
            patch: 5,
            assign_patch: 0,
            note_ranges: [0; 4],
            pchannel: 0,
            flags: 0,
            pan: 0,
            volume: 0,
            transpose: 12,
            channel_priority: 0,
            pitch_bend_range: 0,
            collection: None,
        }],
        ..Band::default()
    });
    let mut perf = StylePerformance::new(&segment(1), Arc::new(style), 20, 1);
    let events = play(&mut perf, MEASURE);
    assert_eq!(
        events[0].event,
        MusicEvent::NoteOn {
            channel: 0,
            instrument: Some(5),
            note: 72,
            velocity: 100,
        }
    );
}

#[test]
fn a_variation_the_part_does_not_allow_is_silent() {
    // Only variation 1 is allowed; the first note plays in variation 0 alone, the second in 1.
    let mut only_zero = note(0, 60);
    only_zero.variation = 0b01;
    let mut only_one = note(4, 62);
    only_one.variation = 0b10;
    let mut p = part(1, 1, vec![only_zero, only_one]);
    p.variation_choices[0] = 0;
    let style = Arc::new(style(vec![p], vec![pattern(1, 100, 1, &[(1, 0)])]));
    let mut perf = StylePerformance::new(&segment(1), style, 20, 1);
    let events = play(&mut perf, MEASURE);
    // The second note is a quarter (grid 4) in: half a second at 120 BPM.
    assert_eq!(on_notes(&events), vec![(0.5, 62)]);
}

#[test]
fn covering_patterns_are_chosen_at_random_from_one_seed() {
    let style = Arc::new(style(
        vec![part(1, 1, vec![note(0, 60)]), part(2, 1, vec![note(0, 72)])],
        vec![pattern(1, 100, 1, &[(1, 0)]), pattern(1, 100, 1, &[(2, 0)])],
    ));
    let seg = segment(16);
    let mut a = StylePerformance::new(&seg, Arc::clone(&style), 60, 7);
    let mut b = StylePerformance::new(&seg, Arc::clone(&style), 60, 7);
    let events_a = play(&mut a, 16.0 * MEASURE);
    let events_b = play(&mut b, 16.0 * MEASURE);
    assert_eq!(events_a, events_b);
    let picked: Vec<usize> = a.choices().iter().map(|c| c.pattern).collect();
    assert_eq!(picked.len(), 16);
    assert!(picked.contains(&0) && picked.contains(&1), "{picked:?}");
}

/// A segment of `measures` measures whose chord track has the chords `(time, chord root)`, each
/// a major triad in the major scale.
fn segment_with_chords(measures: i32, chords: &[(i32, u8)]) -> Segment {
    let mut seg = segment(measures);
    seg.tracks.push(Track {
        class: guid(0),
        position: 0,
        group: 0,
        config: 0,
        priority: 0,
        data: TrackData::Chords {
            scale: 0x00AB_5AB5,
            chords: chords
                .iter()
                .map(|&(time, root)| Chord {
                    name: String::new(),
                    time,
                    measure: 0,
                    beat: 0,
                    flags: 0,
                    subchords: vec![SubChord {
                        chord_pattern: 0x91,
                        scale_pattern: 0xAB5,
                        levels: u32::MAX,
                        chord_root: root,
                        ..SubChord::default()
                    }],
                })
                .collect(),
        },
    });
    seg
}

/// A one-measure style: one NORMALCHORD (0x0E) part playing `notes`, in a pattern that covers
/// every groove level.
fn chord_style(notes: Vec<StyleNote>) -> Style {
    let mut p = part(1, 1, notes);
    p.play_mode = 0x0E;
    style(vec![p], vec![pattern(1, 100, 1, &[(1, 0)])])
}

#[test]
fn a_chord_change_mid_segment_changes_later_notes_only() {
    // Tone 2 of a NORMALCHORD part is the chord's fifth in C major (G, 67) and the third tone
    // of G major (D, 74). The chord changes to G at measure 2.
    let style = Arc::new(chord_style(vec![note(0, 0x0200)]));
    let seg = segment_with_chords(4, &[(0, 0), (2 * 3072, 7)]);
    let mut perf = StylePerformance::new(&seg, style, 50, 1);
    let events = play(&mut perf, 4.0 * MEASURE);
    assert_eq!(
        on_notes(&events),
        vec![(0.0, 67), (2.0, 67), (4.0, 74), (6.0, 74)]
    );
}

#[test]
fn a_segment_with_no_chord_track_plays_over_c_major() {
    let style = Arc::new(chord_style(vec![note(0, 0x0200)]));
    let mut perf = StylePerformance::new(&segment(2), style, 50, 1);
    let events = play(&mut perf, 2.0 * MEASURE);
    assert_eq!(on_notes(&events), vec![(0.0, 67), (2.0, 67)]);
}
