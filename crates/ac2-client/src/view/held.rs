//! Held objects (`CPhysicsObj::UpdateChild`'s holding-location children): a wielded or carried item
//! stands at its holder's holding location every frame and is drawn there, so it moves with the
//! holder's animation (`CPartArray::GetHoldingLocation`'s posed bone).

use std::collections::HashMap;

use ac2_anim::Ac2Transform;
use ac2_math::{Frame, Mat4, Vec3};
use ac2_render::object::ObjectPose;

use super::{world_matrix, ClientView, Drawn};
use crate::frame::HoldPoint;

/// The order `follow_holders` places held objects in: each object after the object that holds it
/// (`holder_of[i]`, an index into the same list), so a held object that holds another is placed
/// before the object it holds (`CPhysicsObj::UpdateChild` runs on the holder first). A cycle is
/// cut where it closes.
pub(crate) fn placement_order(holder_of: &[Option<usize>]) -> Vec<usize> {
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
pub(crate) fn held_place(
    holder: Frame,
    holder_scale: Vec3,
    bone: Ac2Transform,
    point: &HoldPoint,
) -> Frame {
    let at = bone.compose(&point.transform);
    Frame::new(
        holder.origin + holder.orientation * (at.offset * holder_scale),
        (holder.orientation * at.rotation).normalize(),
    )
}

impl ClientView {
    /// `CPhysicsObj::UpdateChild` for each held object: it stands where its holder's posed
    /// holding location puts it this frame. An object whose holder is not drawn is not drawn.
    pub(super) fn follow_holders(
        &self,
        drawn: &[Drawn<'_>],
        frames: &mut [Frame],
        placed: &mut [Option<(u64, Mat4, Option<ObjectPose>)>],
    ) {
        let index: HashMap<u64, usize> = drawn
            .iter()
            .enumerate()
            .map(|(i, drawn)| (drawn.id, i))
            .collect();
        let holder_of: Vec<Option<usize>> = drawn
            .iter()
            .map(|child| child.held.and_then(|(id, _)| index.get(&id).copied()))
            .collect();
        for i in placement_order(&holder_of) {
            let child = &drawn[i];
            let Some((holder_id, point)) = child.held else {
                continue;
            };
            let Some(&holder) = index.get(&holder_id) else {
                placed[i] = None;
                continue;
            };
            let Some((_, holder_scale)) = self.scene.object_rig(drawn[holder].key) else {
                placed[i] = None;
                continue;
            };
            let Some(holder_pose) = placed[holder].as_ref() else {
                placed[i] = None;
                continue;
            };
            let bone = holder_pose
                .2
                .as_ref()
                .and_then(|pose| pose.group_bones.get(point.group)?.get(point.bone))
                .copied()
                .unwrap_or(Ac2Transform::IDENTITY);
            let at = held_place(frames[holder], *holder_scale, bone, &point);
            frames[i] = at;
            let child_scale = self.scene.object_rig(child.key).map(|(_, scale)| *scale);
            placed[i] = match (placed[i].take(), child_scale) {
                (Some((key, _, pose)), Some(scale)) => Some((key, world_matrix(scale, at), pose)),
                _ => None,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_4;
    use std::sync::Arc;

    use ac2_anim::Ac2Transform;
    use ac2_math::{Frame, Quat, Vec3};
    use ac2_weenie::Appearance;

    use super::super::drawn_objects;
    use super::held_place;
    use crate::frame::{ClientFrame, HoldPoint, Holder, ObjectFrame, PlayerFrame};

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
        assert_eq!(
            super::placement_order(&[Some(2), None, Some(1)]),
            vec![1, 2, 0]
        );
        // A cycle terminates and still places every object once.
        let cycle = super::placement_order(&[Some(1), Some(0)]);
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

    fn object(id: u64, visual: Option<u32>, holder: Option<Holder>, in_world: bool) -> ObjectFrame {
        ObjectFrame {
            id,
            visual,
            setup: None,
            name: None,
            cell: 0x0A0B_0101,
            origin: [1.0, 2.0, 3.0],
            rotation: [1.0, 0.0, 0.0, 0.0],
            heading_degrees: None,
            in_world,
            lost_cell: None,
            active: true,
            velocity: [0.0; 3],
            contact: false,
            mode: None,
            animated_mode: None,
            motion: [0.0; 3],
            holder,
            fx_follow: None,
            container: None,
            wielder: None,
            appearance: Arc::new(Appearance::default()),
            pose: Vec::new(),
            anim_scale: [1.0; 4],
        }
    }

    fn player(id: u64) -> PlayerFrame {
        PlayerFrame {
            id,
            cell: 0x0A0B_0101,
            origin: [0.0; 3],
            rotation: [1.0, 0.0, 0.0, 0.0],
            heading_degrees: 0.0,
            visual: Some(0x0200_0001),
            velocity: [0.0; 3],
            contact: true,
            pose: Vec::new(),
            appearance: Arc::new(Appearance::default()),
        }
    }

    fn holder(id: u64, point: Option<HoldPoint>) -> Holder {
        Holder {
            id,
            location: 1,
            orientation: None,
            point,
        }
    }

    fn ids(frame: &ClientFrame) -> Vec<u64> {
        drawn_objects(frame).iter().map(|d| d.id).collect()
    }

    #[test]
    fn a_wielded_item_of_the_player_is_drawn_at_its_holding_point() {
        let frame = ClientFrame {
            player: Some(player(7)),
            objects: vec![object(
                9,
                Some(0x0200_0002),
                Some(holder(7, Some(point(0, Vec3::ZERO)))),
                false,
            )],
            ..ClientFrame::default()
        };
        assert_eq!(
            ids(&frame),
            vec![7, 9],
            "the wielded sword is drawn with the player"
        );
    }

    #[test]
    fn a_held_item_of_a_remote_character_is_drawn_and_one_without_a_point_is_not() {
        let frame = ClientFrame {
            objects: vec![
                object(5, Some(0x0200_0003), None, true),
                object(
                    6,
                    Some(0x0200_0004),
                    Some(holder(5, Some(point(0, Vec3::ZERO)))),
                    false,
                ),
                object(8, Some(0x0200_0005), Some(holder(5, None)), false),
            ],
            ..ClientFrame::default()
        };
        assert_eq!(ids(&frame), vec![5, 6]);
    }
}
