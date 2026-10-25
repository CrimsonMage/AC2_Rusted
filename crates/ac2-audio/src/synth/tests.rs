//! The synthesizer on a synthetic DLS collection built here (no game data): one instrument, one
//! region, a short ramp wave with an optional loop, and an optional release connection.

use std::sync::Arc;

use super::{load_gm_file, Source, Synth};
use crate::music::dls::{read_collection, Collection};
use crate::music::performance::{EventSink, MusicEvent, TimedEvent};
use crate::music::riff::MusicFormatError;

/// The rate the synthetic wave is recorded at (the unity tests use the same output rate, so a
/// unity note steps one source frame per output frame).
const RATE: u32 = 22050;
const PATCH: u32 = 5;

fn chunk(id: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut v = id.to_vec();
    v.extend_from_slice(&(data.len() as u32).to_le_bytes());
    v.extend_from_slice(data);
    if data.len() % 2 == 1 {
        v.push(0);
    }
    v
}

/// A `LIST` of form `form` holding `children`.
fn list(form: &[u8; 4], children: &[Vec<u8>]) -> Vec<u8> {
    let mut body = form.to_vec();
    for c in children {
        body.extend_from_slice(c);
    }
    chunk(b"LIST", &body)
}

fn u32s(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// What the synthetic collection holds.
pub(crate) struct Spec {
    /// Mono 16-bit samples at [`RATE`].
    pub wave: Vec<i16>,
    /// A forward loop `(start, length)` in the region's `wsmp`.
    pub loop_: Option<(u32, u32)>,
    /// The EG1 release time in timecents (an `art1` connection on the instrument).
    pub release_tc: Option<i32>,
    /// More `art1` connections on the instrument: `(source, control, destination, scale)`, the
    /// scale in 16.16 fixed-point (the destination's unit times 65536).
    pub connections: Vec<(u16, u16, u16, i32)>,
}

/// The wave a test expects in the output: `sample[i]` as `f32`.
pub(crate) fn expected(spec: &Spec, i: usize) -> f32 {
    f32::from(spec.wave[i]) / 32768.0
}

/// A `RIFF 'DLS '` file for `spec`: instrument (bank 0, program [`PATCH`]), one region over all
/// keys and velocities with unity note 60, its wave in a `wvpl` pool with a `ptbl` cue.
pub(crate) fn dls_bytes(spec: &Spec) -> Vec<u8> {
    let mut wsmp = u32s(&[20]);
    wsmp.extend_from_slice(&60u16.to_le_bytes());
    wsmp.extend_from_slice(&0i16.to_le_bytes());
    wsmp.extend_from_slice(&0i32.to_le_bytes());
    wsmp.extend_from_slice(&0u32.to_le_bytes());
    match spec.loop_ {
        Some((start, length)) => {
            wsmp.extend_from_slice(&u32s(&[1]));
            wsmp.extend_from_slice(&u32s(&[16, 0, start, length]));
        }
        None => wsmp.extend_from_slice(&u32s(&[0])),
    }
    let mut rgnh = Vec::new();
    for v in [0u16, 127, 0, 127, 0, 0] {
        rgnh.extend_from_slice(&v.to_le_bytes());
    }
    let region = list(
        b"rgn ",
        &[
            chunk(b"rgnh", &rgnh),
            chunk(b"wsmp", &wsmp),
            chunk(b"wlnk", &[0u8; 12]),
        ],
    );
    let insh = u32s(&[1, 0, PATCH]);
    let mut ins_children = vec![chunk(b"insh", &insh), list(b"lrgn", &[region])];
    let mut connections: Vec<(u16, u16, u16, i32)> = spec.connections.clone();
    if let Some(tc) = spec.release_tc {
        // The scale is 16.16 fixed-point timecents.
        connections.push((0, 0, 0x0209, tc * 65_536));
    }
    if !connections.is_empty() {
        // cbSize, cConnections, then src, ctl, dst, transform (u16 each) and scale (i32).
        let mut art = u32s(&[8, connections.len() as u32]);
        for (source, control, destination, scale) in &connections {
            for v in [*source, *control, *destination, 0] {
                art.extend_from_slice(&v.to_le_bytes());
            }
            art.extend_from_slice(&scale.to_le_bytes());
        }
        ins_children.push(list(b"lart", &[chunk(b"art1", &art)]));
    }
    let instrument = list(b"ins ", &ins_children);
    let lins = list(b"lins", &[instrument]);

    let mut fmt = Vec::new();
    fmt.extend_from_slice(&1u16.to_le_bytes());
    fmt.extend_from_slice(&1u16.to_le_bytes());
    fmt.extend_from_slice(&RATE.to_le_bytes());
    fmt.extend_from_slice(&(RATE * 2).to_le_bytes());
    fmt.extend_from_slice(&2u16.to_le_bytes());
    fmt.extend_from_slice(&16u16.to_le_bytes());
    let pcm: Vec<u8> = spec.wave.iter().flat_map(|s| s.to_le_bytes()).collect();
    let wave = list(b"wave", &[chunk(b"fmt ", &fmt), chunk(b"data", &pcm)]);
    let pool = list(b"wvpl", &[wave]);
    let ptbl = chunk(b"ptbl", &u32s(&[8, 1, 0]));
    let dlid = chunk(b"dlid", &GUID);

    let mut body = b"DLS ".to_vec();
    for c in [dlid, lins, pool, ptbl] {
        body.extend_from_slice(&c);
    }
    chunk(b"RIFF", &body)
}

const GUID: [u8; 16] = [7; 16];

/// `n` frames of a ramp: `(i - 8) * 2000`, so every sample is distinct.
pub(crate) fn ramp(n: usize) -> Vec<i16> {
    (0..n).map(|i| (i as i16 - 8) * 2000).collect()
}

pub(crate) fn collection(spec: &Spec) -> Arc<Collection> {
    Arc::new(read_collection(&dls_bytes(spec)).expect("the synthetic collection parses"))
}

pub(crate) fn note_on(
    seconds: f64,
    channel: u32,
    patch: u32,
    note: u8,
    velocity: u8,
) -> TimedEvent {
    TimedEvent {
        seconds,
        event: MusicEvent::NoteOn {
            channel,
            instrument: Some(patch),
            note,
            velocity,
        },
        segment: 0,
    }
}

pub(crate) fn note_off(seconds: f64, channel: u32, note: u8) -> TimedEvent {
    TimedEvent {
        seconds,
        event: MusicEvent::NoteOff { channel, note },
        segment: 0,
    }
}

pub(crate) fn render(synth: &mut Synth, frames: usize, start: f64) -> Vec<f32> {
    let mut out = vec![0.0f32; frames];
    synth.render(&mut out, 1, start);
    out
}

pub(crate) fn ramp_synth(spec: &Spec) -> Synth {
    let mut synth = Synth::new(RATE);
    synth.bind(0, Source::Collection(collection(spec)));
    synth
}

pub(crate) fn plain_ramp() -> Spec {
    Spec {
        wave: ramp(16),
        loop_: None,
        release_tc: None,
        connections: Vec::new(),
    }
}

#[test]
fn a_synthetic_collection_parses_its_instrument_region_wave_and_loop() {
    let spec = Spec {
        wave: ramp(16),
        loop_: Some((4, 4)),
        release_tc: Some(-3986),
        connections: Vec::new(),
    };
    let c = read_collection(&dls_bytes(&spec)).expect("parses");
    assert_eq!(c.guid.0, GUID);
    assert_eq!(c.instruments.len(), 1);
    let instrument = &c.instruments[0];
    assert_eq!((instrument.bank, instrument.program), (0, PATCH));
    assert_eq!(instrument.patch(), PATCH);
    assert_eq!(
        instrument
            .articulation
            .as_ref()
            .map(|a| a.connections.len()),
        Some(1)
    );
    let region = &instrument.regions[0];
    assert_eq!(region.keys, (0, 127));
    assert_eq!(region.velocities, (0, 127));
    let sample = region.sample.as_ref().expect("a wsmp");
    assert_eq!(sample.unity_note, 60);
    assert_eq!(sample.fine_tune, 0);
    assert_eq!(sample.loops.len(), 1);
    assert_eq!((sample.loops[0].start, sample.loops[0].length), (4, 4));
    assert_eq!(region.link.table_index, 0);
    let wave = c.waves[0].as_ref().expect("the pool's wave");
    assert_eq!((wave.pcm.rate, wave.pcm.channels), (RATE, 1));
    assert_eq!(wave.pcm.samples.len(), 16);
    assert_eq!(wave.pcm.samples[0], expected(&spec, 0));
    assert_eq!(wave.pcm.samples[15], expected(&spec, 15));
    assert!(c.instrument(PATCH).is_some());
    assert!(c.instrument(PATCH + 1).is_none());
}

#[test]
fn a_file_that_is_not_dls_is_a_format_error() {
    assert_eq!(
        read_collection(b"RIFF\x04\x00\x00\x00WAVE").unwrap_err(),
        MusicFormatError("not a DLS collection")
    );
}

#[test]
fn a_note_at_the_unity_key_plays_the_sample_at_its_rate() {
    let spec = plain_ramp();
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render(&mut synth, 16, 0.0);
    for (i, &v) in out.iter().enumerate() {
        assert!((v - expected(&spec, i)).abs() < 1e-6, "frame {i}: {v}");
    }
    assert_eq!(synth.voices(), 0, "the sample's end ends the voice");
}

#[test]
fn an_octave_up_plays_the_sample_twice_as_fast() {
    let spec = plain_ramp();
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 72, 127));
    let out = render(&mut synth, 16, 0.0);
    for (i, &v) in out.iter().enumerate().take(8) {
        assert!((v - expected(&spec, 2 * i)).abs() < 1e-6, "frame {i}: {v}");
    }
    assert!(out[8..].iter().all(|&v| v == 0.0), "half the length");
}

