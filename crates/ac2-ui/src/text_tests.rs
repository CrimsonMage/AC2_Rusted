//! Text layout and drawing with a hand-built font.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_assets::{Ac2Font, Ac2FontChar};

use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, Rect, StateDesc};
use crate::text::{recalculate_lines, Glyph, GlyphLine};
use crate::token::{tokenize, STATELESS, TEXT, TEXT_CENTER, TEXT_ENTRY};
use crate::{UiTextData, UiTree};

/// Two rows, 1 bit per pixel: `a` (3 wide, columns 0..=2 inked), `b` (3 wide, 3..=5), a space
/// (2 wide) and `?` (3 wide, 8..=10).
pub(crate) fn font() -> Arc<Ac2Font> {
    let chars = vec![
        Ac2FontChar {
            width: 3,
            unicode: u16::from(b'a'),
            offset: 0,
        },
        Ac2FontChar {
            width: 3,
            unicode: u16::from(b'b'),
            offset: 3,
        },
        Ac2FontChar {
            width: 2,
            unicode: u16::from(b' '),
            offset: 6,
        },
        Ac2FontChar {
            width: 3,
            unicode: u16::from(b'?'),
            offset: 8,
        },
    ];
    Arc::new(Ac2Font::new(
        [2, 3, 3, 16],
        false,
        chars,
        vec![0xFC, 0xE0, 0xFC, 0xE0],
    ))
}

fn glyphs(text: &str) -> Vec<Glyph> {
    let font = font();
    text.encode_utf16()
        .map(|ch| {
            let width = if ch == 10 {
                1
            } else {
                font.char_desc(ch).unwrap().width as i32
            };
            Glyph {
                ch,
                width,
                height: 2,
                font: Some(Arc::clone(&font)),
                color: [1.0; 4],
                outline: false,
            }
        })
        .collect()
}

fn line(index: usize, width: i32) -> GlyphLine {
    GlyphLine {
        index,
        width,
        height: 2,
    }
}

#[test]
fn lines_wrap_at_the_last_break_and_the_next_line_keeps_the_space() {
    // "ab ab" is 14 wide; at 10 the second "ab" moves down with the space before it.
    let (lines, width, height) = recalculate_lines(&glyphs("ab ab"), 10, false);
    assert_eq!(lines, [line(0, 6), line(2, 8)]);
    assert_eq!((width, height), (8, 4));
    // One line never wraps.
    let (lines, _, _) = recalculate_lines(&glyphs("ab ab"), 10, true);
    assert_eq!(lines, [line(0, 14)]);
    // A newline ends its line; the newline glyph is one pixel wide.
    let (lines, _, _) = recalculate_lines(&glyphs("a\nb"), 100, false);
    assert_eq!(lines, [line(0, 4), line(2, 3)]);
}

pub(crate) struct Data;

impl UiTextData for Data {
    fn font(&self, _did: u32) -> Option<Arc<Ac2Font>> {
        Some(font())
    }

    fn default_font(&self) -> Option<Arc<Ac2Font>> {
        Some(font())
    }

    fn string(&self, _table: u32, id: u32) -> Option<Vec<u16>> {
        (id == 7).then(|| "ab".encode_utf16().collect())
    }
}

