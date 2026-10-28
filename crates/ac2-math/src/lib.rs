//! Pure math vocabulary and narrow geometry helpers for AC2_Rusted.

mod box2;
mod clip;
mod frame;
mod geometry;
pub mod matrix;
pub mod perlin;
pub mod waveform;

pub use frame::{AFrame, Frame};
pub use geometry::{BBox, Plane, Sphere};
pub use glam::{EulerRot, Mat4, Quat, Vec2, Vec3, Vec4};
pub use waveform::Waveform;

/// Right-handed view and projection matrices; `perspective` maps depth to `[0, 1]`.
pub mod camera {
    pub use glam::camera::rh::proj::directx::perspective;
    pub use glam::camera::rh::view::look_at_mat4;
}
