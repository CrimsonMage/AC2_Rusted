//! `StringInfo` text over hand-built tables: variables of every kind, nested and private
//! tables, and retail's error texts.

use std::sync::Arc;

use crate::string_meta_text::wide;
use crate::{
    get_log_string, get_string, inq_string, inq_string_internal, render_with_table, GetStringError,
    PreloadedStringTables, RenderInfo, RenderVar, StringState, StringTable, StringTableString,
};

const MAIN: u32 = 0x2500_0001;
const OTHER: u32 = 0x2500_0002;
const PRIVATE: u32 = 0x2500_0003;
const V1: u32 = 0x9B4E_3800;
const V2: u32 = 0x9D06_6C50;

fn entry(pieces: &[&str], variables: &[u32]) -> StringTableString {
    StringTableString {
        table: 0,
        pieces: pieces.iter().map(|p| Some(wide(p))).collect(),
        variables: variables.to_vec(),
        variable_names: Vec::new(),
    }
}

fn english() -> StringState {
    StringState {
        base: 10,
        num_decimal_digits: 5,
        grouping_size: 3,
        numerals: wide("0123456789"),
        decimal_separator: wide("."),
        grouping_separator: wide(","),
        negative_number_format: wide("-%s"),
        singular: [0, 1, 0, 0, 0],
        male_player_letters: wide("mn"),
        female_player_letters: wide("fn"),
        treasure_prefix_letters: wide("v"),
        treasure_middle_letters: wide("mfnp"),
        treasure_suffix_letters: wide("s"),
        ..Default::default()
    }
}

fn table(entries: Vec<(u32, StringTableString)>) -> Arc<StringTable> {
    Arc::new(StringTable {
        header: (0, 0xFFFF, 0xFFFF_FFFF, 0xFFFF),
        strings: entries.into_iter().collect(),
    })
}

fn tables() -> PreloadedStringTables {
    let mut redirect = entry(&[], &[]);
    redirect.table = PRIVATE;
    let mut t = PreloadedStringTables::default();
    t.insert_table(
        MAIN,
        table(vec![
            (1, entry(&["You say, \\q", "\\q"], &[V1])),
            (
                2,
                entry(
                    &["#1:{The[!n]} #1:", " has #2:", " #2:{coins|coin[1]}."],
                    &[V2, V1],
                ),
            ),
            (3, entry(&["\\[", "\\] ", ""], &[V2, V1])),
            (4, entry(&["Took ", "."], &[V1])),
            (5, redirect),
            (6, entry(&["{a||b}"], &[])),
            (7, entry(&["Item: "], &[])),
            (8, entry(&["[", "]"], &[V1])),
        ]),
    );
    t.insert_table(OTHER, table(vec![(9, entry(&["a guard"], &[]))]));
    t.insert_table(PRIVATE, table(vec![(5, entry(&["private ", ""], &[V1]))]));
    t.set_state(Some(Arc::new(english())));
    t
}

fn info(string_id: u32, variables: Vec<(u32, RenderVar)>) -> RenderInfo {
    RenderInfo {
        string_id,
        table_id: MAIN,
        literal: None,
        variables,
    }
}

fn literal(text: &str) -> RenderVar {
    RenderVar::Info(Box::new(RenderInfo {
        literal: Some(wide(text)),
        ..RenderInfo::default()
    }))
}

fn text(t: &PreloadedStringTables, info: &RenderInfo) -> String {
    String::from_utf16_lossy(&get_string(t, info, false))
}

#[test]
fn speech_with_a_nested_literal_unescapes_quotes() {
    let t = tables();
    assert_eq!(
        text(&t, &info(1, vec![(V1, literal("Hello there"))])),
        "You say, \"Hello there\""
    );
}

#[test]
fn numbers_names_and_plurals() {
    let t = tables();
    let line =
        |name: RenderVar, count: RenderVar| text(&t, &info(2, vec![(V2, name), (V1, count)]));
    assert_eq!(
        line(literal("Explorer [M]"), RenderVar::Int(1)),
        "Explorer has 1 coin."
    );
    assert_eq!(
        line(literal("guard"), RenderVar::UInt(12000)),
        "The guard has 12,000 coins."
    );
    // `StringID` variables find the outer table, or the one they name.
    assert_eq!(
        line(RenderVar::string_id(9, OTHER), RenderVar::LInt(-5)),
        "The a guard has -5 coins."
    );
    assert_eq!(
        line(
            literal("x"),
            RenderVar::Float {
                value: 2.5,
                precision: 1
            }
        ),
        "The x has 2.5 coins."
    );
}