#[test]
fn the_loop_holds_while_the_note_is_on() {
    let spec = Spec {
        wave: ramp(16),
        loop_: Some((4, 4)),
        release_tc: None,
        connections: Vec::new(),
    };
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render(&mut synth, 40, 0.0);
    for (i, &v) in out.iter().enumerate() {
        let index = if i < 8 { i } else { 4 + (i - 4) % 4 };
        assert!((v - expected(&spec, index)).abs() < 1e-6, "frame {i}: {v}");
    }
    assert_eq!(synth.voices(), 1, "still sounding while held");
}

#[test]
fn release_decays_to_silence() {
    // A long constant wave so the note sounds through the release.
    let spec = Spec {
        wave: vec![16384; RATE as usize],
        loop_: None,
        release_tc: Some(-3986),
        connections: Vec::new(),
    };
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let held = render(&mut synth, 1102, 0.0);
    assert!(
        held.iter().all(|&v| (v - 0.5).abs() < 1e-6),
        "held at full level"
    );
    synth.event(&note_off(0.05, 0, 60));
    let released = render(&mut synth, 4410, 0.05);
    assert!(released[0] < 0.5 && released[0] > 0.4, "release starts");
    let mut last = f32::MAX;
    for &v in released.iter().take(2100) {
        assert!(v <= last + 1e-6, "the release does not rise");
        last = v;
    }
    assert!(
        released[2300..].iter().all(|&v| v == 0.0),
        "silent after 0.1 s"
    );
    assert_eq!(synth.voices(), 0);
}

