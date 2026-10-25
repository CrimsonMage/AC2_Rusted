//! The pregame screens' 3D scenes in the window path, drawn headless (`AC2_DAT_DIR`, a GPU
//! adapter) against the Legions 7160 server double: character selection's arms and figure,
//! avatar selection's line-up (picked, its selected animation played to the end) and avatar
//! modification's avatar; and the screens without scenes (intro, data patch, credits, controls). Writes
//! `target/ac2-snapshots/pregame_scene_*.png` and `pregame_{intro,data_patch,credits,keyboard}.png`.

#![cfg(feature = "window")]

mod common;

use std::sync::Arc;

use ac2_client::pregame_view::PregameView;
use ac2_client::{Client, FrameInput, ScriptCharGen};
use ac2_core::DataId;
use ac2_gpu::{FrameTarget, Gpu};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_ui::pregame::{
    SceneViewport, MODE_AVATAR_MODIFICATION, MODE_AVATAR_SELECTION, MODE_CHARACTER_SELECTION,
    MODE_CREDITS, MODE_DATA_PATCH, MODE_INTRO, MODE_KEYBOARD,
};
use ac2_ui::{tokenize, UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{CharacterChoice, OnlineSession};
use common::*;
use legions_server_double::{double, Spawn};

type Server = std::rc::Rc<std::cell::RefCell<legions_server_double::State>>;

/// The pregame layouts are authored for 1024 x 768.
const SIZE: (u32, u32) = (1024, 768);
const LEFT: u32 = 0x4000_0004;
/// A keyboard action (not the pointer's).
const KEY: u32 = 0x4000_0100;

struct Rig {
    gpu: Gpu,
    view: PregameView,
    client: Client,
    server: Server,
    t: f64,
}

impl Rig {
    /// The view with its scenes (and the `CharacterGenSystem` script when `chargen`), at
    /// character selection.
    fn new(chargen: bool) -> Option<Self> {
        let mut rig = Self::at_intro(chargen)?;
        while rig.view.mode() != MODE_CHARACTER_SELECTION {
            rig.frame();
            std::thread::sleep(std::time::Duration::from_millis(1));
            assert!(rig.t < 60.0, "no character selection");
        }
        Some(rig)
    }

    /// The view at its first screen, the intro.
    fn at_intro(chargen: bool) -> Option<Self> {
        let dataset = ac2_dat::test_support::env_dataset()?;
        let Ok(gpu) = Gpu::headless() else {
            eprintln!("no GPU adapter; skipping");
            return None;
        };
        let store = Arc::new(ac2_db::ResourceStore::new(dataset.clone()));
        ac2_db::install_ac2_property_types(&store).unwrap();
        let world = ac2_world::Ac2WorldSource::open(Arc::clone(&store))
            .unwrap()
            .unwrap();
        let mut view = PregameView::new(Arc::clone(&store), &gpu, ac2_gpu::COLOR_FORMAT, SIZE)
            .with_scenes(world);
        if chargen {
            let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
                .unwrap()
                .unwrap();
            let program = Arc::new(Program::from_stream(&wlib.stream));
            let data = DatScriptData::new(dataset, (*store).clone());
            data.preload_tables();
            data.preload_states();
            data.set_cache_only(true);
            view.set_char_gen_system(Box::new(ScriptCharGen::new(ScriptHost::new(
                program,
                Box::new(data),
            ))));
        }
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
        let mut rig = Self {
            gpu,
            view,
            client,
            server,
            t: 0.0,
        };
        while rig.view.mode() != MODE_INTRO {
            rig.frame();
            std::thread::sleep(std::time::Duration::from_millis(1));
            assert!(rig.t < 60.0, "no intro");
        }
        Some(rig)
    }

    fn frame(&mut self) {
        for event in step(&mut self.client, &self.server, &FrameInput::default()) {
            self.view.event(&event);
        }
        self.view.update(&self.gpu, self.t);
        self.view.requests(self.client.session_mut());
        self.t += STEP;
    }

    fn element(&self, name: &str) -> (i32, i32) {
        let ui = self.view.ui().unwrap();
        let tree = ui.tree();
        let id = tokenize(name);
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
        ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
    }

    fn point(&mut self, at: (i32, i32)) {
        self.view.input(&[
            (UI_POINTER_X, at.0 as f32, true),
            (UI_POINTER_Y, at.1 as f32, true),
        ]);
    }

    fn click(&mut self, name: &str) {
        let at = self.element(name);
        self.point(at);
        self.view.input(&[(LEFT, 1.0, true), (LEFT, 1.0, false)]);
    }

    fn viewport(&self, name: &str) -> SceneViewport {
        let id = tokenize(name);
        self.view
            .ui()
            .unwrap()
            .viewports()
            .into_iter()
            .find(|v| v.element == id)
            .unwrap_or_else(|| panic!("viewport {name}"))
    }

    /// Every load in, then the frame drawn: (with the scenes, the UI alone).
    fn draw(&mut self) -> (Vec<u8>, Vec<u8>) {
        self.view.settle(&self.gpu, self.t);
        self.draw_now()
    }

    /// The frame drawn as it stands: (with the scenes, the UI alone).
    fn draw_now(&mut self) -> (Vec<u8>, Vec<u8>) {
        self.view.prepare(&self.gpu, self.t);
        let target = FrameTarget::new(&self.gpu.device, SIZE.0, SIZE.1);
        self.view
            .render(&self.gpu, &target.color_view, &target.depth_view);
        let full = target.read_rgba(&self.gpu.device, &self.gpu.queue);
        let view = &self.view;
        target.render(&self.gpu.device, &self.gpu.queue, [0.0; 3], |pass| {
            view.draw(pass)
        });
        let ui = target.read_rgba(&self.gpu.device, &self.gpu.queue);
        (full, ui)
    }

    /// The pick answer at each point of a grid over `viewport`: the first hit of each answer.
    fn picks(&self, viewport: &SceneViewport) -> Vec<((i32, i32), usize)> {
        let scenes = self.view.scenes().unwrap();
        let ui = self.view.ui().unwrap();
        let mut found: Vec<((i32, i32), usize)> = Vec::new();
        for y in (viewport.y..viewport.y + viewport.height as i32).step_by(6) {
            for x in (viewport.x..viewport.x + viewport.width as i32).step_by(6) {
                if let Some(answer) = scenes.pick(ui, (x, y)) {
                    if !found.iter().any(|(_, a)| *a == answer) {
                        found.push(((x, y), answer));
                    }
                }
            }
        }
        found
    }
}

fn save(name: &str, rgba: &[u8]) {
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/ac2-snapshots");
    std::fs::create_dir_all(&dir).unwrap();
    let file = std::fs::File::create(dir.join(name)).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), SIZE.0, SIZE.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(rgba)
        .unwrap();
}

