//! The chat options page (`gmChatOptionsUI`) on the script VM (`AC2_DAT_DIR`): a window's filter
//! ticked on the page and applied through the server double's player script reaches the client's
//! chat settings, and the other settings of the window keep their values. The confirmation of
//! Apply is the ac2-ui tests'; here the page's apply runs directly.

mod common;

use std::sync::Arc;

use ac2_client::{chat_option_commands, ui_chat_settings, FrameInput};
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_ui::{build_gameplay_hud, CHAT_TEXT_TYPES};
use common::*;
use legions_server_double::Spawn;

/// `ID_Option_ChatSayMessages` and its first window's flag.
const SAY: &str = "ID_Option_ChatSayMessages";
const WINDOW_1: &str = "ID_Option_Chat1";
const TEXT_TYPE_SAY: u32 = 0x10;

#[test]
fn a_window_filter_ticked_on_the_page_reaches_the_players_chat_settings() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ui_store = Arc::new(ac2_db::ResourceStore::new(dataset.clone()));
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    step(&mut client, &server, &idle);
    server.borrow_mut().send_init_player();
    for _ in 0..8 {
        step(&mut client, &server, &idle);
    }
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );

    // The page as the HUD builds it, refreshed from the player's chat settings.
    let mut hud = build_gameplay_hud(&ui_store, 1024, 768).unwrap();
    let mut page = hud.chat_options.take().expect("ChatPage");
    let before = ui_chat_settings(&client.chat_settings().unwrap());
    page.refresh(&mut hud.tree, &before);
    assert!(!page.changed());

    // Window 1's say filter is flipped on the page and applied.
    let on = before.filter(1) & TEXT_TYPE_SAY != 0;
    assert!(page.set_value(&mut hud.tree, SAY, WINDOW_1, if on { 0.0 } else { 1.0 }));
    assert!(page.changed());
    page.apply(&mut hud.tree);
    let commands = chat_option_commands(page.take_requests());
    client
        .use_time(&FrameInput {
            commands,
            ..FrameInput::default()
        })
        .unwrap();
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert!(
        client.take_script_problems().is_empty(),
        "{:?}",
        client.take_script_problems()
    );

    // The player's settings now hold the flipped filter; window 1's other types are as they were
    // (the entry line's bit aside, which the page does not show).
    let after = ui_chat_settings(&client.chat_settings().unwrap());
    assert_eq!(after.filter(1) & TEXT_TYPE_SAY != 0, !on);
    for bit in [0x8, 0x20, 0x80, 0x100] {
        assert_eq!(after.filter(1) & bit, before.filter(1) & bit, "{bit:#x}");
    }
    // The sizes and colours the page sent are the ones it read.
    for &(text_type, _) in CHAT_TEXT_TYPES.iter() {
        assert_eq!(
            after.font_sizes.get(&text_type).unwrap_or(&0),
            before.font_sizes.get(&text_type).unwrap_or(&0),
            "size {text_type:#x}"
        );
        assert_eq!(
            after.font_colors.get(&text_type).unwrap_or(&0),
            before.font_colors.get(&text_type).unwrap_or(&0),
            "colour {text_type:#x}"
        );
    }
    // The page has nothing left to apply, and the chat windows were told to reload.
    assert!(!page.changed());
    assert!(page.take_reload());
}
