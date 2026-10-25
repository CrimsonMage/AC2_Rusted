//! Character creation on the real layouts and the real `CharacterGenSystem` script
//! (`AC2_DAT_DIR`) against the Legions 7160 server double: create opens avatar selection with
//! every race and sex, picking one opens avatar modification with its sliders, and the naming
//! dialog sends the character and enters the world with it. The server keeps created characters
//! in a file: a later session lists the character with the appearance it was made with, enters
//! the world with it, and refuses a name in use with the retail error.

mod common;

use std::path::Path;
use std::sync::Arc;

use ac2_client::{pregame_event, pregame_requests, Client, FrameInput, ScriptCharGen};
use ac2_core::DataId;
use ac2_net::protocol::{CharGenResult, ClientEvent, Opcode};
use ac2_script::data::DatScriptData;
use ac2_script::host::{chargen as host_chargen, ScriptHost};
use ac2_script::native::CharGenResult as NativeCgr;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_ui::pregame::{
    load_pregame, phys_type_value, PregameUi, MODE_AVATAR_MODIFICATION, MODE_AVATAR_SELECTION,
    MODE_CHARACTER_SELECTION, MODE_GAMEPLAY,
};
use ac2_ui::{tokenize, UI_POINTER_X, UI_POINTER_Y};
use ac2_weenie::{Appearance, CharacterChoice, OnlineSession};
use common::*;
use legions_server_double::{double, double_with, CharGenRules, Spawn, CREATED, PLAYER};

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

fn click_element(ui: &mut PregameUi, element: usize) {
    point_at(ui, element);
    ui.input(&[(0x4000_0004, 1.0, true), (0x4000_0004, 1.0, false)]);
}

