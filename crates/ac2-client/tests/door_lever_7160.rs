//! A door and a lever animate when used, against the Legions server double on the real DATs
//! (`AC2_DAT_DIR`). The door (`bd_doorsingle`) is placed shut with `SetMode`; each use is a latent
//! `DoMode` to its other mode, and the body's behaviour table plays the transition (the opening or
//! the closing animation) before the body reaches the new mode. The lever (`dn_lever`) is pulled
//! with a `DoBehavior` of its flip. The uses run through the wlib script VM (the weenies' packages,
//! `DoorTemplate` and `LeverTemplate`), and the bodies come from the DATs, so the animation is the
//! retail one; the rest of the world is the synthetic one.

mod common;

use std::rc::Rc;
use std::sync::Arc;

use ac2_audio::{RecordingSink, SinkCommand};
use ac2_client::{Client, Command, DatWorld, FrameInput, ObjectFrame, WorldData};
use ac2_core::DataId;
use ac2_net::protocol::ScriptRequest;
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use common::*;
use legions_server_double::{
    Spawn, State, DOOR, DOOR_OPEN, DOOR_SHUT, DOOR_VISUAL, LEVER, LEVER_VISUAL,
};

/// The synthetic world, with the door's and the lever's bodies from the DATs and the script data
/// from the DATs' wlib.
struct DoorWorld {
    dat: DatWorld,
    scripts: DatScriptData,
}

impl WorldData for DoorWorld {
    fn load_block(&self, terrain: &mut ac2_physics::Terrain, block: u32) -> Result<bool, String> {
        Synthetic.load_block(terrain, block)
    }

    fn body(&self, visual: Option<u32>) -> Result<ac2_client::BodyData, String> {
        match visual {
            Some(v) if v == DOOR_VISUAL || v == LEVER_VISUAL => self.dat.body(visual),
            _ => Synthetic.body(visual),
        }
    }

    fn game_clock(&self) -> Option<ac2_world::GameClock> {
        None
    }

    fn load_script_data(&self, read: ac2_script::vm::DataRequest) {
        self.scripts.load(read);
    }
}

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

type Server = Rc<std::cell::RefCell<State>>;

/// Steps the client with `commands` on the first frame, then idle frames for `frames` more.
fn play(client: &mut Client, server: &Server, commands: Vec<Command>, frames: usize) {
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    step(client, server, &input);
    for _ in 0..frames {
        step(client, server, &FrameInput::default());
    }
}

/// The object's frame (its animation mode and bones), once it is listed.
fn object(client: &Client, id: u64) -> ObjectFrame {
    client
        .frame()
        .objects
        .into_iter()
        .find(|o| o.id == id)
        .unwrap_or_else(|| panic!("object {id:#x} is not listed"))
}

/// A client in the world with the double's door, lever and ground item placed, the bodies
/// loaded and the script VM attached (the wlib's weenie packages run the uses). Its sounds go to
/// the returned recording.
fn world() -> (Client, Server, RecordingSink) {
    let dataset = ac2_dat::test_support::env_dataset().expect("AC2_DAT_DIR");
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let scripts = DatScriptData::new(dataset.clone(), store);
    scripts.preload_tables();
    scripts.preload_states();
    scripts.preload_skill_states(&program);
    scripts.set_cache_only(true);
    let world = DoorWorld {
        dat: DatWorld::new(Arc::new(ac2_db::ResourceStore::new(dataset.clone()))).unwrap(),
        scripts: scripts.clone(),
    };
    let (mut client, server) = entered_with(spawn(), Arc::new(world));
    client.attach_scripts(
        ScriptHost::new(program, Box::new(scripts)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let sounds = RecordingSink::new();
    client.attach_audio(ac2_client::sound_manager(
        Arc::new(ac2_db::ResourceStore::new(dataset)),
        Box::new(sounds.clone()),
    ));
    {
        let mut state = server.borrow_mut();
        state.send_init_player();
        state.give_starter_items();
        state.spawn_radar_objects();
        state.spawn_world_objects();
    }
    play(&mut client, &server, Vec::new(), 64);
    (client, server, sounds)
}

/// The `Use` requests the double heard for `id`, in order.
fn uses_of(server: &Server, id: u64) -> Vec<u64> {
    server
        .borrow()
        .script_requests
        .iter()
        .filter_map(|request| match request {
            ScriptRequest::Use(desc) if desc.item == id => Some(desc.item),
            _ => None,
        })
        .collect()
}

/// Uses `id`: selects it, then the use key; the motion then plays for `frames` frames.
fn use_object(client: &mut Client, server: &Server, id: u64, frames: usize) {
    play(client, server, vec![Command::Select(Some(id))], 0);
    play(client, server, vec![Command::UseSelection], frames);
}

/// Whether each object has its own script process (the weenie's package, `CWDistributor::Construct`)
/// and no script problem came up.
#[test]
fn the_door_and_the_lever_run_their_scripts() {
    let (mut client, _server, _sounds) = world();
    let host = client.script_host().expect("the scripts are attached");
    assert!(
        host.instance(DOOR).is_some(),
        "the door has a script process"
    );
    assert!(
        host.instance(LEVER).is_some(),
        "the lever has a script process"
    );
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn a_door_opens_and_shuts_when_used() {
    let (mut client, server, _sounds) = world();
    let shut = object(&client, DOOR);
    assert_eq!(shut.visual, Some(DOOR_VISUAL));
    assert_eq!(shut.animated_mode, Some(DOOR_SHUT), "placed shut");
    assert!(!shut.pose.is_empty(), "the door has its bones");

    use_object(&mut client, &server, DOOR, 192);
    assert_eq!(
        uses_of(&server, DOOR).len(),
        1,
        "the use reached the server"
    );
    let open = object(&client, DOOR);
    assert_eq!(open.animated_mode, Some(DOOR_OPEN), "the door reached open");
    assert_ne!(open.pose, shut.pose, "the opening moved the door's bones");

    use_object(&mut client, &server, DOOR, 192);
    assert_eq!(uses_of(&server, DOOR).len(), 2);
    let closed = object(&client, DOOR);
    assert_eq!(closed.animated_mode, Some(DOOR_SHUT), "the door shut again");
    assert_ne!(closed.pose, open.pose, "the closing moved the door's bones");
    assert!(server.borrow().script_errors.is_empty());
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn a_lever_pull_plays_its_flip_and_leaves_the_mode() {
    let (mut client, server, sounds) = world();
    let before = object(&client, LEVER);
    assert_eq!(before.visual, Some(LEVER_VISUAL));
    assert!(!before.pose.is_empty(), "the lever has its bones");
    let sounds_before = sounds.commands().len();

    play(&mut client, &server, vec![Command::Select(Some(LEVER))], 0);
    play(&mut client, &server, vec![Command::UseSelection], 0);
    // The flip's frames: the bones move while the behaviour plays, and it plays out.
    let mut moved = false;
    for _ in 0..96 {
        step(&mut client, &server, &FrameInput::default());
        moved |= object(&client, LEVER).pose != before.pose;
    }
    assert!(moved, "the flip moved the lever's bones");
    // The lever's FX sound (`dn_lever`'s FX table) plays from the lever's position.
    let started = sounds.commands()[sounds_before..]
        .iter()
        .any(|command| matches!(command, SinkCommand::Start { .. }));
    assert!(started, "the flip plays the lever's sound");
    assert_eq!(
        object(&client, LEVER).animated_mode,
        before.animated_mode,
        "a pull leaves the lever's mode"
    );
    assert!(server.borrow().script_errors.is_empty());
    assert!(client.take_script_problems().is_empty());
}
