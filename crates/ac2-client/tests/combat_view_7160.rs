//! A fight drawn headless on the real DATs (`AC2_DAT_DIR`) against the Legions 7160 server double:
//! the double's radar creatures are drudges (`mn_drudge.vd`) standing near the player in the town
//! block CB4D; the player selects one and attacks it, the blows are exchanged on the double's
//! timer, and the creature dies. Writes `target/ac2-snapshots/combat_{idle,fight,dead}.png`;
//! skipped without a GPU adapter.

#![cfg(feature = "window")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{Client, ClientFrame, Command, DatWorld, FrameInput};
use ac2_gpu::{FrameTarget, Gpu, COLOR_FORMAT};
use ac2_net::protocol::{heading_to_u8, motion_to_u24};
use ac2_net::Credential;
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession, SessionEvent};
use legions_server_double::{
    addr, double, object_event_bytes, position_bytes, ObjectPosition, Spawn, BEHAVIOR_MELEE_AUTO,
    BEHAVIOR_MELEE_NORMAL, BOW_AUTO_ATTACK, CREATURE_ONE, DRUDGE_VISUAL, MODE_COMBAT_MONSTER,
    PLAYER, SERVER,
};

/// `gmbehaviors.emp`'s `Damage_Hit1`: a hit's reaction on its target.
const BEHAVIOR_DAMAGE_HIT: u32 = 0x4100_007C;

/// `gmmodes.emp`'s `dead`.
const MODE_DEAD: u32 = 0x4100_0005;

const STEP: f64 = 1.0 / 64.0;
const SIZE: (u32, u32) = (800, 600);
/// Seconds between the double's exchanges of blows (`combat::HIT_INTERVAL`).
const HIT_INTERVAL: f64 = 2.0;

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

/// One frame: the loads in, the client's time, the double's clock and its fight.
fn step(client: &mut Client, server: &Shared, input: &FrameInput) -> Vec<SessionEvent> {
    client.settle_loads(Duration::from_secs(30)).unwrap();
    let events = client.use_time(input).unwrap();
    server.borrow_mut().elapsed += STEP;
    server.borrow_mut().combat_tick();
    events
}

fn advance(client: &mut Client, server: &Shared, secs: f64) -> Vec<SessionEvent> {
    let idle = FrameInput::default();
    let mut events = Vec::new();
    for _ in 0..(secs / STEP).round() as usize {
        events.extend(step(client, server, &idle));
    }
    events
}

struct Scene {
    gpu: Gpu,
    view: ClientView,
    target: FrameTarget,
}

impl Scene {
    fn draw(&mut self, frame: &ClientFrame, name: &str) -> Vec<u8> {
        let aspect = SIZE.0 as f32 / SIZE.1 as f32;
        self.view.update(&self.gpu, frame, Duration::ZERO);
        self.view.settle(&self.gpu, frame, aspect);
        assert!(self
            .view
            .prepare(&self.gpu, frame, aspect, frame.server_time));
        self.view.render(
            &self.gpu,
            &self.target.color_view,
            &self.target.depth_view,
            true,
        );
        let rgba = self.target.read_rgba(&self.gpu.device, &self.gpu.queue);
        write_png(name, &rgba);
        rgba
    }
}

/// The client in the world at the town block CB4D facing east, its scripts on the VM, the
/// double's drudges spawned with their fight started, and a scene to draw it; `None` without
/// `AC2_DAT_DIR` or a GPU adapter.
fn setup() -> Option<(Client, Shared, Scene)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return None;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(Arc::clone(&dataset)));
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
    let spawn = Spawn {
        cell: 0xCB4D_0025,
        origin: [96.0, 96.0, top + 2.0],
        // Facing east (a quarter turn about z), toward the drudge the fight is with.
        rotation: [
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            0.0,
            -std::f32::consts::FRAC_1_SQRT_2,
        ],
    };
    let (transport, server) = double(spawn);
    server.borrow_mut().creature_visual = DRUDGE_VISUAL;
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
    let script_data = ac2_script::data::DatScriptData::new(dataset, (*store).clone());
    session.set_string_tables(ac2_client::string_table_ids(&script_data));
    session.set_chat_commands(ac2_client::chat_commands(&script_data));
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
    let mut frames = 0;
    while !client.frame().in_world {
        step(&mut client, &server, &idle);
        frames += 1;
        assert!(frames < 600);
    }
    server.borrow_mut().send_init_player();
    advance(&mut client, &server, 2.0);
    // The login's portal space fades out before play.
    let mut waited = 0;
    while client.frame().portal.in_portal_space {
        step(&mut client, &server, &idle);
        waited += 1;
        assert!(waited < 2000, "the login's portal space did not run out");
    }
    {
        let mut state = server.borrow_mut();
        state.spawn_radar_objects();
        state.start_fight();
    }
    advance(&mut client, &server, 2.0);

    let scene = Scene {
        view: ClientView::new(world, &gpu, COLOR_FORMAT, 1, SIZE),
        target: FrameTarget::new(&gpu.device, SIZE.0, SIZE.1),
        gpu,
    };
    Some((client, server, scene))
}

