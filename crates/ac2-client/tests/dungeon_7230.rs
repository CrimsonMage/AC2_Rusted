//! The start dungeon (cell 0x0299011B, 7230 DATs) with the player placed where the Pyrealium server put
//! the character: the drawn floor under the player is the floor the physics stands it on, the
//! viewer's cell draws without the landscape, and the dungeon's static entities start their
//! particle systems. With `AC2_SHOT_DIR` set it also renders views of the room (quality 5 and 1)
//! into that directory. Skipped without `AC2_DAT_DIR`; the renders also need a GPU adapter.

#![cfg(feature = "window")]

mod common;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{Client, DatWorld, FrameInput};
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
use common::script_rules::{new_host, ScriptRules};
use legions_server_double::{addr, double_with_rules, Spawn, SERVER};

const STEP: f64 = 1.0 / 64.0;
const SIZE: (u32, u32) = (1280, 720);
const CELL: u32 = 0x0299_011B;
const ORIGIN: [f32; 3] = [49.525, 230.409, -43.616];

fn write_png(dir: &std::path::Path, name: &str, rgba: &[u8]) {
    std::fs::create_dir_all(dir).expect("snapshot directory");
    let file = std::fs::File::create(dir.join(name)).expect("png file");
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
fn the_start_dungeon_at_the_players_spot() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let out = std::env::var_os("AC2_SHOT_DIR").map(PathBuf::from);
    let tables_dataset = Arc::clone(&dataset);
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
    let world = Arc::clone(dat_world.world());
    let spawn = Spawn {
        cell: CELL,
        origin: ORIGIN,
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (transport, server) = double_with_rules(spawn, Box::new(ScriptRules(new_host(&store))));
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
    session.set_chat_commands(ac2_client::chat_commands(&script_data));
    let dat_world = dat_world.with_script_data(script_data.clone());
    let mut client = Client::new(session, Arc::new(dat_world));
    let local_dir = std::env::temp_dir().join(format!("ac2-dungeon-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&local_dir);
    client.set_window_positions_dir(Some(local_dir.clone()));
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
    // The Pyrealium training-world placements (content/training-world.tsv): a lifestone and a
    // Shelter Guard in this block.
    server.borrow_mut().spawn_creature(
        0x8140_0000_0003_0001,
        0x1f00_002d,
        0x0299_0109,
        [55.060_577, 240.199, -44.606_205],
    );
    server.borrow_mut().spawn_creature(
        0x8140_0000_0003_0040,
        0x1f00_0d74,
        0x0299_0109,
        [52.159_115, 242.034_38, -44.450_88],
    );
    for _ in 0..128 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }
    let mut spins = 0;
    while client.frame().portal.in_portal_space && spins < 4000 {
        client.settle_loads(Duration::from_secs(30)).unwrap();
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
        spins += 1;
    }
    let frame = client.frame();
    let player = frame.player.as_ref().unwrap();
    assert_eq!(player.cell, CELL);
    let footing = client.footing(player.id).expect("the player's body");
    assert!(
        footing.surface_found && footing.on_ground && footing.height.abs() < 0.05,
        "{footing:?}"
    );
    let me = ac2_math::Vec3::new(
        320.0 + player.origin[0],
        24480.0 + player.origin[1],
        player.origin[2],
    );

    // The drawn floor: the cell mesh's triangles under the player (`CEnvCell::GetNearbyGeometry`
    // reads the same vertices the cell draws), against the physics polygon of the same cell mesh.
    let block = ac2_core::CellId::new(0x0299_0001);
    let settings = ac2_render::object::RenderSettings::default();
    let interiors =
        ac2_render::scene::load_block_interiors(&world, &store, block, &settings, 0.0).unwrap();
    assert_eq!(interiors.cull.cell_at(me), Some(CELL));
    let floor_under = |points: &[ac2_math::Vec3]| floor_under_at(points, me);
    let drawn = ac2_render::cell_geometry::nearby_geometry(&interiors.cull.cells, CELL, me, 0.05)
        .expect("the cell has a mesh");
    let drawn_floor = floor_under(&drawn.origins)
        .into_iter()
        .filter(|z| *z <= me.z + 0.05)
        .fold(f32::MIN, f32::max);
    let cell = world
        .env_cell(ac2_core::CellId::new(CELL))
        .unwrap()
        .unwrap();
    let structure = world.cell_struct(cell.cell_mesh).unwrap().unwrap();
    let frame_of_cell = world.env_cell_world_frame(&cell).to_mat4();
    let mut physics_points = Vec::new();
    for polygon in structure.polygons.iter() {
        let pts: Vec<_> = polygon
            .vertex_ids
            .iter()
            .map(|&i| frame_of_cell.transform_point3(structure.positions[usize::from(i)]))
            .collect();
        for k in 1..pts.len().saturating_sub(1) {
            physics_points.extend([pts[0], pts[k], pts[k + 1]]);
        }
    }
    let physics_floor = floor_under(&physics_points)
        .into_iter()
        .filter(|z| *z <= me.z + 0.05)
        .fold(f32::MIN, f32::max);
    assert!(
        (drawn_floor - physics_floor).abs() < 0.01,
        "the drawn floor {drawn_floor} is the physics floor {physics_floor}"
    );
    // The body's origin rests on the floor it is drawn over (a slope lifts the body's sphere
    // centre, so the origin sits within the sphere radius above the drawn triangle).
    assert!(
        (0.0..0.3).contains(&(me.z - drawn_floor)),
        "feet at {} over a drawn floor at {drawn_floor}",
        me.z
    );

    // Walking: the body keeps to the drawn floor on level ground.
    {
        use ac2_ui::{Controls, InputEvent};
        let mut controls = Controls::load(&store).unwrap().expect("controls");
        controls.event(
            InputEvent::Key {
                scan: 0x11,
                pressed: true,
            },
            0,
            0.0,
        );
        let view_world = |frame: &ac2_client::ClientFrame| {
            let p = frame.player.as_ref().unwrap();
            ac2_math::Vec3::new(320.0 + p.origin[0], 24480.0 + p.origin[1], p.origin[2])
        };
        for step in 0..(3 * 64) {
            client
                .use_time(&ac2_client::frame_input(controls.frame()))
                .unwrap();
            server.borrow_mut().elapsed += STEP;
            if step > 64 && step % 32 == 0 {
                let frame = client.frame();
                let at = view_world(&frame);
                let near = ac2_render::cell_geometry::nearby_geometry(
                    &interiors.cull.cells,
                    frame.player.as_ref().unwrap().cell,
                    at,
                    0.05,
                )
                .unwrap();
                let floor = floor_under_at(&near.origins, at)
                    .into_iter()
                    .filter(|z| *z <= at.z + 0.05)
                    .fold(f32::MIN, f32::max);
                assert!(
                    (at.z - floor).abs() < 0.05,
                    "walking at {at:?}: the floor drawn under the feet is at {floor}"
                );
            }
        }
    }

    // The dungeon's static entities start their particle systems (`CEnvCell::InitStaticObjects`).
    let sources = ac2_world::ac2_block_particle_sources(&world, block).unwrap();
    assert!(
        !sources.is_empty(),
        "the dungeon's torches start particle systems"
    );

    let Some(out) = out else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping the renders");
        return;
    };
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let frame = client.frame();
    let p = frame.player.as_ref().unwrap();
    // (name, eye offset from the player, look-at offset from the player)
    let shots: [(&str, [f32; 3], [f32; 3]); 6] = [
        ("default", [0.0; 3], [0.0; 3]),
        ("side_se", [6.0, -6.0, 1.5], [0.0, 0.0, 0.8]),
        ("far_south", [0.0, -30.0, 6.0], [0.0, 0.0, 0.0]),
        ("fire", [0.0, -1.5, 1.6], [1.47, 5.4, 1.9]),
        ("look_up", [0.0, -3.0, 1.6], [0.0, 3.0, 40.0]),
        ("lifestone", [-2.0, -3.0, 0.0], [5.5, 10.0, -1.0]),
    ];
    for level in [5u32, 1] {
        let mut prefs = ac2_client::ClientPrefs::load(
            local_dir.join(format!("prefs{level}.cfg")),
            Arc::clone(&store),
        )
        .unwrap();
        prefs.apply_startup_quality(level);
        let mut view = ClientView::new(Arc::clone(&world), &gpu, COLOR_FORMAT, 1, SIZE);
        view.set_scene_options(prefs.scene_options());
        let target = FrameTarget::new(&gpu.device, SIZE.0, SIZE.1);
        for (name, eye, look) in shots {
            let mut frame = frame.clone();
            if name != "default" {
                let at = |o: [f32; 3]| [p.origin[0] + o[0], p.origin[1] + o[1], p.origin[2] + o[2]];
                let camera = frame.camera.as_mut().unwrap();
                camera.origin = at(eye);
                camera.target = at(look);
            }
            view.update(&gpu, &frame, Duration::ZERO);
            view.settle(&gpu, &frame, aspect);
            assert!(view.prepare(&gpu, &frame, aspect, 0.0));
            for step in 0..180_u32 {
                if step % 10 == 0 {
                    view.settle(&gpu, &frame, aspect);
                }
                view.update(&gpu, &frame, Duration::from_millis(16));
                // The client time advances a frame at a time, as the running client's does: the
                // particle systems age with it.
                view.prepare(&gpu, &frame, aspect, 0.016 * f64::from(step + 1));
                view.render(&gpu, &target.color_view, &target.depth_view, true);
            }
            let rgba = target.read_rgba(&gpu.device, &gpu.queue);
            if name == "fire" {
                assert_torch_flame(&rgba, level);
            }
            write_png(&out, &format!("{name}_q{level}.png"), &rgba);
        }
    }
}

/// The height of each triangle of `points` (three corners each) at the xy of `at`, for the
/// triangles that cover it.
fn floor_under_at(points: &[ac2_math::Vec3], at: ac2_math::Vec3) -> Vec<f32> {
    points
        .chunks_exact(3)
        .filter_map(|tri| {
            let (a, b, c) = (tri[0], tri[1], tri[2]);
            let d = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
            if d.abs() < 1e-6 {
                return None;
            }
            let l1 = ((b.y - c.y) * (at.x - c.x) + (c.x - b.x) * (at.y - c.y)) / d;
            let l2 = ((c.y - a.y) * (at.x - c.x) + (a.x - c.x) * (at.y - c.y)) / d;
            let l3 = 1.0 - l1 - l2;
            (l1 >= -1e-4 && l2 >= -1e-4 && l3 >= -1e-4).then_some(l1 * a.z + l2 * b.z + l3 * c.z)
        })
        .collect()
}

/// The torch's flame (`PSDesc` 0x39000031 and its siblings: additive `SRCALPHA`, `ONE` quads that
/// write depth) draws at every quality level as warm flame colour over the wall, and not as the
/// pale blue the sky shows where the quads' black texels punched a hole through the cell mesh
/// drawn after them. (`WorldRender::Render` ends with `PSManager::RenderParticleSystems`.)
fn assert_torch_flame(rgba: &[u8], level: u32) {
    let (mut flame, mut sky) = (0, 0);
    for y in 195..315_usize {
        for x in 515..625_usize {
            let i = (y * SIZE.0 as usize + x) * 4;
            let (r, g, b) = (
                i32::from(rgba[i]),
                i32::from(rgba[i + 1]),
                i32::from(rgba[i + 2]),
            );
            if r > 140 && r > b + 60 && g > b {
                flame += 1;
            }
            if b > r + 25 {
                sky += 1;
            }
        }
    }
    assert!(
        flame > 20,
        "quality {level}: the flame draws ({flame} warm pixels)"
    );
    assert_eq!(sky, 0, "quality {level}: no sky-blue hole around the flame");
}
