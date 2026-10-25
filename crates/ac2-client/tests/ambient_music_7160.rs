//! The region's ambient music on the real DATs (`AC2_DAT_DIR`), end to end and headless: the
//! player enters the world at the double's spawn, the frames run, and the sound manager's
//! `SinkCommand`s go to a tee that records them and mixes into a [`MixingSink`] (no device).
//! Standing outdoors in the town block the land points' environment dominates and its music is
//! queued; teleported into an interior cell, the interior environment dominates
//! (`AmbientSound::ChangeCellEnd`: the densest one), its music starts, and the town's music stops
//! once its pass has played out (`NoLongerDominant` -> `StopAllSounds`; a primary's stop plays its
//! pass out, `DXMusicMessageThread::Stop`).
//!
//! The town's music is script-driven: its Cue (no notes) has a script that queues a segment behind
//! it at its end, and that segment's triggers start the notes' segments. So the first notes come at
//! the Cue's pass end (12 s). The groove level is the sum of the nearby groove objects; the empty
//! test world has none, so the harness sets [`GROOVE`] (a level the town's active patterns cover;
//! at 0 the first pattern is a rest).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use ac2_audio::{MixingSink, RecordingSink, SinkCommand, SoundSink};

/// The town's environment and its dominant-only music (`MusicInfo` 0x2C0003FB, segment 0x24000476).
const TOWN_ENV: u32 = 0x2900_0005;
const TOWN_MUSIC: u32 = 0x2C00_03FB;
const TOWN_SEGMENT: u32 = 0x2400_0476;
/// The interior environment's music (`MusicInfo` 0x2C0003F8, segment 0x24000470).
const INTERIOR_ENV: u32 = 0x2900_0003;
const INTERIOR_MUSIC: u32 = 0x2C00_03F8;
const INTERIOR_SEGMENT: u32 = 0x2400_0470;
/// An interior cell of the landblock 0299 (the double's default spawn).
const INTERIOR_CELL: u32 = 0x0299_0109;
const INTERIOR_ORIGIN: [f32; 3] = [59.0, 240.0, -44.5];
/// The groove level of the test world (see the module doc).
const GROOVE: i32 = 30;
/// The town's Cue plays silently for its pass (12 s); its notes start after it.
const FIRST_NOTES_WITHIN: f64 = 16.0;
/// A frame of the double's clock (`Client::use_time` runs 64 of them a second).
const FRAME: f64 = 1.0 / 64.0;
const FRAME_FRAMES: usize = 44_100 / 64;
const LOAD_TIMEOUT: Duration = Duration::from_secs(30);
/// The region's `CSoundDesc`: the ambient sound types of the landblocks and their records.
const REGION_SOUND_DESC: u32 = 0x1000_0000;

/// Forwards every command to a mixer and keeps a copy of each one.
#[derive(Clone)]
struct TeeSink {
    recorded: RecordingSink,
    mixer: MixingSink,
}

impl SoundSink for TeeSink {
    fn send(&mut self, command: SinkCommand) {
        self.recorded.send(command.clone());
        self.mixer.send(command);
    }
}

/// The client with its double, its audio recorded and mixed.
struct Harness {
    client: ac2_client::Client,
    server: Rc<RefCell<legions_server_double::State>>,
    recorded: RecordingSink,
    mixer: MixingSink,
}

fn town_spawn() -> Option<legions_server_double::Spawn> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset);
    let world = ac2_client::DatWorld::new(Arc::new(store)).ok()?;
    let top = world
        .world()
        .load_landblock(ac2_core::CellId::new(0xCB4D_0001))
        .ok()??
        .height_grid()
        .into_iter()
        .fold(0.0, f32::max);
    Some(legions_server_double::Spawn {
        cell: 0xCB4D_0025,
        origin: [96.0, 96.0, top + 2.0],
        rotation: [1.0, 0.0, 0.0, 0.0],
    })
}

