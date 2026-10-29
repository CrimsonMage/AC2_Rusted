//! Held objects in a frame (`CPhysicsObj::UpdateChild`'s holding-location children): each held
//! object is placed where its holder's posed holding location puts it, in the order
//! `ac2_physics::placement_order` gives. The placement math is `ac2_physics::held_place`.

use std::collections::HashMap;

use ac2_anim::Ac2Transform;
use ac2_math::{Frame, Mat4};
use ac2_physics::{held_place_offset, placement_order, scaled_by_parent};
use ac2_render::object::ObjectPose;

use super::{world_matrix, ClientView, Drawn};

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
            .map(|child| child.held.and_then(|(id, _, _)| index.get(&id).copied()))
            .collect();
        // `CPhysicsObj::GetTotalScale`: a held object's scale carries its holder's total scale
        // (and the holder representation's `m_fChildScale`, 1.0 as `PHYSICSOBJREP` makes it).
        let mut totals: Vec<Option<ac2_math::Vec3>> = drawn
            .iter()
            .map(|d| self.scene.object_rig(d.key).map(|(_, scale)| *scale))
            .collect();
        for i in placement_order(&holder_of) {
            let child = &drawn[i];
            let Some((holder_id, point, offset)) = child.held else {
                continue;
            };
            let Some(&holder) = index.get(&holder_id) else {
                placed[i] = None;
                continue;
            };
            let Some(holder_scale) = totals[holder] else {
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
            let at = held_place_offset(frames[holder], holder_scale, bone, &point, offset);
            frames[i] = at;
            let child_scale = totals[i].map(|own| scaled_by_parent(own, holder_scale, 1.0));
            totals[i] = child_scale;
            placed[i] = match (placed[i].take(), child_scale) {
                (Some((key, _, pose)), Some(scale)) => Some((key, world_matrix(scale, at), pose)),
                _ => None,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_anim::Ac2Transform;
    use ac2_math::Vec3;
    use ac2_weenie::Appearance;

    use super::super::drawn_objects;
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

    fn object(id: u64, visual: Option<u32>, holder: Option<Holder>, in_world: bool) -> ObjectFrame {
        ObjectFrame {
            id,
            visual,
            setup: None,
            name: None,
            stack_name: None,
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
            body_scale: [1.0; 3],
            adaptive_degrade: true,
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
            hidden: false,
        }
    }

    fn holder(id: u64, point: Option<HoldPoint>) -> Holder {
        Holder {
            id,
            location: 1,
            orientation: None,
            point,
            offset: ac2_math::Frame::IDENTITY,
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

    /// `AvatarVisibleInFirstPerson`: a first-person camera that hides the avatar hides what it
    /// holds; other objects still draw.
    #[test]
    fn a_hidden_first_person_avatar_hides_its_wielded_item() {
        let frame = ClientFrame {
            player: Some(PlayerFrame {
                hidden: true,
                ..player(7)
            }),
            objects: vec![
                object(
                    9,
                    Some(0x0200_0002),
                    Some(holder(7, Some(point(0, Vec3::ZERO)))),
                    false,
                ),
                object(11, Some(0x0200_0003), None, true),
            ],
            ..ClientFrame::default()
        };
        assert_eq!(ids(&frame), vec![11]);
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
