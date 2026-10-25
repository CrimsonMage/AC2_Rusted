//! `StringInfo` rendering over the Legions DATs (`AC2_DAT_DIR`): the `StringState`, chat and
//! UI strings with variables, and every table entry through the meta-language.

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{
    inq_string, load_string_state, GetStringError, RenderInfo, RenderVar, ResourceStore,
    StoreStringTables, StringTables, STRING_STATE_FIRST, STRING_STATE_LAST, STRING_TABLE_FIRST,
    STRING_TABLE_LAST,
};

/// The `StringState` `StringStateInfo::Startup` finds through enum 0x40000001.
const STRING_STATE: u32 = 0x5800_0000;
/// The communication table (enum 0x41000004) and the UI table (enum 0x41000018).
const COMMUNICATION: u32 = 0x2500_0002;
const UI: u32 = 0x2500_0012;
/// `CCommunicationSystem`'s variables: the message and the speaker's name.
const VAR_TEXT: u32 = 0x9B4E_3800;
const VAR_NAME: u32 = 0x9D06_6C50;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn literal(text: &str) -> RenderVar {
    RenderVar::Info(Box::new(RenderInfo {
        literal: Some(wide(text)),
        ..RenderInfo::default()
    }))
}

fn render(
    tables: &StoreStringTables,
    table_id: u32,
    string_id: u32,
    vars: Vec<(u32, RenderVar)>,
) -> String {
    let info = RenderInfo {
        string_id,
        table_id,
        literal: None,
        variables: vars,
    };
    String::from_utf16_lossy(&inq_string(tables, &info, false).expect("renders"))
}

#[test]
fn the_string_state_decodes_with_english_number_settings() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Local, STRING_STATE_FIRST, STRING_STATE_LAST)
        .collect();
    assert_eq!(ids, [STRING_STATE]);
    let store = ResourceStore::new(dataset);
    let state = load_string_state(&store, DataId::new(STRING_STATE))
        .unwrap()
        .unwrap();
    assert_eq!(
        (state.base, state.grouping_size, state.leading_zero),
        (10, 3, 0)
    );
    assert_eq!(state.decimal_separator, wide("."));
    assert_eq!(state.grouping_separator, wide(","));
    assert_eq!(state.negative_number_format, wide("-%s"));
    assert_eq!(state.male_player_letters, wide("mn"));
    assert_eq!(state.female_player_letters, wide("fn"));
    assert!(state.is_number_singular(1) && !state.is_number_singular(2));
}

#[test]
fn chat_and_ui_strings_render_with_their_variables() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let tables = StoreStringTables::new(ResourceStore::new(dataset), STRING_STATE);
    // "You say, ..." (`CCommunicationSystem` 0x6 for the player).
    assert_eq!(
        render(
            &tables,
            COMMUNICATION,
            0xD85E_9379,
            vec![(VAR_TEXT, literal("Hello there"))]
        ),
        "You say, \"Hello there\"\n"
    );
    // "... says, ...": a player's name carries its meta letters, so no article.
    let says = |name: &str| {
        render(
            &tables,
            COMMUNICATION,
            0xCF6B_AF4F,
            vec![(VAR_NAME, literal(name)), (VAR_TEXT, literal("hi"))],
        )
    };
    assert_eq!(says("Explorer [Mv]"), "Explorer says, \"hi\"\n");
    assert_eq!(says("guard"), "The guard says, \"hi\"\n");
    // "N days, N hours" with singular choices.
    let duration = |days: i32, hours: i32| {
        render(
            &tables,
            UI,
            0x2FAF_12C4,
            vec![
                (0xC987_CF30, RenderVar::Int(days)),
                (0xB1BF_AF2E, RenderVar::Int(hours)),
            ],
        )
    };
    assert_eq!(duration(1, 3), "1 day, 3 hours");
    assert_eq!(duration(2, 1), "2 days, 1 hour");
}

/// Every entry of every table, each variable a literal `1`: the meta-language accepts every
/// shipped string. The 1779 entries that name a private table name tables the DVD does not
/// ship, so they fail as retail's do (`gsMissingPrivateTable`).
#[test]
fn every_table_entry_renders() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Local, STRING_TABLE_FIRST, STRING_TABLE_LAST)
        .collect();
    let tables = StoreStringTables::new(ResourceStore::new(dataset.clone()), STRING_STATE);
    let (mut rendered, mut private) = (0, 0);
    for did in ids {
        let table = tables.table(did).expect("table");
        for (&key, entry) in &table.strings {
            let variables = entry.variables.iter().map(|&v| (v, literal("1"))).collect();
            let info = RenderInfo {
                string_id: key,
                table_id: did,
                literal: None,
                variables,
            };
            let outcome = inq_string(&tables, &info, false).map_err(|f| f.code);
            if entry.table == 0 {
                assert!(
                    outcome.is_ok(),
                    "{did:08x}/{key:08x}: {outcome:?} {entry:?}"
                );
                rendered += 1;
            } else {
                assert!(!dataset.contains(DatFileKind::Local, entry.table));
                assert_eq!(outcome, Err(GetStringError::MissingPrivateTable));
                private += 1;
            }
        }
    }
    assert_eq!((rendered, private), (47_423, 1779));
}
