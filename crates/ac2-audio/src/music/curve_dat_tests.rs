//! The control curves of the DAT segments (`AC2_DAT_DIR`): every segment's controller events are
//! the events its curve items describe, at the times they describe, and the corpus's curves do
//! reach the synthesizer's controllers. Invariants only; nothing from the DATs is stored.

use std::sync::Arc;

use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use super::clock::MusicClock;
use super::curves::curve_events;
use super::library::MusicLibrary;
use super::performance::{note_events, MusicEvent};
use super::segment::TrackData;
use super::MusicObject;

const FIRST: u32 = 0x2400_0000;
const LAST: u32 = 0x24FF_FFFE;

fn is_controller(event: &MusicEvent) -> bool {
    !matches!(
        event,
        MusicEvent::NoteOn { .. } | MusicEvent::NoteOff { .. }
    )
}

#[test]
fn every_segments_controller_events_are_the_events_its_curves_describe() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let library = MusicLibrary::new(Arc::new(ResourceStore::new(Arc::clone(&dataset))));
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, FIRST, LAST)
        .collect();
    let mut segments_with_curves = 0;
    let mut controllers_seen = 0;
    let mut saw_volume = false;
    let mut saw_bend = false;
    for did in ids {
        let Some(MusicObject::Segment(segment)) = library.record(did) else {
            continue;
        };
        let clock = MusicClock::from_segment(&segment);
        let curves: Vec<_> = segment
            .tracks
            .iter()
            .filter_map(|t| match &t.data {
                TrackData::Sequence { curves, .. } => Some(curves.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        if curves.is_empty() {
            continue;
        }
        segments_with_curves += 1;
        let mut want: Vec<(f64, MusicEvent)> = curves
            .iter()
            .flat_map(curve_events)
            .map(|(tick, event)| (clock.seconds_at(tick.max(0)), event))
            .collect();
        let mut got: Vec<(f64, MusicEvent)> = note_events(&segment, &clock, None)
            .into_iter()
            .filter(|e| is_controller(&e.event))
            .map(|e| (e.seconds, e.event))
            .collect();
        let order = |a: &(f64, MusicEvent), b: &(f64, MusicEvent)| {
            a.0.total_cmp(&b.0)
                .then_with(|| format!("{:?}", a.1).cmp(&format!("{:?}", b.1)))
        };
        want.sort_by(order);
        got.sort_by(order);
        assert_eq!(got, want, "{did:08X}: its controllers are its curves'");
        controllers_seen += got.len();
        for (_, event) in &got {
            match event {
                MusicEvent::Control {
                    controller: 7,
                    value: 100,
                    ..
                } => saw_volume = true,
                MusicEvent::PitchBend { value: 8192, .. } => saw_bend = true,
                _ => {}
            }
        }
    }
    assert!(segments_with_curves > 0, "the corpus has curve segments");
    assert!(controllers_seen > 0);
    assert!(saw_volume, "the curves set CC7 (volume)");
    assert!(saw_bend, "the curves set the pitch bend (to centre)");
}