impl Harness {
    fn enter(spawn: legions_server_double::Spawn) -> Option<Self> {
        let dataset = ac2_dat::test_support::env_dataset()?;
        let store = Arc::new(ac2_db::ResourceStore::new(dataset));
        let world = ac2_client::DatWorld::new(Arc::clone(&store)).ok()?;
        let (transport, server) = legions_server_double::double(spawn);
        let config = ac2_weenie::OnlineConfig {
            server: legions_server_double::addr(legions_server_double::SERVER),
            account: "local".into(),
            credential: ac2_net::Credential::Account,
            version: ac2_net::LEGIONS_VERSION.into(),
            character: ac2_weenie::CharacterChoice::First,
            server_key: None,
            language: 0,
            dat_iterations: Vec::new(),
        };
        let session =
            ac2_weenie::OnlineSession::with_transport(config, Box::new(transport)).ok()?;
        let mut client = ac2_client::Client::new(session, Arc::new(world));
        let recorded = RecordingSink::new();
        let mixer = MixingSink::new(44_100);
        let tee = TeeSink {
            recorded: recorded.clone(),
            mixer: mixer.clone(),
        };
        client.attach_audio(ac2_client::sound_manager(store, Box::new(tee)));
        client.set_load_budget(LOAD_TIMEOUT);
        client
            .audio_mut()
            .expect("the client has audio")
            .set_groove_level(GROOVE);
        Some(Self {
            client,
            server,
            recorded,
            mixer,
        })
    }

    /// One frame: the loads settle, the client runs, the mix renders its frame's samples.
    /// Returns the frame's mixed samples.
    fn frame(&mut self) -> Vec<f32> {
        let idle = ac2_client::FrameInput::default();
        self.client.settle_loads(LOAD_TIMEOUT).unwrap();
        self.client.use_time(&idle).unwrap();
        self.client
            .audio_mut()
            .unwrap()
            .wait_for_loads(LOAD_TIMEOUT);
        self.server.borrow_mut().elapsed += FRAME;
        self.mixer.render(FRAME_FRAMES, 2)
    }

    /// The `StartMusic` commands recorded so far, as (voice, `MusicInfo`, segment).
    fn music_starts(&self) -> Vec<(u32, u32, u32)> {
        self.recorded
            .commands()
            .into_iter()
            .filter_map(|c| match c {
                SinkCommand::StartMusic {
                    voice,
                    music,
                    segment,
                    ..
                } => Some((voice, music, segment)),
                _ => None,
            })
            .collect()
    }

    fn command_count(&self) -> usize {
        self.recorded.commands().len()
    }

    /// The `Note` commands recorded so far.
    fn note_count(&self) -> usize {
        self.recorded
            .commands()
            .iter()
            .filter(|c| matches!(c, SinkCommand::Note(_)))
            .count()
    }

    /// The `StartMusic` commands since command `index`, as (voice, `MusicInfo`, segment).
    fn started_since(&self, index: usize) -> Vec<(u32, u32, u32)> {
        self.recorded.commands()[index..]
            .iter()
            .filter_map(|c| match c {
                SinkCommand::StartMusic {
                    voice,
                    music,
                    segment,
                    ..
                } => Some((*voice, *music, *segment)),
                _ => None,
            })
            .collect()
    }

    /// The voices of the town's music: its copies (`TOWN_MUSIC`) and the segments its script and
    /// triggers started (a `StartMusic` with no `MusicInfo`, music 0).
    fn town_voices(&self) -> Vec<u32> {
        self.music_starts()
            .iter()
            .filter(|&&(_, music, _)| music == TOWN_MUSIC || music == 0)
            .map(|&(v, _, _)| v)
            .collect()
    }

    /// Whether any of the town's voices is playing on the queue.
    fn town_playing(&mut self) -> bool {
        let town = self.town_voices();
        let audio = self.client.audio_mut().expect("the client has audio");
        town.iter().any(|&v| audio.music().is_playing(v))
    }
}

fn finite_peak(samples: &[f32]) -> f32 {
    assert!(samples.iter().all(|s| s.is_finite()), "the mix is finite");
    samples.iter().fold(0.0f32, |p, s| p.max(s.abs()))
}

