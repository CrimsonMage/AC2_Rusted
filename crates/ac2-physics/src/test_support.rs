//! Synthetic test inputs (no game data): `MotionInterpDesc` records built from round numbers.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_db::ResourcePackedDecode;
use ac2_world::{Ac2MotionInterpDesc, Ac2MotionValues};

use crate::body::PhysicsBody;
use crate::frame::Frame;
use crate::landdefs::SIDE_VERTEX_COUNT;
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND, MODE_DEFAULT};
use crate::physics_obj::BodyShape;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::{Quaternion, Sphere, Vector};

/// Synthetic locomotion: 4 m/s forward, 2 m/s back, 3 m/s strafe, 10 m/s^2 accelerations,
/// 20 m/s^2 brakes, 60/90 deg/s turns with 360 deg/s^2 turn acceleration.
pub(crate) fn walking_values() -> Ac2MotionValues {
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

/// `MotionValues::StreamPack` field order.
fn pack(values: &Ac2MotionValues) -> Vec<u32> {
    let v = values;
    let mut words = vec![v.lift, v.drag, v.terminal_velocity]
        .into_iter()
        .map(f32::to_bits)
        .collect::<Vec<_>>();
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
}

/// A desc with one default-mode list per element, no terrains or modifiers.
pub(crate) fn desc(lists: &[(u32, Ac2MotionValues)]) -> Arc<Ac2MotionInterpDesc> {
    const DID: u32 = 0x3200_0F00;
    let mut words = vec![DID, lists.len() as u32];
    for (index, (element, _)) in lists.iter().enumerate() {
        words.extend([*element, MODE_DEFAULT, index as u32]);
    }
    words.push(lists.len() as u32);
    for (_, values) in lists {
        words.extend([0, 0]); // no terrains, no global modifiers
        words.extend(pack(values));
    }
    words.extend([0.1_f32.to_bits(), 0.75_f32.to_bits()]);
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let qdid = QualifiedDataId::new(DataId::new(DID), 0);
    Arc::new(Ac2MotionInterpDesc::decode_packed(qdid, 0, &bytes).expect("synthetic desc"))
}

/// A desc with one default-mode list whose global modifier `global_id` has `global` values, over
/// `default` (no terrains).
pub(crate) fn desc_with_global(
    default: &Ac2MotionValues,
    global_id: u32,
    global: &Ac2MotionValues,
) -> Arc<Ac2MotionInterpDesc> {
    const DID: u32 = 0x3200_0F01;
    let element = crate::motion_interp::ELEMENT_GROUND;
    let mut words = vec![DID, 1, element, MODE_DEFAULT, 0, 1, 0, 1, global_id];
    words.extend(pack(global));
    words.extend(pack(default));
    words.extend([0.1_f32.to_bits(), 0.75_f32.to_bits()]);
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let qdid = QualifiedDataId::new(DataId::new(DID), 0);
    Arc::new(Ac2MotionInterpDesc::decode_packed(qdid, 0, &bytes).expect("synthetic desc"))
}

/// A desc with one default-mode list whose `terrain` lists `modifier` with `modified` values,
/// over `default` (no global modifiers).
pub(crate) fn desc_with_terrain(
    default: &Ac2MotionValues,
    terrain: u32,
    modifier: u32,
    modified: &Ac2MotionValues,
) -> Arc<Ac2MotionInterpDesc> {
    const DID: u32 = 0x3200_0F02;
    let element = crate::motion_interp::ELEMENT_GROUND;
    // One list with one terrain, which has one modifier, then the terrain's default values; no
    // global modifiers.
    let mut words = vec![DID, 1, element, MODE_DEFAULT, 0, 1, 1, terrain, 1, modifier];
    words.extend(pack(modified));
    words.extend(pack(default));
    words.push(0);
    words.extend(pack(default));
    words.extend([0.1_f32.to_bits(), 0.75_f32.to_bits()]);
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let qdid = QualifiedDataId::new(DataId::new(DID), 0);
    Arc::new(Ac2MotionInterpDesc::decode_packed(qdid, 0, &bytes).expect("synthetic desc"))
}

/// A `PHYSICSOBJREP` stand-in recording the behaviour, mode and FX calls of `CMotionInterp`.
#[derive(Debug, Default)]
pub(crate) struct RecordingRep {
    pub doing: Vec<u32>,
    pub started: Vec<u32>,
    pub stopped: Vec<u32>,
    pub fx: Vec<(u32, f32)>,
    pub mode: u32,
}

impl crate::motion_interp_behaviors::MotionRep for RecordingRep {
    fn do_autonomous_behavior(&mut self, behavior: u32) {
        self.started.push(behavior);
        if !self.doing.contains(&behavior) {
            self.doing.push(behavior);
        }
    }

    fn is_doing_behavior(&self, behavior: u32) -> bool {
        self.doing.contains(&behavior)
    }

    fn stop_behavior(&mut self, behavior: u32) {
        self.stopped.push(behavior);
        self.doing.retain(|&b| b != behavior);
    }

    fn do_mode(&mut self, _context: u32, mode: u32) {
        self.mode = mode;
    }

    fn desired_mode(&self) -> u32 {
        self.mode
    }

    fn do_fx(&mut self, fx: u32, intensity: f32) {
        self.fx.push((fx, intensity));
    }
}

/// A synthetic behaviour table: ground and air in the default mode, linked both ways with a
/// 0.3 s transition; idle (anim key 1) and move (anim key 2) in both, each a 1 s clip.
pub(crate) fn body_animation() -> crate::anim_rep::BodyAnimation {
    use std::collections::HashMap;

    use ac2_anim::{
        Ac2AnimAsset, Ac2AssetBone, Ac2BehaviorInfo, Ac2BehaviorTable, Ac2CharTemplate,
        Ac2ComboInfo, Ac2DbAnimator, Ac2ModeTransition,
    };

    let (idle, mv) = (0x4000_0002, 0x4000_0001);
    let mut combos = HashMap::new();
    for element in [ELEMENT_GROUND, ELEMENT_AIR] {
        combos.insert(
            (element, MODE_DEFAULT, idle),
            Ac2ComboInfo {
                anim_key: 1,
                layer: 0,
            },
        );
        combos.insert(
            (element, MODE_DEFAULT, mv),
            Ac2ComboInfo {
                anim_key: 2,
                layer: 1,
            },
        );
    }
    let ground = (ELEMENT_GROUND, MODE_DEFAULT);
    let air = (ELEMENT_AIR, MODE_DEFAULT);
    let transition = |src: (u32, u32), dst: (u32, u32)| Ac2ModeTransition {
        src_element: src.0,
        src_mode: src.1,
        dst_element: dst.0,
        dst_mode: dst.1,
    };
    let table = Ac2BehaviorTable {
        ramp_times: HashMap::from([
            (transition(ground, air), 0.3),
            (transition(air, ground), 0.3),
        ]),
        combos,
        transitions: HashMap::new(),
        layers: vec![0, 1],
        tweak_layers: Vec::new(),
        mode_transitions: HashMap::from([(ground, vec![air]), (air, vec![ground])]),
        modes: HashMap::from([(ground, 0), (air, 0)]),
        behavior_info: HashMap::from([(
            mv,
            Ac2BehaviorInfo {
                ramp_time: 0.2,
                effect: 0,
            },
        )]),
        init_behaviors: vec![idle],
        init_mode: MODE_DEFAULT,
    };
    let template = Ac2CharTemplate {
        behaviors: HashMap::from([(idle, 0), (0x4000_0003, 0), (mv, 0)]),
        sliders: HashMap::new(),
        default_element: ELEMENT_GROUND,
        default_mode: MODE_DEFAULT,
    };
    let clip = |x: f32| {
        Arc::new(Ac2DbAnimator::Asset(Arc::new(Ac2AnimAsset {
            period: 1.0,
            bones: vec![Ac2AssetBone {
                scale: vec![([1.0; 3], 0.0)],
                offset: vec![([x, 0.0, 0.0], 0.0)],
                rotation: vec![([1.0, 0.0, 0.0, 0.0], 0.0)],
                weight: vec![(1.0, 0.0)],
            }],
            info: None,
        })))
    };
    crate::anim_rep::BodyAnimation {
        table: Arc::new(table),
        template: Arc::new(template),
        animators: HashMap::from([(1, clip(0.25)), (2, clip(0.5))]),
        setup: None,
        vdesc_scale: [1.0; 3],
    }
}

pub(crate) const BLOCK: u32 = 0x7F7F_FFFF;

/// A block whose vertex heights are `height(x, y)` (vertex steps of 10 m).
pub(crate) fn terrain(height: impl Fn(usize, usize) -> f32) -> Terrain {
    let heights: Vec<f32> = (0..SIDE_VERTEX_COUNT)
        .flat_map(|x| (0..SIDE_VERTEX_COUNT).map(move |y| (x, y)))
        .map(|(x, y)| height(x, y))
        .collect();
    let mut terrain = Terrain::new();
    terrain.insert_heights(BLOCK, &heights, |_, _| false);
    terrain
}

/// Two stacked 0.4 m spheres, 1.8 m tall, 0.5 m steps.
pub(crate) fn shape() -> BodyShape {
    BodyShape {
        spheres: vec![
            Sphere::new(Vector::new(0.0, 0.0, 0.4), 0.4),
            Sphere::new(Vector::new(0.0, 0.0, 1.2), 0.4),
        ],
        height: 1.8,
        step_up_height: 0.5,
        step_down_height: 0.5,
        scale: 1.0,
        allow_free_heading: false,
    }
}

pub(crate) fn body_at(x: f32, y: f32, z: f32) -> PhysicsBody {
    let cell =
        crate::landdefs::lcoord_to_gid(0x7F * 8 + (x / 20.0) as i32, 0x7F * 8 + (y / 20.0) as i32);
    let mut values_air = walking_values();
    values_air.forward_acceleration = 0.0;
    values_air.backward_acceleration = 0.0;
    values_air.strafe_acceleration = 0.0;
    let desc = desc(&[
        (ELEMENT_GROUND, walking_values()),
        (ELEMENT_AIR, values_air),
    ]);
    let mut body = PhysicsBody::new(
        Position::new(cell, Frame::new(Vector::new(x, y, z), Quaternion::IDENTITY)),
        shape(),
        Some(desc),
    );
    body.set_player(true);
    body
}

thread_local! {
    /// The test thread's `Timer::local_time`, advanced by [`animate`].
    static LOCAL_TIME: std::cell::Cell<f64> = const { std::cell::Cell::new(1.0) };
    /// The test thread's shared generator.
    static RANDOM: std::cell::RefCell<ac2_core::Random> = std::cell::RefCell::default();
}

/// One animation step of `quantum` seconds on the test thread's local clock.
pub(crate) fn animate(body: &mut PhysicsBody, quantum: f32) {
    let now = LOCAL_TIME.with(|time| {
        time.set(time.get() + f64::from(quantum));
        time.get()
    });
    let mut random = RANDOM.with(|random| random.take());
    body.update_animation(quantum, now, &mut random);
    RANDOM.with(|cell| cell.replace(random));
}
