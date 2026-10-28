//! The FX objects' placement on their emitter (`CPhysicsObj::FXSetParent`, `CPhysicsObj::UpdateChild`):
//! an object an FX creates is placed at its emitter's attach point when it is made, and each frame
//! it follows the emitter's posed bone or holding point ([`follow_frame`]).

use ac2_anim::Ac2Transform;
use ac2_assets::{Ac2FxAttach, Ac2FxAttachPoint, Ac2FxObjectAttach};
use ac2_math::Quat;

use crate::{Frame, Position, Quaternion, Vector};

/// An FX-created object's emitter and where on it the object hangs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxFollow {
    pub emitter: u64,
    pub attach: ac2_assets::Ac2FxAttach,
}

/// Where an FX-created object is placed at creation, from its emitter's position
/// (`FXManager::DoCreateObjectFX`): the emitter's own position for a node with no attachment (a
/// water-plane node too: its branch writes a water plane that is never read), its rest-pose
/// attach point composed in for a bone or holding point (`CPhysicsObj::FXSetParent`), and nowhere
/// when the emitter lacks the point (the object is not placed). The drawn place then follows the
/// emitter's posed bone each frame ([`follow_frame`]).
pub fn placed_at(emitter: Option<Position>, attach: Ac2FxObjectAttach) -> Option<Position> {
    match attach {
        Ac2FxObjectAttach::Emitter => emitter,
        Ac2FxObjectAttach::Attached(attach) => emitter.map(|at| attached(&at, attach.rest)),
        Ac2FxObjectAttach::Unplaced => None,
    }
}

/// `CPhysicsObj::UpdateChild` for an FX object hung from its emitter: where it stands this frame,
/// from the emitter's frame and total scale and its posed bones. A holding point takes the root
/// bone it names (`PHYSICSOBJREP::GetHoldingLocation`; a bone the Setup lacks is the identity, as
/// at rest); a bone node takes its named bone (`GetBoneTypeMatrix`). The node's offset is in that
/// point's space and scaled by the emitter, as the emitter's particle systems scale theirs; the
/// rotation is dropped when `m_remove_rotation` says so. `None` when the emitter's skeleton has
/// no such named bone, so the object keeps its last place.
pub fn follow_frame(
    emitter: ac2_math::Frame,
    scale: ac2_math::Vec3,
    bones: &[Ac2Transform],
    named: &[(u32, Ac2Transform)],
    attach: &Ac2FxAttach,
) -> Option<ac2_math::Frame> {
    let base = match attach.point {
        Ac2FxAttachPoint::Holding(index) => {
            bones.get(index).map_or(ac2_math::Frame::IDENTITY, |bone| {
                ac2_math::Frame::new(bone.offset, bone.rotation)
            })
        }
        Ac2FxAttachPoint::Bone(name) => {
            let (_, bone) = named.iter().find(|(bone, _)| *bone == name)?;
            ac2_math::Frame::new(bone.offset, bone.rotation)
        }
    };
    let local = ac2_math::Frame::new(attach.local.offset, attach.local.rotation);
    let mut connection = base.compose_frame(local);
    if attach.remove_rotation {
        connection = ac2_math::Frame::new(connection.origin, ac2_math::Quat::IDENTITY);
    }
    Some(emitter.compose_frame(ac2_math::Frame::new(
        connection.origin * scale,
        connection.orientation,
    )))
}

/// The emitter's frame with `local` (the attach point's offset and rotation, in the emitter's
/// space) composed in.
fn attached(emitter: &Position, local: Ac2Transform) -> Position {
    let frame = emitter.frame;
    let offset = local.offset;
    let global = frame.local_to_global_vec(Vector::new(offset.x, offset.y, offset.z));
    let origin = Vector::new(
        frame.origin.x + global.x,
        frame.origin.y + global.y,
        frame.origin.z + global.z,
    );
    let emitter_rotation = Quat::from_xyzw(frame.q.x, frame.q.y, frame.q.z, frame.q.w);
    let q = (emitter_rotation * local.rotation).normalize();
    Position::new(
        emitter.cell_id,
        Frame::new(origin, Quaternion::new(q.w, q.x, q.y, q.z)),
    )
}

#[cfg(test)]
mod tests {
    use ac2_anim::Ac2Transform;
    use ac2_assets::{Ac2FxAttach, Ac2FxAttachPoint, Ac2FxObjectAttach};
    use ac2_math::{Quat, Vec3};

    use super::{follow_frame, placed_at};
    use crate::{Frame, Position, Quaternion, Vector};

    fn emitter(origin: Vector, q: Quaternion) -> Position {
        Position::new(7, Frame::new(origin, q))
    }

    fn local(offset: Vec3, rotation: Quat) -> Ac2Transform {
        Ac2Transform {
            offset,
            rotation,
            ..Ac2Transform::IDENTITY
        }
    }

    /// An attach to bone `name` whose rest place is `rest` (no point transform, not removed).
    fn bone_attach(name: u32, rest: Ac2Transform) -> Ac2FxAttach {
        Ac2FxAttach {
            point: Ac2FxAttachPoint::Bone(name),
            local: rest,
            remove_rotation: false,
            rest,
        }
    }