/// Pixels inside `viewport` where the scene changed the UI's frame.
fn scene_pixels(full: &[u8], ui: &[u8], viewport: &SceneViewport) -> usize {
    let mut changed = 0;
    for y in viewport.y.max(0)..(viewport.y + viewport.height as i32).min(SIZE.1 as i32) {
        for x in viewport.x.max(0)..(viewport.x + viewport.width as i32).min(SIZE.0 as i32) {
            let i = (y as usize * SIZE.0 as usize + x as usize) * 4;
            let differs = (0..3).any(|c| full[i + c].abs_diff(ui[i + c]) > 8);
            changed += usize::from(differs);
        }
    }
    changed
}

#[test]
fn character_selection_shows_the_arms_and_answers_a_pick_on_the_figure() {
    let Some(mut rig) = Rig::new(false) else {
        return;
    };
    let (full, ui) = rig.draw();
    save("pregame_scene_selection.png", &full);
    let objects = rig.view.scene_objects();
    // The pillar, five arms and the character's figure, and the portals.
    assert!(objects >= 7, "{objects} scene objects drawn");
    let viewport = rig.viewport("Viewport");
    let changed = scene_pixels(&full, &ui, &viewport);
    assert!(
        changed > 2000,
        "the scene is drawn in its viewport: {changed}"
    );
    // The figure on the front arm answers with its slot; the corner over nothing with none.
    let picks = rig.picks(&viewport);
    eprintln!("picks: {picks:?}");
    assert_eq!(picks.iter().map(|(_, a)| *a).collect::<Vec<_>>(), vec![0]);
    let scenes = rig.view.scenes().unwrap();
    let ui = rig.view.ui().unwrap();
    assert_eq!(scenes.pick(ui, (viewport.x + 1, viewport.y + 1)), None);
    // A press there selects it through the UI's pick.
    rig.point(picks[0].0);
    rig.view.input(&[(LEFT, 1.0, true), (LEFT, 1.0, false)]);
    rig.frame();
    let ui = rig.view.ui().unwrap();
    assert!(ui.wants_pick().is_none(), "the pick was answered");
}

