//! A stop of a secondary segment at its boundary (`DXMusicMessageThread::Stop` calls
//! `StopEx(state, 0, dwStopFlags)`, the flags from `DirectXPlayingSound::StopInternal`'s timing
//! switch): the pass plays on to the next grid, beat, measure, marker or segment end, and its notes
//! release there; a forced or immediate stop ends it now. Synthetic segments (no game data).

use std::collections::HashMap;
use std::sync::Arc;

use super::tests::{info, run, seq, sequence, Calls, Musics, QUARTER};
use super::{MusicThread, SegmentKind};
use crate::music::library::LoadedMusic;
use crate::music::performance::MusicEvent;
use crate::music::riff::Guid;
use crate::music::segment::{Track, TrackData};
use crate::music::MusicObject;
use crate::play_data::Track as SoundTrack;
use crate::sink::{Boundary, SinkCommand};
use crate::{LoadedSound, MusicInfo, PlaySoundData, RecordingSink, SoundManager};

const WAIT: std::time::Duration = std::time::Duration::from_secs(5);
/// A quarter is half a second at the default 120 quarter notes a minute.
const TOLERANCE: f64 = 1e-9;

/// Two measures (4/4, 2 s each) of quarter notes on channel 0, note 60 up: the note sounding in
/// quarter `q` is 60 + q, from `q / 2` s to `(q + 1) / 2` s.
fn two_measures() -> Arc<LoadedMusic> {
    let notes = (0..8)
        .map(|q| seq(q * QUARTER, QUARTER, 0, 60 + q as u8))
        .collect();
    sequence(8 * QUARTER, notes)
}

/// `music` with a marker track at `ticks` (the `MARK` list's times).
fn with_markers(music: &Arc<LoadedMusic>, ticks: Vec<i32>) -> Arc<LoadedMusic> {
    let MusicObject::Segment(segment) = &music.segment else {
        panic!("a segment");
    };
    let mut segment = (**segment).clone();
    segment.tracks.push(Track {
        class: Guid([9; 16]),
        position: 0,
        group: 0,
        config: 0,
        priority: 0,
        data: TrackData::Markers(ticks),
    });
    Arc::new(LoadedMusic {
        segment: MusicObject::Segment(Arc::new(segment)),
        objects: music.objects.clone(),
        missing: 0,
    })
}

fn note_off_at(events: &[(f64, MusicEvent)], at: f64, note: u8) -> bool {
    events.iter().any(|(t, e)| {
        (*t - at).abs() < TOLERANCE
            && matches!(
                e,
                MusicEvent::NoteOff {
                    channel: 0,
                    note: n
                } if *n == note
            )
    })
}

/// A queue at time 0 with secondary segment 5 playing `music`, run to `until` seconds.
fn playing_until(music: &Arc<LoadedMusic>, until: f64) -> (MusicThread, Calls) {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    run(&mut thread, 0.0, &mut calls);
    assert!(thread.play(info(SegmentKind::Secondary, 5, music)));
    run(&mut thread, until, &mut calls);
    (thread, calls)
}

#[test]
fn a_secondary_stopped_at_a_beat_plays_on_to_the_next_beat() {
    // Stopped at 0.25 s; the beat is at 0.5 s, where quarter 0 (note 60) releases.
    let (mut thread, mut calls) = playing_until(&two_measures(), 0.25);
    thread.stop(5, Boundary::Beat);
    assert!(thread.is_playing(5), "the stop waits for the beat");
    let before = run(&mut thread, 0.4, &mut calls);
    assert!(thread.is_playing(5), "still playing before the beat");
    assert!(before.is_empty(), "nothing new before the beat: {before:?}");
    let after = run(&mut thread, 0.6, &mut calls);
    assert!(!thread.is_playing(5), "ended at the beat");
    assert!(calls.0.contains(&("ended", 5)));
    assert!(
        after.iter().all(|(t, _)| *t <= 0.5 + TOLERANCE),
        "{after:?}"
    );
    assert!(
        note_off_at(&after, 0.5, 60),
        "the note releases at the beat"
    );
}

