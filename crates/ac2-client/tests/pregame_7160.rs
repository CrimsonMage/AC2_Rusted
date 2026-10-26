//! The pregame UI on the real layouts (`AC2_DAT_DIR`) against the Legions 7160 server double:
//! the session waits at the character set, character selection lists it, and the UI's buttons
//! enter the world or delete the character through the session.

#![cfg(feature = "window")]

mod common;

use std::sync::Arc;

use ac2_client::{pregame_event, pregame_requests, Client, FrameInput};
use ac2_ui::pregame::{PregameUi, MODE_CHARACTER_SELECTION, MODE_GAMEPLAY};
use ac2_ui::{UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{CharacterChoice, OnlineSession, SessionEvent};
use common::*;
use legions_server_double::{double, Spawn, PLAYER};

type Server = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

fn connect() -> Option<(Client, Server, PregameUi)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let assets = ac2_ui::pregame::load_pregame(&store).unwrap();
    let ui = PregameUi::new(assets, 1024, 768);
    let (transport, server) = double(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    let mut config = config();
    config.character = CharacterChoice::Wait;
    let session = OnlineSession::with_transport(config, Box::new(transport)).unwrap();
    let mut client = Client::new(session, Arc::new(Synthetic));
    client.set_load_budget(std::time::Duration::from_secs(5));
    Some((client, server, ui))
}

/// One frame: the session's events into the UI, the UI's loop, its requests out.
fn frame(client: &mut Client, server: &Server, ui: &mut PregameUi, t: f64) -> Vec<SessionEvent> {
    let events = step(client, server, &FrameInput::default());
    for event in &events {
        pregame_event(ui, event);
    }
    ui.use_time(t);
    let outcome = pregame_requests(ui, client.session_mut());
    assert!(!outcome.quit);
    events
}

fn click(ui: &mut PregameUi, name: &str) {
    let id = ac2_ui::tokenize(name);
    let tree = ui.tree();
    let element = tree
        .roots()
        .iter()
        .find_map(|&r| {
            (tree.get(r).unwrap().id() == id)
                .then_some(r)
                .or_else(|| tree.child_recursive(r, id))
        })
        .unwrap_or_else(|| panic!("{name}"));
    let b = tree.screen_box(element);
    ui.input(&[
        (UI_POINTER_X, ((b.x0 + b.x1) / 2) as f32, true),
        (UI_POINTER_Y, ((b.y0 + b.y1) / 2) as f32, true),
        (0x4000_0004, 1.0, true),
        (0x4000_0004, 1.0, false),
    ]);
}

#[test]
fn character_selection_enters_the_world_with_the_listed_character() {
    let Some((mut client, server, mut ui)) = connect() else {
        return;
    };
    let mut t = 0.0;
    while ui.mode() != MODE_CHARACTER_SELECTION {
        frame(&mut client, &server, &mut ui, t);
        t += STEP;
        assert!(t < 30.0, "no character selection");
    }
    assert!(!client.frame().in_world, "waits at the character set");
    let selection = ui.character_selection().unwrap();
    assert_eq!(selection.occupants().len(), 1);
    assert_eq!(selection.arms().arms().len(), 5);
    click(&mut ui, "EnterButton");
    while !client.frame().in_world {
        frame(&mut client, &server, &mut ui, t);
        t += STEP;
        assert!(t < 40.0, "no world entry");
    }
    assert_eq!(ui.mode(), MODE_GAMEPLAY);
    assert_eq!(client.frame().player.map(|p| p.id), Some(PLAYER));
}

#[test]
fn delete_is_confirmed_then_sent() {
    let Some((mut client, server, mut ui)) = connect() else {
        return;
    };
    let mut t = 0.0;
    while ui.mode() != MODE_CHARACTER_SELECTION {
        frame(&mut client, &server, &mut ui, t);
        t += STEP;
        assert!(t < 30.0, "no character selection");
    }
    click(&mut ui, "DeleteButton");
    assert!(ui.dialogs().shown().is_some());
    click(&mut ui, "YesButton");
    let mut deleted = false;
    for _ in 0..64 {
        let events = frame(&mut client, &server, &mut ui, t);
        t += STEP;
        deleted |= events.contains(&SessionEvent::CharacterDeleted(PLAYER));
    }
    assert!(deleted);
    assert_eq!(server.borrow().deletions, vec![PLAYER]);
}

/// The pregame screens in the window path, drawn headless: the data loads on its worker, the
/// session's set reaches the screen through the view, and the composed UI is drawn.
/// Writes `target/ac2-snapshots/pregame_view_selection.png`; skipped without a GPU adapter.
#[test]
fn the_pregame_view_draws_character_selection() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = ac2_gpu::Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    // The pregame layouts are authored for 1024 x 768.
    const SIZE: (u32, u32) = (1024, 768);
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut view = ac2_client::pregame_view::PregameView::new(
        Arc::clone(&store),
        &gpu,
        ac2_gpu::COLOR_FORMAT,
        SIZE,
    );
    let (Some((mut client, server, _)), mut t) = (connect(), 0.0) else {
        return;
    };
    while view.mode() != MODE_CHARACTER_SELECTION {
        for event in step(&mut client, &server, &FrameInput::default()) {
            view.event(&event);
        }
        view.update(&gpu, t);
        t += STEP;
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(t < 60.0, "no character selection");
    }
    view.update(&gpu, t);
    assert!(view.shown_layers() > 0);
    let rgba = draw(&gpu, &mut view, SIZE, t, "pregame_view_selection.png");
    // The stone frame and buttons cover a good part of the dark screen.
    let lit = rgba.chunks(4).filter(|p| p[0] > 40 || p[1] > 40).count();
    assert!(
        lit > (SIZE.0 * SIZE.1 / 10) as usize,
        "the screen is drawn: {lit}"
    );
    // A resized window (a deviation from retail): the 1024 x 768 screen is scaled to fit and
    // centred, never cut off or left in a corner.
    let wide = (1600, 900);
    view.resize(wide.0, wide.1);
    view.update(&gpu, t);
    let rgba = draw(
        &gpu,
        &mut view,
        wide,
        t,
        "pregame_view_selection_1600x900.png",
    );
    // 900 / 768 scales the 1024 wide screen to 1200 pixels: 200 black columns on each side.
    let column_lit = |x: u32| {
        (0..wide.1)
            .filter(|&y| {
                let p = &rgba[((y * wide.0 + x) * 4) as usize..][..3];
                p[0] > 40 || p[1] > 40
            })
            .count()
    };
    assert_eq!(column_lit(100), 0, "left bar");
    assert_eq!(column_lit(1500), 0, "right bar");
    assert!(
        column_lit(210) > 0 && column_lit(1390) > 0,
        "the screen's edges"
    );
    let small = (800, 600);
    view.resize(small.0, small.1);
    view.update(&gpu, t);
    let rgba = draw(
        &gpu,
        &mut view,
        small,
        t,
        "pregame_view_selection_800x600.png",
    );
    // The stone pillar at the screen's right edge (x 1000 of 1024) reaches the window instead
    // of being cut off: x 781 at 800 / 1024.
    let lit_rows = (0..small.1)
        .filter(|&y| {
            let p = &rgba[((y * small.0 + 781) * 4) as usize..][..3];
            p[0] > 40 || p[1] > 40
        })
        .count();
    assert!(
        lit_rows > small.1 as usize / 2,
        "the right pillar: {lit_rows}"
    );
}

/// Draws `view` on a `size` target and writes `target/ac2-snapshots/<name>`.
fn draw(
    gpu: &ac2_gpu::Gpu,
    view: &mut ac2_client::pregame_view::PregameView,
    size: (u32, u32),
    t: f64,
    name: &str,
) -> Vec<u8> {
    let target = ac2_gpu::FrameTarget::new(&gpu.device, size.0, size.1);
    view.prepare(gpu, t);
    view.render(gpu, &target.color_view, &target.depth_view);
    let rgba = target.read_rgba(&gpu.device, &gpu.queue);
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).unwrap();
    let file = std::fs::File::create(dir.join(name)).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), size.0, size.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&rgba)
        .unwrap();
    rgba
}
