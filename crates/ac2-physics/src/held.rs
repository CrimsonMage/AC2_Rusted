//! Held objects (`CPhysicsObj::UpdateChild`'s holding-location children): a wielded or carried item
//! stands at its holder's holding location every frame and is drawn there, so it moves with the
//! holder's animation (`CPartArray::GetHoldingLocation`'s posed bone).

use ac2_anim::Ac2Transform;
use ac2_math::{Frame, Vec3};

/// A Setup holding location (`LocationType`): the root bone it rides and its transform in that
/// bone's space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoldPoint {
    /// The part group whose Setup lists the location (`CPartGroup::GetHoldingLocation`): an index
    /// into the holder's drawn groups.
    pub group: usize,
    pub bone: usize,
    pub transform: ac2_anim::Ac2Transform,
}

/// A Setup holding location (`CSetup`'s `LocationType` list) and the part group whose Setup lists
/// it: the group's index in the visual's drawn groups.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupHolding {
    pub group: usize,
    pub location: ac2_anim::Ac2Location,
}

/// `PHYSICSOBJREP::HasHoldingLocation` (`CPartGroup::HasHoldingLocation`): any group lists the
/// location. `holding` is in the groups' search order (`ac2_assets::holding_search_order`).
pub fn has_holding_location(holding: &[GroupHolding], location: u32) -> bool {
    holding.iter().any(|entry| entry.location.id == location)
}

/// `PHYSICSOBJREP::GetHoldingLocation` (`CPartGroup::GetHoldingLocation`): the first entry for
/// `location` in the groups' search order, as the point `UpdateChild` places a child at.
pub fn find_holding_point(holding: &[GroupHolding], location: u32) -> Option<HoldPoint> {
    holding
        .iter()
        .find(|entry| entry.location.id == location)
        .map(|entry| HoldPoint {
            group: entry.group,
            bone: entry.location.bone,
            transform: entry.location.transform,
        })
}

/// The order `follow_holders` places held objects in: each object after the object that holds it
/// (`holder_of[i]`, an index into the same list), so a held object that holds another is placed
/// before the object it holds (`CPhysicsObj::UpdateChild` runs on the holder first). A cycle is
/// cut where it closes.
pub fn placement_order(holder_of: &[Option<usize>]) -> Vec<usize> {
    fn visit(i: usize, holder_of: &[Option<usize>], state: &mut [u8], order: &mut Vec<usize>) {
        if state[i] != 0 {
            return;
        }
        state[i] = 1;
        if let Some(holder) = holder_of[i] {
            visit(holder, holder_of, state, order);
        }
        state[i] = 2;
        order.push(i);
    }
    let mut state = vec![0u8; holder_of.len()];
    let mut order = Vec::with_capacity(holder_of.len());
    for i in 0..holder_of.len() {
        visit(i, holder_of, &mut state, &mut order);
    }
    order
}

/// Where a held object stands: the holder's frame, then its holding location (the holder's posed
/// root bone, then the location's transform in that bone's space), the offset scaled by the
/// holder's total scale (`CPhysicsObj::UpdateChild`'s `Frame::combine` with the holding matrix).
pub fn held_place(
    holder: Frame,
    holder_scale: Vec3,
    bone: Ac2Transform,
    point: &HoldPoint,
) -> Frame {
    held_place_offset(holder, holder_scale, bone, point, Frame::IDENTITY)
}

/// [`held_place`] for a child with its own offset frame (`CHILDINFO::m_offset`), as a bone-index
/// child (`CPhysicsObj::StickySetParent`) has. Its `point` is the bone with an identity
/// transform: `UpdateChild` takes `CPartGroup::GetBoneMatrix` alone.
pub fn held_place_offset(
    holder: Frame,
    holder_scale: Vec3,
    bone: Ac2Transform,
    point: &HoldPoint,
    offset: Frame,
) -> Frame {
    let mut info = crate::children::ChildInfo::holding(0, 0);
    info.offset = offset;
    crate::children::update_child_frame(holder, holder_scale, &info, bone.compose(&point.transform))
}