#[test]
fn a_drudge_is_drawn_fought_and_killed() {
    let Some((mut client, server, mut scene)) = setup() else {
        return;
    };
    let idle_frame = client.frame();
    assert!(
        idle_frame
            .objects
            .iter()
            .any(|o| o.id == CREATURE_ONE && o.visual == Some(DRUDGE_VISUAL)),
        "the drudge is in the client's world with its visual"
    );
    scene.draw(&idle_frame, "combat_idle.png");

    // Select the drudge and attack it: both go into their fighting stances (`DoMode`), and each
    // exchange's swings (`DoStory` with the attacker's behaviour) play on the bodies.
    let attack = FrameInput {
        commands: vec![
            Command::Select(Some(CREATURE_ONE)),
            Command::SetAttacking(true),
        ],
        ..FrameInput::default()
    };
    step(&mut client, &server, &attack);
    advance(&mut client, &server, HIT_INTERVAL - 0.1);
    let mut seen = [false; 3];
    for i in 0..8 {
        advance(&mut client, &server, 0.125);
        // The exchange: the player's `Melee_Auto1`, the drudge's `Melee_Normal1`, and the
        // drudge's reaction to the player's blow (`Damage_Hit1`, which the script's
        // `GetFXAndBehaviorInformationFromWsl` gives the hit).
        for (flag, (object, behavior)) in seen.iter_mut().zip([
            (PLAYER, BEHAVIOR_MELEE_AUTO),
            (CREATURE_ONE, BEHAVIOR_MELEE_NORMAL),
            (CREATURE_ONE, BEHAVIOR_DAMAGE_HIT),
        ]) {
            *flag |= client.is_doing_behavior(object, behavior);
        }
        let frame = client.frame();
        let drudge = frame.objects.iter().find(|o| o.id == CREATURE_ONE).unwrap();
        assert_eq!(
            drudge.animated_mode,
            Some(MODE_COMBAT_MONSTER),
            "the drudge fights in combat_monster, where its swing has a combo"
        );
        scene.draw(&frame, &format!("combat_fight_{i}.png"));
    }
    assert_eq!(
        seen, [true; 3],
        "the swings and the hit reaction play (player swing, drudge swing, drudge hit)"
    );

    // The drudge dies on the third exchange: its death plays, then it leaves for its corpse.
    let mut waited = 0.0;
    while !server.borrow().fight.dead.contains(&CREATURE_ONE) {
        advance(&mut client, &server, 0.125);
        waited += 0.125;
        assert!(waited < 4.0 * HIT_INTERVAL, "the drudge did not die");
    }
    advance(&mut client, &server, 1.0);
    let dying = client.frame();
    // Its `death` behaviour took it into the `dead` mode, where it lies before it leaves.
    let drudge = dying.objects.iter().find(|o| o.id == CREATURE_ONE);
    assert_eq!(
        drudge.map(|o| (o.in_world, o.animated_mode)),
        Some((true, Some(MODE_DEAD))),
        "the drudge lies in its death before it leaves"
    );
    scene.draw(&dying, "combat_dying.png");
    advance(&mut client, &server, legions_server_double::DEATH_SECONDS);
    let dead = client.frame();
    assert!(
        !dead
            .objects
            .iter()
            .any(|o| o.id == CREATURE_ONE && o.in_world),
        "the drudge has left the world"
    );
    scene.draw(&dead, "combat_dead.png");
}

/// `gmbehaviors.emp`'s `move`: the locomotion a moving body's motion plays.
const BEHAVIOR_MOVE: u32 = 0x4000_0001;

