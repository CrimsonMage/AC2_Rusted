//! The music's sound end to end, headless (a [`MixingSink`], no device): the synthetic tests
//! drive the mixer's commands directly; the DAT tests (`AC2_DAT_DIR`) queue a banded segment in
//! the sound manager and hear what it mixes. Nothing from the DATs is stored.

use std::sync::Arc;
use std::time::Duration;

use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use crate::data::{DatSounds, LoadedSound, MusicInfo};
use crate::music::clock::MusicClock;
use crate::music::dls::Collection;
use crate::music::library::{LoadedMusic, MusicLibrary};
use crate::music::performance::{MusicEvent, TimedEvent};
use crate::music::segment::TrackData;
use crate::music::MusicObject;
use crate::play_data::{Channel, PlaySoundData, Track};
use crate::sink::{Boundary, SinkCommand, SoundSink, SILENT};
use crate::synth::tests::{collection, dls_bytes, Spec};
use crate::synth::{performance_bands, style_sources, ChannelSources, Source};
use crate::{MixingSink, SoundManager, SoundPreference, SoundSource};

const RATE: u32 = 44_100;
/// A stereo block of 10 ms: the frame thread's `UseTime` step for the DAT runs.
const BLOCK_FRAMES: usize = 441;
const BLOCK: f64 = 0.01;
/// The DLS instrument's patch in the synthetic collections (`synth::tests`).
const PATCH: u32 = 5;
const WAIT: Duration = Duration::from_secs(10);

/// A sustained tone: a looping square wave at `amplitude`, one region over every key.
fn tone(amplitude: i16) -> Arc<Collection> {
    let wave: Vec<i16> = (0..256)
        .map(|i| if i < 128 { amplitude } else { -amplitude })
        .collect();
    collection(&Spec {
        wave,
        loop_: Some((0, 256)),
        release_tc: None,
        connections: Vec::new(),
    })
}

fn sources(collection: Arc<Collection>) -> ChannelSources {
    Arc::from(vec![(0u32, Source::Collection(collection))])
}

fn start(sink: &mut MixingSink, voice: u32, sources: ChannelSources, millibels: i32) {
    sink.send(SinkCommand::StartMusic {
        voice,
        music: 0,
        segment: 0,
        looping: true,
        millibels,
        boundary: Boundary::Immediate,
        primary: false,
        control: false,
        spatial: None,
        sources,
        band: Arc::from(Vec::new()),
    });
}

fn note(sink: &mut MixingSink, segment: u32, seconds: f64, on: bool) {
    let event = if on {
        MusicEvent::NoteOn {
            channel: 0,
            instrument: Some(PATCH),
            note: 60,
            velocity: 100,
        }
    } else {
        MusicEvent::NoteOff {
            channel: 0,
            note: 60,
        }
    };
    sink.send(SinkCommand::Note(TimedEvent {
        seconds,
        event,
        segment,
    }));
}

fn clock(sink: &mut MixingSink, seconds: f64) {
    sink.send(SinkCommand::MusicClock { seconds });
}

fn sounding(sink: &MixingSink, voice: u32) -> Option<usize> {
    sink.with_mixer(|m| m.music_sounding(voice)).flatten()
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |p, s| p.max(s.abs()))
}

#[test]
fn a_note_sounds_through_its_own_segments_voice() {
    let mut sink = MixingSink::new(RATE);
    start(&mut sink, 7, sources(tone(8000)), 0);
    clock(&mut sink, 0.0);
    note(&mut sink, 7, 0.0, true);
    let out = sink.render(4410, 2);
    assert!(out.iter().all(|s| s.is_finite()));
    assert!(peak(&out) > 0.01, "the note is heard");
    assert_eq!(sounding(&sink, 7), Some(1));
}

#[test]
fn a_note_is_heard_at_its_time_on_the_queue_clock() {
    let mut sink = MixingSink::new(RATE);
    start(&mut sink, 7, sources(tone(8000)), 0);
    clock(&mut sink, 1.0);
    note(&mut sink, 7, 1.1, true);
    let first = sink.render(2205, 2);
    assert_eq!(peak(&first), 0.0, "nothing before the note's time (1.1 s)");
    // From 1.05 s: the note is at frame 2205 of this 0.1 s run.
    let second = sink.render(4410, 2);
    assert_eq!(
        peak(&second[..2 * 2000]),
        0.0,
        "the frames before 2000 precede the note"
    );
    assert!(peak(&second[2 * 2400..]) > 0.01, "the note sounds after it");
}

