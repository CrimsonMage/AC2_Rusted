//! A stop of a primary on synthetic segments and through the sound manager (no game data). Retail's
//! `DXMusicMessageThread::Stop` on the current primary only clears its loop, so the pass plays out
//! to its end whatever boundary the stop names; the sound is cut only when the queue ends it (the
//! manager's flush waits for that end). A stop of a secondary is still immediate here.

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
fn an_immediate_stop_of_a_primary_plays_its_pass_out() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    assert!(thread.play(info(SegmentKind::Primary, 1, &two_measures())));
    let before = run(&mut thread, 0.5, &mut calls);
    assert_eq!(note_ons(&before), vec![0.0, 0.5]);
    thread.stop(1, Boundary::Immediate);
    assert!(thread.is_playing(1), "the stop only clears the loop");
    let after = run(&mut thread, 5.0, &mut calls);
    assert_eq!(note_ons(&after), vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5]);
    assert!(
        after.iter().any(|(t, e)| *t == 4.0
            && matches!(
                e,
                MusicEvent::NoteOff {
                    channel: 0,
                    note: 67
                }
            )),
        "the last note releases where the pass ends: {after:?}"
    );
    assert!(!thread.is_playing(1));
    assert!(calls.0.contains(&("ended", 1)));
}

#[test]
fn a_primary_stopped_at_a_measure_plays_its_pass_out_to_the_end() {
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.use_time(0.0, Some(&mut calls));
    assert!(thread.play(info(SegmentKind::Primary, 1, &two_measures())));
    let _ = run(&mut thread, 0.5, &mut calls);
    // Half a second in, the next measure is at two seconds; retail's stop does not stop there.
    thread.stop(1, Boundary::Measure);
    assert!(thread.is_playing(1), "it plays out, it is not cut");
    let played = run(&mut thread, 2.5, &mut calls);
    assert_eq!(note_ons(&played), vec![1.0, 1.5, 2.0, 2.5]);
    assert!(thread.is_playing(1), "still playing past the measure");
    let end = run(&mut thread, 4.5, &mut calls);
    assert!(
        end.iter().all(|(t, _)| *t <= 4.0 + 1e-9),
        "nothing after the pass: {end:?}"
    );
    assert!(end.iter().any(|(t, e)| *t == 4.0
        && matches!(
            e,
            MusicEvent::NoteOff {
                channel: 0,
                note: 67
            }
        )));
    assert!(!thread.is_playing(1));
    assert!(calls.0.contains(&("ended", 1)));
}

#[test]
fn a_flushed_primary_plays_out_to_its_end_before_the_sound_is_cut() {
    let did = 0x2C00_0043;
    let music = LoadedSound::Music {
        info: MusicInfo {
            did,
            // The stop timing bit for a measure (`m_stopTiming`): the queue ignores it for a primary.
            bit_field: 0x4000,
            priority: 1.0,
            min_distance: 1.0,
            kind: 0,
            segment: 0x2400_0043,
        },
        segment: Some(two_measures()),
        sources: Arc::from(Vec::new()),
        bands: Arc::from(Vec::new()),
        cues: Arc::default(),
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
            SinkCommand::StopMusic { voice, boundary: Boundary::End } if *voice == handle
        )),
        "the stop goes to the voice as a play-out, not a cut"
    );
    manager.use_time(3.5);
    assert!(
        !sink
            .commands()
            .iter()
            .any(|c| matches!(c, SinkCommand::MusicEnded { .. })),
        "the sound is not cut before the pass ends"
    );
    // The queue reports the end in this frame; the sound is flushed at the next frame's start.
    manager.use_time(4.5);
    manager.use_time(4.6);
    assert!(
        sink.commands()
            .iter()
            .any(|c| matches!(c, SinkCommand::MusicEnded { voice } if *voice == handle)),
        "the sound is cut once the pass has played out"
    );
    let late = sink
        .commands()
        .iter()
        .filter(|c| matches!(c, SinkCommand::Note(e) if e.seconds > 4.0 + 1e-9))
        .count();
    assert_eq!(late, 0, "no note after the pass");
}
