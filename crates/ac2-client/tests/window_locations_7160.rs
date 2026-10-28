//! The window locations on the player's script VM (`AC2_DAT_DIR`): the HUD's windows that save
//! their location are saved (`gmCWInterface::SetUISaveLocations`) and uploaded
//! (`UploadUISettings`, `WM_Player` 0x32) by `gmGamePlayUI::RecvNotice_SaveLocationsChanged`;
//! read back from the profile, and applied to the windows on the options' reload.

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::NativeObject;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_ui::{build_gameplay_hud, SavedWindow};
use common::*;
use legions_server_double::Spawn;

/// `WM_Player` 0x32, `UploadUISettings` (`ac2_net` `calls.rs`).
const UPLOAD_UI_SETTINGS: u32 = 0x8212_0031;

fn sorted(mut windows: Vec<SavedWindow>) -> Vec<(u32, u32, [u32; 5])> {
    windows.sort_by_key(|w| (w.layout, w.element));
    windows
        .iter()
        .map(|w| {
            let l = w.location;
            (
                w.layout,
                w.element,
                [
                    l.x0.to_bits(),
                    l.y0.to_bits(),
                    l.x1.to_bits(),
                    l.y1.to_bits(),
                    u32::from(l.shown),
                ],
            )
        })
        .collect()
}

/// The player in the world with the script VM attached, after the first frames.
fn in_world() -> Option<(
    ac2_client::Client,
    std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    Arc<ac2_db::ResourceStore>,
)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
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
    assert!(client.take_script_problems().is_empty());
    Some((client, server, ui_store))
}

#[test]
fn moved_windows_are_saved_uploaded_and_applied_on_reload() {
    let Some((mut client, server, ui_store)) = in_world() else {
        return;
    };
    let idle = FrameInput::default();
    // The HUD as the client built it; its windows save their locations.
    let mut hud = build_gameplay_hud(&ui_store, 1024, 768).unwrap();
    let saved = hud.tree.inq_locations();
    assert!(saved.len() > 1, "more than one window saves its location");
    let fresh = client.saved_window_locations().unwrap();
    assert!(fresh.is_empty(), "a new profile has no saved locations");

    // Saved: the profile holds the windows, and the settings went up once.
    let uploads = |server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>| {
        server
            .borrow()
            .script_bytes
            .iter()
            .filter(|(f, _)| *f == UPLOAD_UI_SETTINGS)
            .count()
    };
    let before = uploads(&server);
    client
        .use_time(&FrameInput {
            commands: vec![Command::SaveWindowLocations(saved.clone())],
            ..FrameInput::default()
        })
        .unwrap();
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert!(client.take_script_problems().is_empty());
    assert_eq!(uploads(&server) - before, 1, "one upload per save");
    // The upload carries the save as the stream holds it: the windows in the bucket order
    // `UISaveLocations::AddLocation` builds.
    let expected = ac2_ui::ui_save_locations(&saved).unwrap();
    let sent = server
        .borrow()
        .script_calls
        .iter()
        .rev()
        .find(|c| c.function == UPLOAD_UI_SETTINGS)
        .map(|c| c.arguments.clone())
        .expect("the upload decodes");
    assert!(
        sent.iter().any(
            |a| matches!(a.as_native(), Some(NativeObject::UiSaveLocations(ui)) if *ui == expected)
        ),
        "{sent:?}"
    );
    assert_eq!(
        sorted(client.saved_window_locations().unwrap()),
        sorted(saved.clone()),
        "the profile holds what was saved"
    );

    // A window moves away; the reload applies the saved locations back.
    let (root, target) = {
        let tree = &hud.tree;
        let first = saved[0];
        let root = tree
            .roots()
            .iter()
            .copied()
            .find(|&r| tree.get(r).unwrap().id() == first.element)
            .expect("the window is a root");
        (root, first)
    };
    let placed = hud.tree.screen_box(root);
    hud.tree.move_to(root, 3, 3);
    let reloaded = client.saved_window_locations().unwrap();
    assert!(hud.tree.apply_locations(&reloaded));
    let back = hud.tree.screen_box(root);
    assert_eq!((back.x0, back.y0), (placed.x0, placed.y0), "{target:?}");

    // Two windows that claim one place are not saved: nothing is sent.
    let before = uploads(&server);
    let mut doubled = saved.clone();
    doubled.push(saved[0]);
    client
        .use_time(&FrameInput {
            commands: vec![Command::SaveWindowLocations(doubled)],
            ..FrameInput::default()
        })
        .unwrap();
    for _ in 0..2 {
        step(&mut client, &server, &idle);
    }
    assert_eq!(
        uploads(&server),
        before,
        "a duplicated window saves nothing"
    );
}

/// A chat window's opacity, set by its slider's release (`SetWindowOpacity`), is kept by the
/// profile and the settings go up with it (`UploadUISettings`).
#[test]
fn a_window_opacity_is_saved_on_the_profile_and_uploaded() {
    let Some((mut client, server, _)) = in_world() else {
        return;
    };
    let idle = FrameInput::default();
    let window = ac2_ui::chat_window_id(1);
    assert!(client.chat_settings().unwrap().opacities.is_empty());
    let uploads = |server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>| {
        server
            .borrow()
            .script_bytes
            .iter()
            .filter(|(f, _)| *f == UPLOAD_UI_SETTINGS)
            .count()
    };
    let before = uploads(&server);
    client
        .use_time(&FrameInput {
            commands: vec![Command::SetWindowOpacity {
                window,
                opacity: 0.25,
            }],
            ..FrameInput::default()
        })
        .unwrap();
    for _ in 0..4 {
        step(&mut client, &server, &idle);
    }
    assert!(client.take_script_problems().is_empty());
    assert_eq!(uploads(&server) - before, 1);
    assert_eq!(
        client.chat_settings().unwrap().opacities.get(&window),
        Some(&0.25)
    );
}
