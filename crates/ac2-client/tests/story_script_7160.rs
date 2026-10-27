//! The story scene notices and the script's `HandlePlayScenesDone` (`AC2_DAT_DIR`). A portal
//! scene's leave with no story in the slot reports its scenes at once
//! (`gmGamePlayUI::CheckPortalScene`), and the played list goes to the player's script as the IFun
//! `ggmCWInterface::CallIFun_HandlePlayScenesDone` (`0xB40090`). The server call the script makes
//! with the played list is checked byte for byte.

mod common;

use std::sync::Arc;

use ac2_core::DataId;
use ac2_net::protocol::StringInfo;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, Keyframe, SceneInfo, SessionEvent, TRACK_VIDEO};
use common::*;
use legions_server_double::Spawn;

const SCENE: u32 = 7;

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// One playable scene of a single keyframe that shows an image.
fn scene(id: u32) -> SceneInfo {
    SceneInfo {
        id,
        name: StringInfo::literal("Intro"),
        playable: true,
        keyframes: vec![Keyframe {
            scene_id: id,
            frame_num: 1,
            video: 0x0600_0001,
            defined_tracks: TRACK_VIDEO,
            ..Keyframe::default()
        }],
        ..SceneInfo::default()
    }
}

/// A logged-on client with the player's script attached, after the login's idle frames.
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
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    server.borrow_mut().send_init_player();
    let idle = ac2_client::FrameInput::default();
    for _ in 0..30 {
        step(&mut client, &server, &idle);
    }
    Some((client, server))
}

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// The packed arguments of the server call the script makes for the played list (function
/// 0x8213_0013): the `PList<gmSceneInfo>` the script hands back, one scene with one keyframe.
const PLAYED_LIST: &[u8] = &[
    5, 0, 0, 0, 1, 0, 0, 0, 0xe1, 0x66, 0, 0, 0, 1, 0, 0x44, 0, 0, 0, 0, 0xa4, 0, 0xff, 0xff, 1, 0,
    0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 5, 0, 0xac, 0x92, 0x62, 0xe5, 0x3a, 0x76,
    0xa, 0xeb, 0xf6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 7, 0, 0, 0, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0xf0, 0xbf, 1, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
];

/// A portal scene's leave with no story in the slot reports the scene to the script at once
/// (`CheckPortalScene`); the script's `HandlePlayScenesDone` sends the played list to the server.
#[test]
fn a_portal_scene_leave_reports_the_played_scene_to_the_script() {
    let Some((mut client, server)) = with_script() else {
        return;
    };
    let before = server.borrow().script_bytes.len();
    let notice = SessionEvent::Notice(GameplayNotice::PortalScene {
        enter: false,
        scenes: vec![scene(SCENE)],
    });
    client.feed_story_notices(&[notice]);
    let idle = ac2_client::FrameInput::default();
    for _ in 0..5 {
        step(&mut client, &server, &idle);
    }
    assert_eq!(
        server.borrow().script_bytes[before..],
        [(0x8213_0013, PLAYED_LIST.to_vec())]
    );
    assert!(client.take_script_problems().is_empty());
}
