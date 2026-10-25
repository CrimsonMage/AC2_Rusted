//! Character creation on the real layouts and the real `CharacterGenSystem` script
//! (`AC2_DAT_DIR`) against the Legions 7160 server double: create opens avatar selection with
//! every race and sex, picking one opens avatar modification with its sliders, and the naming
//! dialog sends the character and enters the world with it.

mod common;

use std::sync::Arc;

use ac2_client::{pregame_event, pregame_requests, Client, FrameInput, ScriptCharGen};
use ac2_core::DataId;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_ui::pregame::{
    phys_type_value, PregameUi, MODE_AVATAR_MODIFICATION, MODE_AVATAR_SELECTION,
    MODE_CHARACTER_SELECTION, MODE_GAMEPLAY,
};
use ac2_ui::{tokenize, UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{CharacterChoice, OnlineSession};
use common::*;
use legions_server_double::{double, Spawn, CREATED};

type Server = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

fn frame(client: &mut Client, server: &Server, ui: &mut PregameUi, t: &mut f64) {
    for event in step(client, server, &FrameInput::default()) {
        pregame_event(ui, &event);
    }
    ui.use_time(*t);
    assert!(!pregame_requests(ui, client.session_mut()).quit);
    *t += STEP;
}

fn find(ui: &PregameUi, name: &str) -> Option<usize> {
    let id = tokenize(name);
    let tree = ui.tree();
    tree.roots().iter().find_map(|&r| {
        (tree.get(r).unwrap().id() == id)
            .then_some(r)
            .or_else(|| tree.child_recursive(r, id))
    })
}

fn point_at(ui: &mut PregameUi, element: usize) {
    let b = ui.tree().screen_box(element);
    ui.input(&[
        (UI_POINTER_X, ((b.x0 + b.x1) / 2) as f32, true),
        (UI_POINTER_Y, ((b.y0 + b.y1) / 2) as f32, true),
    ]);
}

fn click(ui: &mut PregameUi, name: &str) {
    let element = find(ui, name).unwrap_or_else(|| panic!("{name}"));
    point_at(ui, element);
    ui.input(&[(0x4000_0004, 1.0, true), (0x4000_0004, 1.0, false)]);
}

/// The composed screen over black (`target/ac2-snapshots/<name>`).
fn snapshot(ui: &mut PregameUi, name: &str) {
    let (w, h) = (1024_usize, 768_usize);
    let mut rgba = vec![0_u8; w * h * 4];
    for px in rgba.chunks_mut(4) {
        px[3] = 255;
    }
    for surface in ui.compose() {
        for sy in 0..surface.height as usize {
            for sx in 0..surface.width as usize {
                let (x, y) = (surface.x + sx as i32, surface.y + sy as i32);
                if x < 0 || y < 0 || x as usize >= w || y as usize >= h {
                    continue;
                }
                let src = &surface.rgba[(sy * surface.width as usize + sx) * 4..][..4];
                let dst = &mut rgba[(y as usize * w + x as usize) * 4..][..4];
                let a = u32::from(src[3]);
                for c in 0..3 {
                    dst[c] = ((u32::from(src[c]) * a + u32::from(dst[c]) * (255 - a)) / 255) as u8;
                }
            }
        }
    }
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).unwrap();
    let file = std::fs::File::create(dir.join(name)).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&rgba)
        .unwrap();
}

