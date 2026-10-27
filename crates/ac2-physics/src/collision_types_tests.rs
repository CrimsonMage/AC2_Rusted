//! The collision sets of `CollisionParameters`: masks, ethereal and solid lists, and the
//! `CPhysicsObj::IsBlocking` answer they give an object.

use crate::collision_types::{CollisionParameters, PhysicsTypeMask, WeenieTypes};

#[test]
fn set_bit_only_takes_bits_below_64() {
    let mut mask = PhysicsTypeMask::default();
    mask.set_bit(3, true);
    mask.set_bit(63, true);
    mask.set_bit(64, true);
    assert_eq!(mask.bits(), (1 << 3) | (1 << 63));
    mask.set_bit(3, false);
    assert_eq!(mask.bits(), 1 << 63);
}

#[test]
fn set_property_needs_the_master_property_to_exist() {
    let mut mask = PhysicsTypeMask::default();
    let known = |name: u64| name == 5;
    assert!(
        !mask.set_property(9, true, known),
        "unknown name changes nothing"
    );
    assert_eq!(mask.bits(), 0);
    assert!(mask.set_property(5, true, known));
    assert_eq!(mask.bits(), 1 << 5);
    assert!(mask.set_property(5, false, known));
    assert_eq!(mask.bits(), 0);
}

#[test]
fn weenie_lists_hold_each_id_once() {
    let mut types = WeenieTypes::default();
    assert!(types.add_to_ethereal_list(7));
    assert!(types.add_to_ethereal_list(7));
    assert!(types.in_ethereal_list(7));
    assert!(!types.in_solid_list(7));
    assert!(types.add_to_solid_list(8));
    assert!(types.in_solid_list(8));
}

#[test]
fn a_default_body_is_blocked_by_every_object() {
    // `CollisionParameters::CollisionParameters` leaves the weenie sets solid and the masks empty.
    let params = CollisionParameters::default();
    for placement in [false, true] {
        assert!(params.blocks(42, PhysicsTypeMask::default(), placement));
    }
}

#[test]
fn an_ethereal_id_is_not_blocked_on_either_set() {
    let mut params = CollisionParameters::default();
    assert!(params.add_to_ethereal_list(42));
    for placement in [false, true] {
        assert!(!params.blocks(42, PhysicsTypeMask::default(), placement));
        assert!(params.blocks(43, PhysicsTypeMask::default(), placement));
    }
}

#[test]
fn a_shared_physics_type_blocks_only_solid_ids() {
    // An object whose blocking mask shares a bit with the body's movement type blocks when the
    // id is solid, and passes when it is not listed.
    let mut params = CollisionParameters::default();
    assert!(params.update_movement(4, true, |_| true));
    let mut object = PhysicsTypeMask::default();
    object.set_bit(4, true);
    assert!(!params.blocks(1, object, false));
    assert!(params.add_to_ethereal_list(1));
    assert!(!params.blocks(1, object, false));
}

#[test]
fn updates_clear_the_set_on_first_use_then_change_bits() {
    let mut params = CollisionParameters::default();
    let all = |_: u64| true;
    assert!(params.update_placement(2, true, all));
    assert!(params.update_placement(3, true, all));
    assert!(params.update_placement(2, false, all));
    assert!(params.update_physics_type(6, true, all));
    assert_eq!(params.physics_type().bits(), 1 << 6);
    assert!(params.update_physics_type(6, false, all));
    assert_eq!(params.physics_type().bits(), 0);
    // A type beyond the 64-bit mask is accepted by the master property but sets no bit.
    assert!(params.update_physics_type(70, true, all));
    assert_eq!(params.physics_type().bits(), 0);
}
