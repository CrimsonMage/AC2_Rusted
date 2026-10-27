//! Text selection with the clipboard: the keys, the pointer drag, paste limits and password
//! refusal, against a clipboard held in memory (`MemoryClipboard`).

use std::collections::HashMap;
use std::sync::Arc;

use crate::chat::Modifiers;
use crate::text_clip::paste_units;
use crate::text_clipboard::{MemoryClipboard, TextClipboard};
use crate::text_tests::{editable_entry, entry_with};
use crate::{UiTree, UI_POINTER_X, UI_POINTER_Y};

const LEFT: u32 = 0x4000_0004;
const SHIFT_LEFT: u32 = 0x4000_0013;
const HOME: u32 = 0x4000_001B;
const PLAIN: Modifiers = Modifiers {
    ctrl: false,
    shift: false,
    alt: false,
};
const SHIFT: Modifiers = Modifiers {
    ctrl: false,
    shift: true,
    alt: false,
};
const CTRL: Modifiers = Modifiers {
    ctrl: true,
    shift: false,
    alt: false,
};

fn type_text(tree: &mut UiTree, text: &str) {
    for c in text.chars() {
        tree.character(c as u16, 0, PLAIN);
    }
}

fn text(tree: &UiTree, entry: usize) -> String {
    tree.get(entry).unwrap().text().unwrap()
}

fn clipboard(text: &str) -> (Arc<MemoryClipboard>, Arc<dyn TextClipboard>) {
    let clip = Arc::new(MemoryClipboard::with_text(text));
    (Arc::clone(&clip), clip)
}

/// The clipboard's text, read back through the trait the tree holds.
fn held(clip: &MemoryClipboard) -> Option<String> {
    clip.text()
}

#[test]
fn the_units_a_paste_types_keep_only_what_the_field_takes() {
    // CRLF is one line break; a multi-line field keeps it, a one-line one makes it a space.
    assert_eq!(
        paste_units("a\r\nb\tc", false, None),
        "a\nbc".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        paste_units("a\r\nb\tc", true, None),
        "a bc".encode_utf16().collect::<Vec<_>>()
    );
    // A filter takes digits only: the letters, the space and the line break go.
    let digits: fn(u16) -> bool = |c| (u16::from(b'0')..=u16::from(b'9')).contains(&c);
    assert_eq!(
        paste_units("1x2 3\n", true, Some(digits)),
        "123".encode_utf16().collect::<Vec<_>>()
    );
}

#[test]
fn ctrl_a_selects_all_and_ctrl_c_copies_without_changing_the_text() {
    let (mut tree, entry) = editable_entry();
    let (clip, dyn_clip) = clipboard("old");
    tree.set_text_clipboard(dyn_clip);
    type_text(&mut tree, "abc");
    tree.character(u16::from(b'a'), 0, CTRL);
    assert_eq!(tree.selection(entry), Some((0, 3)));
    tree.character(u16::from(b'c'), 0, CTRL);
    assert_eq!(held(&clip).as_deref(), Some("abc"));
    assert_eq!(text(&tree, entry), "abc");
    // The selection stays after a copy.
    assert_eq!(tree.selection(entry), Some((0, 3)));
}

#[test]
fn a_copy_with_nothing_selected_leaves_the_clipboard_alone() {
    let (mut tree, entry) = editable_entry();
    let (clip, dyn_clip) = clipboard("kept");
    tree.set_text_clipboard(dyn_clip);
    type_text(&mut tree, "abc");
    assert!(!tree.copy_selection(entry));
    assert_eq!(held(&clip).as_deref(), Some("kept"));
}

#[test]
fn ctrl_x_cuts_the_selection_into_the_clipboard() {
    let (mut tree, entry) = editable_entry();
    let (clip, dyn_clip) = clipboard("");
    tree.set_text_clipboard(dyn_clip);
    type_text(&mut tree, "abcd");
    // Shift and the left arrow twice select "cd".
    tree.text_action(SHIFT_LEFT, 1.0, SHIFT);
    tree.text_action(SHIFT_LEFT, 1.0, SHIFT);
    tree.character(u16::from(b'x'), 0, CTRL);
    assert_eq!(held(&clip).as_deref(), Some("cd"));
    assert_eq!(text(&tree, entry), "ab");
    assert_eq!(tree.selection(entry), None);
}

