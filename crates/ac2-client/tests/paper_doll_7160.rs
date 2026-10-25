//! The inventory's paper doll on the real DATs (`AC2_DAT_DIR`) against the Legions 7160 server
//! double: the worn sword in its slot, the player's avatar drawn in the doll's viewport
//! (headless), a press on the avatar's torso selecting the item worn there, and the slot
//! emptying when the sword comes off. Writes `target/ac2-snapshots/client_paper_doll.png`.
//! Skipped without a GPU adapter (the HUD's frameworks need the view).

#![cfg(feature = "window")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{Client, Command, DatWorld, FrameInput};
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_ui::{Modifiers, ToolbarChild, UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
use legions_server_double::{addr, double, Spawn, PACK, SERVER, SWORD};

const STEP: f64 = 1.0 / 64.0;
const SIZE: (u32, u32) = (1024, 768);
const LEFT: u32 = 0x4000_0004;
/// `Torso_InvLoc`.
const TORSO: u32 = 0x2;

fn write_png(name: &str, rgba: &[u8]) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).expect("snapshot directory");
    let file = std::fs::File::create(dir.join(name)).expect("snapshot file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), SIZE.0, SIZE.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(rgba)
        .unwrap();
}

#[test]
fn the_paper_doll_wears_the_sword_and_empties_when_it_comes_off() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let tables_dataset = Arc::clone(&dataset);
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
    let world = Arc::clone(dat_world.world());
    let block = ac2_core::CellId::new(0xCB4D_0001);
    let top = world
        .load_landblock(block)
        .unwrap()
        .unwrap()
        .height_grid()
        .into_iter()
        .fold(0.0, f32::max);
    let (transport, server) = double(Spawn {
        cell: 0xCB4D_0025,
        origin: [96.0, 96.0, top + 2.0],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
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
    let script_data = ac2_script::data::DatScriptData::new(tables_dataset, (*store).clone());
    session.set_string_tables(ac2_client::string_table_ids(&script_data));
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
    let step = |client: &mut Client, input: &FrameInput| {
        let events = client.use_time(input).unwrap();
        server.borrow_mut().elapsed += STEP;
        events
    };
    let mut frames = 0;
    while !client.frame().in_world {
        client.settle_loads(Duration::from_secs(30)).unwrap();
        step(&mut client, &idle);
        frames += 1;
        assert!(frames < 600);
    }
    server.borrow_mut().give_starter_items();
    server.borrow_mut().send_init_player();
    for _ in 0..16 {
        step(&mut client, &idle);
    }
    // The pack opened (its items listed), then the sword worn on the torso.
    let open = FrameInput {
        commands: vec![Command::SetContainerOpen {
            container: PACK,
            open: true,
        }],
        ..FrameInput::default()
    };
    step(&mut client, &open);
    for _ in 0..8 {
        step(&mut client, &idle);
    }
    let equip = FrameInput {
        commands: vec![Command::EquipItem {
            item: SWORD,
            location: TORSO,
        }],
        ..FrameInput::default()
    };
    step(&mut client, &equip);
    let mut events = Vec::new();
    for _ in 0..16 {
        events.extend(step(&mut client, &idle));
    }
    let frame = client.frame();
    assert!(
        frame.inventory.equipped.contains(&(TORSO, SWORD)),
        "{:?}",
        frame.inventory.equipped
    );

    // The HUD's frameworks load once the string tables are in; the doll with them.
    let mut view = ClientView::new(world, &gpu, COLOR_FORMAT, 1, SIZE);
    let mut ready = false;
    for _ in 0..300 {
        view.update(&gpu, &frame, Duration::ZERO);
        let notices = client.take_script_notices();
        view.frameworks(&mut client, &frame, &events, &notices);
        events.clear();
        if view.paper_doll_item(TORSO).is_some() {
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(ready, "the paper doll wears the sword");
    assert_eq!(view.paper_doll_item(TORSO), Some(SWORD));

    // The doll shows with its panel; the avatar is drawn in its viewport, over the panel.
    assert!(
        view.paper_doll_viewport().is_none(),
        "the doll hides with its panel"
    );
    view.show_panel(ToolbarChild::Inventory, true);
    view.update(&gpu, &frame, Duration::ZERO);
    let viewport = view
        .paper_doll_viewport()
        .expect("the doll shows with its panel");
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    view.settle(&gpu, &frame, aspect);
    assert!(view.prepare(&gpu, &frame, aspect, 0.0));
    let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
    view.render(&gpu, &target.color_view, &target.depth_view, SIZE, true);
    let full = target.read_rgba(&gpu.device, &gpu.queue);
    write_png("client_paper_doll.png", &full);
    target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| view.draw(pass));
    let without = target.read_rgba(&gpu.device, &gpu.queue);
    let mut changed = 0;
    for y in viewport.y..viewport.y + viewport.height as i32 {
        for x in viewport.x..viewport.x + viewport.width as i32 {
            let i = (y as usize * SIZE.0 as usize + x as usize) * 4;
            changed += usize::from((0..3).any(|c| full[i + c].abs_diff(without[i + c]) > 8));
        }
    }
    eprintln!("doll pixels: {changed}");
    assert!(changed > 1500, "the avatar is drawn in the doll: {changed}");

    // A point on the avatar's torso, found through the scene's pick.
    let mut torso = None;
    'sweep: for y in (viewport.y..viewport.y + viewport.height as i32).step_by(4) {
        for x in (viewport.x..viewport.x + viewport.width as i32).step_by(4) {
            if view
                .paper_doll_pick((x, y))
                .is_some_and(|m| ac2_ui::mesh_type_to_inv_loc(m) == TORSO)
            {
                torso = Some((x, y));
                break 'sweep;
            }
        }
    }
    let (x, y) = torso.expect("the avatar's torso answers a pick");
    // A press there waits for the pick; on the next loop, the button held, the sword is
    // selected (`FinishPaperDollClick`).
    view.ui_input(
        &[
            (UI_POINTER_X, x as f32, true),
            (UI_POINTER_Y, y as f32, true),
            (LEFT, 1.0, true),
        ],
        Modifiers::default(),
    );
    let notices = client.take_script_notices();
    view.frameworks(&mut client, &frame, &[], &notices);
    let commands = view.frameworks(&mut client, &frame, &[], &[]);
    assert!(
        commands.contains(&Command::Select(Some(SWORD))),
        "{commands:?}"
    );
    view.ui_input(&[(LEFT, 1.0, false)], Modifiers::default());

    // Taken off: the frame's equipment loses it and so does the doll's slot.
    let unequip = FrameInput {
        commands: vec![Command::UnequipItem {
            item: SWORD,
            target_slot: 0,
        }],
        ..FrameInput::default()
    };
    events.clear();
    events.extend(step(&mut client, &unequip));
    for _ in 0..16 {
        events.extend(step(&mut client, &idle));
    }
    let mut gone = false;
    for _ in 0..300 {
        let frame = client.frame();
        view.update(&gpu, &frame, Duration::ZERO);
        let notices = client.take_script_notices();
        view.frameworks(&mut client, &frame, &events, &notices);
        events.clear();
        if view.paper_doll_item(TORSO).is_none() {
            gone = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(gone, "the sword comes off the doll");
}
