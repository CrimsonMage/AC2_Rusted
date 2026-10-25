//! A stop at a boundary on synthetic segments and through the sound manager (no game data): an
//! immediate stop cuts a primary at once; a stop at a measure plays the primary out to the next
//! measure, releasing its notes there, and the sound is cut only then (the manager's flush waits
//! for the queue's end).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use super::tests::{info, run, seq, sequence, Calls, Musics, QUARTER};
use super::{MusicThread, SegmentKind};
use crate::music::performance::MusicEvent;
use crate::play_data::Track;
use crate::sink::{Boundary, SinkCommand};
use crate::{LoadedSound, MusicInfo, PlaySoundData, RecordingSink, SoundManager};

const WAIT: Duration = Duration::from_secs(5);

/// Two measures (4/4) of quarter notes, one per quarter, on channel 0 (60 up).
fn two_measures() -> Arc<crate::music::library::LoadedMusic> {
    let notes = (0..8)
        .map(|q| seq(q * QUARTER, QUARTER, 0, 60 + q as u8))
        .collect();
    sequence(8 * QUARTER, notes)
}

fn note_ons(events: &[(f64, MusicEvent)]) -> Vec<f64> {
    events
        .iter()
        .filter_map(|(t, e)| matches!(e, MusicEvent::NoteOn { .. }).then_some(*t))
        .collect()
}

#[test]
fn an_immediate_stop_cuts_a_primary_now_and_releases_its_note() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    assert!(thread.play(info(SegmentKind::Primary, 1, &two_measures())));
    let before = run(&mut thread, 0.5, &mut calls);
    assert_eq!(note_ons(&before), vec![0.0, 0.5]);
    thread.stop(1, Boundary::Immediate);
    let after = run(&mut thread, 2.0, &mut calls);
    assert!(
        after.iter().all(|(t, _)| *t <= 0.5 + 1e-9),
        "nothing sounds after the stop: {after:?}"
    );
    assert!(after.iter().any(|(t, e)| *t == 0.5
        && matches!(
            e,
            MusicEvent::NoteOff {
                channel: 0,
                note: 61
            }
        )));
    assert!(!thread.is_playing(1));
}

#[test]
fn a_primary_stopped_at_a_measure_plays_out_to_the_measure() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    assert!(thread.play(info(SegmentKind::Primary, 1, &two_measures())));
    let _ = run(&mut thread, 0.5, &mut calls);
    // Half a second in (one quarter), the next measure is at two seconds (four quarters).
    thread.stop(1, Boundary::Measure);
    assert!(thread.is_playing(1), "it plays out, it is not cut");
    let played = run(&mut thread, 1.9, &mut calls);
    assert_eq!(note_ons(&played), vec![1.0, 1.5]);
    assert!(thread.is_playing(1));
    let end = run(&mut thread, 2.5, &mut calls);
    assert!(
        end.iter().all(|(t, _)| *t <= 2.0 + 1e-9),
        "nothing after the measure: {end:?}"
    );
    assert!(end.iter().any(|(t, e)| *t == 2.0
        && matches!(
            e,
            MusicEvent::NoteOff {
                channel: 0,
                note: 63
            }
        )));
    assert!(!thread.is_playing(1));
    assert!(calls.0.contains(&("ended", 1)));
}

#[test]
fn a_flushed_primary_plays_out_to_its_measure_before_the_sound_is_cut() {
    let did = 0x2C00_0043;
    let music = LoadedSound::Music {
        info: MusicInfo {
            did,
            // The stop timing bit for a measure (`m_stopTiming`).
            bit_field: 0x4000,
            priority: 1.0,
            min_distance: 1.0,
            kind: 0,
            segment: 0x2400_0043,
        },
        segment: Some(two_measures()),
        sources: Arc::from(Vec::new()),
        bands: Arc::from(Vec::new()),
    };
    let sink = RecordingSink::new();
    let source = Musics(HashMap::from([(did, music)]));
    let mut manager = SoundManager::new(Box::new(sink.clone()), Arc::new(source));
    let data = PlaySoundData {
        track: Track::ForcePrimary,
        ..PlaySoundData::new(did)
    };
    let handle = manager.play(&data).expect("the music is requested");
    assert!(manager.wait_for_loads(WAIT));
    manager.use_time(0.0);
    manager.use_time(0.5);
    assert_eq!(manager.stop_sounds(1), 1, "the primary is stopped");
    assert!(
        sink.commands().iter().any(|c| matches!(
            c,
            SinkCommand::StopMusic { voice, boundary: Boundary::Measure } if *voice == handle
        )),
        "the stop goes to the voice at the measure"
    );
    manager.use_time(1.0);
    assert!(
        !sink
            .commands()
            .iter()
            .any(|c| matches!(c, SinkCommand::MusicEnded { .. })),
        "the sound is not cut before its measure"
    );
    // The queue reports the end in this frame; the sound is flushed at the next frame's start.
    manager.use_time(2.5);
    manager.use_time(2.6);
    assert!(
        sink.commands()
            .iter()
            .any(|c| matches!(c, SinkCommand::MusicEnded { voice } if *voice == handle)),
        "the sound is cut once the measure has played out"
    );
    let late = sink
        .commands()
        .iter()
        .filter(|c| matches!(c, SinkCommand::Note(e) if e.seconds > 2.0 + 1e-9))
        .count();
    assert_eq!(late, 0, "no note after the measure");
}