#[test]
fn ctrl_v_replaces_the_selection_and_stops_at_the_max_length() {
    let (mut tree, entry) = editable_entry();
    let (_, dyn_clip) = clipboard("WXYZ");
    tree.set_text_clipboard(dyn_clip);
    tree.el_mut(entry).text.as_mut().unwrap().max_characters = 5;
    type_text(&mut tree, "ab");
    // Four characters fit after "ab" in a field of five: "WXY" is what lands.
    tree.character(u16::from(b'v'), 0, CTRL);
    assert_eq!(text(&tree, entry), "abWXY");
    // Over a selection the selection goes first, so the room is the field's again.
    tree.select_all(entry);
    tree.character(0x16, 0, PLAIN);
    assert_eq!(text(&tree, entry), "WXYZ");
}

#[test]
fn a_paste_is_filtered_by_the_field_and_a_copied_line_break_is_trimmed() {
    let (mut tree, entry) = entry_with(&["editable", "selectable", "activatable", "one_line"]);
    let digits: fn(u16) -> bool = |c| (u16::from(b'0')..=u16::from(b'9')).contains(&c);
    tree.set_text_filter(entry, Some(digits));
    let (_, dyn_clip) = clipboard("a1b2\r\n");
    tree.set_text_clipboard(dyn_clip);
    tree.character(0x16, 0, CTRL);
    assert_eq!(text(&tree, entry), "12");
}

#[test]
fn a_password_field_never_copies_or_cuts_but_takes_a_paste() {
    let (mut tree, entry) = entry_with(&[
        "editable",
        "selectable",
        "activatable",
        "one_line",
        "password",
    ]);
    let (clip, dyn_clip) = clipboard("kept");
    tree.set_text_clipboard(dyn_clip);
    type_text(&mut tree, "secret");
    tree.character(u16::from(b'a'), 0, CTRL);
    assert_eq!(tree.selection(entry), Some((0, 6)));
    tree.character(u16::from(b'c'), 0, CTRL);
    tree.character(u16::from(b'x'), 0, CTRL);
    assert_eq!(held(&clip).as_deref(), Some("kept"));
    assert_eq!(text(&tree, entry), "secret");
    assert!(!tree.copy_selection(entry));
    // Paste still goes in, over the selection.
    assert!(clip.set_text("pw"));
    tree.character(0x16, 0, CTRL);
    assert_eq!(text(&tree, entry), "pw");
}

#[test]
fn a_read_only_selectable_text_copies_but_is_not_edited() {
    let (mut tree, entry) = entry_with(&["selectable", "activatable", "one_line"]);
    let (clip, dyn_clip) = clipboard("");
    tree.set_text_clipboard(dyn_clip);
    let units: Vec<u16> = "read only".encode_utf16().collect();
    tree.add_text_with_font(entry, &units, 0, 0);
    // Keys reach a text element only while it is the active one.
    assert!(tree.activate(entry));
    tree.character(u16::from(b'a'), 0, CTRL);
    tree.character(u16::from(b'c'), 0, CTRL);
    assert_eq!(held(&clip).as_deref(), Some("read only"));
    // Cut, paste and typing do nothing on a field the player cannot edit.
    tree.character(u16::from(b'x'), 0, CTRL);
    tree.character(0x16, 0, CTRL);
    tree.character(u16::from(b'z'), 0, PLAIN);
    assert_eq!(text(&tree, entry), "read only");
    assert_eq!(held(&clip).as_deref(), Some("read only"));
}

#[test]
fn a_press_and_drag_selects_from_the_press_until_the_release() {
    let (mut tree, entry) = editable_entry();
    type_text(&mut tree, "abcdef");
    // Glyphs are 3 pixels wide: x 3 is in front of "b", x 10 after "d" and before "e".
    let move_to = |tree: &mut UiTree, x: i32| {
        tree.start_input(UI_POINTER_X, x as f32);
        tree.start_input(UI_POINTER_Y, 5.0);
    };
    move_to(&mut tree, 3);
    tree.start_input(LEFT, 1.0);
    assert_eq!(tree.selection(entry), None);
    move_to(&mut tree, 10);
    assert_eq!(tree.selection(entry), Some((1, 4)));
    assert_eq!(
        tree.selected_text(entry),
        "bcd".encode_utf16().collect::<Vec<_>>()
    );
    // Released: the pointer no longer moves the selection.
    tree.stop_input(LEFT);
    move_to(&mut tree, 20);
    assert_eq!(tree.selection(entry), Some((1, 4)));
}