#[test]
fn a_centred_entry_draws_each_glyph_but_its_last_column() {
    let attributes = vec![
        AttributeDesc {
            id: TEXT_ENTRY,
            value: AttributeValue::StringId(7),
        },
        AttributeDesc {
            id: TEXT_CENTER,
            value: AttributeValue::Bool(true),
        },
    ];
    let label = ElementDesc {
        id: tokenize("label"),
        kind: TEXT,
        default_state: STATELESS,
        rect: Rect {
            x0: 0,
            y0: 0,
            x1: 9,
            y1: 1,
        },
        z_level: 0,
        edges: [0; 4],
        states: vec![StateDesc {
            id: STATELESS,
            flags: 0,
            rect: Rect {
                x0: -1,
                y0: -1,
                x1: -1,
                y1: -1,
            },
            z_level: 0,
            attributes,
            media: Vec::new(),
            transitions: Vec::new(),
            children: Vec::new(),
        }],
    };
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0x2500_0001,
        width: 100,
        height: 50,
        elements: vec![Arc::new(label)],
    });
    let mut tree = UiTree::new(100, 50);
    tree.set_text_data(Arc::new(Data));
    let root = tree.create_root(layout, tokenize("label")).unwrap();
    assert_eq!(tree.get(root).unwrap().text().as_deref(), Some("ab"));
    let surface = &tree.compose(&HashMap::new())[0];
    // Text 6 wide centred in 10: "a" at 2, "b" at 5, each drawing two of its three columns.
    let row: Vec<bool> = (0..10).map(|x| surface.rgba[x * 4 + 3] != 0).collect();
    let inked = [
        false, false, true, true, false, true, true, false, false, false,
    ];
    assert_eq!(row, inked);
    assert_eq!(&surface.rgba[8..12], &[255, 255, 255, 255]);
}

pub(crate) fn element(
    name: &str,
    kind: u32,
    rect: Rect,
    attributes: Vec<AttributeDesc>,
    children: Vec<ElementDesc>,
) -> ElementDesc {
    ElementDesc {
        id: tokenize(name),
        kind,
        default_state: STATELESS,
        rect,
        z_level: 0,
        edges: [0; 4],
        states: vec![StateDesc {
            id: STATELESS,
            flags: 0,
            rect: Rect {
                x0: -1,
                y0: -1,
                x1: -1,
                y1: -1,
            },
            z_level: 0,
            attributes,
            media: Vec::new(),
            transitions: Vec::new(),
            children: children.into_iter().map(Arc::new).collect(),
        }],
    }
}

pub(crate) fn flag(name: &str) -> AttributeDesc {
    AttributeDesc {
        id: tokenize(name),
        value: AttributeValue::Bool(true),
    }
}

#[test]
fn clicking_an_editable_field_takes_typing_until_a_click_elsewhere() {
    let entry = element(
        "entry",
        TEXT,
        Rect {
            x0: 0,
            y0: 0,
            x1: 49,
            y1: 9,
        },
        vec![flag("editable"), flag("activatable"), flag("one_line")],
        vec![],
    );
    let panel = element(
        "panel",
        0x4DA7_2ECC,
        Rect {
            x0: 0,
            y0: 0,
            x1: 99,
            y1: 49,
        },
        vec![],
        vec![entry],
    );
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 100,
        height: 50,
        elements: vec![Arc::new(panel)],
    });
    let mut tree = UiTree::new(100, 50);
    tree.set_text_data(Arc::new(Data));
    let panel = tree.create_root(layout, tokenize("panel")).unwrap();
    let entry = tree.child_recursive(panel, tokenize("entry")).unwrap();
    let click = |tree: &mut UiTree, x: i32, y: i32| {
        tree.start_input(crate::UI_POINTER_X, x as f32);
        tree.start_input(crate::UI_POINTER_Y, y as f32);
        tree.start_input(0x4000_0004, 1.0);
        tree.stop_input(0x4000_0004);
        tree.take_messages()
    };
    let messages = click(&mut tree, 10, 5);
    assert_eq!(tree.active(), Some(entry));
    assert!(tree.text_mode());
    assert!(messages.iter().any(|m| m.message == tokenize("activated")));
    let ch = |c: char| c as u16;
    tree.character(ch('a'), 0, crate::chat::Modifiers::default());
    tree.character(ch('b'), 0, crate::chat::Modifiers::default());
    assert_eq!(tree.get(entry).unwrap().text().as_deref(), Some("ab"));
    // Backspace, then the cursor-left action, then a character in front.
    tree.character(8, 0, crate::chat::Modifiers::default());
    tree.text_action(0x4000_0013, 1.0, crate::chat::Modifiers::default());
    tree.character(ch('x'), 0, crate::chat::Modifiers::default());
    assert_eq!(tree.get(entry).unwrap().text().as_deref(), Some("xa"));
    // Enter types nothing on one line, but goes out as character input.
    tree.take_messages();
    tree.character(13, 0, crate::chat::Modifiers::default());
    assert_eq!(tree.get(entry).unwrap().text().as_deref(), Some("xa"));
    let messages = tree.take_messages();
    assert!(messages
        .iter()
        .any(|m| m.message == tokenize("character_input") && m.data == 13));
    // A press elsewhere deactivates it.
    let messages = click(&mut tree, 80, 30);
    assert_eq!(tree.active(), None);
    assert!(!tree.text_mode());
    assert!(messages
        .iter()
        .any(|m| m.message == tokenize("deactivated")));
}