#[test]
fn velocity_scales_the_level() {
    let spec = plain_ramp();
    let mut loud = ramp_synth(&spec);
    loud.event(&note_on(0.0, 0, PATCH, 60, 127));
    let loud = render(&mut loud, 16, 0.0);
    let mut soft = ramp_synth(&spec);
    soft.event(&note_on(0.0, 0, PATCH, 60, 64));
    let soft = render(&mut soft, 16, 0.0);
    for i in 0..16 {
        let want = loud[i] * 64.0 / 127.0;
        assert!(
            (soft[i] - want).abs() < 1e-6,
            "frame {i}: {} vs {want}",
            soft[i]
        );
    }
}

#[test]
fn a_note_is_applied_at_its_time_on_the_timeline() {
    let spec = plain_ramp();
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.01, 0, PATCH, 60, 127));
    let out = render(&mut synth, 240, 0.0);
    // 0.01 s is frame 220.5 at 22050 Hz, so the note starts at frame 221.
    assert!(out[..221].iter().all(|&v| v == 0.0));
    assert!((out[221] - expected(&spec, 0)).abs() < 1e-6);
}

#[test]
fn a_band_program_with_no_collection_and_no_gm_file_stays_silent() {
    let mut synth = Synth::new(RATE);
    synth.bind(0, Source::GeneralMidi);
    synth.bind(1, Source::Missing);
    synth.event(&note_on(0.0, 0, 0x5_0102, 60, 100));
    synth.event(&note_on(0.0, 1, PATCH, 60, 100));
    let out = render(&mut synth, 4410, 0.0);
    assert!(out.iter().all(|&v| v == 0.0));
    assert_eq!(synth.voices(), 0);
}

