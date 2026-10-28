use ac2_db::Ac2PropertyValue;

use super::*;

const VECTOR: u32 = 0x4000_0004;
const BOOL: u32 = 0x4000_0001;

fn vector(name: u32, v: [f32; 3]) -> Ac2PropertyValue {
    let mut bytes = Vec::new();
    for c in v {
        bytes.extend_from_slice(&c.to_le_bytes());
    }
    Ac2PropertyValue {
        name,
        value_type: VECTOR,
        bytes: bytes.into_boxed_slice(),
    }
}

fn flag(name: u32, on: bool) -> Ac2PropertyValue {
    Ac2PropertyValue {
        name,
        value_type: BOOL,
        bytes: u32::from(on).to_le_bytes().to_vec().into_boxed_slice(),
    }
}

#[test]
fn a_pusher_sums_the_general_and_the_element_fields() {
    let mut pushers = Pushers::default();
    assert!(pushers.add_properties(
        7,
        None,
        &[
            vector(GENERAL_VELOCITY, [1.0, 0.0, 0.0]),
            vector(AIR_VELOCITY, [0.0, 2.0, 0.0]),
            vector(AIR_ACCELERATION, [0.0, 0.0, 3.0]),
            vector(WATER_VELOCITY, [9.0, 9.0, 9.0]),
        ],
    ));
    let air = pushers.update_values(ELEMENT_AIR);
    assert!(!air.walk_on_air);
    assert_eq!((air.velocity.x, air.velocity.y), (1.0, 2.0));
    assert_eq!(air.acceleration.z, 3.0);
    // The ground element reads the earth fields: only the general velocity applies.
    let ground = pushers.update_values(ELEMENT_GROUND);
    assert_eq!((ground.velocity.x, ground.velocity.y), (1.0, 0.0));
    // An element no case names adds only the general fields.
    let walking = pushers.update_values(0x4000_0007);
    assert_eq!(walking.velocity.x, 1.0);
}

#[test]
fn the_water_velocity_alone_does_not_make_a_pusher() {
    let mut pushers = Pushers::default();
    assert!(!pushers.add_properties(1, None, &[vector(WATER_VELOCITY, [1.0, 1.0, 1.0])]));
    assert!(pushers.is_empty());
    // With another counted property it is kept with the water velocity.
    assert!(pushers.add_properties(
        1,
        None,
        &[
            vector(WATER_VELOCITY, [1.0, 1.0, 1.0]),
            vector(WATER_ACCELERATION, [0.0, 0.0, 1.0]),
        ],
    ));
    let sum = pushers.update_values(ELEMENT_WATER);
    assert_eq!(
        sum.velocity.x, 1.0,
        "the water velocity is summed for the water element"
    );
    assert_eq!(sum.acceleration.z, 1.0);
}

#[test]
fn a_vector_under_epsilon_on_every_axis_is_ignored() {
    let mut pushers = Pushers::default();
    assert!(!pushers.add_properties(1, None, &[vector(GENERAL_VELOCITY, [0.0001, 0.0, 0.0])]));
}

#[test]
fn a_walk_on_air_pusher_replaces_the_sums_and_is_added_again() {
    let mut pushers = Pushers::default();
    pushers.add_properties(1, None, &[vector(GENERAL_VELOCITY, [1.0, 0.0, 0.0])]);
    pushers.add_properties(
        2,
        None,
        &[vector(GENERAL_VELOCITY_WALKS_ON_AIR, [0.0, 0.0, 5.0])],
    );
    let sum = pushers.update_values(ELEMENT_AIR);
    assert!(sum.walk_on_air);
    // The first pusher's x is replaced; the second's z counts twice.
    assert_eq!(sum.velocity.x, 0.0);
    assert_eq!(sum.velocity.z, 10.0);
}

#[test]
fn an_object_pushes_once_and_removing_it_stops_it() {
    let mut pushers = Pushers::default();
    assert!(pushers.add_properties(3, None, &[vector(GENERAL_ACCELERATION, [0.0, 0.0, 1.0])]));
    assert!(!pushers.add_properties(3, None, &[vector(GENERAL_ACCELERATION, [0.0, 0.0, 9.0])]));
    assert_eq!(pushers.get(3).unwrap().general_acl.z, 1.0);
    assert!(pushers.remove(3));
    assert!(!pushers.remove(3));
    assert_eq!(pushers.update_values(ELEMENT_AIR), PusherSum::default());
}

#[test]
fn local_space_turns_the_vectors_by_the_object_frame() {
    use crate::frame::Frame;
    use crate::vector::Quaternion;
    let h = std::f32::consts::FRAC_1_SQRT_2;
    let position = Position::new(
        0x0101_0001,
        Frame::new(
            Vector::ZERO,
            Quaternion {
                w: h,
                x: 0.0,
                y: 0.0,
                z: h,
            },
        ),
    );
    let mut pushers = Pushers::default();
    pushers.add_properties(
        1,
        Some(&position),
        &[
            flag(LOCAL_SPACE, true),
            vector(GENERAL_VELOCITY, [1.0, 0.0, 0.0]),
        ],
    );
    let v = pushers.get(1).unwrap().general_vel;
    let expected = position.local_to_global_vec(Vector::new(1.0, 0.0, 0.0));
    assert_eq!((v.x, v.y, v.z), (expected.x, expected.y, expected.z));
    assert!(v.x.abs() < 1.0, "the vector was turned");
}