    /// `FXManager::DoCreateObjectFX` with no bone and no holding point: the object stands where
    /// its emitter does.
    #[test]
    fn an_object_with_no_attachment_stands_at_its_emitter() {
        let at = emitter(
            Vector::new(1.0, 2.0, 3.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        );
        assert_eq!(placed_at(Some(at), Ac2FxObjectAttach::Emitter), Some(at));
        assert_eq!(placed_at(None, Ac2FxObjectAttach::Emitter), None);
    }

    /// `CPhysicsObj::FXSetParent` on a bone the emitter has: the offset is in the emitter's
    /// turned frame, so a quarter turn about z carries an x offset onto y.
    #[test]
    fn an_attached_object_takes_the_offset_in_the_emitters_frame() {
        let half = std::f32::consts::FRAC_1_SQRT_2;
        let at = emitter(
            Vector::new(1.0, 2.0, 3.0),
            Quaternion::new(half, 0.0, 0.0, half),
        );
        let placed = placed_at(
            Some(at),
            Ac2FxObjectAttach::Attached(bone_attach(
                1,
                local(Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY),
            )),
        )
        .expect("attached");
        assert_eq!(placed.cell_id, 7);
        assert!((placed.frame.origin.x - 1.0).abs() < 1e-5);
        assert!((placed.frame.origin.y - 3.0).abs() < 1e-5);
        assert!((placed.frame.origin.z - 3.0).abs() < 1e-5);
    }

    /// A bone the emitter lacks leaves the object unplaced, so it is never drawn at the origin.
    #[test]
    fn an_object_on_a_missing_bone_is_not_placed() {
        let at = emitter(Vector::ZERO, Quaternion::new(1.0, 0.0, 0.0, 0.0));
        assert_eq!(placed_at(Some(at), Ac2FxObjectAttach::Unplaced), None);
    }

    /// A bone posed at `x`, with no rotation, under the name `name`.
    fn posed(name: u32, x: f32) -> (u32, Ac2Transform) {
        (
            name,
            Ac2Transform {
                offset: Vec3::new(x, 0.0, 0.0),
                ..Ac2Transform::IDENTITY
            },
        )
    }

    /// `CPhysicsObj::UpdateChild`: the child follows its bone as the animation poses it. The same
    /// node moves from one pose to the next, and the emitter's own place and turn carry it.
    #[test]
    fn a_bone_attached_object_follows_the_posed_bone() {
        let emitter = ac2_math::Frame::new(Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY);
        let attach = bone_attach(7, local(Vec3::new(0.0, 0.5, 0.0), Quat::IDENTITY));
        let at_rest = follow_frame(emitter, Vec3::ONE, &[], &[posed(7, 1.0)], &attach)
            .expect("bone 7 is posed");
        let later = follow_frame(emitter, Vec3::ONE, &[], &[posed(7, 3.0)], &attach)
            .expect("bone 7 is posed");
        assert!((at_rest.origin.x - 11.0).abs() < 1e-5);
        assert!((at_rest.origin.y - 0.5).abs() < 1e-5);
        assert!((later.origin.x - 13.0).abs() < 1e-5);
        assert!((later.origin.y - 0.5).abs() < 1e-5);
    }

    /// A holding point takes its root bone's posed transform, then the point's transform and the
    /// node offset; a removed rotation keeps the translation only.
    #[test]
    fn a_holding_point_follows_its_root_bone_and_can_drop_rotation() {
        let half = std::f32::consts::FRAC_1_SQRT_2;
        let turn = Quat::from_xyzw(0.0, 0.0, half, half);
        let bone = Ac2Transform {
            offset: Vec3::new(0.0, 2.0, 0.0),
            rotation: turn,
            ..Ac2Transform::IDENTITY
        };
        let attach = Ac2FxAttach {
            point: Ac2FxAttachPoint::Holding(0),
            local: local(Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY),
            remove_rotation: false,
            rest: Ac2Transform::IDENTITY,
        };
        let emitter = ac2_math::Frame::new(Vec3::ZERO, Quat::IDENTITY);
        let kept = follow_frame(emitter, Vec3::ONE, &[bone], &[], &attach).expect("root bone");
        // The bone turns the x offset onto +y, then lifts it by the bone's offset.
        assert!((kept.origin.x - 0.0).abs() < 1e-5, "{kept:?}");
        assert!((kept.origin.y - 3.0).abs() < 1e-5, "{kept:?}");
        let dropped = follow_frame(
            emitter,
            Vec3::ONE,
            &[bone],
            &[],
            &Ac2FxAttach {
                remove_rotation: true,
                ..attach
            },
        )
        .expect("root bone");
        assert!((dropped.origin.y - 3.0).abs() < 1e-5);
        assert!((dropped.orientation.w - 1.0).abs() < 1e-5, "{dropped:?}");
    }

    /// A named bone the emitter's skeleton lacks leaves the object where it was.
    #[test]
    fn a_missing_named_bone_keeps_the_last_place() {
        let attach = bone_attach(9, Ac2Transform::IDENTITY);
        assert_eq!(
            follow_frame(
                ac2_math::Frame::new(Vec3::ZERO, Quat::IDENTITY),
                Vec3::ONE,
                &[],
                &[posed(7, 1.0)],
                &attach,
            ),
            None
        );
    }
}
