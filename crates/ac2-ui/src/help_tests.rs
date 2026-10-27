//! Help id to page resolution: a synthetic table for the logic, and the real table and shipped
//! pages (`AC2_DAT_DIR`, `AC2_CONTENT_DIR`) for the data.

use std::path::{Path, PathBuf};

use ac2_db::{ResourceStore, StringTable, StringTableString};

use crate::help::*;
use crate::ksml::*;
use crate::token::tokenize;

/// The content directory: `AC2_CONTENT_DIR`, else the workspace's `payload/ProgramFiles/Content`.
fn content_dir() -> PathBuf {
    match std::env::var_os("AC2_CONTENT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../payload/ProgramFiles/Content"),
    }
}

fn table_with(entries: &[(&str, &str)]) -> StringTable {
    let mut table = StringTable::default();
    for (name, page) in entries {
        table.strings.insert(
            tokenize(name),
            StringTableString {
                pieces: vec![Some(page.encode_utf16().collect())],
                ..StringTableString::default()
            },
        );
    }
    table
}

#[test]
fn token_zero_is_the_default_page_and_other_tokens_read_the_table() {
    let table = table_with(&[("ID_Help_Shop", "example-shop.ksml")]);
    assert_eq!(
        help_page_name(&table, 0).as_deref(),
        Some(DEFAULT_HELP_PAGE)
    );
    assert_eq!(
        help_page_name(&table, tokenize("ID_Help_Shop")).as_deref(),
        Some("example-shop.ksml")
    );
    assert_eq!(help_page_name(&table, tokenize("ID_Help_Missing")), None);
}

#[test]
fn content_prefix_maps_onto_the_content_directory() {
    let dir = Path::new("/content-root");
    assert_eq!(
        help_page_path(dir, "content/ac2-help.ksml"),
        dir.join("ac2-help.ksml")
    );
    assert_eq!(
        help_page_path(dir, "Content\\ac2-help.ksml"),
        dir.join("ac2-help.ksml")
    );
    assert_eq!(
        help_page_path(dir, "panelbook.ksml"),
        dir.join("panelbook.ksml")
    );
}

#[test]
fn unknown_token_fails_before_any_file_is_read() {
    let table = table_with(&[]);
    let err = load_help_page(
        Path::new("/nonexistent-content"),
        &table,
        tokenize("ID_Help_Shop"),
    )
    .unwrap_err();
    assert!(matches!(err, HelpError::UnknownToken(_)), "{err:?}");
}

#[test]
fn topics_are_distinct_id_names() {
    let mut ids: Vec<u32> = HELP_TOPICS.iter().map(help_token).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), HELP_TOPICS.len());
    assert!(HELP_TOPICS.iter().all(|t| t.id.starts_with("ID_Help_")));
}

#[test]
fn every_help_topic_names_a_page_in_the_real_table() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    let table = load_help_table(&store)
        .unwrap()
        .expect("the help string table enum resolves");
    let table = &*table;
    let content = content_dir();
    for topic in HELP_TOPICS {
        let page = help_page_name(table, help_token(topic))
            .unwrap_or_else(|| panic!("{} has a table entry", topic.id));
        assert!(
            page.ends_with(".ksml") && !page.contains(' ') && !page.contains('\0'),
            "{}: {page:?} is a page file name",
            topic.id
        );
        if content.is_dir() {
            assert!(
                help_page_path(&content, &page).is_file(),
                "{}: page {page} exists",
                topic.id
            );
        }
    }
    // Token 0 is the default page, which must also ship.
    if content.is_dir() {
        assert!(help_page_path(&content, DEFAULT_HELP_PAGE).is_file());
    }
}

#[test]
fn every_shipped_ksml_page_parses_and_each_topic_page_loads() {
    let content = content_dir();
    if !content.is_dir() {
        return;
    }
    let mut failures = Vec::new();
    let mut parsed = 0;
    for entry in std::fs::read_dir(&content).unwrap() {
        let path = entry.unwrap().path();
        if path
            .extension()
            .is_none_or(|ext| !ext.eq_ignore_ascii_case("ksml"))
        {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        match parse_ksml(&bytes) {
            Ok(_) => parsed += 1,
            Err(err) => failures.push(format!("{}: {err}", path.display())),
        }
    }
    assert!(failures.is_empty(), "pages that fail: {failures:#?}");
    assert!(parsed > 100, "parsed {parsed} pages");

    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    let table = load_help_table(&store).unwrap().expect("help table");
    let table = &*table;
    for topic in HELP_TOPICS {
        let doc = load_help_page(&content, table, help_token(topic))
            .unwrap_or_else(|e| panic!("{}: {e}", topic.id));
        assert!(!doc.nodes.is_empty(), "{} has content", topic.id);
    }
    let default = load_help_page(&content, table, 0).expect("default page");
    assert!(!default.nodes.is_empty());
}