#[test]
fn entering_the_town_plays_its_script_segment_and_the_mix_is_audible() {
    let Some(spawn) = town_spawn() else {
        return;
    };
    let Some(mut harness) = Harness::enter(spawn) else {
        return;
    };
    let mut frames = 0;
    while !harness.client.frame().in_world {
        harness.frame();
        frames += 1;
        assert!(frames < 600, "the player enters the world");
    }
    let mut first_note: Option<f64> = None;
    let mut peak = 0.0f32;
    let limit = (FIRST_NOTES_WITHIN * 64.0) as usize;
    for frame in 1..=limit {
        let samples = harness.frame();
        if first_note.is_none() && harness.note_count() > 0 {
            first_note = Some(frame as f64 * FRAME);
        }
        if first_note.is_some() {
            peak = peak.max(finite_peak(&samples));
        }
    }
    assert_eq!(harness.client.ambient_environment(), Some(TOWN_ENV));
    let starts = harness.music_starts();
    assert!(
        starts.iter().any(|&(_, music, _)| music == TOWN_MUSIC),
        "the town's music is queued"
    );
    assert!(
        starts
            .iter()
            .all(|&(_, music, segment)| music == 0
                || (music == TOWN_MUSIC && segment == TOWN_SEGMENT)),
        "only the town's music and the segments its script starts: {starts:08X?}"
    );
    assert!(
        starts.iter().any(|&(_, music, _)| music == 0),
        "the town's script starts its segment"
    );
    let first = first_note.unwrap_or_else(|| panic!("no note within {FIRST_NOTES_WITHIN} s"));
    assert!(
        peak > 0.001,
        "the mix is audible after the first note at {first:.2} s (peak {peak})"
    );
}

/// A long stay in the dominant environment: the player steps 5 m every second (more than
/// `m_fSoundUpdateDistance`, 4 m), so `ChangeCellEnd` weighs the cells again and again and each
/// weighing asks for the dominant-only music (`ContinMusicObject::PlaySoundA`). Retail's queue
/// (`DXMusicMessageThread::QueuePrimaryInternal`) keeps one primary playing and at most one
/// request waiting behind it (a newer request aborts the older one), so one copy sounds at a
/// time and no more than one waits.
#[test]
fn a_long_dominant_environment_keeps_one_music_voice() {
    let Some(spawn) = town_spawn() else {
        return;
    };
    let [_, _, z] = spawn.origin;
    let Some(mut harness) = Harness::enter(spawn) else {
        return;
    };
    while !harness.client.frame().in_world {
        harness.frame();
    }
    for _ in 0..128 {
        harness.frame();
    }
    assert_eq!(harness.client.ambient_environment(), Some(TOWN_ENV));
    let mut max_playing = 0;
    let mut max_queued = 0;
    let mut stamp = 200u16;
    let mut x = 96.0f32;
    // Twelve seconds of frames, a step each second.
    for frame in 0..12 * 64 {
        if frame % 64 == 0 {
            x = if x == 96.0 { 101.0 } else { 96.0 };
            harness.server.borrow_mut().send_player_position(
                0xCB4D_0025,
                [x, 96.0, z],
                [stamp, stamp + 1, stamp + 2],
            );
            stamp += 3;
        }
        harness.frame();
        let town: Vec<u32> = harness
            .music_starts()
            .iter()
            .filter(|&&(_, music, _)| music == TOWN_MUSIC)
            .map(|&(v, _, _)| v)
            .collect();
        let audio = harness.client.audio_mut().expect("the client has audio");
        let playing = town
            .iter()
            .filter(|&&v| audio.music().is_playing(v))
            .count();
        let queued = audio
            .live()
            .iter()
            .filter(|h| town.contains(h) && !audio.music().is_playing(**h))
            .count();
        max_playing = max_playing.max(playing);
        max_queued = max_queued.max(queued);
    }
    let requests = harness
        .music_starts()
        .iter()
        .filter(|&&(_, music, _)| music == TOWN_MUSIC)
        .count();
    assert!(
        requests >= 4,
        "the cells were weighed again and again: {requests} requests"
    );
    assert_eq!(
        harness.client.ambient_environment(),
        Some(TOWN_ENV),
        "the town stays dominant"
    );
    assert_eq!(max_playing, 1, "one copy of the music sounds at a time");
    assert!(max_queued <= 1, "at most one copy waits: {max_queued}");
}