#[test]
fn shift_home_and_end_select_to_the_line_ends() {
    let (mut tree, entry) = editable_entry();
    type_text(&mut tree, "abcd");
    tree.text_action(HOME, 1.0, SHIFT);
    assert_eq!(tree.selection(entry), Some((0, 4)));
    // The anchor stays where the selection began (4), so Shift+End only moves the other end.
    tree.text_action(0x4000_001C, 1.0, SHIFT);
    assert_eq!(tree.selection(entry), Some((4, 4)));
    // From a plain Home, Shift+End selects the whole line.
    tree.text_action(HOME, 1.0, PLAIN);
    tree.text_action(0x4000_001C, 1.0, SHIFT);
    assert_eq!(tree.selection(entry), Some((0, 4)));
}

#[test]
fn the_selection_is_drawn_highlighted_and_only_while_it_is_on() {
    let (mut tree, entry) = editable_entry();
    type_text(&mut tree, "abcd");
    // `SELECTION_COLOR` as bytes: 0.18, 0.42 and 0.85 of 255, truncated.
    let highlight = [45u8, 107, 216, 255];
    let count = |tree: &mut UiTree| {
        let surface = &tree.compose(&HashMap::new())[0];
        surface
            .rgba
            .chunks_exact(4)
            .filter(|p| *p == highlight)
            .count()
    };
    assert_eq!(count(&mut tree), 0);
    tree.select_all(entry);
    assert!(count(&mut tree) > 0);
    tree.deselect(entry);
    assert_eq!(count(&mut tree), 0);
}

/// Moves the pointer to `(x, y)` in the tree's window (the press and the hover follow it).
fn move_pointer(tree: &mut UiTree, x: i32, y: i32) {
    tree.start_input(UI_POINTER_X, x as f32);
    tree.start_input(UI_POINTER_Y, y as f32);
}

#[test]
fn a_press_on_read_only_text_gives_it_the_copy_keys_but_not_the_edit_keys() {
    let (mut tree, entry) = entry_with(&["selectable", "activatable", "one_line"]);
    let (clip, dyn_clip) = clipboard("");
    tree.set_text_clipboard(dyn_clip);
    let units: Vec<u16> = "read only".encode_utf16().collect();
    tree.add_text_with_font(entry, &units, 0, 0);
    move_pointer(&mut tree, 3, 5);
    tree.start_input(LEFT, 1.0);
    tree.stop_input(LEFT);
    // Nothing is active (no editable text), yet the text holds the copy focus.
    assert_eq!(tree.active(), None);
    assert!(tree.text_copy_focus());
    tree.character(u16::from(b'a'), 0, CTRL);
    assert_eq!(tree.selection(entry), Some((0, 9)));
    tree.character(u16::from(b'c'), 0, CTRL);
    assert_eq!(held(&clip).as_deref(), Some("read only"));
    // Cut, paste and typing do nothing on it.
    tree.character(u16::from(b'x'), 0, CTRL);
    tree.character(0x16, 0, CTRL);
    tree.character(u16::from(b'z'), 0, PLAIN);
    assert_eq!(text(&tree, entry), "read only");
    assert_eq!(held(&clip).as_deref(), Some("read only"));
}

#[test]
fn a_press_away_from_read_only_text_takes_the_copy_focus_back() {
    let (mut tree, entry) = entry_with(&["selectable", "activatable", "one_line"]);
    let (clip, dyn_clip) = clipboard("kept");
    tree.set_text_clipboard(dyn_clip);
    let units: Vec<u16> = "read only".encode_utf16().collect();
    tree.add_text_with_font(entry, &units, 0, 0);
    move_pointer(&mut tree, 3, 5);
    tree.start_input(LEFT, 1.0);
    tree.stop_input(LEFT);
    assert!(tree.text_copy_focus());
    // The panel outside the text takes the press.
    move_pointer(&mut tree, 80, 40);
    tree.start_input(LEFT, 1.0);
    tree.stop_input(LEFT);
    assert!(!tree.text_copy_focus());
    tree.character(u16::from(b'a'), 0, CTRL);
    tree.character(u16::from(b'c'), 0, CTRL);
    assert_eq!(tree.selection(entry), None);
    assert_eq!(held(&clip).as_deref(), Some("kept"));
}