#[test]
fn a_gm_file_the_user_supplies_plays_the_gm_channels() {
    let spec = plain_ramp();
    let path = std::env::temp_dir().join(format!("ac2-synth-gm-{}.dls", std::process::id()));
    std::fs::write(&path, dls_bytes(&spec)).expect("writes the test file");
    let gm = load_gm_file(&path).expect("loads the GM file");
    let _ = std::fs::remove_file(&path);
    let mut synth = Synth::new(RATE);
    synth.set_gm(Some(gm));
    synth.bind(0, Source::GeneralMidi);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let out = render(&mut synth, 16, 0.0);
    assert!((out[0] - expected(&spec, 0)).abs() < 1e-6);
}

#[test]
fn a_missing_gm_file_is_an_io_error() {
    let path = std::env::temp_dir().join("ac2-synth-no-such-gm.dls");
    assert!(load_gm_file(&path).is_err());
}

#[test]
fn a_channel_without_a_source_is_silent_and_the_output_is_finite() {
    let mut synth = Synth::new(RATE);
    synth.event(&note_on(0.0, 3, PATCH, 60, 127));
    let out = render(&mut synth, 100, 0.0);
    assert!(out.iter().all(|v| v.is_finite() && *v == 0.0));
}

#[test]
fn stereo_output_writes_the_same_level_to_both_channels() {
    let spec = plain_ramp();
    let mut synth = ramp_synth(&spec);
    synth.event(&note_on(0.0, 0, PATCH, 60, 127));
    let mut out = vec![0.0f32; 32];
    synth.render(&mut out, 2, 0.0);
    for i in 0..16 {
        assert_eq!(out[2 * i], out[2 * i + 1]);
        assert!((out[2 * i] - expected(&spec, i)).abs() < 1e-6);
    }
}

/// The sink path: events go in through [`EventSink`] like the music queue sends them.
#[test]
fn the_event_sink_queues_events_for_the_renderer() {
    let spec = plain_ramp();
    let mut synth = ramp_synth(&spec);
    let events = vec![note_on(0.0, 0, PATCH, 60, 127)];
    for e in &events {
        EventSink::event(&mut synth, e);
    }
    let out = render(&mut synth, 16, 0.0);
    assert!((out[0] - expected(&spec, 0)).abs() < 1e-6);
}