#[test]
fn two_segments_keep_their_own_channel_bindings_and_notes() {
    let mut sink = MixingSink::new(RATE);
    start(&mut sink, 7, sources(tone(8000)), 0);
    start(&mut sink, 8, sources(tone(3000)), 0);
    let bound_a = sink.with_mixer(|m| m.music_bindings(7)).flatten();
    let bound_b = sink.with_mixer(|m| m.music_bindings(8)).flatten();
    assert!(bound_a.is_some() && bound_b.is_some());
    assert_ne!(bound_a, bound_b, "each segment binds its own channels");

    clock(&mut sink, 0.0);
    note(&mut sink, 7, 0.0, true);
    sink.render(2205, 2);
    assert_eq!((sounding(&sink, 7), sounding(&sink, 8)), (Some(1), Some(0)));
    note(&mut sink, 8, 0.05, true);
    sink.render(2205, 2);
    assert_eq!((sounding(&sink, 7), sounding(&sink, 8)), (Some(1), Some(1)));
}

#[test]
fn a_segment_at_music_volume_zero_is_silent() {
    let mut sink = MixingSink::new(RATE);
    start(&mut sink, 7, sources(tone(8000)), SILENT);
    clock(&mut sink, 0.0);
    note(&mut sink, 7, 0.0, true);
    let out = sink.render(4410, 2);
    assert_eq!(peak(&out), 0.0, "silence at music volume 0");
}

#[test]
fn a_volume_change_ramps_the_segment_to_silence() {
    let mut sink = MixingSink::new(RATE);
    start(&mut sink, 7, sources(tone(8000)), 0);
    clock(&mut sink, 0.0);
    note(&mut sink, 7, 0.0, true);
    assert!(peak(&sink.render(2205, 2)) > 0.01);
    sink.send(SinkCommand::Volume {
        voice: 7,
        millibels: SILENT,
        ramp_ms: 0,
    });
    assert_eq!(peak(&sink.render(2205, 2)), 0.0);
}

#[test]
fn an_immediate_stop_releases_the_segment_to_silence() {
    let mut sink = MixingSink::new(RATE);
    start(&mut sink, 7, sources(tone(8000)), 0);
    clock(&mut sink, 0.0);
    note(&mut sink, 7, 0.0, true);
    assert!(peak(&sink.render(4410, 2)) > 0.01);
    sink.send(SinkCommand::StopMusic {
        voice: 7,
        boundary: Boundary::Immediate,
    });
    // The release plays out, then the voice is gone and the mix is silent.
    let tail = sink.render(5 * RATE as usize, 2);
    assert!(tail.iter().all(|s| s.is_finite()));
    assert!(
        peak(&tail[tail.len() - 2 * 4410..]) == 0.0,
        "silent at the end"
    );
    assert_eq!(sink.with_mixer(|m| m.music_voices()), Some(0));
}

#[test]
fn a_boundary_stop_lets_the_queue_play_the_segment_out() {
    let mut sink = MixingSink::new(RATE);
    start(&mut sink, 7, sources(tone(8000)), 0);
    clock(&mut sink, 0.0);
    note(&mut sink, 7, 0.0, true);
    sink.render(2205, 2);
    sink.send(SinkCommand::StopMusic {
        voice: 7,
        boundary: Boundary::Measure,
    });
    assert_eq!(
        sounding(&sink, 7),
        Some(1),
        "the notes go on until the queue ends them"
    );
    // The queue's note-off, then the segment's end: the voice releases and goes.
    clock(&mut sink, 0.1);
    note(&mut sink, 7, 0.1, false);
    sink.send(SinkCommand::MusicEnded { voice: 7 });
    sink.render(5 * RATE as usize, 2);
    assert_eq!(sounding(&sink, 7), None);
}

/// Loads nothing (the synthetic tests need no sound records).
struct NoSounds;

impl SoundSource for NoSounds {
    fn load(&self, _did: u32) -> Option<LoadedSound> {
        None
    }
}

