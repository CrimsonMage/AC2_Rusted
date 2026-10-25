//! The filter and vibrato on the DLS collections in the DATs (`AC2_DAT_DIR`): every collection
//! whose blocks set a filter or a vibrato renders finite output with a non-silent note; a cutoff the
//! corpus sets below Nyquist lowers the note's high-frequency energy against the same voice without
//! it; the open maximum is bypassed bit for bit; the one ungated vibrato block changes its note.
//! Invariants and measured changes only; nothing is stored.

use std::collections::BTreeMap;
use std::sync::Arc;

use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use super::filter::BYPASS_FRACTION;
use super::lfo::hz_from_absolute_cents;
use super::modulation::{resolve, Control, NoteModulation, Source};
use super::voice::{Mix, Voice};
use crate::music::dls::{Collection, Instrument, Region};
use crate::music::library::MusicLibrary;
use crate::music::MusicObject;

const FIRST: u32 = 0x2400_0000;
const LAST: u32 = 0x24FF_FFFE;
/// The output rate the regions are rendered at.
const RATE: u32 = 44_100;
/// Half a second per region.
const FRAMES: usize = 22_050;

/// The note a region is rendered at, as the `dat_tests` harness picks it: its middle key, and the
/// middle of its velocity range (100 for a region with none).
fn region_note(region: &Region) -> (u8, u8) {
    let note = ((region.keys.0 + region.keys.1) / 2) as u8;
    let velocity = if region.velocities == (0, 0) {
        100
    } else {
        ((region.velocities.0 + region.velocities.1) / 2).clamp(1, 127) as u8
    };
    (note, velocity)
}

/// The modulation a region's note resolves to (its region's blocks, then its instrument's).
fn modulation_of(
    instrument: &Instrument,
    region: &Region,
    note: u8,
    velocity: u8,
) -> NoteModulation {
    resolve(
        region
            .articulation
            .iter()
            .chain(instrument.articulation.iter()),
        note,
        velocity,
    )
}

/// The mono mix (both channels averaged, so a region panned to one side still shows) of
/// [`FRAMES`] of a region's voice under `modulation`, or `None` when its wave is not in the
/// collection.
fn render(
    collection: &Collection,
    region: &Region,
    note: u8,
    velocity: u8,
    modulation: &NoteModulation,
) -> Option<Vec<f32>> {
    let wave = collection
        .waves
        .get(region.link.table_index as usize)?
        .as_deref()?;
    let mut voice = Voice::start(0, note, velocity, region, wave, modulation, RATE)?;
    let dt = 1.0 / f64::from(RATE);
    Some(
        (0..FRAMES)
            .map(|_| {
                let (l, r) = voice.next(dt, Mix::UNITY);
                ((l + r) * 0.5) as f32
            })
            .collect(),
    )
}

/// The high-frequency energy of a render: the sum of squared first differences.
fn high_frequency_energy(x: &[f32]) -> f64 {
    x.windows(2)
        .map(|p| (f64::from(p[1]) - f64::from(p[0])).powi(2))
        .sum()
}

/// Visits every region of every DLS collection in the DATs. `None` without `AC2_DAT_DIR`.
fn each_region(mut visit: impl FnMut(u32, &Collection, &Instrument, &Region)) -> Option<()> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let library = MusicLibrary::new(Arc::new(ResourceStore::new(Arc::clone(&dataset))));
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, FIRST, LAST)
        .collect();
    for did in ids {
        let Some(MusicObject::Collection(c)) = library.record(did) else {
            continue;
        };
        for instrument in &c.instruments {
            for region in &instrument.regions {
                visit(did, &c, instrument, region);
            }
        }
    }
    Some(())
}

