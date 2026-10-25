//! The navigation panel's detail view through the client on the real DATs (`AC2_DAT_DIR`): the
//! panel opened from its button, the player's marker pressed in the macro view to zoom to the
//! character, the lock to the character and the pan. The expected numbers are written out here
//! from the retail formulas: the 128-cell window around the player (`InitDetailMapFromCellCoords`,
//! clamped to the land), the corners' tiles (the block `cell >> 3` in the region's detail map),
//! the detail frame's pixel (`ConvertLandscapeToPixel`, scale 250 over the 128 cells) and the
//! marker's centre (`CenterMarker`). Skipped without a GPU adapter.

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
/// The land's cells (255 blocks of 8) and the detail window's side.
const LAND: i32 = 2040;
const WINDOW: i32 = 128;
/// `ConvertLandscapeToPixel`'s detail scale (`0xa53500`, 250.0f).
const DETAIL_SCALE: f64 = 250.0;

/// The window's origin: the cell minus 64 on each axis, kept on the land.
fn window(cell: (i32, i32)) -> (i32, i32) {
    let origin = |v: i32| (v - WINDOW / 2).clamp(0, LAND - WINDOW);
    (origin(cell.0), origin(cell.1))
}

/// `ConvertLandscapeToPixel` in the detail view: `(lX - MapX) * 250 / 128` and `250 - (lY -
/// MapY) * 250 / 128`, each stored as a float.
fn detail_pixel(origin: (i32, i32), cell: (i32, i32)) -> (f32, f32) {
    let x = (f64::from(cell.0 - origin.0) * DETAIL_SCALE / f64::from(WINDOW)) as f32;
    let y_up = (f64::from(cell.1 - origin.1) * DETAIL_SCALE / f64::from(WINDOW)) as f32;
    (x, (DETAIL_SCALE - f64::from(y_up)) as f32)
}

/// A block's row in the detail map from the tile rectangles (the structure, not the lookup).
fn tile_containing(rows: &[ac2_world::DetailMapRow], block: (i32, i32)) -> Option<u32> {
    rows.iter()
        .find(|r| {
            (r.x0 as i32..(r.x0 + r.width) as i32).contains(&block.0)
                && (r.y0 as i32..(r.y0 + r.height) as i32).contains(&block.1)
        })
        .map(|r| r.value)
}

/// The corners' blocks of a window: top left, top right, bottom left, bottom right in the
/// `InitDetailMapFromCellCoords` order (top is `MapY + 128`).
fn corner_blocks(origin: (i32, i32)) -> [(i32, i32); 4] {
    let (x, y) = origin;
    [
        (x, y + WINDOW),
        (x + WINDOW, y + WINDOW),
        (x, y),
        (x + WINDOW, y),
    ]
    .map(|(cx, cy)| (cx >> 3, cy >> 3))
}

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
    store: Arc<ac2_db::ResourceStore>,
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
        store,
    })
}

