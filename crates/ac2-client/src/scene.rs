//! `SceneCamera` (Legions `portal/client/src/scenerender/scenecamera.cpp`): the camera's
//! previous and current positions, one physics quantum apart, and the render position slerped
//! between them (`SceneCamera::slerp` by `CPhysics::sRenderInterpolation`).

use ac2_physics::{Frame, Position};

/// The renderer's camera for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraFrame {
    /// `mRenderPosition`: the cell and the origin relative to its landblock (AC axes), and the
    /// rotation (w, x, y, z) whose local +y is the view direction.
    pub cell: u32,
    pub origin: [f32; 3],
    pub rotation: [f32; 4],
    /// The point the camera looks at, relative to the same landblock: the camera distance
    /// ahead along the view axis, which in third person is the top of the player (plus the
    /// camera height and shoulder offset).
    pub target: [f32; 3],
    /// Vertical field of view, degrees (`CameraManager::m_fov`).
    pub fov: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SceneCamera {
    pub previous: Position,
    pub current: Position,
    pub render: Position,
    pub fov: f32,
}

/// `AFrame::IsValid`: no NaN in the origin or rotation, and a rotation of unit length.
pub(crate) fn frame_is_valid(frame: &Frame) -> bool {
    let o = frame.origin;
    let q = frame.q;
    let finite = [o.x, o.y, o.z, q.w, q.x, q.y, q.z]
        .iter()
        .all(|v| !v.is_nan());
    let (w, x, y, z) = (
        f64::from(q.w),
        f64::from(q.x),
        f64::from(q.y),
        f64::from(q.z),
    );
    let norm = w * w + x * x + y * y + z * z;
    finite && !norm.is_nan() && (norm - 1.0).abs() < f64::from(0.0002_f32)
}

impl SceneCamera {
    /// `SceneCamera::slerp`.
    pub(crate) fn slerp(&mut self, t: f32) {
        let mut render = self.render;
        render.lerp_origin(&self.previous, &self.current, t);
        render
            .frame
            .slerp_rotation(&self.previous.frame, &self.current.frame, t);
        self.render = render;
    }
}
