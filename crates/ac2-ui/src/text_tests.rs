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
fn font() -> Arc<Ac2Font> {
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

struct Data;

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

fn element(
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

fn flag(name: &str) -> AttributeDesc {
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
    tree.character(ch('a'), 0, false);
    tree.character(ch('b'), 0, false);
    assert_eq!(tree.get(entry).unwrap().text().as_deref(), Some("ab"));
    // Backspace, then the cursor-left action, then a character in front.
    tree.character(8, 0, false);
    tree.text_action(0x4000_0013, 1.0, false);
    tree.character(ch('x'), 0, false);
    assert_eq!(tree.get(entry).unwrap().text().as_deref(), Some("xa"));
    // Enter types nothing on one line, but goes out as character input.
    tree.take_messages();
    tree.character(13, 0, false);
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