#[test]
fn a_secondary_stopped_at_a_measure_plays_on_to_the_measure() {
    // Stopped at 0.5 s; the measure is at 2 s, where quarter 3 (note 63) releases.
    let (mut thread, mut calls) = playing_until(&two_measures(), 0.5);
    thread.stop(5, Boundary::Measure);
    let played = run(&mut thread, 1.9, &mut calls);
    assert!(thread.is_playing(5), "still playing before the measure");
    assert!(played.iter().all(|(t, _)| *t <= 1.9 + TOLERANCE));
    let end = run(&mut thread, 2.1, &mut calls);
    assert!(!thread.is_playing(5), "ended at the measure");
    assert!(end.iter().all(|(t, _)| *t <= 2.0 + TOLERANCE), "{end:?}");
    assert!(
        note_off_at(&end, 2.0, 63),
        "the last note releases at the measure"
    );
    assert!(calls.0.contains(&("ended", 5)));
}

#[test]
fn a_secondary_stopped_at_a_grid_plays_on_to_the_next_grid() {
    // A grid is 192 ticks (an eighth of a beat's quarter: 0.125 s). Stopped at 0.3 s, the next
    // grid is at 0.375 s.
    let (mut thread, mut calls) = playing_until(&two_measures(), 0.3);
    thread.stop(5, Boundary::Grid);
    run(&mut thread, 0.36, &mut calls);
    assert!(thread.is_playing(5), "the grid is at 0.375 s");
    let end = run(&mut thread, 0.4, &mut calls);
    assert!(!thread.is_playing(5), "ended at the grid");
    assert!(end.iter().all(|(t, _)| *t <= 0.375 + TOLERANCE), "{end:?}");
}

#[test]
fn a_secondary_stopped_at_its_end_plays_its_whole_pass() {
    // The end (`DMUS_SEGF_SEGMENTEND`) is the segment's length, 4 s: nothing is cut.
    let (mut thread, mut calls) = playing_until(&two_measures(), 0.5);
    thread.stop(5, Boundary::End);
    run(&mut thread, 3.9, &mut calls);
    assert!(thread.is_playing(5), "still playing before the end");
    let end = run(&mut thread, 4.1, &mut calls);
    assert!(!thread.is_playing(5));
    assert!(end.iter().all(|(t, _)| *t <= 4.0 + TOLERANCE), "{end:?}");
}

#[test]
fn a_secondary_stopped_at_a_marker_plays_on_to_the_next_marker() {
    // A marker at tick 3840 (2.5 s). Stopped at 0.5 s, the pass plays on to it.
    let music = with_markers(&two_measures(), vec![3840]);
    let (mut thread, mut calls) = playing_until(&music, 0.5);
    thread.stop(5, Boundary::Marker);
    run(&mut thread, 2.4, &mut calls);
    assert!(thread.is_playing(5), "still playing before the marker");
    let end = run(&mut thread, 2.6, &mut calls);
    assert!(!thread.is_playing(5), "ended at the marker");
    assert!(end.iter().all(|(t, _)| *t <= 2.5 + TOLERANCE), "{end:?}");
}

#[test]
fn a_marker_already_passed_is_not_the_stop_point() {
    // The only marker (0.25 s) is behind the stop at 0.5 s: the pass plays on to its end (4 s).
    let music = with_markers(&two_measures(), vec![384]);
    let (mut thread, mut calls) = playing_until(&music, 0.5);
    thread.stop(5, Boundary::Marker);
    run(&mut thread, 3.9, &mut calls);
    assert!(
        thread.is_playing(5),
        "no marker is left: the pass ends at 4 s"
    );
    run(&mut thread, 4.1, &mut calls);
    assert!(!thread.is_playing(5));
}

#[test]
fn an_immediate_stop_after_a_boundary_stop_ends_the_segment_now() {
    // Stopped at a measure (2 s), then immediate at 0.75 s, mid-note: the note sounding (61, from
    // 0.5 s) releases at 0.75 s, and the segment ends there, not at the measure.
    let (mut thread, mut calls) = playing_until(&two_measures(), 0.5);
    thread.stop(5, Boundary::Measure);
    let _ = run(&mut thread, 0.75, &mut calls);
    thread.stop(5, Boundary::Immediate);
    assert!(!thread.is_playing(5), "an immediate stop ends it at once");
    let after = run(&mut thread, 1.1, &mut calls);
    assert!(note_off_at(&after, 0.75, 61), "{after:?}");
    assert!(
        after.iter().all(|(t, _)| *t <= 0.75 + TOLERANCE),
        "{after:?}"
    );
    assert!(calls.0.contains(&("ended", 5)));
}

