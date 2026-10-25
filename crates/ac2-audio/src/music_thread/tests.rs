//! The music queue on synthetic segments (no game data): the primary queue, stops, the segment-end
//! callbacks, `IsPlaying`, and the groove reaching a playing style through the sound manager.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use super::{MusicHandler, MusicThread, SegmentInfo, SegmentKind};
use crate::music::band::Reference;
use crate::music::library::LoadedMusic;
use crate::music::performance::{MusicEvent, TimedEvent};
use crate::music::riff::Guid;
use crate::music::segment::{SeqItem, Track, TrackData};
use crate::music::style::Style;
use crate::music::style_tests::{guid, note, part, pattern, segment, style, two_pattern_style};
use crate::music::MusicObject;
use crate::sink::Boundary;
use crate::{LoadedSound, MusicInfo, PlaySoundData, RecordingSink, SoundManager, SoundSource};

pub(super) const QUARTER: i32 = 768;
const WAIT: Duration = Duration::from_secs(5);

pub(super) fn seq(time: i32, duration: i32, pchannel: u32, note: u8) -> SeqItem {
    SeqItem {
        time,
        duration,
        pchannel,
        offset: 0,
        status: 0x90,
        byte1: note,
        byte2: 100,
    }
}

fn track(class: u8, data: TrackData) -> Track {
    Track {
        class: Guid([class; 16]),
        position: 0,
        group: 0,
        config: 0,
        priority: 0,
        data,
    }
}

/// A 120 BPM sequence segment of `length` ticks (a quarter is half a second).
pub(super) fn sequence(length: i32, notes: Vec<SeqItem>) -> Arc<LoadedMusic> {
    let mut seg = segment(0);
    seg.length = length;
    seg.tracks = vec![
        track(1, TrackData::Tempo(vec![(0, 120.0)])),
        track(
            2,
            TrackData::Sequence {
                events: notes,
                curves: Vec::new(),
            },
        ),
    ];
    Arc::new(LoadedMusic {
        segment: MusicObject::Segment(Arc::new(seg)),
        objects: HashMap::new(),
        missing: 0,
    })
}

/// A segment of `measures` 4/4 measures that plays `style`, which its styles track references.
fn styled(style: Style, measures: i32) -> Arc<LoadedMusic> {
    let style_guid = style.guid.expect("a styled segment's style has a guid");
    let mut seg = segment(measures);
    seg.tracks = vec![track(
        3,
        TrackData::Styles(vec![(
            0,
            Reference {
                class: guid(9),
                valid: 0,
                guid: Some(style_guid),
                name: None,
                file: None,
            },
        )]),
    )];
    let mut objects = HashMap::new();
    objects.insert(style_guid, MusicObject::Style(Arc::new(style)));
    Arc::new(LoadedMusic {
        segment: MusicObject::Segment(Arc::new(seg)),
        objects,
        missing: 0,
    })
}

pub(super) fn info(kind: SegmentKind, handle: u32, music: &Arc<LoadedMusic>) -> SegmentInfo {
    SegmentInfo {
        kind,
        handle,
        segment: Arc::clone(music),
        bands: Arc::from(Vec::new()),
        cues: Arc::default(),
    }
}

