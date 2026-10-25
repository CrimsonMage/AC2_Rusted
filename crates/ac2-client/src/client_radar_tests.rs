use ac2_net::protocol::StringInfo;
use ac2_script::host::radar::{RadarLook, NO_BLIP};

use super::*;

const POINT: u32 = 0x4000_0001;
const FILL: u32 = 0x4000_0002;

fn look(shape: u32) -> Option<RadarLook> {
    Some(RadarLook {
        color: [1.0, 0.0, 0.0, 1.0],
        shape,
    })
}

fn note(kind: u32, id: u64, text: &str) -> MapMarker {
    MapMarker {
        kind,
        id,
        cell: 0x0299_0109,
        image: 0,
        message: StringInfo::literal(text),
    }
}

fn blips(radar: &RadarState) -> Vec<u64> {
    radar.blips.iter().map(|(id, _)| *id).collect()
}

#[test]
fn blips_keep_retail_order_and_removal_moves_the_last_one_in() {
    let mut radar = RadarState::default();
    radar.add(1, 0, look(POINT));
    radar.add(2, 0, look(FILL));
    radar.add(3, 0, look(POINT));
    // `SmartArray<RadarInfo>::RemoveUnOrdered`: the last entry takes the gap.
    radar.remove(1);
    assert_eq!(blips(&radar), [3, 2]);
    // `AddObject` again for a listed object adds nothing (`add_unique`).
    radar.add(2, 0, look(POINT));
    assert_eq!(blips(&radar), [3, 2]);
    assert_eq!(radar.blips[1].1.shape, FILL);
}

#[test]
fn no_blip_leaves_the_object_known_for_its_groove() {
    let mut radar = RadarState::default();
    radar.add(7, 5, None);
    radar.add(8, 0, look(NO_BLIP));
    assert!(radar.blips.is_empty());
    assert_eq!(radar.known.get(&7), Some(&5));
    assert_eq!(radar.known.get(&8), Some(&0));
    radar.remove(7);
    assert!(!radar.known.contains_key(&7));
}

#[test]
fn init_asks_everything_again() {
    let mut radar = RadarState::default();
    radar.add(1, 2, look(POINT));
    radar.ranges = Some((50.0, 75.0));
    radar.init();
    assert!(radar.blips.is_empty() && radar.known.is_empty());
    assert_eq!(radar.ranges, None);
}

#[test]
fn map_notes_replace_by_object_and_clear_by_type() {
    let mut radar = RadarState::default();
    radar.add_marker(note(1, 0x80, "first"));
    radar.add_marker(note(1, 0x81, "second"));
    radar.add_marker(note(2, 0x80, "other type"));
    // The same object's note in the same type is replaced.
    radar.add_marker(note(1, 0x80, "moved"));
    let ids: Vec<(u32, u64)> = radar.markers.iter().map(|m| (m.kind, m.id)).collect();
    assert_eq!(ids, [(1, 0x81), (2, 0x80), (1, 0x80)]);
    radar.remove_marker(2, 0x80);
    radar.clear_markers(1);
    assert!(radar.markers.is_empty());
    // Type 3 is never shown: not added, not cleared.
    radar.add_marker(note(3, 0x80, "hidden"));
    assert!(radar.markers.is_empty());
    radar.add_marker(note(4, 0, "no object"));
    radar.clear_markers(3);
    assert_eq!(radar.markers.len(), 1);
}
