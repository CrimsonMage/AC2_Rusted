//! The headless client end to end against the Legions 7160 server double, on synthetic world
//! data: log on, spawn on flat ground, walk north into a pillar (physics collision), take a
//! server force-position, and log off; every movement report is checked byte for byte.

use std::sync::Arc;
use std::time::Duration;

use ac2_client::{BodyData, Client, FrameInput, Notice, WorldData};
use ac2_core::{DataId, QualifiedDataId};
use ac2_db::ResourcePackedDecode;
use ac2_net::protocol::{ClientEvent, ClientPosition};
use ac2_net::Credential;
use ac2_physics::{
    BodyShape, CylSphere, Frame, ObjectShape, Position, Quaternion, Sphere, StaticObject, Terrain,
    Vector,
};
use ac2_weenie::{
    ActiveAction, CellPosition, CharacterChoice, OnlineConfig, OnlineSession,
    ACTION_CLASS_MOVEMENT, ACTION_FORWARD,
};
use ac2_world::{Ac2MotionInterpDesc, Ac2MotionValues};
use legions_server_double::{addr, double, Spawn, SERVER};

const BLOCK: u32 = 0x7F7F_0000;
/// Land cell (3, 3) of block 0x7F7F (`LandDefs::lcoord_to_gid`).
const CELL: u32 = BLOCK | 0x1C;
const GROUND: f32 = 50.0;
/// The pillar: 2 m radius at (70, 76).
const PILLAR: [f32; 2] = [70.0, 76.0];
const PILLAR_RADIUS: f32 = 2.0;
const STEP: f64 = 1.0 / 64.0;

/// Synthetic locomotion: 4 m/s forward, 10 m/s^2 acceleration, gravity on.
fn values() -> Ac2MotionValues {
    Ac2MotionValues {
        lift: 0.0,
        drag: 0.0,
        terminal_velocity: 50.0,
        gravity: true,
        sink_offset: 0.0,
        hover_height: 0.0,
        jump_impulse: 5.0,
        max_forward_velocity: 4.0,
        max_backward_velocity: 2.0,
        max_strafe_velocity: 3.0,
        still_turn_velocity: 60.0,
        running_turn_velocity: 90.0,
        max_vertical_velocity: 0.0,
        forward_brake: 20.0,
        backward_brake: 20.0,
        strafe_brake: 20.0,
        turn_brake: 360.0,
        forward_acceleration: 10.0,
        backward_acceleration: 10.0,
        strafe_acceleration: 10.0,
        turn_acceleration: 360.0,
        vertical_acceleration: 0.0,
    }
}

/// A `MotionInterpDesc` with one default-mode list for the ground and the air
/// (`MotionValues::StreamPack` order).
fn motion_desc() -> Arc<Ac2MotionInterpDesc> {
    const DID: u32 = 0x3200_0F00;
    let v = values();
    let pack = |v: &Ac2MotionValues| {
        let mut words: Vec<u32> = [v.lift, v.drag, v.terminal_velocity]
            .map(f32::to_bits)
            .to_vec();
        words.push(u32::from(v.gravity));
        words.extend(
            [
                v.sink_offset,
                v.hover_height,
                v.jump_impulse,
                v.max_forward_velocity,
                v.max_backward_velocity,
                v.max_strafe_velocity,
                v.still_turn_velocity,
                v.running_turn_velocity,
                v.max_vertical_velocity,
                v.forward_brake,
                v.backward_brake,
                v.strafe_brake,
                v.turn_brake,
                v.forward_acceleration,
                v.backward_acceleration,
                v.strafe_acceleration,
                v.turn_acceleration,
                v.vertical_acceleration,
            ]
            .map(f32::to_bits),
        );
        words
    };
    let mut air = v;
    air.forward_acceleration = 0.0;
    let lists = [(0x4000_0001u32, v), (0x4000_0002, air)];
    let mut words = vec![DID, lists.len() as u32];
    for (index, (element, _)) in lists.iter().enumerate() {
        words.extend([*element, 0x4000_0001, index as u32]);
    }
    words.push(lists.len() as u32);
    for (_, values) in &lists {
        words.extend([0, 0]);
        words.extend(pack(values));
    }
    words.extend([0.1_f32.to_bits(), 0.75_f32.to_bits()]);
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    let qdid = QualifiedDataId::new(DataId::new(DID), 0);
    Arc::new(Ac2MotionInterpDesc::decode_packed(qdid, 0, &bytes).expect("synthetic desc"))
}

