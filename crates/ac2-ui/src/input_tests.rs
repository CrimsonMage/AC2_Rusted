//! The OS text the input manager takes (`CIme` commits and `WM_CHAR` characters): UTF-16 units
//! in order, and only text mode types.

use crate::input::{typed_text, InputEvent, InputManager};

fn units(text: &str) -> Vec<u16> {
    typed_text(text)
        .map(|event| match event {
            InputEvent::Text { ch } => ch,
            other => panic!("typed_text made {other:?}"),
        })
        .collect()
}

#[test]
fn ascii_text_is_one_unit_per_character() {
    assert_eq!(units("abc"), vec![0x61, 0x62, 0x63]);
}

#[test]
fn a_basic_plane_character_is_its_code_unit() {
    // A Japanese kana and an accented letter from an IME commit.
    assert_eq!(units("か é"), vec![0x304B, 0x20, 0xE9]);
}

#[test]
fn a_supplementary_character_arrives_as_its_surrogate_pair() {
    // U+1F600 is 0xD83D 0xDE00 in UTF-16.
    assert_eq!(units("\u{1F600}"), vec![0xD83D, 0xDE00]);
}

#[test]
fn an_empty_commit_types_nothing() {
    assert!(units("").is_empty());
}

#[test]
fn only_text_mode_takes_the_text() {
    let mut input = InputManager::new();
    let mut out = Vec::new();
    for event in typed_text("hi") {
        input.process(event, 0, &mut out);
    }
    assert!(out.is_empty());
    assert!(input.take_characters().is_empty());

    input.set_text_mode(true, 0, &mut Vec::new());
    for event in typed_text("\u{304B}") {
        input.process(event, 0, &mut out);
    }
    assert_eq!(input.take_characters(), vec![(0x304B, 0)]);
    assert!(out.is_empty());
}
