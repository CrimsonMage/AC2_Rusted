use std::sync::Arc;

use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_core::{DataId, QualifiedDataId};
use ac2_db::{ResourcePackedDecode, ResourceStore};

use super::*;

const DID: u32 = 0x3200_0000;
/// Ground element: every human `MSList` that carries locomotion speeds is keyed by it.
const AC2_ELEMENT_GROUND: u32 = 0x4000_0001;

fn values(forward: f32) -> Vec<u32> {
    // `MotionValues::StreamPack` order: lift, drag, terminal, gravity, sink, hover, jump, max
    // forward/backward/strafe, still/running turn, max vertical, four brakes, five accelerations.
    let floats = [
        0.0,
        0.0,
        50.0,
        f32::from_bits(1),
        0.0,
        0.0,
        6.26,
        forward,
        4.0,
        4.5,
        90.0,
        120.0,
        0.0,
        18.0,
        25.0,
        25.0,
        1850.0,
        15.0,
        15.0,
        15.0,
        1000.0,
        0.0,
    ];
    floats.iter().map(|value| value.to_bits()).collect()
}

/// One ground list with a terrain (carrying one modifier) and a global modifier.
fn record() -> Vec<u8> {
    let mut words = vec![DID, 0x0200_0002];
    words.extend([AC2_ELEMENT_GROUND, AC2_MODE_DEFAULT, 0]);
    words.extend([AC2_ELEMENT_GROUND, 0x4100_000C, 0]);
    words.push(1); // MSList count
    words.extend([1, 0x8100_0003]); // terrain hash: one terrain
    words.extend([1, 0x7]); // its modifier hash: one entry
    words.extend(values(2.0));
    words.extend(values(5.0)); // terrain default
    words.extend([1, 0x9]); // global modifier hash
    words.extend(values(9.0));
    words.extend(values(6.5)); // list default
    words.extend([0.1_f32.to_bits(), 0.75_f32.to_bits()]);
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

fn decode_record(bytes: &[u8]) -> ResourceResult<Ac2MotionInterpDesc> {
    let qdid = QualifiedDataId::new(DataId::new(DID), 0);
    Ac2MotionInterpDesc::decode_packed(qdid, 0, bytes)
}

#[test]
fn motion_values_follow_retail_get_motion_values_precedence() {
    let desc = decode_record(&record()).expect("synthetic record decodes");
    let pick = |mode, global, terrain, modifier| {
        desc.motion_values(AC2_ELEMENT_GROUND, mode, global, terrain, modifier)
            .map(|values| values.max_forward_velocity)
    };
    assert_eq!(
        pick(AC2_MODE_DEFAULT, 0, AC2_TERRAIN_DEFAULT, None),
        Some(6.5)
    );
    assert_eq!(pick(0x4100_000C, 9, AC2_TERRAIN_DEFAULT, None), Some(9.0));
    assert_eq!(pick(AC2_MODE_DEFAULT, 0, 0x8100_0003, None), Some(5.0));
    assert_eq!(pick(AC2_MODE_DEFAULT, 0, 0x8100_0003, Some(7)), Some(2.0));
    assert_eq!(pick(AC2_MODE_DEFAULT, 0, 0x8100_0003, Some(8)), None);
    assert_eq!(pick(0x4100_0002, 0, AC2_TERRAIN_DEFAULT, None), None);
    // `GetDesiredMotionValues` retries the default mode, then the static defaults.
    let fallback = desc.desired_motion_values(
        AC2_ELEMENT_GROUND,
        0x4100_0002,
        0,
        AC2_TERRAIN_DEFAULT,
        None,
    );
    assert_eq!(fallback.max_forward_velocity, 6.5);
    let missing = desc.desired_motion_values(0x4000_0007, AC2_MODE_DEFAULT, 0, 0, None);
    assert_eq!(missing, Ac2MotionValues::default());
    assert_eq!(
        (desc.sea_floor_multiplier(), desc.water_multiplier()),
        (0.1, 0.75)
    );
}

#[test]
fn truncated_and_trailing_records_are_rejected() {
    let bytes = record();
    assert!(decode_record(&bytes[..bytes.len() - 4]).is_err());
    let mut trailing = bytes.clone();
    trailing.extend([0; 4]);
    assert!(decode_record(&trailing).is_err());
}

#[test]
fn ac2_human_ground_motion_values_match_the_retail_desc() {
    let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
        return;
    };
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    let store = ResourceStore::new(Arc::clone(&dataset));
    // Human male `VisualDesc` 0x1F000023 names this desc in `m_midesc_did`.
    let desc = load_ac2_motion_interp_desc(&store, DataId::new(DID))
        .expect("MotionInterpDesc decodes")
        .expect("MotionInterpDesc exists");
    let ground = desc.desired_motion_values(
        AC2_ELEMENT_GROUND,
        AC2_MODE_DEFAULT,
        0,
        AC2_TERRAIN_DEFAULT,
        None,
    );
    assert_eq!(
        (
            ground.max_forward_velocity,
            ground.max_backward_velocity,
            ground.max_strafe_velocity
        ),
        (6.5, 4.0, 4.5)
    );
    assert_eq!(
        (ground.still_turn_velocity, ground.running_turn_velocity),
        (90.0, 120.0)
    );
    assert_eq!(
        (
            ground.forward_acceleration,
            ground.forward_brake,
            ground.backward_brake
        ),
        (15.0, 18.0, 25.0)
    );
    assert_eq!(
        (ground.turn_acceleration, ground.turn_brake),
        (1000.0, 1850.0)
    );
    assert!(ground.gravity && (ground.jump_impulse - 6.26).abs() < 1e-4);
    assert_eq!(desc.water_multiplier(), 0.75);
}

#[test]
fn ac2_human_element_values_match_the_retail_desc() {
    let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
        return;
    };
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    let store = ResourceStore::new(Arc::clone(&dataset));
    let desc = load_ac2_motion_interp_desc(&store, DataId::new(DID))
        .expect("MotionInterpDesc decodes")
        .expect("MotionInterpDesc exists");
    let values = desc.element_values(AC2_MODE_DEFAULT, 0, AC2_TERRAIN_DEFAULT, None);
    // Ground: 50 m/s terminal velocity, gravity, no drag.
    assert_eq!(values.ground.terminal_velocity, 50.0);
    assert_eq!(values.ground.drag, 0.0);
    // Air: the same limits but no acceleration or brake, so no air control.
    assert!(values.air.gravity);
    assert_eq!(
        (
            values.air.forward_acceleration,
            values.air.strafe_acceleration,
            values.air.forward_brake,
            values.air.turn_acceleration
        ),
        (0.0, 0.0, 0.0, 0.0)
    );
    // Swimming: buoyancy 20, drag 0.5, no gravity, 4.5 m/s.
    assert!(!values.water.gravity);
    assert_eq!(
        (
            values.water.lift,
            values.water.drag,
            values.water.max_forward_velocity
        ),
        (20.0, 0.5, 4.5)
    );
    // Wading on the sea floor: 5.5 m/s, drag 0.7, a 4 m/s jump impulse.
    assert_eq!(
        (
            values.sea_floor.max_forward_velocity,
            values.sea_floor.drag,
            values.sea_floor.jump_impulse
        ),
        (5.5, 0.7, 4.0)
    );
    assert_eq!(
        (values.water_multiplier, values.sea_floor_multiplier),
        (0.75, 0.1)
    );
}