/// A server-moved creature walks: the double reports the drudge walking west toward the player
/// at 4 m/s, a position every 0.25 s with its forward motion (`PositionPack`'s do-motion), and
/// the client draws it moving between them with its `move` animation.
#[test]
fn a_drudge_walks_toward_the_player_with_its_move_animation() {
    let Some((mut client, server, mut scene)) = setup() else {
        return;
    };
    let start = client.frame();
    let drudge = start
        .objects
        .iter()
        .find(|o| o.id == CREATURE_ONE)
        .expect("the drudge is in the world");
    let (cell, [x0, y, z]) = (drudge.cell, drudge.origin);
    let heading_and_motion = u32::from(heading_to_u8(270.0)) << 24 | motion_to_u24([0.0, 1.0, 0.0]);
    let mut walked = false;
    for k in 0..6u16 {
        let now = client.session().cur_time();
        server
            .borrow_mut()
            .send_event(&position_bytes(&ObjectPosition {
                id: CREATURE_ONE,
                instance: 1,
                time: now,
                cell,
                origin: [x0 - f32::from(k), y, z],
                heading_and_motion,
                flags: 4,
                stamps: [2 + k, 0, 0],
                velocity: None,
            }));
        for _ in 0..16 {
            step(&mut client, &server, &FrameInput::default());
            walked |= client.is_doing_behavior(CREATURE_ONE, BEHAVIOR_MOVE);
        }
        if k == 3 {
            scene.draw(&client.frame(), "walk_drudge.png");
        }
    }
    assert!(walked, "the drudge plays its move animation while it walks");
    let end = client.frame();
    let x = end
        .objects
        .iter()
        .find(|o| o.id == CREATURE_ONE)
        .map(|o| o.origin[0])
        .unwrap();
    assert!(x < x0 - 3.0, "the drudge walked west: {x0} -> {x}");
}

/// A skill whose effects the script gives the attacker (`GetFXAndBehaviorInformationFromWsl`
/// answers skill 0x11 with attacker FX 0x41000507; the human visual's FX table has particles
/// for it).
const SKILL_WITH_FX: u32 = 0x11;
const SKILL_ATTACKER_FX: u32 = 0x4100_0507;

/// A special attack's story with a skill: the server's `DoStory` on the player names the skill
/// and one damage hook on the drudge, and the story's hook plays the skill's effects: the
/// attacker FX's particles start on the player (`FXManager::DoParticleFX`).
#[test]
fn a_skill_story_plays_the_skills_effects_on_the_attacker() {
    let Some((mut client, server, mut scene)) = setup() else {
        return;
    };
    // As the binary does: a sound manager (silent here); the FX's sound and particle parts run
    // with it.
    let store = Arc::new(ac2_db::ResourceStore::new(
        ac2_dat::test_support::env_dataset().unwrap(),
    ));
    client.attach_audio(ac2_client::sound_manager(
        store,
        Box::new(ac2_audio::NullSink),
    ));
    client.take_object_particle_starts();
    // The view has drawn before (its particle clock runs with the frames, as the window's does).
    scene.draw(&client.frame(), "skill_fx_before.png");
    // The story: attacker and skill (bits 0x2 | 0x10), one hook: damage (1) with the target
    // (0x100), the attack result (0x80) and the target's health change (0x800).
    let args = [
        0x2 | 0x10,
        PLAYER as u32,
        (PLAYER >> 32) as u32,
        SKILL_WITH_FX,
        1,
        1 | 0x100 | 0x80 | 0x800,
        CREATURE_ONE as u32,
        (CREATURE_ONE >> 32) as u32,
        0x4000_0008,
        (-5i32) as u32,
    ];
    server.borrow_mut().send_event(&object_event_bytes(
        ac2_net::protocol::Opcode::DO_STORY,
        PLAYER,
        9,
        &args,
    ));
    let mut starts = Vec::new();
    for _ in 0..16 {
        step(&mut client, &server, &FrameInput::default());
        starts.extend(client.take_object_particle_starts());
    }
    let skill_fx: Vec<_> = starts
        .iter()
        .filter(|s| s.emitter == PLAYER && s.fx == SKILL_ATTACKER_FX)
        .collect();
    assert!(
        !skill_fx.is_empty(),
        "the skill's attacker FX starts its particles on the player: {starts:?}"
    );
    // The view starts the systems, and they emit as the frames go on.
    scene.view.set_object_particles(&starts, &[]);
    for i in 0..4 {
        for _ in 0..8 {
            step(&mut client, &server, &FrameInput::default());
        }
        scene.draw(&client.frame(), &format!("skill_fx_{i}.png"));
        let (systems, alive) = scene.view.object_particles();
        assert!(
            systems > 0 && alive > 0,
            "the skill's particles draw around the player ({systems} systems, {alive} alive)"
        );
    }
}