#[test]
fn character_generation_shows_the_avatars_and_the_chosen_one() {
    let Some(mut rig) = Rig::new(true) else {
        return;
    };
    rig.click("CreateButton");
    while rig.view.mode() != MODE_AVATAR_SELECTION {
        rig.frame();
        assert!(rig.t < 70.0, "no avatar selection");
    }
    let (full, ui) = rig.draw();
    save("pregame_scene_avatar_selection.png", &full);
    let viewport = rig.viewport("AvatarSelectionViewport");
    let changed = scene_pixels(&full, &ui, &viewport);
    assert!(changed > 20_000, "the line-up is drawn: {changed}");
    let avatars = rig
        .view
        .ui()
        .unwrap()
        .avatar_selection()
        .unwrap()
        .avatars()
        .len();
    let picks = rig.picks(&viewport);
    eprintln!("avatar picks: {picks:?} of {avatars}");
    assert!(picks.len() >= avatars / 2, "the avatars answer picks");

    // The pointer over an avatar names it and presents it; a press plays its selected
    // animation, whose end opens avatar modification.
    let (at, avatar) = picks[0];
    rig.point(at);
    let presented = rig.t + 3.0;
    while rig.t < presented {
        rig.frame();
    }
    let ui = rig.view.ui().unwrap();
    assert_eq!(ui.avatar_selection().unwrap().under_mouse(), Some(avatar));
    rig.view.input(&[(LEFT, 1.0, true), (LEFT, 1.0, false)]);
    assert_eq!(rig.view.ui().unwrap().selected_animation(), Some(avatar));
    let started = rig.t;
    while rig.view.mode() != MODE_AVATAR_MODIFICATION {
        rig.frame();
        assert!(rig.t < started + 30.0, "the selected animation never ended");
    }
    eprintln!("the selected animation took {:.2} s", rig.t - started);
    assert!(rig.t - started > 0.25, "the animation played");
    let camera = rig
        .view
        .ui()
        .unwrap()
        .avatar_modification()
        .unwrap()
        .camera();
    eprintln!("modification camera: {camera:?}");
    let (full, ui) = rig.draw();
    save("pregame_scene_avatar_modification.png", &full);
    let race = rig
        .view
        .ui()
        .unwrap()
        .avatar_modification()
        .unwrap()
        .viewport();
    let viewport = rig
        .view
        .ui()
        .unwrap()
        .viewports()
        .into_iter()
        .find(|v| Some(v.root) == race)
        .expect("the race's viewport");
    let changed = scene_pixels(&full, &ui, &viewport);
    assert!(changed > 20_000, "the avatar's scene is drawn: {changed}");
}

/// Lit pixels of a frame.
fn lit(rgba: &[u8]) -> usize {
    rgba.chunks(4).filter(|p| p[0] > 40 || p[1] > 40).count()
}

#[test]
fn the_intro_data_patch_credits_and_controls_screens_draw() {
    let Some(mut rig) = Rig::at_intro(false) else {
        return;
    };
    rig.frame();
    let (intro, _) = rig.draw();
    save("pregame_intro.png", &intro);
    // The Turbine logo, dark grey on black.
    let logo = intro.chunks(4).filter(|p| p[0] > 8).count();
    assert!(logo > 20_000, "the intro picture shows: {logo}");
    // A click ends the intro; the data patch screen shows for the frame it takes the set.
    rig.click("IntroField");
    rig.frame();
    assert_eq!(rig.view.mode(), MODE_DATA_PATCH);
    let (patch, _) = rig.draw_now();
    save("pregame_data_patch.png", &patch);
    assert!(lit(&patch) > (SIZE.0 * SIZE.1 / 10) as usize);
    while rig.view.mode() != MODE_CHARACTER_SELECTION {
        rig.frame();
        assert!(rig.t < 60.0, "no character selection");
    }
    rig.click("CreditsButton");
    while rig.view.mode() != MODE_CREDITS {
        rig.frame();
        assert!(rig.t < 70.0, "no credits");
    }
    let shown = rig.t + 9.0;
    while rig.t < shown {
        rig.frame();
    }
    let (credits, _) = rig.draw();
    save("pregame_credits.png", &credits);
    assert!(lit(&credits) > 2000, "credits text and a picture");
    // A key press ends the credits; the options button opens the controls screen.
    rig.view.input(&[(KEY, 1.0, true), (KEY, 1.0, false)]);
    while rig.view.mode() != MODE_CHARACTER_SELECTION {
        rig.frame();
        assert!(rig.t < 100.0, "the credits never closed");
    }
    rig.click("OptionsButton");
    while rig.view.mode() != MODE_KEYBOARD {
        rig.frame();
        assert!(rig.t < 110.0, "no controls screen");
    }
    let (controls, _) = rig.draw();
    save("pregame_keyboard.png", &controls);
    assert!(lit(&controls) > (SIZE.0 * SIZE.1 / 10) as usize);
}
