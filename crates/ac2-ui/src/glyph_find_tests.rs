//! `GlyphList`'s searches and the cursor moves on a hand-laid text (`glyph_find.rs`).

use crate::text::{Glyph, GlyphLine, Justification, TextState};

/// "ab cd ef" laid out as three lines of three glyphs each ("ab ", "cd ", "ef"), every glyph
/// 4 pixels wide and 10 tall.
fn text() -> TextState {
    let mut state = TextState::new(None);
    for ch in "ab cd ef".encode_utf16() {
        state.glyphs.push(Glyph {
            ch,
            width: 4,
            height: 10,
            font: None,
            color: [1.0; 4],
            outline: false,
        });
    }
    state.lines = vec![
        GlyphLine {
            index: 0,
            width: 12,
            height: 10,
        },
        GlyphLine {
            index: 3,
            width: 12,
            height: 10,
        },
        GlyphLine {
            index: 6,
            width: 8,
            height: 10,
        },
    ];
    state.paper = (40, 30);
    state
}

#[test]
fn a_position_finds_its_line_and_the_neighbours() {
    let t = text();
    assert_eq!(t.find_current_line(0), Some((0, 0)));
    assert_eq!(t.find_current_line(2), Some((0, 0)));
    assert_eq!(t.find_current_line(3), Some((1, 3)));
    assert_eq!(t.find_current_line(8), Some((2, 6)));
    assert_eq!(t.find_current_line(9), None);
    assert_eq!(t.find_next_line(4), Some((2, 6)));
    assert_eq!(t.find_next_line(7), Some((2, 6)));
    assert_eq!(t.find_prev_line(4), Some((0, 0)));
    assert_eq!(t.find_prev_line(1), Some((0, 0)));
}

#[test]
fn a_height_finds_its_line_and_clamps_past_the_ends() {
    let t = text();
    assert_eq!(t.find_line_from_y(-5), 0);
    assert_eq!(t.find_line_from_y(10), 0);
    assert_eq!(t.find_line_from_y(11), 1);
    assert_eq!(t.find_line_from_y(25), 2);
    // Below the last line the last line answers (checked against the machine code).
    assert_eq!(t.find_line_from_y(99), 2);
}

#[test]
fn words_are_found_by_white_space() {
    let t = text();
    assert_eq!(t.find_next_word(0), Some(3));
    assert_eq!(t.find_next_word(2), Some(3));
    assert_eq!(t.find_next_word(3), Some(6));
    // The last word's next word is the end of the text.
    assert_eq!(t.find_next_word(6), Some(8));
    assert_eq!(t.find_prev_word(8), Some(6));
    assert_eq!(t.find_prev_word(6), Some(3));
    assert_eq!(t.find_prev_word(4), Some(3));
    assert_eq!(t.find_prev_word(1), Some(0));
    assert_eq!(t.find_prev_word(9), None);
}

#[test]
fn pixels_and_positions_map_both_ways_within_a_line() {
    let t = text();
    assert_eq!(t.find_pixels_from_pos(5), Some(8));
    assert_eq!(t.find_pixels_from_pos(3), Some(0));
    // 5 pixels into line 1 is past half of the first glyph (2): after one glyph, since the
    // second glyph is passed only once more than half of it is behind.
    assert_eq!(t.find_pos_from_line_and_pixels(1, 5), Some(4));
    assert_eq!(t.find_pos_from_line_and_pixels(1, 2), Some(3));
    assert_eq!(t.find_pos_from_line_and_pixels(1, 7), Some(5));
    // Past the end of a non-last line stops before its last glyph; past the last line's end it
    // is the end of the text.
    assert_eq!(t.find_pos_from_line_and_pixels(1, 99), Some(5));
    assert_eq!(t.find_pos_from_line_and_pixels(2, 99), Some(8));
    assert_eq!(t.find_pos_from_line_and_pixels(3, 0), None);
}

#[test]
fn the_line_end_is_before_the_next_line_and_the_last_two_lines_end_the_text() {
    let t = text();
    assert_eq!(t.move_begin_end_line(true, 4), Some(3));
    // Line 0: the end is the glyph before line 1 starts.
    assert_eq!(t.move_begin_end_line(false, 1), Some(2));
    // Line 1's next line is the last line, so `FindNextLine`'s overwritten line number makes it
    // the end of the whole text.
    assert_eq!(t.move_begin_end_line(false, 4), Some(8));
    assert_eq!(t.move_begin_end_line(false, 7), Some(8));
}

#[test]
fn up_and_down_keep_the_pixel_column_and_stay_at_the_ends() {
    let t = text();
    // Position 4 is 4 pixels into line 1.
    assert_eq!(t.move_up_down(true, 4), Some(1));
    assert_eq!(t.move_up_down(false, 4), Some(7));
    // The first line going up and the last going down stay.
    assert_eq!(t.move_up_down(true, 1), Some(1));
    assert_eq!(t.move_up_down(false, 7), Some(7));
}

#[test]
fn a_centred_line_shifts_the_column_by_the_difference_of_the_lines() {
    let mut t = text();
    t.justification = Justification::Center;
    // Line 1 is 12 wide, line 2 is 8: line 2 starts 2 pixels further in (draw width 40), so
    // the 4 pixel column of position 4 is 2 pixels into line 2.
    assert_eq!(t.move_up_down(false, 4), Some(6));
}

#[test]
fn a_point_finds_the_position_under_it() {
    let mut t = text();
    // No margins or view: x 5 on the first line is after one glyph.
    assert_eq!(t.position_at(5, 4), 1);
    assert_eq!(t.position_at(5, 14), 4);
    assert_eq!(t.position_at(99, 99), 8);
    t.view = (4, 0);
    assert_eq!(t.position_at(5, 4), 2);
}