#[test]
fn a_gm_file_loads_on_a_worker_and_plays_the_gm_channels() {
    let path = std::env::temp_dir().join(format!("ac2-audio-gm-test-{}.dls", std::process::id()));
    let wave: Vec<i16> = (0..256)
        .map(|i| if i < 128 { 8000 } else { -8000 })
        .collect();
    let spec = Spec {
        wave,
        loop_: Some((0, 256)),
        release_tc: None,
        connections: Vec::new(),
    };
    std::fs::write(&path, dls_bytes(&spec)).expect("writes the test GM file");
    let sink = MixingSink::new(RATE);
    let mut manager = SoundManager::new(Box::new(sink.clone()), Arc::new(NoSounds));
    manager.load_gm(path.clone());
    let loaded = manager.wait_for_gm(WAIT);
    let _ = std::fs::remove_file(&path);
    assert!(loaded, "the GM file loads");

    let mut direct = sink.clone();
    start(
        &mut direct,
        7,
        Arc::from(vec![(0u32, Source::GeneralMidi)]),
        0,
    );
    clock(&mut direct, 0.0);
    note(&mut direct, 7, 0.0, true);
    assert!(peak(&sink.render(4410, 2)) > 0.01, "the GM channel plays");
}

#[test]
fn a_gm_channel_without_a_gm_file_stays_silent() {
    let sink = MixingSink::new(RATE);
    let mut direct = sink.clone();
    start(
        &mut direct,
        7,
        Arc::from(vec![(0u32, Source::GeneralMidi)]),
        0,
    );
    clock(&mut direct, 0.0);
    note(&mut direct, 7, 0.0, true);
    assert_eq!(peak(&sink.render(4410, 2)), 0.0, "no GM set, no sound");
    assert_eq!(sounding(&sink, 7), Some(0), "no GM set, so no voice starts");
}

// ---- the DATs -------------------------------------------------------------------------------

/// The `MusicInfo` DID a band segment is queued under (the DATs' band segments have none).
const BAND_DID: u32 = 0x2C00_0001;

fn store() -> Option<Arc<ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ResourceStore::new(dataset)))
}

/// The DAT's sounds, plus the band segment queued under [`BAND_DID`].
struct WithBand {
    band: LoadedSound,
    dats: DatSounds,
}

impl SoundSource for WithBand {
    fn load(&self, did: u32) -> Option<LoadedSound> {
        if did == BAND_DID {
            return Some(self.band.clone());
        }
        self.dats.load(did)
    }
}

/// The length of a segment's first measure, in seconds.
fn measure_seconds(music: &LoadedMusic) -> f64 {
    let MusicObject::Segment(segment) = &music.segment else {
        return 0.0;
    };
    let clock = MusicClock::from_segment(segment);
    clock.seconds_at(MusicClock::measure_ticks(&clock.signature_at(0)))
}

/// A sound manager over `source`, its speakers a [`MixingSink`] rendered by the test in 10 ms
/// steps (the frame thread's `use_time`, then the callback's render, in turn).
struct Session {
    manager: SoundManager,
    sink: MixingSink,
    now: f64,
}

impl Session {
    fn new(source: Arc<dyn SoundSource>) -> Self {
        let sink = MixingSink::new(RATE);
        let manager = SoundManager::new(Box::new(sink.clone()), source);
        Self {
            manager,
            sink,
            now: 0.0,
        }
    }

    /// Queues `did` on the music channel as a secondary segment, and waits for its load (it
    /// starts at the next `run`).
    fn play(&mut self, did: u32) -> u32 {
        let data = PlaySoundData {
            channel: Channel::Music,
            track: Track::ForceSecondary,
            ..PlaySoundData::new(did)
        };
        let handle = self.manager.play(&data).expect("the music is requested");
        assert!(self.manager.wait_for_loads(WAIT), "the segment loads");
        handle
    }

    /// `seconds` of the mix, stereo, rendered in 10 ms steps.
    fn run(&mut self, seconds: f64) -> Vec<f32> {
        let blocks = (seconds / BLOCK).round() as usize;
        let mut out = Vec::with_capacity(blocks * BLOCK_FRAMES * 2);
        for _ in 0..blocks {
            self.manager.use_time(self.now);
            out.extend(self.sink.render(BLOCK_FRAMES, 2));
            self.now += BLOCK;
        }
        out
    }
}