/// A one-line editable, selectable entry, 50 pixels wide, in a panel the tree is rooted at.
pub(crate) fn editable_entry() -> (UiTree, usize) {
    entry_with(&["editable", "selectable", "activatable", "one_line"])
}

/// A one-line, 50 pixels wide text element with the layout flags `flags`, in a panel the tree is
/// rooted at; returns the tree and the element. It is activated when editable.
pub(crate) fn entry_with(flags: &[&str]) -> (UiTree, usize) {
    let entry = element(
        "entry",
        TEXT,
        Rect {
            x0: 0,
            y0: 0,
            x1: 49,
            y1: 9,
        },
        flags.iter().map(|name| flag(name)).collect(),
        vec![],
    );
    let panel = element(
        "panel",
        0x4DA7_2ECC,
        Rect {
            x0: 0,
            y0: 0,
            x1: 99,
            y1: 49,
        },
        vec![],
        vec![entry],
    );
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 100,
        height: 50,
        elements: vec![Arc::new(panel)],
    });
    let mut tree = UiTree::new(100, 50);
    tree.set_text_data(Arc::new(Data));
    let panel = tree.create_root(layout, tokenize("panel")).unwrap();
    let entry = tree.child_recursive(panel, tokenize("entry")).unwrap();
    if flags.contains(&"editable") {
        assert!(tree.activate(entry));
    }
    (tree, entry)
}

#[test]
fn shift_selects_typing_replaces_and_the_clipboard_moves_text() {
    use crate::chat::Modifiers;
    let (mut tree, entry) = editable_entry();
    let plain = Modifiers::default();
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    let text = |tree: &UiTree| tree.get(entry).unwrap().text().unwrap();
    for c in "abcd".chars() {
        tree.character(c as u16, 0, plain);
    }
    // Shift and the left arrow extend the selection from the cursor.
    tree.text_action(0x4000_0013, 1.0, shift);
    tree.text_action(0x4000_0013, 1.0, shift);
    assert_eq!(tree.selection(entry), Some((2, 4)));
    assert_eq!(
        tree.selected_text(entry),
        "cd".encode_utf16().collect::<Vec<_>>()
    );
    // A plain move drops the selection (cursor 2 to 1); typing goes in there.
    tree.text_action(0x4000_0013, 1.0, plain);
    assert_eq!(tree.selection(entry), None);
    tree.character(b'x' as u16, 0, plain);
    assert_eq!(text(&tree), "axbcd");
    // Select all, cut (Ctrl+X), paste (Ctrl+V) at the cursor.
    tree.select_all(entry);
    assert_eq!(tree.selection(entry), Some((0, 5)));
    tree.character(0x18, 0, ctrl);
    assert_eq!(text(&tree), "");
    tree.character(0x16, 0, ctrl);
    assert_eq!(text(&tree), "axbcd");
    // Delete removes the character at the cursor; Home then drops the selection.
    tree.text_action(0x4000_0013, 1.0, plain);
    tree.character(0, 0x2E, plain);
    assert_eq!(text(&tree), "axbc");
    tree.select_all(entry);
    tree.character(8, 0, plain);
    assert_eq!(text(&tree), "");
    tree.character(b'q' as u16, 0, plain);
    tree.select_all(entry);
    tree.text_action(0x4000_001B, 1.0, plain);
    assert_eq!(tree.selection(entry), None);
    assert_eq!(tree.get(entry).unwrap().text.as_ref().unwrap().cursor, 0);
}