#[test]
fn a_new_character_is_made_named_and_sent() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset.clone()));
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, (*store).clone());
    // As the client runs it: everything resident, then cache-only (the frame thread never
    // reads).
    data.preload_tables();
    data.preload_states();
    data.set_cache_only(true);
    let mut ui = PregameUi::new(ac2_ui::pregame::load_pregame(&store).unwrap(), 1024, 768);
    ui.set_char_gen_system(Box::new(ScriptCharGen::new(ScriptHost::new(
        program,
        Box::new(data),
    ))));
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
    let mut t = 0.0;
    while ui.mode() != MODE_CHARACTER_SELECTION {
        frame(&mut client, &server, &mut ui, &mut t);
        assert!(t < 30.0, "no character selection");
    }

    // Create: avatar selection with every race and sex.
    click(&mut ui, "CreateButton");
    frame(&mut client, &server, &mut ui, &mut t);
    assert_eq!(ui.mode(), MODE_AVATAR_SELECTION);
    let avatars = ui.avatar_selection().unwrap().avatars().to_vec();
    eprintln!(
        "avatars: {:?}",
        avatars
            .iter()
            .map(|a| (a.race, a.sex, a.phys_obj, a.location))
            .collect::<Vec<_>>()
    );
    assert!(avatars.len() >= 6, "{} avatars", avatars.len());
    assert!(avatars
        .iter()
        .all(|a| a.phys_obj != 0 && a.location.is_some()));
    // The pointer over the scene picks; the scene answers with the first avatar, whose race and
    // sex name the description; a press plays its animation, whose end opens modification.
    let viewport = find(&ui, "AvatarSelectionViewport").unwrap();
    point_at(&mut ui, viewport);
    assert!(ui.wants_pick().is_some());
    ui.finish_pick(Some(0));
    let desc = find(&ui, "RaceDescField").unwrap();
    assert_eq!(ui.tree().get(desc).unwrap().state(), avatars[0].state);
    ui.input(&[(0x4000_0004, 1.0, true), (0x4000_0004, 1.0, false)]);
    assert_eq!(ui.selected_animation(), Some(0));
    snapshot(&mut ui, "pregame_avatar_selection.png");
    ui.finish_animation();
    frame(&mut client, &server, &mut ui, &mut t);
    assert_eq!(ui.mode(), MODE_AVATAR_MODIFICATION);
    let current = ui.data().current().cloned().unwrap();
    assert_eq!(
        (current.race, current.sex),
        (avatars[0].race, avatars[0].sex)
    );
    let modification = ui.avatar_modification().unwrap();
    let offered = modification.offered();
    eprintln!("offered physique types: {offered:?}");
    assert!(!offered.is_empty());
    snapshot(&mut ui, "pregame_avatar_modification.png");

    // The body part shows first: step its first slider on with the right button.
    let names = [
        (2, "Height"),
        (1, "Build"),
        (4, "SkinTone"),
        (8, "SkinDetail"),
        (128, "Special"),
    ];
    let (phys_type, name) = names
        .into_iter()
        .find(|(t, _)| offered.contains(t))
        .expect("a body slider");
    let before = phys_type_value(&current, phys_type);
    click(&mut ui, &format!("{name}RightButton"));
    let after = phys_type_value(ui.data().current().unwrap(), phys_type);
    eprintln!("{name}: {before} -> {after}");
    let appearance = ui.avatar_modification().unwrap().appearance();
    click(&mut ui, "RandomizeAllButton");
    assert!(ui.avatar_modification().unwrap().appearance() > appearance);

    // Next: name it (the filter drops the digit) and enter with it (the Legions dialog has
    // only enter and cancel).
    click(&mut ui, "NextButton");
    assert!(ui.avatar_modification().unwrap().dialog().is_some());
    click(&mut ui, "NameEntry");
    let typed: Vec<(u16, u32)> = "Zed9a".encode_utf16().map(|c| (c, 0)).collect();
    ui.text_input(&[], &typed, false);
    snapshot(&mut ui, "pregame_naming.png");
    click(&mut ui, "EnterButton");
    while !client.frame().in_world {
        frame(&mut client, &server, &mut ui, &mut t);
        assert!(t < 60.0, "no world entry");
    }
    assert_eq!(ui.mode(), MODE_GAMEPLAY);
    let sent = server.borrow().char_gen_results.clone();
    assert_eq!(sent.len(), 1);
    let (_, result) = &sent[0];
    assert_eq!(result.name, "Zeda");
    assert_eq!((result.race, result.sex), (avatars[0].race, avatars[0].sex));
    assert!(!result.physical.is_empty());
    assert_ne!(result.entity_did, 0);
    // The created character was added to the set the UI keeps.
    let set = ui.data().characters().unwrap();
    assert!(set
        .characters
        .iter()
        .any(|c| c.id == CREATED && c.name == "Zeda"));
}