/// A `MusicInfo` sound of `did` for the sound manager: a secondary segment, with its stop timing
/// in `bit_field` (`m_stopTiming`).
fn secondary_sound(did: u32, bit_field: u32, music: Arc<LoadedMusic>) -> LoadedSound {
    LoadedSound::Music {
        info: MusicInfo {
            did,
            bit_field,
            priority: 1.0,
            min_distance: 1.0,
            kind: 0,
            segment: 0x2400_0051,
        },
        segment: Some(music),
        sources: Arc::from(Vec::new()),
        bands: Arc::from(Vec::new()),
        cues: Arc::default(),
    }
}

#[test]
fn a_secondary_sound_stopped_at_a_measure_is_cut_when_its_pass_plays_out() {
    let did = 0x2C00_0051;
    let sink = RecordingSink::new();
    let source = Musics(HashMap::from([(
        did,
        // The stop timing bit for a measure (`0x4000`, `InitDXPSDFromMusicInfo`).
        secondary_sound(did, 0x4000, two_measures()),
    )]));
    let mut manager = SoundManager::new(Box::new(sink.clone()), Arc::new(source));
    let data = PlaySoundData {
        track: SoundTrack::ForceSecondary,
        ..PlaySoundData::new(did)
    };
    let handle = manager.play(&data).expect("the music is requested");
    assert!(manager.wait_for_loads(WAIT));
    manager.use_time(0.0);
    manager.use_time(0.5);
    assert_eq!(manager.stop_sounds(1), 1, "the secondary is stopped");
    assert!(sink.commands().iter().any(|c| matches!(
        c,
        SinkCommand::StopMusic { voice, boundary: Boundary::Measure } if *voice == handle
    )));
    manager.use_time(1.9);
    assert!(
        !sink
            .commands()
            .iter()
            .any(|c| matches!(c, SinkCommand::MusicEnded { .. })),
        "the voice is not cut before the measure"
    );
    manager.use_time(2.5);
    manager.use_time(2.6);
    assert!(
        sink.commands()
            .iter()
            .any(|c| matches!(c, SinkCommand::MusicEnded { voice } if *voice == handle)),
        "the voice ends once the measure has played out"
    );
    let late = sink
        .commands()
        .iter()
        .filter(|c| matches!(c, SinkCommand::Note(e) if e.seconds > 2.0 + TOLERANCE))
        .count();
    assert_eq!(late, 0, "no note after the measure");
}

#[test]
fn a_forced_immediate_stop_ends_the_secondary_voice_at_once() {
    let did = 0x2C00_0052;
    let sink = RecordingSink::new();
    let source = Musics(HashMap::from([(
        did,
        secondary_sound(did, 0x4000, two_measures()),
    )]));
    let mut manager = SoundManager::new(Box::new(sink.clone()), Arc::new(source));
    let data = PlaySoundData {
        track: SoundTrack::ForceSecondary,
        emitter: Some(7),
        ..PlaySoundData::new(did)
    };
    let handle = manager.play(&data).expect("the music is requested");
    assert!(manager.wait_for_loads(WAIT));
    manager.use_time(0.0);
    manager.use_time(0.5);
    // A forced stop (an emitter's sounds going: `StopSoundsForEmitter`) is immediate, despite the
    // measure the stop timing names.
    assert_eq!(manager.stop_sounds_for_emitter(7), 1);
    assert!(
        sink.commands().iter().any(|c| matches!(
            c,
            SinkCommand::StopMusic { voice, boundary: Boundary::Immediate } if *voice == handle
        )),
        "the stop goes to the voice at once"
    );
    // The voice is cut by the stop; its sound is flushed at the next frame.
    manager.use_time(1.0);
    manager.use_time(1.1);
    assert!(
        sink.commands()
            .iter()
            .any(|c| matches!(c, SinkCommand::MusicEnded { voice } if *voice == handle)),
        "the sound is flushed once the voice has ended"
    );
    let late = sink
        .commands()
        .iter()
        .filter(|c| matches!(c, SinkCommand::Note(e) if e.seconds > 0.5 + TOLERANCE))
        .count();
    assert_eq!(late, 0, "no note after the forced stop");
}
