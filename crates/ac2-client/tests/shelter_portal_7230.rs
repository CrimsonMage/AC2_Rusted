//! The Shelter's pink portal in the human start dungeon (block 0x0299), drawn headless on the
//! real DATs (`AC2_DAT_DIR`) against the server double: AC2e places retail's captured
//! `wsl_genericportal_pink` (entity 0x47001451, visual `it_portal_shard.vd` 0x1F00031E) at
//! (101.0, 240.1, -43.52) in cell 0x02990120, on its dais. The player stands west of it facing
//! east. Writes `target/ac2-snapshots/shelter_portal_*.png`; skipped without a GPU adapter.

#![cfg(feature = "window")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{Client, ClientFrame, DatWorld, FrameInput};
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
use legions_server_double::{addr, double, Spawn, SERVER};

const STEP: f64 = 1.0 / 64.0;
const SIZE: (u32, u32) = (800, 600);
const PORTAL: u64 = 0x8140_0299_13DA_1C16;
const PORTAL_VISUAL: u32 = 0x1F00_031E;
const CELL: u32 = 0x0299_0120;
const PORTAL_AT: [f32; 3] = [101.0, 240.1, -43.52];

type Shared = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

fn write_png(name: &str, rgba: &[u8]) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).expect("snapshot directory");
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

fn step(client: &mut Client, server: &Shared) {
    client.settle_loads(Duration::from_secs(30)).unwrap();
    client.use_time(&FrameInput::default()).unwrap();
    server.borrow_mut().elapsed += STEP;
}

struct Scene {
    gpu: Gpu,
    view: ClientView,
    target: FrameTarget,
    time: f64,
}

impl Scene {
    /// Hands the view the FX particles, lays `seconds` of frames out and draws the last.
    fn draw(&mut self, client: &mut Client, server: &Shared, seconds: f64, name: &str) {
        let aspect = SIZE.0 as f32 / SIZE.1 as f32;
        for _ in 0..(seconds / STEP).round() as usize {
            step(client, server);
            let starts = client.take_object_particle_starts();
            let stops = client.take_object_particle_stops();
            self.view.set_object_particles(&starts, &stops);
            let frame: ClientFrame = client.frame();
            self.view.update(&self.gpu, &frame, Duration::ZERO);
            self.view.settle(&self.gpu, &frame, aspect);
            self.time += STEP;
            assert!(self.view.prepare(&self.gpu, &frame, aspect, self.time));
        }
        self.view.render(
            &self.gpu,
            &self.target.color_view,
            &self.target.depth_view,
            true,
        );
        let rgba = self.target.read_rgba(&self.gpu.device, &self.gpu.queue);
        write_png(name, &rgba);
    }
}

#[test]
fn the_shelter_pink_portal_is_drawn() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(Arc::clone(&dataset)));
    let dat_world = DatWorld::new(Arc::clone(&store)).unwrap();
    let world = Arc::clone(dat_world.world());
    // West of the portal, facing east (a quarter turn about z).
    let spawn = Spawn {
        cell: CELL,
        origin: [PORTAL_AT[0] - 6.0, PORTAL_AT[1], PORTAL_AT[2] + 1.0],
        rotation: [
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            0.0,
            -std::f32::consts::FRAC_1_SQRT_2,
        ],
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
    let session = OnlineSession::with_transport(config, Box::new(transport)).unwrap();
    let mut client = Client::new(session, Arc::new(dat_world));
    client.set_load_budget(Duration::from_secs(30));
    client.attach_audio(ac2_client::sound_manager(
        Arc::clone(&store),
        Box::new(ac2_audio::NullSink),
    ));
    let mut frames = 0;
    while !client.frame().in_world {
        step(&mut client, &server);
        frames += 1;
        assert!(frames < 600);
    }
    let mut waited = 0;
    while client.frame().portal.in_portal_space {
        step(&mut client, &server);
        waited += 1;
        assert!(waited < 2000, "the login's portal space did not run out");
    }
    let mut scene = Scene {
        view: ClientView::new(world, &gpu, COLOR_FORMAT, 1, SIZE),
        target: FrameTarget::new(&gpu.device, SIZE.0, SIZE.1),
        gpu,
        time: 0.0,
    };
    scene.draw(&mut client, &server, 3.0, "shelter_portal_before.png");
    let before = scene.view.object_particles();
    eprintln!("before the portal: {before:?}");
    server
        .borrow_mut()
        .spawn_creature(PORTAL, PORTAL_VISUAL, CELL, PORTAL_AT);
    scene.draw(&mut client, &server, 3.0, "shelter_portal_0.png");
    let (systems, alive) = scene.view.object_particles();
    eprintln!(
        "portal particles: {systems} systems, {alive} alive; player {:?}",
        client.frame().player.map(|p| (p.cell, p.origin))
    );
    let placed = client
        .frame()
        .objects
        .iter()
        .find(|o| o.id == PORTAL)
        .map(|o| (o.cell, o.origin, o.in_world));
    eprintln!("portal placed {placed:x?} (server {CELL:08x} {PORTAL_AT:?})");
    scene.draw(&mut client, &server, 0.5, "shelter_portal_1.png");
    for i in 0..8 {
        scene.draw(
            &mut client,
            &server,
            0.125,
            &format!("shelter_portal_seq{i}.png"),
        );
    }
    assert!(systems > 0 && alive > 0, "the portal's particles draw");
}