/// A DAT band segment, as a band-track `MusicInfo` would queue it: its closure, its band's
/// sources and the performance band it makes, under [`BAND_DID`]. `None` when its band binds no
/// DLS collection.
fn band_sound(band_segment: u32, store: &Arc<ResourceStore>) -> Option<LoadedSound> {
    let library = MusicLibrary::new(Arc::clone(store));
    let record = library.record(band_segment)?;
    let music = Arc::new(library.closure(record));
    let bands = performance_bands(&music);
    let (_, first) = bands.first()?;
    if !first
        .sources
        .iter()
        .any(|(_, source)| matches!(source, Source::Collection(_)))
    {
        return None;
    }
    Some(LoadedSound::Music {
        info: MusicInfo {
            did: BAND_DID,
            bit_field: 0,
            priority: 1.0,
            min_distance: 1.0,
            kind: 0,
            segment: band_segment,
        },
        sources: style_sources(&music).into(),
        bands: bands.into(),
        segment: Some(music),
    })
}

/// A DAT note segment queued under its own `MusicInfo`, with no band of its own (its notes play
/// on the performance's band), and the first measure's length.
struct NoteSegment {
    did: u32,
    measure: f64,
}

/// Whether the note events of `music` (a note segment with no band) resolve to a sounding
/// instrument of `band`: a note's channel and patch name a collection the band binds.
fn resolves(music: &LoadedMusic, band: &LoadedSound) -> bool {
    let (MusicObject::Segment(segment), LoadedSound::Music { bands, .. }) = (&music.segment, band)
    else {
        return false;
    };
    let Some((_, first)) = bands.first() else {
        return false;
    };
    let sources = &first.sources;
    let clock = MusicClock::from_segment(segment);
    crate::music::performance::note_events(segment, &clock, Some(&first.band))
        .iter()
        .any(|e| match e.event {
            MusicEvent::NoteOn {
                channel,
                instrument: Some(patch),
                ..
            } => sources.iter().any(|(c, source)| {
                *c == channel
                    && match source {
                        Source::Collection(collection) => collection.instrument(patch).is_some(),
                        _ => false,
                    }
            }),
            _ => false,
        })
}

/// Finds a band segment and a note segment that play together audibly: the note segment's
/// notes resolve to the band's instruments, and they sound in its first measure. `None` when no
/// pair does.
fn audible_pair(store: &Arc<ResourceStore>) -> Option<(LoadedSound, NoteSegment)> {
    let dats = DatSounds::new(Arc::clone(store));
    let library = MusicLibrary::new(Arc::clone(store));
    let bands: Vec<LoadedSound> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .filter(|&id| {
            matches!(library.record(id), Some(MusicObject::Segment(s))
                if s.tracks.iter().any(|t| matches!(t.data, TrackData::Bands(_))))
        })
        .filter_map(|id| band_sound(id, store))
        .collect();
    let notes: Vec<(u32, Arc<LoadedMusic>)> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2C00_0000, 0x2CFF_FFFE)
        .filter_map(|did| match dats.load(did) {
            Some(LoadedSound::Music {
                segment: Some(music),
                bands,
                ..
            }) if bands.is_empty() => Some((did, music)),
            _ => None,
        })
        .filter(|(_, music)| {
            matches!(&music.segment, MusicObject::Segment(s)
                if s.tracks.iter().any(|t| matches!(t.data, TrackData::Sequence { .. })))
        })
        .collect();
    for band in &bands {
        for (did, music) in &notes {
            if !resolves(music, band) {
                continue;
            }
            let measure = measure_seconds(music);
            let mut session = session_for(store, band);
            session.play(BAND_DID);
            session.play(*did);
            if peak(&session.run(measure)) > 1e-4 {
                return Some((band.clone(), NoteSegment { did: *did, measure }));
            }
        }
    }
    None
}

fn session_for(store: &Arc<ResourceStore>, band: &LoadedSound) -> Session {
    Session::new(Arc::new(WithBand {
        band: band.clone(),
        dats: DatSounds::new(Arc::clone(store)),
    }))
}