#[test]
fn the_panel_zooms_to_the_character_and_pans_the_detail_view() {
    let Some(Scene {
        gpu,
        mut view,
        mut client,
        server,
        store,
    }) = scene()
    else {
        return;
    };
    let frame = client.frame();
    let player_cell = frame.player.as_ref().expect("the player").cell;
    let land =
        ac2_physics::landdefs::gid_to_lcoord(player_cell).expect("the player is on the land");

    // The detail map the region names, read from the DAT for the expectations.
    let table = ac2_world::ac2_region_detail_map_table(&store)
        .unwrap()
        .expect("the region's detail map");
    let detail = ac2_world::ac2_detail_map(&store, table)
        .unwrap()
        .expect("the detail map");

    // The button shows the panel (`NavigationButton`).
    assert!(view.navigation_press("NavigationButton", false));
    pump(&gpu, &mut view, &mut client, &server);
    {
        let (tree, nav) = view.navigation().expect("the navigation panel");
        assert!(nav.is_shown(tree));
        assert!(!nav.is_detail(), "the panel opens in the macro view");
    }

    // The player's marker pressed in the macro view zooms to the character (`ZoomToCharPos`).
    assert!(view.navigation_map_press("PlayerMarker", 0, (0, 0)));
    pump(&gpu, &mut view, &mut client, &server);
    let origin = window(land);
    {
        let (tree, nav) = view.navigation().unwrap();
        assert!(nav.is_detail());
        assert_eq!(nav.map_frame().origin, origin);
        let expected = corner_blocks(origin).map(|block| tile_containing(detail.rows(), block));
        for (tile, want) in expected.into_iter().enumerate() {
            assert_eq!(
                nav.tile_row(tile).map(|r| r.value),
                want,
                "tile {tile} shows the row of its corner"
            );
        }
        assert!(
            expected.iter().any(Option::is_some),
            "the character's window has tiles"
        );
        // The marker's centre at the detail pixel: its rectangle starts half its size before.
        let marker = nav.player_marker().unwrap();
        let rect = tree.get(marker).unwrap().rect();
        let (size_w, size_h) = (rect.width(), rect.height());
        let (px, py) = detail_pixel(origin, land);
        assert_eq!(
            (rect.x0, rect.y0),
            (
                px as i32 - (f64::from(size_w) * 0.5) as i32,
                py as i32 - (f64::from(size_h) * 0.5) as i32
            )
        );
        // The scroll buttons show in the detail view without the lock.
        for button in nav.scroll_buttons().into_iter().flatten() {
            assert!(tree.get(button).unwrap().visible());
        }
    }

    // Unlocked, the right button pans nine cells; with Shift held, three.
    assert!(view.navigation_press("NavigateRightButton", false));
    pump(&gpu, &mut view, &mut client, &server);
    assert_eq!(
        view.navigation().unwrap().1.map_frame().origin,
        (origin.0 + 9, origin.1)
    );
    assert!(view.navigation_press("NavigateDownButton", true));
    pump(&gpu, &mut view, &mut client, &server);
    assert_eq!(
        view.navigation().unwrap().1.map_frame().origin,
        (origin.0 + 9, origin.1 - 3)
    );

    // The lock: on, the view goes back to the character and stays there; the pan does nothing
    // and the scroll buttons hide.
    view.navigation_lock(true);
    pump(&gpu, &mut view, &mut client, &server);
    assert!(view.navigation().unwrap().1.is_locked());
    assert_eq!(view.navigation().unwrap().1.map_frame().origin, origin);
    assert!(view.navigation_press("NavigateLeftButton", false));
    pump(&gpu, &mut view, &mut client, &server);
    assert_eq!(view.navigation().unwrap().1.map_frame().origin, origin);
    {
        let (tree, nav) = view.navigation().unwrap();
        for button in nav.scroll_buttons().into_iter().flatten() {
            assert!(
                !tree.get(button).unwrap().visible(),
                "scroll buttons hide when locked"
            );
        }
    }

    // Unlocked again: the buttons show and the pan moves the window.
    view.navigation_lock(false);
    pump(&gpu, &mut view, &mut client, &server);
    assert!(!view.navigation().unwrap().1.is_locked());
    assert!(view.navigation_press("NavigateLeftButton", false));
    pump(&gpu, &mut view, &mut client, &server);
    assert_eq!(
        view.navigation().unwrap().1.map_frame().origin,
        (origin.0 - 9, origin.1)
    );

    // The parent's press with the switch's data code goes back to the macro view on the next
    // frame (the Legions layout has no macro button).
    assert!(view.navigation_map_press("MapParentField", 0x4000_0005, (0, 0)));
    pump(&gpu, &mut view, &mut client, &server);
    let (_, nav) = view.navigation().unwrap();
    assert!(!nav.is_detail());
    assert_eq!(nav.map_frame().origin, (321, 195));
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
    let date = text("DateText").expect("the date text");
    let time = text("TimeText").expect("the time text");
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