#[test]
fn every_collection_with_a_filter_or_vibrato_block_renders_finite_output_and_a_non_silent_note() {
    // Per collection: whether any of its blocked regions renders a non-silent note.
    let mut collections: BTreeMap<u32, bool> = BTreeMap::new();
    let ran = each_region(|did, c, instrument, region| {
        let (note, velocity) = region_note(region);
        let m = modulation_of(instrument, region, note, velocity);
        let vibrato = m.vibrato.is_some() || m.routes.iter().any(|r| r.source == Source::Vibrato);
        if m.filter.is_none() && !vibrato {
            return;
        }
        let Some(out) = render(c, region, note, velocity, &m) else {
            return;
        };
        assert!(out.iter().all(|v| v.is_finite()), "{did:08X} finite");
        let peak = out.iter().fold(0.0f32, |p, v| p.max(v.abs()));
        let sounds = collections.entry(did).or_insert(false);
        *sounds |= peak > 0.0;
    });
    if ran.is_none() {
        return;
    }
    assert!(
        !collections.is_empty(),
        "the corpus has filtered or vibrato collections"
    );
    for (did, sounds) in &collections {
        assert!(sounds, "{did:08X} renders a non-silent note");
    }
}

#[test]
fn a_corpus_cutoff_below_nyquist_lowers_the_high_frequency_energy_and_the_open_maximum_is_bypassed()
{
    let (mut applied, mut lower, mut bypassed) = (0, 0, 0);
    let mut ratios = Vec::new();
    let ran = each_region(|did, c, instrument, region| {
        let (note, velocity) = region_note(region);
        let m = modulation_of(instrument, region, note, velocity);
        let Some(filter) = m.filter else {
            return;
        };
        let unfiltered = NoteModulation {
            filter: None,
            ..m.clone()
        };
        let (Some(with), Some(without)) = (
            render(c, region, note, velocity, &m),
            render(c, region, note, velocity, &unfiltered),
        ) else {
            return;
        };
        if hz_from_absolute_cents(filter.cutoff_cents) >= BYPASS_FRACTION * f64::from(RATE) {
            // The open maximum (and a route that only moves it): the note is unchanged.
            assert_eq!(with, without, "{did:08X} the open cutoff is bypassed");
            bypassed += 1;
            return;
        }
        let hf_without = high_frequency_energy(&without);
        if hf_without <= 0.0 {
            return;
        }
        let ratio = high_frequency_energy(&with) / hf_without;
        assert!(ratio.is_finite(), "{did:08X} a finite high-frequency ratio");
        ratios.push(ratio);
        applied += 1;
        lower += usize::from(ratio < 1.0);
    });
    if ran.is_none() {
        return;
    }
    assert!(bypassed > 0, "the corpus's open cutoffs are bypassed");
    assert!(applied > 0, "the corpus sets a cutoff below Nyquist");
    assert!(
        lower > 0,
        "a corpus cutoff lowers the high-frequency energy"
    );
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = ratios[ratios.len() / 2];
    assert!(
        median < 1.0,
        "the cutoffs lower the energy in the median region: {median}"
    );
}

#[test]
fn an_ungated_vibrato_block_changes_the_corpus_note() {
    let mut changed = 0;
    let ran = each_region(|_, c, instrument, region| {
        let (note, velocity) = region_note(region);
        let m = modulation_of(instrument, region, note, velocity);
        let ungated = m
            .routes
            .iter()
            .any(|r| r.source == Source::Vibrato && r.control == Control::None);
        if !ungated {
            return;
        }
        let without = NoteModulation {
            routes: m
                .routes
                .iter()
                .copied()
                .filter(|r| r.source != Source::Vibrato)
                .collect(),
            vibrato: None,
            ..m.clone()
        };
        let (Some(with), Some(stripped)) = (
            render(c, region, note, velocity, &m),
            render(c, region, note, velocity, &without),
        ) else {
            return;
        };
        let diff = with
            .iter()
            .zip(&stripped)
            .fold(0.0f32, |d, (a, b)| d.max((a - b).abs()));
        changed += usize::from(diff > 1e-4);
    });
    if ran.is_none() {
        return;
    }
    assert!(changed > 0, "an ungated vibrato block changes its note");
}
