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

#[test]
fn a_script_weenie_set_merges_into_the_body_s_lists() {
    // `CCollisionManager::SetCollisionState` with the weenie bits: `EtherealWeenieType::Update`
    // appends the script's ids to the body's, so an id the body already holds stays.
    let mut body = CollisionParameters::default();
    assert!(body.add_to_ethereal_list(7));
    let mut info = CollisionParameters::default();
    assert!(info.add_to_ethereal_list(9));
    body.merge_set_state(&info);
    for placement in [false, true] {
        assert!(!body.blocks(7, PhysicsTypeMask::default(), placement));
        assert!(!body.blocks(9, PhysicsTypeMask::default(), placement));
        assert!(body.blocks(8, PhysicsTypeMask::default(), placement));
    }
}

#[test]
fn a_script_mask_replaces_only_the_set_its_bit_names() {
    // Bit 1 (movement physics) replaces the body's movement mask; the placement mask it does
    // not name stays as the body had it.
    let all = |_: u64| true;
    let mut body = CollisionParameters::default();
    assert!(body.update_placement(2, true, all));
    let mut info = CollisionParameters::default();
    assert!(info.update_movement(4, true, all));
    body.merge_set_state(&info);
    let mut movement = PhysicsTypeMask::default();
    movement.set_bit(4, true);
    assert!(!body.blocks(1, movement, false), "movement 4 is now shared");
    let mut placement = PhysicsTypeMask::default();
    placement.set_bit(2, true);
    assert!(
        !body.blocks(1, placement, true),
        "placement 2 is kept and shared"
    );
    assert!(
        body.blocks(1, placement, false),
        "the movement mask has no bit 2"
    );
}

#[test]
fn an_empty_script_state_changes_nothing() {
    let all = |_: u64| true;
    let mut body = CollisionParameters::default();
    assert!(body.add_to_ethereal_list(7));
    assert!(body.update_physics_type(6, true, all));
    let before = body.clone();
    body.merge_set_state(&CollisionParameters::default());
    assert_eq!(body, before);
}

#[test]
fn the_physics_type_bit_replaces_the_body_s_physics_type() {
    // `CPhysicsObj::SetPhysicsCollideMask`: the script's physics type replaces the old one.
    let all = |_: u64| true;
    let mut body = CollisionParameters::default();
    assert!(body.update_physics_type(6, true, all));
    let mut info = CollisionParameters::default();
    assert!(info.update_physics_type(8, true, all));
    body.merge_set_state(&info);
    assert_eq!(body.physics_type().bits(), 1 << 8);
}

#[test]
fn the_temp_ethereal_list_adds_each_id_once() {
    // `CollisionParameters::AddToTempEtherealList`: a held id is found, not appended again.
    let mut body = CollisionParameters::default();
    assert!(!body.in_temp_ethereal_list(7));
    assert!(body.add_to_temp_ethereal_list(7));
    assert!(body.add_to_temp_ethereal_list(7));
    assert!(body.in_temp_ethereal_list(7));
    assert_eq!(body.find_in_temp_ethereal_list(7), Some(0));
}

#[test]
fn removing_from_the_temp_ethereal_list_moves_the_last_id_into_the_slot() {
    // `CollisionParameters::RemoveFromTempEtherealList`: the last entry takes the removed slot.
    let mut body = CollisionParameters::default();
    for id in [1, 2, 3] {
        body.add_to_temp_ethereal_list(id);
    }
    assert!(body.remove_from_temp_ethereal_list(1));
    assert_eq!(body.find_in_temp_ethereal_list(3), Some(0));
    assert_eq!(body.find_in_temp_ethereal_list(2), Some(1));
    assert!(!body.in_temp_ethereal_list(1));
}

#[test]
fn removing_an_absent_id_reports_success_and_changes_nothing() {
    // Retail returns 1 from `RemoveFromTempEtherealList` even when the id was not held.
    let mut body = CollisionParameters::default();
    body.add_to_temp_ethereal_list(4);
    let before = body.clone();
    assert!(body.remove_from_temp_ethereal_list(9));
    assert_eq!(body, before);
}