/// A bow attack's story (`Hum_Mi_AutoAttackMissile1`): its hooks make the arrow the script names
/// (`GetMissileInformationFromWSL`: entity `0x470000BE` in holding location 4), hang it from the
/// player and fire it at the drudge (`StoryManager::CreateAndParentMissiles`, `FireMissile`). The
/// arrow flies under its missile manager from the player's hand and sticks in the drudge
/// (`sticks_upon_collide`), drawn on the way.
#[test]
fn a_bow_story_fires_an_arrow_into_the_drudge() {
    let Some((mut client, server, mut scene)) = setup() else {
        return;
    };
    let player = client.frame().player.as_ref().expect("the player").origin;
    server
        .borrow_mut()
        .send_missile_story(PLAYER, CREATURE_ONE, BOW_AUTO_ATTACK);
    let drudge = client
        .frame()
        .objects
        .iter()
        .find(|o| o.id == CREATURE_ONE)
        .map(|o| o.origin)
        .expect("the drudge is drawn");
    let flat = |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
    let mut distances = Vec::new();
    let mut heights = Vec::new();
    for i in 0..24 {
        step(&mut client, &server, &FrameInput::default());
        let missiles = client.story_missiles();
        assert!(missiles.len() <= 1, "one arrow: {missiles:?}");
        let Some(&(_, _, Some(at), flying)) = missiles.first() else {
            continue;
        };
        let o = at.frame.origin;
        let at = [o.x, o.y, o.z];
        // Made at the player; its body loads first, and the next physics quantum gives it its
        // missile manager, places it at the hand and fires it.
        if !flying {
            assert!(flat(at, player) < 0.5, "made at the player: {at:?}");
            continue;
        }
        distances.push(flat(at, drudge));
        heights.push(at[2] - drudge[2]);
        if i % 4 == 0 {
            scene.draw(&client.frame(), &format!("arrow_{i}.png"));
        }
    }
    assert!(!distances.is_empty(), "the arrow flies");
    assert!(
        distances.windows(2).all(|w| w[1] <= w[0] + 0.01),
        "the arrow closes on the drudge: {distances:?}"
    );
    assert!(
        *distances.last().unwrap() < 1.0,
        "the arrow sticks in the drudge: {distances:?}"
    );
    // It leaves the player's hand (holding location 4, chest high) and strikes the drudge's
    // body, three quarters of its height up (`m_targetHeightAdjustment` 0.75), not its feet.
    let hit = *heights.last().unwrap();
    assert!(
        (1.0..1.8).contains(&hit),
        "the arrow strikes the drudge's body: {heights:?}"
    );
    // It sticks in the drudge (`StickySetParent`): hung from the drudge's nearest bone, out of
    // the world, drawn on that bone.
    let missiles = client.story_missiles();
    assert_eq!(missiles.first().map(|m| m.1), Some(Some(CREATURE_ONE)));
    let frame = client.frame();
    let arrow = frame
        .objects
        .iter()
        .find(|o| o.id == missiles[0].0)
        .expect("the stuck arrow is drawn");
    assert_eq!(arrow.holder.map(|h| h.id), Some(CREATURE_ONE));
    assert!(arrow.holder.is_some_and(|h| h.point.is_some()), "on a bone");
    scene.draw(&frame, "arrow_stuck.png");
    // Stuck, the arrow is mostly inside the drudge, and in flight it points away from the camera
    // behind the player. Moved into the open, it draws: a temporary object draws its entity's
    // visual (`ICPhysicsObj::makeObject`).
    let mut open = client.frame();
    let player = open.player.as_ref().expect("the player").clone();
    let arrow = open
        .objects
        .iter_mut()
        .find(|o| Some(o.id) == client.story_missiles().first().map(|m| m.0))
        .expect("the arrow is in the frame");
    arrow.cell = player.cell;
    // Beside the player and across the view (its shaft north-south), not end-on behind it.
    arrow.origin = [
        player.origin[0] + 2.0,
        player.origin[1] + 1.0,
        player.origin[2] + 1.5,
    ];
    arrow.rotation = [1.0, 0.0, 0.0, 0.0];
    arrow.holder = None;
    arrow.in_world = true;
    let with = scene.draw(&open, "arrow_open.png");
    open.objects.retain(|o| o.id >> 56 != 0x13);
    let without = scene.draw(&open, "arrow_open_without.png");
    let changed = with
        .chunks(4)
        .zip(without.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 20, "the arrow draws: {changed} pixels");
}
