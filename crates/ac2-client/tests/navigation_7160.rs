//! The map notes' rules (`gmNavigationUI::RecvNotice_AddMarker` and `AddMarker`, admin 7148):
//! type 3 is never shown; a note near one of its type is ignored; a type keeps ten notes; the
//! same object's note in its type is replaced. The land cells come from `lcoord_to_gid`, so no
//! game data is needed.

use ac2_client::{add_map_note, MapMarker};
use ac2_physics::landdefs::{gid_to_lcoord, lcoord_to_gid};

fn note(kind: u32, id: u64, x: i32, y: i32) -> MapMarker {
    MapMarker {
        kind,
        id,
        cell: lcoord_to_gid(x, y),
        image: 0,
        message: Default::default(),
    }
}

#[test]
fn type_3_is_never_a_note() {
    let mut notes = Vec::new();
    add_map_note(&mut notes, note(3, 1, 10, 10));
    assert!(notes.is_empty());
}

#[test]
fn a_note_within_sqrt_six_cells_of_one_of_its_type_is_ignored() {
    let mut notes = Vec::new();
    add_map_note(&mut notes, note(4, 1, 10, 10));
    // (11, 11): 1 + 1 = 2 < 6. (12, 10): 4 < 6. Both ignored, whatever their object.
    add_map_note(&mut notes, note(4, 2, 11, 11));
    add_map_note(&mut notes, note(4, 3, 12, 10));
    assert_eq!(notes.len(), 1);
    // (13, 10): 9, not under 6, is kept.
    add_map_note(&mut notes, note(4, 4, 13, 10));
    assert_eq!(notes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![1, 4]);
}

#[test]
fn the_nearness_test_is_per_type() {
    let mut notes = Vec::new();
    add_map_note(&mut notes, note(4, 1, 10, 10));
    add_map_note(&mut notes, note(5, 2, 11, 11));
    assert_eq!(notes.len(), 2);
}

#[test]
fn a_type_keeps_ten_notes_and_drops_its_oldest() {
    let mut notes = Vec::new();
    for i in 0..12_i32 {
        add_map_note(&mut notes, note(6, (i + 1) as u64, i * 10, 0));
    }
    let ids: Vec<u64> = notes.iter().map(|n| n.id).collect();
    assert_eq!(ids, (3..=12).collect::<Vec<u64>>());
}

#[test]
fn the_same_object_replaces_its_note_in_its_type() {
    let mut notes = Vec::new();
    add_map_note(&mut notes, note(7, 42, 1, 1));
    add_map_note(&mut notes, note(7, 42, 30, 30));
    assert_eq!(notes.len(), 1);
    assert_eq!(gid_to_lcoord(notes[0].cell), Some((30, 30)));
}

#[test]
fn a_note_off_the_land_is_kept_without_the_nearness_test() {
    let mut notes = Vec::new();
    add_map_note(
        &mut notes,
        MapMarker {
            kind: 8,
            id: 1,
            cell: 0,
            ..Default::default()
        },
    );
    add_map_note(
        &mut notes,
        MapMarker {
            kind: 8,
            id: 2,
            cell: 0,
            ..Default::default()
        },
    );
    assert_eq!(notes.len(), 2);
}
