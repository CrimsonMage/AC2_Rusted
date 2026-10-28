//! The HUD overlays' projection: an object's model-space points through the camera, as
//! `SceneTool::TransformVector` takes them (the camera's view projection after the object's world
//! matrix).

use ac2_math::{Mat4, Vec3};

/// A drawn object's model space to the window's normalised space: `view projection * world`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projector {
    clip: Mat4,
}

impl Projector {
    /// The projection of an object at `world`, seen through the camera's `view_projection`.
    pub fn new(view_projection: Mat4, world: Mat4) -> Self {
        Self {
            clip: view_projection * world,
        }
    }

    /// A model-space point's normalised position (x right, y up) and whether it is in front of
    /// the camera. The clip's w is kept at least 1e-6 in magnitude, as the overlays keep it.
    pub fn project(&self, point: [f32; 3]) -> ([f32; 2], bool) {
        let clip = self.clip * Vec3::from_array(point).extend(1.0);
        let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
        ([clip.x / w, clip.y / w], clip.w > 0.0)
    }

    /// A model-space point's normalised position, when it is in front of the camera.
    pub fn project_front(&self, point: [f32; 3]) -> Option<[f32; 2]> {
        let clip = self.clip * Vec3::from_array(point).extend(1.0);
        (clip.w > 0.0).then(|| [clip.x / clip.w, clip.y / clip.w])
    }
}
