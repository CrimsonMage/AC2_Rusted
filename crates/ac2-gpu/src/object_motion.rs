//! Previous placements for the motion vectors: where each drawn part stood on the previous frame,
//! relative to the previous frame's eye, so the vertex stage can put the same vertex on the
//! previous screen (`object.wgsl`, `taa.wgsl`). The CPU works in f64 (world coordinates are tens
//! of kilometres out) and hands the shader eye-relative numbers, as the current placement does.

use ac2_math::Vec4;

use super::*;

/// An object that moves further than this between two frames was teleported: it has no previous
/// placement (the resolve falls back to the camera's motion), as for a camera cut.
pub const TELEPORT_METRES: f32 = 10.0;

/// Where an instance stood on the previous frame.
#[derive(Debug, Clone, Copy, Default)]
pub enum Previous<'a> {
    /// It did not move: scenery, interiors, water and the like. The previous placement is this
    /// frame's, seen from the previous camera.
    #[default]
    Static,
    /// No record of it (just appeared, a model swap, a teleport): no object motion; the resolve
    /// uses the camera's.
    Unknown,
    /// It stood at `world` with `pose` on the previous frame.
    Moved {
        world: Mat4,
        pose: Option<&'a ObjectPose>,
    },
}

/// The camera of the previous `prepare`.
#[derive(Debug, Clone, Copy)]
pub(super) struct CameraHistory {
    /// The unjittered eye-relative view-projection.
    pub(super) view_proj: Mat4,
    pub(super) eye: Vec3,
    /// The client time, wrapped like the frame uniform's.
    pub(super) time: f32,
}

/// One part's previous placement: its mesh-to-world matrix (translation unused) and its origin
/// relative to the previous eye.
pub(super) struct PartPrevious {
    pub(super) model: Mat4,
    pub(super) rel: Vec4,
}

/// An instance's previous world matrix and pose given its `previous` and its current `world` and
/// `now` pose, or `None` for no usable record.
pub(super) fn previous_placement<'a>(
    previous: Previous<'a>,
    world_now: Mat4,
    now: Option<&'a ObjectPose>,
) -> Option<(Mat4, Option<&'a ObjectPose>)> {
    match previous {
        Previous::Static => Some((world_now, now)),
        Previous::Unknown => None,
        Previous::Moved { world, pose } => {
            let moved = world
                .w_axis
                .truncate()
                .distance(world_now.w_axis.truncate());
            let same_rig = match (pose, now) {
                (None, None) => true,
                (Some(before), Some(now)) => {
                    before.parts.len() == now.parts.len()
                        && before.palette.len() == now.palette.len()
                }
                _ => false,
            };
            (moved <= TELEPORT_METRES && same_rig).then_some((world, pose))
        }
    }
}

/// Whether the previous placement is this frame's: the same world matrix and pose.
pub(super) fn same_placement(
    (world, pose): (Mat4, Option<&ObjectPose>),
    (world_now, now): (Mat4, Option<&ObjectPose>),
) -> bool {
    world == world_now
        && match (pose, now) {
            (None, None) => true,
            (Some(before), Some(now)) => {
                std::ptr::eq(before, now)
                    || (before.parts == now.parts && before.palette == now.palette)
            }
            _ => false,
        }
}

/// The origin of a part that has not moved, relative to the previous eye: `rel` (relative to this
/// frame's eye) plus the eye's step (this frame's minus the previous, taken in f64).
pub(super) fn same_previous_rel(rel: [f32; 4], eye_step: Vec3) -> [f32; 4] {
    [
        rel[0] + eye_step.x,
        rel[1] + eye_step.y,
        rel[2] + eye_step.z,
        1.0,
    ]
}

/// Part `index`'s placement in `world` with `pose` (`rest` when the pose has no matrix for it),
/// its origin taken off `eye` in f64.
pub(super) fn part_previous(
    world: &Mat4,
    pose: Option<&ObjectPose>,
    index: usize,
    rest: Mat4,
    eye: Vec3,
) -> PartPrevious {
    let transform = pose
        .and_then(|pose| pose.parts.get(index))
        .copied()
        .unwrap_or(rest);
    let origin = world.w_axis.truncate().as_dvec3()
        + world
            .transform_vector3(transform.w_axis.truncate())
            .as_dvec3()
        - eye.as_dvec3();
    PartPrevious {
        model: *world * transform,
        rel: origin.as_vec3().extend(1.0),
    }
}

#[cfg(test)]
#[path = "object_motion_tests.rs"]
mod tests;
