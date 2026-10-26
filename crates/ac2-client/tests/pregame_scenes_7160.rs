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
    // `MouseOnObject`: the viewport's tooltip and the race description are real text.
    let root = ui.avatar_selection().unwrap().viewport().unwrap();
    assert_eq!(
        ui.tree().tooltip_text(root).as_deref(),
        Some("Click to choose this avatar.")
    );
    let texts = visible_texts(ui.tree());
    assert!(texts.iter().all(|t| !is_raw_id(t)), "raw id: {texts:?}");
    assert!(
        texts.iter().any(|t| t.len() > 60),
        "the race description is shown: {texts:?}"
    );
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
    // `InitElements`: the slider captions are the script's `StringInfo` labels looked up in
    // their own string tables, never an unresolved `0x...` id.
    let texts = visible_texts(rig.view.ui().unwrap().tree());
    eprintln!("texts: {texts:?}");
    assert!(texts.iter().any(|t| t == "Hairstyle"), "captions are text");
    assert!(
        texts.iter().all(|t| !is_raw_id(t)),
        "an id is printed raw: {texts:?}"
    );
}

/// Every text element's text below the tree's roots.
fn visible_texts(tree: &ac2_ui::UiTree) -> Vec<String> {
    fn walk(tree: &ac2_ui::UiTree, index: usize, out: &mut Vec<String>) {
        if let Some(text) = tree.get(index).and_then(|e| e.text()) {
            out.push(text);
        }
        for &child in tree.children(index) {
            walk(tree, child, out);
        }
    }
    let mut out = Vec::new();
    for &root in tree.roots() {
        walk(tree, root, &mut out);
    }
    out
}

/// The slider captions (`<Name>Label`) inside one page's field.
fn field_labels(rig: &Rig, field: &str) -> Vec<String> {
    let ui = rig.view.ui().unwrap();
    let tree = ui.tree();
    let root = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize(field)));
    let mut out = Vec::new();
    if let Some(root) = root {
        collect_texts(tree, root, &mut out);
    }
    out
}

fn collect_texts(tree: &ac2_ui::UiTree, index: usize, out: &mut Vec<String>) {
    if let Some(text) = tree.get(index).and_then(|e| e.text()) {
        out.push(text);
    }
    for &child in tree.children(index) {
        collect_texts(tree, child, out);
    }
}

/// `0x` and eight hex digits: what `StringInfo::GetString` prints for a missing string.
fn is_raw_id(text: &str) -> bool {
    text.strip_prefix("0x")
        .is_some_and(|h| h.len() >= 8 && h.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Pixels inside `viewport` that differ between two frames.
fn changed_between(a: &[u8], b: &[u8], viewport: &SceneViewport) -> usize {
    let mut changed = 0;
    for y in viewport.y.max(0)..(viewport.y + viewport.height as i32).min(SIZE.1 as i32) {
        for x in viewport.x.max(0)..(viewport.x + viewport.width as i32).min(SIZE.0 as i32) {
            let i = (y as usize * SIZE.0 as usize + x as usize) * 4;
            changed += usize::from((0..3).any(|c| a[i + c].abs_diff(b[i + c]) > 8));
        }
    }
    changed
}

/// `gmAvatarModificationUI::InitCamera` / `UpdateCamera`: the head page frames the head and
/// dragging over `MouseRotationOverField` turns the camera about the avatar; each moves the
/// camera state and changes what the viewport shows.
#[test]
fn the_modification_camera_answers_zoom_and_rotation() {
    let Some(mut rig) = Rig::new(true) else {
        return;
    };
    rig.click("CreateButton");
    while rig.view.mode() != MODE_AVATAR_SELECTION {
        rig.frame();
        assert!(rig.t < 70.0, "no avatar selection");
    }
    let _ = rig.draw();
    let viewport = rig.viewport("AvatarSelectionViewport");
    let picks = rig.picks(&viewport);
    rig.point(picks[0].0);
    rig.frame();
    rig.view.input(&[(LEFT, 1.0, true), (LEFT, 1.0, false)]);
    let started = rig.t;
    while rig.view.mode() != MODE_AVATAR_MODIFICATION {
        rig.frame();
        assert!(rig.t < started + 30.0, "no avatar modification");
    }
    let camera = |rig: &Rig| {
        rig.view
            .ui()
            .unwrap()
            .avatar_modification()
            .unwrap()
            .camera()
    };
    let (before, _) = rig.draw();
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
    assert!(!camera(&rig).zoomed);

    rig.click("HeadButton");
    assert!(camera(&rig).zoomed, "the head page frames the head");
    let (zoomed, _) = rig.draw();
    let zoom_change = changed_between(&before, &zoomed, &viewport);
    assert!(
        zoom_change > 10_000,
        "zooming moved the view: {zoom_change}"
    );
    rig.click("BodyButton");
    assert!(!camera(&rig).zoomed, "the body page frames the body");

    // A drag over the field turns the camera by the pointer's travel over the field's width.
    let (from, rotation) = (rig.element("MouseRotationOverField"), camera(&rig).rotation);
    let (unturned, _) = rig.draw();
    rig.point(from);
    rig.frame();
    rig.view.input(&[(LEFT, 1.0, true)]);
    for step in 1..=8 {
        rig.point((from.0 - step * 12, from.1));
        rig.frame();
    }
    rig.view.input(&[(LEFT, 1.0, false)]);
    let turned_to = camera(&rig).rotation;
    eprintln!("rotation {rotation} -> {turned_to}");
    assert!((turned_to - rotation).abs() > 0.05, "the drag turned it");
    let (turned, _) = rig.draw();
    save("pregame_scene_avatar_rotated.png", &turned);
    let turn_change = changed_between(&unturned, &turned, &viewport);
    assert!(
        turn_change > 5_000,
        "rotating moved the view: {turn_change}"
    );
    assert!(
        scene_pixels(&turned, &unturned, &viewport) > 0 || turn_change > 0,
        "the avatar stays drawn"
    );
    // Each page's captions are its own category's, each once (`InitElements`'s
    // `GetPhysTypeLabel(race, sex, physType)` per slider).
    let pages = [
        ("Head", vec!["Face", "Hairstyle", "Hair Color"]),
        ("Body", vec!["Build", "Height", "Skin Tone"]),
        (
            "Clothing",
            vec!["Shirt Color", "Pants Color", "Boots Color"],
        ),
    ];
    let mut seen = Vec::new();
    for (page, expected) in pages {
        rig.click(&format!("{page}Button"));
        let mut labels = field_labels(&rig, &format!("{page}Field"));
        labels.retain(|t| !t.is_empty() && !["Head", "Body", "Cloth"].contains(&t.as_str()));
        let mut got = labels.clone();
        got.sort();
        let mut want: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        want.sort();
        assert_eq!(got, want, "the {page} page's captions");
        seen.extend(labels);
    }
    let distinct: std::collections::HashSet<_> = seen.iter().collect();
    assert_eq!(distinct.len(), seen.len(), "no caption repeats: {seen:?}");
    // `gmGenerationConfirmationUI`: the name dialog's text is real too.
    rig.click("NextButton");
    rig.frame();
    let texts = visible_texts(rig.view.ui().unwrap().tree());
    eprintln!("confirmation texts: {texts:?}");
    assert!(texts.iter().all(|t| !is_raw_id(t)), "raw id: {texts:?}");
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