#[test]
fn raw_strings_are_escaped_like_add_variable_string() {
    let t = tables();
    let i = info(
        3,
        vec![
            (V2, RenderVar::String(wide("General"))),
            (V1, RenderVar::String(wide("a [b] {c}"))),
        ],
    );
    assert_eq!(text(&t, &i), "[General] a [b] {c}");
    // Meta letters stay escaped in the retained (not unescaped) text.
    let raw = inq_string_internal(&t, &i, true).unwrap();
    assert_eq!(
        String::from_utf16_lossy(&raw),
        "\\[General\\] a \\[b\\] \\{c\\}"
    );
}

#[test]
fn meta_letters_strip_unless_retained() {
    let t = tables();
    let i = info(8, vec![(V1, RenderVar::String(wide("x")))]);
    // The entry's own `[...]` is meta letters: stripped by default.
    assert_eq!(text(&t, &i), "");
    assert_eq!(String::from_utf16_lossy(&get_string(&t, &i, true)), "[x]");
}

#[test]
fn private_tables_hold_the_entry() {
    let t = tables();
    let i = info(5, vec![(V1, literal("text"))]);
    assert_eq!(text(&t, &i), "private text");
    let mut without = PreloadedStringTables::default();
    without.insert_table(
        MAIN,
        table(vec![(5, {
            let mut e = entry(&[], &[]);
            e.table = PRIVATE;
            e
        })]),
    );
    let err = inq_string(&without, &i, false).unwrap_err();
    assert_eq!(err.code, GetStringError::MissingPrivateTable);
}

#[test]
fn failures_report_codes_and_retail_text() {
    let t = tables();
    let err = inq_string(&t, &info(4, vec![]), false).unwrap_err();
    assert_eq!(err.code, GetStringError::MissingVariable);
    assert_eq!(
        String::from_utf16_lossy(&err.text),
        "<string table error; token: 0x00000004 tableDID: 0x25000001>"
    );
    let skipped = info(4, vec![(V1, RenderVar::Skipped)]);
    assert_eq!(
        inq_string(&t, &skipped, false).unwrap_err().code,
        GetStringError::MissingVariable
    );
    let missing = info(0x77, vec![]);
    assert_eq!(
        inq_string(&t, &missing, false).unwrap_err().code,
        GetStringError::MissingString
    );
    let uninit = RenderInfo::default();
    assert_eq!(
        inq_string(&t, &uninit, false).unwrap_err().code,
        GetStringError::Uninitialized
    );
    let no_table = RenderInfo {
        table_id: 0x2500_0099,
        ..info(1, vec![])
    };
    assert_eq!(
        inq_string(&t, &no_table, false).unwrap_err().code,
        GetStringError::MissingTable
    );
    assert_eq!(
        inq_string(&t, &info(6, vec![]), false).unwrap_err().code,
        GetStringError::MetaLanguageError
    );
    // A nested failure becomes the variable's text.
    let nested = info(4, vec![(V1, RenderVar::string_id(0x55, 0))]);
    assert_eq!(
        text(&t, &nested),
        "Took <string table error; token: 0x00000055 tableDID: 0x25000001>."
    );
}

#[test]
fn log_strings_wrap_or_explain() {
    let t = tables();
    let ok = get_log_string(&t, &info(7, vec![]), false);
    assert_eq!(String::from_utf16_lossy(&ok), "(Item: )");
    let bad = get_log_string(&t, &info(4, vec![]), false);
    assert_eq!(
        String::from_utf16_lossy(&bad),
        "<could not render string: table 0x25000001 token 0x00000004. Reason = 6>"
    );
}

#[test]
fn literals_and_the_script_hook() {
    let t = tables();
    let lit = RenderInfo {
        literal: Some(wide("Bob [M]")),
        ..RenderInfo::default()
    };
    assert_eq!(text(&t, &lit), "Bob");
    assert_eq!(
        String::from_utf16_lossy(&get_string(&t, &lit, true)),
        "Bob [M]"
    );
    let main = t.table_ref(MAIN);
    let rendered = render_with_table(&t, &main, &info(1, vec![(V1, literal("x"))])).unwrap();
    assert_eq!(String::from_utf16_lossy(&rendered), "You say, \"x\"");
    // The plain overload joins the pieces as they are, following a private table.
    let plain = main.plain_entry(&t, 1).unwrap();
    assert_eq!(String::from_utf16_lossy(&plain), "You say, \\q\\q");
    let plain = main.plain_entry(&t, 5).unwrap();
    assert_eq!(String::from_utf16_lossy(&plain), "private ");
    assert_eq!(
        main.plain_entry(&t, 0x77),
        Err(GetStringError::MissingString)
    );
}

impl PreloadedStringTables {
    fn table_ref(&self, did: u32) -> StringTable {
        use crate::StringTables;
        (*self.table(did).expect("table")).clone()
    }
}
