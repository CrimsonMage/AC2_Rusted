//! The navigation panel through the client on the real DATs (`AC2_DAT_DIR`): the panel opened
//! from its button draws the world map (`ac2_ui::world_map`, which replaces the painted macro and
//! detail views) with the player's marker on the player's land cell, and shows the game's time and
//! date on one line. Retail's detail-view formulas (`InitDetailMapFromCellCoords`,
//! `ConvertLandscapeToPixel`, the pan and the lock) stay covered by `ac2_ui::navigation`'s tests.
//! Skipped without a GPU adapter.

#![cfg(feature = "window")]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{Client, DatWorld, FrameInput};
use ac2_gpu::{Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
use legions_server_double::{addr, double, Spawn, State, SERVER};

const STEP: f64 = 1.0 / 64.0;
/// The smallest display retail ran at; the HUD layouts are authored for 1024 x 768.
const SIZE: (u32, u32) = (800, 600);
/// Runs one frame: the views' frameworks answer the messages, the frame is prepared (the panel
/// follows the frame in `prepare`), then the client steps.
fn pump(gpu: &Gpu, view: &mut ClientView, client: &mut Client, server: &Rc<RefCell<State>>) {
    let now = client.frame();
    // The string tables arrive on their worker; `update` takes them when they do.
    view.update(gpu, &now, Duration::ZERO);
    let notices = client.take_script_notices();
    let commands = view.frameworks(client, &now, &[], &notices);
    view.prepare(gpu, &now, SIZE.0 as f32 / SIZE.1 as f32, 0.0);
    server.borrow_mut().elapsed += STEP;
    client
        .use_time(&FrameInput {
            commands,
            ..FrameInput::default()
        })
        .unwrap();
}

/// The client on the town block, entered and settled, its view built with the HUD.
struct Scene {
    gpu: Gpu,
    view: ClientView,
    client: Client,
    server: Rc<RefCell<State>>,
}

fn scene() -> Option<Scene> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let gpu = Gpu::headless().ok()?;
    let store = Arc::new(ac2_db::ResourceStore::new(Arc::clone(&dataset)));
    let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
    let world = Arc::clone(dat_world.world());
    // The town block CB4D, its centre cell, above the highest ground (as the walk test does).
    let block = ac2_core::CellId::new(0xCB4D_0001);
    let top = world
        .load_landblock(block)
        .unwrap()
        .unwrap()
        .height_grid()
        .into_iter()
        .fold(0.0, f32::max);
    let spawn = Spawn {
        cell: 0xCB4D_0025,
        origin: [96.0, 96.0, top + 2.0],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (transport, server) = double(spawn);
    let config = OnlineConfig {
        server: addr(SERVER),
        account: "local".into(),
        credential: Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::First,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    };
    let mut session = OnlineSession::with_transport(config, Box::new(transport)).unwrap();
    let script_data = ac2_script::data::DatScriptData::new(Arc::clone(&dataset), (*store).clone());
    session.set_string_tables(ac2_client::string_table_ids(&script_data));
    session.set_chat_commands(ac2_client::chat_commands(&script_data));
    let dat_world = dat_world.with_script_data(script_data.clone());
    let mut client = Client::new(session, Arc::new(dat_world));
    let wlib = ac2_script::load_wlib(&store, ac2_core::DataId::new(ac2_script::LEGIONS_WLIB_DID))
        .unwrap()
        .expect("the script library");
    let program = Arc::new(ac2_script::vm::Program::from_stream(&wlib.stream));
    script_data.preload_tables();
    script_data.preload_states();
    script_data.preload_skill_states(&program);
    script_data.set_cache_only(true);
    client.attach_scripts(
        ac2_script::host::ScriptHost::new(program, Box::new(script_data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    client.set_load_budget(Duration::from_secs(30));
    let idle = FrameInput::default();
    let mut frames = 0;
    while !client.frame().in_world {
        client.settle_loads(Duration::from_secs(30)).unwrap();
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
        frames += 1;
        assert!(frames < 600);
    }
    server.borrow_mut().send_init_player();
    for _ in 0..128 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }

    let frame = client.frame();
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let mut view = ClientView::new(world, &gpu, COLOR_FORMAT, 1, SIZE);
    view.update(&gpu, &frame, Duration::ZERO);
    view.settle(&gpu, &frame, aspect);
    Some(Scene {
        gpu,
        view,
        client,
        server,
    })
}

#[test]
fn the_panel_draws_the_world_map_with_the_player_on_it() {
    let Some(Scene {
        gpu,
        mut view,
        mut client,
        server,
        ..
    }) = scene()
    else {
        return;
    };
    let frame = client.frame();
    let player = frame.player.as_ref().expect("the player");
    // The player's place in land cells: the landblock's corner plus the origin over 20 m.
    let land = (
        f64::from((player.cell >> 24) & 0xFF) * 8.0 + f64::from(player.origin[0]) / 20.0,
        f64::from((player.cell >> 16) & 0xFF) * 8.0 + f64::from(player.origin[1]) / 20.0,
    );

    // The button shows the panel (`NavigationButton`), and the world map takes its map area.
    assert!(view.navigation_press("NavigationButton", false));
    pump(&gpu, &mut view, &mut client, &server);
    pump(&gpu, &mut view, &mut client, &server);
    let window = view
        .world_map_window()
        .expect("the panel draws the world map");
    let (tree, nav) = view.navigation().expect("the navigation panel");
    assert!(nav.is_shown(tree));
    // The painted views' scroll buttons are hidden: the map pans by dragging.
    for button in nav.scroll_buttons().into_iter().flatten() {
        assert!(!tree.get(button).unwrap().visible());
    }
    // The player's marker is centred on the player's land cell in the map's window.
    let marker = nav.player_marker().expect("the player's marker");
    let element = tree.get(marker).unwrap();
    assert!(element.visible());
    let rect = element.rect();
    let area = tree.screen_box(element.parent_index().expect("the map area"));
    let at = window.pixel_of(area.width(), area.height(), land.0, land.1);
    assert_eq!(
        (rect.x0 + rect.width() / 2, rect.y0 + rect.height() / 2),
        (at.0 as i32, at.1 as i32)
    );
}

#[test]
fn the_panel_shows_the_game_date_and_time_of_the_calendar() {
    let Some(Scene {
        gpu,
        mut view,
        mut client,
        server,
        ..
    }) = scene()
    else {
        return;
    };
    assert!(view.navigation_press("NavigationButton", false));
    // The texts wait for the string tables: pump until the coordinates have a text.
    for _ in 0..400 {
        pump(&gpu, &mut view, &mut client, &server);
        if view.navigation().is_some_and(|(tree, nav)| {
            nav.child(tree, "CoordinatesText")
                .and_then(|e| tree.get(e)?.text())
                .is_some_and(|t| !t.is_empty())
        }) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    // The frame's calendar (the clock's year and day of the season, `GameTime::GetDateTimeString`)
    // is what the panel's date line shows: `ID_DateString` lays the day and the year out as
    // `<season> <day>, <year> <year spec>`. The time line is the time of day's name.
    let date_time = client.frame().date_time.expect("the game clock");
    let (tree, nav) = view.navigation().expect("the navigation panel");
    let text = |name: &str| {
        let element = nav.child(tree, name)?;
        tree.get(element)?.text()
    };
    // The world map puts the time and the date on one line (`time \u{2022} date`).
    let line = text("TimeText").expect("the time text");
    let (time, date) = line
        .split_once(" \u{2022} ")
        .expect("the time and the date");
    assert!(
        date.contains(&format!(" {}, {}", date_time.day, date_time.year)),
        "{date}"
    );
    assert!(!time.is_empty());
    assert_eq!(
        text("CoordinatesText").as_deref().map(str::is_empty),
        Some(false)
    );
}
