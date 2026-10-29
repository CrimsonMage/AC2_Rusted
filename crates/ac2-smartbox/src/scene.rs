//! `SceneCamera` (Legions `portal/client/src/scenerender/scenecamera.cpp`): the camera's
//! previous and current positions, one physics quantum apart, and the render position slerped
//! between them (`SceneCamera::slerp` by `CPhysics::sRenderInterpolation`).

use ac2_physics::{Frame, Position};

use crate::camera::{CameraManager, CameraSubject};
use crate::targeting::{CameraObjects, NoObjects};

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
    /// The camera is in first person (`CameraManager::GetFirstPersonIsReady`).
    pub first_person: bool,
    /// The camera effects' screen flash (r, g, b, a; `SceneTool::m_cFlashColor`).
    pub flash: [f32; 4],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SceneCamera {
    pub previous: Position,
    pub current: Position,
    pub render: Position,
    pub fov: f32,
}

/// `AFrame::IsValid`: no NaN in the origin or rotation, and a rotation of unit length.
pub fn frame_is_valid(frame: &Frame) -> bool {
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
    /// `SmartBox::UpdateCamera(quantum, _bInterpolate)`: the camera manager's viewer position
    /// becomes the scene camera's current one, the old current its previous. The camera's
    /// line-of-sight pull-in (`CObjCell::LOSCheck` from the player's head to the desired
    /// position) is not ported: the desired position stands.
    pub fn update(
        &mut self,
        camera: &mut CameraManager,
        subject: &CameraSubject,
        quantum: f32,
        interpolate: bool,
    ) {
        self.update_with(camera, subject, &NoObjects, quantum, interpolate);
    }

    /// [`Self::update`] with the objects other than the player the camera can follow.
    pub fn update_with(
        &mut self,
        camera: &mut CameraManager,
        subject: &CameraSubject,
        objects: &dyn CameraObjects,
        quantum: f32,
        interpolate: bool,
    ) {
        let current = self.current;
        let desired = camera.update_camera_with(quantum, Some(subject), objects, &current);
        self.fov = camera.fov();
        self.previous = self.current;
        if !camera.no_ray_trace() {
            self.set_position(desired, !interpolate);
        } else if frame_is_valid(&desired.frame) {
            self.current = desired;
        }
    }

    /// `SmartBox::SetCameraPosition`.
    pub fn set_position(&mut self, position: Position, set_desired: bool) {
        if !frame_is_valid(&position.frame) {
            return;
        }
        self.current = position;
        if set_desired {
            self.previous = position;
        }
    }

    /// The render position, and the point it looks at: the camera distance ahead along its
    /// view axis (the player's head in third person).
    pub fn frame(&self, camera: &CameraManager, scale_z: f32) -> CameraFrame {
        let render = self.render;
        let forward = render.frame.vector_heading();
        let distance = f64::from(camera.targeting.camera_distance) * f64::from(scale_z);
        let o = render.frame.origin;
        let ahead =
            |origin: f32, axis: f32| (f64::from(origin) + f64::from(axis) * distance) as f32;
        let q = render.frame.q;
        CameraFrame {
            cell: render.cell_id,
            origin: [o.x, o.y, o.z],
            rotation: [q.w, q.x, q.y, q.z],
            target: [
                ahead(o.x, forward.x),
                ahead(o.y, forward.y),
                ahead(o.z, forward.z),
            ],
            fov: self.fov,
            first_person: camera.first_person,
            flash: camera.flash_colour(),
        }
    }

    /// `SceneCamera::slerp`.
    pub fn slerp(&mut self, t: f32) {
        let mut render = self.render;
        render.lerp_origin(&self.previous, &self.current, t);
        render
            .frame
            .slerp_rotation(&self.previous.frame, &self.current.frame, t);
        self.render = render;
    }
}