/// Every callback the queue made, in order.
#[derive(Default)]
pub(super) struct Calls(pub(super) Vec<(&'static str, u32)>);

impl MusicHandler for Calls {
    fn on_segment_start(&mut self, handle: u32) {
        self.0.push(("start", handle));
    }
    fn on_segment_almost_end(&mut self, handle: u32) {
        self.0.push(("almost", handle));
    }
    fn on_segment_ended(&mut self, handle: u32) {
        self.0.push(("ended", handle));
    }
}

/// Runs the queue to `now` and returns the events sent since the last call, in time order.
pub(super) fn run(thread: &mut MusicThread, now: f64, calls: &mut Calls) -> Vec<(f64, MusicEvent)> {
    thread.use_time(now, Some(calls));
    let mut sink: Vec<TimedEvent> = Vec::new();
    thread.send_events(&mut sink);
    sink.iter().map(|e| (e.seconds, e.event)).collect()
}

pub(super) fn on(channel: u32, note: u8) -> MusicEvent {
    MusicEvent::NoteOn {
        channel,
        instrument: None,
        note,
        velocity: 100,
    }
}

pub(super) fn off(channel: u32, note: u8) -> MusicEvent {
    MusicEvent::NoteOff { channel, note }
}

#[test]
fn a_primary_queued_behind_one_starts_at_the_end_of_that_one() {
    // A lasts two quarters (1 s) and B one quarter (0.5 s).
    let a = sequence(2 * QUARTER, vec![seq(0, QUARTER, 0, 60)]);
    let b = sequence(QUARTER, vec![seq(0, QUARTER, 1, 64)]);
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    run(&mut thread, 0.0, &mut calls);
    assert!(thread.play(info(SegmentKind::Primary, 1, &a)));
    // B waits behind A: it is not playing yet.
    assert!(thread.play(info(SegmentKind::Primary, 2, &b)));
    assert!(!thread.is_playing(2));
    let events = run(&mut thread, 1.2, &mut calls);
    assert_eq!(
        events,
        vec![
            (0.0, on(0, 60)),
            (0.5, off(0, 60)),
            // B starts at A's end, not before it.
            (1.0, on(1, 64)),
        ]
    );
    assert_eq!(
        calls.0,
        vec![("start", 1), ("almost", 1), ("ended", 1), ("start", 2)]
    );
    assert!(!thread.is_playing(1));
    assert!(thread.is_playing(2));
}

#[test]
fn a_primary_queued_again_aborts_the_one_it_replaces() {
    let a = sequence(2 * QUARTER, vec![]);
    let b = sequence(QUARTER, vec![]);
    let c = sequence(QUARTER, vec![seq(0, QUARTER, 2, 67)]);
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    run(&mut thread, 0.0, &mut calls);
    thread.play(info(SegmentKind::Primary, 1, &a));
    thread.play(info(SegmentKind::Primary, 2, &b));
    thread.play(info(SegmentKind::Primary, 3, &c));
    let events = run(&mut thread, 1.2, &mut calls);
    assert_eq!(events, vec![(1.0, on(2, 67))]);
    assert!(calls.0.contains(&("ended", 2)), "the waiting one aborts");
    assert!(!calls.0.contains(&("start", 2)));
    assert!(calls.0.contains(&("start", 3)));
}

#[test]
fn a_secondary_stop_ends_it_now_and_releases_its_notes() {
    let s = sequence(4 * QUARTER, vec![seq(0, 4 * QUARTER, 0, 60)]);
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    run(&mut thread, 0.0, &mut calls);
    assert!(thread.play(info(SegmentKind::Secondary, 5, &s)));
    assert_eq!(run(&mut thread, 0.25, &mut calls), vec![(0.0, on(0, 60))]);
    assert!(thread.is_playing(5));
    thread.stop(5, Boundary::Immediate);
    assert!(!thread.is_playing(5), "stopped at once");
    // The note sounding is released at the stop (a quarter-second in), not at its own end.
    assert_eq!(run(&mut thread, 0.5, &mut calls), vec![(0.25, off(0, 60))]);
    assert_eq!(calls.0, vec![("start", 5), ("ended", 5)]);
}

#[test]
fn a_stopped_primary_plays_out_its_pass_and_is_not_repeated() {
    let p = sequence(2 * QUARTER, vec![seq(0, 2 * QUARTER, 0, 60)]);
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    run(&mut thread, 0.0, &mut calls);
    thread.play(info(SegmentKind::Primary, 7, &p));
    assert_eq!(run(&mut thread, 0.5, &mut calls), vec![(0.0, on(0, 60))]);
    thread.stop(7, Boundary::End);
    // A stopped primary is not cut: it plays to its end.
    assert!(thread.is_playing(7));
    let events = run(&mut thread, 3.0, &mut calls);
    assert_eq!(events, vec![(1.0, off(0, 60))]);
    assert!(!thread.is_playing(7));
    assert_eq!(calls.0, vec![("start", 7), ("almost", 7), ("ended", 7)]);
}

#[test]
fn a_looping_primary_plays_again_at_its_end_until_stopped() {
    let p = sequence(2 * QUARTER, vec![seq(0, QUARTER, 0, 60)]);
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    run(&mut thread, 0.0, &mut calls);
    thread.play(info(SegmentKind::Primary, 4, &p));
    let events = run(&mut thread, 2.5, &mut calls);
    let ons: Vec<f64> = events
        .iter()
        .filter(|(_, e)| matches!(e, MusicEvent::NoteOn { .. }))
        .map(|(t, _)| *t)
        .collect();
    assert_eq!(ons, vec![0.0, 1.0, 2.0]);
    // Each new pass is a new segment state; the old pass's end no longer has the handle.
    assert_eq!(
        calls.0,
        vec![
            ("start", 4),
            ("almost", 4),
            ("ended", 0),
            ("start", 4),
            ("almost", 4),
            ("ended", 0),
            ("start", 4),
        ]
    );
    assert!(thread.is_playing(4));
}

#[test]
fn is_playing_follows_the_segment_and_ignores_unknown_handles() {
    let s = sequence(2 * QUARTER, vec![]);
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    run(&mut thread, 0.0, &mut calls);
    assert!(!thread.is_playing(9));
    thread.play(info(SegmentKind::Secondary, 9, &s));
    assert!(thread.is_playing(9));
    run(&mut thread, 0.5, &mut calls);
    assert!(thread.is_playing(9));
    assert!(!thread.is_playing(42), "no segment has that handle");
    run(&mut thread, 1.5, &mut calls);
    assert!(!thread.is_playing(9), "it ended at one second");
}

#[test]
fn a_segment_end_reaches_the_handler_once_and_leaves_the_queue() {
    let s = sequence(2 * QUARTER, vec![]);
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    run(&mut thread, 0.0, &mut calls);
    thread.play(info(SegmentKind::Secondary, 3, &s));
    run(&mut thread, 1.5, &mut calls);
    assert_eq!(calls.0, vec![("start", 3), ("almost", 3), ("ended", 3)]);
    assert!(thread.choices(3).is_none(), "the segment left the queue");
    run(&mut thread, 2.5, &mut calls);
    assert_eq!(calls.0.len(), 3, "the end is reported once");
}

#[test]
fn a_queue_without_a_handler_keeps_its_events_for_the_next_handler() {
    let s = sequence(QUARTER, vec![]);
    let mut thread = MusicThread::new();
    thread.use_time(0.0, None);
    thread.play(info(SegmentKind::Secondary, 6, &s));
    thread.use_time(1.0, None);
    let mut calls = Calls::default();
    thread.use_time(1.0, Some(&mut calls));
    assert_eq!(calls.0, vec![("start", 6), ("almost", 6), ("ended", 6)]);
}

#[test]
fn a_shut_down_queue_plays_nothing() {
    let s = sequence(QUARTER, vec![]);
    let mut thread = MusicThread::new();
    thread.shutdown();
    assert!(!thread.play(info(SegmentKind::Secondary, 1, &s)));
    assert!(!thread.is_playing(1));
}

#[test]
fn the_groove_reaches_a_playing_style_at_its_next_measure() {
    // Two patterns: 1..=50 (a note of 60) and 51..=100 (a note of 72), a measure each.
    let mut two = two_pattern_style();
    two.guid = Some(guid(7));
    let s = styled(two, 2);
    let measure = 2.0;
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.set_groove_level(20);
    run(&mut thread, 0.0, &mut calls);
    thread.play(info(SegmentKind::Secondary, 4, &s));
    run(&mut thread, 0.1, &mut calls);
    assert_eq!(thread.groove_level(), 20);
    let choices = thread.choices(4).expect("a style plays");
    assert_eq!(
        (choices[0].measure, choices[0].pattern, choices[0].groove),
        (0, 0, 20)
    );
    // The level changes mid-measure: measure 1 follows it.
    thread.set_groove_level(80);
    run(&mut thread, measure + 0.1, &mut calls);
    let choices = thread.choices(4).expect("a style plays");
    assert_eq!(
        (choices[1].measure, choices[1].pattern, choices[1].groove),
        (1, 1, 80)
    );
}

#[test]
fn a_level_no_pattern_covers_plays_nothing() {
    let mut only_high = style(
        vec![part(1, 1, vec![note(0, 72)])],
        vec![pattern(51, 100, 1, &[(1, 0)])],
    );
    only_high.guid = Some(guid(8));
    let s = styled(only_high, 1);
    let mut thread = MusicThread::new();
    let mut calls = Calls::default();
    thread.set_groove_level(10);
    run(&mut thread, 0.0, &mut calls);
    thread.play(info(SegmentKind::Secondary, 2, &s));
    let events = run(&mut thread, 1.0, &mut calls);
    assert!(events.is_empty(), "no pattern covers level 10");
    assert!(thread.choices(2).is_some_and(|c| c.is_empty()));
}

/// Sounds made up for the sound-manager test: each DID is one music segment.
pub(super) struct Musics(pub(super) HashMap<u32, LoadedSound>);

impl SoundSource for Musics {
    fn load(&self, did: u32) -> Option<LoadedSound> {
        self.0.get(&did).cloned()
    }
}

#[test]
fn the_sound_manager_forwards_the_groove_and_its_tweaks_to_a_playing_style() {
    let mut two = two_pattern_style();
    two.guid = Some(guid(7));
    let did = 0x2C00_0042;
    let music = LoadedSound::Music {
        info: MusicInfo {
            did,
            bit_field: 0,
            priority: 1.0,
            min_distance: 1.0,
            kind: 0,
            segment: 0x2400_0042,
        },
        segment: Some(styled(two, 2)),
        sources: Arc::from(Vec::new()),
        bands: Arc::from(Vec::new()),
        cues: Arc::default(),
    };
    let source = Musics(HashMap::from([(did, music)]));
    let mut manager = SoundManager::new(Box::new(RecordingSink::new()), Arc::new(source));
    // `SoundManager::SetGrooveLevel` then `TweakGrooveLevel`: the device and the queue get 80.
    manager.set_groove_level(20);
    manager.tweak_groove_level(60);
    assert_eq!(manager.music().groove_level(), 80);
    let handle = manager
        .play(&PlaySoundData::new(did))
        .expect("the music is requested");
    assert!(manager.wait_for_loads(WAIT));
    manager.use_time(0.0);
    manager.use_time(0.1);
    assert!(manager.music().is_playing(handle));
    let choices = manager.music().choices(handle).expect("a style plays");
    assert_eq!((choices[0].pattern, choices[0].groove), (1, 80));
    // Tweaked down to 10: measure 1 chooses the pattern covering 10.
    manager.tweak_groove_level(-70);
    manager.use_time(2.1);
    let choices = manager.music().choices(handle).expect("a style plays");
    assert_eq!((choices[1].pattern, choices[1].groove), (0, 10));
    // Stopping the sound ends its segment.
    manager.stop_context(handle, 0.0);
    manager.use_time(2.2);
    assert!(!manager.music().is_playing(handle));
}
