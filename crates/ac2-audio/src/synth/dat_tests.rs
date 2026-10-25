//! The DLS collections in the DATs (`AC2_DAT_DIR`): every one parses, and a note from its first
//! instrument renders finite and non-silent. Invariants only; no collection is stored.

use std::sync::Arc;

use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use super::{Source, Synth};
use crate::music::library::MusicLibrary;
use crate::music::performance::{EventSink, MusicEvent, TimedEvent};
use crate::music::MusicObject;

const FIRST: u32 = 0x2400_0000;
const LAST: u32 = 0x24FF_FFFE;
/// The collections the DATs hold (the music records test counts them too).
const COLLECTIONS: usize = 61;

#[test]
fn every_dls_collection_parses_and_renders_a_note_from_its_first_instrument() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let library = MusicLibrary::new(Arc::new(ResourceStore::new(Arc::clone(&dataset))));
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, FIRST, LAST)
        .collect();
    let mut parsed = 0;
    for did in ids {
        let Some(MusicObject::Collection(c)) = library.record(did) else {
            continue;
        };
        parsed += 1;
        let instrument = c
            .instruments
            .first()
            .unwrap_or_else(|| panic!("{did:08X} has an instrument"));
        let region = instrument
            .regions
            .first()
            .unwrap_or_else(|| panic!("{did:08X} has a region"));
        let note = ((region.keys.0 + region.keys.1) / 2) as u8;
        let velocity = if region.velocities == (0, 0) {
            100
        } else {
            ((region.velocities.0 + region.velocities.1) / 2).clamp(1, 127) as u8
        };
        let mut synth = Synth::new(44_100);
        synth.bind(0, Source::Collection(Arc::clone(&c)));
        synth.event(&TimedEvent {
            seconds: 0.0,
            event: MusicEvent::NoteOn {
                channel: 0,
                instrument: Some(instrument.patch()),
                note,
                velocity,
            },
            segment: 0,
        });
        let mut out = vec![0.0f32; 2 * 441_000];
        synth.render(&mut out, 2, 0.0);
        assert!(out.iter().all(|v| v.is_finite()), "{did:08X} finite");
        let peak = out.iter().fold(0.0f32, |p, v| p.max(v.abs()));
        let wave = c
            .waves
            .get(region.link.table_index as usize)
            .and_then(|w| w.as_ref());
        assert!(
            peak > 0.0,
            "{did:08X} renders a note (patch {:#X}, key {note}, vel {velocity}, keys {:?}, vels {:?}, wave {} present {} len {:?} sample {:?} covers {})",
            instrument.patch(),
            region.keys,
            region.velocities,
            region.link.table_index,
            wave.is_some(),
            wave.map(|w| (w.pcm.rate, w.pcm.samples.len())),
            region.sample,
            region.covers(note, velocity)
        );
    }
    assert_eq!(parsed, COLLECTIONS);
}

/// The keys and velocities each region's connection blocks are resolved at.
const NOTES: [(u8, u8); 3] = [(36, 1), (60, 100), (96, 127)];

#[test]
fn every_connection_block_resolves_to_finite_parameters_and_the_corpus_routes_are_read() {
    use super::modulation::resolve;
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let library = MusicLibrary::new(Arc::new(ResourceStore::new(Arc::clone(&dataset))));
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, FIRST, LAST)
        .collect();
    let (mut with_pan, mut with_lfo, mut with_lfo_route) = (0, 0, 0);
    for did in ids {
        let Some(MusicObject::Collection(c)) = library.record(did) else {
            continue;
        };
        let (mut pan, mut lfo, mut lfo_route) = (false, false, false);
        for instrument in &c.instruments {
            for region in &instrument.regions {
                for (key, velocity) in NOTES {
                    let m = resolve(
                        region
                            .articulation
                            .iter()
                            .chain(instrument.articulation.iter()),
                        key,
                        velocity,
                    );
                    for env in [m.eg1, m.eg2] {
                        for t in [env.attack, env.decay, env.release] {
                            assert!(t.is_finite() && t >= 0.0, "{did:08X} a time {t}");
                        }
                        assert!((0.0..=1.0).contains(&env.sustain), "{did:08X} sustain");
                    }
                    assert!(m.pitch_cents.is_finite() && m.attenuation_db.is_finite());
                    assert!((-1.0..=1.0).contains(&m.pan), "{did:08X} pan {}", m.pan);
                    if let Some(l) = m.lfo {
                        assert!(l.frequency.is_finite() && l.frequency > 0.0);
                        assert!(l.delay.is_finite() && l.delay >= 0.0);
                        lfo = true;
                    }
                    for route in &m.routes {
                        assert!(route.scale.is_finite(), "{did:08X} route scale");
                        lfo_route |= route.source == super::modulation::Source::Lfo;
                    }
                    pan |= m.pan != 0.0;
                }
            }
        }
        with_pan += usize::from(pan);
        with_lfo += usize::from(lfo);
        with_lfo_route += usize::from(lfo_route);
    }
    // The corpus's connection blocks set pan, the LFO, and LFO routes (art1 and art2 together).
    assert!(with_pan > 0, "a collection sets pan");
    assert!(with_lfo > 0, "a collection sets the LFO");
    assert!(with_lfo_route > 0, "a collection routes the LFO");
}
