//! Synthetic test inputs (no game data): `MotionInterpDesc` records built from round numbers.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_db::ResourcePackedDecode;
use ac2_world::{Ac2MotionInterpDesc, Ac2MotionValues};

use crate::motion_interp::MODE_DEFAULT;

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
