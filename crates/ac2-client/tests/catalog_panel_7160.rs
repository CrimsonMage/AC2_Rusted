//! The storekeeper's catalog (`gmShopCatalogUI`) through the server double with the player's
//! script on the VM (`AC2_DAT_DIR`): the `ViewCatalog` the double sends lists the group's stores
//! in the panel, a press on a row and the enter button send `RequestEnterStore` with the store
//! to the server, and the close sends `RequestLeaveCatalog`. The setup is the store script
//! test's (`store_script_7160.rs`).

mod common;

use std::sync::Arc;

use ac2_client::{catalog_commands, catalog_notice, Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{ScriptRequest, StoreCatalog, StringInfo};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_ui::{
    build_gameplay_hud, tokenize, CatalogUi, ShopSource, UiMessage, UiString, UiStrings, UiTree,
    MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS,
};
use ac2_weenie::GameplayNotice;
use common::*;
use legions_server_double::Spawn;

const VENDOR: u64 = 0x8000_0002_0000_0042;
/// A `StoreGroup` WState in portal.dat (package 0x3858) and one of its stores (sized 5).
const STORE_GROUP: u32 = 0x6F00_3558;
const GROUP_STORE: u32 = 0x6F00_321C;
/// The row's selection press (select) and its double press (select and enter).
const ROW_SELECT: i32 = 0x4000_0004;
const ROW_ENTER: i32 = 0x4000_0007;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// The script with the player attached, after the login's idle frames.
fn with_script() -> Option<(ac2_client::Client, Shared)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    {
        let mut state = server.borrow_mut();
        state.give_starter_items();
        state.send_contents(legions_server_double::PLAYER);
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    Some((client, server))
}

/// Runs the commands for four frames; the notices the frames raised, in order.
fn run(
    client: &mut ac2_client::Client,
    server: &Shared,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let idle = FrameInput::default();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let mut notices = Vec::new();
    for i in 0..4 {
        let frame = if i == 0 { &input } else { &idle };
        for event in step(client, server, frame) {
            if let ac2_weenie::SessionEvent::Notice(notice) = event {
                notices.push(notice);
            }
        }
    }
    notices
}

/// The panel's strings: a wire string as its literal, a UI string as its id.
struct Strings;

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        format!("{:08x}", string.string_id)
    }
}

impl ShopSource for Strings {
    fn render_wire(&self, info: &StringInfo) -> String {
        info.literal.clone().unwrap_or_default()
    }
}

fn release(tree: &UiTree, root: Option<usize>, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: root.and_then(|r| tree.child_recursive(r, tokenize(name))),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    }
}

fn press_row(row: usize, data: i32) -> UiMessage {
    UiMessage {
        element_id: tokenize("SelectionField"),
        element: Some(row),
        message: MSG_MOUSE_PRESS,
        data,
        ..UiMessage::default()
    }
}

/// Routes the notices to the catalog as the frames do.
fn route(catalog: &mut CatalogUi, tree: &mut UiTree, notices: &[GameplayNotice]) {
    for notice in notices {
        catalog_notice(catalog, tree, &Strings, 0.0, notice);
    }
}

#[test]
fn view_catalog_lists_the_group_then_entering_a_store_asks_the_server() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ui_store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&ui_store, 1024, 768).unwrap();
    let mut catalog = hud.catalog.take().expect("the catalog panel");

    server.borrow_mut().open_catalog(
        StoreCatalog {
            group: STORE_GROUP,
            store_sizes: vec![(GROUP_STORE, 5)],
        },
        VENDOR,
    );
    let notices = run(&mut client, &server, Vec::new());
    route(&mut catalog, &mut hud.tree, &notices);
    assert!(catalog.is_shown(), "{notices:?}");
    assert!(catalog.store_ids().contains(&GROUP_STORE));
    let field = catalog.field().expect("the catalog's root");

    let row = catalog.row_of(GROUP_STORE).expect("the group's store");
    catalog.message(&mut hud.tree, &press_row(row, ROW_SELECT), 1.0);
    let enter = release(&hud.tree, Some(field), "EnterStoreButton");
    catalog.message(&mut hud.tree, &enter, 1.0);
    let commands = catalog_commands(catalog.take_requests());
    assert!(
        matches!(commands.as_slice(), [Command::EnterStore(store)] if *store == GROUP_STORE),
        "{commands:?}"
    );
    run(&mut client, &server, commands);
    assert!(
        server
            .borrow()
            .script_requests
            .contains(&ScriptRequest::EnterStore {
                store: GROUP_STORE,
                vendor: VENDOR,
            }),
        "the enter request"
    );

    // A double press enters the store as well.
    catalog.message(&mut hud.tree, &press_row(row, ROW_ENTER), 2.0);
    let commands = catalog_commands(catalog.take_requests());
    assert!(matches!(commands.as_slice(), [Command::EnterStore(store)] if *store == GROUP_STORE));

    // The close sends the leave and hides the catalog.
    let close = release(&hud.tree, Some(field), "CatalogCloseButton");
    catalog.message(&mut hud.tree, &close, 3.0);
    let commands = catalog_commands(catalog.take_requests());
    assert!(matches!(commands.as_slice(), [Command::LeaveCatalog]));
    assert!(!catalog.is_shown());
    run(&mut client, &server, commands);
    assert!(
        server
            .borrow()
            .script_requests
            .contains(&ScriptRequest::LeaveCatalog(VENDOR)),
        "the leave request"
    );
    assert!(client.take_script_problems().is_empty());
}