fn click(ui: &mut PregameUi, name: &str) {
    let element = find(ui, name).unwrap_or_else(|| panic!("{name}"));
    click_element(ui, element);
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

/// The script host the client's screens and the server's rules run on.
fn script_host() -> Option<(ScriptHost, Arc<ac2_db::ResourceStore>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
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
    Some((ScriptHost::new(program, Box::new(data)), store))
}

/// The native `gmCharGenResult` of the wire result (`gmCharGenResult::operator=`).
fn native_of(result: &CharGenResult) -> NativeCgr {
    let mut out = NativeCgr {
        name: result.name.encode_utf16().collect(),
        entity_did: result.entity_did,
        race: result.race,
        sex: result.sex,
        class: result.class,
        implement: result.implement,
        start_area: result.start_area,
        ..NativeCgr::default()
    };
    for &(phys_type, value) in &result.physical {
        out.set_phys_type_value(phys_type, value);
    }
    out
}

/// The server's rules on the `CharacterGenSystem` script: `IsValid` and the applied appearance.
struct ScriptRules(ScriptHost);

impl CharGenRules for ScriptRules {
    fn validate(&mut self, result: &CharGenResult) -> u32 {
        host_chargen::is_valid(self.0.vm_mut(), &native_of(result)).unwrap_or(0)
    }

    fn appearance(&mut self, result: &CharGenResult) -> Option<Appearance> {
        host_chargen::applied_appearance(
            self.0.vm_mut(),
            Appearance::default(),
            &native_of(result),
            u32::MAX,
        )
        .map(|(_, appearance)| appearance)
    }
}

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// A session: the double, the client and the pregame screens, at character selection.
struct Session {
    client: Client,
    server: Server,
    ui: PregameUi,
    t: f64,
}

/// Starts a session: `keep` is the server's character file and its rules (none: a double as it
/// has always been, with no rules and no file). The screens run on `host`, the client's own
/// script host; returns once the character set is shown.
fn start(
    host: ScriptHost,
    store: &Arc<ac2_db::ResourceStore>,
    keep: Option<(&Path, Box<dyn CharGenRules>)>,
) -> Session {
    let mut ui = PregameUi::new(load_pregame(store).unwrap(), 1024, 768);
    ui.set_char_gen_system(Box::new(ScriptCharGen::new(host)));
    let (transport, server) = match keep {
        Some((path, rules)) => {
            let (transport, server) = double_with(spawn(), path, Some(rules)).unwrap();
            (transport, server)
        }
        None => double(spawn()),
    };
    let mut config = config();
    config.character = CharacterChoice::Wait;
    let session = OnlineSession::with_transport(config, Box::new(transport)).unwrap();
    let mut client = Client::new(session, Arc::new(Synthetic));
    client.set_load_budget(std::time::Duration::from_secs(5));
    let mut session = Session {
        client,
        server,
        ui,
        t: 0.0,
    };
    while session.ui.mode() != MODE_CHARACTER_SELECTION {
        frame(
            &mut session.client,
            &session.server,
            &mut session.ui,
            &mut session.t,
        );
        assert!(
            session.t < 30.0,
            "no character selection (mode {:#x})",
            session.ui.mode()
        );
    }
    session
}

/// What a creation made: the appearance the screens applied, and the race and sex.
struct Created {
    appearance: Appearance,
    race: u32,
    sex: u32,
}

impl Session {
    fn frame(&mut self) {
        frame(&mut self.client, &self.server, &mut self.ui, &mut self.t);
    }

    /// The creation pages: avatar selection (the first race and sex), avatar modification (a
    /// slider stepped, then everything randomized), and the naming dialog, which sends the
    /// character. With `enter` it enters the world with it; otherwise it waits for the answer
    /// (the creation screens stay with the error line).
    fn create(&mut self, name: &str, enter: bool) -> Created {
        click(&mut self.ui, "CreateButton");
        self.frame();
        assert_eq!(self.ui.mode(), MODE_AVATAR_SELECTION);
        let viewport = find(&self.ui, "AvatarSelectionViewport").unwrap();
        point_at(&mut self.ui, viewport);
        assert!(self.ui.wants_pick().is_some());
        self.ui.finish_pick(Some(0));
        self.ui
            .input(&[(0x4000_0004, 1.0, true), (0x4000_0004, 1.0, false)]);
        self.ui.finish_animation();
        self.frame();
        assert_eq!(self.ui.mode(), MODE_AVATAR_MODIFICATION);
        let current = self.ui.data().current().cloned().unwrap();
        let applied = self.ui.avatar_modification().unwrap().applied().cloned();
        assert!(applied.is_some(), "the avatar shows its applied appearance");

        // The body part shows first: step its first slider on with the right button.
        let offered = self.ui.avatar_modification().unwrap().offered();
        let names = [
            (2, "Height"),
            (1, "Build"),
            (4, "SkinTone"),
            (8, "SkinDetail"),
            (128, "Special"),
        ];
        let (phys_type, slider) = names
            .into_iter()
            .find(|(t, _)| offered.contains(t))
            .expect("a body slider");
        let before = phys_type_value(&current, phys_type);
        click(&mut self.ui, &format!("{slider}RightButton"));
        let after = phys_type_value(self.ui.data().current().unwrap(), phys_type);
        assert!(after > before, "{slider} steps up: {before} -> {after}");
        click(&mut self.ui, "RandomizeAllButton");
        let appearance = self
            .ui
            .avatar_modification()
            .unwrap()
            .applied()
            .cloned()
            .expect("the randomized appearance");

        // Next: name it (the filter drops the digit) and send it.
        click(&mut self.ui, "NextButton");
        assert!(self.ui.avatar_modification().unwrap().dialog().is_some());
        click(&mut self.ui, "NameEntry");
        let typed: Vec<(u16, u32)> = name.encode_utf16().map(|c| (c, 0)).collect();
        self.ui.text_input(&[], &typed, false);
        click(
            &mut self.ui,
            if enter { "EnterButton" } else { "CreateButton" },
        );
        Created {
            appearance,
            race: current.race,
            sex: current.sex,
        }
    }

    /// The result the screens sent, from the server's side.
    fn sent(&self) -> Vec<(String, CharGenResult)> {
        self.server.borrow().char_gen_results.clone()
    }

    fn error_text(&self) -> String {
        let element = find(&self.ui, "GenerationConfirmationErrorMsg").unwrap();
        self.ui
            .tree()
            .get(element)
            .unwrap()
            .text()
            .unwrap_or_default()
    }

    /// The login's characters, by name, with their visuals.
    fn listed(&self) -> Vec<(String, u64, ac2_net::protocol::VisualDesc)> {
        self.ui
            .data()
            .characters()
            .unwrap()
            .characters
            .iter()
            .map(|c| (c.name.clone(), c.id, c.visual.clone()))
            .collect()
    }
}

/// The creation flow as the client has always run it: one session, every page, the character
/// sent and entered, and the server's answer.
#[test]
fn a_new_character_is_made_named_and_sent() {
    let Some((host, store)) = script_host() else {
        return;
    };
    let mut s = start(host, &store, None);
    let created = s.create("Zed9a", true);
    while !s.client.frame().in_world {
        s.frame();
        assert!(s.t < 60.0, "no world entry");
    }
    assert_eq!(s.ui.mode(), MODE_GAMEPLAY);
    let sent = s.sent();
    assert_eq!(sent.len(), 1);
    let (_, result) = &sent[0];
    assert_eq!(result.name, "Zeda");
    assert_eq!((result.race, result.sex), (created.race, created.sex));
    assert!(!result.physical.is_empty());
    assert_ne!(result.entity_did, 0);
    // The created character was added to the set the UI keeps.
    assert!(s
        .listed()
        .iter()
        .any(|(name, id, _)| name == "Zeda" && *id == CREATED));
}

#[test]
fn the_create_pages_show_every_avatar_and_the_sliders_move_the_appearance() {
    let Some((host, store)) = script_host() else {
        return;
    };
    let mut s = start(host, &store, None);
    click(&mut s.ui, "CreateButton");
    s.frame();
    let avatars = s.ui.avatar_selection().unwrap().avatars().to_vec();
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
    // Each avatar is shown in its race's default appearance (`InitEntities`).
    assert!(avatars.iter().all(|a| a.appearance.is_some()));
    // The applied appearance of the default result: a height slider step changes its scale.
    let viewport = find(&s.ui, "AvatarSelectionViewport").unwrap();
    point_at(&mut s.ui, viewport);
    s.ui.finish_pick(Some(0));
    s.ui.input(&[(0x4000_0004, 1.0, true), (0x4000_0004, 1.0, false)]);
    s.ui.finish_animation();
    s.frame();
    assert_eq!(s.ui.mode(), MODE_AVATAR_MODIFICATION);
    let before =
        s.ui.avatar_modification()
            .unwrap()
            .applied()
            .cloned()
            .unwrap();
    assert_eq!(before.scale, avatars[0].appearance.clone().unwrap().scale);
    let offered = s.ui.avatar_modification().unwrap().offered();
    assert!(offered.contains(&2), "a height slider: {offered:?}");
    click(&mut s.ui, "HeightRightButton");
    let after =
        s.ui.avatar_modification()
            .unwrap()
            .applied()
            .cloned()
            .unwrap();
    assert_ne!(before.scale, after.scale, "the height changes the scale");
    snapshot(&mut s.ui, "pregame_avatar_modification.png");
}

/// The server's rules with the script's appearance taken away: a listing that recomputed the
/// appearance would show none.
struct NoAppearance(ScriptRules);

impl CharGenRules for NoAppearance {
    fn validate(&mut self, result: &CharGenResult) -> u32 {
        self.0.validate(result)
    }

    fn appearance(&mut self, _: &CharGenResult) -> Option<Appearance> {
        None
    }
}

/// Characters are kept with their own ids and their appearance, and a later login lists and
/// enters each as stored: Zeda (created and entered in one server), Ada (created in a second,
/// entered there), and then Zeda entered by a third server whose rules would not make her
/// appearance again.
#[test]
fn created_characters_keep_their_ids_and_appearances_across_sessions() {
    let Some((host, store)) = script_host() else {
        return;
    };
    let dir = std::env::temp_dir().join(format!("ac2-chargen-session-{}", std::process::id()));
    let file = dir.join("characters.bin");
    let _ = std::fs::remove_file(&file);

    // Server one: Zeda is made and entered. The server keeps her in its file.
    let mut first = start(
        host,
        &store,
        Some((file.as_path(), Box::new(ScriptRules(new_host(&store))))),
    );
    let zeda = first.create("Zeda", true);
    while !first.client.frame().in_world {
        first.frame();
        assert!(first.t < 60.0, "no world entry");
    }
    let zeda_entity = first.sent()[0].1.entity_did;
    drop(first);
    assert!(file.exists(), "the character was written");

    // Server two: the list carries Zeda with her stored id and appearance. Ada is made and
    // entered: her id is the next one.
    let (host, _) = script_host().unwrap();
    let mut second = start(
        host,
        &store,
        Some((file.as_path(), Box::new(ScriptRules(new_host(&store))))),
    );
    let listed = second.listed();
    assert_eq!(listed.len(), 2, "{listed:?}");
    assert_eq!((listed[0].1, listed[1].1), (PLAYER, CREATED));
    assert_eq!(listed[1].0, "Zeda");
    let ada = second.create("Ada", true);
    while !second.client.frame().in_world {
        second.frame();
        assert!(second.t < 60.0, "no world entry");
    }
    let ada_entity = second.sent()[0].1.entity_did;
    let ada_player = second.client.frame().player.expect("the player");
    assert_eq!(ada_player.id, CREATED + 1, "Ada enters with her own id");
    assert_eq!(
        ada_player.appearance,
        Appearance {
            parent: ada_entity,
            ..ada.appearance.clone()
        },
        "Ada enters with the appearance she was made with"
    );
    drop(second);

    // Server three, with rules that make no appearance: the list still shows what was stored,
    // with the ids in order, and Zeda enters with hers.
    let (host, _) = script_host().unwrap();
    let mut third = start(
        host,
        &store,
        Some((
            file.as_path(),
            Box::new(NoAppearance(ScriptRules(new_host(&store)))),
        )),
    );
    let listed = third.listed();
    let ids: Vec<u64> = listed.iter().map(|(_, id, _)| *id).collect();
    assert_eq!(ids, vec![PLAYER, CREATED, CREATED + 1]);
    let zeda_listed = &listed[1];
    assert_eq!(
        Appearance::from_delta(&zeda_listed.2),
        Appearance {
            parent: zeda_entity,
            ..zeda.appearance.clone()
        },
        "the saved appearance is listed"
    );
    assert_eq!(
        Appearance::from_delta(&listed[2].2),
        Appearance {
            parent: ada_entity,
            ..ada.appearance.clone()
        },
        "Ada's saved appearance is listed"
    );
    let picked = find(&third.ui, "CharacterListBox").unwrap();
    let items = third.ui.tree().list_items(picked).to_vec();
    click_element(&mut third.ui, items[1]);
    click(&mut third.ui, "EnterButton");
    while !third.client.frame().in_world {
        third.frame();
        assert!(third.t < 60.0, "no world entry");
    }
    let player = third.client.frame().player.expect("the player");
    assert_eq!(player.id, CREATED, "Zeda enters with her id");
    assert_eq!(
        player.appearance,
        Appearance {
            parent: zeda_entity,
            ..zeda.appearance.clone()
        },
        "the world shows Zeda's saved appearance"
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// A `CharacterGenSystem` host of its own, for the server's rules.
fn new_host(store: &Arc<ac2_db::ResourceStore>) -> ScriptHost {
    let wlib = load_wlib(store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let dataset = ac2_dat::test_support::env_dataset().unwrap();
    let data = DatScriptData::new(dataset, (**store).clone());
    data.preload_tables();
    data.preload_states();
    data.set_cache_only(true);
    ScriptHost::new(program, Box::new(data))
}

#[test]
fn a_name_in_use_is_refused_with_the_retail_error() {
    let Some((host, store)) = script_host() else {
        return;
    };
    let dir = std::env::temp_dir().join(format!("ac2-chargen-taken-{}", std::process::id()));
    let file = dir.join("characters.bin");
    let _ = std::fs::remove_file(&file);
    let mut first = start(
        host,
        &store,
        Some((file.as_path(), Box::new(ScriptRules(new_host(&store))))),
    );
    first.create("Zeda", true);
    while !first.client.frame().in_world {
        first.frame();
        assert!(first.t < 60.0, "no world entry");
    }
    drop(first);

    let (host, _) = script_host().unwrap();
    let mut second = start(
        host,
        &store,
        Some((file.as_path(), Box::new(ScriptRules(new_host(&store))))),
    );
    // The Legions naming dialog has enter (and cancel) only: a refused name stays in it.
    second.create("zeda", true);
    let mut t = 0;
    while second.error_text().is_empty() {
        second.frame();
        t += 1;
        assert!(t < 200, "no refusal");
    }
    let expected = load_pregame(&store)
        .unwrap()
        .strings
        .get("ID_Character_Err_NameInUse");
    assert_eq!(second.error_text(), expected);
    assert_eq!(second.ui.mode(), MODE_AVATAR_MODIFICATION, "still naming");
    assert!(!second.client.frame().in_world);
    // The server kept one character: the refusal added none.
    assert_eq!(second.server.borrow().characters.records.len(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_sent_character_is_the_packed_result_byte_for_byte() {
    let Some((host, store)) = script_host() else {
        return;
    };
    let mut s = start(host, &store, None);
    s.create("Zeda", true);
    while !s.client.frame().in_world {
        s.frame();
        assert!(s.t < 60.0, "no world entry");
    }
    let server = s.server.borrow();
    let (account, result) = server.char_gen_results[0].clone();
    let (_, _, sent) = server
        .blobs
        .iter()
        .find(|(_, _, data)| data[..4] == Opcode::CHAR_GEN_RESULT.to_le_bytes())
        .expect("the CharGenResult blob")
        .clone();
    // The bytes are the event's encoding (opcode, account, the result's `Pack`).
    let (queue, encoded) = ClientEvent::CharGenResult {
        account: account.clone(),
        result: result.clone(),
    }
    .encode();
    assert_eq!(queue, 8);
    assert_eq!(sent, encoded);
    // The layout: the opcode, the account's packed string, the name, six words, then the
    // table's count with its bucket index and the (key, value) pairs in table order.
    let mut account_bytes = Vec::new();
    ac2_net::pack_string(&mut account_bytes, 0, account.as_bytes());
    let mut at = 4;
    assert_eq!(&sent[at..at + account_bytes.len()], &account_bytes[..]);
    at += account_bytes.len();
    let mut name_bytes = Vec::new();
    ac2_net::pack_wide_string(&mut name_bytes, 0, &result.name);
    assert_eq!(&sent[at..at + name_bytes.len()], &name_bytes[..]);
    at += name_bytes.len();
    let word = |i: usize| u32::from_le_bytes(sent[at + 4 * i..at + 4 * i + 4].try_into().unwrap());
    assert_eq!(
        [word(0), word(1), word(2), word(3), word(4), word(5)],
        [
            result.entity_did,
            result.start_area,
            result.race,
            result.sex,
            result.class,
            result.implement
        ]
    );
    assert_eq!(word(6), result.physical.len() as u32 | 2 << 24);
    assert_eq!(sent.len(), 4 + account_bytes.len() + result.pack_size());
    assert_eq!(
        result.pack_size(),
        name_bytes.len() + 0x18 + 4 + 8 * result.physical.len()
    );
}

/// The script's rules, except that a name with "Bad" in it is refused with the weenie's banned
/// name error (200, `ID_Character_Err_NameBanned` on the confirmation screen).
struct RefusingRules(ScriptRules);

impl CharGenRules for RefusingRules {
    fn validate(&mut self, result: &CharGenResult) -> u32 {
        if result.name.contains("Bad") {
            200
        } else {
            self.0.validate(result)
        }
    }

    fn appearance(&mut self, result: &CharGenResult) -> Option<Appearance> {
        self.0.appearance(result)
    }
}

#[test]
fn a_refused_result_shows_the_weenies_error_and_keeps_nothing() {
    let Some((host, store)) = script_host() else {
        return;
    };
    let dir = std::env::temp_dir().join(format!("ac2-chargen-refused-{}", std::process::id()));
    let file = dir.join("characters.bin");
    let _ = std::fs::remove_file(&file);
    let rules = RefusingRules(ScriptRules(new_host(&store)));
    let mut s = start(host, &store, Some((file.as_path(), Box::new(rules))));
    s.create("Badger", true);
    let mut t = 0;
    while s.error_text().is_empty() {
        s.frame();
        t += 1;
        assert!(t < 200, "no refusal");
    }
    let expected = load_pregame(&store)
        .unwrap()
        .strings
        .get("ID_Character_Err_NameBanned");
    assert_eq!(s.error_text(), expected);
    assert!(!s.client.frame().in_world);
    assert!(s.server.borrow().characters.records.is_empty());
    assert!(!file.exists(), "nothing was written");
    let _ = std::fs::remove_dir_all(dir);
}