#[test]
fn the_temp_ethereal_list_is_separate_from_the_weenie_sets() {
    // The weenie ethereal lists (`AddToEtherealList`) do not feed the temp list.
    let mut body = CollisionParameters::default();
    assert!(body.add_to_ethereal_list(5));
    assert!(!body.in_temp_ethereal_list(5));
}

#[test]
fn the_set_type_calls_replace_the_set_and_raise_its_bit() {
    let mut params = CollisionParameters::default();
    params.set_physics_movement_type(PhysicsTypeMask::from_bits(0b110));
    params.set_physics_placement_type(PhysicsTypeMask::from_bits(0b1));
    let mut weenie = WeenieTypes::default();
    weenie.add_to_ethereal_list(9);
    params.set_weenie_movement_type(&weenie);
    params.set_weenie_placement_type(&weenie);
    let state = params.collision_state(PhysicsTypeMask::from_bits(0b100));
    assert_eq!(state.placement_physics.bits(), 0b1);
    assert_eq!(state.movement_physics.bits(), 0b100);
    assert_eq!(state.bitfield, 0x1 | 0x4);
    assert_eq!(params.bitfield, 0x1 | 0x2 | 0x4 | 0x8);
    assert!(params.movement_weenie.in_ethereal_list(9));
    assert!(params.placement_weenie.in_ethereal_list(9));
}

#[test]
fn the_current_physics_mask_gains_the_desired_types() {
    let mut params = CollisionParameters::default();
    params.set_physics_movement_type(PhysicsTypeMask::from_bits(0b001));
    params.update_current_physics_mask(PhysicsTypeMask::from_bits(0b110));
    assert_eq!(params.movement_physics.bits(), 0b111);
}

#[test]
fn update_ethereality_adds_the_lowest_missing_type_only() {
    let mut mask = PhysicsTypeMask::from_bits(0b0001);
    assert!(mask.update_ethereality(PhysicsTypeMask::from_bits(0b1101)));
    assert_eq!(mask.bits(), 0b0101);
    assert!(mask.update_ethereality(PhysicsTypeMask::from_bits(0b1101)));
    assert_eq!(mask.bits(), 0b1101);
    // Nothing missing: unchanged.
    assert!(mask.update_ethereality(PhysicsTypeMask::from_bits(0b0100)));
    assert_eq!(mask.bits(), 0b1101);
    let mut high = PhysicsTypeMask::default();
    high.update_ethereality(PhysicsTypeMask::from_bits(1 << 63));
    assert_eq!(high.bits(), 1 << 63);
}

#[test]
fn the_camera_ethereal_mask_holds_only_the_camera_type() {
    let mut mask = PhysicsTypeMask::from_bits(u64::MAX);
    mask.set_camera_ethereal(5, |bit| bit == 5);
    assert_eq!(mask.bits(), 1 << 5);
    // A property the table does not know leaves the mask empty.
    mask.set_camera_ethereal(5, |_| false);
    assert_eq!(mask.bits(), 0);
}

#[test]
fn the_water_layer_bit_ends_cleared() {
    let mut mask = PhysicsTypeMask::from_bits(1 << 7);
    mask.set_ethereal_to_water_layer(true, 7, |_| true);
    assert_eq!(mask.bits(), 0);
    let mut open = PhysicsTypeMask::default();
    open.set_ethereal_to_water_layer(false, 7, |_| true);
    assert_eq!(open.bits(), 0);
}

#[test]
fn a_weenie_set_needs_a_line_of_sight_check_unless_empty() {
    let mut types = WeenieTypes::empty();
    assert!(!types.need_los_check());
    types.add_to_ethereal_list(9);
    assert!(!types.need_los_check());
    types.add_to_solid_list(9);
    assert!(types.need_los_check());
    let mut solid = WeenieTypes::empty();
    solid.set_to_solid();
    assert!(solid.need_los_check());
}