#[test]
fn a_one_line_entry_scrolls_to_keep_the_cursor_in_view() {
    let (mut tree, entry) = editable_entry();
    for _ in 0..30 {
        tree.character(u16::from(b'a'), 0, crate::chat::Modifiers::default());
    }
    // Each "a" is 3 pixels: the cursor sits at 90, the last visible column of 50 is 49.
    let text = tree.get(entry).unwrap().text.as_ref().unwrap();
    assert_eq!(text.view.0, 90 - 50 + 1);
}

/// `UIElement_Text::DetermineFont` on the real DATs (`AC2_DAT_DIR`): Japanese the element's font
/// lacks switches it to a fallback font (the Japanese one), whose glyphs it has; Latin text keeps the font.
#[test]
fn text_the_font_lacks_switches_to_a_fallback_font_that_has_it() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = crate::build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let name = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == crate::token::tokenize("SelectionName"))
        .expect("SelectionName");
    let font_of =
        |tree: &crate::UiTree| tree.get(name).unwrap().text.as_ref().unwrap().font.clone();
    tree.set_text(name, "Gnome");
    let latin = font_of(tree).expect("a font");
    // `InqSizewMargins` measures in the font `DetermineFont` picks: Japanese is measured in the
    // fallback (the element's own font would draw each character as '?').
    let measured = tree.text_line_width(name, "こんにちは漢字").unwrap();
    let as_unknown = tree.text_line_width(name, "???????").unwrap();
    assert_ne!(measured, as_unknown, "measured in the fallback font");
    assert!(
        font_of(tree).is_some_and(|f| Arc::ptr_eq(&f, &latin)),
        "measuring switches nothing"
    );
    assert!(latin.contains('G' as u16));
    let japanese: Vec<u16> = "こんにちは漢字".encode_utf16().collect();
    assert!(
        japanese.iter().any(|&c| !latin.contains(c)),
        "the default font lacks kana"
    );
    tree.set_text(name, "こんにちは漢字");
    let switched = font_of(tree).expect("a font");
    assert!(
        japanese.iter().all(|&c| switched.char_desc(c).is_some()),
        "the fallback has it"
    );
    assert_eq!(
        tree.get(name).unwrap().text(),
        Some("こんにちは漢字".to_owned())
    );
}

/// `UIElement_Text::ScrollToPosition` (`GlyphList::FindXYFromPosition`): the view's top goes to
/// the line holding the position, kept in bounds; `ScrollToEnd` and `ScrollToHome` go through
/// it, and a position past the text moves nothing.
#[test]
fn scrolling_to_a_position_puts_its_line_at_the_top() {
    let (mut tree, entry) = entry_with(&[]);
    tree.set_text(entry, "a\na\na\na\na\na\na\na\na\na");
    let view = |tree: &UiTree| tree.get(entry).unwrap().text.as_ref().unwrap().view.1;
    tree.scroll_to_position(entry, 4);
    let text = tree.get(entry).unwrap().text.as_ref().unwrap();
    let height = text.lines[0].height;
    assert!(height > 0 && text.lines.len() == 10);
    assert_eq!(view(&tree), 2 * height, "glyph 4 is on the third line");
    let bottom = text.paper.1 - 10;
    tree.scroll_to_end(entry);
    assert_eq!(view(&tree), bottom, "the last line's top, kept in bounds");
    tree.scroll_to_position(entry, 99);
    assert_eq!(view(&tree), bottom, "outside the text");
    tree.scroll_to_home(entry);
    assert_eq!(view(&tree), 0);
}
