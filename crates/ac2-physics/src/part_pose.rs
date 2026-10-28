//! The drawn state of a body: where it is drawn (`CPhysics::sRenderInterpolation`) and the
//! animated bones the renderer poses its model with.

use crate::body::PhysicsBody;

/// One Setup bone as the animation left it (`CPartArray::m_boneTransforms`): offset, rotation
/// (w, x, y, z) and scale relative to its parent bone (AC axes). The renderer builds the bone and
/// deform matrices from these (`ac2_render::object::ObjectModel::pose`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartPose {
    pub offset: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl PartPose {
    /// As the `TransformBase` the renderer poses a model with (`ObjectModel::pose`).
    pub fn transform(&self) -> ac2_anim::Ac2Transform {
        let [w, x, y, z] = self.rotation;
        ac2_anim::Ac2Transform {
            offset: self.offset.into(),
            scale: self.scale.into(),
            rotation: ac2_math::Quat::from_xyzw(x, y, z, w),
        }
    }
}

/// Where the body is drawn: `PhysicsBody::drawing_position` at the frame's render interpolation
/// (`CPhysics::sRenderInterpolation`), as the cell and the origin and rotation (w, x, y, z).
pub fn drawn_place(body: &PhysicsBody, render_interpolation: f32) -> (u32, [f32; 3], [f32; 4]) {
    let place = body.drawing_position(render_interpolation);
    let o = place.frame.origin;
    let q = place.frame.q;
    (place.cell_id, [o.x, o.y, o.z], [q.w, q.x, q.y, q.z])
}

/// The body's bones as the renderer poses them: blended at `drawn_at` (the frame's time) when
/// pose blending is on, else the retail refresh (`PhysicsBody::bone_transforms`).
pub fn part_poses(body: &PhysicsBody, drawn_at: Option<f64>) -> Vec<PartPose> {
    let bones = match drawn_at {
        Some(now) => body.blended_bone_transforms(now),
        None => body.bone_transforms(),
    };
    bones
        .iter()
        .map(|t| PartPose {
            offset: [t.offset.x, t.offset.y, t.offset.z],
            rotation: [t.rotation.w, t.rotation.x, t.rotation.y, t.rotation.z],
            scale: [t.scale.x, t.scale.y, t.scale.z],
        })
        .collect()
}
