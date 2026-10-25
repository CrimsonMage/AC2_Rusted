//! Shared by the server-double tests: a synthetic world (flat ground everywhere, a pillar in
//! block 0x7F7F), a synthetic motion description, and the session configuration.

#![allow(dead_code)]

use std::sync::Arc;

use ac2_client::{BodyData, WorldData};
use ac2_core::{DataId, QualifiedDataId};
use ac2_db::ResourcePackedDecode;
use ac2_net::Credential;
use ac2_physics::{
    BodyShape, CylSphere, Frame, ObjectShape, Position, Quaternion, Sphere, StaticObject, Terrain,
    Vector,
};
use ac2_weenie::{CharacterChoice, OnlineConfig};
use ac2_world::{Ac2MotionInterpDesc, Ac2MotionValues};
use legions_server_double::{addr, SERVER};

pub const BLOCK: u32 = 0x7F7F_0000;
/// Land cell (3, 3) of block 0x7F7F (`LandDefs::lcoord_to_gid`).
pub const CELL: u32 = BLOCK | 0x1C;
pub const GROUND: f32 = 50.0;
/// The pillar: 2 m radius at (70, 76).
pub const PILLAR: [f32; 2] = [70.0, 76.0];
pub const PILLAR_RADIUS: f32 = 2.0;
pub const STEP: f64 = 1.0 / 64.0;

/// Synthetic locomotion: 4 m/s forward, 10 m/s^2 acceleration, gravity on.
pub fn values() -> Ac2MotionValues {
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
pub fn motion_desc() -> Arc<Ac2MotionInterpDesc> {
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
pub struct Synthetic;

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
            animation: None,
        })
    }

    fn game_clock(&self) -> Option<ac2_world::GameClock> {
        None
    }
}

pub fn config() -> OnlineConfig {
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

/// One client frame with every load asked for so far taken in first (deterministic), then
/// the server's clock steps a frame.
pub fn step(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    input: &ac2_client::FrameInput,
) -> Vec<ac2_weenie::SessionEvent> {
    client
        .settle_loads(std::time::Duration::from_secs(5))
        .unwrap();
    let events = client.use_time(input).unwrap();
    server.borrow_mut().elapsed += STEP;
    events
}

/// A client logged on through the double and standing in the world at `spawn`.
pub fn entered(
    spawn: legions_server_double::Spawn,
) -> (
    ac2_client::Client,
    std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) {
    entered_with(spawn, Arc::new(Synthetic))
}

/// [`entered`] over another world.
pub fn entered_with(
    spawn: legions_server_double::Spawn,
    world: Arc<dyn WorldData>,
) -> (
    ac2_client::Client,
    std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
) {
    let (transport, server) = legions_server_double::double(spawn);
    let session = ac2_weenie::OnlineSession::with_transport(config(), Box::new(transport)).unwrap();
    let mut client = ac2_client::Client::new(session, world);
    client.set_load_budget(std::time::Duration::from_secs(5));
    let idle = ac2_client::FrameInput::default();
    let mut frames = 0;
    while !client.frame().in_world {
        step(&mut client, &server, &idle);
        frames += 1;
        assert!(frames < 600, "no world entry");
    }
    for _ in 0..32 {
        step(&mut client, &server, &idle);
    }
    (client, server)
}