/// The town's music keeps playing past its first pass with no gap: the Cue's pass hands over to
/// the segment its script queued, which hands over to the next copy (`OnMusicStopping`). Standing
/// still, from the first frame that plays, no frame has the town's music stopped.
#[test]
fn the_town_music_keeps_playing_past_its_first_pass() {
    let Some(spawn) = town_spawn() else {
        return;
    };
    let Some(mut harness) = Harness::enter(spawn) else {
        return;
    };
    while !harness.client.frame().in_world {
        harness.frame();
    }
    let mut started = false;
    let mut gap_frames = 0;
    // Forty seconds: the Cue (12 s), its script's segment and the next copy.
    for _ in 0..40 * 64 {
        harness.frame();
        if harness.town_playing() {
            started = true;
        } else if started {
            gap_frames += 1;
        }
    }
    assert!(started, "the town's music plays");
    assert_eq!(gap_frames, 0, "the music never stops between its passes");
}

#[test]
fn walking_into_an_interior_cell_switches_the_dominant_environment_and_music() {
    let Some(spawn) = town_spawn() else {
        return;
    };
    let Some(mut harness) = Harness::enter(spawn) else {
        return;
    };
    while !harness.client.frame().in_world {
        harness.frame();
    }
    for _ in 0..128 {
        harness.frame();
    }
    assert_eq!(harness.client.ambient_environment(), Some(TOWN_ENV));
    assert!(!harness.town_voices().is_empty());
    let before = harness.command_count();

    // The double teleports the player into the interior cell (`RecvEvt_Position`'s teleport).
    // The stamps must be newer than the spawn's (4 to 7), or the position is not applied.
    harness.server.borrow_mut().send_player_position(
        INTERIOR_CELL,
        INTERIOR_ORIGIN,
        [100, 101, 102],
    );
    let mut town_copy_stopped: Option<f64> = None;
    let mut interior_playing: Option<f64> = None;
    let mut peak = 0.0f32;
    // Sixty seconds: the town's pass (its Cue, then the segment its script queued) plays out first.
    for frame in 1..=60 * 64 {
        let samples = harness.frame();
        let seconds = frame as f64 * FRAME;
        let town_copies: Vec<u32> = harness
            .music_starts()
            .iter()
            .filter(|&&(_, music, _)| music == TOWN_MUSIC)
            .map(|&(v, _, _)| v)
            .collect();
        let copy_playing = {
            let audio = harness.client.audio_mut().expect("the client has audio");
            town_copies.iter().any(|&v| audio.music().is_playing(v))
        };
        if town_copy_stopped.is_none() && frame > 128 && !copy_playing {
            town_copy_stopped = Some(seconds);
        }
        // The interior's copies: the last request replaces an earlier one still waiting.
        let interior: Vec<u32> = harness
            .started_since(before)
            .iter()
            .filter(|&&(_, music, _)| music == INTERIOR_MUSIC)
            .map(|&(v, _, _)| v)
            .collect();
        let interior_copy_playing = {
            let audio = harness.client.audio_mut().expect("the client has audio");
            interior.iter().any(|&v| audio.music().is_playing(v))
        };
        if interior_playing.is_none() && interior_copy_playing {
            interior_playing = Some(seconds);
        }
        peak = peak.max(finite_peak(&samples));
    }
    assert_eq!(
        harness.client.ambient_environment(),
        Some(INTERIOR_ENV),
        "the interior environment dominates once the cells are weighed"
    );
    let started = harness.started_since(before);
    assert!(
        started
            .iter()
            .any(|&(_, music, segment)| music == INTERIOR_MUSIC && segment == INTERIOR_SEGMENT),
        "the interior's music is requested: {started:08X?}"
    );
    let stopped_at =
        town_copy_stopped.expect("the town's music stops once its pass has played out");
    let playing_at = interior_playing.expect("the interior's music plays");
    assert!(
        stopped_at <= playing_at,
        "the interior waits for the town's pass to end (the town stops at {stopped_at:.2} s, \
         the interior plays at {playing_at:.2} s)"
    );
    // The interior's music is its band only: its script's segment triggers a band segment and no
    // styled segment (no notes in the DAT's tree), so the mix's peak is the environment's sounds.
    assert!(peak > 0.0, "the interior mix is not silent (peak {peak})");
}

