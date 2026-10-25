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