/// `m_posDrawing` of a parented child (`CPhysicsObj::UpdateChild`), which
/// `CMissileManager::InitPosition` reads through `GetDrawingPosition` while the missile
/// `HasParent`: the holder's drawing position in its cell, then the child's holding matrix and
/// offset. `bone` is the holder's posed root bone the holding location rides; the caller passes
/// the identity when only the rest pose is known. `None` when the holder's Setup lacks the
/// holding location.
pub fn parent_drawing_position(
    holder: &crate::position::Position,
    holder_scale: crate::vector::Vector,
    info: &crate::children::ChildInfo,
    bone: Ac2Transform,
    point: Option<&HoldPoint>,
) -> Option<crate::position::Position> {
    use crate::children::{holding_matrix, update_child_frame, HOLDING_LOCATION_ORIGIN};

    let matrix = if info.kind == crate::children::ChildType::BoneIndex {
        // `CPartGroup::GetBoneMatrix(index)`: the bone alone.
        bone
    } else if info.location == HOLDING_LOCATION_ORIGIN {
        Ac2Transform::IDENTITY
    } else {
        holding_matrix(info.location, bone, point?)
    };
    let from = &holder.frame;
    let frame = Frame::new(
        Vec3::new(from.origin.x, from.origin.y, from.origin.z),
        ac2_math::Quat::from_xyzw(from.q.x, from.q.y, from.q.z, from.q.w),
    );
    let at = update_child_frame(
        frame,
        Vec3::new(holder_scale.x, holder_scale.y, holder_scale.z),
        info,
        matrix,
    );
    let q = at.orientation;
    Some(crate::position::Position::new(
        holder.cell_id,
        crate::frame::Frame::new(
            crate::vector::Vector::new(at.origin.x, at.origin.y, at.origin.z),
            crate::vector::Quaternion {
                w: q.w,
                x: q.x,
                y: q.y,
                z: q.z,
            },
        ),
    ))
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_4;

    use ac2_anim::Ac2Transform;
    use ac2_math::{Frame, Quat, Vec3};

    use super::{
        find_holding_point, has_holding_location, held_place, parent_drawing_position,
        placement_order, GroupHolding, HoldPoint,
    };

    #[test]
    fn the_first_group_listing_a_location_supplies_it() {
        let at = |group, id, bone| GroupHolding {
            group,
            location: ac2_anim::Ac2Location {
                id,
                bone,
                transform: Ac2Transform::IDENTITY,
            },
        };
        let holding = [at(0, 5, 1), at(1, 5, 2), at(1, 6, 3)];
        assert!(has_holding_location(&holding, 6) && !has_holding_location(&holding, 7));
        let point = find_holding_point(&holding, 5).expect("listed");
        assert_eq!((point.group, point.bone), (0, 1));
        assert!(find_holding_point(&holding, 7).is_none());
    }

    fn translated(offset: Vec3) -> Ac2Transform {
        Ac2Transform {
            offset,
            ..Ac2Transform::IDENTITY
        }
    }

    fn point(bone: usize, offset: Vec3) -> HoldPoint {
        HoldPoint {
            group: 0,
            bone,
            transform: translated(offset),
        }
    }

    #[test]
    fn a_bone_index_child_keeps_its_rotation_to_the_bone() {
        use super::held_place_offset;
        // The child sits on the holder's bone (z = 1) and keeps a quarter turn about z.
        let holder = Frame::new(Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY);
        let bone = translated(Vec3::new(0.0, 0.0, 1.0));
        let turn = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let at = held_place_offset(
            holder,
            Vec3::ONE,
            bone,
            &point(0, Vec3::ZERO),
            Frame::new(Vec3::ZERO, turn),
        );
        assert!(
            (at.origin - Vec3::new(10.0, 0.0, 1.0)).length() < 1e-5,
            "{at:?}"
        );
        assert!(at.orientation.dot(turn).abs() > 0.9999, "{at:?}");
    }

    #[test]
    fn a_held_item_stands_at_its_holding_point() {
        let holder = Frame::new(Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY);
        let bone = translated(Vec3::new(0.0, 0.0, 1.0));
        let at = held_place(holder, Vec3::ONE, bone, &point(0, Vec3::new(0.5, 0.0, 0.0)));
        assert!(
            (at.origin - Vec3::new(10.5, 0.0, 1.0)).length() < 1e-5,
            "{at:?}"
        );
    }

    #[test]
    fn a_holder_that_is_itself_held_is_placed_before_the_object_it_holds() {
        // 0 is held by 2, 2 by 1, 1 is not held: the chain 1, 2, 0 is the placement order.
        assert_eq!(placement_order(&[Some(2), None, Some(1)]), vec![1, 2, 0]);
        // A cycle terminates and still places every object once.
        let cycle = placement_order(&[Some(1), Some(0)]);
        assert_eq!(cycle.len(), 2);
        assert!(cycle.contains(&0) && cycle.contains(&1));
    }

    #[test]
    fn a_held_item_moves_with_its_holders_bone_and_scale() {
        let holder = Frame::new(Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY);
        let hold = point(2, Vec3::new(1.0, 0.0, 0.0));
        // A quarter turn about z: the quaternion carries half the angle.
        let quarter_turn = Quat::from_xyzw(0.0, 0.0, FRAC_PI_4.sin(), FRAC_PI_4.cos());
        let turned = Ac2Transform {
            rotation: quarter_turn,
            ..Ac2Transform::IDENTITY
        };
        // The bone turns a quarter about the holder: the point at (1, 0, 0) swings to (0, 1, 0).
        let at = held_place(holder, Vec3::ONE, turned, &hold);
        assert!(
            (at.origin - Vec3::new(10.0, 1.0, 0.0)).length() < 1e-5,
            "{at:?}"
        );
        // The holder's total scale scales the offset of the place.
        let scaled = held_place(holder, Vec3::splat(2.0), turned, &hold);
        assert!(
            (scaled.origin - Vec3::new(10.0, 2.0, 0.0)).length() < 1e-5,
            "{scaled:?}"
        );
    }

    #[test]
    fn a_parented_missile_starts_where_its_holder_holds_it() {
        use crate::children::{ChildInfo, HOLDING_LOCATION_ORIGIN};
        use crate::position::Position;
        use crate::vector::{Quaternion, Vector};
        let holder = Position::new(
            0x7F7F_001C,
            crate::frame::Frame::new(Vector::new(10.0, 20.0, 30.0), Quaternion::IDENTITY),
        );
        let at_origin = ChildInfo::holding(5, HOLDING_LOCATION_ORIGIN);
        let placed = parent_drawing_position(
            &holder,
            Vector::new(1.0, 1.0, 1.0),
            &at_origin,
            Ac2Transform::IDENTITY,
            None,
        )
        .expect("the holder's own origin needs no Setup location");
        assert_eq!(placed.cell_id, 0x7F7F_001C);
        assert_eq!(placed.frame.origin, Vector::new(10.0, 20.0, 30.0));

        let at_hand = ChildInfo::holding(5, 7);
        assert!(
            parent_drawing_position(
                &holder,
                Vector::new(2.0, 2.0, 2.0),
                &at_hand,
                Ac2Transform::IDENTITY,
                None
            )
            .is_none(),
            "a location the Setup lacks places nothing"
        );
        let hand = point(0, Vec3::new(0.5, 0.0, 1.0));
        let placed = parent_drawing_position(
            &holder,
            Vector::new(2.0, 2.0, 2.0),
            &at_hand,
            Ac2Transform::IDENTITY,
            Some(&hand),
        )
        .expect("listed");
        assert!(
            (placed.frame.origin.x - 11.0).abs() < 1e-5
                && (placed.frame.origin.z - 32.0).abs() < 1e-5
        );
    }
}
