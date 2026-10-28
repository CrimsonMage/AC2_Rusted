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
    let info = crate::children::ChildInfo::holding(0, 0);
    crate::children::update_child_frame(holder, holder_scale, &info, bone.compose(&point.transform))
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_4;

    use ac2_anim::Ac2Transform;
    use ac2_math::{Frame, Quat, Vec3};

    use super::{
        find_holding_point, has_holding_location, held_place, placement_order, GroupHolding,
        HoldPoint,
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
}