#[test]
fn a_dat_note_segment_sounds_on_the_band_that_plays_with_it() {
    let Some(store) = store() else {
        return;
    };
    let Some((band, note)) = audible_pair(&store) else {
        panic!("a band segment and a note segment sound together in the first measure");
    };
    let mut session = session_for(&store, &band);
    session.play(BAND_DID);
    session.play(note.did);
    let out = session.run(note.measure);
    assert!(out.iter().all(|s| s.is_finite()), "finite samples");
    assert!(peak(&out) > 1e-4, "non-silent within the first measure");
}

#[test]
fn music_volume_zero_silences_a_dat_segment() {
    let Some(store) = store() else {
        return;
    };
    let Some((band, note)) = audible_pair(&store) else {
        panic!("a band segment and a note segment sound together in the first measure");
    };
    let mut session = session_for(&store, &band);
    session
        .manager
        .set_preference(SoundPreference::MusicVolume(0.0));
    session.play(BAND_DID);
    session.play(note.did);
    assert_eq!(
        peak(&session.run(note.measure)),
        0.0,
        "silence at music volume 0"
    );
}

#[test]
fn stopping_a_dat_segment_releases_it_to_silence() {
    let Some(store) = store() else {
        return;
    };
    let Some((band, note)) = audible_pair(&store) else {
        panic!("a band segment and a note segment sound together in the first measure");
    };
    let mut session = session_for(&store, &band);
    session.play(BAND_DID);
    let handle = session.play(note.did);
    assert!(peak(&session.run(note.measure)) > 1e-4);
    session.manager.stop_context(handle, 0.0);
    let tail = session.run(10.0);
    assert!(tail.iter().all(|s| s.is_finite()));
    assert_eq!(
        peak(&tail[tail.len() - 2 * BLOCK_FRAMES * 20..]),
        0.0,
        "silent after the release"
    );
}

#[test]
fn two_dat_segments_at_once_use_their_own_bindings() {
    let Some(store) = store() else {
        return;
    };
    let Some((band, note)) = audible_pair(&store) else {
        panic!("a band segment and a note segment sound together in the first measure");
    };
    let dats = DatSounds::new(Arc::clone(&store));
    // A second note segment with no band of its own.
    let second = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2C00_0000, 0x2CFF_FFFE)
        .find(|&did| {
            did != note.did
                && matches!(
                    dats.load(did),
                    Some(LoadedSound::Music {
                        segment: Some(_),
                        bands,
                        ..
                    }) if bands.is_empty()
                )
        })
        .expect("a second note segment");
    // What each voice binds: the performance band's sources (its band part), then its style's.
    let performance = match &band {
        LoadedSound::Music { bands, .. } => bands
            .first()
            .map(|(_, b)| b.sources.to_vec())
            .unwrap_or_default(),
        _ => panic!("the band segment is a music sound"),
    };
    let expected = |did: u32| -> Vec<(u32, Source)> {
        match dats.load(did) {
            Some(LoadedSound::Music { sources, .. }) => {
                performance.iter().chain(sources.iter()).cloned().collect()
            }
            _ => panic!("the segment loads again"),
        }
    };
    let mut session = session_for(&store, &band);
    session.play(BAND_DID);
    let first_handle = session.play(note.did);
    let second_handle = session.play(second);
    session.run(note.measure);
    for (did, handle) in [(note.did, first_handle), (second, second_handle)] {
        let bound = session
            .sink
            .with_mixer(|m| m.music_bindings(handle))
            .flatten();
        assert_eq!(
            bound.as_deref(),
            Some(&expected(did)[..]),
            "{did:08X} binds the performance band's sources, then its style's"
        );
    }
}

#[test]
fn a_band_change_rebinds_a_playing_music_voice() {
    let mut sink = MixingSink::new(RATE);
    start(&mut sink, 7, Arc::from(Vec::new()), 0);
    let bound = sink.with_mixer(|m| m.music_bindings(7)).flatten();
    assert_eq!(bound, Some(Vec::new()), "no band and no style yet");
    let collection = tone(1000);
    sink.send(SinkCommand::MusicBands {
        voice: 7,
        sources: sources(Arc::clone(&collection)),
    });
    let bound = sink.with_mixer(|m| m.music_bindings(7)).flatten();
    assert_eq!(
        bound,
        Some(vec![(0, Source::Collection(collection))]),
        "the voice binds the band's sources"
    );
}