/// Flat ground on every block, and the pillar in block 0x7F7F.
struct Synthetic;

impl WorldData for Synthetic {
    fn load_block(&self, terrain: &mut Terrain, block: u32) -> Result<bool, String> {
        terrain.insert_heights(block, &[GROUND; 17 * 17], |_, _| false);
        if block == BLOCK {
            let shape = ObjectShape::new(
                Vec::new(),
                vec![CylSphere {
                    low: Vector::new(0.0, 0.0, 0.0),
                    height: 3.0,
                    radius: PILLAR_RADIUS,
                }],
                Vec::new(),
            );
            let at = Position::new(
                CELL,
                Frame::new(
                    Vector::new(PILLAR[0], PILLAR[1], GROUND),
                    Quaternion::new(1.0, 0.0, 0.0, 0.0),
                ),
            );
            terrain.insert_object(StaticObject::new(
                at,
                Vector::new(1.0, 1.0, 1.0),
                Arc::new(shape),
            ));
        }
        Ok(true)
    }

    fn body(&self, _visual: Option<u32>) -> Result<BodyData, String> {
        Ok(BodyData {
            shape: BodyShape {
                spheres: vec![
                    Sphere::new(Vector::new(0.0, 0.0, 0.4), 0.4),
                    Sphere::new(Vector::new(0.0, 0.0, 1.2), 0.4),
                ],
                height: 1.8,
                step_up_height: 0.5,
                step_down_height: 0.5,
                scale: 1.0,
                allow_free_heading: false,
            },
            motion: Some(motion_desc()),
            radius: 0.4,
        })
    }

    fn game_clock(&self) -> Option<ac2_world::GameClock> {
        None
    }
}

fn config() -> OnlineConfig {
    OnlineConfig {
        server: addr(SERVER),
        account: "local".into(),
        credential: Credential::Account,
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::First,
        server_key: None,
        language: 0,
        dat_iterations: Vec::new(),
    }
}

/// `ShouldSendPositionEvent`, written out: a do-motion change, or more than 0.2 s and a moved
/// body (the heading never changes here).
fn should_report(
    last: Option<(f64, CellPosition, [f32; 3])>,
    now: (f64, CellPosition, [f32; 3]),
) -> bool {
    let Some((time, position, motion)) = last else {
        return true;
    };
    let differs = |a: f32, b: f32| f64::from(0.0002f32) <= (f64::from(a) - f64::from(b)).abs();
    if (0..3).any(|i| differs(now.2[i], motion[i])) {
        return true;
    }
    if now.0 - time <= 0.2 {
        return false;
    }
    now.1.cell != position.cell || (0..3).any(|i| differs(now.1.origin[i], position.origin[i]))
}