/// A clipboard the OS refused: nothing copies, nothing pastes.
#[derive(Debug, Default)]
struct RefusingClipboard;

impl TextClipboard for RefusingClipboard {
    fn text(&self) -> Option<String> {
        None
    }

    fn set_text(&self, _text: &str) -> bool {
        false
    }
}

#[test]
fn a_refused_clipboard_leaves_the_text_and_the_selection_as_they_were() {
    let (mut tree, entry) = editable_entry();
    tree.set_text_clipboard(Arc::new(RefusingClipboard));
    type_text(&mut tree, "abc");
    tree.character(u16::from(b'a'), 0, CTRL);
    // Copy is refused, so cut must not delete the selection it could not copy.
    tree.character(u16::from(b'c'), 0, CTRL);
    tree.character(u16::from(b'x'), 0, CTRL);
    assert_eq!(text(&tree, entry), "abc");
    assert_eq!(tree.selection(entry), Some((0, 3)));
    // An empty or unreadable clipboard pastes nothing.
    tree.character(0x16, 0, CTRL);
    assert_eq!(text(&tree, entry), "abc");
}

#[test]
fn a_paste_drops_control_characters_and_keeps_a_line_break_only_on_a_multi_line_field() {
    let tab_and_bell = "a\tb\u{7}c\u{1b}d\r\ne";
    assert_eq!(
        paste_units(tab_and_bell, false, None),
        "abcd\ne".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        paste_units("x\r\n\r\ny", true, None),
        "x  y".encode_utf16().collect::<Vec<_>>()
    );
}

#[test]
fn the_clipboard_shortcut_letters_map_to_the_control_codes_the_tree_acts_on() {
    use crate::clipboard_control;
    assert_eq!(clipboard_control('a'), Some(1));
    assert_eq!(clipboard_control('C'), Some(3));
    assert_eq!(clipboard_control('v'), Some(0x16));
    assert_eq!(clipboard_control('X'), Some(0x18));
    assert_eq!(clipboard_control('z'), None);
}

#[test]
fn a_drag_held_below_a_multi_line_text_scrolls_it_and_extends_the_selection() {
    let (mut tree, entry) = entry_with(&["selectable", "activatable"]);
    let units: Vec<u16> = "one\ntwo\nthree\nfour\nfive\nsix".encode_utf16().collect();
    tree.add_text_with_font(entry, &units, 0, 0);
    // Press on the first line, then hold the pointer below the 9 pixel high element.
    move_pointer(&mut tree, 1, 3);
    tree.start_input(LEFT, 1.0);
    move_pointer(&mut tree, 1, 30);
    let view_y = |tree: &UiTree| tree.get(entry).unwrap().text.as_ref().unwrap().view.1;
    let mut now = 1.0;
    for _ in 0..10 {
        now += 0.05;
        tree.global_loop(now);
    }
    assert!(view_y(&tree) > 0, "the view should scroll down while held");
    // The view runs to the bottom; the pointer then lands on the last visible line.
    for _ in 0..400 {
        now += 0.05;
        tree.global_loop(now);
    }
    let (start, end) = tree.selection(entry).unwrap();
    // The anchor is where the press landed (x 1 is in front of the first glyph's right half).
    assert_eq!(start, 1);
    assert!(
        end >= units.len() - 3,
        "selection reaches the last visible line"
    );
    // Released: the view stays where it is.
    tree.stop_input(LEFT);
    let at_release = view_y(&tree);
    now += 1.0;
    tree.global_loop(now);
    assert_eq!(view_y(&tree), at_release);
}

#[test]
fn a_drag_above_a_one_line_text_does_not_scroll_it() {
    let (mut tree, entry) = entry_with(&["selectable", "activatable", "one_line"]);
    let units: Vec<u16> = "a long line of text".encode_utf16().collect();
    tree.add_text_with_font(entry, &units, 0, 0);
    move_pointer(&mut tree, 1, 3);
    tree.start_input(LEFT, 1.0);
    move_pointer(&mut tree, 1, -30);
    for frame in 1..40 {
        tree.global_loop(f64::from(frame) * 0.05);
    }
    assert_eq!(tree.get(entry).unwrap().text.as_ref().unwrap().view.1, 0);
}