/// The town's ambient emitters come from the region record: one per continuous record of each
/// sound type the cells weighed. The Legions record has no intermittent sounds and no four-copy
/// spread (bit 0x10); its continuous sounds are dominant-only music, not 3D, so each emitter sits
/// on the listener.
#[test]
fn the_town_ambient_emitters_are_the_region_records_continuous_sounds() {
    let Some(spawn) = town_spawn() else {
        return;
    };
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let region = ac2_audio::AmbientData::load(&store, REGION_SOUND_DESC)
        .expect("the region's CSoundDesc decodes")
        .expect("the region's CSoundDesc");
    let Some(mut harness) = Harness::enter(spawn) else {
        return;
    };
    while !harness.client.frame().in_world {
        harness.frame();
    }
    for _ in 0..128 {
        harness.frame();
    }
    assert_eq!(harness.client.ambient_environment(), Some(TOWN_ENV));
    let types = harness.client.ambient_sound_types();
    assert!(
        !types.is_empty(),
        "the cells weighed the town's sound types"
    );
    let mut expected = 0;
    for sound_type in &types {
        let info = region
            .desc()
            .sound_info(*sound_type)
            .expect("a sound type the region names");
        assert!(info.intermit.is_empty(), "no intermittent sound: {info:?}");
        for contin in &info.contin {
            assert!(contin.test(2) && !contin.test(0x10), "{contin:?}");
        }
        expected += info.contin.len();
    }
    let emitters = harness.client.ambient_sound_emitters();
    assert_eq!(
        emitters.len(),
        expected,
        "one emitter per continuous record"
    );
    for (id, offset) in &emitters {
        assert_eq!(id >> 56, 0x13, "an ambient emitter id");
        assert_eq!(*offset, [0.0; 3], "a 2D sound sits on the listener");
    }
    let records: HashSet<u32> = region
        .desc()
        .sounds
        .iter()
        .flat_map(|info| info.contin.iter().map(|c| c.sound))
        .collect();
    let starts = harness.music_starts();
    assert!(!starts.is_empty(), "the town's music is queued");
    for (_, music, _) in starts.iter().filter(|(_, music, _)| *music != 0) {
        assert!(
            records.contains(music),
            "{music:08X} is a continuous record"
        );
    }
}

/// The town's run repeats: the same frames give the same music requests and notes, second by
/// second (the Legions random stream is fixed, and the double's clock is the frame count).
#[test]
fn the_town_ambient_run_repeats_second_by_second() {
    fn schedule() -> Option<Vec<(usize, usize)>> {
        let mut harness = Harness::enter(town_spawn()?)?;
        while !harness.client.frame().in_world {
            harness.frame();
        }
        let mut seconds = Vec::new();
        for _ in 0..20 {
            for _ in 0..64 {
                harness.frame();
            }
            seconds.push((harness.music_starts().len(), harness.note_count()));
        }
        Some(seconds)
    }
    let Some(first) = schedule() else {
        return;
    };
    let second = schedule().expect("the second run starts");
    assert_eq!(first, second);
    assert!(
        first.last().is_some_and(|&(_, notes)| notes > 0),
        "the town plays notes"
    );
}