#[test]
fn the_headless_client_walks_into_a_pillar_and_takes_a_forced_position() {
    let spawn = Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    };
    let (transport, server) = double(spawn);
    let session = OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    let mut client = Client::new(session, Arc::new(Synthetic));
    client.set_load_budget(Duration::from_secs(5));
    let idle = FrameInput::default();
    let mut frames = 0;
    let mut notices = Vec::new();
    while !client.frame().in_world {
        // Every load asked for in a frame is in by the next one.
        client.settle_loads(Duration::from_secs(5)).unwrap();
        client.use_time(&idle).unwrap();
        let frame = client.frame();
        // `IsBlockingForCells` holds the body out of the world (and its physics) until the
        // cells around it are in.
        if frame.blocking_for_cells {
            assert!(!frame.in_world);
        }
        notices.extend(client.take_notices());
        server.borrow_mut().elapsed += STEP;
        frames += 1;
        assert!(frames < 600, "no world entry");
    }
    // World entry: ready to enter the world, then the frame-drawn notice in the same frame,
    // which sent `SceneRenderingComplete` once.
    assert_eq!(
        notices,
        vec![Notice::PlayerReadyToEnterWorld, Notice::NextFrameDrawn]
    );
    assert_eq!(client.frame().blocks.len(), 9);
    client.use_time(&idle).unwrap();
    server.borrow_mut().elapsed += STEP;
    let scene_complete = ClientEvent::SceneRenderingComplete.encode().1;
    let sent = |state: &legions_server_double::State| {
        state
            .blobs
            .iter()
            .filter(|(_, _, b)| *b == scene_complete)
            .count()
    };
    assert_eq!(sent(&server.borrow()), 1);
    // Settle onto the ground for a second.
    for _ in 0..64 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }
    let start = client.frame().player.unwrap();
    assert!(start.contact, "{start:?}");
    // The camera sits 4.5 m behind the player at the default 0.25 rad pitch, lifted by its
    // height, looking at three quarters of its height.
    let camera = client.frame().camera.expect("camera");
    let pitch = 0.25_f64;
    let behind = [
        start.origin[0] as f64,
        start.origin[1] as f64 - 4.5 * pitch.cos(),
        start.origin[2] as f64 + 4.5 * pitch.sin() + 1.8,
    ];
    for axis in 0..3 {
        assert!(
            (camera.origin[axis] as f64 - behind[axis]).abs() < 0.05,
            "{camera:?} {behind:?}"
        );
    }
    // It looks at the top of the player (its height above the origin).
    let head = [start.origin[0], start.origin[1], start.origin[2] + 1.8];
    for axis in 0..3 {
        assert!(
            (camera.target[axis] - head[axis]).abs() < 0.01,
            "{camera:?}"
        );
    }
    assert_eq!(camera.fov, 65.0);
    // `GetGlobalDirection`: the negated viewer offset (0, -1, pitch) the movement module steers
    // by.
    assert_eq!(client.camera().global_direction(), [0.0, 1.0, -0.25]);
    let reports_before = position_reports(&server.borrow());

    // Walk north for three seconds: the pillar stops the body short of its surface.
    let walk = FrameInput {
        actions: vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: ACTION_FORWARD,
            map_extent: 1.0,
            extent: 1.0,
        }],
        ..FrameInput::default()
    };
    let mut expected = Vec::new();
    let mut last = None;
    for _ in 0..192 {
        // The report reads the body as the previous frame left it, with this frame's motion.
        let before = client.frame().player.unwrap();
        client.use_time(&walk).unwrap();
        let time = client.session().cur_time();
        let position = CellPosition {
            cell: before.cell,
            origin: before.origin,
        };
        let now = (time, position, [0.0, 1.0, 0.0]);
        if should_report(last, now) {
            last = Some(now);
            expected.push(
                ClientEvent::Position(ClientPosition {
                    time,
                    position,
                    heading_degrees: before.heading_degrees,
                    motion: [0.0, 1.0, 0.0],
                    cancel_moveto: true,
                    instance_stamp: 9,
                    teleport_stamp: 5,
                    force_position_stamp: 6,
                    moveto_stamp: 7,
                    jump_velocity: None,
                })
                .encode()
                .1,
            );
        }
        server.borrow_mut().elapsed += STEP;
    }
    let stopped = client.frame().player.unwrap();
    let face = PILLAR[1] - PILLAR_RADIUS - 0.4;
    assert!(stopped.origin[1] < face + 0.01, "{stopped:?}");
    assert!(stopped.origin[1] > face - 0.25, "{stopped:?}");
    assert!((stopped.origin[0] - 70.0).abs() < 0.01, "{stopped:?}");
    let reports = position_reports(&server.borrow());
    assert_eq!(&reports[reports_before.len()..], &expected[..]);
    // About 1.4 s of walking at 4 m/s reaches the pillar; stuck, the body no longer reports.
    assert!(expected.len() >= 5, "{}", expected.len());

    // A newer force-position stamp moves the body (`RecvEvt_Position`, player branch).
    server
        .borrow_mut()
        .send_player_position(CELL, [60.0, 60.0, GROUND], [20, 5, 7]);
    for _ in 0..4 {
        client.use_time(&idle).unwrap();
        server.borrow_mut().elapsed += STEP;
    }
    // The body keeps the momentum it had (it may coast a little north on the next quantum).
    let moved = client.frame().player.unwrap();
    assert!((moved.origin[0] - 60.0).abs() < 0.01, "{moved:?}");
    assert!((0.0..0.5).contains(&(moved.origin[1] - 60.0)), "{moved:?}");

    assert!(client.session_mut().log_off(Duration::from_secs(3)));
    assert!(server.borrow().logged_off);
}

fn position_reports(state: &legions_server_double::State) -> Vec<Vec<u8>> {
    state
        .blobs
        .iter()
        .filter(|(_, _, b)| b[..4] == 0x0001_008Eu32.to_le_bytes())
        .map(|(_, _, b)| b.clone())
        .collect()
}
